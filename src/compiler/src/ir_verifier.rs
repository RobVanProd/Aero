use crate::ir::{
    BlockMetadata, CheckedIr, EnumSchema, EnumVariantSchema, FunctionMetadata, FunctionSignature,
    Inst, IrMetadata, LogicalType, PlaceId, PlaceMetadata, RawIr, ResultId, Value,
};
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
        checked_copy_struct: bool,
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
}

#[derive(Debug, Clone, Copy)]
struct Definition {
    block: usize,
    position: usize,
}

fn logical_type(type_name: &str) -> Option<LogicalType> {
    match type_name {
        "int" | "i32" => Some(LogicalType::Int),
        "float" | "f64" | "double" => Some(LogicalType::Float),
        "bool" | "i1" => Some(LogicalType::Bool),
        "string" | "str" => Some(LogicalType::String),
        "void" => Some(LogicalType::Void),
        _ => None,
    }
}

fn valid_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn valid_immutable_reference_pointee(ty: &LogicalType) -> bool {
    matches!(
        ty,
        LogicalType::Int | LogicalType::Float | LogicalType::Bool
    )
}

fn valid_enum_schema(schema: &EnumSchema) -> bool {
    let mut unique = BTreeSet::new();
    valid_symbol(&schema.name)
        && !schema.variants.is_empty()
        && schema.variants.iter().all(|variant| {
            valid_symbol(&variant.name)
                && unique.insert(&variant.name)
                && variant.payload.as_ref().is_none_or(|payload| {
                    matches!(
                        payload,
                        LogicalType::Int | LogicalType::Float | LogicalType::Bool
                    )
                })
        })
}

fn valid_struct_schema(fields: &[LogicalType]) -> bool {
    !fields.is_empty() && fields.iter().all(valid_copy_struct_field_type)
}

fn valid_copy_struct_field_type(logical_type: &LogicalType) -> bool {
    match logical_type {
        LogicalType::Int | LogicalType::Float | LogicalType::Bool => true,
        logical_type @ LogicalType::Struct { .. } => valid_copy_struct_type(logical_type),
        LogicalType::Array { element, .. } => {
            matches!(element.as_ref(), LogicalType::Int | LogicalType::Float)
                || valid_copy_struct_type(element)
        }
        LogicalType::Void | LogicalType::String | LogicalType::Enum { .. } => false,
    }
}

fn valid_copy_struct_type(logical_type: &LogicalType) -> bool {
    matches!(
        logical_type,
        LogicalType::Struct { name, fields }
            if valid_symbol(name) && valid_struct_schema(fields)
    )
}

