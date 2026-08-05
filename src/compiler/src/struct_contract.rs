use crate::ast::{AstNode, Expression, Statement, Type};
use crate::ir::LogicalType;
use crate::types::Ty;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarStructFieldKind {
    Int,
    Float,
    Bool,
}

impl ScalarStructFieldKind {
    pub(crate) fn ty(self) -> Ty {
        match self {
            Self::Int => Ty::Int,
            Self::Float => Ty::Float,
            Self::Bool => Ty::Bool,
        }
    }

    pub(crate) fn logical_type(self) -> LogicalType {
        match self {
            Self::Int => LogicalType::Int,
            Self::Float => LogicalType::Float,
            Self::Bool => LogicalType::Bool,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructFieldContract {
    pub(crate) name: String,
    pub(crate) kind: ScalarStructFieldKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructContract {
    pub(crate) name: String,
    pub(crate) fields: Vec<StructFieldContract>,
}

impl StructContract {
    pub(crate) fn logical_type(&self) -> LogicalType {
        LogicalType::Struct {
            name: self.name.clone(),
            fields: self
                .fields
                .iter()
                .map(|field| field.kind.logical_type())
                .collect(),
        }
    }

    pub(crate) fn field(&self, name: &str) -> Option<(usize, &StructFieldContract)> {
        self.fields
            .iter()
            .enumerate()
            .find(|(_, field)| field.name == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyTypeContract {
    pub(crate) ty: Ty,
    pub(crate) logical_type: LogicalType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyStructArrayContract {
    pub(crate) element: StructContract,
    pub(crate) count: usize,
}

impl CopyStructArrayContract {
    pub(crate) fn ty(&self) -> Ty {
        Ty::Array(Box::new(Ty::Struct(self.element.name.clone())), self.count)
    }

    pub(crate) fn logical_type(&self) -> LogicalType {
        LogicalType::Array {
            element: Box::new(self.element.logical_type()),
            count: self.count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CopyStructArrayIndexDisposition {
    PreserveExistingBehavior,
    Accepted {
        contract: CopyStructArrayContract,
        index: usize,
    },
    NonConstant,
    OutOfBounds {
        index: i64,
        count: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyFunctionContract {
    pub(crate) name: String,
    pub(crate) parameters: Vec<(String, CopyTypeContract)>,
    pub(crate) result: CopyTypeContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StructDefinitionDisposition {
    Supported(StructContract),
    Unsupported,
    Ambiguous,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct StructRegistry {
    definitions: BTreeMap<String, StructDefinitionDisposition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructExecutionContext {
    AdmittedFunction,
    PreservedContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedStructConstruction {
    pub(crate) contract: StructContract,
    /// One declaration index for every construction field in written source order.
    pub(crate) source_to_declaration: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StructContractError {
    PreserveExistingBehavior,
    FieldTypeMismatch {
        struct_name: String,
        field_name: String,
        expected: Ty,
        actual: Ty,
    },
    BindingAnnotationMismatch {
        expected: String,
        actual: String,
    },
    UnknownField {
        struct_name: String,
        field_name: String,
    },
    LocalMoveOrCopy,
    UnsupportedFunctionParameter {
        parameter_name: String,
    },
    UnsupportedFunctionArrayParameter {
        parameter_name: String,
    },
    UnsupportedFunctionReturn,
    UnsupportedFunctionArrayReturn,
    ProcessEntryStructTransport,
    ProcessEntryArrayTransport,
    CopyStructArrayElementMismatch {
        expected: String,
        actual: String,
    },
    CopyStructArrayAnnotationMismatch {
        expected: String,
        actual: String,
    },
}

impl StructContractError {
    pub(crate) fn diagnostic(&self) -> String {
        match self {
            Self::PreserveExistingBehavior => {
                "Struct construction expressions are not supported.".to_string()
            }
            Self::FieldTypeMismatch {
                struct_name,
                field_name,
                expected,
                actual,
            } => format!(
                "struct `{struct_name}` field `{field_name}` type mismatch: expected {expected}, actual {actual}"
            ),
            Self::BindingAnnotationMismatch { expected, actual } => {
                format!("struct binding annotation mismatch: expected {expected}, actual {actual}")
            }
            Self::UnknownField {
                struct_name,
                field_name,
            } => format!("struct `{struct_name}` has no field `{field_name}`"),
            Self::LocalMoveOrCopy => "local struct moves and copies are not admitted".to_string(),
            Self::UnsupportedFunctionParameter { parameter_name } => format!(
                "function parameter `{parameter_name}` is not an admitted scalar or Copy-struct type"
            ),
            Self::UnsupportedFunctionArrayParameter { parameter_name } => format!(
                "function parameter `{parameter_name}` uses an unsupported fixed-array element type"
            ),
            Self::UnsupportedFunctionReturn => {
                "function return type is not an admitted scalar, Copy-struct, or Void type"
                    .to_string()
            }
            Self::UnsupportedFunctionArrayReturn => {
                "function return uses an unsupported fixed-array element type".to_string()
            }
            Self::ProcessEntryStructTransport => {
                "process entry `main` cannot use struct parameters or returns".to_string()
            }
            Self::ProcessEntryArrayTransport => {
                "process entry `main` cannot use aggregate parameters or returns".to_string()
            }
            Self::CopyStructArrayElementMismatch { expected, actual } => format!(
                "fixed Copy-struct arrays require one exact element type: expected {expected}, actual {actual}"
            ),
            Self::CopyStructArrayAnnotationMismatch { expected, actual } => format!(
                "fixed Copy-struct array annotation mismatch: expected {expected}, actual {actual}"
            ),
        }
    }
}

impl StructRegistry {
    pub(crate) fn from_top_level_ast(ast: &[AstNode]) -> Self {
        let mut registry = Self::default();
        for node in ast {
            let AstNode::Statement(Statement::StructDef {
                name,
                fields,
                type_params,
            }) = node
            else {
                continue;
            };

            let disposition = Self::classify_definition(name, fields, type_params);
            match registry.definitions.entry(name.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(disposition);
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    entry.insert(StructDefinitionDisposition::Ambiguous);
                }
            }
        }
        registry
    }

    fn classify_definition(
        name: &str,
        fields: &[crate::ast::FieldDecl],
        type_params: &[String],
    ) -> StructDefinitionDisposition {
        if !admitted_symbol(name) || !type_params.is_empty() || fields.is_empty() {
            return StructDefinitionDisposition::Unsupported;
        }

        let mut seen = HashSet::new();
        let mut contracts = Vec::with_capacity(fields.len());
        for field in fields {
            let Some(kind) = scalar_kind(&field.field_type) else {
                return StructDefinitionDisposition::Unsupported;
            };
            if !admitted_symbol(&field.name) || !seen.insert(field.name.as_str()) {
                return StructDefinitionDisposition::Unsupported;
            }
            contracts.push(StructFieldContract {
                name: field.name.clone(),
                kind,
            });
        }

        StructDefinitionDisposition::Supported(StructContract {
            name: name.to_string(),
            fields: contracts,
        })
    }

    pub(crate) fn resolve_construction(
        &self,
        name: &str,
        fields: &[(String, Expression)],
        context: StructExecutionContext,
    ) -> Result<ResolvedStructConstruction, StructContractError> {
        if context != StructExecutionContext::AdmittedFunction {
            return Err(StructContractError::PreserveExistingBehavior);
        }
        let Some(StructDefinitionDisposition::Supported(contract)) = self.definitions.get(name)
        else {
            return Err(StructContractError::PreserveExistingBehavior);
        };
        if fields.len() != contract.fields.len() {
            return Err(StructContractError::PreserveExistingBehavior);
        }

        let mut seen = HashSet::new();
        let mut source_to_declaration = Vec::with_capacity(fields.len());
        for (field_name, _) in fields {
            if !seen.insert(field_name.as_str()) {
                return Err(StructContractError::PreserveExistingBehavior);
            }
            let Some((index, _)) = contract.field(field_name) else {
                return Err(StructContractError::PreserveExistingBehavior);
            };
            source_to_declaration.push(index);
        }
        if seen.len() != contract.fields.len() {
            return Err(StructContractError::PreserveExistingBehavior);
        }

        Ok(ResolvedStructConstruction {
            contract: contract.clone(),
            source_to_declaration,
        })
    }

    pub(crate) fn validate_construction_types(
        &self,
        resolved: &ResolvedStructConstruction,
        actual_types: &[Ty],
    ) -> Result<(), StructContractError> {
        if actual_types.len() != resolved.source_to_declaration.len() {
            return Err(StructContractError::PreserveExistingBehavior);
        }
        for (actual, declaration_index) in actual_types.iter().zip(&resolved.source_to_declaration)
        {
            let field = &resolved.contract.fields[*declaration_index];
            let expected = field.kind.ty();
            if actual != &expected {
                return Err(StructContractError::FieldTypeMismatch {
                    struct_name: resolved.contract.name.clone(),
                    field_name: field.name.clone(),
                    expected,
                    actual: actual.clone(),
                });
            }
        }
        Ok(())
    }

    pub(crate) fn validate_binding_annotation(
        &self,
        struct_name: &str,
        annotation: Option<&Type>,
    ) -> Result<(), StructContractError> {
        match annotation {
            None => Ok(()),
            Some(Type::Named(name)) if name == struct_name => Ok(()),
            Some(annotation) => Err(StructContractError::BindingAnnotationMismatch {
                expected: struct_name.to_string(),
                actual: annotation_name(annotation),
            }),
        }
    }

    pub(crate) fn validate_direct_binding_initializer(
        &self,
        initializer: &Expression,
        inferred: &Ty,
    ) -> Result<(), StructContractError> {
        let Ty::Struct(struct_name) = inferred else {
            return Ok(());
        };
        if !self.is_copy_struct_name(struct_name) {
            return Err(StructContractError::LocalMoveOrCopy);
        }
        match initializer {
            Expression::StructLiteral { .. }
            | Expression::Identifier(_)
            | Expression::FunctionCall { .. }
            | Expression::IndexAccess { .. } => Ok(()),
            _ => Err(StructContractError::LocalMoveOrCopy),
        }
    }

    pub(crate) fn is_copy_struct_ty(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Struct(name) if self.is_copy_struct_name(name))
    }

    pub(crate) fn is_copy_type(&self, ty: &Ty) -> bool {
        match ty {
            Ty::Array(element, _) => self.is_copy_type(element),
            _ => ty.is_copy_type() || self.is_copy_struct_ty(ty),
        }
    }

    pub(crate) fn copy_struct_array_contract(&self, ty: &Ty) -> Option<CopyStructArrayContract> {
        let Ty::Array(element, count) = ty else {
            return None;
        };
        self.copy_struct_contract(element)
            .map(|element| CopyStructArrayContract {
                element,
                count: *count,
            })
    }

    pub(crate) fn copy_struct_array_annotation_contract(
        &self,
        annotation: &Type,
    ) -> Option<CopyStructArrayContract> {
        let Type::Array(element, count) = annotation else {
            return None;
        };
        let Type::Named(name) = element.as_ref() else {
            return None;
        };
        self.copy_struct_contract(&Ty::Struct(name.clone()))
            .map(|element| CopyStructArrayContract {
                element,
                count: *count,
            })
    }

    pub(crate) fn typed_empty_copy_struct_array_contract(
        &self,
        annotation: &Type,
        initializer: &Expression,
    ) -> Option<CopyStructArrayContract> {
        let contract = self.copy_struct_array_annotation_contract(annotation)?;
        (contract.count == 0
            && matches!(initializer, Expression::ArrayLiteral(elements) if elements.is_empty()))
        .then_some(contract)
    }

    pub(crate) fn validate_copy_struct_array_binding(
        &self,
        annotation: Option<&Type>,
        inferred: &Ty,
    ) -> Result<(), StructContractError> {
        let inferred_contract = self.copy_struct_array_contract(inferred);
        let annotated_contract = annotation
            .and_then(|annotation| self.copy_struct_array_annotation_contract(annotation));
        match (annotation, inferred_contract, annotated_contract) {
            (None, _, _) | (Some(_), None, None) => Ok(()),
            (Some(_), Some(_), Some(expected)) if expected.ty() == *inferred => Ok(()),
            (Some(_), _, Some(expected)) => {
                Err(StructContractError::CopyStructArrayAnnotationMismatch {
                    expected: expected.ty().to_string(),
                    actual: inferred.to_string(),
                })
            }
            (Some(annotation), Some(expected), None) => {
                Err(StructContractError::CopyStructArrayAnnotationMismatch {
                    expected: expected.ty().to_string(),
                    actual: annotation_name(annotation),
                })
            }
        }
    }

    pub(crate) fn validate_copy_struct_array_elements(
        &self,
        expected: &Ty,
        actual_types: impl IntoIterator<Item = Ty>,
    ) -> Result<(), StructContractError> {
        if !self.is_copy_struct_ty(expected) {
            return Ok(());
        }
        for actual in actual_types {
            if actual != *expected {
                return Err(StructContractError::CopyStructArrayElementMismatch {
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                });
            }
        }
        Ok(())
    }

    pub(crate) fn classify_copy_struct_array_index(
        &self,
        receiver: &Ty,
        index: &Expression,
    ) -> CopyStructArrayIndexDisposition {
        let Some(contract) = self.copy_struct_array_contract(receiver) else {
            return CopyStructArrayIndexDisposition::PreserveExistingBehavior;
        };
        let Some(index) = constant_integer(index) else {
            return CopyStructArrayIndexDisposition::NonConstant;
        };
        let Ok(index_usize) = usize::try_from(index) else {
            return CopyStructArrayIndexDisposition::OutOfBounds {
                index,
                count: contract.count,
            };
        };
        if index_usize >= contract.count {
            return CopyStructArrayIndexDisposition::OutOfBounds {
                index,
                count: contract.count,
            };
        }
        CopyStructArrayIndexDisposition::Accepted {
            contract,
            index: index_usize,
        }
    }

    pub(crate) fn copy_struct_contract(&self, ty: &Ty) -> Option<StructContract> {
        let Ty::Struct(name) = ty else {
            return None;
        };
        match self.definitions.get(name) {
            Some(StructDefinitionDisposition::Supported(contract)) => Some(contract.clone()),
            _ => None,
        }
    }

    pub(crate) fn resolve_copy_function_contract(
        &self,
        name: &str,
        parameters: &[crate::ast::Parameter],
        return_type: Option<&Type>,
        type_params: &[String],
    ) -> Result<Option<CopyFunctionContract>, StructContractError> {
        let mentions_aggregate_candidate = parameters
            .iter()
            .any(|parameter| self.annotation_is_aggregate_candidate(&parameter.param_type))
            || return_type.is_some_and(|result| self.annotation_is_aggregate_candidate(result));
        if !mentions_aggregate_candidate {
            return Ok(None);
        }
        if name == "main" {
            let mentions_array = parameters
                .iter()
                .any(|parameter| matches!(parameter.param_type, Type::Array(_, _)))
                || return_type.is_some_and(|result| matches!(result, Type::Array(_, _)));
            return Err(if mentions_array {
                StructContractError::ProcessEntryArrayTransport
            } else {
                StructContractError::ProcessEntryStructTransport
            });
        }
        if !type_params.is_empty() || !admitted_symbol(name) {
            return Err(StructContractError::PreserveExistingBehavior);
        }

        let mut seen_parameters = HashSet::new();
        let mut resolved_parameters = Vec::with_capacity(parameters.len());
        for parameter in parameters {
            if !admitted_symbol(&parameter.name) || !seen_parameters.insert(parameter.name.as_str())
            {
                return Err(StructContractError::PreserveExistingBehavior);
            }
            let Some(contract) = self.resolve_copy_annotation(&parameter.param_type) else {
                return Err(if matches!(parameter.param_type, Type::Array(_, _)) {
                    StructContractError::UnsupportedFunctionArrayParameter {
                        parameter_name: parameter.name.clone(),
                    }
                } else {
                    StructContractError::UnsupportedFunctionParameter {
                        parameter_name: parameter.name.clone(),
                    }
                });
            };
            resolved_parameters.push((parameter.name.clone(), contract));
        }

        let result = match return_type {
            Some(result) => self.resolve_copy_annotation(result).ok_or_else(|| {
                if matches!(result, Type::Array(_, _)) {
                    StructContractError::UnsupportedFunctionArrayReturn
                } else {
                    StructContractError::UnsupportedFunctionReturn
                }
            })?,
            None => CopyTypeContract {
                ty: Ty::Void,
                logical_type: LogicalType::Void,
            },
        };
        Ok(Some(CopyFunctionContract {
            name: name.to_string(),
            parameters: resolved_parameters,
            result,
        }))
    }

    pub(crate) fn resolve_field(
        &self,
        receiver: &Ty,
        field_name: &str,
        context: StructExecutionContext,
    ) -> Result<(StructContract, usize, StructFieldContract), StructContractError> {
        if context != StructExecutionContext::AdmittedFunction {
            return Err(StructContractError::PreserveExistingBehavior);
        }
        let Ty::Struct(struct_name) = receiver else {
            return Err(StructContractError::PreserveExistingBehavior);
        };
        let Some(StructDefinitionDisposition::Supported(contract)) =
            self.definitions.get(struct_name)
        else {
            return Err(StructContractError::PreserveExistingBehavior);
        };
        let Some((index, field)) = contract.field(field_name) else {
            return Err(StructContractError::UnknownField {
                struct_name: struct_name.clone(),
                field_name: field_name.to_string(),
            });
        };
        Ok((contract.clone(), index, field.clone()))
    }

    fn is_copy_struct_name(&self, name: &str) -> bool {
        matches!(
            self.definitions.get(name),
            Some(StructDefinitionDisposition::Supported(_))
        )
    }

    fn annotation_is_copy_struct(&self, annotation: &Type) -> bool {
        matches!(annotation, Type::Named(name) if self.is_copy_struct_name(name))
    }

    fn annotation_is_aggregate_candidate(&self, annotation: &Type) -> bool {
        matches!(annotation, Type::Array(_, _)) || self.annotation_is_copy_struct(annotation)
    }

    fn resolve_copy_annotation(&self, annotation: &Type) -> Option<CopyTypeContract> {
        match annotation {
            Type::Named(name) if matches!(name.as_str(), "int" | "i32") => Some(CopyTypeContract {
                ty: Ty::Int,
                logical_type: LogicalType::Int,
            }),
            Type::Named(name) if matches!(name.as_str(), "float" | "f64") => {
                Some(CopyTypeContract {
                    ty: Ty::Float,
                    logical_type: LogicalType::Float,
                })
            }
            Type::Named(name) if name == "bool" => Some(CopyTypeContract {
                ty: Ty::Bool,
                logical_type: LogicalType::Bool,
            }),
            Type::Named(name) => {
                let Some(StructDefinitionDisposition::Supported(contract)) =
                    self.definitions.get(name)
                else {
                    return None;
                };
                Some(CopyTypeContract {
                    ty: Ty::Struct(name.clone()),
                    logical_type: contract.logical_type(),
                })
            }
            Type::Array(element, count) => {
                let element = match element.as_ref() {
                    Type::Named(name) if matches!(name.as_str(), "int" | "i32") => {
                        CopyTypeContract {
                            ty: Ty::Int,
                            logical_type: LogicalType::Int,
                        }
                    }
                    Type::Named(name) if matches!(name.as_str(), "float" | "f64") => {
                        CopyTypeContract {
                            ty: Ty::Float,
                            logical_type: LogicalType::Float,
                        }
                    }
                    Type::Named(name) => {
                        let Some(StructDefinitionDisposition::Supported(contract)) =
                            self.definitions.get(name)
                        else {
                            return None;
                        };
                        CopyTypeContract {
                            ty: Ty::Struct(name.clone()),
                            logical_type: contract.logical_type(),
                        }
                    }
                    _ => return None,
                };
                Some(CopyTypeContract {
                    ty: Ty::Array(Box::new(element.ty), *count),
                    logical_type: LogicalType::Array {
                        element: Box::new(element.logical_type),
                        count: *count,
                    },
                })
            }
            _ => None,
        }
    }
}

fn constant_integer(expression: &Expression) -> Option<i64> {
    match expression {
        Expression::IntegerLiteral(value) => Some(*value),
        Expression::Unary {
            op: crate::ast::UnaryOp::Negate,
            operand,
        } => constant_integer(operand).and_then(i64::checked_neg),
        _ => None,
    }
}

fn scalar_kind(annotation: &Type) -> Option<ScalarStructFieldKind> {
    match annotation {
        Type::Named(name) if matches!(name.as_str(), "int" | "i32") => {
            Some(ScalarStructFieldKind::Int)
        }
        Type::Named(name) if matches!(name.as_str(), "float" | "f64") => {
            Some(ScalarStructFieldKind::Float)
        }
        Type::Named(name) if name == "bool" => Some(ScalarStructFieldKind::Bool),
        _ => None,
    }
}

fn annotation_name(annotation: &Type) -> String {
    match annotation {
        Type::Named(name) => name.clone(),
        Type::Array(_, count) => format!("array[{count}]"),
        Type::Tuple(_) => "tuple".to_string(),
        Type::Reference(_, mutable) => {
            if *mutable {
                "&mut".to_string()
            } else {
                "&".to_string()
            }
        }
        Type::Generic(name, _) => name.clone(),
    }
}

fn admitted_symbol(name: &str) -> bool {
    let mut characters = name.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::FieldDecl;

    fn field(name: &str, annotation: Type) -> FieldDecl {
        FieldDecl {
            name: name.to_string(),
            field_type: annotation,
        }
    }

    fn definition(name: &str, fields: Vec<FieldDecl>, type_params: Vec<&str>) -> AstNode {
        AstNode::Statement(Statement::StructDef {
            name: name.to_string(),
            fields,
            type_params: type_params.into_iter().map(str::to_string).collect(),
        })
    }

    fn literal(name: &str, field_names: &[&str]) -> Expression {
        Expression::StructLiteral {
            name: name.to_string(),
            fields: field_names
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    (
                        (*name).to_string(),
                        Expression::IntegerLiteral(index as i64),
                    )
                })
                .collect(),
        }
    }

    #[test]
    fn finite_definition_and_alias_partition_is_exact() {
        let supported = StructRegistry::from_top_level_ast(&[definition(
            "Aliases",
            vec![
                field("a", Type::Named("int".to_string())),
                field("b", Type::Named("i32".to_string())),
                field("c", Type::Named("float".to_string())),
                field("d", Type::Named("f64".to_string())),
                field("e", Type::Named("bool".to_string())),
            ],
            vec![],
        )]);
        let Expression::StructLiteral { fields, .. } =
            literal("Aliases", &["e", "d", "c", "b", "a"])
        else {
            unreachable!()
        };
        let resolved = supported
            .resolve_construction("Aliases", &fields, StructExecutionContext::AdmittedFunction)
            .expect("all scalar aliases are supported");
        assert_eq!(resolved.source_to_declaration, [4, 3, 2, 1, 0]);
        assert_eq!(
            resolved.contract.logical_type(),
            LogicalType::Struct {
                name: "Aliases".to_string(),
                fields: vec![
                    LogicalType::Int,
                    LogicalType::Int,
                    LogicalType::Float,
                    LogicalType::Float,
                    LogicalType::Bool,
                ],
            }
        );

        let unsupported_types = [
            Type::Named("String".to_string()),
            Type::Named("Custom".to_string()),
            Type::Array(Box::new(Type::Named("int".to_string())), 1),
            Type::Tuple(vec![Type::Named("int".to_string())]),
            Type::Reference(Box::new(Type::Named("int".to_string())), false),
            Type::Generic("Box".to_string(), vec![Type::Named("int".to_string())]),
        ];
        for (index, annotation) in unsupported_types.into_iter().enumerate() {
            let name = format!("Unsupported{index}");
            let registry = StructRegistry::from_top_level_ast(&[definition(
                &name,
                vec![field("value", annotation)],
                vec![],
            )]);
            let Expression::StructLiteral { fields, .. } = literal(&name, &["value"]) else {
                unreachable!()
            };
            assert_eq!(
                registry.resolve_construction(
                    &name,
                    &fields,
                    StructExecutionContext::AdmittedFunction,
                ),
                Err(StructContractError::PreserveExistingBehavior)
            );
        }
    }

    #[test]
    fn complete_shape_context_annotation_and_field_partition_is_shared() {
        let registry = StructRegistry::from_top_level_ast(&[definition(
            "Pair",
            vec![
                field("left", Type::Named("int".to_string())),
                field("right", Type::Named("bool".to_string())),
            ],
            vec![],
        )]);
        for names in [vec!["left"], vec!["left", "extra"], vec!["left", "left"]] {
            let Expression::StructLiteral { fields, .. } = literal("Pair", &names) else {
                unreachable!()
            };
            assert_eq!(
                registry.resolve_construction(
                    "Pair",
                    &fields,
                    StructExecutionContext::AdmittedFunction,
                ),
                Err(StructContractError::PreserveExistingBehavior)
            );
        }
        let Expression::StructLiteral { fields, .. } = literal("Pair", &["right", "left"]) else {
            unreachable!()
        };
        assert!(
            registry
                .resolve_construction("Pair", &fields, StructExecutionContext::PreservedContext,)
                .is_err()
        );
        let resolved = registry
            .resolve_construction("Pair", &fields, StructExecutionContext::AdmittedFunction)
            .expect("reordered exact fields are supported");
        assert!(
            registry
                .validate_construction_types(&resolved, &[Ty::Bool, Ty::Int])
                .is_ok()
        );
        assert!(matches!(
            registry.validate_construction_types(&resolved, &[Ty::Int, Ty::Int]),
            Err(StructContractError::FieldTypeMismatch { .. })
        ));
        assert!(registry.validate_binding_annotation("Pair", None).is_ok());
        assert!(
            registry
                .validate_binding_annotation("Pair", Some(&Type::Named("Pair".to_string())))
                .is_ok()
        );
        assert!(matches!(
            registry.validate_binding_annotation("Pair", Some(&Type::Named("Other".to_string()))),
            Err(StructContractError::BindingAnnotationMismatch { .. })
        ));
        assert_eq!(
            registry
                .resolve_field(
                    &Ty::Struct("Pair".to_string()),
                    "right",
                    StructExecutionContext::AdmittedFunction,
                )
                .expect("known field")
                .1,
            1
        );
        assert!(matches!(
            registry.resolve_field(
                &Ty::Struct("Pair".to_string()),
                "missing",
                StructExecutionContext::AdmittedFunction,
            ),
            Err(StructContractError::UnknownField { .. })
        ));
        assert_eq!(
            registry.resolve_field(&Ty::Int, "right", StructExecutionContext::AdmittedFunction,),
            Err(StructContractError::PreserveExistingBehavior)
        );
    }

    #[test]
    fn duplicate_empty_generic_and_invalid_symbol_definitions_never_activate() {
        let cases = vec![
            vec![definition("Empty", vec![], vec![])],
            vec![definition(
                "Generic",
                vec![field("value", Type::Named("int".to_string()))],
                vec!["T"],
            )],
            vec![definition(
                "DuplicateField",
                vec![
                    field("value", Type::Named("int".to_string())),
                    field("value", Type::Named("int".to_string())),
                ],
                vec![],
            )],
            vec![
                definition(
                    "Duplicate",
                    vec![field("value", Type::Named("int".to_string()))],
                    vec![],
                ),
                definition(
                    "Duplicate",
                    vec![field("value", Type::Named("int".to_string()))],
                    vec![],
                ),
            ],
            vec![definition(
                "invalid-name",
                vec![field("value", Type::Named("int".to_string()))],
                vec![],
            )],
        ];
        for ast in cases {
            let registry = StructRegistry::from_top_level_ast(&ast);
            let AstNode::Statement(Statement::StructDef { name, .. }) = &ast[0] else {
                unreachable!()
            };
            let Expression::StructLiteral { fields, .. } = literal(name, &["value"]) else {
                unreachable!()
            };
            assert_eq!(
                registry.resolve_construction(
                    name,
                    &fields,
                    StructExecutionContext::AdmittedFunction,
                ),
                Err(StructContractError::PreserveExistingBehavior)
            );
        }
    }

    #[test]
    fn copy_function_classifier_closes_scalar_struct_and_flat_array_signature_product() {
        let registry = StructRegistry::from_top_level_ast(&[
            definition(
                "Packet",
                vec![
                    field("count", Type::Named("int".to_string())),
                    field("ready", Type::Named("bool".to_string())),
                ],
                vec![],
            ),
            definition(
                "Text",
                vec![field("value", Type::Named("String".to_string()))],
                vec![],
            ),
        ]);
        let parameters = vec![
            crate::ast::Parameter {
                name: "prefix".to_string(),
                param_type: Type::Named("i32".to_string()),
            },
            crate::ast::Parameter {
                name: "packet".to_string(),
                param_type: Type::Named("Packet".to_string()),
            },
            crate::ast::Parameter {
                name: "suffix".to_string(),
                param_type: Type::Named("bool".to_string()),
            },
        ];
        let contract = registry
            .resolve_copy_function_contract(
                "transport",
                &parameters,
                Some(&Type::Named("Packet".to_string())),
                &[],
            )
            .expect("signature is classifiable")
            .expect("signature contains a Copy struct");
        assert_eq!(contract.name, "transport");
        assert_eq!(contract.parameters[0].1.ty, Ty::Int);
        assert_eq!(
            contract.parameters[1].1.ty,
            Ty::Struct("Packet".to_string())
        );
        assert_eq!(contract.parameters[2].1.ty, Ty::Bool);
        assert_eq!(contract.result.ty, Ty::Struct("Packet".to_string()));
        assert_eq!(
            contract.result.logical_type,
            LogicalType::Struct {
                name: "Packet".to_string(),
                fields: vec![LogicalType::Int, LogicalType::Bool],
            }
        );
        assert!(registry.is_copy_struct_ty(&contract.result.ty));

        let unsupported = vec![crate::ast::Parameter {
            name: "text".to_string(),
            param_type: Type::Named("Text".to_string()),
        }];
        assert_eq!(
            registry.resolve_copy_function_contract(
                "mixed",
                &[
                    crate::ast::Parameter {
                        name: "packet".to_string(),
                        param_type: Type::Named("Packet".to_string()),
                    },
                    unsupported[0].clone(),
                ],
                None,
                &[],
            ),
            Err(StructContractError::UnsupportedFunctionParameter {
                parameter_name: "text".to_string(),
            })
        );
        assert_eq!(
            registry.resolve_copy_function_contract(
                "main",
                &parameters[1..2],
                Some(&Type::Named("int".to_string())),
                &[],
            ),
            Err(StructContractError::ProcessEntryStructTransport)
        );
        assert_eq!(
            registry
                .resolve_copy_function_contract(
                    "scalar_only",
                    &parameters[..1],
                    Some(&Type::Named("int".to_string())),
                    &[],
                )
                .expect("scalar signature preserves existing classifier"),
            None
        );

        let array_parameters = vec![
            crate::ast::Parameter {
                name: "integers".to_string(),
                param_type: Type::Array(Box::new(Type::Named("i32".to_string())), 0),
            },
            crate::ast::Parameter {
                name: "floats".to_string(),
                param_type: Type::Array(Box::new(Type::Named("float".to_string())), 2),
            },
            crate::ast::Parameter {
                name: "packets".to_string(),
                param_type: Type::Array(Box::new(Type::Named("Packet".to_string())), 3),
            },
        ];
        let array_contract = registry
            .resolve_copy_function_contract(
                "array_transport",
                &array_parameters,
                Some(&Type::Array(Box::new(Type::Named("Packet".to_string())), 3)),
                &[],
            )
            .expect("flat existing array signature is classifiable")
            .expect("array signature activates the shared aggregate classifier");
        assert_eq!(
            array_contract.parameters[0].1.ty,
            Ty::Array(Box::new(Ty::Int), 0)
        );
        assert_eq!(
            array_contract.parameters[1].1.logical_type,
            LogicalType::Array {
                element: Box::new(LogicalType::Float),
                count: 2,
            }
        );
        assert_eq!(
            array_contract.result.logical_type,
            LogicalType::Array {
                element: Box::new(LogicalType::Struct {
                    name: "Packet".to_string(),
                    fields: vec![LogicalType::Int, LogicalType::Bool],
                }),
                count: 3,
            }
        );

        for unsupported in [
            Type::Array(Box::new(Type::Named("bool".to_string())), 1),
            Type::Array(Box::new(Type::Named("String".to_string())), 1),
            Type::Array(
                Box::new(Type::Array(Box::new(Type::Named("int".to_string())), 1)),
                1,
            ),
        ] {
            assert_eq!(
                registry.resolve_copy_function_contract(
                    "unsupported_array",
                    &[crate::ast::Parameter {
                        name: "values".to_string(),
                        param_type: unsupported,
                    }],
                    Some(&Type::Named("int".to_string())),
                    &[],
                ),
                Err(StructContractError::UnsupportedFunctionArrayParameter {
                    parameter_name: "values".to_string(),
                })
            );
        }
        assert_eq!(
            registry.resolve_copy_function_contract(
                "main",
                &array_parameters[..1],
                Some(&Type::Named("int".to_string())),
                &[],
            ),
            Err(StructContractError::ProcessEntryArrayTransport)
        );
    }

    #[test]
    fn fixed_copy_struct_array_classifier_closes_type_annotation_element_and_index_product() {
        let registry = StructRegistry::from_top_level_ast(&[definition(
            "Packet",
            vec![
                field("count", Type::Named("int".to_string())),
                field("ready", Type::Named("bool".to_string())),
            ],
            vec![],
        )]);
        let packet = Ty::Struct("Packet".to_string());
        let array = Ty::Array(Box::new(packet.clone()), 2);
        let contract = registry
            .copy_struct_array_contract(&array)
            .expect("supported Copy struct array");
        assert_eq!(contract.ty(), array);
        assert_eq!(
            contract.logical_type(),
            LogicalType::Array {
                element: Box::new(LogicalType::Struct {
                    name: "Packet".to_string(),
                    fields: vec![LogicalType::Int, LogicalType::Bool],
                }),
                count: 2,
            }
        );
        assert!(registry.is_copy_type(&array));

        let annotation = Type::Array(Box::new(Type::Named("Packet".to_string())), 2);
        assert_eq!(
            registry
                .copy_struct_array_annotation_contract(&annotation)
                .expect("exact annotation")
                .ty(),
            array
        );
        assert!(
            registry
                .validate_copy_struct_array_binding(Some(&annotation), &array)
                .is_ok()
        );
        assert!(matches!(
            registry.validate_copy_struct_array_binding(
                Some(&Type::Array(Box::new(Type::Named("Packet".to_string())), 1,)),
                &array,
            ),
            Err(StructContractError::CopyStructArrayAnnotationMismatch { .. })
        ));
        assert!(
            registry
                .validate_copy_struct_array_elements(&packet, [packet.clone(), packet.clone()],)
                .is_ok()
        );
        assert!(matches!(
            registry
                .validate_copy_struct_array_elements(&packet, [Ty::Struct("Other".to_string())],),
            Err(StructContractError::CopyStructArrayElementMismatch { .. })
        ));

        let empty_annotation = Type::Array(Box::new(Type::Named("Packet".to_string())), 0);
        assert!(
            registry
                .typed_empty_copy_struct_array_contract(
                    &empty_annotation,
                    &Expression::ArrayLiteral(Vec::new()),
                )
                .is_some()
        );
        assert!(
            registry
                .typed_empty_copy_struct_array_contract(
                    &empty_annotation,
                    &Expression::ArrayRepeat {
                        value: Box::new(literal("Packet", &["count", "ready"])),
                        count: 0,
                    },
                )
                .is_none()
        );

        assert!(matches!(
            registry.classify_copy_struct_array_index(&array, &Expression::IntegerLiteral(1),),
            CopyStructArrayIndexDisposition::Accepted { index: 1, .. }
        ));
        assert_eq!(
            registry.classify_copy_struct_array_index(
                &array,
                &Expression::Identifier("index".to_string()),
            ),
            CopyStructArrayIndexDisposition::NonConstant
        );
        assert_eq!(
            registry.classify_copy_struct_array_index(
                &array,
                &Expression::Unary {
                    op: crate::ast::UnaryOp::Negate,
                    operand: Box::new(Expression::IntegerLiteral(1)),
                },
            ),
            CopyStructArrayIndexDisposition::OutOfBounds {
                index: -1,
                count: 2
            }
        );
        assert_eq!(
            registry.classify_copy_struct_array_index(
                &Ty::Array(Box::new(Ty::Int), 2),
                &Expression::IntegerLiteral(0),
            ),
            CopyStructArrayIndexDisposition::PreserveExistingBehavior
        );
    }
}
