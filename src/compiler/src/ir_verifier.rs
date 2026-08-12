use crate::ir::{
    BlockMetadata, CheckedIr, EnumSchema, EnumVariantSchema, FunctionMetadata, FunctionSignature,
    Inst, IrMetadata, LogicalType, PlaceId, PlaceMetadata, RawIr, ResultId, Value,
};
use crate::local_reference::admitted_reference_parameter_topology;
use crate::primitive_contract::PrimitiveKind;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;

pub(crate) type PlaceTypeHints = BTreeMap<String, BTreeMap<PlaceId, LogicalType>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct IrVerificationError {
    pub function: String,
    pub block: Option<String>,
    pub kind: IrVerificationErrorKind,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IrVerificationErrorKind {
    DuplicateResultDefinition(ResultId),
    DuplicatePlaceDefinition(PlaceId),
    IdentifierKindCollision(u32),
    UndefinedResultUse(ResultId),
    ResultUseBeforeDefinition(ResultId),
    ResultDoesNotDominateUse(ResultId),
    UndefinedPlaceUse(PlaceId),
    PlaceUseBeforeDefinition(PlaceId),
    PlaceDoesNotDominateUse(PlaceId),
    ExpectedResultIdentifier(&'static str),
    ExpectedPlaceIdentifier(&'static str),
    TypeMismatch {
        operation: &'static str,
        role: &'static str,
        expected: String,
        actual: LogicalType,
    },
    UnknownFunction(String),
    CallArity {
        function: String,
        expected: usize,
        actual: usize,
    },
    MissingCallResult(String),
    VoidCallHasResult(String),
    VoidOperand(&'static str),
    DuplicateLabel(String),
    MissingTarget {
        operation: &'static str,
        label: String,
    },
    MissingTerminator {
        reachable: bool,
    },
    TerminatorNotFinal {
        reachable: bool,
    },
    UnsupportedInstruction(&'static str),
    UnsupportedType(String),
    IntegerOutOfRange(i64),
    InvalidPredicate {
        operation: &'static str,
        predicate: String,
    },
    InvalidSymbol {
        role: &'static str,
        name: String,
    },
    ConstantIntegerDivisionByZero,
    GepElementTypeMismatch {
        expected: String,
        actual: String,
    },
    MetadataMismatch(String),
}

impl IrVerificationError {
    fn new(
        function: impl Into<String>,
        block: Option<&str>,
        kind: IrVerificationErrorKind,
    ) -> Self {
        Self {
            function: function.into(),
            block: block.map(str::to_string),
            kind,
        }
    }
}

impl fmt::Display for IrVerificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IR Verification Error: function `{}`", self.function)?;
        if let Some(block) = &self.block {
            write!(f, ", block `{block}`")?;
        }
        write!(f, ": ")?;
        match &self.kind {
            IrVerificationErrorKind::DuplicateResultDefinition(id) => {
                write!(f, "duplicate result definition for identifier {}", id.0)
            }
            IrVerificationErrorKind::DuplicatePlaceDefinition(id) => {
                write!(f, "duplicate place definition for identifier {}", id.0)
            }
            IrVerificationErrorKind::IdentifierKindCollision(id) => {
                write!(f, "identifier {id} is defined as both a place and a result")
            }
            IrVerificationErrorKind::UndefinedResultUse(id) => {
                write!(f, "undefined result use for identifier {}", id.0)
            }
            IrVerificationErrorKind::ResultUseBeforeDefinition(id) => {
                write!(f, "result {} use occurs before its definition", id.0)
            }
            IrVerificationErrorKind::ResultDoesNotDominateUse(id) => {
                write!(f, "result {} definition does not dominate this use", id.0)
            }
            IrVerificationErrorKind::UndefinedPlaceUse(id) => {
                write!(f, "undefined place use for identifier {}", id.0)
            }
            IrVerificationErrorKind::PlaceUseBeforeDefinition(id) => {
                write!(f, "place {} use occurs before its definition", id.0)
            }
            IrVerificationErrorKind::PlaceDoesNotDominateUse(id) => {
                write!(f, "place {} definition does not dominate this use", id.0)
            }
            IrVerificationErrorKind::ExpectedResultIdentifier(operation) => {
                write!(f, "{operation} result must be a result identifier")
            }
            IrVerificationErrorKind::ExpectedPlaceIdentifier(operation) => {
                write!(f, "{operation} requires a place identifier")
            }
            IrVerificationErrorKind::TypeMismatch {
                operation,
                role,
                expected,
                actual,
            } => write!(
                f,
                "{operation} {role} type mismatch: expected {expected}, found {actual}"
            ),
            IrVerificationErrorKind::UnknownFunction(function) => {
                write!(f, "call targets unknown function `{function}`")
            }
            IrVerificationErrorKind::CallArity {
                function,
                expected,
                actual,
            } => write!(
                f,
                "call to `{function}` has {actual} arguments but its signature requires {expected}"
            ),
            IrVerificationErrorKind::MissingCallResult(function) => {
                write!(
                    f,
                    "call to non-void function `{function}` has result missing"
                )
            }
            IrVerificationErrorKind::VoidCallHasResult(function) => {
                write!(f, "void call to `{function}` must not have a result")
            }
            IrVerificationErrorKind::VoidOperand(operation) => {
                write!(f, "void value cannot be used as an {operation} operand")
            }
            IrVerificationErrorKind::DuplicateLabel(label) => {
                write!(f, "duplicate label definition `{label}`")
            }
            IrVerificationErrorKind::MissingTarget { operation, label } => {
                write!(f, "{operation} target `{label}` is missing")
            }
            IrVerificationErrorKind::MissingTerminator { reachable } => write!(
                f,
                "{} block has terminator missing",
                if *reachable {
                    "reachable"
                } else {
                    "unreachable"
                }
            ),
            IrVerificationErrorKind::TerminatorNotFinal { reachable } => write!(
                f,
                "{} block terminator is not final",
                if *reachable {
                    "reachable"
                } else {
                    "unreachable"
                }
            ),
            IrVerificationErrorKind::UnsupportedInstruction(instruction) => {
                write!(f, "unsupported {instruction} instruction")
            }
            IrVerificationErrorKind::UnsupportedType(ty) => {
                write!(f, "unsupported logical type `{ty}`")
            }
            IrVerificationErrorKind::IntegerOutOfRange(value) => {
                write!(
                    f,
                    "integer immediate {value} is outside the admitted i32 range"
                )
            }
            IrVerificationErrorKind::InvalidPredicate {
                operation,
                predicate,
            } => write!(f, "{operation} predicate `{predicate}` is not admitted"),
            IrVerificationErrorKind::InvalidSymbol { role, name } => {
                write!(
                    f,
                    "{role} symbol `{name}` is not admitted for LLVM emission"
                )
            }
            IrVerificationErrorKind::ConstantIntegerDivisionByZero => {
                write!(f, "constant integer division by zero is not admitted")
            }
            IrVerificationErrorKind::GepElementTypeMismatch { expected, actual } => write!(
                f,
                "getelementptr element type mismatch: base uses `{expected}`, instruction uses `{actual}`"
            ),
            IrVerificationErrorKind::MetadataMismatch(message) => {
                write!(f, "checked logical metadata mismatch: {message}")
            }
        }
    }
}

impl Error for IrVerificationError {}

#[derive(Debug, Clone)]
struct Body<'a> {
    name: String,
    instructions: Vec<&'a Inst>,
    signature: FunctionSignature,
}

#[derive(Debug)]
struct Block<'a> {
    label: String,
    instructions: Vec<(usize, &'a Inst)>,
    successors: Vec<String>,
    reachable: bool,
}

#[derive(Debug, Clone)]
enum PlaceType {
    Known(LogicalType),
    Numeric,
    Array {
        logical_element: Option<LogicalType>,
        physical_element: String,
        count: usize,
        checked_copy_data: bool,
    },
}

impl PlaceType {
    fn logical(&self) -> Option<LogicalType> {
        match self {
            Self::Known(ty) => Some(ty.clone()),
            Self::Numeric => None,
            Self::Array {
                logical_element,
                count,
                ..
            } => logical_element.clone().map(|element| LogicalType::Array {
                element: Box::new(element),
                count: *count,
            }),
        }
    }

    fn checked_copy_array_contract(&self) -> Option<(LogicalType, usize)> {
        match self {
            Self::Known(LogicalType::Array { element, count }) if valid_copy_data_type(element) => {
                Some((element.as_ref().clone(), *count))
            }
            Self::Array {
                logical_element: Some(element),
                count,
                checked_copy_data: true,
                ..
            } => Some((element.clone(), *count)),
            Self::Known(_)
            | Self::Numeric
            | Self::Array {
                logical_element: None,
                ..
            }
            | Self::Array {
                checked_copy_data: false,
                ..
            } => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Definition {
    block: usize,
    position: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EnumOwner {
    Result(ResultId),
    Place(PlaceId),
}

impl fmt::Display for EnumOwner {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Result(id) => write!(formatter, "result {}", id.0),
            Self::Place(id) => write!(formatter, "place {}", id.0),
        }
    }
}

fn logical_type(type_name: &str) -> Option<LogicalType> {
    match type_name {
        "int" | "i32" => Some(LogicalType::Int),
        "float" | "f64" | "double" => Some(LogicalType::Float),
        "bool" | "i1" => Some(LogicalType::Bool),
        "char" => Some(LogicalType::Char),
        "string" | "str" => Some(LogicalType::String),
        "void" => Some(LogicalType::Void),
        _ => None,
    }
}

fn physical_copy_type_hint(logical_type: &LogicalType) -> String {
    match logical_type {
        ty if PrimitiveKind::from_logical_type(ty).is_some() => {
            PrimitiveKind::from_logical_type(ty)
                .unwrap()
                .copy_data_llvm_type()
                .to_string()
        }
        LogicalType::Int | LogicalType::Float | LogicalType::Bool | LogicalType::Char => {
            unreachable!("primitive logical types matched above")
        }
        LogicalType::Array { element, count } => {
            format!("[{count} x {}]", physical_copy_type_hint(element))
        }
        LogicalType::Struct { name, .. } => format!("%aero.struct.{name}"),
        LogicalType::Tuple { elements } => format!(
            "{{ {} }}",
            elements
                .iter()
                .map(physical_copy_type_hint)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        LogicalType::EnumFields { fields } => format!(
            "{{ {} }}",
            fields
                .iter()
                .map(physical_copy_type_hint)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        LogicalType::Void
        | LogicalType::String
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. }
        | LogicalType::Enum { .. } => logical_type.to_string(),
    }
}

fn valid_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn valid_enum_symbol(name: &str) -> bool {
    crate::generic_enum_contract::valid_generic_aware_enum_symbol(name, |name| {
        crate::builtin_carrier_contract::valid_carrier_aware_enum_symbol(name, valid_symbol)
    })
}

fn valid_struct_symbol(name: &str) -> bool {
    crate::generic_struct_contract::valid_generic_aware_struct_symbol(name, valid_symbol)
}

fn valid_function_symbol(name: &str) -> bool {
    crate::generic_function_contract::valid_generic_aware_function_symbol(name, valid_symbol)
}

fn valid_immutable_reference_pointee(ty: &LogicalType) -> bool {
    valid_owned_place_type(ty)
}

fn valid_mutable_reference_pointee(ty: &LogicalType) -> bool {
    valid_owned_place_type(ty)
}

fn valid_owned_place_type(ty: &LogicalType) -> bool {
    valid_copy_data_type(ty)
        || matches!(
            ty,
            LogicalType::Enum { name, variants }
                if valid_enum_schema(&EnumSchema {
                    name: name.clone(),
                    variants: variants.clone(),
                })
        )
}

fn valid_enum_payload_type(payload: &LogicalType) -> bool {
    match payload {
        LogicalType::EnumFields { fields } => {
            fields.len() >= 2 && fields.iter().all(valid_copy_data_type)
        }
        _ => valid_copy_data_type(payload),
    }
}

fn valid_enum_schema(schema: &EnumSchema) -> bool {
    let mut unique = BTreeSet::new();
    valid_enum_symbol(&schema.name)
        && crate::generic_enum_contract::valid_generic_enum_schema(&schema.name, &schema.variants)
        && !schema.variants.is_empty()
        && schema.variants.iter().all(|variant| {
            valid_symbol(&variant.name)
                && unique.insert(&variant.name)
                && variant.payload.as_ref().is_none_or(valid_enum_payload_type)
        })
}

fn valid_struct_schema(fields: &[LogicalType]) -> bool {
    !fields.is_empty() && fields.iter().all(valid_copy_data_type)
}

fn valid_named_struct_schema(name: &str, fields: &[LogicalType]) -> bool {
    valid_struct_symbol(name)
        && valid_struct_schema(fields)
        && crate::generic_struct_contract::valid_generic_struct_schema(name, fields)
}

fn valid_copy_data_type(logical_type: &LogicalType) -> bool {
    if PrimitiveKind::from_logical_type(logical_type).is_some() {
        return true;
    }
    match logical_type {
        LogicalType::Array { element, .. } => valid_copy_data_type(element),
        LogicalType::Tuple { elements } => {
            elements.len() >= 2 && elements.iter().all(valid_copy_data_type)
        }
        LogicalType::Struct { name, fields } => valid_named_struct_schema(name, fields),
        LogicalType::Int
        | LogicalType::Float
        | LogicalType::Bool
        | LogicalType::Char
        | LogicalType::Void
        | LogicalType::String
        | LogicalType::EnumFields { .. }
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. }
        | LogicalType::Enum { .. } => false,
    }
}

fn valid_checked_transport_type(logical_type: &LogicalType) -> bool {
    if valid_copy_data_type(logical_type) {
        return true;
    }
    match logical_type {
        LogicalType::Enum { name, variants } => valid_enum_schema(&EnumSchema {
            name: name.clone(),
            variants: variants.clone(),
        }),
        LogicalType::Int
        | LogicalType::Float
        | LogicalType::Bool
        | LogicalType::Char
        | LogicalType::Array { .. }
        | LogicalType::Struct { .. }
        | LogicalType::Tuple { .. }
        | LogicalType::EnumFields { .. }
        | LogicalType::Void
        | LogicalType::String
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. } => false,
    }
}

fn valid_checked_parameter_type(logical_type: &LogicalType) -> bool {
    valid_checked_transport_type(logical_type)
        || matches!(
            logical_type,
            LogicalType::ImmutableReference { pointee }
                if valid_immutable_reference_pointee(pointee)
        )
        || matches!(
            logical_type,
            LogicalType::MutableReference { pointee }
                if valid_mutable_reference_pointee(pointee)
        )
}

fn collides_with_generated_local(name: &str) -> bool {
    ["reg", "ptr"].into_iter().any(|prefix| {
        name.strip_prefix(prefix).is_some_and(|suffix| {
            !suffix.is_empty() && suffix.chars().all(|ch| ch.is_ascii_digit())
        })
    })
}

fn signature(
    function: &str,
    parameters: &[(String, String)],
    return_type: &Option<String>,
) -> Result<FunctionSignature, IrVerificationError> {
    if !valid_function_symbol(function) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::InvalidSymbol {
                role: "function",
                name: function.to_string(),
            },
        ));
    }
    let mut typed_parameters = Vec::with_capacity(parameters.len());
    let mut parameter_names = BTreeSet::new();
    for (name, ty) in parameters {
        if !valid_symbol(name) {
            return Err(IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::InvalidSymbol {
                    role: "parameter",
                    name: name.clone(),
                },
            ));
        }
        if !parameter_names.insert(name.as_str()) {
            return Err(IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "function signature defines duplicate parameter `{name}`"
                )),
            ));
        }
        let Some(ty) = logical_type(ty) else {
            return Err(IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::UnsupportedType(ty.clone()),
            ));
        };
        if matches!(ty, LogicalType::Void | LogicalType::String) {
            return Err(IrVerificationError::new(
                function,
                None,
                if ty == LogicalType::Void {
                    IrVerificationErrorKind::VoidOperand("parameter")
                } else {
                    IrVerificationErrorKind::UnsupportedType("string parameter".to_string())
                },
            ));
        }
        typed_parameters.push((name.clone(), ty));
    }
    let result = match return_type {
        Some(ty) => logical_type(ty).ok_or_else(|| {
            IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::UnsupportedType(ty.clone()),
            )
        })?,
        None if function == "main" => LogicalType::Int,
        None => LogicalType::Void,
    };
    if result == LogicalType::String {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::UnsupportedType("string return".to_string()),
        ));
    }
    if !crate::generic_function_contract::valid_private_generic_function_signature(
        function,
        &typed_parameters
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect::<Vec<_>>(),
        &result,
    ) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "private generic-function identity does not match its checked signature"
                    .to_string(),
            ),
        ));
    }
    if function == "main" && (!typed_parameters.is_empty() || result != LogicalType::Int) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "process entry must have exact signature `i32 @main()`".to_string(),
            ),
        ));
    }
    Ok(FunctionSignature {
        parameters: typed_parameters,
        result,
    })
}

fn checked_signature(
    function: &str,
    parameters: &[(String, LogicalType)],
    result: &LogicalType,
) -> Result<FunctionSignature, IrVerificationError> {
    if !valid_function_symbol(function) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::InvalidSymbol {
                role: "function",
                name: function.to_string(),
            },
        ));
    }
    let mut parameter_names = BTreeSet::new();
    for (name, ty) in parameters {
        if !valid_symbol(name) {
            return Err(IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::InvalidSymbol {
                    role: "parameter",
                    name: name.clone(),
                },
            ));
        }
        if !parameter_names.insert(name.as_str()) {
            return Err(IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "function signature defines duplicate parameter `{name}`"
                )),
            ));
        }
        if !valid_checked_parameter_type(ty) {
            return Err(IrVerificationError::new(
                function,
                None,
                IrVerificationErrorKind::UnsupportedType(format!(
                    "checked function parameter {ty}"
                )),
            ));
        }
    }
    if *result != LogicalType::Void && !valid_checked_transport_type(result) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::UnsupportedType(format!("checked function return {result}")),
        ));
    }
    if !crate::generic_function_contract::valid_private_generic_function_signature(
        function,
        &parameters
            .iter()
            .map(|(_, ty)| ty.clone())
            .collect::<Vec<_>>(),
        result,
    ) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "private generic-function identity does not match its checked signature"
                    .to_string(),
            ),
        ));
    }
    let reference_parameters = parameters
        .iter()
        .filter(|(_, ty)| {
            matches!(
                ty,
                LogicalType::ImmutableReference { .. } | LogicalType::MutableReference { .. }
            )
        })
        .count();
    let mutable_parameters = parameters
        .iter()
        .filter(|(_, ty)| matches!(ty, LogicalType::MutableReference { .. }))
        .count();
    if !admitted_reference_parameter_topology(reference_parameters, mutable_parameters) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "checked reference transport supports at most one mutable-reference parameter; simultaneous mutable-reference parameters are not supported".to_string(),
            ),
        ));
    }
    let mentions_checked_transport = parameters.iter().any(|(_, ty)| {
        matches!(
            ty,
            LogicalType::Struct { .. }
                | LogicalType::Array { .. }
                | LogicalType::Tuple { .. }
                | LogicalType::Enum { .. }
                | LogicalType::ImmutableReference { .. }
                | LogicalType::MutableReference { .. }
        )
    }) || matches!(
        result,
        LogicalType::Struct { .. }
            | LogicalType::Array { .. }
            | LogicalType::Tuple { .. }
            | LogicalType::Enum { .. }
    );
    if !mentions_checked_transport {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "checked function definition requires an aggregate-, enum-, or scalar-reference-bearing signature"
                    .to_string(),
            ),
        ));
    }
    if function == "main" && (!parameters.is_empty() || *result != LogicalType::Int) {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "process entry must have exact signature `i32 @main()`".to_string(),
            ),
        ));
    }
    Ok(FunctionSignature {
        parameters: parameters.to_vec(),
        result: result.clone(),
    })
}

fn collect_bodies<'a>(
    ir: &'a RawIr,
) -> Result<(Vec<Body<'a>>, BTreeMap<String, FunctionSignature>), IrVerificationError> {
    let mut signatures = BTreeMap::new();
    let mut definitions: BTreeMap<String, (&'a Vec<Inst>, FunctionSignature)> = BTreeMap::new();
    let mut functions = ir.iter().collect::<Vec<_>>();
    functions.sort_by(|(left, _), (right, _)| left.cmp(right));

    for (map_key, function) in &functions {
        if map_key.as_str() != function.name {
            return Err(IrVerificationError::new(
                &function.name,
                None,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "function map key `{map_key}` disagrees with body name `{}`",
                    function.name
                )),
            ));
        }
        if !valid_function_symbol(&function.name) {
            return Err(IrVerificationError::new(
                &function.name,
                None,
                IrVerificationErrorKind::InvalidSymbol {
                    role: "function",
                    name: function.name.clone(),
                },
            ));
        }
        if function.name == "printf" {
            return Err(IrVerificationError::new(
                &function.name,
                None,
                IrVerificationErrorKind::MetadataMismatch(
                    "`printf` is reserved by the checked runtime ABI".to_string(),
                ),
            ));
        }
        for instruction in &function.body {
            let definition = match instruction {
                Inst::FunctionDef {
                    name,
                    parameters,
                    return_type,
                    body,
                } => Some((name, body, signature(name, parameters, return_type)?)),
                Inst::CheckedFunctionDef {
                    name,
                    parameters,
                    result,
                    body,
                } => Some((name, body, checked_signature(name, parameters, result)?)),
                _ => None,
            };
            if let Some((name, body, sig)) = definition {
                if definitions
                    .insert(name.clone(), (body, sig.clone()))
                    .is_some()
                {
                    return Err(IrVerificationError::new(
                        name,
                        None,
                        IrVerificationErrorKind::DuplicateResultDefinition(ResultId(0)),
                    ));
                }
                signatures.insert(name.clone(), sig);
            }
        }
    }

    if !ir.contains_key("main") {
        return Err(IrVerificationError::new(
            "<module>",
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "process entry `main` is missing; checked modules require exact `i32 @main()`"
                    .to_string(),
            ),
        ));
    }

    for (_, function) in &functions {
        signatures
            .entry(function.name.clone())
            .or_insert_with(|| FunctionSignature {
                parameters: Vec::new(),
                result: LogicalType::Int,
            });
    }

    let mut bodies = Vec::new();
    for (name, (body, sig)) in &definitions {
        if !ir.contains_key(name) {
            return Err(IrVerificationError::new(
                name,
                None,
                IrVerificationErrorKind::MetadataMismatch(
                    "function definition has no matching module entry".to_string(),
                ),
            ));
        }
        bodies.push(Body {
            name: name.clone(),
            instructions: body.iter().collect(),
            signature: sig.clone(),
        });
    }
    for (_, function) in functions {
        let runtime = function
            .body
            .iter()
            .filter(|instruction| {
                !matches!(
                    instruction,
                    Inst::FunctionDef { .. } | Inst::CheckedFunctionDef { .. }
                )
            })
            .collect::<Vec<_>>();
        if definitions.contains_key(&function.name) {
            if !runtime.is_empty() {
                return Err(IrVerificationError::new(
                    &function.name,
                    None,
                    IrVerificationErrorKind::MetadataMismatch(
                        "module entry mixes an emitted FunctionDef with ignored runtime instructions"
                            .to_string(),
                    ),
                ));
            }
            continue;
        }
        bodies.push(Body {
            name: function.name.clone(),
            instructions: runtime,
            signature: signatures[&function.name].clone(),
        });
    }
    bodies.sort_by(|left, right| left.name.cmp(&right.name));
    Ok((bodies, signatures))
}

fn is_terminator(instruction: &Inst) -> bool {
    matches!(
        instruction,
        Inst::Return(_) | Inst::Jump(_) | Inst::Branch { .. } | Inst::CheckedEnumDispatch { .. }
    )
}

fn unsupported_name(instruction: &Inst) -> Option<&'static str> {
    match instruction {
        Inst::FunctionDef { .. } | Inst::CheckedFunctionDef { .. } => {
            Some("nested function definition")
        }
        Inst::AllocaStruct { .. } => Some("alloca struct"),
        Inst::GetFieldPtr { .. } => Some("field pointer"),
        Inst::VecAlloca { .. } => Some("vec alloca"),
        Inst::VecPush { .. } => Some("vec push"),
        Inst::VecPop { .. } => Some("vec pop"),
        Inst::VecLength { .. } => Some("vec length"),
        Inst::VecCapacity { .. } => Some("vec capacity"),
        Inst::VecAccess { .. } => Some("vec access"),
        Inst::VecInit { .. } => Some("vec init"),
        Inst::ArrayLength { .. } => Some("array length"),
        Inst::ArrayAccess { .. } => Some("array access"),
        Inst::EnumDiscriminant { .. } => Some("enum discriminant"),
        Inst::EnumVariantData { .. } => Some("enum variant data"),
        Inst::EnumConstruct { .. } => Some("enum construct"),
        _ => None,
    }
}

