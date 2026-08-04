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
    #[cfg(test)]
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
        if matches!(inferred, Ty::Struct(_))
            && !matches!(initializer, Expression::StructLiteral { .. })
        {
            Err(StructContractError::LocalMoveOrCopy)
        } else {
            Ok(())
        }
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
}
