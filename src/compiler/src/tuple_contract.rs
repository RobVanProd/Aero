use crate::ast::Type;
use crate::ir::LogicalType;
use crate::struct_contract::{CopyTypeContract, StructRegistry};
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TupleExecutionContext {
    AdmittedFunction,
    PreservedContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyTupleContract {
    pub(crate) elements: Vec<Ty>,
    logical_elements: Vec<LogicalType>,
}

impl CopyTupleContract {
    pub(crate) fn ty(&self) -> Ty {
        Ty::Tuple(self.elements.clone())
    }

    pub(crate) fn logical_type(&self) -> LogicalType {
        LogicalType::Tuple {
            elements: self.logical_elements.clone(),
        }
    }
}

fn tuple_contract(contract: CopyTypeContract) -> Option<CopyTupleContract> {
    let Ty::Tuple(elements) = contract.ty else {
        return None;
    };
    let LogicalType::Tuple {
        elements: logical_elements,
    } = contract.logical_type
    else {
        return None;
    };
    Some(CopyTupleContract {
        elements,
        logical_elements,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TupleProjectionContract {
    pub(crate) tuple: CopyTupleContract,
    pub(crate) index: usize,
    pub(crate) element: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TupleContractDisposition<T> {
    Supported(T),
    ExplicitlyRejected(String),
    Preserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TupleBindingValidationError {
    Explicit(String),
    PreserveInitializedDirectAnnotationRejection,
}

pub(crate) fn classify_copy_tuple_elements(
    elements: &[Ty],
    registry: &StructRegistry,
    context: TupleExecutionContext,
) -> TupleContractDisposition<CopyTupleContract> {
    if context != TupleExecutionContext::AdmittedFunction {
        return TupleContractDisposition::Preserved;
    }
    registry
        .resolve_copy_type(&Ty::Tuple(elements.to_vec()))
        .and_then(tuple_contract)
        .map_or_else(
            || {
                TupleContractDisposition::ExplicitlyRejected(
                    "Copy tuples require at least two recursively admitted CopyData elements"
                        .to_string(),
                )
            },
            TupleContractDisposition::Supported,
        )
}

pub(crate) fn classify_copy_tuple_annotation(
    annotation: &Type,
    registry: &StructRegistry,
) -> TupleContractDisposition<CopyTupleContract> {
    let Type::Tuple(_) = annotation else {
        return TupleContractDisposition::Preserved;
    };
    registry
        .resolve_copy_annotation(annotation)
        .and_then(tuple_contract)
        .map_or_else(
            || {
                TupleContractDisposition::ExplicitlyRejected(
                    "Copy tuple annotations require at least two recursively admitted CopyData elements"
                        .to_string(),
                )
            },
            TupleContractDisposition::Supported,
        )
}

pub(crate) fn classify_tuple_projection(
    receiver: &Ty,
    index: usize,
    registry: &StructRegistry,
    context: TupleExecutionContext,
) -> TupleContractDisposition<TupleProjectionContract> {
    if context != TupleExecutionContext::AdmittedFunction {
        return TupleContractDisposition::Preserved;
    }
    let Ty::Tuple(elements) = receiver else {
        return TupleContractDisposition::ExplicitlyRejected(
            "tuple projection requires a recursively admitted Copy tuple".to_string(),
        );
    };
    let tuple = match classify_copy_tuple_elements(elements, registry, context) {
        TupleContractDisposition::Supported(contract) => contract,
        TupleContractDisposition::ExplicitlyRejected(message) => {
            return TupleContractDisposition::ExplicitlyRejected(message);
        }
        TupleContractDisposition::Preserved => unreachable!("context was admitted"),
    };
    let Some(element) = tuple.elements.get(index).cloned() else {
        return TupleContractDisposition::ExplicitlyRejected(format!(
            "tuple projection index {index} is outside 0..{}",
            tuple.elements.len()
        ));
    };
    TupleContractDisposition::Supported(TupleProjectionContract {
        tuple,
        index,
        element,
    })
}

pub(crate) fn validate_tuple_binding(
    annotation: Option<&Type>,
    inferred: &Ty,
    _mutable: bool,
    registry: &StructRegistry,
) -> Result<(), TupleBindingValidationError> {
    let inferred_contract = match inferred {
        Ty::Tuple(elements) => match classify_copy_tuple_elements(
            elements,
            registry,
            TupleExecutionContext::AdmittedFunction,
        ) {
            TupleContractDisposition::Supported(contract) => Some(contract),
            TupleContractDisposition::ExplicitlyRejected(message) => {
                return Err(TupleBindingValidationError::Explicit(message));
            }
            TupleContractDisposition::Preserved => unreachable!("context was admitted"),
        },
        _ => None,
    };
    let annotation_contract =
        annotation.map(|annotation| classify_copy_tuple_annotation(annotation, registry));

    match (annotation, annotation_contract, inferred_contract) {
        (None, _, _) => Ok(()),
        (
            Some(Type::Tuple(_)),
            Some(TupleContractDisposition::Supported(expected)),
            Some(actual),
        ) if expected == actual => Ok(()),
        (
            Some(Type::Tuple(_)),
            Some(TupleContractDisposition::Supported(expected)),
            Some(actual),
        ) => Err(TupleBindingValidationError::Explicit(format!(
            "tuple binding annotation mismatch: expected {}, actual {}",
            expected.ty(),
            actual.ty()
        ))),
        (Some(Type::Tuple(_)), Some(TupleContractDisposition::Supported(expected)), None) => {
            let _ = expected;
            Err(TupleBindingValidationError::PreserveInitializedDirectAnnotationRejection)
        }
        (Some(Type::Tuple(_)), Some(TupleContractDisposition::ExplicitlyRejected(message)), _) => {
            Err(TupleBindingValidationError::Explicit(message))
        }
        (Some(Type::Tuple(_)), Some(TupleContractDisposition::Preserved), _) => {
            unreachable!("tuple annotation classifier handles tuple annotations")
        }
        (Some(_), _, Some(actual)) => Err(TupleBindingValidationError::Explicit(format!(
            "tuple binding annotation mismatch: expected {}, actual {}",
            annotation_name(annotation.expect("present")),
            actual.ty()
        ))),
        (Some(_), _, None) => Ok(()),
    }
}

fn annotation_name(annotation: &Type) -> String {
    match annotation {
        Type::Named(name) => name.clone(),
        Type::Array(_, count) => format!("array[{count}]"),
        Type::Tuple(elements) => format!("tuple[{}]", elements.len()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, FieldDecl, Statement};

    fn registry() -> StructRegistry {
        StructRegistry::from_top_level_ast(&[AstNode::Statement(Statement::StructDef {
            name: "Leaf".to_string(),
            fields: vec![FieldDecl {
                name: "value".to_string(),
                field_type: Type::Named("int".to_string()),
            }],
            type_params: vec![],
        })])
    }

    #[test]
    fn recursive_tuple_classifier_delegates_the_complete_product_to_the_registry() {
        fn assert_products(prefix: &mut Vec<Ty>, remaining: usize) {
            if remaining == 0 {
                let registry = registry();
                assert!(matches!(
                    classify_copy_tuple_elements(
                        prefix,
                        &registry,
                        TupleExecutionContext::AdmittedFunction
                    ),
                    TupleContractDisposition::Supported(_)
                ));
                return;
            }
            for element in [Ty::Int, Ty::Float, Ty::Bool] {
                prefix.push(element);
                assert_products(prefix, remaining - 1);
                prefix.pop();
            }
        }

        for arity in 2..=6 {
            assert_products(&mut Vec::with_capacity(arity), arity);
        }
        for int_name in ["int", "i32"] {
            for float_name in ["float", "f64"] {
                let registry = registry();
                let annotation = Type::Tuple(vec![
                    Type::Named(int_name.to_string()),
                    Type::Named(float_name.to_string()),
                    Type::Named("bool".to_string()),
                ]);
                assert!(matches!(
                    classify_copy_tuple_annotation(&annotation, &registry),
                    TupleContractDisposition::Supported(_)
                ));
            }
        }

        let registry = registry();
        let recursive = vec![
            Ty::Array(Box::new(Ty::Bool), 0),
            Ty::Tuple(vec![Ty::Int, Ty::Array(Box::new(Ty::Float), 2)]),
            Ty::Struct("Leaf".to_string()),
        ];
        let TupleContractDisposition::Supported(contract) = classify_copy_tuple_elements(
            &recursive,
            &registry,
            TupleExecutionContext::AdmittedFunction,
        ) else {
            panic!("recursive tuple product was not admitted");
        };
        assert_eq!(contract.ty(), Ty::Tuple(recursive));
        assert_eq!(
            contract.logical_type(),
            LogicalType::Tuple {
                elements: vec![
                    LogicalType::Array {
                        element: Box::new(LogicalType::Bool),
                        count: 0,
                    },
                    LogicalType::Tuple {
                        elements: vec![
                            LogicalType::Int,
                            LogicalType::Array {
                                element: Box::new(LogicalType::Float),
                                count: 2,
                            },
                        ],
                    },
                    LogicalType::Struct {
                        name: "Leaf".to_string(),
                        fields: vec![LogicalType::Int],
                    },
                ],
            }
        );

        for elements in [
            Vec::new(),
            vec![Ty::Int],
            vec![Ty::Int, Ty::String],
            vec![Ty::Int, Ty::Reference(Box::new(Ty::Int), false)],
            vec![Ty::Int, Ty::Enum("Mode".to_string())],
            vec![Ty::Int, Ty::TypeParam("T".to_string())],
        ] {
            assert!(matches!(
                classify_copy_tuple_elements(
                    &elements,
                    &registry,
                    TupleExecutionContext::AdmittedFunction
                ),
                TupleContractDisposition::ExplicitlyRejected(_)
            ));
        }
        assert!(matches!(
            classify_copy_tuple_elements(
                &[Ty::Int, Ty::Bool],
                &registry,
                TupleExecutionContext::PreservedContext
            ),
            TupleContractDisposition::Preserved
        ));

        let tuple_ty = Ty::Tuple(vec![
            Ty::Array(Box::new(Ty::Bool), 1),
            Ty::Struct("Leaf".to_string()),
        ]);
        let TupleContractDisposition::Supported(projection) = classify_tuple_projection(
            &tuple_ty,
            1,
            &registry,
            TupleExecutionContext::AdmittedFunction,
        ) else {
            panic!("recursive tuple projection was not admitted");
        };
        assert_eq!(projection.element, Ty::Struct("Leaf".to_string()));
        assert!(matches!(
            classify_tuple_projection(
                &tuple_ty,
                2,
                &registry,
                TupleExecutionContext::AdmittedFunction,
            ),
            TupleContractDisposition::ExplicitlyRejected(_)
        ));
    }
}