fn split_blocks<'a>(body: &Body<'a>) -> Result<Vec<Block<'a>>, IrVerificationError> {
    let mut blocks = vec![Block {
        label: "entry".to_string(),
        instructions: Vec::new(),
        successors: Vec::new(),
        reachable: false,
    }];
    let mut labels = HashSet::new();
    labels.insert("entry".to_string());

    for (position, instruction) in body.instructions.iter().enumerate() {
        if let Inst::Label(label) = instruction {
            if !valid_symbol(label) || collides_with_generated_local(label) {
                return Err(IrVerificationError::new(
                    &body.name,
                    Some(label),
                    IrVerificationErrorKind::InvalidSymbol {
                        role: "block label",
                        name: label.clone(),
                    },
                ));
            }
            if !labels.insert(label.clone()) {
                return Err(IrVerificationError::new(
                    &body.name,
                    Some(label),
                    IrVerificationErrorKind::DuplicateLabel(label.clone()),
                ));
            }
            blocks.push(Block {
                label: label.clone(),
                instructions: Vec::new(),
                successors: Vec::new(),
                reachable: false,
            });
        } else {
            blocks
                .last_mut()
                .expect("entry block")
                .instructions
                .push((position, instruction));
        }
    }

    let label_set = blocks
        .iter()
        .map(|block| block.label.clone())
        .collect::<HashSet<_>>();
    for block in &mut blocks {
        if let Some((_, instruction)) = block.instructions.last() {
            block.successors = match instruction {
                Inst::Jump(label) => vec![label.clone()],
                Inst::Branch {
                    true_label,
                    false_label,
                    ..
                } => vec![true_label.clone(), false_label.clone()],
                Inst::CheckedEnumDispatch { targets, .. } => targets.clone(),
                _ => Vec::new(),
            };
        }
        for label in &block.successors {
            if !label_set.contains(label) {
                let operation = match block.instructions.last() {
                    Some((_, Inst::Jump(_))) => "jump",
                    Some((_, Inst::CheckedEnumDispatch { .. })) => "checked enum dispatch",
                    _ => "branch",
                };
                return Err(IrVerificationError::new(
                    &body.name,
                    Some(&block.label),
                    IrVerificationErrorKind::MissingTarget {
                        operation,
                        label: label.clone(),
                    },
                ));
            }
        }
    }

    let by_label = blocks
        .iter()
        .enumerate()
        .map(|(index, block)| (block.label.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut queue = VecDeque::from([0]);
    while let Some(index) = queue.pop_front() {
        if blocks[index].reachable {
            continue;
        }
        blocks[index].reachable = true;
        let successors = blocks[index].successors.clone();
        for successor in successors {
            queue.push_back(by_label[&successor]);
        }
    }

    for block in &blocks {
        let terminator_index = block
            .instructions
            .iter()
            .position(|(_, instruction)| is_terminator(instruction));
        match terminator_index {
            None => {
                return Err(IrVerificationError::new(
                    &body.name,
                    Some(&block.label),
                    IrVerificationErrorKind::MissingTerminator {
                        reachable: block.reachable,
                    },
                ));
            }
            Some(index) if index + 1 != block.instructions.len() => {
                return Err(IrVerificationError::new(
                    &body.name,
                    Some(&block.label),
                    IrVerificationErrorKind::TerminatorNotFinal {
                        reachable: block.reachable,
                    },
                ));
            }
            Some(_) => {}
        }
    }
    Ok(blocks)
}

fn reg(value: &Value) -> Option<u32> {
    match value {
        Value::Reg(id) => Some(*id),
        _ => None,
    }
}

fn result_definition(instruction: &Inst) -> Option<&Value> {
    match instruction {
        Inst::Add(result, ..)
        | Inst::FAdd(result, ..)
        | Inst::Sub(result, ..)
        | Inst::FSub(result, ..)
        | Inst::Mul(result, ..)
        | Inst::FMul(result, ..)
        | Inst::Div(result, ..)
        | Inst::FDiv(result, ..)
        | Inst::Load(result, _)
        | Inst::SIToFP(result, _)
        | Inst::FPToSI(result, _)
        | Inst::CheckedEnumParameter { result, .. }
        | Inst::CheckedEnumVariant { result, .. }
        | Inst::CheckedEnumVariantFields { result, .. }
        | Inst::CheckedEnumPayload { result, .. }
        | Inst::CheckedEnumField { result, .. }
        | Inst::CheckedImmutableEnumMatchRead { result, .. }
        | Inst::CheckedMutableEnumMatchRead { result, .. } => Some(result),
        Inst::Call { result, .. } => result.as_ref(),
        Inst::ICmp { result, .. }
        | Inst::FCmp { result, .. }
        | Inst::And { result, .. }
        | Inst::Or { result, .. }
        | Inst::Not { result, .. }
        | Inst::Neg { result, .. } => Some(result),
        _ => None,
    }
}

fn place_definition(instruction: &Inst) -> Option<&Value> {
    match instruction {
        Inst::Alloca(result, _)
        | Inst::CheckedMutableOwnedPlaceAlloca { result, .. }
        | Inst::CheckedImmutableEnumOwnerPlaceAlloca { result, .. }
        | Inst::CheckedMatchResultPlaceAlloca { result, .. }
        | Inst::AllocaArray { result, .. }
        | Inst::GetElementPtr { result, .. }
        | Inst::CheckedCopyStructArrayAlloca { result, .. }
        | Inst::CheckedCopyStructArrayElementPtr { result, .. }
        | Inst::CheckedStructAlloca { result, .. }
        | Inst::CheckedStructFieldPtr { result, .. }
        | Inst::CheckedTupleAlloca { result, .. }
        | Inst::CheckedTupleFieldPtr { result, .. }
        | Inst::CheckedImmutableBorrow { result, .. }
        | Inst::CheckedMutableBorrow { result, .. }
        | Inst::CheckedProjectedBorrow { result, .. }
        | Inst::CheckedImmutableReferenceParameter { result, .. }
        | Inst::CheckedMutableReferenceParameter { result, .. } => Some(result),
        _ => None,
    }
}

fn definition_type(
    instruction: &Inst,
    signatures: &BTreeMap<String, FunctionSignature>,
    places: &BTreeMap<PlaceId, PlaceType>,
    results: &BTreeMap<ResultId, LogicalType>,
) -> Option<LogicalType> {
    match instruction {
        Inst::Add(..) | Inst::Sub(..) | Inst::Mul(..) | Inst::Div(..) | Inst::FPToSI(..) => {
            Some(LogicalType::Int)
        }
        Inst::FAdd(..) | Inst::FSub(..) | Inst::FMul(..) | Inst::FDiv(..) | Inst::SIToFP(..) => {
            Some(LogicalType::Float)
        }
        Inst::ICmp { .. }
        | Inst::FCmp { .. }
        | Inst::And { .. }
        | Inst::Or { .. }
        | Inst::Not { .. } => Some(LogicalType::Bool),
        Inst::Neg { operand, .. } => match operand {
            Value::ImmInt(_) => Some(LogicalType::Int),
            Value::ImmFloat(_) => Some(LogicalType::Float),
            Value::ImmChar(_) => Some(LogicalType::Char),
            Value::Reg(id) => results.get(&ResultId(*id)).cloned(),
            Value::ImmString(_) => None,
        },
        Inst::Load(_, place) => reg(place)
            .and_then(|id| places.get(&PlaceId(id)))
            .and_then(PlaceType::logical),
        Inst::Call { function, .. } => signatures.get(function).map(|sig| sig.result.clone()),
        Inst::CheckedEnumVariant { schema, .. }
        | Inst::CheckedEnumVariantFields { schema, .. }
        | Inst::CheckedEnumParameter { schema, .. }
        | Inst::CheckedImmutableEnumMatchRead { schema, .. }
        | Inst::CheckedMutableEnumMatchRead { schema, .. } => Some(schema.logical_type()),
        Inst::CheckedEnumPayload {
            schema,
            variant_index,
            ..
        } => schema
            .variants
            .get(*variant_index)
            .and_then(|variant| variant.payload.clone()),
        Inst::CheckedEnumField {
            schema,
            variant_index,
            field_index,
            ..
        } => schema
            .variants
            .get(*variant_index)
            .and_then(|variant| variant.payload.as_ref())
            .and_then(|payload| match payload {
                LogicalType::EnumFields { fields } => fields.get(*field_index).cloned(),
                _ => None,
            }),
        _ => None,
    }
}

fn predecessors(blocks: &[Block<'_>]) -> Vec<BTreeSet<usize>> {
    let labels = blocks
        .iter()
        .enumerate()
        .map(|(index, block)| (block.label.as_str(), index))
        .collect::<HashMap<_, _>>();
    let mut result = vec![BTreeSet::new(); blocks.len()];
    for (index, block) in blocks.iter().enumerate() {
        for successor in &block.successors {
            result[labels[successor.as_str()]].insert(index);
        }
    }
    result
}

fn dominators(blocks: &[Block<'_>]) -> Vec<BTreeSet<usize>> {
    let predecessors = predecessors(blocks);
    let reachable = blocks
        .iter()
        .enumerate()
        .filter_map(|(index, block)| block.reachable.then_some(index))
        .collect::<BTreeSet<_>>();
    let mut dom = vec![BTreeSet::new(); blocks.len()];
    dom[0].insert(0);
    for &index in reachable.iter().skip(1) {
        dom[index] = reachable.clone();
    }
    let mut changed = true;
    while changed {
        changed = false;
        for &index in reachable.iter().skip(1) {
            let incoming = predecessors[index]
                .iter()
                .filter(|predecessor| blocks[**predecessor].reachable)
                .copied()
                .collect::<Vec<_>>();
            let mut next = if let Some((first, rest)) = incoming.split_first() {
                rest.iter()
                    .fold(dom[*first].clone(), |intersection, predecessor| {
                        intersection
                            .intersection(&dom[*predecessor])
                            .copied()
                            .collect()
                    })
            } else {
                BTreeSet::new()
            };
            next.insert(index);
            if next != dom[index] {
                dom[index] = next;
                changed = true;
            }
        }
    }
    for (index, block) in blocks.iter().enumerate() {
        if !block.reachable {
            dom[index].insert(index);
        }
    }
    dom
}

struct FunctionVerifier<'a> {
    body: &'a Body<'a>,
    blocks: Vec<Block<'a>>,
    signatures: &'a BTreeMap<String, FunctionSignature>,
    definitions: BTreeMap<ResultId, Definition>,
    place_definitions: BTreeMap<PlaceId, Definition>,
    result_types: BTreeMap<ResultId, LogicalType>,
    places: BTreeMap<PlaceId, PlaceType>,
    place_names: BTreeMap<PlaceId, Option<String>>,
    element_owners: BTreeMap<PlaceId, PlaceId>,
    projection_parents: BTreeMap<PlaceId, PlaceId>,
    immutable_enum_owner_places: BTreeSet<PlaceId>,
    mutable_owned_places: BTreeSet<PlaceId>,
    projected_owned_roots: BTreeSet<PlaceId>,
    match_result_places: BTreeSet<PlaceId>,
    immutable_enum_reference_origins: BTreeMap<PlaceId, PlaceId>,
    immutable_reference_origins: BTreeMap<PlaceId, PlaceId>,
    mutable_reference_origins: BTreeMap<PlaceId, PlaceId>,
    projected_reference_roots: BTreeMap<PlaceId, (PlaceId, PlaceId, bool)>,
    mutable_reference_parameters: BTreeSet<PlaceId>,
    dominators: Vec<BTreeSet<usize>>,
    infer_bool_places: bool,
}

impl<'a> FunctionVerifier<'a> {
    fn new(
        body: &'a Body<'a>,
        signatures: &'a BTreeMap<String, FunctionSignature>,
        seed: Option<&FunctionMetadata>,
        place_hints: Option<&BTreeMap<PlaceId, LogicalType>>,
        infer_bool_places: bool,
    ) -> Result<Self, IrVerificationError> {
        let blocks = split_blocks(body)?;
        let dominators = dominators(&blocks);
        let mut verifier = Self {
            body,
            blocks,
            signatures,
            definitions: BTreeMap::new(),
            place_definitions: BTreeMap::new(),
            result_types: seed.map_or_else(BTreeMap::new, |metadata| metadata.results.clone()),
            places: BTreeMap::new(),
            place_names: BTreeMap::new(),
            element_owners: BTreeMap::new(),
            projection_parents: BTreeMap::new(),
            immutable_enum_owner_places: BTreeSet::new(),
            mutable_owned_places: BTreeSet::new(),
            projected_owned_roots: BTreeSet::new(),
            match_result_places: BTreeSet::new(),
            immutable_enum_reference_origins: BTreeMap::new(),
            immutable_reference_origins: BTreeMap::new(),
            mutable_reference_origins: BTreeMap::new(),
            projected_reference_roots: BTreeMap::new(),
            mutable_reference_parameters: BTreeSet::new(),
            dominators,
            infer_bool_places,
        };
        if let Some(metadata) = seed {
            for (id, place) in &metadata.places {
                let place_type = match &place.pointee {
                    LogicalType::Array { element, count } => PlaceType::Array {
                        logical_element: Some((**element).clone()),
                        physical_element: physical_copy_type_hint(element),
                        count: *count,
                        checked_copy_data: valid_copy_data_type(element),
                    },
                    ty => PlaceType::Known(ty.clone()),
                };
                verifier.places.insert(*id, place_type);
                verifier.place_names.insert(*id, place.name.clone());
            }
        }
        verifier.collect_definitions(seed.is_some(), place_hints)?;
        verifier.resolve_order_independent_types()?;
        Ok(verifier)
    }

    fn error(&self, block: usize, kind: IrVerificationErrorKind) -> IrVerificationError {
        IrVerificationError::new(&self.body.name, Some(&self.blocks[block].label), kind)
    }

    fn enum_place(&self, value: &Value) -> Option<PlaceId> {
        let place = PlaceId(reg(value)?);
        self.places
            .get(&place)
            .and_then(PlaceType::logical)
            .is_some_and(|ty| matches!(ty, LogicalType::Enum { .. }))
            .then_some(place)
    }

    fn enum_owner(&self, value: &Value) -> Option<EnumOwner> {
        let result = ResultId(reg(value)?);
        if !matches!(
            self.result_types.get(&result),
            Some(LogicalType::Enum { .. })
        ) {
            return None;
        }
        let definition = self.definitions.get(&result)?;
        match self.body.instructions[definition.position] {
            Inst::Load(_, source) => self
                .enum_place(source)
                .map(EnumOwner::Place)
                .or(Some(EnumOwner::Result(result))),
            _ => Some(EnumOwner::Result(result)),
        }
    }

    fn mutable_reference_root(&self, place: PlaceId) -> Option<PlaceId> {
        let mut current = place;
        let mut seen = BTreeSet::new();
        while seen.insert(current) {
            if self.mutable_reference_parameters.contains(&current) {
                return None;
            }
            let Some(parent) = self.mutable_reference_origins.get(&current).copied() else {
                return Some(current);
            };
            current = parent;
        }
        None
    }

    fn mutable_reference_root_identity(&self, place: PlaceId) -> Option<PlaceId> {
        if let Some((root, _, true)) = self.projected_reference_roots.get(&place) {
            return Some(*root);
        }
        let mut current = place;
        let mut seen = BTreeSet::new();
        while seen.insert(current) {
            if self.mutable_reference_parameters.contains(&current) {
                return Some(current);
            }
            let Some(parent) = self.mutable_reference_origins.get(&current).copied() else {
                return Some(current);
            };
            current = parent;
        }
        None
    }

    fn immutable_reference_place(&self, place: PlaceId) -> bool {
        let Some(definition) = self.place_definitions.get(&place) else {
            return false;
        };
        match self.body.instructions[definition.position] {
            Inst::CheckedImmutableBorrow { .. }
            | Inst::CheckedImmutableReferenceParameter { .. } => true,
            Inst::CheckedProjectedBorrow { mutable, .. } => !mutable,
            _ => false,
        }
    }

    fn immutable_reference_root_identity(&self, place: PlaceId) -> Option<PlaceId> {
        if let Some((root, _, false)) = self.projected_reference_roots.get(&place) {
            return Some(*root);
        }
        let mut current = place;
        let mut seen = BTreeSet::new();
        while seen.insert(current) {
            let Some(parent) = self.immutable_reference_origins.get(&current).copied() else {
                return Some(current);
            };
            current = parent;
        }
        None
    }

    fn projection_root_identity(&self, place: PlaceId) -> Option<PlaceId> {
        let mut current = place;
        let mut seen = BTreeSet::new();
        while seen.insert(current) {
            let Some(parent) = self.projection_parents.get(&current).copied() else {
                return Some(current);
            };
            current = parent;
        }
        None
    }

    fn immutable_enum_reference_place(&self, place: PlaceId) -> bool {
        self.immutable_reference_place(place)
            && matches!(
                self.places.get(&place).and_then(PlaceType::logical),
                Some(LogicalType::Enum { .. })
            )
    }

    fn verify_enum_reference_match_read_shape(
        &self,
        block_index: usize,
        position: usize,
        result: &Value,
        reference: &Value,
        schema: &EnumSchema,
        mutability: &'static str,
    ) -> Result<PlaceId, IrVerificationError> {
        if !valid_enum_schema(schema) {
            return Err(self.error(
                block_index,
                IrVerificationErrorKind::UnsupportedType(format!(
                    "checked {mutability} enum Match read schema `{}`",
                    schema.name
                )),
            ));
        }
        let reference = self.require_place(
            reference,
            "checked enum-reference Match read reference",
            block_index,
            position,
        )?;
        if self.places.get(&reference).and_then(PlaceType::logical) != Some(schema.logical_type()) {
            return Err(self.error(
                block_index,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "checked {mutability} enum Match read reference place {} does not carry the exact enum schema",
                    reference.0
                )),
            ));
        }
        let result = reg(result).map(ResultId);
        let valued_dispatch = matches!(
            (
                self.body.instructions.get(position + 1),
                self.body.instructions.get(position + 2),
            ),
            (
                Some(Inst::CheckedMatchResultPlaceAlloca {
                    dispatch_schema,
                    ..
                }),
                Some(Inst::CheckedEnumDispatch {
                    value,
                    schema: dispatch_schema_again,
                    ..
                })
            ) if dispatch_schema == schema
                && dispatch_schema_again == schema
                && reg(value).map(ResultId) == result
        );
        let void_dispatch = matches!(
            self.body.instructions.get(position + 1),
            Some(Inst::CheckedEnumDispatch {
                value,
                schema: dispatch_schema,
                ..
            }) if dispatch_schema == schema && reg(value).map(ResultId) == result
        );
        if !valued_dispatch && !void_dispatch {
            return Err(self.error(
                block_index,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "checked {mutability} enum Match read must be followed by its exact exhaustive dispatch, with a result place only for a value-producing Match"
                )),
            ));
        }
        Ok(reference)
    }

    fn consume_enum_owner(
        &self,
        value: &Value,
        consumed: &mut BTreeSet<EnumOwner>,
        block: usize,
        operation: &'static str,
    ) -> Result<(), IrVerificationError> {
        let Some(owner) = self.enum_owner(value) else {
            return Ok(());
        };
        if !consumed.insert(owner) {
            return Err(self.error(
                block,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "{operation} consumes enum owner {owner} more than once on a reachable control-flow path"
                )),
            ));
        }
        Ok(())
    }

    fn reset_enum_result(&self, instruction: &Inst, consumed: &mut BTreeSet<EnumOwner>) {
        let Some(result) = result_definition(instruction).and_then(reg).map(ResultId) else {
            return;
        };
        if matches!(
            self.result_types.get(&result),
            Some(LogicalType::Enum { .. })
        ) && !matches!(
            self.enum_owner(&Value::Reg(result.0)),
            Some(EnumOwner::Place(_))
        ) {
            consumed.remove(&EnumOwner::Result(result));
        }
    }

    fn verify_mutable_owner_immutable_enum_loan_flow(&self) -> Result<(), IrVerificationError> {
        let by_label = self
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.label.clone(), index))
            .collect::<HashMap<_, _>>();
        let mut incoming = vec![None::<BTreeMap<PlaceId, PlaceId>>; self.blocks.len()];
        incoming[0] = Some(BTreeMap::new());
        let mut queue = VecDeque::from([0]);

        while let Some(block_index) = queue.pop_front() {
            let mut active = incoming[block_index]
                .clone()
                .expect("queued loan-flow block has an incoming state");
            for (position, instruction) in &self.blocks[block_index].instructions {
                match instruction {
                    Inst::CheckedImmutableBorrow { result, source, .. } => {
                        let Some(reference) = reg(result).map(PlaceId) else {
                            continue;
                        };
                        let Some(source) = reg(source).map(PlaceId) else {
                            continue;
                        };
                        if self.mutable_owned_places.contains(&source)
                            && self.immutable_enum_reference_origins.get(&reference)
                                == Some(&source)
                            && active.insert(reference, source).is_some()
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "immutable enum reference place {} is activated more than once on one control-flow path",
                                    reference.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                        reference, source, ..
                    } => {
                        let Some(reference) = reg(reference).map(PlaceId) else {
                            continue;
                        };
                        let Some(source) = reg(source).map(PlaceId) else {
                            continue;
                        };
                        if active.remove(&reference) != Some(source) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "mutable-owner immutable enum loan end for reference place {} does not match the active control-flow state",
                                    reference.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedImmutableEnumMatchRead { reference, .. } => {
                        let Some(reference) = reg(reference).map(PlaceId) else {
                            continue;
                        };
                        if let Some(source) = self
                            .immutable_enum_reference_origins
                            .get(&reference)
                            .copied()
                            && self.mutable_owned_places.contains(&source)
                            && active.get(&reference) != Some(&source)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "immutable enum Match read through reference place {} is outside its active control-flow loan",
                                    reference.0
                                )),
                            ));
                        }
                    }
                    Inst::Call { arguments, .. } => {
                        for argument in arguments {
                            let Some(reference) = reg(argument).map(PlaceId) else {
                                continue;
                            };
                            if let Some(source) = self
                                .immutable_enum_reference_origins
                                .get(&reference)
                                .copied()
                                && self.mutable_owned_places.contains(&source)
                                && active.get(&reference) != Some(&source)
                            {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "immutable enum reference place {} is passed outside its active control-flow loan",
                                        reference.0
                                    )),
                                ));
                            }
                        }
                    }
                    Inst::Store(target, _)
                    | Inst::CheckedOwnedPlaceAssignment { target, .. }
                    | Inst::Load(_, target)
                    | Inst::CheckedMutableBorrow { source: target, .. } => {
                        let Some(target) = reg(target).map(PlaceId) else {
                            continue;
                        };
                        if active.values().any(|source| *source == target) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "owner place {} is accessed while an immutable enum loan is active on this control-flow path",
                                    target.0
                                )),
                            ));
                        }
                    }
                    Inst::Return(_) if !active.is_empty() => {
                        return Err(self.error(
                            block_index,
                            IrVerificationErrorKind::MetadataMismatch(
                                "mutable-owner immutable enum loans must end before every return"
                                    .to_string(),
                            ),
                        ));
                    }
                    _ => {
                        let _ = position;
                    }
                }
            }

            for successor in &self.blocks[block_index].successors {
                let successor = by_label[successor];
                match &incoming[successor] {
                    None => {
                        incoming[successor] = Some(active.clone());
                        queue.push_back(successor);
                    }
                    Some(existing) if existing == &active => {}
                    Some(_) => {
                        return Err(self.error(
                            successor,
                            IrVerificationErrorKind::MetadataMismatch(
                                "mutable-owner immutable enum loan state differs across a control-flow join"
                                    .to_string(),
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn transfer_enum_ownership(
        &self,
        block: usize,
        consumed: &mut BTreeSet<EnumOwner>,
    ) -> Result<(), IrVerificationError> {
        for (_, instruction) in &self.blocks[block].instructions {
            if let Inst::CheckedMutableOwnedPlaceAlloca { result, ty, .. } = instruction
                && matches!(ty, LogicalType::Enum { .. })
                && let Some(place) = self.enum_place(result)
            {
                consumed.remove(&EnumOwner::Place(place));
            }
            if let Inst::CheckedMatchResultPlaceAlloca { result, .. } = instruction
                && let Some(place) = self.enum_place(result)
            {
                consumed.insert(EnumOwner::Place(place));
            }

            match instruction {
                Inst::Store(target, value) if self.enum_place(target).is_some() => {
                    let target = self.enum_place(target).expect("enum place checked above");
                    if self.enum_owner(value) == Some(EnumOwner::Place(target)) {
                        return Err(self.error(
                            block,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "enum initialization of place {} cannot consume that same place",
                                target.0
                            )),
                        ));
                    }
                    self.consume_enum_owner(value, consumed, block, "enum initialization")?;
                    consumed.remove(&EnumOwner::Place(target));
                }
                Inst::CheckedOwnedPlaceAssignment { target, value, ty }
                    if matches!(ty, LogicalType::Enum { .. }) =>
                {
                    let target = self
                        .enum_place(target)
                        .expect("checked enum assignment target type was verified");
                    if self.enum_owner(value) == Some(EnumOwner::Place(target)) {
                        return Err(self.error(
                            block,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "checked enum assignment cannot replace place {} from its own consumed value",
                                target.0
                            )),
                        ));
                    }
                    self.consume_enum_owner(value, consumed, block, "checked enum assignment")?;
                    consumed.remove(&EnumOwner::Place(target));
                }
                Inst::CheckedMutableBorrow {
                    source, pointee, ..
                } if matches!(pointee, LogicalType::Enum { .. }) => {
                    let source = PlaceId(reg(source).expect("checked mutable borrow source"));
                    if let Some(root) = self.mutable_reference_root(source)
                        && self.enum_place(&Value::Reg(root.0)).is_some()
                        && consumed.contains(&EnumOwner::Place(root))
                    {
                        return Err(self.error(
                            block,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "checked mutable enum borrow cannot use consumed owner place {}",
                                root.0
                            )),
                        ));
                    }
                }
                Inst::CheckedMutableDereferenceAssignment {
                    target,
                    value,
                    pointee,
                } if matches!(pointee, LogicalType::Enum { .. }) => {
                    let target = PlaceId(reg(target).expect("checked mutable reference target"));
                    let root = self
                        .mutable_reference_root(target)
                        .filter(|root| self.enum_place(&Value::Reg(root.0)).is_some());
                    if root
                        .is_some_and(|root| self.enum_owner(value) == Some(EnumOwner::Place(root)))
                    {
                        return Err(self.error(
                            block,
                            IrVerificationErrorKind::MetadataMismatch(
                                "checked mutable enum dereference assignment cannot replace an owner from that same owner's consumed value"
                                    .to_string(),
                            ),
                        ));
                    }
                    self.consume_enum_owner(
                        value,
                        consumed,
                        block,
                        "checked mutable enum dereference assignment",
                    )?;
                    if let Some(root) = root {
                        consumed.remove(&EnumOwner::Place(root));
                    }
                }
                Inst::Load(_, source)
                    if self
                        .enum_place(source)
                        .is_some_and(|place| self.match_result_places.contains(&place)) =>
                {
                    let place = self.enum_place(source).expect("checked above");
                    if consumed.contains(&EnumOwner::Place(place)) {
                        return Err(self.error(
                            block,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "checked Match result place {} is loaded before every reachable dispatch path initializes it",
                                place.0
                            )),
                        ));
                    }
                }
                Inst::Call {
                    function,
                    arguments,
                    ..
                } => {
                    let signature = &self.signatures[function];
                    for (argument, (_, expected)) in arguments.iter().zip(&signature.parameters) {
                        if matches!(expected, LogicalType::Enum { .. }) {
                            self.consume_enum_owner(
                                argument,
                                consumed,
                                block,
                                "by-value enum call argument",
                            )?;
                        }
                    }
                }
                Inst::Return(value)
                    if matches!(self.body.signature.result, LogicalType::Enum { .. }) =>
                {
                    self.consume_enum_owner(value, consumed, block, "enum return")?;
                }
                Inst::CheckedEnumDispatch { value, .. } => {
                    self.consume_enum_owner(value, consumed, block, "checked enum dispatch")?;
                }
                _ => {}
            }
            self.reset_enum_result(instruction, consumed);
        }
        Ok(())
    }

    fn verify_match_result_flow(&self) -> Result<(), IrVerificationError> {
        const UNINITIALIZED: u8 = 1;
        const INITIALIZED: u8 = 2;

        if self.match_result_places.is_empty() {
            return Ok(());
        }

        let labels = self
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.label.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut load_counts = self
            .match_result_places
            .iter()
            .map(|place| (*place, 0usize))
            .collect::<BTreeMap<_, _>>();
        let mut assignment_blocks = self
            .match_result_places
            .iter()
            .map(|place| (*place, Vec::<usize>::new()))
            .collect::<BTreeMap<_, _>>();
        for (block_index, block) in self.blocks.iter().enumerate() {
            for (_, instruction) in &block.instructions {
                if let Inst::Store(target, _) = instruction
                    && reg(target)
                        .map(PlaceId)
                        .is_some_and(|place| self.match_result_places.contains(&place))
                {
                    return Err(self.error(
                        block_index,
                        IrVerificationErrorKind::MetadataMismatch(
                            "checked Match result places require exact checked arm assignments; generic Store is not admitted"
                                .to_string(),
                        ),
                    ));
                }
                if let Inst::Load(_, source) = instruction
                    && let Some(place) = reg(source).map(PlaceId)
                    && let Some(count) = load_counts.get_mut(&place)
                {
                    *count += 1;
                }
                if let Inst::CheckedOwnedPlaceAssignment { target, .. } = instruction
                    && let Some(place) = reg(target).map(PlaceId)
                    && let Some(blocks) = assignment_blocks.get_mut(&place)
                {
                    blocks.push(block_index);
                }
            }
        }
        if let Some((place, count)) = load_counts.iter().find(|(_, count)| **count != 1) {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "checked Match result place {} requires exactly one merged load, actual {count}",
                    place.0
                )),
            ));
        }

        for place in &self.match_result_places {
            let definition = self
                .place_definitions
                .get(place)
                .expect("checked Match result place definition was collected");
            let Inst::CheckedEnumDispatch { targets, .. } =
                self.body.instructions[definition.position + 1]
            else {
                unreachable!("checked Match result place adjacency was already verified")
            };
            let assignments = &assignment_blocks[place];
            if assignments.len() != targets.len() {
                return Err(self.error(
                    definition.block,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "checked Match result place {} requires one static assignment per dispatch target: expected {}, actual {}",
                        place.0,
                        targets.len(),
                        assignments.len()
                    )),
                ));
            }
            let target_blocks = targets
                .iter()
                .map(|target| labels[target.as_str()])
                .collect::<Vec<_>>();
            let mut target_assignment_counts = vec![0usize; target_blocks.len()];
            for assignment in assignments {
                let dominated_by = target_blocks
                    .iter()
                    .enumerate()
                    .filter_map(|(target, block)| {
                        self.dominators[*assignment]
                            .contains(block)
                            .then_some(target)
                    })
                    .collect::<Vec<_>>();
                if dominated_by.len() != 1 {
                    return Err(self.error(
                        *assignment,
                        IrVerificationErrorKind::MetadataMismatch(format!(
                            "checked Match result place {} assignment must be dominated by exactly one dispatch target",
                            place.0
                        )),
                    ));
                }
                target_assignment_counts[dominated_by[0]] += 1;
            }
            if target_assignment_counts.iter().any(|count| *count != 1) {
                return Err(self.error(
                    definition.block,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "checked Match result place {} assignments do not map one-to-one onto its dispatch targets",
                        place.0
                    )),
                ));
            }
        }

        let mut incoming = vec![None::<BTreeMap<PlaceId, u8>>; self.blocks.len()];
        incoming[0] = Some(BTreeMap::new());
        let mut worklist = VecDeque::from([0]);
        while let Some(block_index) = worklist.pop_front() {
            if !self.blocks[block_index].reachable {
                continue;
            }
            let mut state = incoming[block_index]
                .clone()
                .expect("reachable Match-result worklist block has an initialization state");
            for (_, instruction) in &self.blocks[block_index].instructions {
                match instruction {
                    Inst::CheckedMatchResultPlaceAlloca { result, .. } => {
                        let place = PlaceId(reg(result).expect("checked Match result place"));
                        state.insert(place, UNINITIALIZED);
                    }
                    Inst::CheckedOwnedPlaceAssignment { target, .. } => {
                        let Some(place) = reg(target)
                            .map(PlaceId)
                            .filter(|place| self.match_result_places.contains(place))
                        else {
                            continue;
                        };
                        let actual = state.get(&place).copied().unwrap_or_default();
                        if actual != UNINITIALIZED {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked Match result place {} must be uninitialized on every path before its arm assignment",
                                    place.0
                                )),
                            ));
                        }
                        state.insert(place, INITIALIZED);
                    }
                    Inst::Load(_, source) => {
                        let Some(place) = reg(source)
                            .map(PlaceId)
                            .filter(|place| self.match_result_places.contains(place))
                        else {
                            continue;
                        };
                        let actual = state.get(&place).copied().unwrap_or_default();
                        if actual != INITIALIZED {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked Match result place {} must be initialized on every reachable path before its merged load",
                                    place.0
                                )),
                            ));
                        }
                    }
                    _ => {}
                }
            }
            for successor in &self.blocks[block_index].successors {
                let successor = labels[successor.as_str()];
                let changed = match &mut incoming[successor] {
                    Some(existing) => {
                        let before = existing.clone();
                        for (place, possible) in &state {
                            existing
                                .entry(*place)
                                .and_modify(|current| *current |= *possible)
                                .or_insert(*possible);
                        }
                        *existing != before
                    }
                    slot @ None => {
                        *slot = Some(state.clone());
                        true
                    }
                };
                if changed {
                    worklist.push_back(successor);
                }
            }
        }
        Ok(())
    }

    fn verify_enum_ownership_flow(&self) -> Result<(), IrVerificationError> {
        let labels = self
            .blocks
            .iter()
            .enumerate()
            .map(|(index, block)| (block.label.as_str(), index))
            .collect::<HashMap<_, _>>();
        let mut incoming = vec![None::<BTreeSet<EnumOwner>>; self.blocks.len()];
        incoming[0] = Some(BTreeSet::new());
        let mut worklist = VecDeque::from([0]);
        while let Some(block) = worklist.pop_front() {
            if !self.blocks[block].reachable {
                continue;
            }
            let mut consumed = incoming[block]
                .clone()
                .expect("reachable worklist block has an ownership state");
            self.transfer_enum_ownership(block, &mut consumed)?;
            for successor in &self.blocks[block].successors {
                let successor = labels[successor.as_str()];
                let changed = match &mut incoming[successor] {
                    Some(existing) => {
                        let before = existing.len();
                        existing.extend(consumed.iter().copied());
                        existing.len() != before
                    }
                    slot @ None => {
                        *slot = Some(consumed.clone());
                        true
                    }
                };
                if changed {
                    worklist.push_back(successor);
                }
            }
        }
        Ok(())
    }

    fn collect_definitions(
        &mut self,
        seeded: bool,
        place_hints: Option<&BTreeMap<PlaceId, LogicalType>>,
    ) -> Result<(), IrVerificationError> {
        if !seeded {
            // Array allocations establish the only legal GEP aggregate bases. Seed
            // them before the textual scan so block serialization does not affect
            // place construction; dominance is checked separately at each use.
            for block in &self.blocks {
                for (_, instruction) in &block.instructions {
                    let Inst::AllocaArray {
                        result,
                        elem_type,
                        count,
                    } = instruction
                    else {
                        continue;
                    };
                    let Some(id) = reg(result).map(PlaceId) else {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::ExpectedPlaceIdentifier("alloca array"),
                        ));
                    };
                    if elem_type != "double" {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::UnsupportedType(format!(
                                "physical array element `{elem_type}`; checked arrays require `double`"
                            )),
                        ));
                    }
                    let hinted_element = place_hints.and_then(|hints| hints.get(&id));
                    if hinted_element.is_some_and(|hint| !hint.is_numeric()) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::UnsupportedType(format!(
                                "{} array element hint",
                                hinted_element.expect("checked above")
                            )),
                        ));
                    }
                    self.places.insert(
                        id,
                        PlaceType::Array {
                            logical_element: hinted_element.cloned(),
                            physical_element: elem_type.clone(),
                            count: *count,
                            checked_copy_data: false,
                        },
                    );
                    self.place_names.insert(id, None);
                }
            }
            for block in &self.blocks {
                for (_, instruction) in &block.instructions {
                    let Inst::CheckedCopyStructArrayAlloca {
                        result,
                        element,
                        count,
                    } = instruction
                    else {
                        continue;
                    };
                    let Some(id) = reg(result).map(PlaceId) else {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                "checked Copy-data array alloca",
                            ),
                        ));
                    };
                    if !valid_copy_data_type(element) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::UnsupportedType(format!(
                                "checked Copy-data array element `{element}`"
                            )),
                        ));
                    }
                    self.places.insert(
                        id,
                        PlaceType::Array {
                            logical_element: Some(element.clone()),
                            physical_element: physical_copy_type_hint(element),
                            count: *count,
                            checked_copy_data: true,
                        },
                    );
                    self.place_names.insert(id, None);
                }
            }
            for block in &self.blocks {
                for (_, instruction) in &block.instructions {
                    let Inst::CheckedStructAlloca {
                        result,
                        struct_name,
                        field_types,
                    } = instruction
                    else {
                        continue;
                    };
                    let Some(id) = reg(result).map(PlaceId) else {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                "checked struct alloca",
                            ),
                        ));
                    };
                    if !valid_named_struct_schema(struct_name, field_types) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::UnsupportedType(format!(
                                "checked struct schema `{struct_name}`"
                            )),
                        ));
                    }
                    self.places.insert(
                        id,
                        PlaceType::Known(LogicalType::Struct {
                            name: struct_name.clone(),
                            fields: field_types.clone(),
                        }),
                    );
                    self.place_names.insert(id, None);
                }
            }
        }
        let mut seen_places = BTreeSet::new();
        let mut claimed_parameters = HashSet::new();
        for (block_index, block) in self.blocks.iter().enumerate() {
            for (position, instruction) in &block.instructions {
                if let Some(name) = unsupported_name(instruction) {
                    return Err(IrVerificationError::new(
                        &self.body.name,
                        Some(&block.label),
                        IrVerificationErrorKind::UnsupportedInstruction(name),
                    ));
                }
                if let Some(value) = place_definition(instruction) {
                    let Some(id) = reg(value) else {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::ExpectedPlaceIdentifier(match instruction {
                                Inst::Alloca(..) => "alloca",
                                Inst::CheckedMutableOwnedPlaceAlloca { .. } => {
                                    "checked mutable owned-place alloca"
                                }
                                Inst::CheckedImmutableEnumOwnerPlaceAlloca { .. } => {
                                    "checked immutable enum owner-place alloca"
                                }
                                Inst::CheckedMatchResultPlaceAlloca { .. } => {
                                    "checked Match result-place alloca"
                                }
                                Inst::AllocaArray { .. } => "alloca array",
                                Inst::CheckedStructAlloca { .. } => "checked struct alloca",
                                Inst::CheckedStructFieldPtr { .. } => {
                                    "checked struct field pointer"
                                }
                                Inst::CheckedTupleAlloca { .. } => "checked tuple alloca",
                                Inst::CheckedTupleFieldPtr { .. } => "checked tuple field pointer",
                                Inst::CheckedImmutableBorrow { .. } => "checked immutable borrow",
                                Inst::CheckedMutableBorrow { .. } => "checked mutable borrow",
                                Inst::CheckedImmutableReferenceParameter { .. } => {
                                    "checked immutable reference parameter"
                                }
                                Inst::CheckedMutableReferenceParameter { .. } => {
                                    "checked mutable reference parameter"
                                }
                                _ => "getelementptr",
                            }),
                        ));
                    };
                    let id = PlaceId(id);
                    if !seen_places.insert(id) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::DuplicatePlaceDefinition(id),
                        ));
                    }
                    self.place_definitions.insert(
                        id,
                        Definition {
                            block: block_index,
                            position: *position,
                        },
                    );
                    if matches!(
                        instruction,
                        Inst::Alloca(..)
                            | Inst::AllocaArray { .. }
                            | Inst::CheckedCopyStructArrayAlloca { .. }
                            | Inst::CheckedStructAlloca { .. }
                            | Inst::CheckedTupleAlloca { .. }
                            | Inst::CheckedMutableOwnedPlaceAlloca { .. }
                    ) {
                        self.projected_owned_roots.insert(id);
                    }
                    if self.definitions.contains_key(&ResultId(id.0)) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::IdentifierKindCollision(id.0),
                        ));
                    }
                    let (place_type, name) = match instruction {
                        Inst::CheckedMutableOwnedPlaceAlloca { name, ty, .. } => {
                            if !valid_symbol(name) || !valid_owned_place_type(ty) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked mutable owned place `{name}` requires a valid name and admitted CopyData-or-enum metadata"
                                    )),
                                ));
                            }
                            self.mutable_owned_places.insert(id);
                            let place_type = match ty {
                                LogicalType::Array { element, count } => PlaceType::Array {
                                    logical_element: Some((**element).clone()),
                                    physical_element: physical_copy_type_hint(element),
                                    count: *count,
                                    checked_copy_data: true,
                                },
                                _ => PlaceType::Known(ty.clone()),
                            };
                            (place_type, Some(name.clone()))
                        }
                        Inst::CheckedImmutableEnumOwnerPlaceAlloca { name, schema, .. } => {
                            if !valid_symbol(name) || !valid_enum_schema(schema) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked immutable enum owner place `{name}` requires a valid name and admitted enum schema"
                                    )),
                                ));
                            }
                            self.immutable_enum_owner_places.insert(id);
                            (PlaceType::Known(schema.logical_type()), Some(name.clone()))
                        }
                        Inst::CheckedMatchResultPlaceAlloca {
                            result_type,
                            dispatch_schema,
                            ..
                        } => {
                            if !valid_owned_place_type(result_type)
                                || !valid_enum_schema(dispatch_schema)
                                || !matches!(
                                    self.body.instructions.get(position + 1),
                                    Some(Inst::CheckedEnumDispatch {
                                        schema: actual_dispatch_schema,
                                        ..
                                    }) if actual_dispatch_schema == dispatch_schema
                                )
                            {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(
                                        "checked Match result place requires an admitted CopyData-or-enum result type, an admitted dispatch schema, and an immediately following exact dispatch schema"
                                            .to_string(),
                                    ),
                                ));
                            }
                            self.match_result_places.insert(id);
                            let place_type = match result_type {
                                LogicalType::Array { element, count } => PlaceType::Array {
                                    logical_element: Some((**element).clone()),
                                    physical_element: physical_copy_type_hint(element),
                                    count: *count,
                                    checked_copy_data: true,
                                },
                                _ => PlaceType::Known(result_type.clone()),
                            };
                            (place_type, None)
                        }
                        Inst::Alloca(_, name) => {
                            let parameter_type = self
                                .body
                                .signature
                                .parameters
                                .iter()
                                .find(|(parameter, _)| parameter == name)
                                .and_then(|(_, ty)| {
                                    claimed_parameters.insert(name.clone()).then(|| ty.clone())
                                });
                            if matches!(parameter_type.as_ref(), Some(LogicalType::Enum { .. })) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "enum parameter `{name}` requires a direct checked parameter binder"
                                    )),
                                ));
                            }
                            if matches!(
                                parameter_type.as_ref(),
                                Some(LogicalType::ImmutableReference { .. })
                            ) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "immutable reference parameter `{name}` requires a direct checked parameter binder"
                                    )),
                                ));
                            }
                            let place_type =
                                parameter_type.map_or(PlaceType::Numeric, |ty| match ty {
                                    LogicalType::Array { element, count } => PlaceType::Array {
                                        physical_element: physical_copy_type_hint(element.as_ref()),
                                        checked_copy_data: true,
                                        logical_element: Some(*element),
                                        count,
                                    },
                                    ty => PlaceType::Known(ty),
                                });
                            (place_type, Some(name.clone()))
                        }
                        Inst::AllocaArray {
                            elem_type, count, ..
                        } => {
                            if elem_type != "double" {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "physical array element `{elem_type}`; checked arrays require `double`"
                                    )),
                                ));
                            }
                            let element = logical_type(elem_type).ok_or_else(|| {
                                IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(elem_type.clone()),
                                )
                            })?;
                            if !element.is_numeric() {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "{elem_type} array element"
                                    )),
                                ));
                            }
                            let hinted_element = place_hints.and_then(|hints| hints.get(&id));
                            if hinted_element.is_some_and(|hint| !hint.is_numeric()) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "{} array element hint",
                                        hinted_element.expect("checked above")
                                    )),
                                ));
                            }
                            (
                                PlaceType::Array {
                                    logical_element: hinted_element
                                        .cloned()
                                        .or_else(|| (elem_type != "double").then_some(element)),
                                    physical_element: elem_type.clone(),
                                    count: *count,
                                    checked_copy_data: false,
                                },
                                None,
                            )
                        }
                        Inst::GetElementPtr {
                            base, elem_type, ..
                        } => {
                            let Some(base_id) = reg(base) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "getelementptr base",
                                    ),
                                ));
                            };
                            let base_type =
                                self.places.get(&PlaceId(base_id)).ok_or_else(|| {
                                    IrVerificationError::new(
                                        &self.body.name,
                                        Some(&block.label),
                                        IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                            "getelementptr base",
                                        ),
                                    )
                                })?;
                            let PlaceType::Array {
                                logical_element,
                                physical_element,
                                count,
                                checked_copy_data,
                                ..
                            } = base_type
                            else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "getelementptr array base",
                                    ),
                                ));
                            };
                            if *checked_copy_data {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(
                                        "legacy getelementptr cannot address a checked Copy-data array"
                                            .to_string(),
                                    ),
                                ));
                            }
                            let aggregate_descriptor = format!("[{count} x {physical_element}]");
                            if aggregate_descriptor != *elem_type {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::GepElementTypeMismatch {
                                        expected: physical_element.clone(),
                                        actual: elem_type.clone(),
                                    },
                                ));
                            }
                            let element_place = if let Some(logical_element) = logical_element {
                                PlaceType::Known(logical_element.clone())
                            } else if physical_element == "double" {
                                PlaceType::Numeric
                            } else {
                                PlaceType::Known(
                                    logical_element.clone().unwrap_or(LogicalType::Float),
                                )
                            };
                            self.element_owners.insert(id, PlaceId(base_id));
                            (element_place, None)
                        }
                        Inst::CheckedCopyStructArrayAlloca { element, count, .. } => {
                            if !valid_copy_data_type(element) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked Copy-data array element `{element}`"
                                    )),
                                ));
                            }
                            (
                                PlaceType::Array {
                                    logical_element: Some(element.clone()),
                                    physical_element: physical_copy_type_hint(element),
                                    count: *count,
                                    checked_copy_data: true,
                                },
                                None,
                            )
                        }
                        Inst::CheckedCopyStructArrayElementPtr {
                            base,
                            element,
                            count,
                            ..
                        } => {
                            if !valid_copy_data_type(element) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked Copy-data array element `{element}`"
                                    )),
                                ));
                            }
                            let Some(base_id) = reg(base).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked Copy-data array base",
                                    ),
                                ));
                            };
                            let Some((actual_element, actual_count)) = self
                                .places
                                .get(&base_id)
                                .and_then(PlaceType::checked_copy_array_contract)
                            else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked Copy-data array base",
                                    ),
                                ));
                            };
                            if &actual_element != element || &actual_count != count {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(
                                        "checked Copy-data array element pointer schema/count mismatch"
                                            .to_string(),
                                    ),
                                ));
                            }
                            self.element_owners.insert(id, base_id);
                            self.projection_parents.insert(id, base_id);
                            (PlaceType::Known(element.clone()), None)
                        }
                        Inst::CheckedStructAlloca {
                            struct_name,
                            field_types,
                            ..
                        } => {
                            if !valid_named_struct_schema(struct_name, field_types) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked struct schema `{struct_name}`"
                                    )),
                                ));
                            }
                            (
                                PlaceType::Known(LogicalType::Struct {
                                    name: struct_name.clone(),
                                    fields: field_types.clone(),
                                }),
                                None,
                            )
                        }
                        Inst::CheckedStructFieldPtr {
                            base,
                            struct_name,
                            field_index,
                            field_type,
                            ..
                        } => {
                            let Some(base_id) = reg(base) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked struct field base",
                                    ),
                                ));
                            };
                            let Some(PlaceType::Known(LogicalType::Struct { name, fields })) =
                                self.places.get(&PlaceId(base_id))
                            else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked struct field base",
                                    ),
                                ));
                            };
                            let actual = fields.get(*field_index as usize);
                            if name != struct_name || actual != Some(field_type) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked struct field pointer schema mismatch for `{struct_name}` field {field_index}"
                                    )),
                                ));
                            }
                            self.projection_parents.insert(id, PlaceId(base_id));
                            (PlaceType::Known(field_type.clone()), None)
                        }
                        Inst::CheckedTupleAlloca { element_types, .. } => {
                            let tuple_type = LogicalType::Tuple {
                                elements: element_types.clone(),
                            };
                            if !valid_copy_data_type(&tuple_type) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(
                                        "checked recursive Copy tuple schema".to_string(),
                                    ),
                                ));
                            }
                            (
                                PlaceType::Known(LogicalType::Tuple {
                                    elements: element_types.clone(),
                                }),
                                None,
                            )
                        }
                        Inst::CheckedTupleFieldPtr {
                            base,
                            element_types,
                            field_index,
                            field_type,
                            ..
                        } => {
                            let tuple_type = LogicalType::Tuple {
                                elements: element_types.clone(),
                            };
                            if !valid_copy_data_type(&tuple_type) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(
                                        "checked recursive Copy tuple field schema".to_string(),
                                    ),
                                ));
                            }
                            let Some(base_id) = reg(base).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked tuple field base",
                                    ),
                                ));
                            };
                            let expected = LogicalType::Tuple {
                                elements: element_types.clone(),
                            };
                            if self.places.get(&base_id).and_then(PlaceType::logical)
                                != Some(expected)
                                || element_types.get(*field_index) != Some(field_type)
                            {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked tuple field pointer schema mismatch at index {field_index}"
                                    )),
                                ));
                            }
                            self.projection_parents.insert(id, base_id);
                            (PlaceType::Known(field_type.clone()), None)
                        }
                        Inst::CheckedImmutableBorrow {
                            source, pointee, ..
                        } => {
                            if !valid_immutable_reference_pointee(pointee) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked immutable reference pointee `{pointee}`"
                                    )),
                                ));
                            }
                            let Some(source_id) = reg(source).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked immutable borrow source",
                                    ),
                                ));
                            };
                            let Some(source_type) = self.places.get(&source_id) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked immutable borrow source",
                                    ),
                                ));
                            };
                            let actual = source_type.logical();
                            if actual.as_ref().is_some_and(|actual| actual != pointee)
                                || (actual.is_none()
                                    && !matches!(
                                        pointee,
                                        LogicalType::Int
                                            | LogicalType::Float
                                            | LogicalType::Bool
                                            | LogicalType::Char
                                    ))
                            {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked immutable borrow pointee `{pointee}` disagrees with source place {}",
                                        source_id.0
                                    )),
                                ));
                            }
                            self.immutable_reference_origins.insert(id, source_id);
                            if matches!(pointee, LogicalType::Enum { .. }) {
                                self.immutable_enum_reference_origins.insert(id, source_id);
                            }
                            (PlaceType::Known(pointee.clone()), None)
                        }
                        Inst::CheckedMutableBorrow {
                            source, pointee, ..
                        } => {
                            if !valid_mutable_reference_pointee(pointee) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked mutable reference pointee `{pointee}`"
                                    )),
                                ));
                            }
                            let Some(source_id) = reg(source).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked mutable borrow source",
                                    ),
                                ));
                            };
                            if !self.mutable_owned_places.contains(&source_id)
                                && !self.mutable_reference_origins.contains_key(&source_id)
                                && !self.mutable_reference_parameters.contains(&source_id)
                            {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked mutable borrow or reborrow source",
                                    ),
                                ));
                            }
                            self.mutable_reference_origins.insert(id, source_id);
                            (PlaceType::Known(pointee.clone()), None)
                        }
                        Inst::CheckedProjectedBorrow {
                            root,
                            source,
                            root_type,
                            pointee,
                            mutable,
                            ..
                        } => {
                            let valid_pointee = if *mutable {
                                valid_mutable_reference_pointee(pointee)
                            } else {
                                valid_immutable_reference_pointee(pointee)
                            };
                            let Some(root_id) = reg(root).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked projected borrow root",
                                    ),
                                ));
                            };
                            let Some(source_id) = reg(source).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked projected borrow source",
                                    ),
                                ));
                            };
                            let root_actual =
                                self.places.get(&root_id).and_then(PlaceType::logical);
                            let source_actual =
                                self.places.get(&source_id).and_then(PlaceType::logical);
                            if !valid_pointee
                                || root_actual.as_ref() != Some(root_type)
                                || source_actual.as_ref() != Some(pointee)
                                || source_id == root_id
                                || !self.projected_owned_roots.contains(&root_id)
                                || self.projection_root_identity(source_id) != Some(root_id)
                                || (*mutable && !self.mutable_owned_places.contains(&root_id))
                            {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked projected borrow metadata does not bind root place {} to projected source place {} with pointee `{pointee}`",
                                        root_id.0, source_id.0
                                    )),
                                ));
                            }
                            if self
                                .projected_reference_roots
                                .insert(id, (root_id, source_id, *mutable))
                                .is_some()
                            {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked projected reference place {} has duplicate root metadata",
                                        id.0
                                    )),
                                ));
                            }
                            if *mutable {
                                self.mutable_reference_origins.insert(id, source_id);
                            } else {
                                self.immutable_reference_origins.insert(id, source_id);
                            }
                            (PlaceType::Known(pointee.clone()), None)
                        }
                        Inst::CheckedImmutableReferenceParameter {
                            parameter, pointee, ..
                        } => {
                            if block_index != 0 || !valid_immutable_reference_pointee(pointee) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked immutable reference parameter `{parameter}` must bind a supported pointee in the entry block"
                                    )),
                                ));
                            }
                            let expected = self
                                .body
                                .signature
                                .parameters
                                .iter()
                                .find(|(name, _)| name == parameter)
                                .map(|(_, ty)| ty);
                            if !matches!(
                                expected,
                                Some(LogicalType::ImmutableReference { pointee: expected })
                                    if expected.as_ref() == pointee
                            ) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked immutable reference parameter `{parameter}` disagrees with its function signature"
                                    )),
                                ));
                            }
                            (PlaceType::Known(pointee.clone()), Some(parameter.clone()))
                        }
                        Inst::CheckedMutableReferenceParameter {
                            parameter, pointee, ..
                        } => {
                            if block_index != 0 || !valid_mutable_reference_pointee(pointee) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked mutable reference parameter `{parameter}` must bind a supported mutable pointee in the entry block"
                                    )),
                                ));
                            }
                            let expected = self
                                .body
                                .signature
                                .parameters
                                .iter()
                                .find(|(name, _)| name == parameter)
                                .map(|(_, ty)| ty);
                            if !matches!(
                                expected,
                                Some(LogicalType::MutableReference { pointee: expected })
                                    if expected.as_ref() == pointee
                            ) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked mutable reference parameter `{parameter}` disagrees with its function signature"
                                    )),
                                ));
                            }
                            self.mutable_reference_parameters.insert(id);
                            (PlaceType::Known(pointee.clone()), Some(parameter.clone()))
                        }
                        _ => unreachable!(),
                    };
                    if !seeded {
                        self.places.insert(id, place_type);
                        self.place_names.insert(id, name);
                    }
                }
                if let Some(value) = result_definition(instruction) {
                    let Some(id) = reg(value) else {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::ExpectedResultIdentifier(match instruction {
                                Inst::Call { .. } => "call",
                                Inst::Load(..) => "load",
                                _ => "instruction",
                            }),
                        ));
                    };
                    let id = ResultId(id);
                    if self
                        .definitions
                        .insert(
                            id,
                            Definition {
                                block: block_index,
                                position: *position,
                            },
                        )
                        .is_some()
                    {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::DuplicateResultDefinition(id),
                        ));
                    }
                    if self.places.contains_key(&PlaceId(id.0)) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::IdentifierKindCollision(id.0),
                        ));
                    }
                    if !seeded {
                        if let Some(ty) = definition_type(
                            instruction,
                            self.signatures,
                            &self.places,
                            &self.result_types,
                        ) {
                            if ty != LogicalType::Void {
                                self.result_types.insert(id, ty);
                            }
                        }
                    }
                }
            }
        }
        if let Some(place_hints) = place_hints {
            for (id, expected) in place_hints {
                match self.places.get(id) {
                    Some(PlaceType::Array {
                        logical_element: Some(actual),
                        ..
                    }) if actual == expected => {}
                    Some(PlaceType::Array { .. }) => {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            None,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "array place {} does not preserve its source element hint",
                                id.0
                            )),
                        ));
                    }
                    Some(_) => {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            None,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "place {} has an array element hint but is not an array",
                                id.0
                            )),
                        ));
                    }
                    None => {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            None,
                            IrVerificationErrorKind::MetadataMismatch(format!(
                                "array element hint references undefined place {}",
                                id.0
                            )),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn known_value_type(&self, value: &Value) -> Option<LogicalType> {
        match value {
            Value::ImmInt(value) => i32::try_from(*value).ok().map(|_| LogicalType::Int),
            Value::ImmFloat(_) => Some(LogicalType::Float),
            Value::ImmChar(_) => Some(LogicalType::Char),
            Value::ImmString(_) => Some(LogicalType::String),
            Value::Reg(id) => self.result_types.get(&ResultId(*id)).cloned(),
        }
    }

    fn resolve_order_independent_types(&mut self) -> Result<(), IrVerificationError> {
        let instructions = self
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(block, body)| {
                body.instructions
                    .iter()
                    .map(move |(_, instruction)| (block, (*instruction).clone()))
            })
            .collect::<Vec<_>>();

        loop {
            let mut progressed = false;
            for (block, instruction) in &instructions {
                if let Inst::Store(place, value) = instruction
                    && let Some(id) = reg(place).map(PlaceId)
                    && matches!(self.places.get(&id), Some(PlaceType::Numeric))
                    && let Some(actual) = self.known_value_type(value)
                    && (actual.is_numeric()
                        || (actual == LogicalType::Bool
                            && self.infer_bool_places
                            && !self.element_owners.contains_key(&id))
                        || (actual == LogicalType::Char && !self.element_owners.contains_key(&id)))
                {
                    self.places.insert(id, PlaceType::Known(actual.clone()));
                    self.refine_loaded_results(id, &actual);
                    self.refine_array_element(id, &actual, *block)?;
                    progressed = true;
                }

                let Some(result) = result_definition(instruction).and_then(reg).map(ResultId)
                else {
                    continue;
                };
                if self.result_types.contains_key(&result) {
                    continue;
                }
                if let Some(ty) = definition_type(
                    instruction,
                    self.signatures,
                    &self.places,
                    &self.result_types,
                ) && ty != LogicalType::Void
                {
                    self.result_types.insert(result, ty);
                    progressed = true;
                }
            }
            if !progressed {
                break;
            }
        }
        Ok(())
    }

    fn value_type(
        &self,
        value: &Value,
        block: usize,
        position: usize,
    ) -> Result<LogicalType, IrVerificationError> {
        match value {
            Value::ImmInt(value) => i32::try_from(*value)
                .map(|_| LogicalType::Int)
                .map_err(|_| self.error(block, IrVerificationErrorKind::IntegerOutOfRange(*value))),
            Value::ImmFloat(_) => Ok(LogicalType::Float),
            Value::ImmChar(_) => Ok(LogicalType::Char),
            Value::ImmString(_) => Ok(LogicalType::String),
            Value::Reg(id) => {
                let id = ResultId(*id);
                let Some(definition) = self.definitions.get(&id) else {
                    return Err(self.error(block, IrVerificationErrorKind::UndefinedResultUse(id)));
                };
                if definition.block == block && definition.position >= position {
                    return Err(self.error(
                        block,
                        IrVerificationErrorKind::ResultUseBeforeDefinition(id),
                    ));
                }
                if definition.block != block && !self.dominators[block].contains(&definition.block)
                {
                    return Err(
                        self.error(block, IrVerificationErrorKind::ResultDoesNotDominateUse(id))
                    );
                }
                self.result_types.get(&id).cloned().ok_or_else(|| {
                    self.error(
                        block,
                        IrVerificationErrorKind::MetadataMismatch(format!(
                            "result {} has no logical type",
                            id.0
                        )),
                    )
                })
            }
        }
    }

    fn require_place(
        &self,
        value: &Value,
        operation: &'static str,
        block: usize,
        position: usize,
    ) -> Result<PlaceId, IrVerificationError> {
        let Some(id) = reg(value) else {
            return Err(self.error(
                block,
                IrVerificationErrorKind::ExpectedPlaceIdentifier(operation),
            ));
        };
        let id = PlaceId(id);
        if !self.places.contains_key(&id) {
            return Err(self.error(block, IrVerificationErrorKind::UndefinedPlaceUse(id)));
        }
        let Some(definition) = self.place_definitions.get(&id) else {
            return Err(self.error(block, IrVerificationErrorKind::UndefinedPlaceUse(id)));
        };
        if definition.block == block && definition.position >= position {
            return Err(self.error(block, IrVerificationErrorKind::PlaceUseBeforeDefinition(id)));
        }
        if definition.block != block && !self.dominators[block].contains(&definition.block) {
            return Err(self.error(block, IrVerificationErrorKind::PlaceDoesNotDominateUse(id)));
        }
        Ok(id)
    }

    fn require_type(
        &self,
        value: &Value,
        expected: &LogicalType,
        operation: &'static str,
        role: &'static str,
        block: usize,
        position: usize,
    ) -> Result<(), IrVerificationError> {
        let actual = self.value_type(value, block, position)?;
        if &actual == expected {
            Ok(())
        } else if actual == LogicalType::Void {
            Err(self.error(block, IrVerificationErrorKind::VoidOperand(role)))
        } else {
            Err(self.error(
                block,
                IrVerificationErrorKind::TypeMismatch {
                    operation,
                    role,
                    expected: expected.to_string(),
                    actual,
                },
            ))
        }
    }

    fn require_enum_variant_guard(
        &self,
        value: &Value,
        schema: &EnumSchema,
        variant_index: usize,
        operation: &'static str,
        block: usize,
    ) -> Result<(), IrVerificationError> {
        let block_label = &self.blocks[block].label;
        let incoming = self
            .blocks
            .iter()
            .filter(|candidate| candidate.successors.contains(block_label))
            .collect::<Vec<_>>();
        let guarded = incoming.len() == 1
            && incoming[0]
                .instructions
                .last()
                .is_some_and(|(_, instruction)| {
                    matches!(
                        instruction,
                        Inst::CheckedEnumDispatch {
                            value: dispatched,
                            schema: dispatched_schema,
                            targets,
                        } if dispatched == value
                            && dispatched_schema == schema
                            && targets.get(variant_index) == Some(block_label)
                    )
                });
        if guarded {
            Ok(())
        } else {
            Err(self.error(
                block,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "{operation} for `{}` is not uniquely guarded by its selected variant target",
                    schema.variants[variant_index].name
                )),
            ))
        }
    }

    fn require_numeric(
        &self,
        value: &Value,
        expected: LogicalType,
        operation: &'static str,
        block: usize,
        position: usize,
    ) -> Result<(), IrVerificationError> {
        self.require_type(value, &expected, operation, "operand", block, position)
    }

    fn refine_loaded_results(&mut self, place: PlaceId, ty: &LogicalType) {
        for block in &self.blocks {
            for (_, instruction) in &block.instructions {
                let Inst::Load(result, source) = instruction else {
                    continue;
                };
                if reg(source) == Some(place.0)
                    && let Some(result) = reg(result)
                {
                    self.result_types.insert(ResultId(result), ty.clone());
                }
            }
        }
    }

    fn refine_array_element(
        &mut self,
        element_place: PlaceId,
        ty: &LogicalType,
        block: usize,
    ) -> Result<(), IrVerificationError> {
        let Some(array_place) = self.element_owners.get(&element_place).copied() else {
            return Ok(());
        };
        if !ty.is_numeric() {
            return Err(self.error(
                block,
                IrVerificationErrorKind::UnsupportedType(format!(
                    "{ty} array element; checked arrays require numeric elements"
                )),
            ));
        }
        let current = match self.places.get(&array_place) {
            Some(PlaceType::Array {
                logical_element, ..
            }) => logical_element.clone(),
            _ => {
                return Err(self.error(
                    block,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "element place {} has no array owner",
                        element_place.0
                    )),
                ));
            }
        };
        if let Some(expected) = current {
            if expected != *ty {
                return Err(self.error(
                    block,
                    IrVerificationErrorKind::TypeMismatch {
                        operation: "store",
                        role: "array element",
                        expected: expected.to_string(),
                        actual: ty.clone(),
                    },
                ));
            }
            return Ok(());
        }
        if let Some(PlaceType::Array {
            logical_element, ..
        }) = self.places.get_mut(&array_place)
        {
            *logical_element = Some(ty.clone());
        }
        let element_places = self
            .element_owners
            .iter()
            .filter_map(|(element, owner)| (*owner == array_place).then_some(*element))
            .collect::<Vec<_>>();
        for element in element_places {
            if matches!(self.places.get(&element), Some(PlaceType::Numeric)) {
                self.places.insert(element, PlaceType::Known(ty.clone()));
                self.refine_loaded_results(element, ty);
            }
        }
        Ok(())
    }

    fn verify(mut self) -> Result<FunctionMetadata, IrVerificationError> {
        let mut bound_enum_parameters = BTreeSet::new();
        let mut bound_reference_parameters = BTreeSet::new();
        let mut bound_mutable_reference_parameters = BTreeSet::new();
        let mut initialized_immutable_enum_places = BTreeSet::new();
        let mut initialized_mutable_places = BTreeSet::new();
        let mut active_mutable_owner_immutable_enum_references = BTreeSet::new();
        let mut active_mutable_owner_immutable_enum_sources = BTreeMap::<PlaceId, usize>::new();
        let mut active_mutable_references = BTreeSet::new();
        let mut active_mutable_sources = BTreeSet::new();
        let mut active_projected_references = BTreeSet::new();
        let mut called_projected_references = BTreeSet::new();
        for block_index in 0..self.blocks.len() {
            let instructions = self.blocks[block_index].instructions.clone();
            for (position, instruction) in instructions {
                if !called_projected_references.is_empty()
                    && !matches!(
                        instruction,
                        Inst::CheckedProjectedBorrowEnd { .. }
                            | Inst::CheckedMutableBorrowEnd { .. }
                    )
                {
                    return Err(self.error(
                        block_index,
                        IrVerificationErrorKind::MetadataMismatch(
                            "projected call loans must end immediately after their call"
                                .to_string(),
                        ),
                    ));
                }
                match instruction {
                    Inst::Add(_, left, right)
                    | Inst::Sub(_, left, right)
                    | Inst::Mul(_, left, right)
                    | Inst::Div(_, left, right) => {
                        let operation = match instruction {
                            Inst::Add(..) => "add",
                            Inst::Sub(..) => "sub",
                            Inst::Mul(..) => "mul",
                            _ => "div",
                        };
                        self.require_numeric(
                            left,
                            LogicalType::Int,
                            operation,
                            block_index,
                            position,
                        )?;
                        self.require_numeric(
                            right,
                            LogicalType::Int,
                            operation,
                            block_index,
                            position,
                        )?;
                        if matches!(instruction, Inst::Div(..)) && matches!(right, Value::ImmInt(0))
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ConstantIntegerDivisionByZero,
                            ));
                        }
                    }
                    Inst::FAdd(_, left, right)
                    | Inst::FSub(_, left, right)
                    | Inst::FMul(_, left, right)
                    | Inst::FDiv(_, left, right) => {
                        let operation = match instruction {
                            Inst::FAdd(..) => "fadd",
                            Inst::FSub(..) => "fsub",
                            Inst::FMul(..) => "fmul",
                            _ => "fdiv",
                        };
                        self.require_numeric(
                            left,
                            LogicalType::Float,
                            operation,
                            block_index,
                            position,
                        )?;
                        self.require_numeric(
                            right,
                            LogicalType::Float,
                            operation,
                            block_index,
                            position,
                        )?;
                    }
                    Inst::Alloca(..)
                    | Inst::CheckedMutableOwnedPlaceAlloca { .. }
                    | Inst::CheckedImmutableEnumOwnerPlaceAlloca { .. }
                    | Inst::AllocaArray { .. }
                    | Inst::CheckedStructAlloca { .. }
                    | Inst::CheckedTupleAlloca { .. }
                    | Inst::Label(_) => {}
                    Inst::Store(place, value) => {
                        let id = self.require_place(place, "store", block_index, position)?;
                        if self.immutable_enum_reference_place(id) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "generic store through immutable enum reference place {} is forbidden",
                                    id.0
                                )),
                            ));
                        }
                        if self.mutable_reference_origins.contains_key(&id)
                            || self.mutable_reference_parameters.contains(&id)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "generic store through mutable reference place {} is forbidden; mutable-reference writes require CheckedMutableDereferenceAssignment",
                                    id.0
                                )),
                            ));
                        }
                        if active_mutable_sources.contains(&id) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "generic store to source place {} is forbidden while its mutable reference is active",
                                    id.0
                                )),
                            ));
                        }
                        if active_mutable_owner_immutable_enum_sources.contains_key(&id) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "generic store to source place {} is forbidden while an immutable enum reference is active",
                                    id.0
                                )),
                            ));
                        }
                        if self.mutable_owned_places.contains(&id) {
                            let definition = self
                                .place_definitions
                                .get(&id)
                                .expect("mutable place definition was collected");
                            if definition.block != block_index
                                || definition.position.checked_add(1) != Some(position)
                                || !initialized_mutable_places.insert(id)
                            {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "generic store to mutable place {} is permitted only once as the adjacent initializer; later writes require a checked assignment",
                                        id.0
                                    )),
                                ));
                            }
                        }
                        if self.immutable_enum_owner_places.contains(&id) {
                            let definition = self
                                .place_definitions
                                .get(&id)
                                .expect("immutable enum owner place definition was collected");
                            if definition.block != block_index
                                || definition.position.checked_add(1) != Some(position)
                                || !initialized_immutable_enum_places.insert(id)
                            {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked immutable enum owner place {} requires exactly one adjacent initializer store",
                                        id.0
                                    )),
                                ));
                            }
                        }
                        let Some(place_type) = self.places.get(&id).cloned() else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier("store"),
                            ));
                        };
                        let actual = self.value_type(value, block_index, position)?;
                        match place_type {
                            PlaceType::Numeric => {
                                if actual == LogicalType::Bool && self.infer_bool_places {
                                    self.places.insert(id, PlaceType::Known(LogicalType::Bool));
                                    self.refine_loaded_results(id, &LogicalType::Bool);
                                    self.refine_array_element(id, &LogicalType::Bool, block_index)?;
                                } else if actual == LogicalType::Char {
                                    self.places.insert(id, PlaceType::Known(LogicalType::Char));
                                    self.refine_loaded_results(id, &LogicalType::Char);
                                    self.refine_array_element(id, &LogicalType::Char, block_index)?;
                                } else if !actual.is_numeric() {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::TypeMismatch {
                                            operation: "store",
                                            role: if actual == LogicalType::String {
                                                "string value into numeric place"
                                            } else {
                                                "bool value into numeric place"
                                            },
                                            expected: "numeric".to_string(),
                                            actual,
                                        },
                                    ));
                                } else {
                                    self.places.insert(id, PlaceType::Known(actual.clone()));
                                    self.refine_loaded_results(id, &actual);
                                    self.refine_array_element(id, &actual, block_index)?;
                                }
                            }
                            PlaceType::Known(expected) => {
                                if actual != expected {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::TypeMismatch {
                                            operation: "store",
                                            role: "value",
                                            expected: expected.to_string(),
                                            actual,
                                        },
                                    ));
                                }
                            }
                            array @ PlaceType::Array { .. } => {
                                let expected = array
                                    .logical()
                                    .expect("checked array places retain exact logical type");
                                if actual != expected {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::TypeMismatch {
                                            operation: "store",
                                            role: "aggregate value",
                                            expected: expected.to_string(),
                                            actual,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                    Inst::CheckedOwnedPlaceAssignment { target, value, ty } => {
                        let target = self.require_place(
                            target,
                            "checked owned-place assignment",
                            block_index,
                            position,
                        )?;
                        let match_result = self.match_result_places.contains(&target);
                        let assignment_root =
                            self.projection_root_identity(target).ok_or_else(|| {
                                self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(
                                        "checked owned-place assignment projection contains a cycle"
                                            .to_string(),
                                    ),
                                )
                            })?;
                        if !valid_owned_place_type(ty)
                            || (!self.mutable_owned_places.contains(&assignment_root)
                                && !match_result)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(
                                    "checked owned-place assignment target is not a declared mutable CopyData-or-enum place or checked Match result place"
                                        .to_string(),
                                ),
                            ));
                        }
                        if !match_result && !initialized_mutable_places.contains(&assignment_root) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked owned-place assignment target {} is not initialized by its adjacent declaration store",
                                    assignment_root.0
                                )),
                            ));
                        }
                        if active_mutable_sources.contains(&assignment_root) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked owned-place assignment to source place {} is forbidden while its mutable reference is active",
                                    assignment_root.0
                                )),
                            ));
                        }
                        if active_mutable_owner_immutable_enum_sources
                            .contains_key(&assignment_root)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked owned-place assignment to source place {} is forbidden while an immutable enum reference is active",
                                    assignment_root.0
                                )),
                            ));
                        }
                        let actual = self.places.get(&target).and_then(PlaceType::logical);
                        if actual.as_ref() != Some(ty) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked owned-place assignment metadata `{ty}` disagrees with target place {}",
                                    target.0
                                )),
                            ));
                        }
                        self.require_type(
                            value,
                            ty,
                            "checked owned-place assignment",
                            "value",
                            block_index,
                            position,
                        )?;
                    }
                    Inst::Load(_, place) => {
                        let place = self.require_place(place, "load", block_index, position)?;
                        if self.immutable_enum_reference_place(place) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "generic load from immutable enum reference place {} is forbidden; CORE-084 requires CheckedImmutableEnumMatchRead",
                                    place.0
                                )),
                            ));
                        }
                        if (self.mutable_reference_origins.contains_key(&place)
                            || self.mutable_reference_parameters.contains(&place))
                            && self
                                .places
                                .get(&place)
                                .and_then(PlaceType::logical)
                                .is_some_and(|ty| matches!(ty, LogicalType::Enum { .. }))
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "generic load from mutable enum reference place {} is forbidden; CORE-086 requires CheckedMutableEnumMatchRead for exhaustive observation",
                                    place.0
                                )),
                            ));
                        }
                        if active_mutable_sources.contains(&place) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "load from source place {} is forbidden while its mutable reference is active",
                                    place.0
                                )),
                            ));
                        }
                        if active_mutable_owner_immutable_enum_sources.contains_key(&place) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "load from source place {} is forbidden while an immutable enum reference is active",
                                    place.0
                                )),
                            ));
                        }
                        if self.mutable_reference_origins.contains_key(&place)
                            && !active_mutable_references.contains(&place)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "load from mutable reference place {} occurs outside its active lexical borrow",
                                    place.0
                                )),
                            ));
                        }
                    }
                    Inst::Return(value) => {
                        if self.body.signature.result == LogicalType::Void {
                            if !matches!(value, Value::ImmInt(0)) {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::VoidOperand("return"),
                                ));
                            }
                        } else {
                            let actual = self.value_type(value, block_index, position)?;
                            if actual != self.body.signature.result {
                                let expected = match self.body.signature.result {
                                    LogicalType::Int => "i32 (Int)".to_string(),
                                    ref ty => ty.to_string(),
                                };
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::TypeMismatch {
                                        operation: "return",
                                        role: "value",
                                        expected,
                                        actual,
                                    },
                                ));
                            }
                        }
                    }
                    Inst::SIToFP(_, value) => self.require_type(
                        value,
                        &LogicalType::Int,
                        "sitofp",
                        "operand",
                        block_index,
                        position,
                    )?,
                    Inst::FPToSI(_, value) => self.require_type(
                        value,
                        &LogicalType::Float,
                        "fptosi",
                        "operand",
                        block_index,
                        position,
                    )?,
                    Inst::FunctionDef { .. } | Inst::CheckedFunctionDef { .. } => {}
                    Inst::Call {
                        function,
                        arguments,
                        result,
                    } => {
                        let Some(signature) = self.signatures.get(function) else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::UnknownFunction(function.clone()),
                            ));
                        };
                        if arguments.len() != signature.parameters.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::CallArity {
                                    function: function.clone(),
                                    expected: signature.parameters.len(),
                                    actual: arguments.len(),
                                },
                            ));
                        }
                        let mut immutable_argument_roots = BTreeSet::new();
                        let mut mutable_call_arguments = Vec::new();
                        for (index, (argument, (_, expected))) in
                            arguments.iter().zip(&signature.parameters).enumerate()
                        {
                            if let LogicalType::ImmutableReference { pointee } = expected {
                                let place = self.require_place(
                                    argument,
                                    "call immutable reference argument",
                                    block_index,
                                    position,
                                )?;
                                if !self.immutable_reference_place(place) {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(format!(
                                            "call immutable reference argument place {} is not an exact checked immutable borrow or immutable-reference parameter",
                                            place.0
                                        )),
                                    ));
                                }
                                let actual = self.places.get(&place).and_then(PlaceType::logical);
                                if actual.as_ref() != Some(pointee.as_ref()) {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::TypeMismatch {
                                            operation: "call",
                                            role: "argument type",
                                            expected: expected.to_string(),
                                            actual: actual.unwrap_or(LogicalType::Void),
                                        },
                                    ));
                                }
                                let root = self
                                    .immutable_reference_root_identity(place)
                                    .ok_or_else(|| {
                                        self.error(
                                            block_index,
                                            IrVerificationErrorKind::MetadataMismatch(format!(
                                                "call immutable reference argument place {} has cyclic origin metadata",
                                                place.0
                                            )),
                                        )
                                    })?;
                                immutable_argument_roots.insert(root);
                            } else if let LogicalType::MutableReference { pointee } = expected {
                                let place = self.require_place(
                                    argument,
                                    "call mutable reference argument",
                                    block_index,
                                    position,
                                )?;
                                let actual = self.places.get(&place).and_then(PlaceType::logical);
                                if actual.as_ref() != Some(pointee.as_ref()) {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::TypeMismatch {
                                            operation: "call",
                                            role: "argument type",
                                            expected: expected.to_string(),
                                            actual: actual.unwrap_or(LogicalType::Void),
                                        },
                                    ));
                                }
                                if !active_mutable_references.contains(&place)
                                    || !self.mutable_reference_origins.contains_key(&place)
                                {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(format!(
                                            "call mutable reference argument place {} is not an active verified mutable borrow temporary",
                                            place.0
                                        )),
                                    ));
                                }
                                let source = self.mutable_reference_origins[&place];
                                mutable_call_arguments.push((
                                    place,
                                    source,
                                    pointee.as_ref().clone(),
                                ));
                            } else {
                                self.require_type(
                                    argument,
                                    expected,
                                    "call",
                                    "argument",
                                    block_index,
                                    position,
                                )
                                .map_err(|mut error| {
                                    if let IrVerificationErrorKind::TypeMismatch { role, .. } =
                                        &mut error.kind
                                    {
                                        *role = "argument type";
                                    }
                                    let _ = index;
                                    error
                                })?;
                            }
                        }
                        let projected_argument_sequence = arguments
                            .iter()
                            .filter_map(|argument| reg(argument).map(PlaceId))
                            .filter(|place| self.projected_reference_roots.contains_key(place))
                            .collect::<Vec<_>>();
                        let projected_arguments = projected_argument_sequence
                            .iter()
                            .copied()
                            .collect::<BTreeSet<_>>();
                        if projected_argument_sequence.len() != projected_arguments.len()
                            || projected_arguments != active_projected_references
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(
                                    "every active projected loan must be consumed by exactly its adjacent call"
                                        .to_string(),
                                ),
                            ));
                        }
                        called_projected_references.extend(projected_arguments);
                        if !mutable_call_arguments.is_empty() {
                            let mut mutable_roots = BTreeSet::new();
                            for (place, _, _) in &mutable_call_arguments {
                                let root = self.mutable_reference_root_identity(*place).ok_or_else(
                                    || {
                                        self.error(
                                            block_index,
                                            IrVerificationErrorKind::MetadataMismatch(format!(
                                                "call mutable reference argument place {} has cyclic origin metadata",
                                                place.0
                                            )),
                                        )
                                    },
                                )?;
                                if !mutable_roots.insert(root) {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(format!(
                                            "call mutable reference arguments do not have pairwise-distinct root identities; root place {} is repeated",
                                            root.0
                                        )),
                                    ));
                                }
                            }
                            if let Some(overlap) = mutable_roots
                                .intersection(&immutable_argument_roots)
                                .next()
                                .copied()
                            {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "call mutable and immutable reference arguments overlap at root place {}",
                                        overlap.0
                                    )),
                                ));
                            }

                            let has_projected_mutable_argument =
                                mutable_call_arguments.iter().any(|(place, _, _)| {
                                    self.projected_reference_roots.contains_key(place)
                                });
                            if !has_projected_mutable_argument {
                                let borrow_start = position
                                .checked_sub(mutable_call_arguments.len())
                                .ok_or_else(|| {
                                    self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(
                                            "call mutable reference window has too few preceding instructions"
                                                .to_string(),
                                        ),
                                    )
                                })?;
                                for (offset, (place, source, pointee)) in
                                    mutable_call_arguments.iter().enumerate()
                                {
                                    let exact_borrow = matches!(
                                        self.body.instructions.get(borrow_start + offset).copied(),
                                        Some(Inst::CheckedMutableBorrow {
                                            result: Value::Reg(result),
                                            source: Value::Reg(origin),
                                            pointee: borrow_pointee,
                                        }) if PlaceId(*result) == *place
                                            && PlaceId(*origin) == *source
                                            && borrow_pointee == pointee
                                    ) || matches!(
                                        self.body.instructions.get(borrow_start + offset).copied(),
                                        Some(Inst::CheckedProjectedBorrow {
                                            result: Value::Reg(result),
                                            source: Value::Reg(origin),
                                            pointee: borrow_pointee,
                                            mutable: true,
                                            ..
                                        }) if PlaceId(*result) == *place
                                            && PlaceId(*origin) == *source
                                            && borrow_pointee == pointee
                                    );
                                    if !exact_borrow {
                                        return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(format!(
                                            "call mutable reference argument place {} is not in the exact ordered N-borrow/call window",
                                            place.0
                                        )),
                                    ));
                                    }
                                }
                                for (offset, (place, source, pointee)) in
                                    mutable_call_arguments.iter().rev().enumerate()
                                {
                                    let exact_end = matches!(
                                        self.body.instructions.get(position + 1 + offset).copied(),
                                        Some(Inst::CheckedMutableBorrowEnd {
                                            reference: Value::Reg(reference),
                                            source: Value::Reg(origin),
                                            pointee: end_pointee,
                                        }) if PlaceId(*reference) == *place
                                            && PlaceId(*origin) == *source
                                            && end_pointee == pointee
                                    ) || matches!(
                                        self.body.instructions.get(position + 1 + offset).copied(),
                                        Some(Inst::CheckedProjectedBorrowEnd {
                                            reference: Value::Reg(reference),
                                            source: Value::Reg(origin),
                                            pointee: end_pointee,
                                            mutable: true,
                                            ..
                                        }) if PlaceId(*reference) == *place
                                            && PlaceId(*origin) == *source
                                            && end_pointee == pointee
                                    );
                                    if !exact_end {
                                        return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(format!(
                                            "call mutable reference argument place {} is not in the exact reverse-N-end window",
                                            place.0
                                        )),
                                    ));
                                    }
                                }
                            }
                        }
                        match (&signature.result, result) {
                            (LogicalType::Void, Some(_)) => {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::VoidCallHasResult(function.clone()),
                                ));
                            }
                            (LogicalType::Void, None) => {}
                            (_, None) => {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MissingCallResult(function.clone()),
                                ));
                            }
                            (_, Some(Value::Reg(_))) => {}
                            (_, Some(_)) => {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::ExpectedResultIdentifier("call"),
                                ));
                            }
                        }
                    }
                    Inst::Branch { condition, .. } => self.require_type(
                        condition,
                        &LogicalType::Bool,
                        "branch",
                        "condition",
                        block_index,
                        position,
                    )?,
                    Inst::Jump(_) => {}
                    Inst::ICmp {
                        op, left, right, ..
                    } => {
                        let operand_type = self.value_type(left, block_index, position)?;
                        let admitted_predicate =
                            match PrimitiveKind::from_logical_type(&operand_type) {
                                Some(primitive) if primitive != PrimitiveKind::Float => {
                                    primitive.admits_integer_predicate(op)
                                }
                                _ => {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::TypeMismatch {
                                            operation: "icmp",
                                            role: "operand",
                                            expected: "Int, Bool, or Char".to_string(),
                                            actual: operand_type.clone(),
                                        },
                                    ));
                                }
                            };
                        self.require_type(
                            right,
                            &operand_type,
                            "icmp",
                            "operand",
                            block_index,
                            position,
                        )?;
                        if !admitted_predicate {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::InvalidPredicate {
                                    operation: "icmp",
                                    predicate: op.clone(),
                                },
                            ));
                        }
                    }
                    Inst::FCmp {
                        op, left, right, ..
                    } => {
                        self.require_type(
                            left,
                            &LogicalType::Float,
                            "fcmp",
                            "operand",
                            block_index,
                            position,
                        )?;
                        self.require_type(
                            right,
                            &LogicalType::Float,
                            "fcmp",
                            "operand",
                            block_index,
                            position,
                        )?;
                        if !matches!(op.as_str(), "oeq" | "one" | "olt" | "ogt" | "ole" | "oge") {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::InvalidPredicate {
                                    operation: "fcmp",
                                    predicate: op.clone(),
                                },
                            ));
                        }
                    }
                    Inst::Print { arguments, .. } | Inst::Println { arguments, .. } => {
                        for argument in arguments {
                            let actual = self.value_type(argument, block_index, position)?;
                            if actual == LogicalType::Void {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::VoidOperand("print"),
                                ));
                            }
                        }
                    }
                    Inst::And { left, right, .. } | Inst::Or { left, right, .. } => {
                        self.require_type(
                            left,
                            &LogicalType::Bool,
                            "logical",
                            "operand",
                            block_index,
                            position,
                        )?;
                        self.require_type(
                            right,
                            &LogicalType::Bool,
                            "logical",
                            "operand",
                            block_index,
                            position,
                        )?;
                    }
                    Inst::Not { operand, .. } => self.require_type(
                        operand,
                        &LogicalType::Bool,
                        "not",
                        "operand",
                        block_index,
                        position,
                    )?,
                    Inst::Neg { result, operand } => {
                        let actual = self.value_type(operand, block_index, position)?;
                        if !actual.is_numeric() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::TypeMismatch {
                                    operation: "neg",
                                    role: "operand",
                                    expected: "numeric".to_string(),
                                    actual,
                                },
                            ));
                        }
                        self.result_types
                            .insert(ResultId(reg(result).expect("collected result")), actual);
                    }
                    Inst::GetElementPtr { base, index, .. } => {
                        let base =
                            self.require_place(base, "getelementptr base", block_index, position)?;
                        if !matches!(self.places.get(&base), Some(PlaceType::Array { .. })) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                    "getelementptr base",
                                ),
                            ));
                        }
                        self.require_type(
                            index,
                            &LogicalType::Int,
                            "getelementptr",
                            "index",
                            block_index,
                            position,
                        )
                        .map_err(|mut error| {
                            if let IrVerificationErrorKind::TypeMismatch { expected, .. } =
                                &mut error.kind
                            {
                                *expected = "integer".to_string();
                            }
                            error
                        })?;
                    }
                    Inst::CheckedCopyStructArrayElementPtr {
                        base, index, count, ..
                    } => {
                        let base = self.require_place(
                            base,
                            "checked Copy-data array base",
                            block_index,
                            position,
                        )?;
                        if self
                            .places
                            .get(&base)
                            .and_then(PlaceType::checked_copy_array_contract)
                            .is_none()
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                    "checked Copy-data array base",
                                ),
                            ));
                        }
                        self.require_type(
                            index,
                            &LogicalType::Int,
                            "checked Copy-data array element pointer",
                            "index",
                            block_index,
                            position,
                        )?;
                        if let Value::ImmInt(constant) = index
                            && usize::try_from(*constant)
                                .map_or(true, |constant| constant >= *count)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked Copy-data array constant index {constant} is outside 0..{count}"
                                )),
                            ));
                        }
                    }
                    Inst::CheckedStructFieldPtr { base, .. } => {
                        self.require_place(
                            base,
                            "checked struct field base",
                            block_index,
                            position,
                        )?;
                    }
                    Inst::CheckedTupleFieldPtr { base, .. } => {
                        self.require_place(
                            base,
                            "checked tuple field base",
                            block_index,
                            position,
                        )?;
                    }
                    Inst::CheckedImmutableBorrow {
                        result,
                        source,
                        pointee,
                    } => {
                        let Some(reference) = reg(result).map(PlaceId) else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                    "checked immutable borrow result",
                                ),
                            ));
                        };
                        let source = self.require_place(
                            source,
                            "checked immutable borrow source",
                            block_index,
                            position,
                        )?;
                        if active_mutable_sources.contains(&source) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable borrow of source place {} is forbidden while its mutable reference is active",
                                    source.0
                                )),
                            ));
                        }
                        let actual = self.places.get(&source).and_then(PlaceType::logical);
                        if actual.as_ref() != Some(pointee) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable borrow pointee mismatch: declared {pointee}, source {}",
                                    actual.map_or_else(|| "unknown".to_string(), |ty| ty.to_string())
                                )),
                            ));
                        }
                        if matches!(pointee, LogicalType::Enum { .. }) {
                            let initialized_immutable_owner =
                                self.immutable_enum_owner_places.contains(&source)
                                    && initialized_immutable_enum_places.contains(&source);
                            let initialized_mutable_owner =
                                self.mutable_owned_places.contains(&source)
                                    && initialized_mutable_places.contains(&source);
                            if (!initialized_immutable_owner && !initialized_mutable_owner)
                                || self.immutable_enum_reference_origins.get(&reference)
                                    != Some(&source)
                            {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked immutable enum borrow source place {} is not an initialized admitted enum owner or its origin metadata is inconsistent",
                                        source.0
                                    )),
                                ));
                            }
                            if initialized_mutable_owner {
                                if !active_mutable_owner_immutable_enum_references.insert(reference)
                                {
                                    return Err(self.error(
                                        block_index,
                                        IrVerificationErrorKind::MetadataMismatch(format!(
                                            "immutable enum reference place {} is already active",
                                            reference.0
                                        )),
                                    ));
                                }
                                *active_mutable_owner_immutable_enum_sources
                                    .entry(source)
                                    .or_insert(0) += 1;
                            }
                        }
                    }
                    Inst::CheckedProjectedBorrow {
                        result,
                        root,
                        source,
                        root_type,
                        pointee,
                        mutable,
                    } => {
                        let Some(reference) = reg(result).map(PlaceId) else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                    "checked projected borrow result",
                                ),
                            ));
                        };
                        let root = self.require_place(
                            root,
                            "checked projected borrow root",
                            block_index,
                            position,
                        )?;
                        let source = self.require_place(
                            source,
                            "checked projected borrow source",
                            block_index,
                            position,
                        )?;
                        let valid_pointee = if *mutable {
                            valid_mutable_reference_pointee(pointee)
                        } else {
                            valid_immutable_reference_pointee(pointee)
                        };
                        let exact_static = self.projected_reference_roots.get(&reference)
                            == Some(&(root, source, *mutable));
                        let exact_types = self.places.get(&root).and_then(PlaceType::logical)
                            == Some(root_type.clone())
                            && self.places.get(&source).and_then(PlaceType::logical)
                                == Some(pointee.clone())
                            && self.places.get(&reference).and_then(PlaceType::logical)
                                == Some(pointee.clone());
                        if !valid_pointee
                            || !exact_static
                            || !exact_types
                            || !self.projected_owned_roots.contains(&root)
                            || self.projection_root_identity(source) != Some(root)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked projected borrow does not authenticate root place {}, source place {}, and reference place {}",
                                    root.0, source.0, reference.0
                                )),
                            ));
                        }
                        if active_projected_references.iter().any(|active| {
                            self.projected_reference_roots.get(active).is_some_and(
                                |(active_root, _, active_mutable)| {
                                    *active_root == root && (*mutable || *active_mutable)
                                },
                            )
                        }) || !active_projected_references.insert(reference)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "projected call loans overlap at conservatively borrowed root place {}",
                                    root.0
                                )),
                            ));
                        }
                        if *mutable {
                            if !self.mutable_owned_places.contains(&root)
                                || !initialized_mutable_places.contains(&root)
                                || active_mutable_sources.contains(&root)
                                || !active_mutable_references.insert(reference)
                                || !active_mutable_sources.insert(root)
                            {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::MetadataMismatch(format!(
                                        "checked mutable projected borrow root place {} is not an initialized exclusive owner",
                                        root.0
                                    )),
                                ));
                            }
                        }
                    }
                    Inst::CheckedImmutableEnumMatchRead {
                        result,
                        reference,
                        schema,
                    } => {
                        let reference = self.verify_enum_reference_match_read_shape(
                            block_index,
                            position,
                            result,
                            reference,
                            schema,
                            "immutable",
                        )?;
                        if !self.immutable_enum_reference_place(reference) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable enum Match read reference place {} does not carry the exact immutable enum schema",
                                    reference.0
                                )),
                            ));
                        }
                        if let Some(source) = self
                            .immutable_enum_reference_origins
                            .get(&reference)
                            .copied()
                            && self.mutable_owned_places.contains(&source)
                            && !active_mutable_owner_immutable_enum_references.contains(&reference)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable enum Match read reference place {} occurs outside its active mutable-owner loan",
                                    reference.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedMutableEnumMatchRead {
                        result,
                        reference,
                        schema,
                    } => {
                        let reference = self.verify_enum_reference_match_read_shape(
                            block_index,
                            position,
                            result,
                            reference,
                            schema,
                            "mutable",
                        )?;
                        let active_local = active_mutable_references.contains(&reference)
                            && self.mutable_reference_origins.contains_key(&reference);
                        let parameter = self.mutable_reference_parameters.contains(&reference);
                        if (!active_local && !parameter)
                            || active_mutable_sources.contains(&reference)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable enum Match read reference place {} is not an active exclusive mutable enum reference",
                                    reference.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedMutableBorrow {
                        result,
                        source,
                        pointee,
                    } => {
                        let Some(reference) = reg(result).map(PlaceId) else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                    "checked mutable borrow result",
                                ),
                            ));
                        };
                        let source = self.require_place(
                            source,
                            "checked mutable borrow source",
                            block_index,
                            position,
                        )?;
                        let initialized_owner = self.mutable_owned_places.contains(&source)
                            && initialized_mutable_places.contains(&source);
                        let active_local_parent = active_mutable_references.contains(&source)
                            && self.mutable_reference_origins.contains_key(&source);
                        let parameter_parent = self.mutable_reference_parameters.contains(&source);
                        if !valid_mutable_reference_pointee(pointee)
                            || (!initialized_owner && !active_local_parent && !parameter_parent)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable borrow source place {} is not an initialized mutable admitted owner, active local mutable reference, or mutable-reference parameter",
                                    source.0
                                )),
                            ));
                        }
                        if active_mutable_owner_immutable_enum_sources.contains_key(&source) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable borrow of source place {} is forbidden while an immutable enum reference is active",
                                    source.0
                                )),
                            ));
                        }
                        let source_type = self.places.get(&source).and_then(PlaceType::logical);
                        let reference_type =
                            self.places.get(&reference).and_then(PlaceType::logical);
                        if source_type.as_ref() != Some(pointee)
                            || reference_type.as_ref() != Some(pointee)
                            || self.mutable_reference_origins.get(&reference) != Some(&source)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable borrow metadata disagrees with reference place {} or source place {}",
                                    reference.0, source.0
                                )),
                            ));
                        }
                        if active_mutable_sources.contains(&source)
                            || !active_mutable_references.insert(reference)
                            || !active_mutable_sources.insert(source)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "source or parent place {} already has an active mutable child reference",
                                    source.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedMutableDereferenceAssignment {
                        target,
                        value,
                        pointee,
                    } => {
                        let target = self.require_place(
                            target,
                            "checked mutable dereference assignment target",
                            block_index,
                            position,
                        )?;
                        let is_active_local_reference = active_mutable_references.contains(&target)
                            && self.mutable_reference_origins.contains_key(&target);
                        if !valid_mutable_reference_pointee(pointee)
                            || (!is_active_local_reference
                                && !self.mutable_reference_parameters.contains(&target))
                            || active_mutable_sources.contains(&target)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable dereference assignment target place {} is not an active verified mutable reference",
                                    target.0
                                )),
                            ));
                        }
                        let actual = self.places.get(&target).and_then(PlaceType::logical);
                        if actual.as_ref() != Some(pointee) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable dereference assignment metadata `{pointee}` disagrees with target place {}",
                                    target.0
                                )),
                            ));
                        }
                        self.require_type(
                            value,
                            pointee,
                            "checked mutable dereference assignment",
                            "value",
                            block_index,
                            position,
                        )?;
                    }
                    Inst::CheckedMutableBorrowEnd {
                        reference,
                        source,
                        pointee,
                    } => {
                        let reference = self.require_place(
                            reference,
                            "checked mutable borrow end reference",
                            block_index,
                            position,
                        )?;
                        let source = self.require_place(
                            source,
                            "checked mutable borrow end source",
                            block_index,
                            position,
                        )?;
                        let reference_type =
                            self.places.get(&reference).and_then(PlaceType::logical);
                        let source_type = self.places.get(&source).and_then(PlaceType::logical);
                        if !valid_mutable_reference_pointee(pointee)
                            || reference_type.as_ref() != Some(pointee)
                            || source_type.as_ref() != Some(pointee)
                            || self.mutable_reference_origins.get(&reference) != Some(&source)
                            || !active_mutable_references.remove(&reference)
                            || !active_mutable_sources.remove(&source)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable borrow end does not match active reference place {} and source place {}",
                                    reference.0, source.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedProjectedBorrowEnd {
                        reference,
                        root,
                        source,
                        root_type,
                        pointee,
                        mutable,
                    } => {
                        let reference = self.require_place(
                            reference,
                            "checked projected borrow end reference",
                            block_index,
                            position,
                        )?;
                        let root = self.require_place(
                            root,
                            "checked projected borrow end root",
                            block_index,
                            position,
                        )?;
                        let source = self.require_place(
                            source,
                            "checked projected borrow end source",
                            block_index,
                            position,
                        )?;
                        let exact = self.projected_reference_roots.get(&reference)
                            == Some(&(root, source, *mutable))
                            && self.places.get(&root).and_then(PlaceType::logical)
                                == Some(root_type.clone())
                            && self.places.get(&source).and_then(PlaceType::logical)
                                == Some(pointee.clone())
                            && self.places.get(&reference).and_then(PlaceType::logical)
                                == Some(pointee.clone());
                        let lifecycle = active_projected_references.remove(&reference)
                            && called_projected_references.remove(&reference);
                        let ended = if *mutable {
                            active_mutable_references.remove(&reference)
                                && active_mutable_sources.remove(&root)
                        } else {
                            true
                        };
                        if !exact || !lifecycle || !ended {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked projected borrow end does not match reference place {}, root place {}, and source place {}",
                                    reference.0, root.0, source.0
                                )),
                            ));
                        }
                    }
                    Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
                        reference,
                        source,
                        schema,
                    } => {
                        let reference = self.require_place(
                            reference,
                            "checked mutable-owner immutable enum borrow end reference",
                            block_index,
                            position,
                        )?;
                        let source = self.require_place(
                            source,
                            "checked mutable-owner immutable enum borrow end source",
                            block_index,
                            position,
                        )?;
                        let exact_schema = valid_enum_schema(schema)
                            && self.places.get(&reference).and_then(PlaceType::logical)
                                == Some(schema.logical_type())
                            && self.places.get(&source).and_then(PlaceType::logical)
                                == Some(schema.logical_type());
                        let exact_origin =
                            self.immutable_enum_reference_origins.get(&reference) == Some(&source);
                        if !exact_schema
                            || !exact_origin
                            || !self.mutable_owned_places.contains(&source)
                            || !active_mutable_owner_immutable_enum_references.remove(&reference)
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable-owner immutable enum borrow end does not match active reference place {} and source place {}",
                                    reference.0, source.0
                                )),
                            ));
                        }
                        let Some(count) =
                            active_mutable_owner_immutable_enum_sources.get_mut(&source)
                        else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable-owner immutable enum borrow end has no active source place {}",
                                    source.0
                                )),
                            ));
                        };
                        *count -= 1;
                        if *count == 0 {
                            active_mutable_owner_immutable_enum_sources.remove(&source);
                        }
                    }
                    Inst::CheckedImmutableReferenceParameter {
                        parameter, pointee, ..
                    } => {
                        if block_index != 0 || !valid_immutable_reference_pointee(pointee) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable reference parameter `{parameter}` must bind a supported pointee in the entry block"
                                )),
                            ));
                        }
                        let expected = self
                            .body
                            .signature
                            .parameters
                            .iter()
                            .find(|(name, _)| name == parameter)
                            .map(|(_, ty)| ty);
                        if !matches!(
                            expected,
                            Some(LogicalType::ImmutableReference { pointee: expected })
                                if expected.as_ref() == pointee
                        ) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable reference parameter `{parameter}` disagrees with its function signature"
                                )),
                            ));
                        }
                        if !bound_reference_parameters.insert(parameter.clone()) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked immutable reference parameter `{parameter}` is bound more than once"
                                )),
                            ));
                        }
                    }
                    Inst::CheckedMutableReferenceParameter {
                        parameter, pointee, ..
                    } => {
                        if block_index != 0 || !valid_mutable_reference_pointee(pointee) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable reference parameter `{parameter}` must bind a supported mutable pointee in the entry block"
                                )),
                            ));
                        }
                        let expected = self
                            .body
                            .signature
                            .parameters
                            .iter()
                            .find(|(name, _)| name == parameter)
                            .map(|(_, ty)| ty);
                        if !matches!(
                            expected,
                            Some(LogicalType::MutableReference { pointee: expected })
                                if expected.as_ref() == pointee
                        ) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable reference parameter `{parameter}` disagrees with its function signature"
                                )),
                            ));
                        }
                        if !bound_mutable_reference_parameters.insert(parameter.clone()) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked mutable reference parameter `{parameter}` is bound more than once"
                                )),
                            ));
                        }
                    }
                    Inst::CheckedEnumParameter {
                        parameter, schema, ..
                    } => {
                        if block_index != 0 || !valid_enum_schema(schema) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum parameter `{parameter}` must bind a valid schema in the entry block"
                                )),
                            ));
                        }
                        let actual = schema.logical_type();
                        let expected = self
                            .body
                            .signature
                            .parameters
                            .iter()
                            .find(|(name, _)| name == parameter)
                            .map(|(_, ty)| ty);
                        if expected != Some(&actual) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum parameter `{parameter}` disagrees with its function signature"
                                )),
                            ));
                        }
                        if !bound_enum_parameters.insert(parameter.clone()) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum parameter `{parameter}` is bound more than once"
                                )),
                            ));
                        }
                    }
                    Inst::CheckedEnumVariant {
                        schema,
                        variant_index,
                        payload,
                        ..
                    } => {
                        if !valid_enum_schema(schema) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::UnsupportedType(format!(
                                    "checked enum schema `{}`",
                                    schema.name
                                )),
                            ));
                        }
                        if *variant_index >= schema.variants.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum variant index {variant_index} is outside 0..{}",
                                    schema.variants.len()
                                )),
                            ));
                        }
                        let expected_payload = schema.variants[*variant_index].payload.as_ref();
                        if matches!(expected_payload, Some(LogicalType::EnumFields { .. })) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum variant `{}` requires the multi-field construction identity",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        }
                        if expected_payload.is_some() != payload.is_some() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum variant `{}` payload presence disagrees with its schema",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        }
                        if let (Some(expected), Some(payload)) = (expected_payload, payload) {
                            self.require_type(
                                payload,
                                expected,
                                "checked enum construction",
                                "payload",
                                block_index,
                                position,
                            )?;
                        }
                    }
                    Inst::CheckedEnumVariantFields {
                        schema,
                        variant_index,
                        fields,
                        ..
                    } => {
                        if !valid_enum_schema(schema) || *variant_index >= schema.variants.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::UnsupportedType(format!(
                                    "checked multi-field enum schema `{}`",
                                    schema.name
                                )),
                            ));
                        }
                        let Some(LogicalType::EnumFields {
                            fields: expected_fields,
                        }) = schema.variants[*variant_index].payload.as_ref()
                        else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked multi-field enum construction names non-product variant `{}`",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        };
                        if fields.len() != expected_fields.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum variant `{}` carries {} fields but its schema requires {}",
                                    schema.variants[*variant_index].name,
                                    fields.len(),
                                    expected_fields.len()
                                )),
                            ));
                        }
                        for (field, expected) in fields.iter().zip(expected_fields) {
                            self.require_type(
                                field,
                                expected,
                                "checked multi-field enum construction",
                                "field",
                                block_index,
                                position,
                            )?;
                        }
                    }
                    Inst::CheckedEnumPayload {
                        value,
                        schema,
                        variant_index,
                        ..
                    } => {
                        if !valid_enum_schema(schema) || *variant_index >= schema.variants.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::UnsupportedType(format!(
                                    "checked enum payload schema `{}`",
                                    schema.name
                                )),
                            ));
                        }
                        if schema.variants[*variant_index].payload.is_none() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum payload extraction names unit variant `{}`",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        }
                        if matches!(
                            schema.variants[*variant_index].payload.as_ref(),
                            Some(LogicalType::EnumFields { .. })
                        ) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum payload extraction for `{}` must use field-indexed identities",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        }
                        self.require_type(
                            value,
                            &schema.logical_type(),
                            "checked enum payload extraction",
                            "value",
                            block_index,
                            position,
                        )?;
                        self.require_enum_variant_guard(
                            value,
                            schema,
                            *variant_index,
                            "checked enum payload extraction",
                            block_index,
                        )?;
                    }
                    Inst::CheckedEnumField {
                        value,
                        schema,
                        variant_index,
                        field_index,
                        ..
                    } => {
                        if !valid_enum_schema(schema) || *variant_index >= schema.variants.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::UnsupportedType(format!(
                                    "checked enum field schema `{}`",
                                    schema.name
                                )),
                            ));
                        }
                        let Some(LogicalType::EnumFields { fields }) =
                            schema.variants[*variant_index].payload.as_ref()
                        else {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum field extraction names non-product variant `{}`",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        };
                        if *field_index >= fields.len() {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum field index {field_index} is outside 0..{} for variant `{}`",
                                    fields.len(),
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        }
                        self.require_type(
                            value,
                            &schema.logical_type(),
                            "checked enum field extraction",
                            "value",
                            block_index,
                            position,
                        )?;
                        self.require_enum_variant_guard(
                            value,
                            schema,
                            *variant_index,
                            "checked enum field extraction",
                            block_index,
                        )?;
                    }
                    Inst::CheckedEnumDispatch {
                        value,
                        schema,
                        targets,
                    } => {
                        if !valid_enum_schema(schema) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::UnsupportedType(format!(
                                    "checked enum schema `{}`",
                                    schema.name
                                )),
                            ));
                        }
                        let unique_targets = targets.iter().collect::<BTreeSet<_>>();
                        if targets.len() != schema.variants.len()
                            || unique_targets.len() != targets.len()
                        {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum dispatch for `{}` must carry one unique target per variant",
                                    schema.name
                                )),
                            ));
                        }
                        self.require_type(
                            value,
                            &schema.logical_type(),
                            "checked enum dispatch",
                            "value",
                            block_index,
                            position,
                        )?;
                    }
                    _ if unsupported_name(instruction).is_some() => unreachable!(),
                    _ => {}
                }
            }
        }

        self.verify_match_result_flow()?;
        self.verify_enum_ownership_flow()?;
        self.verify_mutable_owner_immutable_enum_loan_flow()?;

        if !active_mutable_owner_immutable_enum_references.is_empty()
            || !active_mutable_owner_immutable_enum_sources.is_empty()
        {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "mutable-owner immutable enum loans must end on every admitted function path"
                        .to_string(),
                ),
            ));
        }
        if !active_projected_references.is_empty() || !called_projected_references.is_empty() {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "projected call loans must be called and ended on every admitted path"
                        .to_string(),
                ),
            ));
        }

        if initialized_immutable_enum_places != self.immutable_enum_owner_places {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "checked immutable enum owner places must each have one adjacent initializer store"
                        .to_string(),
                ),
            ));
        }

        let declared_mutable_places = self.mutable_owned_places.clone();
        if initialized_mutable_places != declared_mutable_places {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "checked mutable places must each have one adjacent initializer store"
                        .to_string(),
                ),
            ));
        }

        let expected_enum_parameters = self
            .body
            .signature
            .parameters
            .iter()
            .filter_map(|(name, ty)| matches!(ty, LogicalType::Enum { .. }).then_some(name.clone()))
            .collect::<BTreeSet<_>>();
        if bound_enum_parameters != expected_enum_parameters {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "checked enum parameter binders do not exactly cover the enum signature"
                        .to_string(),
                ),
            ));
        }
        let expected_reference_parameters = self
            .body
            .signature
            .parameters
            .iter()
            .filter_map(|(name, ty)| {
                matches!(ty, LogicalType::ImmutableReference { .. }).then_some(name.clone())
            })
            .collect::<BTreeSet<_>>();
        if bound_reference_parameters != expected_reference_parameters {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "checked immutable reference parameter binders do not exactly cover the reference signature"
                        .to_string(),
                ),
            ));
        }
        let expected_mutable_reference_parameters = self
            .body
            .signature
            .parameters
            .iter()
            .filter_map(|(name, ty)| {
                matches!(ty, LogicalType::MutableReference { .. }).then_some(name.clone())
            })
            .collect::<BTreeSet<_>>();
        if bound_mutable_reference_parameters != expected_mutable_reference_parameters {
            return Err(self.error(
                0,
                IrVerificationErrorKind::MetadataMismatch(
                    "checked mutable reference parameter binders do not exactly cover the mutable-reference signature"
                        .to_string(),
                ),
            ));
        }
        for id in self.places.keys() {
            if !self.place_definitions.contains_key(id) {
                return Err(self.error(
                    0,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "place {} has metadata but no raw definition",
                        id.0
                    )),
                ));
            }
        }
        for id in self.result_types.keys() {
            if !self.definitions.contains_key(id) {
                return Err(self.error(
                    0,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "result {} has metadata but no raw definition",
                        id.0
                    )),
                ));
            }
        }
        for id in self.definitions.keys() {
            if !self.result_types.contains_key(id) {
                return Err(self.error(
                    0,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "result {} has no logical type",
                        id.0
                    )),
                ));
            }
        }
        for (id, place) in &self.places {
            if place.logical().is_none() {
                let subject = if matches!(place, PlaceType::Array { .. }) {
                    "array place"
                } else {
                    "place"
                };
                return Err(self.error(
                    0,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "{subject} {} has no admitted logical pointee type",
                        id.0,
                    )),
                ));
            }
        }

        let places = self
            .places
            .iter()
            .map(|(id, ty)| {
                (
                    *id,
                    PlaceMetadata {
                        id: *id,
                        name: self.place_names.get(id).cloned().flatten(),
                        pointee: ty
                            .logical()
                            .expect("unresolved place type rejected before metadata publication"),
                    },
                )
            })
            .collect();
        let blocks = self
            .blocks
            .iter()
            .map(|block| BlockMetadata {
                label: block.label.clone(),
                reachable: block.reachable,
                successors: block.successors.clone(),
            })
            .collect();
        Ok(FunctionMetadata {
            signature: self.body.signature.clone(),
            results: self.result_types,
            places,
            blocks,
        })
    }
}

