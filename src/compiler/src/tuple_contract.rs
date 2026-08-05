use crate::ast::Type;
use crate::ir::LogicalType;
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TupleExecutionContext {
    AdmittedFunction,
    PreservedContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FlatCopyTupleContract {
    pub(crate) elements: Vec<Ty>,
}

impl FlatCopyTupleContract {
    pub(crate) fn ty(&self) -> Ty {
        Ty::Tuple(self.elements.clone())
    }

    pub(crate) fn logical_type(&self) -> LogicalType {
        LogicalType::Tuple {
            elements: self
                .elements
                .iter()
                .map(|element| match element {
                    Ty::Int => LogicalType::Int,
                    Ty::Float => LogicalType::Float,
                    Ty::Bool => LogicalType::Bool,
                    _ => unreachable!("flat Copy tuple contract contains only scalar elements"),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TupleProjectionContract {
    pub(crate) tuple: FlatCopyTupleContract,
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

pub(crate) fn classify_flat_copy_tuple_elements(
    elements: &[Ty],
    context: TupleExecutionContext,
) -> TupleContractDisposition<FlatCopyTupleContract> {
    if context != TupleExecutionContext::AdmittedFunction {
        return TupleContractDisposition::Preserved;
    }
    if elements.len() < 2 {
        return TupleContractDisposition::ExplicitlyRejected(
            "flat Copy tuples require at least two elements".to_string(),
        );
    }
    if let Some((index, element)) = elements
        .iter()
        .enumerate()
        .find(|(_, element)| !matches!(element, Ty::Int | Ty::Float | Ty::Bool))
    {
        return TupleContractDisposition::ExplicitlyRejected(format!(
            "flat Copy tuple element {} has unsupported type {}; expected Int, Float, or Bool",
            index + 1,
            element
        ));
    }
    TupleContractDisposition::Supported(FlatCopyTupleContract {
        elements: elements.to_vec(),
    })
}

pub(crate) fn classify_flat_copy_tuple_annotation(
    annotation: &Type,
) -> TupleContractDisposition<FlatCopyTupleContract> {
    let Type::Tuple(elements) = annotation else {
        return TupleContractDisposition::Preserved;
    };
    let mut resolved = Vec::with_capacity(elements.len());
    for element in elements {
        let ty = match element {
            Type::Named(name) if matches!(name.as_str(), "int" | "i32") => Ty::Int,
            Type::Named(name) if matches!(name.as_str(), "float" | "f64") => Ty::Float,
            Type::Named(name) if name == "bool" => Ty::Bool,
            _ => {
                return TupleContractDisposition::ExplicitlyRejected(format!(
                    "flat Copy tuple annotation element {} is unsupported; expected Int, Float, or Bool",
                    resolved.len() + 1
                ));
            }
        };
        resolved.push(ty);
    }
    classify_flat_copy_tuple_elements(&resolved, TupleExecutionContext::AdmittedFunction)
}

pub(crate) fn classify_tuple_projection(
    receiver: &Ty,
    index: usize,
    context: TupleExecutionContext,
) -> TupleContractDisposition<TupleProjectionContract> {
    if context != TupleExecutionContext::AdmittedFunction {
        return TupleContractDisposition::Preserved;
    }
    let Ty::Tuple(elements) = receiver else {
        return TupleContractDisposition::ExplicitlyRejected(
            "tuple projection requires a flat Copy tuple".to_string(),
        );
    };
    let tuple = match classify_flat_copy_tuple_elements(elements, context) {
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
    mutable: bool,
) -> Result<(), TupleBindingValidationError> {
    let inferred_contract = match inferred {
        Ty::Tuple(elements) => match classify_flat_copy_tuple_elements(
            elements,
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
    let annotation_contract = annotation.map(classify_flat_copy_tuple_annotation);

    if inferred_contract.is_some() && mutable {
        return Err(TupleBindingValidationError::Explicit(
            "mutable flat Copy tuple bindings are not admitted".to_string(),
        ));
    }

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

    #[test]
    fn flat_tuple_classifier_owns_the_complete_scalar_product() {
        fn assert_products(prefix: &mut Vec<Ty>, remaining: usize) {
            if remaining == 0 {
                assert!(matches!(
                    classify_flat_copy_tuple_elements(
                        prefix,
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
                let annotation = Type::Tuple(vec![
                    Type::Named(int_name.to_string()),
                    Type::Named(float_name.to_string()),
                    Type::Named("bool".to_string()),
                ]);
                assert!(matches!(
                    classify_flat_copy_tuple_annotation(&annotation),
                    TupleContractDisposition::Supported(_)
                ));
            }
        }
        for elements in [
            Vec::new(),
            vec![Ty::Int],
            vec![Ty::Int, Ty::String],
            vec![Ty::Int, Ty::Tuple(vec![Ty::Int, Ty::Int])],
        ] {
            assert!(matches!(
                classify_flat_copy_tuple_elements(
                    &elements,
                    TupleExecutionContext::AdmittedFunction
                ),
                TupleContractDisposition::ExplicitlyRejected(_)
            ));
        }
        assert!(matches!(
            classify_flat_copy_tuple_elements(
                &[Ty::Int, Ty::Bool],
                TupleExecutionContext::PreservedContext
            ),
            TupleContractDisposition::Preserved
        ));
    }
}
