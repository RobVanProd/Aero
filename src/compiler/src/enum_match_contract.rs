use crate::ast::{
    AstNode, Expression, MatchArm, Parameter, Pattern, Statement, Type, VariantDeclKind,
};
use crate::ir::{EnumSchema, EnumVariantSchema, LogicalType};
use crate::primitive_contract::PrimitiveKind;
use crate::struct_contract::StructRegistry;
use crate::types::Ty;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnumExecutionContext {
    AdmittedFunction,
    PreservedContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumContract {
    pub(crate) schema: EnumSchema,
}

impl EnumContract {
    pub(crate) fn ty(&self) -> Ty {
        Ty::Enum(self.schema.name.clone())
    }

    pub(crate) fn variant_index(&self, variant: &str) -> Option<usize> {
        self.schema
            .variants
            .iter()
            .position(|candidate| candidate.name == variant)
    }

    pub(crate) fn payload_types(&self, variant_index: usize) -> Vec<Ty> {
        match self.schema.variants[variant_index].payload.as_ref() {
            None => Vec::new(),
            Some(LogicalType::EnumFields { fields }) => {
                fields.iter().map(copy_logical_ty).collect()
            }
            Some(payload) => vec![copy_logical_ty(payload)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumTransportType {
    pub(crate) ty: Ty,
    pub(crate) logical_type: LogicalType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumFunctionContract {
    pub(crate) name: String,
    pub(crate) parameters: Vec<(String, EnumTransportType)>,
    pub(crate) result: EnumTransportType,
}

impl EnumFunctionContract {
    pub(crate) fn parameter_types(&self) -> Vec<Ty> {
        self.parameters
            .iter()
            .map(|(_, parameter)| parameter.ty.clone())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedEnumConstructor {
    pub(crate) contract: EnumContract,
    pub(crate) variant_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EnumPayloadBinding {
    pub(crate) name: String,
    pub(crate) ty: Ty,
    pub(crate) variant_index: usize,
    pub(crate) field_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedEnumPatterns {
    pub(crate) contract: EnumContract,
    pub(crate) arm_for_variant: Vec<usize>,
    pub(crate) payload_bindings: Vec<Vec<EnumPayloadBinding>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedEnumMatch {
    pub(crate) contract: EnumContract,
    /// Source-arm index for each variant in declaration order.
    pub(crate) arm_for_variant: Vec<usize>,
    pub(crate) payload_bindings: Vec<Vec<EnumPayloadBinding>>,
    pub(crate) result: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnedEnumMatchResult {
    pub(crate) contract: EnumContract,
    /// One direct-owner consumption set for each reachable result leaf. Empty sets
    /// represent fresh origins; repeated names in different sets are mutually
    /// exclusive rather than duplicate moves on one runtime path.
    pub(crate) consumption_paths: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnumDefinitionDisposition {
    Supported(EnumContract),
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
pub(crate) struct EnumRegistry {
    definitions: BTreeMap<String, EnumDefinitionDisposition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum EnumError {
    PreserveExistingBehavior,
    NoUniqueDefinition(String),
    UnsupportedDefinition(String),
    UnknownVariant {
        enum_name: String,
        variant: String,
    },
    UnexpectedPayload {
        enum_name: String,
        variant: String,
    },
    MissingPayload {
        enum_name: String,
        variant: String,
        expected: Ty,
    },
    EmptyPositionalPayload {
        enum_name: String,
        variant: String,
    },
    ConstructorArityMismatch {
        enum_name: String,
        variant: String,
        expected: usize,
        actual: usize,
    },
    PayloadTypeMismatch {
        enum_name: String,
        variant: String,
        expected: Ty,
        actual: Ty,
    },
    BindingAnnotationMismatch {
        expected: String,
        actual: String,
    },
    ExplicitVariantPatternsRequired,
    MissingIdentifierPayloadBinding(String),
    PatternArityMismatch {
        variant: String,
        expected: usize,
        actual: usize,
    },
    DuplicatePayloadBinding(String),
    UnexpectedPayloadBinding(String),
    PayloadBindingShadowsConsumed(String),
    ForeignArm {
        actual: String,
        expected: String,
    },
    DuplicateArm(String),
    IncompleteCoverage,
    UnsupportedResult,
    UnsupportedOwnedResultOrigin(usize),
    ResultMismatch {
        expected: Ty,
        actual: Ty,
    },
    ArmReusesConsumedScrutinee(String),
    DuplicateConsumption(String),
    ProcessEntryTransport,
    GenericTransportFunction,
    InvalidTransportFunction,
    InvalidTransportParameter(String),
    UnsupportedTransportParameter {
        function: String,
        parameter: String,
    },
    UnsupportedTransportResult(String),
}

impl EnumError {
    pub(crate) fn diagnostic(&self) -> String {
        match self {
            Self::PreserveExistingBehavior => "Match expressions are not supported.".to_string(),
            Self::NoUniqueDefinition(name) => {
                format!("enum `{name}` has no unique admitted definition")
            }
            Self::UnsupportedDefinition(name) => {
                format!(
                    "enum `{name}` is not an admitted non-generic unit-or-positional-CopyData enum"
                )
            }
            Self::UnknownVariant { enum_name, variant } => {
                format!("enum `{enum_name}` has no variant `{variant}`")
            }
            Self::UnexpectedPayload { enum_name, variant } => {
                format!("enum `{enum_name}` variant `{variant}` does not accept payload data")
            }
            Self::MissingPayload {
                enum_name,
                variant,
                expected,
            } => format!(
                "enum `{enum_name}` variant `{variant}` requires one {} payload",
                expected.to_string().to_ascii_lowercase()
            ),
            Self::EmptyPositionalPayload { enum_name, variant } => format!(
                "enum `{enum_name}` variant `{variant}` cannot use an empty positional field list"
            ),
            Self::ConstructorArityMismatch {
                enum_name,
                variant,
                expected,
                actual,
            } => format!(
                "enum `{enum_name}` variant `{variant}` requires {expected} positional field(s), actual {actual}"
            ),
            Self::PayloadTypeMismatch {
                enum_name,
                variant,
                expected,
                actual,
            } => format!(
                "enum `{enum_name}` variant `{variant}` payload type mismatch: expected {expected}, actual {actual}"
            ),
            Self::BindingAnnotationMismatch { expected, actual } => {
                format!("enum binding annotation mismatch: expected {expected}, actual {actual}")
            }
            Self::ExplicitVariantPatternsRequired => {
                "enum match requires one explicit variant arm per declared variant".to_string()
            }
            Self::MissingIdentifierPayloadBinding(variant) => {
                format!("enum match variant `{variant}` requires one identifier payload binding")
            }
            Self::PatternArityMismatch {
                variant,
                expected,
                actual,
            } => format!(
                "enum match variant `{variant}` requires {expected} identifier field binding(s), actual {actual}"
            ),
            Self::DuplicatePayloadBinding(name) => {
                format!("enum match contains duplicate payload binding `{name}`")
            }
            Self::UnexpectedPayloadBinding(variant) => {
                format!("enum match variant `{variant}` does not accept a payload binding")
            }
            Self::PayloadBindingShadowsConsumed(name) => {
                format!("enum payload binding `{name}` shadows consumed scrutinee")
            }
            Self::ForeignArm { actual, expected } => {
                format!("enum match arm names `{actual}`, expected `{expected}`")
            }
            Self::DuplicateArm(variant) => {
                format!("enum match contains duplicate variant `{variant}`")
            }
            Self::IncompleteCoverage => {
                "enum match must cover every declared variant exactly once".to_string()
            }
            Self::UnsupportedResult => {
                "enum match arms must return Int, Float, Bool, Char, or one admitted owned enum result"
                    .to_string()
            }
            Self::UnsupportedOwnedResultOrigin(arm) => format!(
                "owned enum match result arm {arm} has an unsupported origin; expected a fresh constructor, exact enum-returning call without additional owned-enum consumption, initialized direct local owner, or nested admitted-result Match"
            ),
            Self::ResultMismatch { expected, actual } => {
                format!("enum match arm result mismatch: expected {expected}, actual {actual}")
            }
            Self::ArmReusesConsumedScrutinee(name) => {
                format!("enum match arm reuses consumed scrutinee `{name}`")
            }
            Self::DuplicateConsumption(name) => {
                format!("enum `{name}` is consumed more than once in one expression")
            }
            Self::ProcessEntryTransport => "process entry cannot transport enums".to_string(),
            Self::GenericTransportFunction => {
                "generic enum transport functions are not admitted".to_string()
            }
            Self::InvalidTransportFunction => {
                "enum transport function name is not admitted".to_string()
            }
            Self::InvalidTransportParameter(parameter) => format!(
                "enum transport function defines invalid or duplicate parameter `{parameter}`"
            ),
            Self::UnsupportedTransportParameter {
                function,
                parameter,
            } => format!(
                "enum transport function `{function}` parameter `{parameter}` is not an admitted by-value type"
            ),
            Self::UnsupportedTransportResult(function) => format!(
                "enum transport function `{function}` result is not an admitted by-value type"
            ),
        }
    }
}

impl EnumRegistry {
    pub(crate) fn from_top_level_ast(ast: &[AstNode], structs: &StructRegistry) -> Self {
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
                        Self::classify_definition(&name, definition, structs)
                    }
                };
                (name, disposition)
            })
            .collect();
        Self { definitions }
    }

    fn classify_definition(
        name: &str,
        definition: RawEnumDefinition,
        structs: &StructRegistry,
    ) -> EnumDefinitionDisposition {
        let mut names = BTreeSet::new();
        if !valid_symbol(name)
            || !definition.type_params.is_empty()
            || definition.variants.is_empty()
        {
            return EnumDefinitionDisposition::Unsupported;
        }
        let mut variants = Vec::with_capacity(definition.variants.len());
        for variant in definition.variants {
            if !valid_symbol(&variant.name) || !names.insert(variant.name.clone()) {
                return EnumDefinitionDisposition::Unsupported;
            }
            let payload = match variant.kind {
                VariantDeclKind::Unit => None,
                VariantDeclKind::Tuple(types) if !types.is_empty() => {
                    let mut fields = Vec::with_capacity(types.len());
                    for ty in types {
                        let Some(contract) = structs.resolve_copy_annotation(&ty) else {
                            return EnumDefinitionDisposition::Unsupported;
                        };
                        fields.push(contract.logical_type);
                    }
                    match fields.len() {
                        1 => fields.into_iter().next(),
                        _ => Some(LogicalType::EnumFields { fields }),
                    }
                }
                VariantDeclKind::Tuple(_) | VariantDeclKind::Struct(_) => {
                    return EnumDefinitionDisposition::Unsupported;
                }
            };
            variants.push(EnumVariantSchema {
                name: variant.name,
                payload,
            });
        }
        EnumDefinitionDisposition::Supported(EnumContract {
            schema: EnumSchema {
                name: name.to_string(),
                variants,
            },
        })
    }

    fn contract(&self, name: &str) -> Result<EnumContract, EnumError> {
        match self.definitions.get(name) {
            Some(EnumDefinitionDisposition::Supported(contract)) => Ok(contract.clone()),
            Some(EnumDefinitionDisposition::Unsupported) => {
                Err(EnumError::UnsupportedDefinition(name.to_string()))
            }
            Some(EnumDefinitionDisposition::Ambiguous) | None => {
                Err(EnumError::NoUniqueDefinition(name.to_string()))
            }
        }
    }

    pub(crate) fn owned_place_logical_type(&self, name: &str) -> Result<LogicalType, EnumError> {
        self.contract(name)
            .map(|contract| contract.schema.logical_type())
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
    ) -> Result<Option<EnumTransportType>, EnumError>
    where
        F: FnMut(&Type) -> Option<(Ty, LogicalType)>,
    {
        if matches!(annotation, Type::Tuple(_)) {
            return Ok(None);
        }
        if let Type::Named(name) = annotation
            && self.definitions.contains_key(name)
        {
            let contract = self.contract(name)?;
            return Ok(Some(EnumTransportType {
                ty: contract.ty(),
                logical_type: contract.schema.logical_type(),
            }));
        }
        Ok(resolve_existing(annotation)
            .map(|(ty, logical_type)| EnumTransportType { ty, logical_type }))
    }

    pub(crate) fn resolve_function_contract<F>(
        &self,
        name: &str,
        parameters: &[Parameter],
        return_type: Option<&Type>,
        type_params: &[String],
        mut resolve_existing: F,
    ) -> Result<Option<EnumFunctionContract>, EnumError>
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
            return Err(EnumError::ProcessEntryTransport);
        }
        if !type_params.is_empty() {
            return Err(EnumError::GenericTransportFunction);
        }
        if !valid_symbol(name) || name == "printf" {
            return Err(EnumError::InvalidTransportFunction);
        }

        let mut seen = BTreeSet::new();
        let mut resolved_parameters = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            if !valid_symbol(&parameter.name) || !seen.insert(parameter.name.as_str()) {
                return Err(EnumError::InvalidTransportParameter(parameter.name.clone()));
            }
            let Some(contract) =
                self.resolve_transport_annotation(&parameter.param_type, &mut resolve_existing)?
            else {
                return Err(EnumError::UnsupportedTransportParameter {
                    function: name.to_string(),
                    parameter: parameter.name.clone(),
                });
            };
            resolved_parameters.push((parameter.name.clone(), contract));
        }

        let result = match return_type {
            Some(result) => self
                .resolve_transport_annotation(result, &mut resolve_existing)?
                .ok_or_else(|| EnumError::UnsupportedTransportResult(name.to_string()))?,
            None => EnumTransportType {
                ty: Ty::Void,
                logical_type: LogicalType::Void,
            },
        };
        Ok(Some(EnumFunctionContract {
            name: name.to_string(),
            parameters: resolved_parameters,
            result,
        }))
    }

    pub(crate) fn resolve_constructor(
        &self,
        enum_name: &str,
        variant: &str,
        payload_arity: Option<usize>,
        context: EnumExecutionContext,
    ) -> Result<ResolvedEnumConstructor, EnumError> {
        if context != EnumExecutionContext::AdmittedFunction
            || matches!(enum_name, "Option" | "Result")
        {
            return Err(EnumError::PreserveExistingBehavior);
        }
        let contract = self.contract(enum_name)?;
        let Some(variant_index) = contract.variant_index(variant) else {
            return Err(EnumError::UnknownVariant {
                enum_name: enum_name.to_string(),
                variant: variant.to_string(),
            });
        };
        let expected = contract.payload_types(variant_index);
        match (expected.as_slice(), payload_arity) {
            ([], None) => {}
            ([], Some(0)) => {
                return Err(EnumError::EmptyPositionalPayload {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                });
            }
            ([], Some(_)) => {
                return Err(EnumError::UnexpectedPayload {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                });
            }
            ([expected], None) => {
                return Err(EnumError::MissingPayload {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                    expected: expected.clone(),
                });
            }
            (_, None) => {
                return Err(EnumError::ConstructorArityMismatch {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                    expected: expected.len(),
                    actual: 0,
                });
            }
            (_, Some(0)) => {
                return Err(EnumError::EmptyPositionalPayload {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                });
            }
            (_, Some(actual)) if actual != expected.len() => {
                return Err(EnumError::ConstructorArityMismatch {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                    expected: expected.len(),
                    actual,
                });
            }
            _ => {}
        }
        Ok(ResolvedEnumConstructor {
            contract,
            variant_index,
        })
    }

    pub(crate) fn validate_constructor_payload(
        &self,
        resolved: &ResolvedEnumConstructor,
        actual: Option<&[Ty]>,
    ) -> Result<(), EnumError> {
        let expected = resolved.contract.payload_types(resolved.variant_index);
        let actual = actual.unwrap_or_default();
        if expected == actual {
            return Ok(());
        }
        let variant = &resolved.contract.schema.variants[resolved.variant_index].name;
        if expected.len() != actual.len() {
            return Err(EnumError::ConstructorArityMismatch {
                enum_name: resolved.contract.schema.name.clone(),
                variant: variant.clone(),
                expected: expected.len(),
                actual: actual.len(),
            });
        }
        let (expected, actual) = expected
            .into_iter()
            .zip(actual)
            .find(|(expected, actual)| expected != *actual)
            .expect("different equal-length payload lists have a mismatched field");
        Err(EnumError::PayloadTypeMismatch {
            enum_name: resolved.contract.schema.name.clone(),
            variant: variant.clone(),
            expected,
            actual: actual.clone(),
        })
    }

    pub(crate) fn validate_binding(
        &self,
        enum_name: &str,
        annotation: Option<&Type>,
    ) -> Result<(), EnumError> {
        self.contract(enum_name)?;
        match annotation {
            None => Ok(()),
            Some(Type::Named(actual)) if actual == enum_name => Ok(()),
            Some(actual) => Err(EnumError::BindingAnnotationMismatch {
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
        contract: &EnumContract,
        arms: &[MatchArm],
        consumed_scrutinees: &[String],
    ) -> Result<(Vec<usize>, Vec<Vec<EnumPayloadBinding>>), EnumError> {
        let mut arm_for_variant = vec![usize::MAX; contract.schema.variants.len()];
        let mut payload_bindings = vec![Vec::new(); arms.len()];
        for (arm_index, arm) in arms.iter().enumerate() {
            let Pattern::Enum {
                enum_name,
                variant,
                data,
            } = &arm.pattern
            else {
                return Err(EnumError::ExplicitVariantPatternsRequired);
            };
            if enum_name != &contract.schema.name {
                return Err(EnumError::ForeignArm {
                    actual: enum_name.clone(),
                    expected: contract.schema.name.clone(),
                });
            }
            let Some(variant_index) = contract.variant_index(variant) else {
                return Err(EnumError::UnknownVariant {
                    enum_name: contract.schema.name.clone(),
                    variant: variant.clone(),
                });
            };
            if arm_for_variant[variant_index] != usize::MAX {
                return Err(EnumError::DuplicateArm(variant.clone()));
            }
            let expected_fields = contract.payload_types(variant_index);
            match (expected_fields.as_slice(), data.as_deref()) {
                ([], None) => {}
                ([], Some(_)) => {
                    return Err(EnumError::UnexpectedPayloadBinding(variant.clone()));
                }
                ([_], None) => {
                    return Err(EnumError::MissingIdentifierPayloadBinding(variant.clone()));
                }
                (_, None) => {
                    return Err(EnumError::PatternArityMismatch {
                        variant: variant.clone(),
                        expected: expected_fields.len(),
                        actual: 0,
                    });
                }
                (_, Some(patterns)) if patterns.len() != expected_fields.len() => {
                    return Err(EnumError::PatternArityMismatch {
                        variant: variant.clone(),
                        expected: expected_fields.len(),
                        actual: patterns.len(),
                    });
                }
                (_, Some(patterns)) => {
                    let mut seen = BTreeSet::new();
                    let mut bindings = Vec::with_capacity(patterns.len());
                    for (field_index, (pattern, ty)) in
                        patterns.iter().zip(expected_fields).enumerate()
                    {
                        let Pattern::Identifier(name) = pattern else {
                            return Err(EnumError::MissingIdentifierPayloadBinding(
                                variant.clone(),
                            ));
                        };
                        if !valid_symbol(name) {
                            return Err(EnumError::MissingIdentifierPayloadBinding(
                                variant.clone(),
                            ));
                        }
                        if !seen.insert(name.clone()) {
                            return Err(EnumError::DuplicatePayloadBinding(name.clone()));
                        }
                        if consumed_scrutinees.contains(name) {
                            return Err(EnumError::PayloadBindingShadowsConsumed(name.clone()));
                        }
                        bindings.push(EnumPayloadBinding {
                            name: name.clone(),
                            ty,
                            variant_index,
                            field_index,
                        });
                    }
                    payload_bindings[arm_index] = bindings;
                }
            }
            if let Some(name) = consumed_scrutinees
                .iter()
                .find(|name| expression_mentions_identifier(&arm.body, name))
            {
                return Err(EnumError::ArmReusesConsumedScrutinee(name.clone()));
            }
            arm_for_variant[variant_index] = arm_index;
        }
        if arms.len() != contract.schema.variants.len() || arm_for_variant.contains(&usize::MAX) {
            return Err(EnumError::IncompleteCoverage);
        }
        Ok((arm_for_variant, payload_bindings))
    }

    pub(crate) fn resolve_match_patterns(
        &self,
        scrutinee: &Ty,
        scrutinee_expression: &Expression,
        arms: &[MatchArm],
        context: EnumExecutionContext,
    ) -> Result<ResolvedEnumPatterns, EnumError> {
        if context != EnumExecutionContext::AdmittedFunction {
            return Err(EnumError::PreserveExistingBehavior);
        }
        let Ty::Enum(enum_name) = scrutinee else {
            return Err(EnumError::PreserveExistingBehavior);
        };
        let contract = self.contract(enum_name)?;
        let consumed = match scrutinee_expression {
            Expression::Identifier(name) => vec![name.clone()],
            _ => Vec::new(),
        };
        let (arm_for_variant, payload_bindings) = self.resolve_arms(&contract, arms, &consumed)?;
        Ok(ResolvedEnumPatterns {
            contract,
            arm_for_variant,
            payload_bindings,
        })
    }

    pub(crate) fn resolve_match_with_consumed(
        &self,
        scrutinee: &Ty,
        scrutinee_expression: &Expression,
        arms: &[MatchArm],
        result_types: &[Ty],
        externally_consumed: &[String],
        arm_owned_consumptions: &[Vec<String>],
        lookup_binding: impl Fn(&str) -> Option<Ty>,
        lookup_call_result: impl Fn(&str) -> Option<Ty>,
        context: EnumExecutionContext,
    ) -> Result<ResolvedEnumMatch, EnumError> {
        if context != EnumExecutionContext::AdmittedFunction {
            return Err(EnumError::PreserveExistingBehavior);
        }
        let Ty::Enum(enum_name) = scrutinee else {
            return Err(EnumError::PreserveExistingBehavior);
        };
        let contract = self.contract(enum_name)?;
        let mut consumed = externally_consumed.to_vec();
        if let Expression::Identifier(name) = scrutinee_expression
            && !consumed.contains(name)
        {
            consumed.push(name.clone());
        }
        let (arm_for_variant, payload_bindings) = self.resolve_arms(&contract, arms, &consumed)?;
        let Some(result) = result_types.first().cloned() else {
            return Err(EnumError::IncompleteCoverage);
        };
        match &result {
            result if PrimitiveKind::from_ty(result).is_some() => {}
            Ty::Enum(_) => {
                if arm_owned_consumptions.len() != arms.len() {
                    return Err(EnumError::IncompleteCoverage);
                }
                if let Some((arm, _)) = arm_owned_consumptions
                    .iter()
                    .enumerate()
                    .find(|(_, consumed)| !consumed.is_empty())
                {
                    return Err(EnumError::UnsupportedOwnedResultOrigin(arm + 1));
                }
                let owned_result =
                    self.resolve_owned_match_result(arms, &lookup_binding, &lookup_call_result)?;
                if owned_result.contract.ty() != result {
                    return Err(EnumError::ResultMismatch {
                        expected: result,
                        actual: owned_result.contract.ty(),
                    });
                }
            }
            _ => return Err(EnumError::UnsupportedResult),
        }
        for actual in result_types.iter().skip(1) {
            if actual != &result {
                return Err(EnumError::ResultMismatch {
                    expected: result,
                    actual: actual.clone(),
                });
            }
        }
        Ok(ResolvedEnumMatch {
            contract,
            arm_for_variant,
            payload_bindings,
            result,
        })
    }

    fn owned_result_origin<F, G>(
        &self,
        expression: &Expression,
        lookup_binding: &F,
        lookup_call_result: &G,
    ) -> Option<(Ty, Vec<Vec<String>>)>
    where
        F: Fn(&str) -> Option<Ty>,
        G: Fn(&str) -> Option<Ty>,
    {
        let (ty, paths) = match expression {
            Expression::EnumVariant { enum_name, .. } => {
                (Ty::Enum(enum_name.clone()), vec![Vec::new()])
            }
            Expression::FunctionCall { name, arguments } => {
                if arguments.iter().any(|argument| {
                    self.expression_contains_conditional_owner(
                        argument,
                        lookup_binding,
                        lookup_call_result,
                    )
                }) {
                    return None;
                }
                (lookup_call_result(name)?, vec![Vec::new()])
            }
            Expression::Identifier(name) => (lookup_binding(name)?, vec![vec![name.clone()]]),
            Expression::Match { arms, .. } => {
                let result = self
                    .resolve_owned_match_result(arms, lookup_binding, lookup_call_result)
                    .ok()?;
                return Some((result.contract.ty(), result.consumption_paths));
            }
            _ => return None,
        };
        match &ty {
            Ty::Enum(name) if self.contract(name).is_ok() => Some((ty, paths)),
            _ => None,
        }
    }

    fn expression_contains_conditional_owner<F, G>(
        &self,
        expression: &Expression,
        lookup_binding: &F,
        lookup_call_result: &G,
    ) -> bool
    where
        F: Fn(&str) -> Option<Ty>,
        G: Fn(&str) -> Option<Ty>,
    {
        match expression {
            Expression::Match { arms, .. } => self
                .resolve_owned_match_result(arms, lookup_binding, lookup_call_result)
                .is_ok_and(|result| result.consumption_paths.iter().any(|path| !path.is_empty())),
            Expression::FunctionCall { arguments, .. } => arguments.iter().any(|argument| {
                self.expression_contains_conditional_owner(
                    argument,
                    lookup_binding,
                    lookup_call_result,
                )
            }),
            _ => false,
        }
    }

    pub(crate) fn resolve_owned_match_result<F, G>(
        &self,
        arms: &[MatchArm],
        lookup_binding: &F,
        lookup_call_result: &G,
    ) -> Result<OwnedEnumMatchResult, EnumError>
    where
        F: Fn(&str) -> Option<Ty>,
        G: Fn(&str) -> Option<Ty>,
    {
        let Some(first_arm) = arms.first() else {
            return Err(EnumError::IncompleteCoverage);
        };
        let Some((result, mut consumption_paths)) =
            self.owned_result_origin(&first_arm.body, lookup_binding, lookup_call_result)
        else {
            return Err(EnumError::UnsupportedOwnedResultOrigin(1));
        };
        for (arm, expression) in arms.iter().enumerate().skip(1) {
            let Some((actual, mut paths)) =
                self.owned_result_origin(&expression.body, lookup_binding, lookup_call_result)
            else {
                return Err(EnumError::UnsupportedOwnedResultOrigin(arm + 1));
            };
            if actual != result {
                return Err(EnumError::ResultMismatch {
                    expected: result,
                    actual,
                });
            }
            consumption_paths.append(&mut paths);
        }
        let Ty::Enum(name) = result else {
            return Err(EnumError::UnsupportedResult);
        };
        Ok(OwnedEnumMatchResult {
            contract: self.contract(&name)?,
            consumption_paths,
        })
    }

    pub(crate) fn consumed_owned_values<F, G>(
        &self,
        expression: &Expression,
        lookup_binding: F,
        lookup_function_parameters: G,
    ) -> Result<Vec<String>, EnumError>
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
                return Err(EnumError::DuplicateConsumption(name.clone()));
            }
        }
        Ok(names)
    }
}

fn collect_consumed_owned_values<F, G>(
    registry: &EnumRegistry,
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
            if let Some(fields) = data {
                for field in fields {
                    collect_consumed_owned_values(
                        registry,
                        field,
                        names,
                        lookup_binding,
                        lookup_function_parameters,
                    );
                }
            }
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
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
        Expression::EnumVariant { data, .. } => data.as_ref().is_some_and(|fields| {
            fields
                .iter()
                .any(|value| expression_mentions_identifier(value, target))
        }),
        Expression::Match { expr, arms } => {
            expression_mentions_identifier(expr, target)
                || arms
                    .iter()
                    .any(|arm| expression_mentions_identifier(&arm.body, target))
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_) => false,
    }
}

fn copy_logical_ty(logical_type: &LogicalType) -> Ty {
    match logical_type {
        LogicalType::Int => Ty::Int,
        LogicalType::Float => Ty::Float,
        LogicalType::Bool => Ty::Bool,
        LogicalType::Char => Ty::Char,
        LogicalType::Array { element, count } => {
            Ty::Array(Box::new(copy_logical_ty(element)), *count)
        }
        LogicalType::Tuple { elements } => {
            Ty::Tuple(elements.iter().map(copy_logical_ty).collect())
        }
        LogicalType::Struct { name, .. } => Ty::Struct(name.clone()),
        LogicalType::EnumFields { .. }
        | LogicalType::Void
        | LogicalType::String
        | LogicalType::ImmutableReference { .. }
        | LogicalType::MutableReference { .. }
        | LogicalType::Enum { .. } => {
            unreachable!("admitted enum payload schemas are recursive CopyData")
        }
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