fn validate_program_struct_schemas(ir: &RawIr) -> Result<(), IrVerificationError> {
    fn register_type(
        logical_type: &LogicalType,
        schemas: &mut BTreeMap<String, Vec<LogicalType>>,
        enum_schemas: &mut BTreeMap<String, Vec<EnumVariantSchema>>,
    ) -> Result<(), IrVerificationError> {
        if let LogicalType::Array { element, .. } = logical_type {
            if !valid_copy_data_type(logical_type) {
                return Err(IrVerificationError::new(
                    "<module>",
                    None,
                    IrVerificationErrorKind::UnsupportedType(
                        "checked recursive Copy array schema".to_string(),
                    ),
                ));
            }
            return register_type(element, schemas, enum_schemas);
        }
        if let LogicalType::Tuple { elements } = logical_type {
            if !valid_copy_data_type(logical_type) {
                return Err(IrVerificationError::new(
                    "<module>",
                    None,
                    IrVerificationErrorKind::UnsupportedType(
                        "checked recursive Copy tuple schema".to_string(),
                    ),
                ));
            }
            for element in elements {
                register_type(element, schemas, enum_schemas)?;
            }
            return Ok(());
        }
        if let LogicalType::EnumFields { fields } = logical_type {
            if fields.len() < 2 || !fields.iter().all(valid_copy_data_type) {
                return Err(IrVerificationError::new(
                    "<module>",
                    None,
                    IrVerificationErrorKind::UnsupportedType(
                        "checked enum payload product schema".to_string(),
                    ),
                ));
            }
            for field in fields {
                register_type(field, schemas, enum_schemas)?;
            }
            return Ok(());
        }
        if let LogicalType::Enum { name, variants } = logical_type {
            return register_enum(
                &EnumSchema {
                    name: name.clone(),
                    variants: variants.clone(),
                },
                schemas,
                enum_schemas,
            );
        }
        let LogicalType::Struct { name, fields } = logical_type else {
            return Ok(());
        };
        if !valid_named_struct_schema(name, fields) {
            return Err(IrVerificationError::new(
                "<module>",
                None,
                IrVerificationErrorKind::UnsupportedType(format!("checked struct schema `{name}`")),
            ));
        }
        if enum_schemas.contains_key(name) {
            return Err(IrVerificationError::new(
                "<module>",
                None,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "checked type name `{name}` is used by both a struct and an enum"
                )),
            ));
        }
        if let Some(existing) = schemas.get(name) {
            if existing != fields {
                return Err(IrVerificationError::new(
                    "<module>",
                    None,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "conflicting checked struct schemas for `{name}`"
                    )),
                ));
            }
        } else {
            schemas.insert(name.clone(), fields.clone());
        }
        for field in fields {
            register_type(field, schemas, enum_schemas)?;
        }
        Ok(())
    }

    fn register_enum(
        schema: &EnumSchema,
        schemas: &mut BTreeMap<String, Vec<LogicalType>>,
        enum_schemas: &mut BTreeMap<String, Vec<EnumVariantSchema>>,
    ) -> Result<(), IrVerificationError> {
        if !valid_enum_schema(schema) {
            return Err(IrVerificationError::new(
                "<module>",
                None,
                IrVerificationErrorKind::UnsupportedType(format!(
                    "checked enum schema `{}`",
                    schema.name
                )),
            ));
        }
        if schemas.contains_key(&schema.name) {
            return Err(IrVerificationError::new(
                "<module>",
                None,
                IrVerificationErrorKind::MetadataMismatch(format!(
                    "checked type name `{}` is used by both a struct and an enum",
                    schema.name
                )),
            ));
        }
        if let Some(existing) = enum_schemas.get(&schema.name) {
            if existing != &schema.variants {
                return Err(IrVerificationError::new(
                    "<module>",
                    None,
                    IrVerificationErrorKind::MetadataMismatch(format!(
                        "conflicting checked enum schemas for `{}`",
                        schema.name
                    )),
                ));
            }
        } else {
            enum_schemas.insert(schema.name.clone(), schema.variants.clone());
        }
        for payload in schema
            .variants
            .iter()
            .filter_map(|variant| variant.payload.as_ref())
        {
            register_type(payload, schemas, enum_schemas)?;
        }
        Ok(())
    }

    fn visit(
        instructions: &[Inst],
        schemas: &mut BTreeMap<String, Vec<LogicalType>>,
        enum_schemas: &mut BTreeMap<String, Vec<EnumVariantSchema>>,
    ) -> Result<(), IrVerificationError> {
        for instruction in instructions {
            match instruction {
                Inst::CheckedStructAlloca {
                    struct_name,
                    field_types,
                    ..
                } => {
                    register_type(
                        &LogicalType::Struct {
                            name: struct_name.clone(),
                            fields: field_types.clone(),
                        },
                        schemas,
                        enum_schemas,
                    )?;
                }
                Inst::CheckedCopyStructArrayAlloca { element, .. } => {
                    register_type(element, schemas, enum_schemas)?;
                }
                Inst::CheckedMutableOwnedPlaceAlloca { ty, .. }
                | Inst::CheckedOwnedPlaceAssignment { ty, .. } => {
                    register_type(ty, schemas, enum_schemas)?;
                }
                Inst::CheckedProjectedBorrow {
                    root_type, pointee, ..
                }
                | Inst::CheckedProjectedBorrowEnd {
                    root_type, pointee, ..
                } => {
                    register_type(root_type, schemas, enum_schemas)?;
                    register_type(pointee, schemas, enum_schemas)?;
                }
                Inst::CheckedImmutableEnumOwnerPlaceAlloca { schema, .. }
                | Inst::CheckedImmutableEnumMatchRead { schema, .. }
                | Inst::CheckedMutableEnumMatchRead { schema, .. }
                | Inst::CheckedMutableOwnerImmutableEnumBorrowEnd { schema, .. } => {
                    register_enum(schema, schemas, enum_schemas)?;
                }
                Inst::CheckedTupleAlloca { element_types, .. }
                | Inst::CheckedTupleFieldPtr { element_types, .. } => {
                    register_type(
                        &LogicalType::Tuple {
                            elements: element_types.clone(),
                        },
                        schemas,
                        enum_schemas,
                    )?;
                }
                Inst::CheckedMatchResultPlaceAlloca {
                    result_type,
                    dispatch_schema,
                    ..
                } => {
                    register_type(result_type, schemas, enum_schemas)?;
                    register_enum(dispatch_schema, schemas, enum_schemas)?;
                }
                Inst::CheckedEnumVariant { schema, .. }
                | Inst::CheckedEnumVariantFields { schema, .. }
                | Inst::CheckedEnumPayload { schema, .. }
                | Inst::CheckedEnumField { schema, .. }
                | Inst::CheckedEnumParameter { schema, .. }
                | Inst::CheckedEnumDispatch { schema, .. } => {
                    register_enum(schema, schemas, enum_schemas)?
                }
                Inst::FunctionDef { body, .. } => visit(body, schemas, enum_schemas)?,
                Inst::CheckedFunctionDef {
                    parameters,
                    result,
                    body,
                    ..
                } => {
                    for (_, parameter) in parameters {
                        register_type(parameter, schemas, enum_schemas)?;
                    }
                    register_type(result, schemas, enum_schemas)?;
                    visit(body, schemas, enum_schemas)?;
                }
                _ => {}
            }
        }
        Ok(())
    }

    let mut schemas = BTreeMap::new();
    let mut enum_schemas = BTreeMap::new();
    for function in ir.values() {
        visit(&function.body, &mut schemas, &mut enum_schemas)?;
    }
    Ok(())
}

