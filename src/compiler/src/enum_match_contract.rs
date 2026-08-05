use crate::ast::{
    AstNode, Expression, MatchArm, Parameter, Pattern, Statement, Type, VariantDeclKind,
};
use crate::ir::LogicalType;
use crate::types::Ty;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnumExecutionContext {
    AdmittedFunction,
    PreservedContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnitEnumContract {
    pub(crate) name: String,
    pub(crate) variants: Vec<String>,
}

impl UnitEnumContract {
    pub(crate) fn ty(&self) -> Ty {
        Ty::Enum(self.name.clone())
    }

    pub(crate) fn variant_index(&self, variant: &str) -> Option<usize> {
        self.variants
            .iter()
            .position(|candidate| candidate == variant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnitEnumTransportType {
    pub(crate) ty: Ty,
    pub(crate) logical_type: LogicalType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct UnitEnumFunctionContract {
    pub(crate) name: String,
    pub(crate) parameters: Vec<(String, UnitEnumTransportType)>,
    pub(crate) result: UnitEnumTransportType,
}

impl UnitEnumFunctionContract {
    pub(crate) fn parameter_types(&self) -> Vec<Ty> {
        self.parameters
            .iter()
            .map(|(_, parameter)| parameter.ty.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedUnitEnumConstructor {
    pub(crate) contract: UnitEnumContract,
    pub(crate) variant_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedUnitEnumMatch {
    pub(crate) contract: UnitEnumContract,
    /// Source-arm index for each variant in declaration order.
    pub(crate) arm_for_variant: Vec<usize>,
    pub(crate) result: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnumDefinitionDisposition {
    Supported(UnitEnumContract),
    Unsupported,
    Ambiguous,
}

#[derive(Debug, Clone)]
struct RawEnumDefinition {
    variants: Vec<crate::ast::VariantDecl>,
    type_params: Vec<String>,
}

#[derive(Debug, Clone)]
enum RawEnumDefinitionDisposition {
    Unique(RawEnumDefinition),
    Ambiguous,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UnitEnumRegistry {
    definitions: BTreeMap<String, EnumDefinitionDisposition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UnitEnumError {
    PreserveExistingBehavior,
    NoUniqueDefinition(String),
    UnsupportedDefinition(String),
    UnknownVariant { enum_name: String, variant: String },
    UnexpectedPayload { enum_name: String, variant: String },
    MutableBinding,
    BindingAnnotationMismatch { expected: String, actual: String },
    ExplicitVariantPatternsRequired,
    ForeignArm { actual: String, expected: String },
    DuplicateArm(String),
    IncompleteCoverage,
    UnsupportedResult,
    ResultMismatch { expected: Ty, actual: Ty },
    ArmReusesConsumedScrutinee(String),
    DuplicateConsumption(String),
    ProcessEntryTransport,
    GenericTransportFunction,
    InvalidTransportFunction,
    InvalidTransportParameter(String),
    UnsupportedTransportParameter { function: String, parameter: String },
    UnsupportedTransportResult(String),
}

impl UnitEnumError {
    pub(crate) fn diagnostic(&self) -> String {
        match self {
            Self::PreserveExistingBehavior => "Match expressions are not supported.".to_string(),
            Self::NoUniqueDefinition(name) => {
                format!("enum `{name}` has no unique admitted unit definition")
            }
            Self::UnsupportedDefinition(name) => {
                format!("enum `{name}` is not an admitted non-generic unit enum")
            }
            Self::UnknownVariant { enum_name, variant } => {
                format!("unit enum `{enum_name}` has no variant `{variant}`")
            }
            Self::UnexpectedPayload { enum_name, variant } => format!(
                "unit enum `{enum_name}` variant `{variant}` does not accept payload data"
            ),
            Self::MutableBinding => "mutable unit-enum bindings are not admitted".to_string(),
            Self::BindingAnnotationMismatch { expected, actual } => format!(
                "unit enum binding annotation mismatch: expected {expected}, actual {actual}"
            ),
            Self::ExplicitVariantPatternsRequired =>
                "unit enum match requires one explicit payload-free variant arm per declared variant"
                    .to_string(),
            Self::ForeignArm { actual, expected } => {
                format!("unit enum match arm names `{actual}`, expected `{expected}`")
            }
            Self::DuplicateArm(variant) => {
                format!("unit enum match contains duplicate variant `{variant}`")
            }
            Self::IncompleteCoverage => {
                "unit enum match must cover every declared variant exactly once".to_string()
            }
            Self::UnsupportedResult => {
                "unit enum match arms must return Int, Float, or Bool".to_string()
            }
            Self::ResultMismatch { expected, actual } => format!(
                "unit enum match arm result mismatch: expected {expected}, actual {actual}"
            ),
            Self::ArmReusesConsumedScrutinee(name) => {
                format!("unit enum match arm reuses consumed scrutinee `{name}`")
            }
            Self::DuplicateConsumption(name) => {
                format!("unit enum `{name}` is consumed more than once in one expression")
            }
            Self::ProcessEntryTransport => {
                "process entry cannot transport unit enums".to_string()
            }
            Self::GenericTransportFunction => {
                "generic unit-enum transport functions are not admitted".to_string()
            }
            Self::InvalidTransportFunction => {
                "unit-enum transport function name is not admitted".to_string()
            }
            Self::InvalidTransportParameter(parameter) => format!(
                "unit-enum transport function defines invalid or duplicate parameter `{parameter}`"
            ),
            Self::UnsupportedTransportParameter {
                function,
                parameter,
            } => format!(
                "unit-enum transport function `{function}` parameter `{parameter}` is not an admitted by-value type"
            ),
            Self::UnsupportedTransportResult(function) => format!(
                "unit-enum transport function `{function}` result is not an admitted by-value type"
            ),
        }
    }
}

impl UnitEnumRegistry {
    pub(crate) fn from_top_level_ast(ast: &[AstNode]) -> Self {
        let mut raw = BTreeMap::<String, RawEnumDefinitionDisposition>::new();
        let mut struct_names = BTreeSet::new();
        for node in ast {
            match node {
                AstNode::Statement(Statement::StructDef { name, .. }) => {
                    struct_names.insert(name.clone());
                }
                AstNode::Statement(Statement::EnumDef {
                    name,
                    variants,
                    type_params,
                }) => {
                    let definition = RawEnumDefinition {
                        variants: variants.clone(),
                        type_params: type_params.clone(),
                    };
                    if raw.contains_key(name) {
                        raw.insert(name.clone(), RawEnumDefinitionDisposition::Ambiguous);
                    } else {
                        raw.insert(
                            name.clone(),
                            RawEnumDefinitionDisposition::Unique(definition),
                        );
                    }
                }
                _ => {}
            }
        }

        let definitions = raw
            .into_iter()
            .map(|(name, raw)| {
                let disposition = match raw {
                    RawEnumDefinitionDisposition::Ambiguous => EnumDefinitionDisposition::Ambiguous,
                    RawEnumDefinitionDisposition::Unique(_) if struct_names.contains(&name) => {
                        EnumDefinitionDisposition::Ambiguous
                    }
                    RawEnumDefinitionDisposition::Unique(definition) => {
                        Self::classify_definition(&name, definition)
                    }
                };
                (name, disposition)
            })
            .collect();
        Self { definitions }
    }

    fn classify_definition(name: &str, definition: RawEnumDefinition) -> EnumDefinitionDisposition {
        let mut names = BTreeSet::new();
        if !valid_symbol(name)
            || !definition.type_params.is_empty()
            || definition.variants.is_empty()
            || definition.variants.iter().any(|variant| {
                !valid_symbol(&variant.name)
                    || !matches!(variant.kind, VariantDeclKind::Unit)
                    || !names.insert(variant.name.clone())
            })
        {
            return EnumDefinitionDisposition::Unsupported;
        }
        EnumDefinitionDisposition::Supported(UnitEnumContract {
            name: name.to_string(),
            variants: definition
                .variants
                .into_iter()
                .map(|variant| variant.name)
                .collect(),
        })
    }

    fn contract(&self, name: &str) -> Result<UnitEnumContract, UnitEnumError> {
        match self.definitions.get(name) {
            Some(EnumDefinitionDisposition::Supported(contract)) => Ok(contract.clone()),
            Some(EnumDefinitionDisposition::Unsupported) => {
                Err(UnitEnumError::UnsupportedDefinition(name.to_string()))
            }
            Some(EnumDefinitionDisposition::Ambiguous) | None => {
                Err(UnitEnumError::NoUniqueDefinition(name.to_string()))
            }
        }
    }

    fn annotation_mentions_declared_enum(&self, annotation: &Type) -> bool {
        match annotation {
            Type::Named(name) => self.definitions.contains_key(name),
            Type::Array(element, _) | Type::Reference(element, _) => {
                self.annotation_mentions_declared_enum(element)
            }
            Type::Tuple(elements) => elements
                .iter()
                .any(|element| self.annotation_mentions_declared_enum(element)),
            Type::Generic(_, arguments) => arguments
                .iter()
                .any(|argument| self.annotation_mentions_declared_enum(argument)),
        }
    }

    fn resolve_transport_annotation<F>(
        &self,
        annotation: &Type,
        resolve_existing: &mut F,
    ) -> Result<Option<UnitEnumTransportType>, UnitEnumError>
    where
        F: FnMut(&Type) -> Option<(Ty, LogicalType)>,
    {
        if let Type::Named(name) = annotation
            && self.definitions.contains_key(name)
        {
            let contract = self.contract(name)?;
            return Ok(Some(UnitEnumTransportType {
                ty: contract.ty(),
                logical_type: LogicalType::Enum {
                    name: contract.name,
                    variants: contract.variants,
                },
            }));
        }
        Ok(resolve_existing(annotation)
            .map(|(ty, logical_type)| UnitEnumTransportType { ty, logical_type }))
    }

    pub(crate) fn resolve_function_contract<F>(
        &self,
        name: &str,
        parameters: &[Parameter],
        return_type: Option<&Type>,
        type_params: &[String],
        mut resolve_existing: F,
    ) -> Result<Option<UnitEnumFunctionContract>, UnitEnumError>
    where
        F: FnMut(&Type) -> Option<(Ty, LogicalType)>,
    {
        let mentions_enum = parameters
            .iter()
            .any(|parameter| self.annotation_mentions_declared_enum(&parameter.param_type))
            || return_type.is_some_and(|result| self.annotation_mentions_declared_enum(result));
        if !mentions_enum {
            return Ok(None);
        }
        if name == "main" {
            return Err(UnitEnumError::ProcessEntryTransport);
        }
        if !type_params.is_empty() {
            return Err(UnitEnumError::GenericTransportFunction);
        }
        if !valid_symbol(name) || name == "printf" {
            return Err(UnitEnumError::InvalidTransportFunction);
        }

        let mut seen = BTreeSet::new();
        let mut resolved_parameters = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            if !valid_symbol(&parameter.name) || !seen.insert(parameter.name.as_str()) {
                return Err(UnitEnumError::InvalidTransportParameter(
                    parameter.name.clone(),
                ));
            }
            let Some(contract) =
                self.resolve_transport_annotation(&parameter.param_type, &mut resolve_existing)?
            else {
                return Err(UnitEnumError::UnsupportedTransportParameter {
                    function: name.to_string(),
                    parameter: parameter.name.clone(),
                });
            };
            resolved_parameters.push((parameter.name.clone(), contract));
        }

        let result = match return_type {
            Some(result) => self
                .resolve_transport_annotation(result, &mut resolve_existing)?
                .ok_or_else(|| UnitEnumError::UnsupportedTransportResult(name.to_string()))?,
            None => UnitEnumTransportType {
                ty: Ty::Void,
                logical_type: LogicalType::Void,
            },
        };
        Ok(Some(UnitEnumFunctionContract {
            name: name.to_string(),
            parameters: resolved_parameters,
            result,
        }))
    }

    pub(crate) fn resolve_constructor(
        &self,
        enum_name: &str,
        variant: &str,
        has_payload: bool,
        context: EnumExecutionContext,
    ) -> Result<ResolvedUnitEnumConstructor, UnitEnumError> {
        if context != EnumExecutionContext::AdmittedFunction
            || matches!(enum_name, "Option" | "Result")
        {
            return Err(UnitEnumError::PreserveExistingBehavior);
        }
        let contract = self.contract(enum_name)?;
        let Some(variant_index) = contract.variant_index(variant) else {
            return Err(UnitEnumError::UnknownVariant {
                enum_name: enum_name.to_string(),
                variant: variant.to_string(),
            });
        };
        if has_payload {
            return Err(UnitEnumError::UnexpectedPayload {
                enum_name: enum_name.to_string(),
                variant: variant.to_string(),
            });
        }
        Ok(ResolvedUnitEnumConstructor {
            contract,
            variant_index,
        })
    }

    pub(crate) fn validate_binding(
        &self,
        enum_name: &str,
        annotation: Option<&Type>,
        mutable: bool,
    ) -> Result<(), UnitEnumError> {
        self.contract(enum_name)?;
        if mutable {
            return Err(UnitEnumError::MutableBinding);
        }
        match annotation {
            None => Ok(()),
            Some(Type::Named(actual)) if actual == enum_name => Ok(()),
            Some(actual) => Err(UnitEnumError::BindingAnnotationMismatch {
                expected: enum_name.to_string(),
                actual: display_type(actual),
            }),
        }
    }

    pub(crate) fn is_candidate_match_scrutinee<F, G>(
        &self,
        expression: &Expression,
        context: EnumExecutionContext,
        lookup_binding: F,
        lookup_call_result: G,
    ) -> bool
    where
        F: FnOnce(&str) -> Option<Ty>,
        G: FnOnce(&str) -> Option<Ty>,
    {
        if context != EnumExecutionContext::AdmittedFunction {
            return false;
        }
        match expression {
            Expression::EnumVariant { enum_name, .. } => {
                !matches!(enum_name.as_str(), "Option" | "Result")
            }
            Expression::Identifier(name) => {
                matches!(lookup_binding(name), Some(Ty::Enum(_)))
            }
            Expression::FunctionCall { name, .. } => {
                matches!(lookup_call_result(name), Some(Ty::Enum(_)))
            }
            _ => false,
        }
    }

    fn resolve_arms(
        &self,
        contract: &UnitEnumContract,
        arms: &[MatchArm],
        consumed_scrutinees: &[String],
    ) -> Result<Vec<usize>, UnitEnumError> {
        let mut arm_for_variant = vec![usize::MAX; contract.variants.len()];
        for (arm_index, arm) in arms.iter().enumerate() {
            let Pattern::Enum {
                enum_name,
                variant,
                data: None,
            } = &arm.pattern
            else {
                return Err(UnitEnumError::ExplicitVariantPatternsRequired);
            };
            if enum_name != &contract.name {
                return Err(UnitEnumError::ForeignArm {
                    actual: enum_name.clone(),
                    expected: contract.name.clone(),
                });
            }
            let Some(variant_index) = contract.variant_index(variant) else {
                return Err(UnitEnumError::UnknownVariant {
                    enum_name: contract.name.clone(),
                    variant: variant.clone(),
                });
            };
            if arm_for_variant[variant_index] != usize::MAX {
                return Err(UnitEnumError::DuplicateArm(variant.clone()));
            }
            if let Some(name) = consumed_scrutinees
                .iter()
                .find(|name| expression_mentions_identifier(&arm.body, name))
            {
                return Err(UnitEnumError::ArmReusesConsumedScrutinee(name.clone()));
            }
            arm_for_variant[variant_index] = arm_index;
        }
        if arms.len() != contract.variants.len() || arm_for_variant.contains(&usize::MAX) {
            return Err(UnitEnumError::IncompleteCoverage);
        }
        Ok(arm_for_variant)
    }

    pub(crate) fn resolve_match_patterns(
        &self,
        scrutinee: &Ty,
        scrutinee_expression: &Expression,
        arms: &[MatchArm],
        context: EnumExecutionContext,
    ) -> Result<(UnitEnumContract, Vec<usize>), UnitEnumError> {
        if context != EnumExecutionContext::AdmittedFunction {
            return Err(UnitEnumError::PreserveExistingBehavior);
        }
        let Ty::Enum(enum_name) = scrutinee else {
            return Err(UnitEnumError::PreserveExistingBehavior);
        };
        let contract = self.contract(enum_name)?;
        let consumed = match scrutinee_expression {
            Expression::Identifier(name) => vec![name.clone()],
            _ => Vec::new(),
        };
        let mapping = self.resolve_arms(&contract, arms, &consumed)?;
        Ok((contract, mapping))
    }

    pub(crate) fn resolve_match_with_consumed(
        &self,
        scrutinee: &Ty,
        scrutinee_expression: &Expression,
        arms: &[MatchArm],
        result_types: &[Ty],
        externally_consumed: &[String],
        context: EnumExecutionContext,
    ) -> Result<ResolvedUnitEnumMatch, UnitEnumError> {
        if context != EnumExecutionContext::AdmittedFunction {
            return Err(UnitEnumError::PreserveExistingBehavior);
        }
        let Ty::Enum(enum_name) = scrutinee else {
            return Err(UnitEnumError::PreserveExistingBehavior);
        };
        let contract = self.contract(enum_name)?;
        let mut consumed = externally_consumed.to_vec();
        if let Expression::Identifier(name) = scrutinee_expression
            && !consumed.contains(name)
        {
            consumed.push(name.clone());
        }
        let arm_for_variant = self.resolve_arms(&contract, arms, &consumed)?;
        let Some(result) = result_types.first().cloned() else {
            return Err(UnitEnumError::IncompleteCoverage);
        };
        if !matches!(result, Ty::Int | Ty::Float | Ty::Bool) {
            return Err(UnitEnumError::UnsupportedResult);
        }
        for actual in result_types.iter().skip(1) {
            if actual != &result {
                return Err(UnitEnumError::ResultMismatch {
                    expected: result,
                    actual: actual.clone(),
                });
            }
        }
        Ok(ResolvedUnitEnumMatch {
            contract,
            arm_for_variant,
            result,
        })
    }

    pub(crate) fn consumed_owned_values<F, G>(
        &self,
        expression: &Expression,
        lookup_binding: F,
        lookup_function_parameters: G,
    ) -> Result<Vec<String>, UnitEnumError>
    where
        F: Fn(&str) -> Option<Ty>,
        G: Fn(&str) -> Option<Vec<Ty>>,
    {
        let mut names = Vec::new();
        collect_consumed_owned_values(
            self,
            expression,
            &mut names,
            &lookup_binding,
            &lookup_function_parameters,
        );
        let mut unique = BTreeSet::new();
        for name in &names {
            if !unique.insert(name.clone()) {
                return Err(UnitEnumError::DuplicateConsumption(name.clone()));
            }
        }
        Ok(names)
    }
}

fn collect_consumed_owned_values<F, G>(
    registry: &UnitEnumRegistry,
    expression: &Expression,
    names: &mut Vec<String>,
    lookup_binding: &F,
    lookup_function_parameters: &G,
) where
    F: Fn(&str) -> Option<Ty>,
    G: Fn(&str) -> Option<Vec<Ty>>,
{
    match expression {
        Expression::Match { expr, arms } => {
            collect_consumed_owned_values(
                registry,
                expr,
                names,
                lookup_binding,
                lookup_function_parameters,
            );
            if let Expression::Identifier(name) = expr.as_ref()
                && matches!(lookup_binding(name), Some(Ty::Enum(enum_name)) if registry.contract(&enum_name).is_ok())
            {
                names.push(name.clone());
            }
            for arm in arms {
                collect_consumed_owned_values(
                    registry,
                    &arm.body,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
        }
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            collect_consumed_owned_values(
                registry,
                left,
                names,
                lookup_binding,
                lookup_function_parameters,
            );
            collect_consumed_owned_values(
                registry,
                right,
                names,
                lookup_binding,
                lookup_function_parameters,
            );
        }
        Expression::FunctionCall { name, arguments } => {
            for argument in arguments {
                collect_consumed_owned_values(
                    registry,
                    argument,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
            if let Some(parameters) = lookup_function_parameters(name) {
                for (argument, expected) in arguments.iter().zip(parameters) {
                    if let (Expression::Identifier(name), Ty::Enum(expected_name)) =
                        (argument, expected)
                        && matches!(lookup_binding(name), Some(Ty::Enum(actual_name)) if actual_name == expected_name)
                    {
                        names.push(name.clone());
                    }
                }
            }
        }
        Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. }
        | Expression::TupleLiteral(arguments) => {
            for argument in arguments {
                collect_consumed_owned_values(
                    registry,
                    argument,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
        }
        Expression::MethodCall {
            object, arguments, ..
        } => {
            collect_consumed_owned_values(
                registry,
                object,
                names,
                lookup_binding,
                lookup_function_parameters,
            );
            for argument in arguments {
                collect_consumed_owned_values(
                    registry,
                    argument,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand)
        | Expression::Closure { body: operand, .. } => collect_consumed_owned_values(
            registry,
            operand,
            names,
            lookup_binding,
            lookup_function_parameters,
        ),
        Expression::IndexAccess { object, index } => {
            collect_consumed_owned_values(
                registry,
                object,
                names,
                lookup_binding,
                lookup_function_parameters,
            );
            collect_consumed_owned_values(
                registry,
                index,
                names,
                lookup_binding,
                lookup_function_parameters,
            );
        }
        Expression::ArrayLiteral(elements) => {
            for element in elements {
                collect_consumed_owned_values(
                    registry,
                    element,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
        }
        Expression::ArrayRepeat { value, .. } => collect_consumed_owned_values(
            registry,
            value,
            names,
            lookup_binding,
            lookup_function_parameters,
        ),
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_consumed_owned_values(
                    registry,
                    value,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
        }
        Expression::EnumVariant { data, .. } => {
            if let Some(data) = data {
                collect_consumed_owned_values(
                    registry,
                    data,
                    names,
                    lookup_binding,
                    lookup_function_parameters,
                );
            }
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::Identifier(_) => {}
    }
}

fn expression_mentions_identifier(expression: &Expression, target: &str) -> bool {
    match expression {
        Expression::Identifier(name) => name == target,
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            expression_mentions_identifier(left, target)
                || expression_mentions_identifier(right, target)
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. }
        | Expression::TupleLiteral(arguments)
        | Expression::ArrayLiteral(arguments) => arguments
            .iter()
            .any(|argument| expression_mentions_identifier(argument, target)),
        Expression::MethodCall {
            object, arguments, ..
        } => {
            expression_mentions_identifier(object, target)
                || arguments
                    .iter()
                    .any(|argument| expression_mentions_identifier(argument, target))
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand)
        | Expression::Closure { body: operand, .. }
        | Expression::ArrayRepeat { value: operand, .. } => {
            expression_mentions_identifier(operand, target)
        }
        Expression::IndexAccess { object, index } => {
            expression_mentions_identifier(object, target)
                || expression_mentions_identifier(index, target)
        }
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expression_mentions_identifier(value, target)),
        Expression::EnumVariant { data, .. } => data
            .as_deref()
            .is_some_and(|value| expression_mentions_identifier(value, target)),
        Expression::Match { expr, arms } => {
            expression_mentions_identifier(expr, target)
                || arms
                    .iter()
                    .any(|arm| expression_mentions_identifier(&arm.body, target))
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::StringLiteral(_) => false,
    }
}

fn valid_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn display_type(ty: &Type) -> String {
    match ty {
        Type::Named(name) => name.clone(),
        Type::Array(element, count) => format!("[{}; {count}]", display_type(element)),
        Type::Tuple(elements) => format!(
            "({})",
            elements
                .iter()
                .map(display_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Type::Reference(inner, mutable) => {
            format!(
                "&{}{}",
                if *mutable { "mut " } else { "" },
                display_type(inner)
            )
        }
        Type::Generic(name, arguments) => format!(
            "{}<{}>",
            name,
            arguments
                .iter()
                .map(display_type)
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