fn valid_checked_transport_type(logical_type: &LogicalType) -> bool {
    match logical_type {
        LogicalType::Int | LogicalType::Float | LogicalType::Bool => true,
        logical_type @ LogicalType::Struct { .. } => valid_copy_struct_type(logical_type),
        LogicalType::Array { element, .. } => {
            matches!(element.as_ref(), LogicalType::Int | LogicalType::Float)
                || valid_copy_struct_type(element)
        }
        LogicalType::Enum { name, variants } => valid_enum_schema(&EnumSchema {
            name: name.clone(),
            variants: variants.clone(),
        }),
        LogicalType::Void | LogicalType::String => false,
    }
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
    if !valid_symbol(function) {
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
    if !valid_symbol(function) {
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
        if !valid_checked_transport_type(ty) {
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
    let mentions_aggregate = parameters.iter().any(|(_, ty)| {
        matches!(
            ty,
            LogicalType::Struct { .. } | LogicalType::Array { .. } | LogicalType::Enum { .. }
        )
    }) || matches!(
        result,
        LogicalType::Struct { .. } | LogicalType::Array { .. } | LogicalType::Enum { .. }
    );
    if !mentions_aggregate {
        return Err(IrVerificationError::new(
            function,
            None,
            IrVerificationErrorKind::MetadataMismatch(
                "checked function definition requires an aggregate- or enum-bearing signature"
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
        if !valid_symbol(&function.name) {
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
        | Inst::CheckedEnumPayload { result, .. } => Some(result),
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
        | Inst::AllocaArray { result, .. }
        | Inst::GetElementPtr { result, .. }
        | Inst::CheckedCopyStructArrayAlloca { result, .. }
        | Inst::CheckedCopyStructArrayElementPtr { result, .. }
        | Inst::CheckedStructAlloca { result, .. }
        | Inst::CheckedStructFieldPtr { result, .. }
        | Inst::CheckedImmutableBorrow { result, .. } => Some(result),
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
            Value::Reg(id) => results.get(&ResultId(*id)).cloned(),
            Value::ImmString(_) => None,
        },
        Inst::Load(_, place) => reg(place)
            .and_then(|id| places.get(&PlaceId(id)))
            .and_then(PlaceType::logical),
        Inst::Call { function, .. } => signatures.get(function).map(|sig| sig.result.clone()),
        Inst::CheckedEnumVariant { schema, .. } | Inst::CheckedEnumParameter { schema, .. } => {
            Some(schema.logical_type())
        }
        Inst::CheckedEnumPayload {
            schema,
            variant_index,
            ..
        } => schema
            .variants
            .get(*variant_index)
            .and_then(|variant| variant.payload.clone()),
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
            dominators,
            infer_bool_places,
        };
        if let Some(metadata) = seed {
            for (id, place) in &metadata.places {
                let place_type = match &place.pointee {
                    LogicalType::Array { element, count } => PlaceType::Array {
                        logical_element: Some((**element).clone()),
                        physical_element: match element.as_ref() {
                            LogicalType::Bool => "i1".to_string(),
                            LogicalType::Struct { name, .. } => {
                                format!("%aero.struct.{name}")
                            }
                            _ => "double".to_string(),
                        },
                        count: *count,
                        checked_copy_struct: matches!(element.as_ref(), LogicalType::Struct { .. }),
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
                            checked_copy_struct: false,
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
                                "checked Copy-struct array alloca",
                            ),
                        ));
                    };
                    if !valid_copy_struct_type(element) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::UnsupportedType(format!(
                                "checked Copy-struct array element `{element}`"
                            )),
                        ));
                    }
                    let LogicalType::Struct { name, .. } = element else {
                        unreachable!("validated above")
                    };
                    self.places.insert(
                        id,
                        PlaceType::Array {
                            logical_element: Some(element.clone()),
                            physical_element: format!("%aero.struct.{name}"),
                            count: *count,
                            checked_copy_struct: true,
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
                    if !valid_symbol(struct_name) || !valid_struct_schema(field_types) {
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
                                Inst::AllocaArray { .. } => "alloca array",
                                Inst::CheckedStructAlloca { .. } => "checked struct alloca",
                                Inst::CheckedStructFieldPtr { .. } => {
                                    "checked struct field pointer"
                                }
                                Inst::CheckedImmutableBorrow { .. } => "checked immutable borrow",
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
                    if self.definitions.contains_key(&ResultId(id.0)) {
                        return Err(IrVerificationError::new(
                            &self.body.name,
                            Some(&block.label),
                            IrVerificationErrorKind::IdentifierKindCollision(id.0),
                        ));
                    }
                    let (place_type, name) = match instruction {
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
                            let place_type = parameter_type.map_or(PlaceType::Numeric, |ty| {
                                match ty {
                                    LogicalType::Array { element, count } => PlaceType::Array {
                                        physical_element: match element.as_ref() {
                                            LogicalType::Int | LogicalType::Float => {
                                                "double".to_string()
                                            }
                                            LogicalType::Struct { name, .. } => {
                                                format!("%aero.struct.{name}")
                                            }
                                            _ => unreachable!(
                                                "checked signature validation admits only flat executable arrays"
                                            ),
                                        },
                                        checked_copy_struct: matches!(
                                            element.as_ref(),
                                            LogicalType::Struct { .. }
                                        ),
                                        logical_element: Some(*element),
                                        count,
                                    },
                                    ty => PlaceType::Known(ty),
                                }
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
                                    checked_copy_struct: false,
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
                                checked_copy_struct,
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
                            if *checked_copy_struct {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(
                                        "legacy getelementptr cannot address a checked Copy-struct array"
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
                            if !valid_copy_struct_type(element) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked Copy-struct array element `{element}`"
                                    )),
                                ));
                            }
                            let LogicalType::Struct { name, .. } = element else {
                                unreachable!("validated above")
                            };
                            (
                                PlaceType::Array {
                                    logical_element: Some(element.clone()),
                                    physical_element: format!("%aero.struct.{name}"),
                                    count: *count,
                                    checked_copy_struct: true,
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
                            if !valid_copy_struct_type(element) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::UnsupportedType(format!(
                                        "checked Copy-struct array element `{element}`"
                                    )),
                                ));
                            }
                            let Some(base_id) = reg(base).map(PlaceId) else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked Copy-struct array base",
                                    ),
                                ));
                            };
                            let Some(PlaceType::Array {
                                logical_element: Some(actual_element),
                                count: actual_count,
                                checked_copy_struct: true,
                                ..
                            }) = self.places.get(&base_id)
                            else {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked Copy-struct array base",
                                    ),
                                ));
                            };
                            if actual_element != element || actual_count != count {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::MetadataMismatch(
                                        "checked Copy-struct array element pointer schema/count mismatch"
                                            .to_string(),
                                    ),
                                ));
                            }
                            self.element_owners.insert(id, base_id);
                            (PlaceType::Known(element.clone()), None)
                        }
                        Inst::CheckedStructAlloca {
                            struct_name,
                            field_types,
                            ..
                        } => {
                            if !valid_symbol(struct_name) || !valid_struct_schema(field_types) {
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
                            if !self.places.contains_key(&source_id) {
                                return Err(IrVerificationError::new(
                                    &self.body.name,
                                    Some(&block.label),
                                    IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                        "checked immutable borrow source",
                                    ),
                                ));
                            }
                            (PlaceType::Known(pointee.clone()), None)
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
                            && !self.element_owners.contains_key(&id)))
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
        for block_index in 0..self.blocks.len() {
            let instructions = self.blocks[block_index].instructions.clone();
            for (position, instruction) in instructions {
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
                    | Inst::AllocaArray { .. }
                    | Inst::CheckedStructAlloca { .. }
                    | Inst::Label(_) => {}
                    Inst::Store(place, value) => {
                        let id = self.require_place(place, "store", block_index, position)?;
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
                    Inst::Load(_, place) => {
                        self.require_place(place, "load", block_index, position)?;
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
                        for (index, (argument, (_, expected))) in
                            arguments.iter().zip(&signature.parameters).enumerate()
                        {
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
                        let admitted_predicate = match &operand_type {
                            LogicalType::Int => {
                                matches!(op.as_str(), "eq" | "ne" | "slt" | "sgt" | "sle" | "sge")
                            }
                            LogicalType::Bool => matches!(op.as_str(), "eq" | "ne"),
                            _ => {
                                return Err(self.error(
                                    block_index,
                                    IrVerificationErrorKind::TypeMismatch {
                                        operation: "icmp",
                                        role: "operand",
                                        expected: "Int or Bool".to_string(),
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
                            "checked Copy-struct array base",
                            block_index,
                            position,
                        )?;
                        if !matches!(
                            self.places.get(&base),
                            Some(PlaceType::Array {
                                checked_copy_struct: true,
                                ..
                            })
                        ) {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::ExpectedPlaceIdentifier(
                                    "checked Copy-struct array base",
                                ),
                            ));
                        }
                        self.require_type(
                            index,
                            &LogicalType::Int,
                            "checked Copy-struct array element pointer",
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
                                    "checked Copy-struct array constant index {constant} is outside 0..{count}"
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
                    Inst::CheckedImmutableBorrow {
                        source, pointee, ..
                    } => {
                        let source = self.require_place(
                            source,
                            "checked immutable borrow source",
                            block_index,
                            position,
                        )?;
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
                        self.require_type(
                            value,
                            &schema.logical_type(),
                            "checked enum payload extraction",
                            "value",
                            block_index,
                            position,
                        )?;
                        let block_label = &self.blocks[block_index].label;
                        let incoming = self
                            .blocks
                            .iter()
                            .filter(|block| block.successors.contains(block_label))
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
                                            && targets.get(*variant_index) == Some(block_label)
                                    )
                                });
                        if !guarded {
                            return Err(self.error(
                                block_index,
                                IrVerificationErrorKind::MetadataMismatch(format!(
                                    "checked enum payload extraction for `{}` is not uniquely guarded by its selected variant target",
                                    schema.variants[*variant_index].name
                                )),
                            ));
                        }
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
            return register_type(element, schemas, enum_schemas);
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
        if !valid_symbol(name) || !valid_struct_schema(fields) {
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
        schemas: &BTreeMap<String, Vec<LogicalType>>,
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
                Inst::CheckedEnumVariant { schema, .. }
                | Inst::CheckedEnumPayload { schema, .. }
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
                "direct nested array field",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(0),
                        struct_name: "NestedArray".to_string(),
                        field_types: vec![LogicalType::Array {
                            element: Box::new(LogicalType::Array {
                                element: Box::new(LogicalType::Int),
                                count: 1,
                            }),
                            count: 1,
                        }],
                    },
                    Inst::Return(Value::ImmInt(0)),
                ],
            ),
            (
                "Bool array field",
                vec![
                    Inst::CheckedStructAlloca {
                        result: Value::Reg(0),
                        struct_name: "BoolArray".to_string(),
                        field_types: vec![LogicalType::Array {
                            element: Box::new(LogicalType::Bool),
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
}