fn verify_with_seed(
    ir: &RawIr,
    seed: Option<&IrMetadata>,
    place_hints: Option<&PlaceTypeHints>,
    infer_bool_places: bool,
) -> Result<IrMetadata, IrVerificationError> {
    validate_program_struct_schemas(ir)?;
    let (bodies, signatures) = collect_bodies(ir)?;
    let mut metadata = IrMetadata::default();
    for body in &bodies {
        let seeded_function = seed.and_then(|seed| seed.functions.get(&body.name));
        let function_hints = place_hints.and_then(|hints| hints.get(&body.name));
        let verified = FunctionVerifier::new(
            body,
            &signatures,
            seeded_function,
            function_hints,
            infer_bool_places,
        )?
        .verify()?;
        metadata.functions.insert(body.name.clone(), verified);
    }
    if let Some(seed) = seed {
        if metadata != *seed {
            return Err(IrVerificationError::new(
                "<module>",
                None,
                IrVerificationErrorKind::MetadataMismatch(
                    "reverified metadata differs from the checked wrapper".to_string(),
                ),
            ));
        }
    }
    Ok(metadata)
}

pub(crate) fn verify_ir(ir: RawIr) -> Result<CheckedIr, IrVerificationError> {
    let metadata = verify_with_seed(&ir, None, None, true)?;
    Ok(CheckedIr::new(ir, metadata))
}

pub(crate) fn verify_ir_with_place_hints(
    ir: RawIr,
    place_hints: &PlaceTypeHints,
) -> Result<CheckedIr, IrVerificationError> {
    let metadata = verify_with_seed(&ir, None, Some(place_hints), true)?;
    Ok(CheckedIr::new(ir, metadata))
}

