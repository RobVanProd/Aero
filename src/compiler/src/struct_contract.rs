use crate::ast::{AstNode, Expression, FieldDecl, Statement, Type};
use crate::ir::LogicalType;
use crate::types::Ty;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StructFieldContract {
    pub(crate) name: String,
    copy_type: CopyTypeContract,
}

impl StructFieldContract {
    pub(crate) fn ty(&self) -> Ty {
        self.copy_type.ty.clone()
    }

    pub(crate) fn logical_type(&self) -> LogicalType {
        self.copy_type.logical_type.clone()
    }
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
                .map(StructFieldContract::logical_type)
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

fn resolve_copy_annotation_shape(
    annotation: &Type,
    resolve_named: &mut impl FnMut(&str) -> Option<CopyTypeContract>,
) -> Option<CopyTypeContract> {
    match annotation {
        Type::Named(name) if matches!(name.as_str(), "int" | "i32") => Some(CopyTypeContract {
            ty: Ty::Int,
            logical_type: LogicalType::Int,
        }),
        Type::Named(name) if matches!(name.as_str(), "float" | "f64") => Some(CopyTypeContract {
            ty: Ty::Float,
            logical_type: LogicalType::Float,
        }),
        Type::Named(name) if name == "bool" => Some(CopyTypeContract {
            ty: Ty::Bool,
            logical_type: LogicalType::Bool,
        }),
        Type::Named(name) => resolve_named(name),
        Type::Array(element, count) => {
            let element = resolve_copy_annotation_shape(element, resolve_named)?;
            Some(CopyTypeContract {
                ty: Ty::Array(Box::new(element.ty), *count),
                logical_type: LogicalType::Array {
                    element: Box::new(element.logical_type),
                    count: *count,
                },
            })
        }
        Type::Tuple(elements) if elements.len() >= 2 => {
            let elements = elements
                .iter()
                .map(|element| resolve_copy_annotation_shape(element, resolve_named))
                .collect::<Option<Vec<_>>>()?;
            Some(CopyTypeContract {
                ty: Ty::Tuple(elements.iter().map(|element| element.ty.clone()).collect()),
                logical_type: LogicalType::Tuple {
                    elements: elements
                        .into_iter()
                        .map(|element| element.logical_type)
                        .collect(),
                },
            })
        }
        Type::Tuple(_) | Type::Reference(_, _) | Type::Generic(_, _) => None,
    }
}

fn resolve_copy_ty_shape(
    ty: &Ty,
    resolve_named: &mut impl FnMut(&str) -> Option<CopyTypeContract>,
) -> Option<CopyTypeContract> {
    match ty {
        Ty::Int => Some(CopyTypeContract {
            ty: Ty::Int,
            logical_type: LogicalType::Int,
        }),
        Ty::Float => Some(CopyTypeContract {
            ty: Ty::Float,
            logical_type: LogicalType::Float,
        }),
        Ty::Bool => Some(CopyTypeContract {
            ty: Ty::Bool,
            logical_type: LogicalType::Bool,
        }),
        Ty::Struct(name) => resolve_named(name),
        Ty::Array(element, count) => {
            let element = resolve_copy_ty_shape(element, resolve_named)?;
            Some(CopyTypeContract {
                ty: Ty::Array(Box::new(element.ty), *count),
                logical_type: LogicalType::Array {
                    element: Box::new(element.logical_type),
                    count: *count,
                },
            })
        }
        Ty::Tuple(elements) if elements.len() >= 2 => {
            let elements = elements
                .iter()
                .map(|element| resolve_copy_ty_shape(element, resolve_named))
                .collect::<Option<Vec<_>>>()?;
            Some(CopyTypeContract {
                ty: Ty::Tuple(elements.iter().map(|element| element.ty.clone()).collect()),
                logical_type: LogicalType::Tuple {
                    elements: elements
                        .into_iter()
                        .map(|element| element.logical_type)
                        .collect(),
                },
            })
        }
        Ty::Tuple(_)
        | Ty::Void
        | Ty::String
        | Ty::Enum(_)
        | Ty::Reference(_, _)
        | Ty::Option(_)
        | Ty::Result(_, _)
        | Ty::Vec(_)
        | Ty::HashMap(_, _)
        | Ty::TypeParam(_)
        | Ty::Fn(_) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyArrayContract {
    pub(crate) element: CopyTypeContract,
    pub(crate) count: usize,
}

impl CopyArrayContract {
    pub(crate) fn ty(&self) -> Ty {
        Ty::Array(Box::new(self.element.ty.clone()), self.count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CopyArrayIndexDisposition {
    PreserveExistingBehavior,
    Accepted {
        contract: CopyArrayContract,
        constant_index: Option<usize>,
    },
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

#[derive(Debug, Clone)]
struct RawStructDefinition {
    fields: Vec<FieldDecl>,
    type_params: Vec<String>,
}

#[derive(Debug, Clone)]
enum RawStructDefinitionDisposition {
    Unique(RawStructDefinition),
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
    ProcessEntryTupleTransport,
    CopyArrayElementMismatch {
        expected: String,
        actual: String,
    },
    CopyArrayAnnotationMismatch {
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
            Self::ProcessEntryTupleTransport => {
                "process entry `main` cannot use tuple parameters or returns".to_string()
            }
            Self::CopyArrayElementMismatch { expected, actual } => {
                format!("Error: array element type mismatch: expected {expected}, actual {actual}.")
            }
            Self::CopyArrayAnnotationMismatch { expected, actual } => format!(
                "fixed Copy-data array annotation mismatch: expected {expected}, actual {actual}"
            ),
        }
    }
}

impl StructRegistry {
    pub(crate) fn from_top_level_ast(ast: &[AstNode]) -> Self {
        let mut raw_definitions = BTreeMap::new();
        for node in ast {
            let AstNode::Statement(Statement::StructDef {
                name,
                fields,
                type_params,
            }) = node
            else {
                continue;
            };

            let definition = RawStructDefinitionDisposition::Unique(RawStructDefinition {
                fields: fields.clone(),
                type_params: type_params.clone(),
            });
            match raw_definitions.entry(name.clone()) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(definition);
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => {
                    entry.insert(RawStructDefinitionDisposition::Ambiguous);
                }
            }
        }

        let mut definitions = BTreeMap::new();
        let mut visiting = HashSet::new();
        for name in raw_definitions.keys() {
            Self::resolve_definition(name, &raw_definitions, &mut definitions, &mut visiting);
        }
        Self { definitions }
    }

    fn resolve_definition(
        name: &str,
        raw_definitions: &BTreeMap<String, RawStructDefinitionDisposition>,
        resolved: &mut BTreeMap<String, StructDefinitionDisposition>,
        visiting: &mut HashSet<String>,
    ) -> StructDefinitionDisposition {
        if let Some(disposition) = resolved.get(name) {
            return disposition.clone();
        }

        let Some(raw) = raw_definitions.get(name) else {
            return StructDefinitionDisposition::Unsupported;
        };
        let RawStructDefinitionDisposition::Unique(raw) = raw else {
            let disposition = StructDefinitionDisposition::Ambiguous;
            resolved.insert(name.to_string(), disposition.clone());
            return disposition;
        };
        if !admitted_symbol(name)
            || !raw.type_params.is_empty()
            || raw.fields.is_empty()
            || !visiting.insert(name.to_string())
        {
            let disposition = StructDefinitionDisposition::Unsupported;
            resolved.insert(name.to_string(), disposition.clone());
            return disposition;
        }

        let mut seen = HashSet::new();
        let mut contracts = Vec::with_capacity(raw.fields.len());
        let mut supported = true;
        for field in &raw.fields {
            if !admitted_symbol(&field.name) || !seen.insert(field.name.as_str()) {
                supported = false;
                break;
            };
            let Some(copy_type) = Self::resolve_field_copy_type(
                &field.field_type,
                raw_definitions,
                resolved,
                visiting,
            ) else {
                supported = false;
                break;
            };
            contracts.push(StructFieldContract {
                name: field.name.clone(),
                copy_type,
            });
        }
        visiting.remove(name);

        let disposition = if supported {
            StructDefinitionDisposition::Supported(StructContract {
                name: name.to_string(),
                fields: contracts,
            })
        } else {
            StructDefinitionDisposition::Unsupported
        };
        resolved.insert(name.to_string(), disposition.clone());
        disposition
    }

    fn resolve_field_copy_type(
        annotation: &Type,
        raw_definitions: &BTreeMap<String, RawStructDefinitionDisposition>,
        resolved: &mut BTreeMap<String, StructDefinitionDisposition>,
        visiting: &mut HashSet<String>,
    ) -> Option<CopyTypeContract> {
        resolve_copy_annotation_shape(annotation, &mut |name| {
            let StructDefinitionDisposition::Supported(contract) =
                Self::resolve_definition(name, raw_definitions, resolved, visiting)
            else {
                return None;
            };
            Some(CopyTypeContract {
                ty: Ty::Struct(name.to_string()),
                logical_type: contract.logical_type(),
            })
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
            let expected = field.ty();
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
            | Expression::IndexAccess { .. }
            | Expression::FieldAccess { .. }
            | Expression::Deref(_) => Ok(()),
            _ => Err(StructContractError::LocalMoveOrCopy),
        }
    }

    pub(crate) fn is_copy_struct_ty(&self, ty: &Ty) -> bool {
        matches!(ty, Ty::Struct(name) if self.is_copy_struct_name(name))
    }

    pub(crate) fn is_copy_type(&self, ty: &Ty) -> bool {
        self.resolve_copy_type(ty).is_some()
    }

    pub(crate) fn resolve_copy_type(&self, ty: &Ty) -> Option<CopyTypeContract> {
        resolve_copy_ty_shape(ty, &mut |name| {
            self.copy_struct_contract(&Ty::Struct(name.to_string()))
                .map(|contract| CopyTypeContract {
                    ty: Ty::Struct(name.to_string()),
                    logical_type: contract.logical_type(),
                })
        })
    }

    pub(crate) fn copy_array_contract(&self, ty: &Ty) -> Option<CopyArrayContract> {
        let Ty::Array(element, count) = ty else {
            return None;
        };
        self.resolve_copy_type(element)
            .map(|element| CopyArrayContract {
                element,
                count: *count,
            })
    }

    pub(crate) fn copy_array_annotation_contract(
        &self,
        annotation: &Type,
    ) -> Option<CopyArrayContract> {
        let Type::Array(element, count) = annotation else {
            return None;
        };
        self.resolve_copy_annotation(element)
            .map(|element| CopyArrayContract {
                element,
                count: *count,
            })
    }

    pub(crate) fn typed_empty_copy_array_contract(
        &self,
        annotation: &Type,
        initializer: &Expression,
    ) -> Option<CopyArrayContract> {
        let contract = self.copy_array_annotation_contract(annotation)?;
        (contract.count == 0
            && matches!(initializer, Expression::ArrayLiteral(elements) if elements.is_empty()))
        .then_some(contract)
    }

    pub(crate) fn validate_copy_array_binding(
        &self,
        annotation: Option<&Type>,
        inferred: &Ty,
    ) -> Result<(), StructContractError> {
        let inferred_contract = self.copy_array_contract(inferred);
        let annotated_contract =
            annotation.and_then(|annotation| self.copy_array_annotation_contract(annotation));
        match (annotation, inferred_contract, annotated_contract) {
            (None, _, _) => Ok(()),
            (Some(annotation @ Type::Array(_, _)), None, None) => {
                Err(StructContractError::CopyArrayAnnotationMismatch {
                    expected: annotation_name(annotation),
                    actual: inferred.to_string(),
                })
            }
            (Some(_), None, None) => Ok(()),
            (Some(_), Some(_), Some(expected)) if expected.ty() == *inferred => Ok(()),
            (Some(_), _, Some(expected)) => Err(StructContractError::CopyArrayAnnotationMismatch {
                expected: expected.ty().to_string(),
                actual: inferred.to_string(),
            }),
            (Some(annotation), Some(expected), None) => {
                Err(StructContractError::CopyArrayAnnotationMismatch {
                    expected: expected.ty().to_string(),
                    actual: annotation_name(annotation),
                })
            }
        }
    }

    pub(crate) fn validate_copy_array_elements(
        &self,
        expected: &Ty,
        actual_types: impl IntoIterator<Item = Ty>,
    ) -> Result<(), StructContractError> {
        if self.resolve_copy_type(expected).is_none() {
            return Ok(());
        }
        for actual in actual_types {
            if actual != *expected {
                return Err(StructContractError::CopyArrayElementMismatch {
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                });
            }
        }
        Ok(())
    }

    pub(crate) fn classify_copy_array_index(
        &self,
        receiver: &Ty,
        index: &Expression,
    ) -> CopyArrayIndexDisposition {
        let Some(contract) = self.copy_array_contract(receiver) else {
            return CopyArrayIndexDisposition::PreserveExistingBehavior;
        };
        let Some(index) = constant_integer(index) else {
            return CopyArrayIndexDisposition::Accepted {
                contract,
                constant_index: None,
            };
        };
        let Ok(index_usize) = usize::try_from(index) else {
            return CopyArrayIndexDisposition::OutOfBounds {
                index,
                count: contract.count,
            };
        };
        if index_usize >= contract.count {
            return CopyArrayIndexDisposition::OutOfBounds {
                index,
                count: contract.count,
            };
        }
        CopyArrayIndexDisposition::Accepted {
            contract,
            constant_index: Some(index_usize),
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
            let mentions_tuple = parameters
                .iter()
                .any(|parameter| matches!(parameter.param_type, Type::Tuple(_)))
                || return_type.is_some_and(|result| matches!(result, Type::Tuple(_)));
            let mentions_array = parameters
                .iter()
                .any(|parameter| matches!(parameter.param_type, Type::Array(_, _)))
                || return_type.is_some_and(|result| matches!(result, Type::Array(_, _)));
            return Err(if mentions_tuple {
                StructContractError::ProcessEntryTupleTransport
            } else if mentions_array {
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
        matches!(annotation, Type::Array(_, _) | Type::Tuple(_))
            || self.annotation_is_copy_struct(annotation)
    }

    pub(crate) fn resolve_copy_annotation(&self, annotation: &Type) -> Option<CopyTypeContract> {
        resolve_copy_annotation_shape(annotation, &mut |name| {
            self.copy_struct_contract(&Ty::Struct(name.to_string()))
                .map(|contract| CopyTypeContract {
                    ty: Ty::Struct(name.to_string()),
                    logical_type: contract.logical_type(),
                })
        })
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
    fn least_fixed_point_classifier_covers_every_immediate_constructor_pair() {
        let registry = StructRegistry::from_top_level_ast(&[
            definition(
                "Envelope",
                vec![
                    field(
                        "struct_array",
                        Type::Array(Box::new(Type::Named("bool".to_string())), 1),
                    ),
                    field(
                        "struct_tuple",
                        Type::Tuple(vec![
                            Type::Named("int".to_string()),
                            Type::Named("bool".to_string()),
                        ]),
                    ),
                    field("struct_struct", Type::Named("Leaf".to_string())),
                ],
                vec![],
            ),
            definition(
                "Leaf",
                vec![field("value", Type::Named("int".to_string()))],
                vec![],
            ),
        ]);

        let immediate_pairs = [
            Type::Array(
                Box::new(Type::Array(Box::new(Type::Named("bool".to_string())), 1)),
                0,
            ),
            Type::Array(
                Box::new(Type::Tuple(vec![
                    Type::Named("int".to_string()),
                    Type::Named("bool".to_string()),
                ])),
                1,
            ),
            Type::Array(Box::new(Type::Named("Leaf".to_string())), 2),
            Type::Tuple(vec![
                Type::Array(Box::new(Type::Named("bool".to_string())), 1),
                Type::Named("int".to_string()),
            ]),
            Type::Tuple(vec![
                Type::Tuple(vec![
                    Type::Named("int".to_string()),
                    Type::Named("bool".to_string()),
                ]),
                Type::Named("float".to_string()),
            ]),
            Type::Tuple(vec![
                Type::Named("Leaf".to_string()),
                Type::Named("bool".to_string()),
            ]),
            Type::Named("Envelope".to_string()),
        ];

        for annotation in immediate_pairs {
            let contract = registry
                .resolve_copy_annotation(&annotation)
                .unwrap_or_else(|| panic!("recursive annotation was rejected: {annotation:?}"));
            assert_eq!(
                registry
                    .resolve_copy_type(&contract.ty)
                    .expect("resolved Ty must have the same recursive proof"),
                contract
            );
        }

        let envelope = registry
            .resolve_copy_type(&Ty::Struct("Envelope".to_string()))
            .expect("struct fields cover struct-array, struct-tuple, and struct-struct");
        assert_eq!(
            envelope.logical_type,
            LogicalType::Struct {
                name: "Envelope".to_string(),
                fields: vec![
                    LogicalType::Array {
                        element: Box::new(LogicalType::Bool),
                        count: 1,
                    },
                    LogicalType::Tuple {
                        elements: vec![LogicalType::Int, LogicalType::Bool],
                    },
                    LogicalType::Struct {
                        name: "Leaf".to_string(),
                        fields: vec![LogicalType::Int],
                    },
                ],
            }
        );

        for unsupported in [
            Ty::Void,
            Ty::String,
            Ty::Enum("Mode".to_string()),
            Ty::Reference(Box::new(Ty::Int), false),
            Ty::Reference(Box::new(Ty::Int), true),
            Ty::Option(Box::new(Ty::Int)),
            Ty::Result(Box::new(Ty::Int), Box::new(Ty::Bool)),
            Ty::Vec(Box::new(Ty::Int)),
            Ty::HashMap(Box::new(Ty::Int), Box::new(Ty::Bool)),
            Ty::TypeParam("T".to_string()),
            Ty::Fn("callable".to_string()),
            Ty::Tuple(vec![]),
            Ty::Tuple(vec![Ty::Int]),
            Ty::Array(Box::new(Ty::String), 1),
        ] {
            assert!(
                registry.resolve_copy_type(&unsupported).is_none(),
                "excluded Ty family was admitted: {unsupported}"
            );
            assert!(!registry.is_copy_type(&unsupported));
        }

        for unsupported in [
            Type::Named("String".to_string()),
            Type::Named("Unknown".to_string()),
            Type::Tuple(vec![]),
            Type::Tuple(vec![Type::Named("int".to_string())]),
            Type::Reference(Box::new(Type::Named("int".to_string())), false),
            Type::Reference(Box::new(Type::Named("int".to_string())), true),
            Type::Generic("Box".to_string(), vec![Type::Named("int".to_string())]),
            Type::Array(Box::new(Type::Named("String".to_string())), 1),
        ] {
            assert!(
                registry.resolve_copy_annotation(&unsupported).is_none(),
                "excluded Type family was admitted: {unsupported:?}"
            );
        }
    }

    #[test]
    fn acyclic_definition_graph_is_resolved_once_with_forward_dependencies() {
        let registry = StructRegistry::from_top_level_ast(&[
            definition(
                "Outer",
                vec![
                    field("inner", Type::Named("Inner".to_string())),
                    field(
                        "values",
                        Type::Array(Box::new(Type::Named("Inner".to_string())), 2),
                    ),
                ],
                vec![],
            ),
            definition(
                "Inner",
                vec![field("value", Type::Named("int".to_string()))],
                vec![],
            ),
        ]);
        let contract = registry
            .copy_struct_contract(&Ty::Struct("Outer".to_string()))
            .expect("forward acyclic aggregate is Copy");
        assert_eq!(
            contract.logical_type(),
            LogicalType::Struct {
                name: "Outer".to_string(),
                fields: vec![
                    LogicalType::Struct {
                        name: "Inner".to_string(),
                        fields: vec![LogicalType::Int],
                    },
                    LogicalType::Array {
                        element: Box::new(LogicalType::Struct {
                            name: "Inner".to_string(),
                            fields: vec![LogicalType::Int],
                        }),
                        count: 2,
                    },
                ],
            }
        );

        for ast in [
            vec![definition(
                "SelfCycle",
                vec![field("value", Type::Named("SelfCycle".to_string()))],
                vec![],
            )],
            vec![
                definition(
                    "Left",
                    vec![field("right", Type::Named("Right".to_string()))],
                    vec![],
                ),
                definition(
                    "Right",
                    vec![field("left", Type::Named("Left".to_string()))],
                    vec![],
                ),
            ],
            vec![definition(
                "ArrayCycle",
                vec![field(
                    "values",
                    Type::Array(Box::new(Type::Named("ArrayCycle".to_string())), 0),
                )],
                vec![],
            )],
            vec![definition(
                "TupleCycle",
                vec![field(
                    "values",
                    Type::Tuple(vec![
                        Type::Named("int".to_string()),
                        Type::Named("TupleCycle".to_string()),
                    ]),
                )],
                vec![],
            )],
            vec![definition(
                "NestedArrayCycle",
                vec![field(
                    "values",
                    Type::Array(
                        Box::new(Type::Array(
                            Box::new(Type::Named("NestedArrayCycle".to_string())),
                            0,
                        )),
                        0,
                    ),
                )],
                vec![],
            )],
        ] {
            let registry = StructRegistry::from_top_level_ast(&ast);
            let AstNode::Statement(Statement::StructDef { name, .. }) = &ast[0] else {
                unreachable!()
            };
            assert!(
                registry
                    .copy_struct_contract(&Ty::Struct(name.clone()))
                    .is_none(),
                "cyclic definition {name} activated"
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
    fn copy_function_classifier_closes_the_recursive_signature_product() {
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

        let recursive_parameters = vec![
            crate::ast::Parameter {
                name: "tuple".to_string(),
                param_type: Type::Tuple(vec![
                    Type::Named("Packet".to_string()),
                    Type::Array(Box::new(Type::Named("bool".to_string())), 2),
                ]),
            },
            crate::ast::Parameter {
                name: "nested".to_string(),
                param_type: Type::Array(
                    Box::new(Type::Array(
                        Box::new(Type::Tuple(vec![
                            Type::Named("int".to_string()),
                            Type::Named("Packet".to_string()),
                        ])),
                        1,
                    )),
                    0,
                ),
            },
        ];
        let recursive_contract = registry
            .resolve_copy_function_contract(
                "recursive_transport",
                &recursive_parameters,
                Some(&Type::Named("Packet".to_string())),
                &[],
            )
            .expect("recursive signature is classifiable")
            .expect("recursive signature activates aggregate transport");
        assert!(matches!(
            recursive_contract.parameters[0].1.ty,
            Ty::Tuple(_)
        ));
        assert!(matches!(
            recursive_contract.parameters[1].1.ty,
            Ty::Array(_, 0)
        ));

        assert_eq!(
            registry.resolve_copy_function_contract(
                "unsupported_array",
                &[crate::ast::Parameter {
                    name: "values".to_string(),
                    param_type: Type::Array(Box::new(Type::Named("String".to_string())), 1,),
                }],
                Some(&Type::Named("int".to_string())),
                &[],
            ),
            Err(StructContractError::UnsupportedFunctionArrayParameter {
                parameter_name: "values".to_string(),
            })
        );
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
    fn fixed_copy_array_classifier_delegates_every_recursive_element_to_copy_data() {
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
            .copy_array_contract(&array)
            .expect("supported recursive Copy-data array");
        assert_eq!(contract.ty(), array);
        assert_eq!(
            registry
                .resolve_copy_type(&array)
                .expect("supported recursive Copy-data type")
                .logical_type,
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
                .copy_array_annotation_contract(&annotation)
                .expect("exact annotation")
                .ty(),
            array
        );
        assert!(
            registry
                .validate_copy_array_binding(Some(&annotation), &array)
                .is_ok()
        );
        assert!(matches!(
            registry.validate_copy_array_binding(
                Some(&Type::Array(Box::new(Type::Named("Packet".to_string())), 1,)),
                &array,
            ),
            Err(StructContractError::CopyArrayAnnotationMismatch { .. })
        ));
        assert!(
            registry
                .validate_copy_array_elements(&packet, [packet.clone(), packet.clone()],)
                .is_ok()
        );
        assert!(matches!(
            registry.validate_copy_array_elements(&packet, [Ty::Struct("Other".to_string())],),
            Err(StructContractError::CopyArrayElementMismatch { .. })
        ));

        let empty_annotation = Type::Array(Box::new(Type::Named("Packet".to_string())), 0);
        assert!(
            registry
                .typed_empty_copy_array_contract(
                    &empty_annotation,
                    &Expression::ArrayLiteral(Vec::new()),
                )
                .is_some()
        );
        assert!(
            registry
                .typed_empty_copy_array_contract(
                    &empty_annotation,
                    &Expression::ArrayRepeat {
                        value: Box::new(literal("Packet", &["count", "ready"])),
                        count: 0,
                    },
                )
                .is_none()
        );

        assert!(matches!(
            registry.classify_copy_array_index(&array, &Expression::IntegerLiteral(1),),
            CopyArrayIndexDisposition::Accepted {
                constant_index: Some(1),
                ..
            }
        ));
        assert!(matches!(
            registry
                .classify_copy_array_index(&array, &Expression::Identifier("index".to_string()),),
            CopyArrayIndexDisposition::Accepted {
                constant_index: None,
                ..
            }
        ));
        assert_eq!(
            registry.classify_copy_array_index(
                &array,
                &Expression::Unary {
                    op: crate::ast::UnaryOp::Negate,
                    operand: Box::new(Expression::IntegerLiteral(1)),
                },
            ),
            CopyArrayIndexDisposition::OutOfBounds {
                index: -1,
                count: 2
            }
        );
        assert_eq!(
            registry.classify_copy_array_index(
                &Ty::Array(Box::new(Ty::String), 2),
                &Expression::IntegerLiteral(0),
            ),
            CopyArrayIndexDisposition::PreserveExistingBehavior
        );
    }
}
