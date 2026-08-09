use crate::ast::{Expression, Type};
use crate::primitive_contract::PrimitiveKind;
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingAnnotationDisposition {
    SupportedBindingAnnotation(BindingContractKind),
    ExplicitlyRejectedAnnotationTopology(RejectedAnnotationTopology),
    PreservedQuarantinedTopology,
}

impl BindingAnnotationDisposition {
    pub(crate) fn supported_contract(self) -> Option<BindingContractKind> {
        match self {
            Self::SupportedBindingAnnotation(contract) => Some(contract),
            Self::ExplicitlyRejectedAnnotationTopology(_) | Self::PreservedQuarantinedTopology => {
                None
            }
        }
    }

    pub(crate) fn rejected_topology(self) -> Option<RejectedAnnotationTopology> {
        match self {
            Self::ExplicitlyRejectedAnnotationTopology(topology) => Some(topology),
            Self::SupportedBindingAnnotation(_) | Self::PreservedQuarantinedTopology => None,
        }
    }

    pub(crate) fn defers_to_tuple_contract(self) -> bool {
        self.rejected_topology().is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RejectedAnnotationTopology {
    Tuple,
    ArrayTuple,
    DoubleArrayTuple,
    ReferenceTuple,
    ReferenceArrayTuple,
}

impl RejectedAnnotationTopology {
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
    Char,
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
            Self::Char => Ty::Char,
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
        Type::Named(name) => match PrimitiveKind::from_source_name(name) {
            Some(PrimitiveKind::Int) => Some(NumericBindingKind::Int),
            Some(PrimitiveKind::Float) => Some(NumericBindingKind::Float),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn is_legacy_numeric_array_annotation(annotation: &Type) -> bool {
    matches!(annotation, Type::Array(element, _) if numeric_kind(element).is_some())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnnotationWrapper {
    Array { count: usize },
    Reference { mutable: bool },
}

#[derive(Debug, Clone)]
struct AnnotationTopology<'a> {
    wrappers: Vec<AnnotationWrapper>,
    leaf: &'a Type,
}

impl<'a> AnnotationTopology<'a> {
    fn decompose(annotation: &'a Type) -> Self {
        let mut wrappers = Vec::new();
        let mut leaf = annotation;
        loop {
            match leaf {
                Type::Array(inner, count) => {
                    wrappers.push(AnnotationWrapper::Array { count: *count });
                    leaf = inner;
                }
                Type::Reference(inner, mutable) => {
                    wrappers.push(AnnotationWrapper::Reference { mutable: *mutable });
                    leaf = inner;
                }
                Type::Named(_) | Type::Tuple(_) | Type::Generic(_, _) => break,
            }
        }
        Self { wrappers, leaf }
    }
}

fn explicit_rejection(annotation: &Type, initialized: bool) -> Option<RejectedAnnotationTopology> {
    let topology = AnnotationTopology::decompose(annotation);
    if !matches!(topology.leaf, Type::Tuple(_)) {
        return None;
    }

    match topology.wrappers.as_slice() {
        [] if !initialized => Some(RejectedAnnotationTopology::Tuple),
        [AnnotationWrapper::Array { .. }] if !initialized => {
            Some(RejectedAnnotationTopology::ArrayTuple)
        }
        [
            AnnotationWrapper::Array { .. },
            AnnotationWrapper::Array { .. },
        ] if !initialized => Some(RejectedAnnotationTopology::DoubleArrayTuple),
        [AnnotationWrapper::Reference { mutable: _ }] => {
            Some(RejectedAnnotationTopology::ReferenceTuple)
        }
        [
            AnnotationWrapper::Reference { mutable: _ },
            AnnotationWrapper::Array { count },
        ] if !initialized || *count > 0 => Some(RejectedAnnotationTopology::ReferenceArrayTuple),
        _ => None,
    }
}

pub(crate) fn classify_binding_annotation(
    annotation: &Type,
    initialized: bool,
) -> BindingAnnotationDisposition {
    if let Some(kind) = explicit_rejection(annotation, initialized) {
        return BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology(kind);
    }

    let contract = match annotation {
        Type::Named(name) => match PrimitiveKind::from_source_name(name) {
            Some(PrimitiveKind::Int) => {
                Some(BindingContractKind::NumericScalar(NumericBindingKind::Int))
            }
            Some(PrimitiveKind::Float) => Some(BindingContractKind::NumericScalar(
                NumericBindingKind::Float,
            )),
            Some(PrimitiveKind::Bool) => Some(BindingContractKind::Bool),
            Some(PrimitiveKind::Char) => Some(BindingContractKind::Char),
            None if name == "String" => Some(BindingContractKind::String),
            None => None,
        },
        Type::Array(element, count) if *count > 0 => {
            numeric_kind(element).map(|element| BindingContractKind::FixedNumericArray {
                element,
                count: *count,
            })
        }
        _ => None,
    };

    contract.map_or(
        BindingAnnotationDisposition::PreservedQuarantinedTopology,
        BindingAnnotationDisposition::SupportedBindingAnnotation,
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

    #[derive(Debug, Clone, Copy)]
    enum WrapperSpec {
        Array(usize),
        Reference(bool),
        Generic,
    }

    fn wrap_tuple(arity: usize, wrappers: &[WrapperSpec]) -> Type {
        wrappers
            .iter()
            .rev()
            .fold(tuple(arity), |inner, wrapper| match wrapper {
                WrapperSpec::Array(count) => Type::Array(Box::new(inner), *count),
                WrapperSpec::Reference(mutable) => Type::Reference(Box::new(inner), *mutable),
                WrapperSpec::Generic => Type::Generic("Vec".to_string(), vec![inner]),
            })
    }

    fn expected_tuple_path_disposition(
        wrappers: &[WrapperSpec],
        initialized: bool,
    ) -> BindingAnnotationDisposition {
        let rejection = match wrappers {
            [] if !initialized => Some(RejectedAnnotationTopology::Tuple),
            [WrapperSpec::Array(_)] if !initialized => Some(RejectedAnnotationTopology::ArrayTuple),
            [WrapperSpec::Array(_), WrapperSpec::Array(_)] if !initialized => {
                Some(RejectedAnnotationTopology::DoubleArrayTuple)
            }
            [WrapperSpec::Reference(_)] => Some(RejectedAnnotationTopology::ReferenceTuple),
            [WrapperSpec::Reference(_), WrapperSpec::Array(count)]
                if !initialized || *count > 0 =>
            {
                Some(RejectedAnnotationTopology::ReferenceArrayTuple)
            }
            _ => None,
        };
        rejection.map_or(
            BindingAnnotationDisposition::PreservedQuarantinedTopology,
            BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology,
        )
    }

    fn wrapper_paths(max_depth: usize) -> Vec<Vec<WrapperSpec>> {
        fn extend(
            paths: &mut Vec<Vec<WrapperSpec>>,
            current: &mut Vec<WrapperSpec>,
            max_depth: usize,
        ) {
            paths.push(current.clone());
            if current.len() == max_depth {
                return;
            }
            for wrapper in [
                WrapperSpec::Array(0),
                WrapperSpec::Array(1),
                WrapperSpec::Reference(false),
                WrapperSpec::Reference(true),
                WrapperSpec::Generic,
            ] {
                current.push(wrapper);
                extend(paths, current, max_depth);
                current.pop();
            }
        }

        let mut paths = Vec::new();
        extend(&mut paths, &mut Vec::new(), max_depth);
        paths
    }

    #[test]
    fn exhaustive_wrapper_path_policy_is_behavior_neutral() {
        for wrappers in wrapper_paths(4) {
            for initialized in [false, true] {
                for arity in [0, 1, 3] {
                    assert_eq!(
                        classify_binding_annotation(&wrap_tuple(arity, &wrappers), initialized,),
                        expected_tuple_path_disposition(&wrappers, initialized),
                        "wrapper path {wrappers:?}, initialized {initialized}, tuple arity {arity}",
                    );
                }
            }
        }
    }

    #[test]
    fn exact_frozen_disposition_table_is_finite() {
        for initialized in [false, true] {
            assert_eq!(
                classify_binding_annotation(&tuple(0), initialized),
                if initialized {
                    BindingAnnotationDisposition::PreservedQuarantinedTopology
                } else {
                    BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology(
                        RejectedAnnotationTopology::Tuple,
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
                        BindingAnnotationDisposition::PreservedQuarantinedTopology
                    } else {
                        BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology(
                            RejectedAnnotationTopology::ArrayTuple,
                        )
                    }
                );
                assert_eq!(
                    classify_binding_annotation(
                        &Type::Array(Box::new(Type::Array(Box::new(tuple(1)), count)), count,),
                        initialized,
                    ),
                    if initialized {
                        BindingAnnotationDisposition::PreservedQuarantinedTopology
                    } else {
                        BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology(
                            RejectedAnnotationTopology::DoubleArrayTuple,
                        )
                    }
                );
                for mutable in [false, true] {
                    assert_eq!(
                        classify_binding_annotation(
                            &Type::Reference(Box::new(tuple(1)), mutable),
                            initialized,
                        ),
                        BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology(
                            RejectedAnnotationTopology::ReferenceTuple,
                        )
                    );
                    let reference_array =
                        Type::Reference(Box::new(Type::Array(Box::new(tuple(3)), count)), mutable);
                    let expected = if initialized && count == 0 {
                        BindingAnnotationDisposition::PreservedQuarantinedTopology
                    } else {
                        BindingAnnotationDisposition::ExplicitlyRejectedAnnotationTopology(
                            RejectedAnnotationTopology::ReferenceArrayTuple,
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
            BindingAnnotationDisposition::PreservedQuarantinedTopology
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
                BindingAnnotationDisposition::SupportedBindingAnnotation(
                    BindingContractKind::NumericScalar(expected)
                )
            );
            assert_eq!(
                classify_binding_annotation(&Type::Array(Box::new(named(name)), 2), true,),
                BindingAnnotationDisposition::SupportedBindingAnnotation(
                    BindingContractKind::FixedNumericArray {
                        element: expected,
                        count: 2,
                    }
                )
            );
            let zero = Type::Array(Box::new(named(name)), 0);
            assert_eq!(
                classify_binding_annotation(&zero, true),
                BindingAnnotationDisposition::PreservedQuarantinedTopology
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