pub(crate) fn verify_checked_ir(checked: &CheckedIr) -> Result<IrMetadata, IrVerificationError> {
    if checked.metadata().functions.is_empty() {
        verify_with_seed(checked.raw(), None, None, false)
    } else {
        verify_with_seed(checked.raw(), Some(checked.metadata()), None, true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Function;

    fn unit_schema(name: &str, variants: &[&str]) -> EnumSchema {
        EnumSchema {
            name: name.to_string(),
            variants: variants
                .iter()
                .map(|variant| EnumVariantSchema {
                    name: (*variant).to_string(),
                    payload: None,
                })
                .collect(),
        }
    }

    fn scalar_payload_schema(name: &str) -> EnumSchema {
        EnumSchema {
            name: name.to_string(),
            variants: vec![
                EnumVariantSchema {
                    name: "Idle".to_string(),
                    payload: None,
                },
                EnumVariantSchema {
                    name: "Count".to_string(),
                    payload: Some(LogicalType::Int),
                },
                EnumVariantSchema {
                    name: "Ratio".to_string(),
                    payload: Some(LogicalType::Float),
                },
                EnumVariantSchema {
                    name: "Ready".to_string(),
                    payload: Some(LogicalType::Bool),
                },
            ],
        }
    }

    fn checked_variant(result: Value, schema: EnumSchema, variant_index: usize) -> Inst {
        Inst::CheckedEnumVariant {
            result,
            schema,
            variant_index,
            payload: None,
        }
    }

    fn checked_dispatch(value: Value, schema: EnumSchema, targets: &[&str]) -> Inst {
        Inst::CheckedEnumDispatch {
            value,
            schema,
            targets: targets.iter().map(|target| (*target).to_string()).collect(),
        }
    }

    fn function(body: Vec<Inst>) -> RawIr {
        HashMap::from([(
            "main".to_string(),
            Function {
                name: "main".to_string(),
                body,
                next_reg: 8,
                next_ptr: 8,
            },
        )])
    }

    fn checked_enum_transport_program(
        schema: &EnumSchema,
        forward_body: Vec<Inst>,
        main_runtime: Vec<Inst>,
    ) -> RawIr {
        let enum_type = schema.logical_type();
        let mut main_body = vec![Inst::CheckedFunctionDef {
            name: "forward".to_string(),
            parameters: vec![("value".to_string(), enum_type.clone())],
            result: enum_type,
            body: forward_body,
        }];
        main_body.extend(main_runtime);
        HashMap::from([
            (
                "main".to_string(),
                Function {
                    name: "main".to_string(),
                    body: main_body,
                    next_reg: 8,
                    next_ptr: 8,
                },
            ),
            (
                "forward".to_string(),
                Function {
                    name: "forward".to_string(),
                    body: Vec::new(),
                    next_reg: 8,
                    next_ptr: 8,
                },
            ),
        ])
    }

    #[test]
    fn verifies_a_minimal_typed_numeric_function() {
        let checked = verify_ir(function(vec![Inst::Return(Value::ImmInt(0))])).unwrap();
        assert_eq!(
            checked.metadata().functions["main"].signature.result,
            LogicalType::Int
        );
    }

    #[test]
    fn projected_call_loans_reject_checked_ir_corruption() {
        fn mutate_first(ir: &mut RawIr, mut action: impl FnMut(&mut Inst) -> bool) {
            fn visit(
                instructions: &mut [Inst],
                changed: &mut bool,
                action: &mut impl FnMut(&mut Inst) -> bool,
            ) {
                for instruction in instructions {
                    if !*changed && action(instruction) {
                        *changed = true;
                        return;
                    }
                    match instruction {
                        Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                            visit(body, changed, action);
                            if *changed {
                                return;
                            }
                        }
                        _ => {}
                    }
                }
            }

            let mut changed = false;
            for function in ir.values_mut() {
                visit(&mut function.body, &mut changed, &mut action);
                if changed {
                    break;
                }
            }
            assert!(changed, "requested checked-IR corruption anchor was absent");
        }

        let source = r#"
            struct Cell { value: int }
            fn read_pair(left: &int, right: &int) -> int { *left + *right }
            fn set(value: &mut int, replacement: int) -> int {
                *value = replacement;
                *value
            }
            fn main() -> int {
                let mut cell = Cell { value: 4 };
                let other = Cell { value: 2 };
                let observed = read_pair(&cell.value, &other.value);
                let changed = set(&mut cell.value, observed + 3);
                cell.value + changed
            }
        "#;
        let tokens = crate::lexer::try_tokenize_with_locations(source, None).expect("lex");
        let ast = crate::parser::parse_with_locations(tokens).expect("parse");
        let exact = crate::ir_generator::IrGenerator::new()
            .try_generate_ir(ast)
            .expect("generate and verify exact projected call loans")
            .into_raw();
        verify_ir(exact.clone()).expect("exact projected call loans must reverify");

        let mut cases = Vec::new();

        let mut wrong_root = exact.clone();
        mutate_first(&mut wrong_root, |instruction| {
            if let Inst::CheckedProjectedBorrow { root, source, .. } = instruction {
                *root = source.clone();
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow root identity", wrong_root));

        let mut wrong_source = exact.clone();
        mutate_first(&mut wrong_source, |instruction| {
            if let Inst::CheckedProjectedBorrow { root, source, .. } = instruction {
                *source = root.clone();
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow source identity", wrong_source));

        let mut wrong_root_type = exact.clone();
        mutate_first(&mut wrong_root_type, |instruction| {
            if let Inst::CheckedProjectedBorrow { root_type, .. } = instruction {
                *root_type = LogicalType::Bool;
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow root type", wrong_root_type));

        let mut wrong_pointee = exact.clone();
        mutate_first(&mut wrong_pointee, |instruction| {
            if let Inst::CheckedProjectedBorrow { pointee, .. } = instruction {
                *pointee = LogicalType::Bool;
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow pointee", wrong_pointee));

        let mut wrong_mutability = exact.clone();
        mutate_first(&mut wrong_mutability, |instruction| {
            if let Inst::CheckedProjectedBorrow { mutable, .. } = instruction {
                *mutable = !*mutable;
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow mutability", wrong_mutability));

        let mut raw_call_operand = exact.clone();
        mutate_first(&mut raw_call_operand, |instruction| {
            if let Inst::Call {
                function,
                arguments,
                ..
            } = instruction
                && function == "read_pair"
            {
                arguments[0] = Value::ImmInt(0);
                true
            } else {
                false
            }
        });
        cases.push(("projected reference call operand", raw_call_operand));

        let mut duplicate_call_operand = exact.clone();
        mutate_first(&mut duplicate_call_operand, |instruction| {
            if let Inst::Call {
                function,
                arguments,
                ..
            } = instruction
                && function == "read_pair"
            {
                arguments[1] = arguments[0].clone();
                true
            } else {
                false
            }
        });
        cases.push(("duplicate projected call operand", duplicate_call_operand));

        let mut wrong_end_source = exact.clone();
        mutate_first(&mut wrong_end_source, |instruction| {
            if let Inst::CheckedProjectedBorrowEnd { root, source, .. } = instruction {
                *source = root.clone();
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow end source", wrong_end_source));

        let mut missing_end = exact.clone();
        mutate_first(&mut missing_end, |instruction| {
            if matches!(instruction, Inst::CheckedProjectedBorrowEnd { .. }) {
                *instruction = Inst::Add(Value::Reg(31), Value::ImmInt(1), Value::ImmInt(2));
                true
            } else {
                false
            }
        });
        cases.push(("projected borrow lifecycle", missing_end));

        for (label, candidate) in cases {
            assert!(
                verify_ir(candidate).is_err(),
                "{label} corruption passed checked-IR verification"
            );
        }
    }

    #[test]
    fn private_generic_struct_identity_rejects_checked_schema_corruption() {
        let identity = crate::generic_struct_contract::private_name_for_test(
            "Reading<int>",
            &[
                crate::ast::Type::Named("int".to_string()),
                crate::ast::Type::Named("bool".to_string()),
            ],
        );
        verify_ir(function(vec![
            Inst::CheckedStructAlloca {
                result: Value::Reg(0),
                struct_name: identity.clone(),
                field_types: vec![LogicalType::Int, LogicalType::Bool],
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("exact private generic struct schema must verify");

        let error = verify_ir(function(vec![
            Inst::CheckedStructAlloca {
                result: Value::Reg(0),
                struct_name: identity,
                field_types: vec![LogicalType::Char, LogicalType::Bool],
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("private generic struct schema corruption must fail verification");
        assert!(matches!(
            error.kind,
            IrVerificationErrorKind::UnsupportedType(_)
        ));
    }

    #[test]
    fn private_generic_enum_identity_rejects_checked_schema_corruption() {
        let identity = crate::generic_enum_contract::private_name_for_test(
            "Sample<int>",
            &[
                ("Present", vec![crate::ast::Type::Named("int".to_string())]),
                ("Missing", Vec::new()),
            ],
        );
        let exact = EnumSchema {
            name: identity.clone(),
            variants: vec![
                EnumVariantSchema {
                    name: "Present".to_string(),
                    payload: Some(LogicalType::Int),
                },
                EnumVariantSchema {
                    name: "Missing".to_string(),
                    payload: None,
                },
            ],
        };
        verify_ir(function(vec![
            checked_variant(Value::Reg(0), exact.clone(), 1),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("exact private generic enum schema must verify");

        let mut corrupted = exact.clone();
        corrupted.variants[0].payload = Some(LogicalType::Char);
        let error = verify_ir(function(vec![
            checked_variant(Value::Reg(0), corrupted, 1),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("private generic enum schema corruption must fail verification");
        assert!(matches!(
            error.kind,
            IrVerificationErrorKind::UnsupportedType(_)
        ));

        let mut corrupted_name = exact;
        corrupted_name.name.push_str("00");
        let error = verify_ir(function(vec![
            checked_variant(Value::Reg(0), corrupted_name, 1),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("private generic enum identity corruption must fail verification");
        assert!(matches!(
            error.kind,
            IrVerificationErrorKind::UnsupportedType(_)
        ));
    }

    #[test]
    fn private_generic_function_identity_rejects_signature_corruption() {
        fn specialization(source: &str, canonical: &str) -> String {
            let tokens = crate::lexer::try_tokenize_with_locations(source, None).expect("lex");
            let ast = crate::parser::parse_with_locations(tokens).expect("parse");
            crate::generic_function_contract::normalize_generic_copydata_functions(ast)
                .expect("specialize")
                .into_iter()
                .find_map(|node| match node {
                    crate::ast::AstNode::Statement(crate::ast::Statement::Function {
                        name,
                        ..
                    }) if crate::generic_function_contract::private_generic_function_source_name(
                        &name,
                    )
                    .as_deref() == Some(canonical) =>
                    {
                        Some(name)
                    }
                    _ => None,
                })
                .expect("requested private specialization")
        }

        let scalar_identity = specialization(
            "fn choose<T>(first: T, second: T, take_first: bool) -> T { first } fn main() -> int { choose(1, 2, 1 < 2) }",
            "choose<int>",
        );
        signature(
            &scalar_identity,
            &[
                ("first".to_string(), "int".to_string()),
                ("second".to_string(), "int".to_string()),
                ("take_first".to_string(), "bool".to_string()),
            ],
            &Some("int".to_string()),
        )
        .expect("exact private scalar specialization signature must verify");
        let scalar_error = signature(
            &scalar_identity,
            &[
                ("first".to_string(), "char".to_string()),
                ("second".to_string(), "int".to_string()),
                ("take_first".to_string(), "bool".to_string()),
            ],
            &Some("int".to_string()),
        )
        .expect_err("private scalar substitution corruption must fail verification");
        assert!(matches!(
            scalar_error.kind,
            IrVerificationErrorKind::MetadataMismatch(_)
        ));
        let forged_substitution = crate::generic_function_contract::private_name_for_test(
            "choose<char>",
            "g0.g0.c626f6f6c>g0",
            &[LogicalType::Int, LogicalType::Int, LogicalType::Bool],
            &LogicalType::Int,
        );
        let substitution_error = signature(
            &forged_substitution,
            &[
                ("first".to_string(), "int".to_string()),
                ("second".to_string(), "int".to_string()),
                ("take_first".to_string(), "bool".to_string()),
            ],
            &Some("int".to_string()),
        )
        .expect_err("canonical substitution/signature disagreement must fail verification");
        assert!(matches!(
            substitution_error.kind,
            IrVerificationErrorKind::MetadataMismatch(_)
        ));

        let aggregate = LogicalType::Struct {
            name: "Point".to_string(),
            fields: vec![LogicalType::Int, LogicalType::Int],
        };
        let aggregate_identity = specialization(
            "struct Point { x: int, y: int } fn choose<T>(first: T, second: T, take_first: bool) -> T { first } fn main() -> int { let first: Point = Point { x: 1, y: 2 }; let second: Point = Point { x: 3, y: 4 }; let selected: Point = choose(first, second, 1 < 2); selected.x }",
            "choose<Point>",
        );
        checked_signature(
            &aggregate_identity,
            &[
                ("first".to_string(), aggregate.clone()),
                ("second".to_string(), aggregate.clone()),
                ("take_first".to_string(), LogicalType::Bool),
            ],
            &aggregate,
        )
        .expect("exact private aggregate specialization signature must verify");
        let aggregate_error = checked_signature(
            &aggregate_identity,
            &[
                ("first".to_string(), aggregate.clone()),
                ("second".to_string(), LogicalType::Int),
                ("take_first".to_string(), LogicalType::Bool),
            ],
            &aggregate,
        )
        .expect_err("private aggregate substitution corruption must fail verification");
        assert!(matches!(
            aggregate_error.kind,
            IrVerificationErrorKind::MetadataMismatch(_)
        ));
    }

    #[test]
    fn private_trait_dispatch_rejects_identity_signature_call_and_borrow_corruption() {
        fn visit_mut(instructions: &mut [Inst], action: &mut impl FnMut(&mut Inst)) {
            for instruction in instructions {
                action(instruction);
                match instruction {
                    Inst::FunctionDef { body, .. } | Inst::CheckedFunctionDef { body, .. } => {
                        visit_mut(body, action);
                    }
                    _ => {}
                }
            }
        }

        let source = r#"
            struct Reading { value: int }
            trait Score {
                fn combine(&self, delta: int, enabled: bool) -> int;
            }
            impl Score for Reading {
                fn combine(&self, delta: int, enabled: bool) -> int {
                    if enabled { return (*self).value + delta; }
                    (*self).value
                }
            }
            fn evaluate<T: Score>(value: T, delta: int, enabled: bool) -> int {
                value.combine(delta, enabled)
            }
            fn main() -> int {
                evaluate(Reading { value: 4 }, 3, 1 < 2)
            }
        "#;
        let tokens = crate::lexer::try_tokenize_with_locations(source, None).expect("lex");
        let ast = crate::parser::parse_with_locations(tokens).expect("parse");
        let exact = crate::ir_generator::IrGenerator::new()
            .try_generate_ir(ast)
            .expect("generate and verify exact trait dispatch")
            .into_raw();
        verify_ir(exact.clone()).expect("exact trait dispatch must reverify");

        let helper = exact
            .values()
            .flat_map(|function| function.body.iter())
            .find_map(|instruction| match instruction {
                Inst::CheckedFunctionDef { name, .. }
                    if name
                        .starts_with(crate::copydata_trait_dispatch::PRIVATE_TRAIT_IMPL_PREFIX) =>
                {
                    Some(name.clone())
                }
                _ => None,
            })
            .expect("private trait helper");

        let reading = LogicalType::Struct {
            name: "Reading".to_string(),
            fields: vec![LogicalType::Int],
        };
        let exact_parameters = vec![
            (
                "self".to_string(),
                LogicalType::ImmutableReference {
                    pointee: Box::new(reading.clone()),
                },
            ),
            ("delta".to_string(), LogicalType::Int),
            ("enabled".to_string(), LogicalType::Bool),
        ];
        checked_signature(&helper, &exact_parameters, &LogicalType::Int)
            .expect("exact helper identity and schema");

        let mut wrong_target = exact_parameters.clone();
        wrong_target[0].1 = LogicalType::ImmutableReference {
            pointee: Box::new(LogicalType::Struct {
                name: "Other".to_string(),
                fields: vec![LogicalType::Int],
            }),
        };
        assert!(
            checked_signature(&helper, &wrong_target, &LogicalType::Int).is_err(),
            "encoded impl target accepted a different receiver struct"
        );

        let mut mutable_receiver = exact_parameters.clone();
        mutable_receiver[0].1 = LogicalType::MutableReference {
            pointee: Box::new(reading),
        };
        assert!(
            checked_signature(&helper, &mutable_receiver, &LogicalType::Int).is_err(),
            "private helper accepted mutable receiver corruption"
        );
        assert!(
            checked_signature(&helper, &exact_parameters, &LogicalType::Char).is_err(),
            "private helper accepted result-schema corruption"
        );

        let mut wrong_helper_signature = exact.clone();
        for function in wrong_helper_signature.values_mut() {
            visit_mut(&mut function.body, &mut |instruction| {
                if let Inst::CheckedFunctionDef {
                    name, parameters, ..
                } = instruction
                    && name == &helper
                {
                    parameters[1].1 = LogicalType::Char;
                }
            });
        }
        assert!(
            verify_ir(wrong_helper_signature).is_err(),
            "helper parameter corruption passed independent verification"
        );

        let mut wrong_identity = exact.clone();
        for function in wrong_identity.values_mut() {
            visit_mut(&mut function.body, &mut |instruction| {
                if let Inst::CheckedFunctionDef { name, .. } = instruction
                    && name == &helper
                {
                    name.push('0');
                }
            });
        }
        assert!(
            verify_ir(wrong_identity).is_err(),
            "helper identity corruption passed independent verification"
        );

        let mut wrong_callee = exact.clone();
        for function in wrong_callee.values_mut() {
            visit_mut(&mut function.body, &mut |instruction| {
                if let Inst::Call { function, .. } = instruction
                    && function == &helper
                {
                    function.push('0');
                }
            });
        }
        assert!(
            verify_ir(wrong_callee).is_err(),
            "trait call callee corruption passed independent verification"
        );

        let mut wrong_arity = exact.clone();
        for function in wrong_arity.values_mut() {
            visit_mut(&mut function.body, &mut |instruction| {
                if let Inst::Call {
                    function,
                    arguments,
                    ..
                } = instruction
                    && function == &helper
                {
                    arguments.pop();
                }
            });
        }
        assert!(
            verify_ir(wrong_arity).is_err(),
            "trait call arity corruption passed independent verification"
        );

        let mut wrong_order = exact.clone();
        for function in wrong_order.values_mut() {
            visit_mut(&mut function.body, &mut |instruction| {
                if let Inst::Call {
                    function,
                    arguments,
                    ..
                } = instruction
                    && function == &helper
                {
                    arguments.swap(1, 2);
                }
            });
        }
        assert!(
            verify_ir(wrong_order).is_err(),
            "trait call argument-order corruption passed independent verification"
        );

        let mut wrong_borrow_source = exact;
        for function in wrong_borrow_source.values_mut() {
            visit_mut(&mut function.body, &mut |instruction| {
                if let Inst::CheckedImmutableBorrow { source, .. } = instruction {
                    *source = Value::ImmInt(0);
                }
            });
        }
        assert!(
            verify_ir(wrong_borrow_source).is_err(),
            "trait receiver borrow-provenance corruption passed independent verification"
        );
    }

    #[test]
    fn character_immediates_keep_exact_identity_and_predicates_fail_closed() {
        let checked = verify_ir(function(vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmChar('界'),
                right: Value::ImmChar('界'),
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("exact character equality is valid raw IR");
        assert_eq!(
            checked.metadata().functions["main"].results[&ResultId(0)],
            LogicalType::Bool
        );

        let ordering = verify_ir(function(vec![
            Inst::ICmp {
                op: "slt".to_string(),
                result: Value::Reg(0),
                left: Value::ImmChar('A'),
                right: Value::ImmChar('B'),
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("character ordering must fail verification");
        assert!(matches!(
            ordering.kind,
            IrVerificationErrorKind::InvalidPredicate {
                operation: "icmp",
                predicate
            } if predicate == "slt"
        ));

        let substituted = verify_ir(function(vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(0),
                left: Value::ImmChar('A'),
                right: Value::ImmInt(65),
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("integer immediate must not substitute for character identity");
        assert!(matches!(
            substituted.kind,
            IrVerificationErrorKind::TypeMismatch {
                operation: "icmp",
                role: "operand",
                actual: LogicalType::Int,
                ..
            }
        ));
    }

    #[test]
    fn checked_character_places_reject_integer_immediate_substitution() {
        let error = verify_ir(function(vec![
            Inst::CheckedMutableOwnedPlaceAlloca {
                result: Value::Reg(0),
                name: "character".to_string(),
                ty: LogicalType::Char,
            },
            Inst::Store(Value::Reg(0), Value::ImmChar('A')),
            Inst::CheckedOwnedPlaceAssignment {
                target: Value::Reg(0),
                value: Value::ImmInt(65),
                ty: LogicalType::Char,
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("checked character place must reject integer substitution");
        assert!(matches!(
            error.kind,
            IrVerificationErrorKind::TypeMismatch {
                actual: LogicalType::Int,
                ..
            }
        ));
    }

    #[test]
    fn checked_enum_transport_signatures_and_binders_are_fail_closed() {
        let schema = unit_schema("Phase", &["Cold", "Warm"]);
        let enum_type = schema.logical_type();
        let binder = || Inst::CheckedEnumParameter {
            result: Value::Reg(0),
            parameter: "value".to_string(),
            schema: schema.clone(),
        };
        let valid_main = vec![
            checked_variant(Value::Reg(0), schema.clone(), 1),
            Inst::Call {
                function: "forward".to_string(),
                arguments: vec![Value::Reg(0)],
                result: Some(Value::Reg(1)),
            },
            checked_dispatch(Value::Reg(1), schema.clone(), &["cold", "warm"]),
            Inst::Label("cold".to_string()),
            Inst::Return(Value::ImmInt(0)),
            Inst::Label("warm".to_string()),
            Inst::Return(Value::ImmInt(1)),
        ];
        let checked = verify_ir(checked_enum_transport_program(
            &schema,
            vec![binder(), Inst::Return(Value::Reg(0))],
            valid_main,
        ))
        .expect("exact enum signature, direct binder, call, and return are valid");
        assert_eq!(
            checked.metadata().functions["forward"].signature.parameters,
            vec![("value".to_string(), enum_type.clone())]
        );
        assert_eq!(
            checked.metadata().functions["forward"].signature.result,
            enum_type.clone()
        );
        assert_eq!(
            checked.metadata().functions["main"].results[&ResultId(1)],
            enum_type
        );

        let invalid_forward_bodies = [
            (
                "missing direct binder",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
            (
                "alloca binder",
                vec![
                    Inst::Alloca(Value::Reg(0), "value".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "wrong binder parameter",
                vec![
                    Inst::CheckedEnumParameter {
                        result: Value::Reg(0),
                        parameter: "other".to_string(),
                        schema: schema.clone(),
                    },
                    Inst::Return(Value::Reg(0)),
                ],
            ),
            (
                "wrong binder schema",
                vec![
                    Inst::CheckedEnumParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        schema: unit_schema("Phase", &["Cold", "Hot"]),
                    },
                    Inst::Return(Value::Reg(0)),
                ],
            ),
            (
                "duplicate binder",
                vec![
                    binder(),
                    Inst::CheckedEnumParameter {
                        result: Value::Reg(1),
                        parameter: "value".to_string(),
                        schema: schema.clone(),
                    },
                    Inst::Return(Value::Reg(1)),
                ],
            ),
            (
                "binder outside entry block",
                vec![
                    Inst::Jump("later".to_string()),
                    Inst::Label("later".to_string()),
                    binder(),
                    Inst::Return(Value::Reg(0)),
                ],
            ),
            (
                "wrong enum return",
                vec![
                    binder(),
                    checked_variant(Value::Reg(1), unit_schema("Other", &["Cold", "Warm"]), 0),
                    Inst::Return(Value::Reg(1)),
                ],
            ),
        ];
        for (label, body) in invalid_forward_bodies {
            assert!(
                verify_ir(checked_enum_transport_program(
                    &schema,
                    body,
                    vec![Inst::Return(Value::ImmInt(0))]
                ))
                .is_err(),
                "{label} passed checked IR verification"
            );
        }

        let wrong_call = vec![
            checked_variant(Value::Reg(0), unit_schema("Other", &["Cold", "Warm"]), 0),
            Inst::Call {
                function: "forward".to_string(),
                arguments: vec![Value::Reg(0)],
                result: Some(Value::Reg(1)),
            },
            Inst::Return(Value::ImmInt(0)),
        ];
        assert!(
            verify_ir(checked_enum_transport_program(
                &schema,
                vec![binder(), Inst::Return(Value::Reg(0))],
                wrong_call,
            ))
            .is_err(),
            "wrong enum call argument passed checked IR verification"
        );

        let payload_schema = scalar_payload_schema("Signal");
        let payload_type = payload_schema.logical_type();
        let payload_binder = || Inst::CheckedEnumParameter {
            result: Value::Reg(0),
            parameter: "value".to_string(),
            schema: payload_schema.clone(),
        };
        let payload_main = vec![
            Inst::CheckedEnumVariant {
                result: Value::Reg(0),
                schema: payload_schema.clone(),
                variant_index: 1,
                payload: Some(Value::ImmInt(7)),
            },
            Inst::Call {
                function: "forward".to_string(),
                arguments: vec![Value::Reg(0)],
                result: Some(Value::Reg(1)),
            },
            checked_dispatch(
                Value::Reg(1),
                payload_schema.clone(),
                &["idle", "count", "ratio", "ready"],
            ),
            Inst::Label("idle".to_string()),
            Inst::Return(Value::ImmInt(0)),
            Inst::Label("count".to_string()),
            Inst::Return(Value::ImmInt(1)),
            Inst::Label("ratio".to_string()),
            Inst::Return(Value::ImmInt(2)),
            Inst::Label("ready".to_string()),
            Inst::Return(Value::ImmInt(3)),
        ];
        let checked = verify_ir(checked_enum_transport_program(
            &payload_schema,
            vec![payload_binder(), Inst::Return(Value::Reg(0))],
            payload_main,
        ))
        .expect("exact payload-enum binder, call, return, and dispatch are valid");
        assert_eq!(
            checked.metadata().functions["forward"].signature.parameters,
            vec![("value".to_string(), payload_type.clone())]
        );
        assert_eq!(
            checked.metadata().functions["main"].results[&ResultId(1)],
            payload_type
        );

        for (label, body) in [
            (
                "payload binder replaced by unit schema",
                vec![
                    Inst::CheckedEnumParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        schema: unit_schema("Signal", &["Idle", "Count", "Ratio", "Ready"]),
                    },
                    Inst::Return(Value::Reg(0)),
                ],
            ),
            (
                "payload binder carries unsupported lane type",
                vec![
                    Inst::CheckedEnumParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        schema: EnumSchema {
                            name: "Signal".to_string(),
                            variants: vec![EnumVariantSchema {
                                name: "Text".to_string(),
                                payload: Some(LogicalType::String),
                            }],
                        },
                    },
                    Inst::Return(Value::Reg(0)),
                ],
            ),
            (
                "payload return changes schema",
                vec![
                    payload_binder(),
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(1),
                        schema: scalar_payload_schema("Other"),
                        variant_index: 1,
                        payload: Some(Value::ImmInt(7)),
                    },
                    Inst::Return(Value::Reg(1)),
                ],
            ),
        ] {
            assert!(
                verify_ir(checked_enum_transport_program(
                    &payload_schema,
                    body,
                    vec![Inst::Return(Value::ImmInt(0))],
                ))
                .is_err(),
                "{label} passed checked IR verification"
            );
        }

        let wrong_payload_call = vec![
            Inst::CheckedEnumVariant {
                result: Value::Reg(0),
                schema: scalar_payload_schema("Other"),
                variant_index: 1,
                payload: Some(Value::ImmInt(7)),
            },
            Inst::Call {
                function: "forward".to_string(),
                arguments: vec![Value::Reg(0)],
                result: Some(Value::Reg(1)),
            },
            Inst::Return(Value::ImmInt(0)),
        ];
        assert!(
            verify_ir(checked_enum_transport_program(
                &payload_schema,
                vec![payload_binder(), Inst::Return(Value::Reg(0))],
                wrong_payload_call,
            ))
            .is_err(),
            "wrong payload-enum call argument passed checked IR verification"
        );
    }

    #[test]
    fn checked_unit_enum_identity_and_exhaustive_dispatch_are_fail_closed() {
        let schema = unit_schema("Phase", &["Cold", "Warm"]);
        let checked = verify_ir(function(vec![
            checked_variant(Value::Reg(0), schema.clone(), 1),
            Inst::Alloca(Value::Reg(1), "result".to_string()),
            checked_dispatch(Value::Reg(0), schema.clone(), &["cold", "warm"]),
            Inst::Label("cold".to_string()),
            Inst::Store(Value::Reg(1), Value::ImmInt(11)),
            Inst::Jump("end".to_string()),
            Inst::Label("warm".to_string()),
            Inst::Store(Value::Reg(1), Value::ImmInt(22)),
            Inst::Jump("end".to_string()),
            Inst::Label("end".to_string()),
            Inst::Load(Value::Reg(2), Value::Reg(1)),
            Inst::Return(Value::Reg(2)),
        ]))
        .expect("exact unit-enum identity and exhaustive dispatch are valid");
        let metadata = &checked.metadata().functions["main"];
        assert_eq!(metadata.results[&ResultId(0)], schema.logical_type());
        assert_eq!(metadata.results[&ResultId(2)], LogicalType::Int);
        assert_eq!(
            metadata.blocks[0].successors,
            vec!["cold".to_string(), "warm".to_string()]
        );

        let invalid_cases = [
            (
                "immediate constructor result",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::ImmInt(0),
                        schema: schema.clone(),
                        variant_index: 0,
                        payload: None,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "empty schema",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: EnumSchema {
                            name: "Phase".to_string(),
                            variants: vec![],
                        },
                        variant_index: 0,
                        payload: None,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "duplicate variant schema",
                vec![
                    checked_variant(Value::Reg(0), unit_schema("Phase", &["Cold", "Cold"]), 0),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "out of range variant",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 2),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "immediate dispatch value",
                vec![
                    checked_dispatch(Value::ImmInt(0), schema.clone(), &["cold", "warm"]),
                    Inst::Label("cold".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("warm".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "undefined dispatch value",
                vec![
                    checked_dispatch(Value::Reg(7), schema.clone(), &["cold", "warm"]),
                    Inst::Label("cold".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("warm".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "dispatch schema mismatch",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    checked_dispatch(
                        Value::Reg(0),
                        unit_schema("Other", &["Cold", "Warm"]),
                        &["cold", "warm"],
                    ),
                    Inst::Label("cold".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("warm".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "incomplete dispatch",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    checked_dispatch(Value::Reg(0), schema.clone(), &["cold"]),
                    Inst::Label("cold".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "duplicate dispatch targets",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    checked_dispatch(Value::Reg(0), schema.clone(), &["cold", "cold"]),
                    Inst::Label("cold".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "missing dispatch target",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    checked_dispatch(Value::Reg(0), schema.clone(), &["cold", "missing"]),
                    Inst::Label("cold".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "conflicting enum schema",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    checked_variant(Value::Reg(1), unit_schema("Phase", &["Cold", "Hot"]), 1),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "struct enum type collision",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(2),
                        struct_name: "Phase".to_string(),
                        field_types: vec![LogicalType::Int],
                    },
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "result place collision",
                vec![
                    Inst::Alloca(Value::Reg(0), "slot".to_string()),
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];

        for (label, body) in invalid_cases {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn checked_scalar_payload_enum_construction_dispatch_and_extraction_are_fail_closed() {
        let schema = scalar_payload_schema("Signal");
        let valid = verify_ir(function(vec![
            Inst::CheckedEnumVariant {
                result: Value::Reg(0),
                schema: schema.clone(),
                variant_index: 1,
                payload: Some(Value::ImmInt(41)),
            },
            checked_dispatch(
                Value::Reg(0),
                schema.clone(),
                &["idle", "count", "ratio", "ready"],
            ),
            Inst::Label("idle".to_string()),
            Inst::Return(Value::ImmInt(0)),
            Inst::Label("count".to_string()),
            Inst::CheckedEnumPayload {
                result: Value::Reg(1),
                value: Value::Reg(0),
                schema: schema.clone(),
                variant_index: 1,
            },
            Inst::Return(Value::Reg(1)),
            Inst::Label("ratio".to_string()),
            Inst::Return(Value::ImmInt(0)),
            Inst::Label("ready".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("exact scalar-payload construction, guard, and extraction are valid");
        assert_eq!(
            valid.metadata().functions["main"].results[&ResultId(0)],
            schema.logical_type()
        );
        assert_eq!(
            valid.metadata().functions["main"].results[&ResultId(1)],
            LogicalType::Int
        );

        let invalid_cases = [
            (
                "missing constructor payload",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 1),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "constructor payload on unit variant",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 0,
                        payload: Some(Value::ImmInt(1)),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "constructor payload exact-type mismatch",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 1,
                        payload: Some(Value::ImmFloat(1.0)),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "payload extraction from a unit variant",
                vec![
                    checked_variant(Value::Reg(0), schema.clone(), 0),
                    checked_dispatch(
                        Value::Reg(0),
                        schema.clone(),
                        &["idle", "count", "ratio", "ready"],
                    ),
                    Inst::Label("idle".to_string()),
                    Inst::CheckedEnumPayload {
                        result: Value::Reg(1),
                        value: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 0,
                    },
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("count".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("ratio".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("ready".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "payload extraction under the wrong variant target",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 1,
                        payload: Some(Value::ImmInt(1)),
                    },
                    checked_dispatch(
                        Value::Reg(0),
                        schema.clone(),
                        &["idle", "count", "ratio", "ready"],
                    ),
                    Inst::Label("idle".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("count".to_string()),
                    Inst::CheckedEnumPayload {
                        result: Value::Reg(1),
                        value: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 2,
                    },
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("ratio".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("ready".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "payload extraction without dispatch guard",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 1,
                        payload: Some(Value::ImmInt(1)),
                    },
                    Inst::CheckedEnumPayload {
                        result: Value::Reg(1),
                        value: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 1,
                    },
                    Inst::Return(Value::Reg(1)),
                ],
            ),
            (
                "unsupported payload metadata",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: EnumSchema {
                            name: "Signal".to_string(),
                            variants: vec![EnumVariantSchema {
                                name: "Text".to_string(),
                                payload: Some(LogicalType::String),
                            }],
                        },
                        variant_index: 0,
                        payload: Some(Value::ImmString("x".to_string())),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];

        for (label, body) in invalid_cases {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed checked IR verification"
            );
        }

        let payload_transport = schema.logical_type();
        let transport_error = verify_ir(function(vec![
            Inst::CheckedFunctionDef {
                name: "forward".to_string(),
                parameters: vec![("value".to_string(), payload_transport.clone())],
                result: payload_transport,
                body: vec![Inst::Return(Value::ImmInt(0))],
            },
            Inst::Return(Value::ImmInt(0)),
        ]));
        assert!(
            transport_error.is_err(),
            "payload enum function transport passed checked IR verification"
        );
    }

    #[test]
    fn checked_multi_field_enum_construction_and_extraction_are_fail_closed() {
        let schema = EnumSchema {
            name: "Packet".to_string(),
            variants: vec![
                EnumVariantSchema {
                    name: "Idle".to_string(),
                    payload: None,
                },
                EnumVariantSchema {
                    name: "Pair".to_string(),
                    payload: Some(LogicalType::EnumFields {
                        fields: vec![LogicalType::Int, LogicalType::Bool],
                    }),
                },
            ],
        };
        let boolean = Inst::ICmp {
            op: "slt".to_string(),
            result: Value::Reg(0),
            left: Value::ImmInt(0),
            right: Value::ImmInt(1),
        };
        let constructor = Inst::CheckedEnumVariantFields {
            result: Value::Reg(1),
            schema: schema.clone(),
            variant_index: 1,
            fields: vec![Value::ImmInt(41), Value::Reg(0)],
        };
        let valid = verify_ir(function(vec![
            boolean.clone(),
            constructor.clone(),
            checked_dispatch(Value::Reg(1), schema.clone(), &["idle", "pair"]),
            Inst::Label("idle".to_string()),
            Inst::Return(Value::ImmInt(0)),
            Inst::Label("pair".to_string()),
            Inst::CheckedEnumField {
                result: Value::Reg(2),
                value: Value::Reg(1),
                schema: schema.clone(),
                variant_index: 1,
                field_index: 0,
            },
            Inst::CheckedEnumField {
                result: Value::Reg(3),
                value: Value::Reg(1),
                schema: schema.clone(),
                variant_index: 1,
                field_index: 1,
            },
            Inst::Return(Value::Reg(2)),
        ]))
        .expect("exact multi-field construction, guard, and ordered extraction are valid");
        assert_eq!(
            valid.metadata().functions["main"].results[&ResultId(1)],
            schema.logical_type()
        );
        assert_eq!(
            valid.metadata().functions["main"].results[&ResultId(2)],
            LogicalType::Int
        );
        assert_eq!(
            valid.metadata().functions["main"].results[&ResultId(3)],
            LogicalType::Bool
        );

        let invalid_cases = [
            (
                "multi-field constructor omits a field",
                vec![
                    Inst::CheckedEnumVariantFields {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 1,
                        fields: vec![Value::ImmInt(41)],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "multi-field constructor changes ordered field type",
                vec![
                    boolean.clone(),
                    Inst::CheckedEnumVariantFields {
                        result: Value::Reg(1),
                        schema: schema.clone(),
                        variant_index: 1,
                        fields: vec![Value::Reg(0), Value::ImmInt(41)],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "multi-field constructor names unit variant",
                vec![
                    Inst::CheckedEnumVariantFields {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 0,
                        fields: vec![],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "unary identity constructs a multi-field product",
                vec![
                    Inst::CheckedEnumVariant {
                        result: Value::Reg(0),
                        schema: schema.clone(),
                        variant_index: 1,
                        payload: Some(Value::ImmInt(41)),
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "whole-payload identity extracts a multi-field product",
                vec![
                    boolean.clone(),
                    constructor.clone(),
                    checked_dispatch(Value::Reg(1), schema.clone(), &["idle", "pair"]),
                    Inst::Label("idle".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("pair".to_string()),
                    Inst::CheckedEnumPayload {
                        result: Value::Reg(2),
                        value: Value::Reg(1),
                        schema: schema.clone(),
                        variant_index: 1,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "field extraction is outside product bounds",
                vec![
                    boolean.clone(),
                    constructor.clone(),
                    checked_dispatch(Value::Reg(1), schema.clone(), &["idle", "pair"]),
                    Inst::Label("idle".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("pair".to_string()),
                    Inst::CheckedEnumField {
                        result: Value::Reg(2),
                        value: Value::Reg(1),
                        schema: schema.clone(),
                        variant_index: 1,
                        field_index: 2,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "field extraction lacks a dispatch guard",
                vec![
                    boolean.clone(),
                    constructor.clone(),
                    Inst::CheckedEnumField {
                        result: Value::Reg(2),
                        value: Value::Reg(1),
                        schema: schema.clone(),
                        variant_index: 1,
                        field_index: 0,
                    },
                    Inst::Return(Value::Reg(2)),
                ],
            ),
            (
                "multi-field schema nests a private payload product",
                vec![
                    Inst::CheckedEnumVariantFields {
                        result: Value::Reg(0),
                        schema: EnumSchema {
                            name: "Nested".to_string(),
                            variants: vec![EnumVariantSchema {
                                name: "Bad".to_string(),
                                payload: Some(LogicalType::EnumFields {
                                    fields: vec![
                                        LogicalType::Int,
                                        LogicalType::EnumFields {
                                            fields: vec![LogicalType::Int, LogicalType::Bool],
                                        },
                                    ],
                                }),
                            }],
                        },
                        variant_index: 0,
                        fields: vec![Value::ImmInt(1), Value::ImmInt(2)],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];

        for (label, body) in invalid_cases {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn checked_recursive_copydata_enum_schemas_and_payload_identity_are_fail_closed() {
        let schema = EnumSchema {
            name: "Payload".to_string(),
            variants: vec![
                EnumVariantSchema {
                    name: "Idle".to_string(),
                    payload: None,
                },
                EnumVariantSchema {
                    name: "Flags".to_string(),
                    payload: Some(LogicalType::Array {
                        element: Box::new(LogicalType::Bool),
                        count: 2,
                    }),
                },
                EnumVariantSchema {
                    name: "Pair".to_string(),
                    payload: Some(LogicalType::Tuple {
                        elements: vec![LogicalType::Int, LogicalType::Bool],
                    }),
                },
                EnumVariantSchema {
                    name: "Row".to_string(),
                    payload: Some(LogicalType::Struct {
                        name: "Row".to_string(),
                        fields: vec![LogicalType::Int, LogicalType::Bool],
                    }),
                },
            ],
        };
        let checked = verify_ir(function(vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("finite recursive CopyData enum schemas are valid");
        assert_eq!(
            checked.metadata().functions["main"].results[&ResultId(0)],
            schema.logical_type()
        );

        let invalid_schemas = [
            (
                "nested String leaf",
                EnumSchema {
                    name: "BadString".to_string(),
                    variants: vec![EnumVariantSchema {
                        name: "Value".to_string(),
                        payload: Some(LogicalType::Array {
                            element: Box::new(LogicalType::Tuple {
                                elements: vec![LogicalType::Int, LogicalType::String],
                            }),
                            count: 1,
                        }),
                    }],
                },
            ),
            (
                "nested reference leaf",
                EnumSchema {
                    name: "BadReference".to_string(),
                    variants: vec![EnumVariantSchema {
                        name: "Value".to_string(),
                        payload: Some(LogicalType::Tuple {
                            elements: vec![
                                LogicalType::Int,
                                LogicalType::ImmutableReference {
                                    pointee: Box::new(LogicalType::Int),
                                },
                            ],
                        }),
                    }],
                },
            ),
            (
                "nested enum leaf",
                EnumSchema {
                    name: "BadEnum".to_string(),
                    variants: vec![EnumVariantSchema {
                        name: "Value".to_string(),
                        payload: Some(unit_schema("Inner", &["Unit"]).logical_type()),
                    }],
                },
            ),
            (
                "conflicting named struct payload schemas",
                EnumSchema {
                    name: "BadStructIdentity".to_string(),
                    variants: vec![
                        EnumVariantSchema {
                            name: "Left".to_string(),
                            payload: Some(LogicalType::Struct {
                                name: "Row".to_string(),
                                fields: vec![LogicalType::Int],
                            }),
                        },
                        EnumVariantSchema {
                            name: "Right".to_string(),
                            payload: Some(LogicalType::Struct {
                                name: "Row".to_string(),
                                fields: vec![LogicalType::Bool],
                            }),
                        },
                    ],
                },
            ),
        ];
        for (label, invalid) in invalid_schemas {
            assert!(
                verify_ir(function(vec![
                    checked_variant(Value::Reg(0), invalid, 0),
                    Inst::Return(Value::ImmInt(0)),
                ]))
                .is_err(),
                "{label} passed checked IR verification"
            );
        }

        assert!(
            verify_ir(function(vec![
                Inst::CheckedEnumVariant {
                    result: Value::Reg(0),
                    schema: schema.clone(),
                    variant_index: 1,
                    payload: Some(Value::ImmInt(1)),
                },
                Inst::Return(Value::ImmInt(0)),
            ]))
            .is_err(),
            "scalar fallback value passed as an aggregate enum payload"
        );

        let mut changed_schema = schema.clone();
        changed_schema.variants[1].payload = Some(LogicalType::Array {
            element: Box::new(LogicalType::Bool),
            count: 3,
        });
        assert!(
            verify_ir(function(vec![
                checked_variant(Value::Reg(0), schema, 0),
                checked_dispatch(
                    Value::Reg(0),
                    changed_schema,
                    &["idle", "flags", "pair", "row"],
                ),
                Inst::Label("idle".to_string()),
                Inst::Return(Value::ImmInt(0)),
                Inst::Label("flags".to_string()),
                Inst::Return(Value::ImmInt(0)),
                Inst::Label("pair".to_string()),
                Inst::Return(Value::ImmInt(0)),
                Inst::Label("row".to_string()),
                Inst::Return(Value::ImmInt(0)),
            ]))
            .is_err(),
            "changed aggregate lane schema passed checked IR dispatch"
        );
    }

    #[test]
    fn immutable_scalar_borrow_place_integrity_is_fail_closed() {
        let checked = verify_ir(function(vec![
            Inst::Alloca(Value::Reg(0), "owner".to_string()),
            Inst::Store(Value::Reg(0), Value::ImmInt(7)),
            Inst::CheckedImmutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(0),
                pointee: LogicalType::Int,
            },
            Inst::Load(Value::Reg(2), Value::Reg(1)),
            Inst::Return(Value::Reg(2)),
        ]))
        .expect("an exact dominating immutable scalar borrow is valid");
        let metadata = &checked.metadata().functions["main"];
        assert_eq!(metadata.places[&PlaceId(0)].pointee, LogicalType::Int);
        assert_eq!(metadata.places[&PlaceId(1)].pointee, LogicalType::Int);
        assert_eq!(metadata.results[&ResultId(2)], LogicalType::Int);

        let invalid_cases = [
            (
                "immediate alias identifier",
                vec![
                    Inst::Alloca(Value::Reg(0), "owner".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmInt(7)),
                    Inst::CheckedImmutableBorrow {
                        result: Value::ImmInt(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "immediate source identifier",
                vec![
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::ImmInt(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "undefined source place",
                vec![
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(7),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "exact pointee mismatch",
                vec![
                    Inst::Alloca(Value::Reg(0), "owner".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmFloat(1.5)),
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "unsupported pointee metadata",
                vec![
                    Inst::Alloca(Value::Reg(0), "owner".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmString("aero".to_string())),
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::String,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "duplicate alias definition",
                vec![
                    Inst::Alloca(Value::Reg(0), "owner".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmInt(7)),
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "result/place kind collision",
                vec![
                    Inst::Alloca(Value::Reg(0), "owner".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmInt(7)),
                    Inst::Add(Value::Reg(1), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];
        for (label, body) in invalid_cases {
            let error = verify_ir(function(body)).expect_err(label).to_string();
            assert!(error.contains("IR Verification Error"), "{label}: {error}");
        }

        let non_dominating = verify_ir(function(vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(3),
                left: Value::ImmInt(1),
                right: Value::ImmInt(1),
            },
            Inst::Branch {
                condition: Value::Reg(3),
                true_label: "define".to_string(),
                false_label: "use".to_string(),
            },
            Inst::Label("define".to_string()),
            Inst::Alloca(Value::Reg(0), "owner".to_string()),
            Inst::Store(Value::Reg(0), Value::ImmInt(7)),
            Inst::Jump("use".to_string()),
            Inst::Label("use".to_string()),
            Inst::CheckedImmutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(0),
                pointee: LogicalType::Int,
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("borrow source must dominate alias creation");
        assert!(matches!(
            non_dominating.kind,
            IrVerificationErrorKind::PlaceDoesNotDominateUse(PlaceId(0))
        ));
    }

    #[test]
    fn immutable_copy_place_reference_schema_integrity_is_fail_closed() {
        let row = LogicalType::Struct {
            name: "Row".to_string(),
            fields: vec![LogicalType::Int, LogicalType::Bool],
        };
        let checked = verify_ir(function(vec![
            Inst::CheckedStructAlloca {
                result: Value::Reg(0),
                struct_name: "Row".to_string(),
                field_types: vec![LogicalType::Int, LogicalType::Bool],
            },
            Inst::CheckedImmutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(0),
                pointee: row.clone(),
            },
            Inst::Load(Value::Reg(2), Value::Reg(1)),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("an exact immutable Copy-struct borrow is valid");
        assert_eq!(
            checked.metadata().functions["main"].places[&PlaceId(1)].pointee,
            row
        );

        let wrong_source_schema = verify_ir(function(vec![
            Inst::CheckedStructAlloca {
                result: Value::Reg(0),
                struct_name: "Row".to_string(),
                field_types: vec![LogicalType::Int, LogicalType::Bool],
            },
            Inst::CheckedImmutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(0),
                pointee: LogicalType::Struct {
                    name: "Other".to_string(),
                    fields: vec![LogicalType::Int, LogicalType::Bool],
                },
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect_err("aggregate borrow metadata must equal its source place schema");
        assert!(
            wrong_source_schema
                .to_string()
                .contains("disagrees with source place")
        );

        let reference = LogicalType::ImmutableReference {
            pointee: Box::new(row.clone()),
        };
        let definition = Inst::CheckedFunctionDef {
            name: "copy".to_string(),
            parameters: vec![("value".to_string(), reference)],
            result: row.clone(),
            body: vec![
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: row.clone(),
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Return(Value::Reg(1)),
            ],
        };
        let program = HashMap::from([
            (
                "main".to_string(),
                Function {
                    name: "main".to_string(),
                    body: vec![definition.clone(), Inst::Return(Value::ImmInt(0))],
                    next_reg: 8,
                    next_ptr: 8,
                },
            ),
            (
                "copy".to_string(),
                Function {
                    name: "copy".to_string(),
                    body: Vec::new(),
                    next_reg: 8,
                    next_ptr: 8,
                },
            ),
        ]);
        verify_ir(program).expect("exact aggregate reference parameter schema is valid");

        let mut wrong_binder = definition;
        let Inst::CheckedFunctionDef { body, .. } = &mut wrong_binder else {
            unreachable!("fixture retains checked function definition")
        };
        let Inst::CheckedImmutableReferenceParameter { pointee, .. } = &mut body[0] else {
            unreachable!("fixture retains immutable reference binder")
        };
        *pointee = LogicalType::Struct {
            name: "Row".to_string(),
            fields: vec![LogicalType::Float, LogicalType::Bool],
        };
        let corrupt = HashMap::from([
            (
                "main".to_string(),
                Function {
                    name: "main".to_string(),
                    body: vec![wrong_binder, Inst::Return(Value::ImmInt(0))],
                    next_reg: 8,
                    next_ptr: 8,
                },
            ),
            (
                "copy".to_string(),
                Function {
                    name: "copy".to_string(),
                    body: Vec::new(),
                    next_reg: 8,
                    next_ptr: 8,
                },
            ),
        ]);
        assert!(
            verify_ir(corrupt).is_err(),
            "aggregate reference binder schema corruption passed verification"
        );
    }

    #[test]
    fn immutable_reference_parameter_signatures_binders_and_calls_are_fail_closed() {
        let reference = LogicalType::ImmutableReference {
            pointee: Box::new(LogicalType::Int),
        };
        let reader = |body| Inst::CheckedFunctionDef {
            name: "read".to_string(),
            parameters: vec![("value".to_string(), reference.clone())],
            result: LogicalType::Int,
            body,
        };
        let program = |definition: Inst, mut main_runtime: Vec<Inst>| {
            let name = match &definition {
                Inst::CheckedFunctionDef { name, .. } => name.clone(),
                _ => panic!("reference verifier fixture requires a checked definition"),
            };
            let mut main_body = vec![definition];
            main_body.append(&mut main_runtime);
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: main_body,
                        next_reg: 8,
                        next_ptr: 8,
                    },
                ),
                (
                    name.clone(),
                    Function {
                        name,
                        body: Vec::new(),
                        next_reg: 8,
                        next_ptr: 8,
                    },
                ),
            ])
        };
        let checked = verify_ir(program(
            reader(vec![
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Return(Value::Reg(1)),
            ]),
            vec![
                Inst::Alloca(Value::Reg(0), "owner".to_string()),
                Inst::Store(Value::Reg(0), Value::ImmInt(7)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                    pointee: LogicalType::Int,
                },
                Inst::Call {
                    function: "read".to_string(),
                    arguments: vec![Value::Reg(1)],
                    result: Some(Value::Reg(2)),
                },
                Inst::Return(Value::Reg(2)),
            ],
        ))
        .expect("exact reference signature, binder, place call, load, and return are valid");
        let read = &checked.metadata().functions["read"];
        assert_eq!(read.signature.parameters[0].1, reference);
        assert_eq!(read.places[&PlaceId(0)].pointee, LogicalType::Int);

        let invalid_readers = [
            (
                "missing binder",
                reader(vec![Inst::Return(Value::ImmInt(0))]),
            ),
            (
                "duplicate binder",
                reader(vec![
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(1),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ]),
            ),
            (
                "wrong binder name",
                reader(vec![
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "other".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ]),
            ),
            (
                "wrong binder pointee",
                reader(vec![
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Float,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ]),
            ),
            (
                "scalar alloca binder",
                reader(vec![
                    Inst::Alloca(Value::Reg(0), "value".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ]),
            ),
            (
                "binder result collision",
                reader(vec![
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ]),
            ),
            (
                "misplaced binder",
                reader(vec![
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(0),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Branch {
                        condition: Value::Reg(0),
                        true_label: "bind".to_string(),
                        false_label: "done".to_string(),
                    },
                    Inst::Label("bind".to_string()),
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(1),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("done".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ]),
            ),
        ];
        for (label, definition) in invalid_readers {
            assert!(
                verify_ir(program(definition, vec![Inst::Return(Value::ImmInt(0))])).is_err(),
                "{label} passed checked IR verification"
            );
        }

        let scalar_call_argument = verify_ir(program(
            reader(vec![
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Return(Value::Reg(1)),
            ]),
            vec![
                Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                Inst::Call {
                    function: "read".to_string(),
                    arguments: vec![Value::Reg(0)],
                    result: Some(Value::Reg(1)),
                },
                Inst::Return(Value::Reg(1)),
            ],
        ));
        assert!(
            scalar_call_argument.is_err(),
            "scalar result passed as a reference place"
        );

        let wrong_place_pointee = verify_ir(program(
            reader(vec![
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Return(Value::Reg(1)),
            ]),
            vec![
                Inst::Alloca(Value::Reg(0), "owner".to_string()),
                Inst::Store(Value::Reg(0), Value::ImmFloat(1.5)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                    pointee: LogicalType::Float,
                },
                Inst::Call {
                    function: "read".to_string(),
                    arguments: vec![Value::Reg(1)],
                    result: Some(Value::Reg(2)),
                },
                Inst::Return(Value::Reg(2)),
            ],
        ));
        assert!(
            wrong_place_pointee.is_err(),
            "wrong-pointee place passed reference call verification"
        );

        let non_dominating_argument = verify_ir(program(
            reader(vec![
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Return(Value::Reg(1)),
            ]),
            vec![
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(3),
                    left: Value::ImmInt(1),
                    right: Value::ImmInt(1),
                },
                Inst::Branch {
                    condition: Value::Reg(3),
                    true_label: "define".to_string(),
                    false_label: "use".to_string(),
                },
                Inst::Label("define".to_string()),
                Inst::Alloca(Value::Reg(0), "owner".to_string()),
                Inst::Store(Value::Reg(0), Value::ImmInt(7)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                    pointee: LogicalType::Int,
                },
                Inst::Jump("use".to_string()),
                Inst::Label("use".to_string()),
                Inst::Call {
                    function: "read".to_string(),
                    arguments: vec![Value::Reg(1)],
                    result: Some(Value::Reg(2)),
                },
                Inst::Return(Value::Reg(2)),
            ],
        ));
        assert!(
            non_dominating_argument.is_err(),
            "non-dominating reference place passed call verification"
        );

        let reference_result = verify_ir(program(
            Inst::CheckedFunctionDef {
                name: "escape".to_string(),
                parameters: vec![("value".to_string(), reference.clone())],
                result: reference,
                body: vec![Inst::Return(Value::ImmInt(0))],
            },
            vec![Inst::Return(Value::ImmInt(0))],
        ));
        assert!(
            reference_result.is_err(),
            "reference result escaped checked signature validation"
        );
    }

    #[test]
    fn checked_mutable_copy_places_and_assignments_are_fail_closed() {
        let place = || Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(0),
            name: "value".to_string(),
            ty: LogicalType::Int,
        };
        let initialize = || Inst::Store(Value::Reg(0), Value::ImmInt(1));
        let assign = || Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(0),
            value: Value::ImmInt(2),
            ty: LogicalType::Int,
        };

        verify_ir(function(vec![
            place(),
            initialize(),
            assign(),
            Inst::Load(Value::Reg(1), Value::Reg(0)),
            Inst::Return(Value::Reg(1)),
        ]))
        .expect("exact checked scalar reassignment is valid");

        let invalid = [
            (
                "undefined target",
                vec![
                    place(),
                    initialize(),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(9),
                        value: Value::ImmInt(2),
                        ty: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "non-place target",
                vec![
                    place(),
                    initialize(),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::ImmInt(0),
                        value: Value::ImmInt(2),
                        ty: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "generic alloca substitution",
                vec![
                    Inst::Alloca(Value::Reg(0), "value".to_string()),
                    initialize(),
                    assign(),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "unsupported place metadata",
                vec![
                    Inst::CheckedMutableOwnedPlaceAlloca {
                        result: Value::Reg(0),
                        name: "value".to_string(),
                        ty: LogicalType::String,
                    },
                    Inst::Store(Value::Reg(0), Value::ImmString("x".to_string())),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "assignment metadata mismatch",
                vec![
                    place(),
                    initialize(),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(0),
                        value: Value::ImmFloat(2.0),
                        ty: LogicalType::Float,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "wrong RHS type",
                vec![
                    place(),
                    initialize(),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(0),
                        value: Value::ImmFloat(2.0),
                        ty: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "undefined RHS result",
                vec![
                    place(),
                    initialize(),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(0),
                        value: Value::Reg(8),
                        ty: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "identifier-kind collision",
                vec![
                    place(),
                    initialize(),
                    Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "non-adjacent initializer",
                vec![
                    place(),
                    Inst::Add(Value::Reg(1), Value::ImmInt(1), Value::ImmInt(2)),
                    initialize(),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "generic reassignment store",
                vec![
                    place(),
                    initialize(),
                    Inst::Store(Value::Reg(0), Value::ImmInt(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "missing initializer",
                vec![place(), Inst::Return(Value::ImmInt(0))],
            ),
            (
                "assignment before initializer",
                vec![
                    place(),
                    assign(),
                    initialize(),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "non-dominating RHS result",
                vec![
                    place(),
                    initialize(),
                    Inst::ICmp {
                        op: "eq".to_string(),
                        result: Value::Reg(3),
                        left: Value::ImmInt(1),
                        right: Value::ImmInt(1),
                    },
                    Inst::Branch {
                        condition: Value::Reg(3),
                        true_label: "define".to_string(),
                        false_label: "use".to_string(),
                    },
                    Inst::Label("define".to_string()),
                    Inst::Add(Value::Reg(1), Value::ImmInt(1), Value::ImmInt(2)),
                    Inst::Jump("use".to_string()),
                    Inst::Label("use".to_string()),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(0),
                        value: Value::Reg(1),
                        ty: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];

        for (label, body) in invalid {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn projected_copydata_assignment_requires_a_typed_mutable_owner_root() {
        let tuple = LogicalType::Tuple {
            elements: vec![LogicalType::Int, LogicalType::Int],
        };
        let valid = || {
            vec![
                Inst::CheckedTupleAlloca {
                    result: Value::Reg(0),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                },
                Inst::CheckedTupleFieldPtr {
                    result: Value::Reg(1),
                    base: Value::Reg(0),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                    field_index: 0,
                    field_type: LogicalType::Int,
                },
                Inst::Store(Value::Reg(1), Value::ImmInt(1)),
                Inst::CheckedTupleFieldPtr {
                    result: Value::Reg(2),
                    base: Value::Reg(0),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                    field_index: 1,
                    field_type: LogicalType::Int,
                },
                Inst::Store(Value::Reg(2), Value::ImmInt(2)),
                Inst::Load(Value::Reg(3), Value::Reg(0)),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(4),
                    name: "pair".to_string(),
                    ty: tuple.clone(),
                },
                Inst::Store(Value::Reg(4), Value::Reg(3)),
                Inst::CheckedTupleFieldPtr {
                    result: Value::Reg(5),
                    base: Value::Reg(4),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                    field_index: 1,
                    field_type: LogicalType::Int,
                },
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(5),
                    value: Value::ImmInt(9),
                    ty: LogicalType::Int,
                },
                Inst::Return(Value::ImmInt(0)),
            ]
        };

        verify_ir(function(valid()))
            .expect("an exact projected write rooted at a mutable CopyData owner must verify");

        let mut temporary_root = valid();
        temporary_root[9] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(2),
            value: Value::ImmInt(9),
            ty: LogicalType::Int,
        };
        assert!(
            verify_ir(function(temporary_root)).is_err(),
            "a projection of compiler-temporary storage passed as a source-level mutable target"
        );

        let mut wrong_leaf_type = valid();
        wrong_leaf_type[9] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(5),
            value: Value::ImmFloat(9.5),
            ty: LogicalType::Float,
        };
        assert!(
            verify_ir(function(wrong_leaf_type)).is_err(),
            "a projected write with substituted leaf metadata passed verification"
        );

        let mut bypassed_projection = valid();
        bypassed_projection[8] = Inst::GetElementPtr {
            result: Value::Reg(5),
            base: Value::Reg(4),
            index: Value::ImmInt(1),
            elem_type: "{ double, double }".to_string(),
        };
        assert!(
            verify_ir(function(bypassed_projection)).is_err(),
            "a legacy untyped pointer substituted for a checked tuple projection"
        );
    }

    #[test]
    fn checked_mutable_scalar_borrows_writes_and_ends_are_fail_closed() {
        let place = |result, name: &str| Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(result),
            name: name.to_string(),
            ty: LogicalType::Int,
        };
        let borrow = |result, source| Inst::CheckedMutableBorrow {
            result: Value::Reg(result),
            source: Value::Reg(source),
            pointee: LogicalType::Int,
        };
        let write = |target, value| Inst::CheckedMutableDereferenceAssignment {
            target: Value::Reg(target),
            value,
            pointee: LogicalType::Int,
        };
        let end = |reference, source| Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(reference),
            source: Value::Reg(source),
            pointee: LogicalType::Int,
        };

        verify_ir(function(vec![
            place(0, "value"),
            Inst::Store(Value::Reg(0), Value::ImmInt(1)),
            borrow(1, 0),
            write(1, Value::ImmInt(2)),
            Inst::Load(Value::Reg(2), Value::Reg(1)),
            end(1, 0),
            Inst::CheckedOwnedPlaceAssignment {
                target: Value::Reg(0),
                value: Value::ImmInt(3),
                ty: LogicalType::Int,
            },
            Inst::Load(Value::Reg(3), Value::Reg(0)),
            Inst::Return(Value::Reg(3)),
        ]))
        .expect("exact mutable borrow, dereference write, lexical end, and owner reuse are valid");

        let invalid = [
            (
                "undefined source",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 9),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "non-place source",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    Inst::CheckedMutableBorrow {
                        result: Value::Reg(1),
                        source: Value::ImmInt(0),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "generic alloca source substitution",
                vec![
                    Inst::Alloca(Value::Reg(0), "value".to_string()),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "pointee metadata mismatch",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    Inst::CheckedMutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Float,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "wrong write value type",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    write(1, Value::ImmFloat(2.0)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "undefined write value",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    write(1, Value::Reg(9)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "immutable borrow substitution",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    Inst::CheckedImmutableBorrow {
                        result: Value::Reg(1),
                        source: Value::Reg(0),
                        pointee: LogicalType::Int,
                    },
                    write(1, Value::ImmInt(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "raw store through mutable alias",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    Inst::Store(Value::Reg(1), Value::ImmInt(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "wrong lexical end origin",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    place(2, "other"),
                    Inst::Store(Value::Reg(2), Value::ImmInt(2)),
                    borrow(1, 0),
                    end(1, 2),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "write after lexical end",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    end(1, 0),
                    write(1, Value::ImmInt(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "second active mutable borrow",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    borrow(2, 0),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "owner load while mutable alias active",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    Inst::Load(Value::Reg(2), Value::Reg(0)),
                    Inst::Return(Value::Reg(2)),
                ],
            ),
            (
                "owner write while mutable alias active",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    borrow(1, 0),
                    Inst::CheckedOwnedPlaceAssignment {
                        target: Value::Reg(0),
                        value: Value::ImmInt(2),
                        ty: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "reference identifier collision",
                vec![
                    place(0, "value"),
                    Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                    Inst::Add(Value::Reg(1), Value::ImmInt(1), Value::ImmInt(2)),
                    borrow(1, 0),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];

        for (label, body) in invalid {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn checked_mutable_reference_parameters_and_call_temporaries_are_fail_closed() {
        let signature = vec![(
            "value".to_string(),
            LogicalType::MutableReference {
                pointee: Box::new(LogicalType::Int),
            },
        )];
        let binder = || Inst::CheckedMutableReferenceParameter {
            result: Value::Reg(0),
            parameter: "value".to_string(),
            pointee: LogicalType::Int,
        };
        let write = |target, value| Inst::CheckedMutableDereferenceAssignment {
            target: Value::Reg(target),
            value,
            pointee: LogicalType::Int,
        };
        let borrow = || Inst::CheckedMutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
            pointee: LogicalType::Int,
        };
        let call = || Inst::Call {
            function: "bump".to_string(),
            arguments: vec![Value::Reg(1)],
            result: Some(Value::Reg(2)),
        };
        let end = || Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(1),
            source: Value::Reg(0),
            pointee: LogicalType::Int,
        };
        let callee = || {
            vec![
                binder(),
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::Add(Value::Reg(2), Value::Reg(1), Value::ImmInt(1)),
                write(0, Value::Reg(2)),
                Inst::Load(Value::Reg(3), Value::Reg(0)),
                Inst::Return(Value::Reg(3)),
            ]
        };
        let caller = || {
            vec![
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(0),
                    name: "owner".to_string(),
                    ty: LogicalType::Int,
                },
                Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                borrow(),
                call(),
                end(),
                Inst::Load(Value::Reg(3), Value::Reg(0)),
                Inst::Return(Value::Reg(3)),
            ]
        };
        let program = |callee_body: Vec<Inst>, mut caller_body: Vec<Inst>| {
            caller_body.insert(
                0,
                Inst::CheckedFunctionDef {
                    name: "bump".to_string(),
                    parameters: signature.clone(),
                    result: LogicalType::Int,
                    body: callee_body,
                },
            );
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: caller_body,
                        next_reg: 8,
                        next_ptr: 8,
                    },
                ),
                (
                    "bump".to_string(),
                    Function {
                        name: "bump".to_string(),
                        body: Vec::new(),
                        next_reg: 8,
                        next_ptr: 8,
                    },
                ),
            ])
        };

        verify_ir(program(callee(), caller()))
            .expect("exact mutable parameter binder and borrow/call/end temporary are valid");

        let invalid_callees = [
            ("missing binder", vec![Inst::Return(Value::ImmInt(0))]),
            (
                "immutable binder substitution",
                vec![
                    Inst::CheckedImmutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "raw alloca binder substitution",
                vec![
                    Inst::Alloca(Value::Reg(0), "value".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "raw store through mutable parameter",
                vec![
                    binder(),
                    Inst::Store(Value::Reg(0), Value::ImmInt(2)),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "duplicate mutable binder",
                vec![binder(), binder(), Inst::Return(Value::ImmInt(0))],
            ),
            (
                "wrong mutable binder name",
                vec![
                    Inst::CheckedMutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "other".to_string(),
                        pointee: LogicalType::Int,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "wrong mutable binder pointee",
                vec![
                    Inst::CheckedMutableReferenceParameter {
                        result: Value::Reg(0),
                        parameter: "value".to_string(),
                        pointee: LogicalType::Float,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "mutable binder outside entry",
                vec![
                    Inst::Jump("bind".to_string()),
                    Inst::Label("bind".to_string()),
                    binder(),
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ];
        for (label, body) in invalid_callees {
            assert!(
                verify_ir(program(body, caller())).is_err(),
                "{label} passed checked IR verification"
            );
        }

        let mut owner_argument = caller();
        owner_argument[3] = Inst::Call {
            function: "bump".to_string(),
            arguments: vec![Value::Reg(0)],
            result: Some(Value::Reg(2)),
        };
        let mut ended_before_call = caller();
        ended_before_call.swap(3, 4);
        let mut instruction_between_borrow_and_call = caller();
        instruction_between_borrow_and_call.insert(
            3,
            Inst::Add(Value::Reg(4), Value::ImmInt(1), Value::ImmInt(2)),
        );
        let mut instruction_between_call_and_end = caller();
        instruction_between_call_and_end.insert(
            4,
            Inst::Add(Value::Reg(4), Value::ImmInt(1), Value::ImmInt(2)),
        );
        let mut immutable_argument = caller();
        immutable_argument[2] = Inst::CheckedImmutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
            pointee: LogicalType::Int,
        };
        let mut missing_end = caller();
        missing_end.remove(4);

        for (label, body) in [
            ("owner passed directly", owner_argument),
            ("borrow ended before call", ended_before_call),
            (
                "instruction between borrow and call",
                instruction_between_borrow_and_call,
            ),
            (
                "instruction between call and end",
                instruction_between_call_and_end,
            ),
            ("immutable borrow argument substitution", immutable_argument),
            ("missing post-call release", missing_end),
        ] {
            assert!(
                verify_ir(program(callee(), body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn checked_mutable_reference_child_reborrows_are_fail_closed() {
        let signature = vec![(
            "value".to_string(),
            LogicalType::MutableReference {
                pointee: Box::new(LogicalType::Int),
            },
        )];
        let binder = || Inst::CheckedMutableReferenceParameter {
            result: Value::Reg(0),
            parameter: "value".to_string(),
            pointee: LogicalType::Int,
        };
        let borrow = |result, source| Inst::CheckedMutableBorrow {
            result: Value::Reg(result),
            source: Value::Reg(source),
            pointee: LogicalType::Int,
        };
        let end = |reference, source| Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(reference),
            source: Value::Reg(source),
            pointee: LogicalType::Int,
        };
        let write = |target, value| Inst::CheckedMutableDereferenceAssignment {
            target: Value::Reg(target),
            value,
            pointee: LogicalType::Int,
        };
        let call = |function: &str, argument: u32, result: Option<u32>| Inst::Call {
            function: function.to_string(),
            arguments: vec![Value::Reg(argument)],
            result: result.map(Value::Reg),
        };
        let inner_body = || {
            vec![
                binder(),
                write(0, Value::ImmInt(2)),
                Inst::Return(Value::ImmInt(0)),
            ]
        };
        let outer_body = || {
            vec![
                binder(),
                borrow(1, 0),
                call("inner", 1, None),
                end(1, 0),
                write(0, Value::ImmInt(3)),
                Inst::Load(Value::Reg(2), Value::Reg(0)),
                Inst::Return(Value::Reg(2)),
            ]
        };
        let main_body = || {
            vec![
                Inst::CheckedFunctionDef {
                    name: "inner".to_string(),
                    parameters: signature.clone(),
                    result: LogicalType::Void,
                    body: inner_body(),
                },
                Inst::CheckedFunctionDef {
                    name: "outer".to_string(),
                    parameters: signature.clone(),
                    result: LogicalType::Int,
                    body: outer_body(),
                },
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(0),
                    name: "owner".to_string(),
                    ty: LogicalType::Int,
                },
                Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                borrow(1, 0),
                borrow(2, 1),
                call("outer", 2, Some(3)),
                end(2, 1),
                end(1, 0),
                Inst::Load(Value::Reg(4), Value::Reg(0)),
                Inst::Return(Value::Reg(4)),
            ]
        };
        let program = |outer: Vec<Inst>, main: Vec<Inst>| {
            let mut main = main;
            if let Inst::CheckedFunctionDef { body, .. } = &mut main[1] {
                *body = outer;
            } else {
                unreachable!("test main retains outer definition")
            }
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: main,
                        next_reg: 12,
                        next_ptr: 12,
                    },
                ),
                (
                    "inner".to_string(),
                    Function {
                        name: "inner".to_string(),
                        body: Vec::new(),
                        next_reg: 12,
                        next_ptr: 12,
                    },
                ),
                (
                    "outer".to_string(),
                    Function {
                        name: "outer".to_string(),
                        body: Vec::new(),
                        next_reg: 12,
                        next_ptr: 12,
                    },
                ),
            ])
        };

        verify_ir(program(outer_body(), main_body())).expect(
            "active local aliases and mutable parameters permit exact child borrow/call/end reborrows",
        );

        let mut direct_parameter = outer_body();
        direct_parameter[2] = call("inner", 0, None);
        let mut immutable_child = outer_body();
        immutable_child[1] = Inst::CheckedImmutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
            pointee: LogicalType::Int,
        };
        let mut missing_end = outer_body();
        missing_end.remove(3);
        let mut ended_before_call = outer_body();
        ended_before_call.swap(2, 3);
        let mut between_borrow_and_call = outer_body();
        between_borrow_and_call.insert(
            2,
            Inst::Add(Value::Reg(4), Value::ImmInt(1), Value::ImmInt(2)),
        );
        let mut between_call_and_end = outer_body();
        between_call_and_end.insert(
            3,
            Inst::Add(Value::Reg(4), Value::ImmInt(1), Value::ImmInt(2)),
        );
        let mut overlapping_child = outer_body();
        overlapping_child.insert(2, borrow(4, 0));
        let mut parent_load_during_child = outer_body();
        parent_load_during_child.insert(2, Inst::Load(Value::Reg(4), Value::Reg(0)));
        let mut parent_write_during_child = outer_body();
        parent_write_during_child.insert(2, write(0, Value::ImmInt(4)));
        let mut wrong_parent_end = outer_body();
        wrong_parent_end[3] = end(1, 1);
        let mut wrong_pointee_child = outer_body();
        wrong_pointee_child[1] = Inst::CheckedMutableBorrow {
            result: Value::Reg(1),
            source: Value::Reg(0),
            pointee: LogicalType::Float,
        };

        for (label, body) in [
            ("parameter passed without child reborrow", direct_parameter),
            ("immutable child substitution", immutable_child),
            ("missing child end", missing_end),
            ("child ended before call", ended_before_call),
            (
                "instruction between child borrow and call",
                between_borrow_and_call,
            ),
            (
                "instruction between child call and end",
                between_call_and_end,
            ),
            ("overlapping child reborrow", overlapping_child),
            ("parent load during child", parent_load_during_child),
            ("parent write during child", parent_write_during_child),
            ("wrong parent at child end", wrong_parent_end),
            ("wrong child pointee", wrong_pointee_child),
        ] {
            assert!(
                verify_ir(program(body, main_body())).is_err(),
                "{label} passed checked IR verification"
            );
        }

        let mut ended_local_parent = main_body();
        ended_local_parent.swap(5, 8);
        let mut local_parent_load_during_child = main_body();
        local_parent_load_during_child.insert(6, Inst::Load(Value::Reg(5), Value::Reg(1)));
        let mut local_parent_write_during_child = main_body();
        local_parent_write_during_child.insert(6, write(1, Value::ImmInt(5)));
        for (label, body) in [
            ("ended local parent reborrow", ended_local_parent),
            (
                "local parent load during child",
                local_parent_load_during_child,
            ),
            (
                "local parent write during child",
                local_parent_write_during_child,
            ),
        ] {
            assert!(
                verify_ir(program(outer_body(), body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn rejects_a_boolean_store_into_a_legacy_numeric_place() {
        let error = verify_checked_ir(
            &function(vec![
                Inst::Alloca(Value::Reg(0), "number".to_string()),
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmInt(1),
                    right: Value::ImmInt(1),
                },
                Inst::Store(Value::Reg(0), Value::Reg(1)),
                Inst::Return(Value::ImmInt(0)),
            ])
            .into(),
        )
        .unwrap_err();
        let diagnostic = error.to_string().to_ascii_lowercase();
        assert!(diagnostic.contains("store"));
        assert!(diagnostic.contains("bool"));
        assert!(diagnostic.contains("numeric"));
    }

    #[test]
    fn rejects_boolean_elements_during_array_type_inference() {
        let error = verify_ir(function(vec![
            Inst::AllocaArray {
                result: Value::Reg(0),
                elem_type: "double".to_string(),
                count: 1,
            },
            Inst::GetElementPtr {
                result: Value::Reg(1),
                base: Value::Reg(0),
                index: Value::ImmInt(0),
                elem_type: "[1 x double]".to_string(),
            },
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(2),
                left: Value::ImmInt(1),
                right: Value::ImmInt(1),
            },
            Inst::Store(Value::Reg(1), Value::Reg(2)),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .unwrap_err();

        let diagnostic = error.to_string().to_ascii_lowercase();
        assert!(diagnostic.contains("unsupported"));
        assert!(diagnostic.contains("bool array element"));
        assert!(diagnostic.contains("numeric"));
    }

    #[test]
    fn resolves_dominating_result_and_load_types_independently_of_block_text_order() {
        let checked = verify_ir(function(vec![
            Inst::Alloca(Value::Reg(3), "slot".to_string()),
            Inst::Add(Value::Reg(0), Value::ImmInt(1), Value::ImmInt(2)),
            Inst::Jump("define".to_string()),
            Inst::Label("use".to_string()),
            Inst::Load(Value::Reg(2), Value::Reg(3)),
            Inst::Return(Value::Reg(2)),
            Inst::Label("define".to_string()),
            Inst::Neg {
                result: Value::Reg(1),
                operand: Value::Reg(0),
            },
            Inst::Store(Value::Reg(3), Value::Reg(1)),
            Inst::Jump("use".to_string()),
        ]))
        .expect("dominating definitions must not depend on serialized block order");

        let metadata = &checked.metadata().functions["main"];
        assert_eq!(metadata.results[&ResultId(1)], LogicalType::Int);
        assert_eq!(metadata.results[&ResultId(2)], LogicalType::Int);
        assert_eq!(metadata.places[&PlaceId(3)].pointee, LogicalType::Int);
    }

    #[test]
    fn resolves_dominating_array_bases_independently_of_block_text_order() {
        let checked = verify_ir(function(vec![
            Inst::Jump("define".to_string()),
            Inst::Label("use".to_string()),
            Inst::GetElementPtr {
                result: Value::Reg(1),
                base: Value::Reg(0),
                index: Value::ImmInt(0),
                elem_type: "[1 x double]".to_string(),
            },
            Inst::Store(Value::Reg(1), Value::ImmInt(7)),
            Inst::Return(Value::ImmInt(0)),
            Inst::Label("define".to_string()),
            Inst::AllocaArray {
                result: Value::Reg(0),
                elem_type: "double".to_string(),
                count: 1,
            },
            Inst::Jump("use".to_string()),
        ]))
        .expect("a dominating array base may appear in a later serialized block");

        assert_eq!(
            checked.metadata().functions["main"].places[&PlaceId(0)].pointee,
            LogicalType::Array {
                element: Box::new(LogicalType::Int),
                count: 1,
            }
        );
    }

    #[test]
    fn recursive_copy_aggregate_schema_integrity_is_fail_closed() {
        let inner = LogicalType::Struct {
            name: "Inner".to_string(),
            fields: vec![LogicalType::Int],
        };
        let outer_fields = vec![
            inner.clone(),
            LogicalType::Array {
                element: Box::new(inner.clone()),
                count: 2,
            },
            LogicalType::Array {
                element: Box::new(LogicalType::Float),
                count: 0,
            },
            LogicalType::Array {
                element: Box::new(LogicalType::Array {
                    element: Box::new(LogicalType::Int),
                    count: 1,
                }),
                count: 1,
            },
            LogicalType::Array {
                element: Box::new(LogicalType::Bool),
                count: 1,
            },
            LogicalType::Tuple {
                elements: vec![
                    inner.clone(),
                    LogicalType::Array {
                        element: Box::new(LogicalType::Bool),
                        count: 2,
                    },
                ],
            },
        ];
        verify_ir(function(vec![
            Inst::CheckedStructAlloca {
                result: Value::Reg(0),
                struct_name: "Outer".to_string(),
                field_types: outer_fields.clone(),
            },
            Inst::CheckedStructFieldPtr {
                result: Value::Reg(1),
                base: Value::Reg(0),
                struct_name: "Outer".to_string(),
                field_index: 0,
                field_type: inner.clone(),
            },
            Inst::Load(Value::Reg(2), Value::Reg(1)),
            Inst::CheckedStructAlloca {
                result: Value::Reg(3),
                struct_name: "Inner".to_string(),
                field_types: vec![LogicalType::Int],
            },
            Inst::Store(Value::Reg(3), Value::Reg(2)),
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("one exact acyclic recursive schema is valid");

        for (label, body) in [
            (
                "conflicting dependency schema",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(0),
                        struct_name: "Outer".to_string(),
                        field_types: vec![inner.clone()],
                    },
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(1),
                        struct_name: "Inner".to_string(),
                        field_types: vec![LogicalType::Bool],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "self dependency schema",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(0),
                        struct_name: "Cycle".to_string(),
                        field_types: vec![LogicalType::Struct {
                            name: "Cycle".to_string(),
                            fields: vec![LogicalType::Int],
                        }],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "unsupported nested String array field",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(0),
                        struct_name: "StringArray".to_string(),
                        field_types: vec![LogicalType::Array {
                            element: Box::new(LogicalType::Array {
                                element: Box::new(LogicalType::String),
                                count: 1,
                            }),
                            count: 1,
                        }],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "stored reference array field",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(0),
                        struct_name: "ReferenceArray".to_string(),
                        field_types: vec![LogicalType::Array {
                            element: Box::new(LogicalType::ImmutableReference {
                                pointee: Box::new(LogicalType::Int),
                            }),
                            count: 1,
                        }],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ] {
            let error = match verify_ir(function(body)) {
                Err(error) => error,
                Ok(_) => panic!("{label} passed checked IR verification"),
            };
            let diagnostic = error.to_string().to_ascii_lowercase();
            assert!(
                diagnostic.contains("schema") || diagnostic.contains("unsupported"),
                "{label}: {diagnostic}"
            );
        }
    }

    #[test]
    fn flat_copy_tuple_schema_integrity_is_fail_closed() {
        let schema = vec![LogicalType::Int, LogicalType::Bool];
        verify_ir(function(vec![
            Inst::CheckedTupleAlloca {
                result: Value::Reg(0),
                element_types: schema.clone(),
            },
            Inst::CheckedTupleFieldPtr {
                result: Value::Reg(1),
                base: Value::Reg(0),
                element_types: schema.clone(),
                field_index: 1,
                field_type: LogicalType::Bool,
            },
            Inst::Return(Value::ImmInt(0)),
        ]))
        .expect("one exact recursive Copy tuple schema is valid");

        for (label, body) in [
            (
                "unary schema",
                vec![
                    Inst::CheckedTupleAlloca {
                        result: Value::Reg(0),
                        element_types: vec![LogicalType::Int],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "field type mismatch",
                vec![
                    Inst::CheckedTupleAlloca {
                        result: Value::Reg(0),
                        element_types: schema.clone(),
                    },
                    Inst::CheckedTupleFieldPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        element_types: schema.clone(),
                        field_index: 1,
                        field_type: LogicalType::Float,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "base schema mismatch",
                vec![
                    Inst::CheckedTupleAlloca {
                        result: Value::Reg(0),
                        element_types: schema.clone(),
                    },
                    Inst::CheckedTupleFieldPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        element_types: vec![LogicalType::Float, LogicalType::Bool],
                        field_index: 1,
                        field_type: LogicalType::Bool,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "out of range field",
                vec![
                    Inst::CheckedTupleAlloca {
                        result: Value::Reg(0),
                        element_types: schema.clone(),
                    },
                    Inst::CheckedTupleFieldPtr {
                        result: Value::Reg(1),
                        base: Value::Reg(0),
                        element_types: schema.clone(),
                        field_index: 2,
                        field_type: LogicalType::Bool,
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
        ] {
            let error = match verify_ir(function(body)) {
                Err(error) => error,
                Ok(_) => panic!("{label} passed checked IR verification"),
            };
            let diagnostic = error.to_string().to_ascii_lowercase();
            assert!(
                diagnostic.contains("tuple")
                    && (diagnostic.contains("schema") || diagnostic.contains("unsupported")),
                "{label}: {diagnostic}"
            );
        }
    }

    #[test]
    fn mutable_copy_place_metadata_and_loan_identity_are_fail_closed() {
        let leaf = LogicalType::Struct {
            name: "Leaf".to_string(),
            fields: vec![LogicalType::Int, LogicalType::Bool],
        };
        let valid = || {
            vec![
                Inst::CheckedStructAlloca {
                    result: Value::Reg(0),
                    struct_name: "Leaf".to_string(),
                    field_types: vec![LogicalType::Int, LogicalType::Bool],
                },
                Inst::Load(Value::Reg(1), Value::Reg(0)),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(2),
                    name: "owner".to_string(),
                    ty: leaf.clone(),
                },
                Inst::Store(Value::Reg(2), Value::Reg(1)),
                Inst::CheckedMutableBorrow {
                    result: Value::Reg(3),
                    source: Value::Reg(2),
                    pointee: leaf.clone(),
                },
                Inst::CheckedMutableDereferenceAssignment {
                    target: Value::Reg(3),
                    value: Value::Reg(1),
                    pointee: leaf.clone(),
                },
                Inst::CheckedMutableBorrowEnd {
                    reference: Value::Reg(3),
                    source: Value::Reg(2),
                    pointee: leaf.clone(),
                },
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(2),
                    value: Value::Reg(1),
                    ty: leaf.clone(),
                },
                Inst::Return(Value::ImmInt(0)),
            ]
        };

        verify_ir(function(valid()))
            .expect("exact mutable Copy-place loan and subsequent owned assignment must verify");

        let mut wrong_alloca = valid();
        wrong_alloca[2] = Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(2),
            name: "owner".to_string(),
            ty: LogicalType::String,
        };

        let mut missing_initializer = valid();
        missing_initializer.remove(3);

        let mut wrong_borrow_schema = valid();
        wrong_borrow_schema[4] = Inst::CheckedMutableBorrow {
            result: Value::Reg(3),
            source: Value::Reg(2),
            pointee: LogicalType::Struct {
                name: "Leaf".to_string(),
                fields: vec![LogicalType::Float, LogicalType::Bool],
            },
        };

        let mut wrong_write_value = valid();
        wrong_write_value[5] = Inst::CheckedMutableDereferenceAssignment {
            target: Value::Reg(3),
            value: Value::ImmInt(1),
            pointee: leaf.clone(),
        };

        let mut generic_store = valid();
        generic_store[5] = Inst::Store(Value::Reg(3), Value::Reg(1));

        let mut wrong_end_source = valid();
        wrong_end_source[6] = Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(3),
            source: Value::Reg(0),
            pointee: leaf.clone(),
        };

        let mut wrong_owned_assignment_schema = valid();
        wrong_owned_assignment_schema[7] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(2),
            value: Value::Reg(1),
            ty: LogicalType::Struct {
                name: "Leaf".to_string(),
                fields: vec![LogicalType::Float, LogicalType::Bool],
            },
        };

        let mut wrong_owned_assignment_value = valid();
        wrong_owned_assignment_value[7] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(2),
            value: Value::ImmInt(1),
            ty: LogicalType::Struct {
                name: "Leaf".to_string(),
                fields: vec![LogicalType::Int, LogicalType::Bool],
            },
        };

        let mut generic_owner_store = valid();
        generic_owner_store[7] = Inst::Store(Value::Reg(2), Value::Reg(1));

        let mut assignment_during_loan = valid();
        assignment_during_loan.swap(6, 7);

        for (label, body) in [
            ("unsupported mutable owner schema", wrong_alloca),
            ("missing adjacent initializer", missing_initializer),
            ("borrow schema mismatch", wrong_borrow_schema),
            ("whole-write value mismatch", wrong_write_value),
            ("generic store through reference", generic_store),
            ("borrow-end provenance mismatch", wrong_end_source),
            (
                "owned assignment schema mismatch",
                wrong_owned_assignment_schema,
            ),
            (
                "owned assignment value mismatch",
                wrong_owned_assignment_value,
            ),
            ("generic owner reassignment store", generic_owner_store),
            (
                "owned assignment during active loan",
                assignment_during_loan,
            ),
        ] {
            let error = match verify_ir(function(body)) {
                Err(error) => error,
                Ok(_) => panic!("{label} passed checked IR verification"),
            };
            let diagnostic = error.to_string().to_ascii_lowercase();
            assert!(
                diagnostic.contains("mutable")
                    || diagnostic.contains("store")
                    || diagnostic.contains("type mismatch")
                    || diagnostic.contains("owned-place assignment")
                    || diagnostic.contains("schema"),
                "{label}: {diagnostic}"
            );
        }
    }

    #[test]
    fn owned_enum_match_result_cfg_and_schema_identity_are_fail_closed() {
        let input = EnumSchema {
            name: "Input".to_string(),
            variants: vec![
                EnumVariantSchema {
                    name: "First".to_string(),
                    payload: None,
                },
                EnumVariantSchema {
                    name: "Second".to_string(),
                    payload: None,
                },
            ],
        };
        let output = EnumSchema {
            name: "Output".to_string(),
            variants: vec![
                EnumVariantSchema {
                    name: "Left".to_string(),
                    payload: None,
                },
                EnumVariantSchema {
                    name: "Right".to_string(),
                    payload: None,
                },
            ],
        };
        let valid = || {
            vec![
                checked_variant(Value::Reg(0), input.clone(), 0),
                Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(10),
                    result_type: output.logical_type(),
                    dispatch_schema: input.clone(),
                },
                checked_dispatch(Value::Reg(0), input.clone(), &["left", "right"]),
                Inst::Label("left".to_string()),
                checked_variant(Value::Reg(1), output.clone(), 0),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(1),
                    ty: output.logical_type(),
                },
                Inst::Jump("merge".to_string()),
                Inst::Label("right".to_string()),
                checked_variant(Value::Reg(2), output.clone(), 1),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(2),
                    ty: output.logical_type(),
                },
                Inst::Jump("merge".to_string()),
                Inst::Label("merge".to_string()),
                Inst::Load(Value::Reg(3), Value::Reg(10)),
                checked_dispatch(Value::Reg(3), output.clone(), &["done_left", "done_right"]),
                Inst::Label("done_left".to_string()),
                Inst::Return(Value::ImmInt(0)),
                Inst::Label("done_right".to_string()),
                Inst::Return(Value::ImmInt(0)),
            ]
        };

        verify_ir(function(valid()))
            .expect("exact all-path initialized owned-enum Match result must verify");

        let mut generic_alloca = valid();
        generic_alloca[1] = Inst::Alloca(Value::Reg(10), "result".to_string());

        let mut generic_store = valid();
        generic_store[5] = Inst::Store(Value::Reg(10), Value::Reg(1));

        let mut missing_arm_assignment = valid();
        missing_arm_assignment.remove(9);

        let mut bypass_edge = valid();
        bypass_edge[2] = checked_dispatch(Value::Reg(0), input.clone(), &["left", "merge"]);

        let mut wrong_dispatch_identity = valid();
        wrong_dispatch_identity[1] = Inst::CheckedMatchResultPlaceAlloca {
            result: Value::Reg(10),
            result_type: output.logical_type(),
            dispatch_schema: output.clone(),
        };

        let other_output = EnumSchema {
            name: "OtherOutput".to_string(),
            variants: output.variants.clone(),
        };
        let mut wrong_result_identity = valid();
        wrong_result_identity[1] = Inst::CheckedMatchResultPlaceAlloca {
            result: Value::Reg(10),
            result_type: other_output.logical_type(),
            dispatch_schema: input.clone(),
        };

        let mut wrong_assignment_schema = valid();
        wrong_assignment_schema[5] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(10),
            value: Value::Reg(1),
            ty: input.logical_type(),
        };

        let mut non_dominating_value = valid();
        non_dominating_value[9] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(10),
            value: Value::Reg(1),
            ty: output.logical_type(),
        };

        let mut premature_load = valid();
        premature_load.insert(5, Inst::Load(Value::Reg(4), Value::Reg(10)));

        let mut repeated_assignment = valid();
        repeated_assignment.insert(
            6,
            Inst::CheckedOwnedPlaceAssignment {
                target: Value::Reg(10),
                value: Value::Reg(1),
                ty: output.logical_type(),
            },
        );

        let mut duplicate_merged_load = valid();
        duplicate_merged_load.insert(13, Inst::Load(Value::Reg(4), Value::Reg(10)));

        let mut post_merge_single_assignment = valid();
        post_merge_single_assignment.remove(9);
        post_merge_single_assignment.remove(5);
        post_merge_single_assignment.insert(10, checked_variant(Value::Reg(4), output.clone(), 0));
        post_merge_single_assignment.insert(
            11,
            Inst::CheckedOwnedPlaceAssignment {
                target: Value::Reg(10),
                value: Value::Reg(4),
                ty: output.logical_type(),
            },
        );

        for (label, body) in [
            ("generic result allocation", generic_alloca),
            ("generic arm store", generic_store),
            ("missing arm assignment", missing_arm_assignment),
            ("dispatch bypass edge", bypass_edge),
            ("wrong dispatch identity", wrong_dispatch_identity),
            ("wrong result identity", wrong_result_identity),
            ("wrong assignment schema", wrong_assignment_schema),
            ("non-dominating arm value", non_dominating_value),
            ("premature arm load", premature_load),
            ("repeated arm assignment", repeated_assignment),
            ("duplicate merged load", duplicate_merged_load),
            (
                "post-merge fabricated assignment",
                post_merge_single_assignment,
            ),
        ] {
            let error = match verify_ir(function(body)) {
                Err(error) => error,
                Ok(_) => panic!("{label} passed checked IR verification"),
            };
            let diagnostic = error.to_string().to_ascii_lowercase();
            assert!(
                diagnostic.contains("match result")
                    || diagnostic.contains("owned-place assignment")
                    || diagnostic.contains("domin")
                    || diagnostic.contains("schema")
                    || diagnostic.contains("terminator"),
                "{label}: {diagnostic}"
            );
        }
    }

    #[test]
    fn copydata_match_result_cfg_and_type_identity_are_fail_closed() {
        let input = unit_schema("Input", &["Left", "Right"]);
        let result_type = LogicalType::Tuple {
            elements: vec![LogicalType::Int, LogicalType::Int],
        };
        let valid = || {
            vec![
                Inst::CheckedTupleAlloca {
                    result: Value::Reg(20),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                },
                Inst::CheckedTupleFieldPtr {
                    result: Value::Reg(21),
                    base: Value::Reg(20),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                    field_index: 0,
                    field_type: LogicalType::Int,
                },
                Inst::Store(Value::Reg(21), Value::ImmInt(7)),
                Inst::CheckedTupleFieldPtr {
                    result: Value::Reg(22),
                    base: Value::Reg(20),
                    element_types: vec![LogicalType::Int, LogicalType::Int],
                    field_index: 1,
                    field_type: LogicalType::Int,
                },
                Inst::Store(Value::Reg(22), Value::ImmInt(9)),
                Inst::Load(Value::Reg(23), Value::Reg(20)),
                checked_variant(Value::Reg(0), input.clone(), 0),
                Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(10),
                    result_type: result_type.clone(),
                    dispatch_schema: input.clone(),
                },
                checked_dispatch(Value::Reg(0), input.clone(), &["left", "right"]),
                Inst::Label("left".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(23),
                    ty: result_type.clone(),
                },
                Inst::Jump("merge".to_string()),
                Inst::Label("right".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(23),
                    ty: result_type.clone(),
                },
                Inst::Jump("merge".to_string()),
                Inst::Label("merge".to_string()),
                Inst::Load(Value::Reg(24), Value::Reg(10)),
                Inst::Return(Value::ImmInt(0)),
            ]
        };

        verify_ir(function(valid()))
            .expect("exact all-path initialized CopyData Match result must verify");

        let mut generic_alloca = valid();
        generic_alloca[7] = Inst::Alloca(Value::Reg(10), "result".to_string());

        let mut generic_store = valid();
        generic_store[10] = Inst::Store(Value::Reg(10), Value::Reg(23));

        let mut missing_arm_assignment = valid();
        missing_arm_assignment.remove(13);

        let mut wrong_result_type = valid();
        wrong_result_type[7] = Inst::CheckedMatchResultPlaceAlloca {
            result: Value::Reg(10),
            result_type: LogicalType::Array {
                element: Box::new(LogicalType::Int),
                count: 2,
            },
            dispatch_schema: input.clone(),
        };

        let mut wrong_assignment_type = valid();
        wrong_assignment_type[10] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(10),
            value: Value::Reg(23),
            ty: LogicalType::Int,
        };

        let mut wrong_assignment_value = valid();
        wrong_assignment_value[10] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(10),
            value: Value::ImmInt(7),
            ty: result_type,
        };

        for (label, body) in [
            ("generic result allocation", generic_alloca),
            ("generic arm store", generic_store),
            ("missing arm assignment", missing_arm_assignment),
            ("wrong result type", wrong_result_type),
            ("wrong assignment type", wrong_assignment_type),
            ("wrong assignment value", wrong_assignment_value),
        ] {
            let error = match verify_ir(function(body)) {
                Err(error) => error,
                Ok(_) => panic!("{label} passed checked IR verification"),
            };
            let diagnostic = error.to_string().to_ascii_lowercase();
            assert!(
                diagnostic.contains("match result")
                    || diagnostic.contains("owned-place assignment")
                    || diagnostic.contains("type"),
                "{label}: {diagnostic}"
            );
        }
    }

    #[test]
    fn mutable_owned_enum_places_and_assignments_are_fail_closed() {
        let schema = EnumSchema {
            name: "E".to_string(),
            variants: vec![
                EnumVariantSchema {
                    name: "Empty".to_string(),
                    payload: None,
                },
                EnumVariantSchema {
                    name: "Pair".to_string(),
                    payload: Some(LogicalType::Tuple {
                        elements: vec![LogicalType::Int, LogicalType::Bool],
                    }),
                },
            ],
        };
        let logical = schema.logical_type();
        let valid = || {
            vec![
                Inst::CheckedEnumVariant {
                    result: Value::Reg(0),
                    schema: schema.clone(),
                    variant_index: 0,
                    payload: None,
                },
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(5),
                    name: "owner".to_string(),
                    ty: logical.clone(),
                },
                Inst::Store(Value::Reg(5), Value::Reg(0)),
                Inst::CheckedEnumVariant {
                    result: Value::Reg(1),
                    schema: schema.clone(),
                    variant_index: 0,
                    payload: None,
                },
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(5),
                    value: Value::Reg(1),
                    ty: logical.clone(),
                },
                Inst::Load(Value::Reg(2), Value::Reg(5)),
                Inst::Return(Value::ImmInt(0)),
            ]
        };

        verify_ir(function(valid())).expect("exact mutable owned enum replacement must verify");

        let mut unsupported_payload = valid();
        unsupported_payload[1] = Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(5),
            name: "owner".to_string(),
            ty: LogicalType::Enum {
                name: "E".to_string(),
                variants: vec![EnumVariantSchema {
                    name: "Bad".to_string(),
                    payload: Some(LogicalType::String),
                }],
            },
        };

        let mut changed_assignment_schema = valid();
        changed_assignment_schema[4] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(5),
            value: Value::Reg(1),
            ty: LogicalType::Enum {
                name: "E".to_string(),
                variants: vec![EnumVariantSchema {
                    name: "Changed".to_string(),
                    payload: None,
                }],
            },
        };

        let mut wrong_value = valid();
        wrong_value[4] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(5),
            value: Value::ImmInt(1),
            ty: logical.clone(),
        };

        let mut generic_later_store = valid();
        generic_later_store[4] = Inst::Store(Value::Reg(5), Value::Reg(1));

        let mut generic_alloca = valid();
        generic_alloca[1] = Inst::Alloca(Value::Reg(5), "owner".to_string());

        let mut missing_initializer = valid();
        missing_initializer.remove(2);

        let mut non_adjacent_initializer = valid();
        non_adjacent_initializer.insert(2, Inst::Load(Value::Reg(3), Value::Reg(5)));

        let mut kind_collision = valid();
        kind_collision[1] = Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(0),
            name: "owner".to_string(),
            ty: logical,
        };

        for (label, body) in [
            ("unsupported enum payload", unsupported_payload),
            ("changed assignment schema", changed_assignment_schema),
            ("wrong assignment value", wrong_value),
            ("generic later store", generic_later_store),
            ("generic alloca substitution", generic_alloca),
            ("missing initializer", missing_initializer),
            ("non-adjacent initializer", non_adjacent_initializer),
            ("result/place kind collision", kind_collision),
        ] {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed mutable owned enum verification"
            );
        }
    }

    #[test]
    fn immutable_enum_match_reference_provenance_and_use_are_fail_closed() {
        let schema = unit_schema("State", &["Idle"]);
        let logical = schema.logical_type();
        let valid = || {
            vec![
                checked_variant(Value::Reg(0), schema.clone(), 0),
                Inst::CheckedImmutableEnumOwnerPlaceAlloca {
                    result: Value::Reg(10),
                    name: "owner".to_string(),
                    schema: schema.clone(),
                },
                Inst::Store(Value::Reg(10), Value::Reg(0)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(11),
                    source: Value::Reg(10),
                    pointee: logical.clone(),
                },
                Inst::CheckedImmutableEnumMatchRead {
                    result: Value::Reg(1),
                    reference: Value::Reg(11),
                    schema: schema.clone(),
                },
                Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(12),
                    result_type: LogicalType::Int,
                    dispatch_schema: schema.clone(),
                },
                checked_dispatch(Value::Reg(1), schema.clone(), &["arm"]),
                Inst::Label("arm".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(12),
                    value: Value::ImmInt(9),
                    ty: LogicalType::Int,
                },
                Inst::Jump("end".to_string()),
                Inst::Label("end".to_string()),
                Inst::Load(Value::Reg(2), Value::Reg(12)),
                Inst::Return(Value::Reg(2)),
            ]
        };

        verify_ir(function(valid())).expect("exact immutable enum Match read must verify");

        let mut generic_owner = valid();
        generic_owner[1] = Inst::Alloca(Value::Reg(10), "owner".to_string());

        let mut wrong_owner_schema = valid();
        wrong_owner_schema[1] = Inst::CheckedImmutableEnumOwnerPlaceAlloca {
            result: Value::Reg(10),
            name: "owner".to_string(),
            schema: unit_schema("Other", &["Idle"]),
        };

        let mut missing_initializer = valid();
        missing_initializer.remove(2);

        let mut non_adjacent_initializer = valid();
        non_adjacent_initializer.insert(2, Inst::Load(Value::Reg(3), Value::Reg(10)));

        let mut mutable_reference = valid();
        mutable_reference[3] = Inst::CheckedMutableBorrow {
            result: Value::Reg(11),
            source: Value::Reg(10),
            pointee: logical.clone(),
        };

        let mut owner_as_reference = valid();
        owner_as_reference[4] = Inst::CheckedImmutableEnumMatchRead {
            result: Value::Reg(1),
            reference: Value::Reg(10),
            schema: schema.clone(),
        };

        let mut wrong_read_schema = valid();
        wrong_read_schema[4] = Inst::CheckedImmutableEnumMatchRead {
            result: Value::Reg(1),
            reference: Value::Reg(11),
            schema: unit_schema("Other", &["Idle"]),
        };

        let mut generic_reference_load = valid();
        generic_reference_load[4] = Inst::Load(Value::Reg(1), Value::Reg(11));

        let mut separated_read = valid();
        separated_read.insert(5, Inst::Load(Value::Reg(3), Value::Reg(10)));

        let mut wrong_dispatch_value = valid();
        wrong_dispatch_value[6] = checked_dispatch(Value::Reg(0), schema, &["arm"]);

        for (label, body) in [
            ("generic owner alloca", generic_owner),
            ("wrong owner schema", wrong_owner_schema),
            ("missing owner initializer", missing_initializer),
            ("non-adjacent owner initializer", non_adjacent_initializer),
            ("mutable reference substitution", mutable_reference),
            ("owner substituted for reference", owner_as_reference),
            ("wrong Match-read schema", wrong_read_schema),
            ("generic reference load", generic_reference_load),
            ("separated Match read", separated_read),
            ("wrong dispatch value", wrong_dispatch_value),
        ] {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed immutable enum Match-read verification"
            );
        }
    }

    #[test]
    fn mutable_enum_match_reference_provenance_identity_and_topology_are_fail_closed() {
        let schema = unit_schema("State", &["Idle"]);
        let logical = schema.logical_type();
        let mutable_reference = LogicalType::MutableReference {
            pointee: Box::new(logical.clone()),
        };
        let valued_body = || {
            vec![
                Inst::CheckedMutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: logical.clone(),
                },
                Inst::CheckedMutableEnumMatchRead {
                    result: Value::Reg(1),
                    reference: Value::Reg(0),
                    schema: schema.clone(),
                },
                Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(12),
                    result_type: LogicalType::Int,
                    dispatch_schema: schema.clone(),
                },
                checked_dispatch(Value::Reg(1), schema.clone(), &["arm"]),
                Inst::Label("arm".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(12),
                    value: Value::ImmInt(9),
                    ty: LogicalType::Int,
                },
                Inst::Jump("end".to_string()),
                Inst::Label("end".to_string()),
                Inst::Load(Value::Reg(2), Value::Reg(12)),
                Inst::Return(Value::Reg(2)),
            ]
        };
        let void_body = || {
            vec![
                Inst::CheckedMutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: logical.clone(),
                },
                Inst::CheckedMutableEnumMatchRead {
                    result: Value::Reg(1),
                    reference: Value::Reg(0),
                    schema: schema.clone(),
                },
                checked_dispatch(Value::Reg(1), schema.clone(), &["arm"]),
                Inst::Label("arm".to_string()),
                Inst::Jump("end".to_string()),
                Inst::Label("end".to_string()),
                Inst::Return(Value::ImmInt(0)),
            ]
        };
        let program = |body: Vec<Inst>| {
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: vec![
                            Inst::CheckedFunctionDef {
                                name: "observe".to_string(),
                                parameters: vec![("value".to_string(), mutable_reference.clone())],
                                result: LogicalType::Int,
                                body,
                            },
                            Inst::Return(Value::ImmInt(0)),
                        ],
                        next_reg: 32,
                        next_ptr: 32,
                    },
                ),
                (
                    "observe".to_string(),
                    Function {
                        name: "observe".to_string(),
                        body: Vec::new(),
                        next_reg: 32,
                        next_ptr: 32,
                    },
                ),
            ])
        };

        verify_ir(program(valued_body()))
            .expect("exact valued Match through a mutable enum parameter must verify");
        verify_ir(program(void_body()))
            .expect("exact Void Match through a mutable enum parameter must verify");

        let mut immutable_identity = valued_body();
        immutable_identity[1] = Inst::CheckedImmutableEnumMatchRead {
            result: Value::Reg(1),
            reference: Value::Reg(0),
            schema: schema.clone(),
        };

        let mut wrong_reference = valued_body();
        wrong_reference[1] = Inst::CheckedMutableEnumMatchRead {
            result: Value::Reg(1),
            reference: Value::Reg(9),
            schema: schema.clone(),
        };

        let mut wrong_schema = valued_body();
        wrong_schema[1] = Inst::CheckedMutableEnumMatchRead {
            result: Value::Reg(1),
            reference: Value::Reg(0),
            schema: unit_schema("Other", &["Idle"]),
        };

        let mut generic_load = valued_body();
        generic_load[1] = Inst::Load(Value::Reg(1), Value::Reg(0));

        let mut separated_read = valued_body();
        separated_read.insert(
            2,
            Inst::Add(Value::Reg(7), Value::ImmInt(1), Value::ImmInt(2)),
        );

        let mut wrong_dispatch_value = valued_body();
        wrong_dispatch_value[3] = checked_dispatch(Value::Reg(7), schema.clone(), &["arm"]);

        let mut unbound_parameter = valued_body();
        unbound_parameter.remove(0);

        let mut separated_void_read = void_body();
        separated_void_read.insert(
            2,
            Inst::Add(Value::Reg(7), Value::ImmInt(1), Value::ImmInt(2)),
        );

        for (label, body) in [
            ("immutable read identity", immutable_identity),
            ("wrong reference identity", wrong_reference),
            ("wrong read schema", wrong_schema),
            ("generic load substitution", generic_load),
            ("lost valued dispatch adjacency", separated_read),
            ("wrong dispatch value", wrong_dispatch_value),
            ("unbound mutable parameter", unbound_parameter),
            ("lost Void dispatch adjacency", separated_void_read),
        ] {
            assert!(
                verify_ir(program(body)).is_err(),
                "{label} passed mutable enum Match-read verification"
            );
        }

        let local_body = || {
            vec![
                checked_variant(Value::Reg(0), schema.clone(), 0),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(10),
                    name: "owner".to_string(),
                    ty: logical.clone(),
                },
                Inst::Store(Value::Reg(10), Value::Reg(0)),
                Inst::CheckedMutableBorrow {
                    result: Value::Reg(11),
                    source: Value::Reg(10),
                    pointee: logical.clone(),
                },
                Inst::CheckedMutableEnumMatchRead {
                    result: Value::Reg(1),
                    reference: Value::Reg(11),
                    schema: schema.clone(),
                },
                Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(12),
                    result_type: LogicalType::Int,
                    dispatch_schema: schema.clone(),
                },
                checked_dispatch(Value::Reg(1), schema.clone(), &["arm"]),
                Inst::Label("arm".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(12),
                    value: Value::ImmInt(9),
                    ty: LogicalType::Int,
                },
                Inst::Jump("end".to_string()),
                Inst::Label("end".to_string()),
                Inst::Load(Value::Reg(2), Value::Reg(12)),
                Inst::CheckedMutableBorrowEnd {
                    reference: Value::Reg(11),
                    source: Value::Reg(10),
                    pointee: logical.clone(),
                },
                Inst::Return(Value::Reg(2)),
            ]
        };
        verify_ir(function(local_body()))
            .expect("an exact active local mutable enum reference Match must verify");

        let mut inactive_local = local_body();
        inactive_local.insert(
            4,
            Inst::CheckedMutableBorrowEnd {
                reference: Value::Reg(11),
                source: Value::Reg(10),
                pointee: logical,
            },
        );
        assert!(
            verify_ir(function(inactive_local)).is_err(),
            "mutable enum Match read after the exact local loan end passed verification"
        );
    }

    #[test]
    fn mutable_owner_immutable_enum_loan_identity_and_overlap_are_fail_closed() {
        let schema = unit_schema("State", &["Idle", "Ready"]);
        let logical = schema.logical_type();
        let variant = || checked_variant(Value::Reg(0), schema.clone(), 0);
        let owner = || Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(10),
            name: "owner".to_string(),
            ty: logical.clone(),
        };
        let borrow = |reference| Inst::CheckedImmutableBorrow {
            result: Value::Reg(reference),
            source: Value::Reg(10),
            pointee: logical.clone(),
        };
        let end = |reference| Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
            reference: Value::Reg(reference),
            source: Value::Reg(10),
            schema: schema.clone(),
        };
        let valid = || {
            vec![
                variant(),
                owner(),
                Inst::Store(Value::Reg(10), Value::Reg(0)),
                borrow(11),
                borrow(12),
                end(12),
                end(11),
                checked_variant(Value::Reg(1), schema.clone(), 1),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(1),
                    ty: logical.clone(),
                },
                Inst::Load(Value::Reg(2), Value::Reg(10)),
                Inst::Return(Value::ImmInt(0)),
            ]
        };

        verify_ir(function(valid()))
            .expect("two exact immutable aliases must preserve the mutable owner after both ends");

        let immutable_reference = LogicalType::ImmutableReference {
            pointee: Box::new(logical.clone()),
        };
        let observe_body = vec![
            Inst::CheckedImmutableReferenceParameter {
                result: Value::Reg(0),
                parameter: "value".to_string(),
                pointee: logical.clone(),
            },
            Inst::Return(Value::ImmInt(0)),
        ];
        let observe_definition = Inst::CheckedFunctionDef {
            name: "observe".to_string(),
            parameters: vec![("value".to_string(), immutable_reference)],
            result: LogicalType::Int,
            body: observe_body,
        };
        let call_program = |runtime: Vec<Inst>| {
            let mut main = vec![observe_definition.clone()];
            main.extend(runtime);
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: main,
                        next_reg: 32,
                        next_ptr: 32,
                    },
                ),
                (
                    "observe".to_string(),
                    Function {
                        name: "observe".to_string(),
                        body: Vec::new(),
                        next_reg: 32,
                        next_ptr: 32,
                    },
                ),
            ])
        };
        let valid_call = vec![
            variant(),
            owner(),
            Inst::Store(Value::Reg(10), Value::Reg(0)),
            borrow(11),
            Inst::Call {
                function: "observe".to_string(),
                arguments: vec![Value::Reg(11)],
                result: Some(Value::Reg(1)),
            },
            end(11),
            Inst::Return(Value::Reg(1)),
        ];
        verify_ir(call_program(valid_call.clone()))
            .expect("an exact active immutable enum alias may be passed to its checked callee");
        let mut call_after_end = valid_call;
        call_after_end.swap(4, 5);
        assert!(
            verify_ir(call_program(call_after_end)).is_err(),
            "immutable enum reference call after its exact loan end passed verification"
        );

        let mut missing_end = valid();
        missing_end.remove(6);

        let mut duplicate_end = valid();
        duplicate_end.insert(7, end(11));

        let mut wrong_reference = valid();
        wrong_reference[5] = Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
            reference: Value::Reg(11),
            source: Value::Reg(10),
            schema: schema.clone(),
        };

        let mut wrong_source = valid();
        wrong_source[5] = Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
            reference: Value::Reg(12),
            source: Value::Reg(11),
            schema: schema.clone(),
        };

        let mut wrong_schema = valid();
        wrong_schema[5] = Inst::CheckedMutableOwnerImmutableEnumBorrowEnd {
            reference: Value::Reg(12),
            source: Value::Reg(10),
            schema: unit_schema("Other", &["Idle", "Ready"]),
        };

        let mut assignment_while_one_alias_remains = valid();
        assignment_while_one_alias_remains.swap(6, 8);

        let mut load_while_live = valid();
        load_while_live.insert(5, Inst::Load(Value::Reg(3), Value::Reg(10)));

        let mut mutable_borrow_while_live = valid();
        mutable_borrow_while_live.insert(
            5,
            Inst::CheckedMutableBorrow {
                result: Value::Reg(13),
                source: Value::Reg(10),
                pointee: logical.clone(),
            },
        );

        let mut generic_store_while_live = valid();
        generic_store_while_live.insert(5, Inst::Store(Value::Reg(10), Value::Reg(0)));

        let mut immutable_owner_end = valid();
        immutable_owner_end[1] = Inst::CheckedImmutableEnumOwnerPlaceAlloca {
            result: Value::Reg(10),
            name: "owner".to_string(),
            schema: schema.clone(),
        };

        let fabricated_end = vec![
            variant(),
            owner(),
            Inst::Store(Value::Reg(10), Value::Reg(0)),
            end(11),
            Inst::Return(Value::ImmInt(0)),
        ];

        let end_on_only_one_branch = vec![
            variant(),
            owner(),
            Inst::Store(Value::Reg(10), Value::Reg(0)),
            borrow(11),
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(3),
                left: Value::ImmInt(1),
                right: Value::ImmInt(1),
            },
            Inst::Branch {
                condition: Value::Reg(3),
                true_label: "ended".to_string(),
                false_label: "live".to_string(),
            },
            Inst::Label("ended".to_string()),
            end(11),
            Inst::Jump("merge".to_string()),
            Inst::Label("live".to_string()),
            Inst::Jump("merge".to_string()),
            Inst::Label("merge".to_string()),
            checked_variant(Value::Reg(1), schema.clone(), 1),
            Inst::CheckedOwnedPlaceAssignment {
                target: Value::Reg(10),
                value: Value::Reg(1),
                ty: logical.clone(),
            },
            Inst::Return(Value::ImmInt(0)),
        ];

        for (label, body) in [
            ("missing end", missing_end),
            ("duplicate end", duplicate_end),
            ("wrong reference", wrong_reference),
            ("wrong source", wrong_source),
            ("wrong schema", wrong_schema),
            (
                "assignment while one alias remains",
                assignment_while_one_alias_remains,
            ),
            ("owner load while live", load_while_live),
            ("mutable borrow while live", mutable_borrow_while_live),
            ("generic store while live", generic_store_while_live),
            ("immutable owner end substitution", immutable_owner_end),
            ("fabricated end", fabricated_end),
            ("end on only one CFG branch", end_on_only_one_branch),
        ] {
            assert!(
                verify_ir(function(body)).is_err(),
                "{label} passed mutable-owner immutable enum loan verification"
            );
        }
    }

    #[test]
    fn mutable_enum_reference_schema_loans_and_writes_are_fail_closed() {
        let schema = unit_schema("State", &["Idle", "Ready"]);
        let logical = schema.logical_type();
        let mutable_reference = LogicalType::MutableReference {
            pointee: Box::new(logical.clone()),
        };
        let callee = || {
            vec![
                Inst::CheckedMutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    pointee: logical.clone(),
                },
                checked_variant(Value::Reg(1), schema.clone(), 1),
                Inst::CheckedMutableDereferenceAssignment {
                    target: Value::Reg(0),
                    value: Value::Reg(1),
                    pointee: logical.clone(),
                },
                Inst::Return(Value::ImmInt(0)),
            ]
        };
        let caller = || {
            vec![
                Inst::CheckedFunctionDef {
                    name: "replace".to_string(),
                    parameters: vec![("value".to_string(), mutable_reference.clone())],
                    result: LogicalType::Int,
                    body: callee(),
                },
                checked_variant(Value::Reg(0), schema.clone(), 0),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(5),
                    name: "owner".to_string(),
                    ty: logical.clone(),
                },
                Inst::Store(Value::Reg(5), Value::Reg(0)),
                Inst::CheckedMutableBorrow {
                    result: Value::Reg(6),
                    source: Value::Reg(5),
                    pointee: logical.clone(),
                },
                Inst::Call {
                    function: "replace".to_string(),
                    arguments: vec![Value::Reg(6)],
                    result: Some(Value::Reg(7)),
                },
                Inst::CheckedMutableBorrowEnd {
                    reference: Value::Reg(6),
                    source: Value::Reg(5),
                    pointee: logical.clone(),
                },
                Inst::Load(Value::Reg(2), Value::Reg(5)),
                Inst::Return(Value::Reg(7)),
            ]
        };
        let program = |callee_body: Vec<Inst>, mut caller_body: Vec<Inst>| {
            let Inst::CheckedFunctionDef { body, .. } = &mut caller_body[0] else {
                unreachable!("caller fixture starts with the checked callee definition")
            };
            *body = callee_body;
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: caller_body,
                        next_reg: 16,
                        next_ptr: 16,
                    },
                ),
                (
                    "replace".to_string(),
                    Function {
                        name: "replace".to_string(),
                        body: Vec::new(),
                        next_reg: 16,
                        next_ptr: 16,
                    },
                ),
            ])
        };

        verify_ir(program(callee(), caller())).expect(
            "exact mutable enum reference signature, replacement, call, and loan end verify",
        );

        let mut wrong_binder = callee();
        wrong_binder[0] = Inst::CheckedMutableReferenceParameter {
            result: Value::Reg(0),
            parameter: "value".to_string(),
            pointee: unit_schema("Other", &["Idle", "Ready"]).logical_type(),
        };

        let mut wrong_write = callee();
        wrong_write[2] = Inst::CheckedMutableDereferenceAssignment {
            target: Value::Reg(0),
            value: Value::ImmInt(1),
            pointee: logical.clone(),
        };

        let mut generic_write = callee();
        generic_write[2] = Inst::Store(Value::Reg(0), Value::Reg(1));

        let mut read_through_reference = callee();
        read_through_reference.insert(1, Inst::Load(Value::Reg(8), Value::Reg(0)));

        for (label, body) in [
            ("wrong mutable enum binder schema", wrong_binder),
            ("wrong enum replacement value", wrong_write),
            ("generic store through enum reference", generic_write),
            (
                "enum read through mutable reference",
                read_through_reference,
            ),
        ] {
            assert!(
                verify_ir(program(body, caller())).is_err(),
                "{label} passed checked IR verification"
            );
        }

        let mut wrong_borrow_schema = caller();
        wrong_borrow_schema[4] = Inst::CheckedMutableBorrow {
            result: Value::Reg(6),
            source: Value::Reg(5),
            pointee: unit_schema("Other", &["Idle", "Ready"]).logical_type(),
        };

        let mut generic_call_store = caller();
        generic_call_store[5] = Inst::Store(Value::Reg(6), Value::Reg(0));

        let mut missing_end = caller();
        missing_end.remove(6);

        let mut wrong_end = caller();
        wrong_end[6] = Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(6),
            source: Value::Reg(6),
            pointee: logical.clone(),
        };

        let mut consumed_owner = caller();
        consumed_owner.splice(
            4..4,
            [
                Inst::Load(Value::Reg(8), Value::Reg(5)),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(9),
                    name: "moved".to_string(),
                    ty: logical.clone(),
                },
                Inst::Store(Value::Reg(9), Value::Reg(8)),
            ],
        );

        let invalid_schema = LogicalType::Enum {
            name: "State".to_string(),
            variants: vec![EnumVariantSchema {
                name: "Bad".to_string(),
                payload: Some(LogicalType::String),
            }],
        };
        let mut unsupported_schema = caller();
        unsupported_schema[2] = Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(5),
            name: "owner".to_string(),
            ty: invalid_schema,
        };

        for (label, body) in [
            ("borrow schema substitution", wrong_borrow_schema),
            ("generic store through call temporary", generic_call_store),
            ("missing lexical loan end", missing_end),
            ("wrong lexical loan identity", wrong_end),
            ("borrow of consumed enum owner", consumed_owner),
            ("unsupported enum reference schema", unsupported_schema),
        ] {
            assert!(
                verify_ir(program(callee(), body)).is_err(),
                "{label} passed checked IR verification"
            );
        }
    }

    #[test]
    fn conditional_enum_owner_dataflow_is_path_sensitive_and_fail_closed() {
        let schema = unit_schema("E", &["A", "B"]);
        let logical = schema.logical_type();
        let consume_definition = Inst::CheckedFunctionDef {
            name: "consume".to_string(),
            parameters: vec![("value".to_string(), logical.clone())],
            result: LogicalType::Int,
            body: vec![
                Inst::CheckedEnumParameter {
                    result: Value::Reg(0),
                    parameter: "value".to_string(),
                    schema: schema.clone(),
                },
                Inst::Return(Value::ImmInt(0)),
            ],
        };
        let program = |runtime: Vec<Inst>| {
            let mut main = vec![consume_definition.clone()];
            main.extend(runtime);
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: main,
                        next_reg: 16,
                        next_ptr: 16,
                    },
                ),
                (
                    "consume".to_string(),
                    Function {
                        name: "consume".to_string(),
                        body: Vec::new(),
                        next_reg: 16,
                        next_ptr: 16,
                    },
                ),
            ])
        };
        let construct_and_branch = || {
            vec![
                checked_variant(Value::Reg(0), schema.clone(), 0),
                Inst::ICmp {
                    op: "eq".to_string(),
                    result: Value::Reg(1),
                    left: Value::ImmInt(0),
                    right: Value::ImmInt(0),
                },
                Inst::Branch {
                    condition: Value::Reg(1),
                    true_label: "then".to_string(),
                    false_label: "else".to_string(),
                },
            ]
        };
        let call = |result| Inst::Call {
            function: "consume".to_string(),
            arguments: vec![Value::Reg(0)],
            result: Some(Value::Reg(result)),
        };

        let direct_match_result = |same_owner: bool, reuse_left: bool| {
            let mut body = vec![
                checked_variant(Value::Reg(0), schema.clone(), 0),
                checked_variant(Value::Reg(1), schema.clone(), 0),
                checked_variant(Value::Reg(2), schema.clone(), 1),
                Inst::CheckedMatchResultPlaceAlloca {
                    result: Value::Reg(10),
                    result_type: schema.logical_type(),
                    dispatch_schema: schema.clone(),
                },
                checked_dispatch(Value::Reg(0), schema.clone(), &["left", "right"]),
                Inst::Label("left".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(1),
                    ty: logical.clone(),
                },
                Inst::Jump("merge".to_string()),
                Inst::Label("right".to_string()),
                Inst::CheckedOwnedPlaceAssignment {
                    target: Value::Reg(10),
                    value: Value::Reg(if same_owner { 1 } else { 2 }),
                    ty: logical.clone(),
                },
                Inst::Jump("merge".to_string()),
                Inst::Label("merge".to_string()),
                Inst::Load(Value::Reg(3), Value::Reg(10)),
            ];
            if reuse_left {
                body.extend([
                    checked_dispatch(Value::Reg(1), schema.clone(), &["done_a", "done_b"]),
                    Inst::Label("done_a".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                    Inst::Label("done_b".to_string()),
                    Inst::Return(Value::ImmInt(0)),
                ]);
            } else {
                body.push(Inst::Return(Value::ImmInt(0)));
            }
            body
        };

        verify_ir(program(direct_match_result(true, false))).expect(
            "one direct owner repeated only across mutually exclusive result arms must verify",
        );
        verify_ir(program(direct_match_result(false, false))).expect(
            "different direct owners selected by mutually exclusive result arms must verify",
        );
        assert!(
            verify_ir(program(direct_match_result(true, true))).is_err(),
            "all-path direct Match-result owner reuse passed ownership verification"
        );
        assert!(
            verify_ir(program(direct_match_result(false, true))).is_err(),
            "partial-path direct Match-result owner reuse passed ownership verification"
        );

        let mut exclusive = construct_and_branch();
        exclusive.extend([
            Inst::Label("then".to_string()),
            call(2),
            Inst::Jump("merge".to_string()),
            Inst::Label("else".to_string()),
            call(3),
            Inst::Jump("merge".to_string()),
            Inst::Label("merge".to_string()),
            Inst::Return(Value::ImmInt(0)),
        ]);
        verify_ir(program(exclusive))
            .expect("one consumption in each mutually exclusive arm must verify");

        let mut partial_then_merge = construct_and_branch();
        partial_then_merge.extend([
            Inst::Label("then".to_string()),
            call(2),
            Inst::Jump("merge".to_string()),
            Inst::Label("else".to_string()),
            Inst::Jump("merge".to_string()),
            Inst::Label("merge".to_string()),
            call(3),
            Inst::Return(Value::ImmInt(0)),
        ]);
        assert!(
            verify_ir(program(partial_then_merge)).is_err(),
            "post-merge consumption passed after one predecessor consumed the enum owner"
        );

        let serial = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            call(1),
            call(2),
            Inst::Return(Value::ImmInt(0)),
        ];
        assert!(
            verify_ir(program(serial)).is_err(),
            "serial double consumption passed enum ownership verification"
        );

        let cyclic = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            Inst::Jump("cycle".to_string()),
            Inst::Label("cycle".to_string()),
            call(1),
            Inst::Jump("cycle".to_string()),
        ];
        assert!(
            verify_ir(program(cyclic)).is_err(),
            "loop-carried enum consumption passed fixed-point verification"
        );

        let fresh_result_cycle = vec![
            Inst::Jump("cycle".to_string()),
            Inst::Label("cycle".to_string()),
            checked_variant(Value::Reg(0), schema.clone(), 0),
            call(1),
            Inst::Jump("cycle".to_string()),
        ];
        verify_ir(program(fresh_result_cycle))
            .expect("an exact fresh enum result before each cyclic consumption must verify");

        let fresh_double_consumption = vec![
            Inst::Jump("cycle".to_string()),
            Inst::Label("cycle".to_string()),
            checked_variant(Value::Reg(0), schema.clone(), 0),
            call(1),
            call(2),
            Inst::Jump("cycle".to_string()),
        ];
        assert!(
            verify_ir(program(fresh_double_consumption)).is_err(),
            "two consumptions after one loop-local enum definition passed verification"
        );

        let bypassed_fresh_definition = vec![
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(2),
                left: Value::ImmInt(0),
                right: Value::ImmInt(0),
            },
            Inst::Branch {
                condition: Value::Reg(2),
                true_label: "define".to_string(),
                false_label: "consume".to_string(),
            },
            Inst::Label("define".to_string()),
            checked_variant(Value::Reg(0), schema.clone(), 0),
            Inst::Jump("consume".to_string()),
            Inst::Label("consume".to_string()),
            call(1),
            Inst::Return(Value::ImmInt(0)),
        ];
        assert!(
            verify_ir(program(bypassed_fresh_definition)).is_err(),
            "enum consumption passed when one predecessor bypassed the fresh definition"
        );

        let place = |result| Inst::CheckedMutableOwnedPlaceAlloca {
            result: Value::Reg(result),
            name: "owner".to_string(),
            ty: logical.clone(),
        };
        let load = |result| Inst::Load(Value::Reg(result), Value::Reg(5));
        let replace = |value| Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(5),
            value: Value::Reg(value),
            ty: logical.clone(),
        };
        let place_call = |argument, result| Inst::Call {
            function: "consume".to_string(),
            arguments: vec![Value::Reg(argument)],
            result: Some(Value::Reg(result)),
        };

        let replaced_place = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            load(1),
            place_call(1, 2),
            checked_variant(Value::Reg(3), schema.clone(), 1),
            replace(3),
            load(4),
            place_call(4, 6),
            Inst::Return(Value::ImmInt(0)),
        ];
        verify_ir(program(replaced_place))
            .expect("exact replacement must restore one consumable enum place owner");

        let maybe_moved_then_reinitialized = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(6),
                left: Value::ImmInt(0),
                right: Value::ImmInt(0),
            },
            Inst::Branch {
                condition: Value::Reg(6),
                true_label: "then".to_string(),
                false_label: "else".to_string(),
            },
            Inst::Label("then".to_string()),
            load(1),
            place_call(1, 2),
            Inst::Jump("merge".to_string()),
            Inst::Label("else".to_string()),
            Inst::Jump("merge".to_string()),
            Inst::Label("merge".to_string()),
            checked_variant(Value::Reg(3), schema.clone(), 1),
            replace(3),
            load(4),
            place_call(4, 7),
            Inst::Return(Value::ImmInt(0)),
        ];
        verify_ir(program(maybe_moved_then_reinitialized.clone()))
            .expect("exact checked assignment must kill a maybe-consumed enum place at a CFG join");

        let mut missing_reinitialization = maybe_moved_then_reinitialized.clone();
        missing_reinitialization.remove(13);
        assert!(
            verify_ir(program(missing_reinitialization)).is_err(),
            "post-join enum consumption passed after removing the reinitializing write"
        );

        let mut generic_reinitialization = maybe_moved_then_reinitialized.clone();
        generic_reinitialization[13] = Inst::Store(Value::Reg(5), Value::Reg(3));
        assert!(
            verify_ir(program(generic_reinitialization)).is_err(),
            "generic Store manufactured enum reinitialization at a CFG join"
        );

        let mut wrong_reinitialization_schema = maybe_moved_then_reinitialized.clone();
        wrong_reinitialization_schema[13] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(5),
            value: Value::Reg(3),
            ty: unit_schema("Other", &["A", "B"]).logical_type(),
        };
        assert!(
            verify_ir(program(wrong_reinitialization_schema)).is_err(),
            "wrong-schema checked assignment manufactured enum reinitialization"
        );

        let mut nondominating_reinitialization = maybe_moved_then_reinitialized.clone();
        nondominating_reinitialization.remove(12);
        nondominating_reinitialization.insert(8, checked_variant(Value::Reg(3), schema.clone(), 1));
        assert!(
            verify_ir(program(nondominating_reinitialization)).is_err(),
            "one-arm enum result manufactured reinitialization without dominating the join"
        );

        let fresh_place_cycle = vec![
            Inst::Jump("cycle".to_string()),
            Inst::Label("cycle".to_string()),
            checked_variant(Value::Reg(0), schema.clone(), 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            load(1),
            place_call(1, 2),
            Inst::Jump("cycle".to_string()),
        ];
        verify_ir(program(fresh_place_cycle))
            .expect("an exact fresh mutable enum place per loop iteration must verify");

        let balanced_place_cycle = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            Inst::Jump("cycle".to_string()),
            Inst::Label("cycle".to_string()),
            load(1),
            place_call(1, 2),
            checked_variant(Value::Reg(3), schema.clone(), 1),
            replace(3),
            Inst::Jump("cycle".to_string()),
        ];
        verify_ir(program(balanced_place_cycle.clone())).expect(
            "an exact checked reinitialization before every owned-place backedge must verify",
        );

        let mut missing_cycle_reinitialization = balanced_place_cycle.clone();
        missing_cycle_reinitialization.remove(8);
        assert!(
            verify_ir(program(missing_cycle_reinitialization)).is_err(),
            "a consumed enum place reached a cycle without its checked reinitialization"
        );

        let mut generic_cycle_reinitialization = balanced_place_cycle.clone();
        generic_cycle_reinitialization[8] = Inst::Store(Value::Reg(5), Value::Reg(3));
        assert!(
            verify_ir(program(generic_cycle_reinitialization)).is_err(),
            "generic Store manufactured balanced loop enum ownership"
        );

        let mut wrong_cycle_schema = balanced_place_cycle.clone();
        wrong_cycle_schema[8] = Inst::CheckedOwnedPlaceAssignment {
            target: Value::Reg(5),
            value: Value::Reg(3),
            ty: unit_schema("Other", &["A", "B"]).logical_type(),
        };
        assert!(
            verify_ir(program(wrong_cycle_schema)).is_err(),
            "wrong-schema checked assignment manufactured balanced loop ownership"
        );

        let bypassed_cycle_reinitialization = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            Inst::Jump("cycle".to_string()),
            Inst::Label("cycle".to_string()),
            load(1),
            place_call(1, 2),
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(6),
                left: Value::ImmInt(0),
                right: Value::ImmInt(0),
            },
            Inst::Branch {
                condition: Value::Reg(6),
                true_label: "repair".to_string(),
                false_label: "cycle".to_string(),
            },
            Inst::Label("repair".to_string()),
            checked_variant(Value::Reg(3), schema.clone(), 1),
            replace(3),
            Inst::Jump("cycle".to_string()),
        ];
        assert!(
            verify_ir(program(bypassed_cycle_reinitialization)).is_err(),
            "one cyclic predecessor bypassed balanced enum reinitialization"
        );

        let bypassed_break_reinitialization = vec![
            checked_variant(Value::Reg(0), schema.clone(), 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            load(1),
            place_call(1, 2),
            Inst::ICmp {
                op: "eq".to_string(),
                result: Value::Reg(6),
                left: Value::ImmInt(0),
                right: Value::ImmInt(0),
            },
            Inst::Branch {
                condition: Value::Reg(6),
                true_label: "repair".to_string(),
                false_label: "exit".to_string(),
            },
            Inst::Label("repair".to_string()),
            checked_variant(Value::Reg(3), schema.clone(), 1),
            replace(3),
            Inst::Jump("exit".to_string()),
            Inst::Label("exit".to_string()),
            load(4),
            place_call(4, 7),
            Inst::Return(Value::ImmInt(0)),
        ];
        assert!(
            verify_ir(program(bypassed_break_reinitialization)).is_err(),
            "one loop-exit predecessor bypassed balanced enum reinitialization"
        );

        let consumed_place = vec![
            checked_variant(Value::Reg(0), schema, 0),
            place(5),
            Inst::Store(Value::Reg(5), Value::Reg(0)),
            load(1),
            place_call(1, 2),
            load(3),
            place_call(3, 4),
            Inst::Return(Value::ImmInt(0)),
        ];
        assert!(
            verify_ir(program(consumed_place)).is_err(),
            "a second load and consumption passed without exact enum place replacement"
        );
    }

    #[test]
    fn mixed_reference_signature_identity_is_fail_closed() {
        let immutable_int = LogicalType::ImmutableReference {
            pointee: Box::new(LogicalType::Int),
        };
        let mutable_int = LogicalType::MutableReference {
            pointee: Box::new(LogicalType::Int),
        };
        let signature = || {
            vec![
                ("left".to_string(), immutable_int.clone()),
                ("value".to_string(), mutable_int.clone()),
                ("right".to_string(), immutable_int.clone()),
                ("bias".to_string(), LogicalType::Int),
            ]
        };
        let callee = || {
            vec![
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(0),
                    parameter: "left".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::CheckedMutableReferenceParameter {
                    result: Value::Reg(1),
                    parameter: "value".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(2),
                    parameter: "right".to_string(),
                    pointee: LogicalType::Int,
                },
                Inst::Alloca(Value::Reg(3), "bias".to_string()),
                Inst::Return(Value::ImmInt(0)),
            ]
        };
        let caller = || {
            vec![
                Inst::Alloca(Value::Reg(0), "left_owner".to_string()),
                Inst::Store(Value::Reg(0), Value::ImmInt(1)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                    pointee: LogicalType::Int,
                },
                Inst::Alloca(Value::Reg(2), "right_owner".to_string()),
                Inst::Store(Value::Reg(2), Value::ImmInt(2)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(3),
                    source: Value::Reg(2),
                    pointee: LogicalType::Int,
                },
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(4),
                    name: "owner".to_string(),
                    ty: LogicalType::Int,
                },
                Inst::Store(Value::Reg(4), Value::ImmInt(3)),
                Inst::CheckedMutableBorrow {
                    result: Value::Reg(5),
                    source: Value::Reg(4),
                    pointee: LogicalType::Int,
                },
                Inst::Call {
                    function: "mix".to_string(),
                    arguments: vec![
                        Value::Reg(1),
                        Value::Reg(5),
                        Value::Reg(3),
                        Value::ImmInt(4),
                    ],
                    result: Some(Value::Reg(6)),
                },
                Inst::CheckedMutableBorrowEnd {
                    reference: Value::Reg(5),
                    source: Value::Reg(4),
                    pointee: LogicalType::Int,
                },
                Inst::Return(Value::Reg(6)),
            ]
        };
        let program =
            |parameters: Vec<(String, LogicalType)>, body: Vec<Inst>, runtime: Vec<Inst>| {
                let mut main = vec![Inst::CheckedFunctionDef {
                    name: "mix".to_string(),
                    parameters,
                    result: LogicalType::Int,
                    body,
                }];
                main.extend(runtime);
                HashMap::from([
                    (
                        "main".to_string(),
                        Function {
                            name: "main".to_string(),
                            body: main,
                            next_reg: 16,
                            next_ptr: 16,
                        },
                    ),
                    (
                        "mix".to_string(),
                        Function {
                            name: "mix".to_string(),
                            body: Vec::new(),
                            next_reg: 16,
                            next_ptr: 16,
                        },
                    ),
                ])
            };

        verify_ir(program(signature(), callee(), caller()))
            .expect("one exact mixed exclusive/shared-reference signature must verify");

        let mut duplicate_mutable_signature = signature();
        duplicate_mutable_signature[2].1 = mutable_int.clone();
        let mut wrong_immutable_binder = callee();
        wrong_immutable_binder[0] = Inst::CheckedImmutableReferenceParameter {
            result: Value::Reg(0),
            parameter: "right".to_string(),
            pointee: LogicalType::Int,
        };
        let mut missing_immutable_binder = callee();
        missing_immutable_binder.remove(2);
        let mut missing_side_argument = caller();
        if let Inst::Call { arguments, .. } = &mut missing_side_argument[9] {
            arguments.pop();
        }
        let mut wrong_argument_order = caller();
        if let Inst::Call { arguments, .. } = &mut wrong_argument_order[9] {
            arguments.swap(0, 1);
        }
        let mut raw_owner_as_immutable = caller();
        if let Inst::Call { arguments, .. } = &mut raw_owner_as_immutable[9] {
            arguments[0] = Value::Reg(0);
        }
        let mut mutable_reference_as_immutable = caller();
        if let Inst::Call { arguments, .. } = &mut mutable_reference_as_immutable[9] {
            arguments[0] = Value::Reg(5);
        }
        let mut separated_temporary = caller();
        separated_temporary.insert(
            9,
            Inst::Add(Value::Reg(7), Value::ImmInt(1), Value::ImmInt(2)),
        );
        let mut forged_end = caller();
        forged_end[10] = Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(5),
            source: Value::Reg(7),
            pointee: LogicalType::Int,
        };

        for (label, candidate) in [
            (
                "duplicate mutable-reference parameter",
                program(duplicate_mutable_signature, callee(), caller()),
            ),
            (
                "wrong immutable-reference binder identity",
                program(signature(), wrong_immutable_binder, caller()),
            ),
            (
                "missing immutable-reference binder",
                program(signature(), missing_immutable_binder, caller()),
            ),
            (
                "missing ordered argument",
                program(signature(), callee(), missing_side_argument),
            ),
            (
                "wrong ordered argument types",
                program(signature(), callee(), wrong_argument_order),
            ),
            (
                "raw owner substituted for immutable reference",
                program(signature(), callee(), raw_owner_as_immutable),
            ),
            (
                "mutable reference substituted for immutable reference",
                program(signature(), callee(), mutable_reference_as_immutable),
            ),
            (
                "separated borrow/call/end window",
                program(signature(), callee(), separated_temporary),
            ),
            (
                "forged borrow end source",
                program(signature(), callee(), forged_end),
            ),
        ] {
            assert!(
                verify_ir(candidate).is_err(),
                "{label} passed mixed-reference checked IR verification"
            );
        }
    }

    #[test]
    fn multiple_mutable_call_window_is_fail_closed() {
        let immutable_int = LogicalType::ImmutableReference {
            pointee: Box::new(LogicalType::Int),
        };
        let mutable_int = LogicalType::MutableReference {
            pointee: Box::new(LogicalType::Int),
        };
        let signature = || {
            vec![
                ("left".to_string(), mutable_int.clone()),
                ("observed".to_string(), immutable_int.clone()),
                ("right".to_string(), mutable_int.clone()),
                ("third".to_string(), mutable_int.clone()),
            ]
        };
        let mutable_binder = |result, parameter: &str| Inst::CheckedMutableReferenceParameter {
            result: Value::Reg(result),
            parameter: parameter.to_string(),
            pointee: LogicalType::Int,
        };
        let callee = || {
            vec![
                mutable_binder(0, "left"),
                Inst::CheckedImmutableReferenceParameter {
                    result: Value::Reg(1),
                    parameter: "observed".to_string(),
                    pointee: LogicalType::Int,
                },
                mutable_binder(2, "right"),
                mutable_binder(3, "third"),
                Inst::Return(Value::ImmInt(0)),
            ]
        };
        let borrow = |result, source| Inst::CheckedMutableBorrow {
            result: Value::Reg(result),
            source: Value::Reg(source),
            pointee: LogicalType::Int,
        };
        let end = |reference, source| Inst::CheckedMutableBorrowEnd {
            reference: Value::Reg(reference),
            source: Value::Reg(source),
            pointee: LogicalType::Int,
        };
        let caller = || {
            vec![
                Inst::Alloca(Value::Reg(0), "observed".to_string()),
                Inst::Store(Value::Reg(0), Value::ImmInt(4)),
                Inst::CheckedImmutableBorrow {
                    result: Value::Reg(1),
                    source: Value::Reg(0),
                    pointee: LogicalType::Int,
                },
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(2),
                    name: "left_owner".to_string(),
                    ty: LogicalType::Int,
                },
                Inst::Store(Value::Reg(2), Value::ImmInt(1)),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(3),
                    name: "right_owner".to_string(),
                    ty: LogicalType::Int,
                },
                Inst::Store(Value::Reg(3), Value::ImmInt(2)),
                Inst::CheckedMutableOwnedPlaceAlloca {
                    result: Value::Reg(4),
                    name: "third_owner".to_string(),
                    ty: LogicalType::Int,
                },
                Inst::Store(Value::Reg(4), Value::ImmInt(3)),
                borrow(5, 2),
                borrow(6, 3),
                borrow(7, 4),
                Inst::Call {
                    function: "mix_many".to_string(),
                    arguments: vec![Value::Reg(5), Value::Reg(1), Value::Reg(6), Value::Reg(7)],
                    result: Some(Value::Reg(8)),
                },
                end(7, 4),
                end(6, 3),
                end(5, 2),
                Inst::Return(Value::Reg(8)),
            ]
        };
        let program = |body: Vec<Inst>, runtime: Vec<Inst>| {
            let mut main = vec![Inst::CheckedFunctionDef {
                name: "mix_many".to_string(),
                parameters: signature(),
                result: LogicalType::Int,
                body,
            }];
            main.extend(runtime);
            HashMap::from([
                (
                    "main".to_string(),
                    Function {
                        name: "main".to_string(),
                        body: main,
                        next_reg: 20,
                        next_ptr: 20,
                    },
                ),
                (
                    "mix_many".to_string(),
                    Function {
                        name: "mix_many".to_string(),
                        body: Vec::new(),
                        next_reg: 20,
                        next_ptr: 20,
                    },
                ),
            ])
        };

        verify_ir(program(callee(), caller()))
            .expect("exact three-mutable ordered call window must verify");

        let mut duplicate_root = caller();
        duplicate_root[10] = borrow(6, 2);
        let mut reordered_borrows = caller();
        reordered_borrows.swap(9, 10);
        let mut reordered_operands = caller();
        if let Inst::Call { arguments, .. } = &mut reordered_operands[12] {
            arguments.swap(0, 2);
        }
        let mut separated_window = caller();
        separated_window.insert(
            12,
            Inst::Add(Value::Reg(9), Value::ImmInt(1), Value::ImmInt(2)),
        );
        let mut reordered_ends = caller();
        reordered_ends.swap(13, 14);
        let mut missing_end = caller();
        missing_end.remove(14);
        let mut forged_end = caller();
        forged_end[13] = end(7, 3);
        let mut raw_owner_operand = caller();
        if let Inst::Call { arguments, .. } = &mut raw_owner_operand[12] {
            arguments[2] = Value::Reg(3);
        }
        let mut duplicate_operand = caller();
        if let Inst::Call { arguments, .. } = &mut duplicate_operand[12] {
            arguments[2] = Value::Reg(5);
        }
        let mut immutable_overlap = caller();
        immutable_overlap.remove(2);
        immutable_overlap.insert(
            9,
            Inst::CheckedImmutableBorrow {
                result: Value::Reg(1),
                source: Value::Reg(2),
                pointee: LogicalType::Int,
            },
        );
        let mut wrong_binder = callee();
        wrong_binder[2] = mutable_binder(2, "third");
        let mut missing_binder = callee();
        missing_binder.remove(3);

        for (label, body, runtime) in [
            ("duplicate mutable root", callee(), duplicate_root),
            ("reordered mutable borrows", callee(), reordered_borrows),
            ("reordered mutable operands", callee(), reordered_operands),
            ("separated multi-borrow window", callee(), separated_window),
            ("reordered mutable ends", callee(), reordered_ends),
            ("missing mutable end", callee(), missing_end),
            ("forged mutable end source", callee(), forged_end),
            ("raw owner mutable operand", callee(), raw_owner_operand),
            ("duplicate mutable operand", callee(), duplicate_operand),
            (
                "mutable/immutable root overlap",
                callee(),
                immutable_overlap,
            ),
            ("wrong mutable binder", wrong_binder, caller()),
            ("missing mutable binder", missing_binder, caller()),
        ] {
            assert!(
                verify_ir(program(body, runtime)).is_err(),
                "{label} passed multiple-mutable checked IR verification"
            );
        }
    }
}
