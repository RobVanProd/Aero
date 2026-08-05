use crate::ast::{Expression, Type};
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingAnnotationDisposition {
    ExistingExplicitRejection(BindingAnnotationRejectKind),
    MatchesExistingContractShape(BindingContractKind),
    PreserveExistingBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingAnnotationRejectKind {
    Tuple,
    ArrayTuple,
    DoubleArrayTuple,
    ReferenceTuple,
    ReferenceArrayTuple,
}

impl BindingAnnotationRejectKind {
    pub(crate) fn topology(self) -> &'static str {
        match self {
            Self::Tuple => "tuple type annotation",
            Self::ArrayTuple => "tuple type annotation directly beneath an array",
            Self::DoubleArrayTuple => "tuple type annotation directly beneath two array layers",
            Self::ReferenceTuple => "tuple type annotation directly beneath a reference",
            Self::ReferenceArrayTuple => {
                "tuple type annotation directly beneath an array directly beneath a reference"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NumericBindingKind {
    Int,
    Float,
}

impl NumericBindingKind {
    fn ty(self) -> Ty {
        match self {
            Self::Int => Ty::Int,
            Self::Float => Ty::Float,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingContractKind {
    NumericScalar(NumericBindingKind),
    Bool,
    String,
    FixedNumericArray {
        element: NumericBindingKind,
        count: usize,
    },
}

impl BindingContractKind {
    pub(crate) fn ty(self) -> Ty {
        match self {
            Self::NumericScalar(kind) => kind.ty(),
            Self::Bool => Ty::Bool,
            Self::String => Ty::String,
            Self::FixedNumericArray { element, count } => Ty::Array(Box::new(element.ty()), count),
        }
    }

    pub(crate) fn is_numeric_scalar(self) -> bool {
        matches!(self, Self::NumericScalar(_))
    }
}

fn numeric_kind(annotation: &Type) -> Option<NumericBindingKind> {
    match annotation {
        Type::Named(name) if matches!(name.as_str(), "i32" | "int") => {
            Some(NumericBindingKind::Int)
        }
        Type::Named(name) if matches!(name.as_str(), "f64" | "float") => {
            Some(NumericBindingKind::Float)
        }
        _ => None,
    }
}

pub(crate) fn is_legacy_numeric_array_annotation(annotation: &Type) -> bool {
    matches!(annotation, Type::Array(element, _) if numeric_kind(element).is_some())
}

fn explicit_rejection(annotation: &Type, initialized: bool) -> Option<BindingAnnotationRejectKind> {
    match annotation {
        Type::Tuple(_) if !initialized => Some(BindingAnnotationRejectKind::Tuple),
        Type::Array(inner, _) if !initialized && matches!(inner.as_ref(), Type::Tuple(_)) => {
            Some(BindingAnnotationRejectKind::ArrayTuple)
        }
        Type::Array(first, _)
            if !initialized
                && matches!(
                    first.as_ref(),
                    Type::Array(second, _) if matches!(second.as_ref(), Type::Tuple(_))
                ) =>
        {
            Some(BindingAnnotationRejectKind::DoubleArrayTuple)
        }
        Type::Reference(inner, _) if matches!(inner.as_ref(), Type::Tuple(_)) => {
            Some(BindingAnnotationRejectKind::ReferenceTuple)
        }
        Type::Reference(inner, _)
            if matches!(
                inner.as_ref(),
                Type::Array(element, count)
                    if matches!(element.as_ref(), Type::Tuple(_))
                        && (!initialized || *count > 0)
            ) =>
        {
            Some(BindingAnnotationRejectKind::ReferenceArrayTuple)
        }
        _ => None,
    }
}

pub(crate) fn classify_binding_annotation(
    annotation: &Type,
    initialized: bool,
) -> BindingAnnotationDisposition {
    if let Some(kind) = explicit_rejection(annotation, initialized) {
        return BindingAnnotationDisposition::ExistingExplicitRejection(kind);
    }

    let contract = match annotation {
        Type::Named(name) if matches!(name.as_str(), "i32" | "int") => {
            Some(BindingContractKind::NumericScalar(NumericBindingKind::Int))
        }
        Type::Named(name) if matches!(name.as_str(), "f64" | "float") => Some(
            BindingContractKind::NumericScalar(NumericBindingKind::Float),
        ),
        Type::Named(name) if name == "bool" => Some(BindingContractKind::Bool),
        Type::Named(name) if name == "String" => Some(BindingContractKind::String),
        Type::Array(element, count) if *count > 0 => {
            numeric_kind(element).map(|element| BindingContractKind::FixedNumericArray {
                element,
                count: *count,
            })
        }
        _ => None,
    };

    contract.map_or(
        BindingAnnotationDisposition::PreserveExistingBehavior,
        BindingAnnotationDisposition::MatchesExistingContractShape,
    )
}

pub(crate) fn typed_empty_numeric_array_contract(
    annotation: &Type,
    initializer: &Expression,
) -> Option<BindingContractKind> {
    match (annotation, initializer) {
        (Type::Array(element, 0), Expression::ArrayLiteral(elements)) if elements.is_empty() => {
            numeric_kind(element)
                .map(|element| BindingContractKind::FixedNumericArray { element, count: 0 })
        }
        _ => None,
    }
}

pub(crate) fn is_statically_empty_fixed_array(ty: &Ty) -> bool {
    matches!(ty, Ty::Array(_, 0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str) -> Type {
        Type::Named(name.to_string())
    }

    fn tuple(arity: usize) -> Type {
        Type::Tuple((0..arity).map(|_| named("int")).collect())
    }

    #[test]
    fn exact_nonrecursive_disposition_table_is_finite() {
        for initialized in [false, true] {
            assert_eq!(
                classify_binding_annotation(&tuple(0), initialized),
                if initialized {
                    BindingAnnotationDisposition::PreserveExistingBehavior
                } else {
                    BindingAnnotationDisposition::ExistingExplicitRejection(
                        BindingAnnotationRejectKind::Tuple,
                    )
                }
            );
            for count in [0, 1] {
                assert_eq!(
                    classify_binding_annotation(
                        &Type::Array(Box::new(tuple(2)), count),
                        initialized,
                    ),
                    if initialized {
                        BindingAnnotationDisposition::PreserveExistingBehavior
                    } else {
                        BindingAnnotationDisposition::ExistingExplicitRejection(
                            BindingAnnotationRejectKind::ArrayTuple,
                        )
                    }
                );
                assert_eq!(
                    classify_binding_annotation(
                        &Type::Array(Box::new(Type::Array(Box::new(tuple(1)), count)), count,),
                        initialized,
                    ),
                    if initialized {
                        BindingAnnotationDisposition::PreserveExistingBehavior
                    } else {
                        BindingAnnotationDisposition::ExistingExplicitRejection(
                            BindingAnnotationRejectKind::DoubleArrayTuple,
                        )
                    }
                );
                for mutable in [false, true] {
                    assert_eq!(
                        classify_binding_annotation(
                            &Type::Reference(Box::new(tuple(1)), mutable),
                            initialized,
                        ),
                        BindingAnnotationDisposition::ExistingExplicitRejection(
                            BindingAnnotationRejectKind::ReferenceTuple,
                        )
                    );
                    let reference_array =
                        Type::Reference(Box::new(Type::Array(Box::new(tuple(3)), count)), mutable);
                    let expected = if initialized && count == 0 {
                        BindingAnnotationDisposition::PreserveExistingBehavior
                    } else {
                        BindingAnnotationDisposition::ExistingExplicitRejection(
                            BindingAnnotationRejectKind::ReferenceArrayTuple,
                        )
                    };
                    assert_eq!(
                        classify_binding_annotation(&reference_array, initialized),
                        expected
                    );
                }
            }
        }

        let triple_array = Type::Array(
            Box::new(Type::Array(Box::new(Type::Array(Box::new(tuple(2)), 1)), 1)),
            1,
        );
        assert_eq!(
            classify_binding_annotation(&triple_array, false),
            BindingAnnotationDisposition::PreserveExistingBehavior
        );
    }

    #[test]
    fn contracts_and_typed_empty_capability_do_not_absorb_nearby_shapes() {
        for (name, expected) in [
            ("int", NumericBindingKind::Int),
            ("i32", NumericBindingKind::Int),
            ("float", NumericBindingKind::Float),
            ("f64", NumericBindingKind::Float),
        ] {
            assert_eq!(
                classify_binding_annotation(&named(name), true),
                BindingAnnotationDisposition::MatchesExistingContractShape(
                    BindingContractKind::NumericScalar(expected)
                )
            );
            assert_eq!(
                classify_binding_annotation(&Type::Array(Box::new(named(name)), 2), true,),
                BindingAnnotationDisposition::MatchesExistingContractShape(
                    BindingContractKind::FixedNumericArray {
                        element: expected,
                        count: 2,
                    }
                )
            );
            let zero = Type::Array(Box::new(named(name)), 0);
            assert_eq!(
                classify_binding_annotation(&zero, true),
                BindingAnnotationDisposition::PreserveExistingBehavior
            );
            assert_eq!(
                typed_empty_numeric_array_contract(&zero, &Expression::ArrayLiteral(Vec::new())),
                Some(BindingContractKind::FixedNumericArray {
                    element: expected,
                    count: 0,
                })
            );
            assert_eq!(
                typed_empty_numeric_array_contract(
                    &zero,
                    &Expression::ArrayRepeat {
                        value: Box::new(Expression::IntegerLiteral(1)),
                        count: 0,
                    },
                ),
                None
            );
        }

        for annotation in [
            Type::Array(Box::new(named("bool")), 0),
            Type::Array(Box::new(named("int")), 1),
            Type::Array(Box::new(Type::Array(Box::new(named("int")), 0)), 0),
        ] {
            assert_eq!(
                typed_empty_numeric_array_contract(
                    &annotation,
                    &Expression::ArrayLiteral(Vec::new())
                ),
                None
            );
        }
    }

    #[test]
    fn statically_empty_fixed_array_class_is_exact_and_element_agnostic() {
        for element in [
            Ty::Int,
            Ty::Float,
            Ty::Bool,
            Ty::String,
            Ty::Array(Box::new(Ty::Int), 1),
        ] {
            assert!(is_statically_empty_fixed_array(&Ty::Array(
                Box::new(element),
                0
            )));
        }

        for ty in [
            Ty::Int,
            Ty::Vec(Box::new(Ty::Int)),
            Ty::Array(Box::new(Ty::Int), 1),
        ] {
            assert!(!is_statically_empty_fixed_array(&ty));
        }
    }
}
