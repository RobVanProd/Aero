use crate::fixed_array_method::{
    FixedArrayQueryDisposition, FixedArrayQueryKind, FixedArrayQueryValue,
    classify_fixed_array_query,
};
use crate::static_string_method::{StaticStringLenDisposition, classify_static_string_len};
use crate::static_string_predicate::{
    StaticStringPredicateDisposition, StaticStringPredicateKind, classify_static_string_predicate,
};
use crate::struct_contract::StructRegistry;
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IntrinsicMethodPhase {
    Semantic,
    Checked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntrinsicStringQueryKind {
    Length,
    Predicate(StaticStringPredicateKind),
}

impl IntrinsicStringQueryKind {
    fn result(self) -> Ty {
        match self {
            Self::Length => Ty::Int,
            Self::Predicate(_) => Ty::Bool,
        }
    }

    fn expected_arguments(self) -> usize {
        match self {
            Self::Length | Self::Predicate(StaticStringPredicateKind::IsEmpty) => 0,
            Self::Predicate(
                StaticStringPredicateKind::Contains
                | StaticStringPredicateKind::StartsWith
                | StaticStringPredicateKind::EndsWith,
            ) => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IntrinsicMethodLowering {
    ConstantInt(i32),
    ConstantBool(bool),
    Receiver,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IntrinsicMethodDisposition {
    Supported {
        result: Ty,
        lowering: Option<IntrinsicMethodLowering>,
    },
    ExplicitlyRejected(String),
    PreservedContext(String),
}

impl IntrinsicMethodDisposition {
    pub(crate) fn result_type(self) -> Result<Ty, String> {
        match self {
            Self::Supported { result, .. } => Ok(result),
            Self::ExplicitlyRejected(diagnostic) | Self::PreservedContext(diagnostic) => {
                Err(diagnostic)
            }
        }
    }
}

fn unsupported(receiver: &Ty, method: &str, reason: impl AsRef<str>) -> String {
    format!(
        "Unsupported intrinsic method call `{}.{method}()`: {}.",
        receiver,
        reason.as_ref()
    )
}

fn supported(result: Ty, lowering: Option<IntrinsicMethodLowering>) -> IntrinsicMethodDisposition {
    IntrinsicMethodDisposition::Supported { result, lowering }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn classify_intrinsic_method(
    receiver: &Ty,
    method: &str,
    argument_count: usize,
    receiver_static_string: Option<&str>,
    argument_static_strings: &[Option<&str>],
    structs: &StructRegistry,
    phase: IntrinsicMethodPhase,
    preserved_context: bool,
) -> IntrinsicMethodDisposition {
    if preserved_context {
        return IntrinsicMethodDisposition::PreservedContext(unsupported(
            receiver,
            method,
            "syntax is preserved in a generic/impl context, but executable method semantics are not admitted",
        ));
    }

    if let Ty::Array(_, _) = receiver {
        let query = match method {
            "len" => Some(FixedArrayQueryKind::Length),
            "is_empty" => Some(FixedArrayQueryKind::IsEmpty),
            _ => None,
        };
        if let Some(query) = query {
            return match classify_fixed_array_query(receiver, query, argument_count, structs) {
                FixedArrayQueryDisposition::StaticValue { value, .. } => match value {
                    FixedArrayQueryValue::Length(count) => {
                        supported(Ty::Int, Some(IntrinsicMethodLowering::ConstantInt(count)))
                    }
                    FixedArrayQueryValue::IsEmpty(value) => {
                        supported(Ty::Bool, Some(IntrinsicMethodLowering::ConstantBool(value)))
                    }
                },
                FixedArrayQueryDisposition::WrongArity {
                    kind,
                    query,
                    actual,
                } => IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                    receiver,
                    method,
                    format!(
                        "{} .{}() expects exactly 0 arguments, got {actual}",
                        kind.diagnostic_subject(),
                        query.method()
                    ),
                )),
                FixedArrayQueryDisposition::CountOutsideIntRange { kind, query, count } => {
                    IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                        receiver,
                        method,
                        format!(
                            "{} .{}() count {count} is outside the admitted i32 range",
                            kind.diagnostic_subject(),
                            query.method()
                        ),
                    ))
                }
                FixedArrayQueryDisposition::PreserveExistingBehavior => {
                    IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                        receiver,
                        method,
                        "receiver is not an admitted recursive CopyData fixed array",
                    ))
                }
            };
        }
        if method == "iter" {
            return if argument_count == 0 {
                supported(receiver.clone(), Some(IntrinsicMethodLowering::Receiver))
            } else {
                IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                    receiver,
                    method,
                    format!("array .iter() expects exactly 0 arguments, got {argument_count}"),
                ))
            };
        }
        return IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
            receiver,
            method,
            "no executable fixed-array method contract exists",
        ));
    }

    if matches!(receiver, Ty::Vec(_)) {
        return if method == "iter" && argument_count == 0 {
            supported(receiver.clone(), Some(IntrinsicMethodLowering::Receiver))
        } else {
            IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                receiver,
                method,
                "only exact zero-argument Vec .iter() compatibility is admitted",
            ))
        };
    }

    if matches!(receiver, Ty::String) {
        let query = match method {
            "len" => IntrinsicStringQueryKind::Length,
            "is_empty" => IntrinsicStringQueryKind::Predicate(StaticStringPredicateKind::IsEmpty),
            "contains" => IntrinsicStringQueryKind::Predicate(StaticStringPredicateKind::Contains),
            "starts_with" => {
                IntrinsicStringQueryKind::Predicate(StaticStringPredicateKind::StartsWith)
            }
            "ends_with" => IntrinsicStringQueryKind::Predicate(StaticStringPredicateKind::EndsWith),
            _ => {
                return IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                    receiver,
                    method,
                    "no executable compile-time String method contract exists",
                ));
            }
        };
        let expected = query.expected_arguments();
        if argument_count != expected {
            return IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                receiver,
                method,
                format!(
                    "compile-time string .{method}() expects exactly {expected} {}, got {argument_count}",
                    if expected == 1 {
                        "argument"
                    } else {
                        "arguments"
                    }
                ),
            ));
        }
        if phase == IntrinsicMethodPhase::Semantic {
            return supported(query.result(), None);
        }

        match query {
            IntrinsicStringQueryKind::Length => {
                match classify_static_string_len(receiver_static_string, argument_count) {
                    StaticStringLenDisposition::StaticLength(count) => {
                        supported(Ty::Int, Some(IntrinsicMethodLowering::ConstantInt(count)))
                    }
                    StaticStringLenDisposition::WrongArity { actual } => {
                        IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                            receiver,
                            method,
                            format!(
                                "compile-time string .len() expects exactly 0 arguments, got {actual}"
                            ),
                        ))
                    }
                    StaticStringLenDisposition::LengthOutsideIntRange { count } => {
                        IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                            receiver,
                            method,
                            format!(
                                "compile-time string .len() count {count} is outside the admitted i32 range"
                            ),
                        ))
                    }
                    StaticStringLenDisposition::PreserveExistingBehavior => {
                        IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                            receiver,
                            method,
                            "an immutable compile-time String receiver is required",
                        ))
                    }
                }
            }
            IntrinsicStringQueryKind::Predicate(kind) => match classify_static_string_predicate(
                receiver_static_string,
                kind,
                argument_static_strings,
            ) {
                StaticStringPredicateDisposition::StaticBool(value) => {
                    supported(Ty::Bool, Some(IntrinsicMethodLowering::ConstantBool(value)))
                }
                StaticStringPredicateDisposition::WrongArity {
                    kind,
                    expected,
                    actual,
                } => IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                    receiver,
                    method,
                    format!(
                        "compile-time string .{}() expects exactly {expected} {}, got {actual}",
                        kind.method(),
                        if expected == 1 {
                            "argument"
                        } else {
                            "arguments"
                        }
                    ),
                )),
                StaticStringPredicateDisposition::PreserveExistingBehavior => {
                    IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
                        receiver,
                        method,
                        "immutable compile-time String receiver and arguments are required",
                    ))
                }
            },
        }
    } else {
        IntrinsicMethodDisposition::ExplicitlyRejected(unsupported(
            receiver,
            method,
            "receiver type has no executable intrinsic method contract",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IntrinsicMethodDisposition, IntrinsicMethodLowering, IntrinsicMethodPhase,
        classify_intrinsic_method,
    };
    use crate::struct_contract::StructRegistry;
    use crate::types::Ty;

    fn classify(
        receiver: &Ty,
        method: &str,
        argument_count: usize,
        phase: IntrinsicMethodPhase,
    ) -> IntrinsicMethodDisposition {
        classify_intrinsic_method(
            receiver,
            method,
            argument_count,
            None,
            &vec![None; argument_count],
            &StructRegistry::default(),
            phase,
            false,
        )
    }

    #[test]
    fn shared_classifier_closes_receiver_method_arity_phase_and_context_product() {
        assert_eq!(
            classify(
                &Ty::Array(Box::new(Ty::Bool), 0),
                "is_empty",
                0,
                IntrinsicMethodPhase::Checked,
            ),
            IntrinsicMethodDisposition::Supported {
                result: Ty::Bool,
                lowering: Some(IntrinsicMethodLowering::ConstantBool(true)),
            }
        );
        assert_eq!(
            classify(
                &Ty::Array(Box::new(Ty::Tuple(vec![Ty::Int, Ty::Bool])), 2),
                "len",
                0,
                IntrinsicMethodPhase::Semantic,
            ),
            IntrinsicMethodDisposition::Supported {
                result: Ty::Int,
                lowering: Some(IntrinsicMethodLowering::ConstantInt(2)),
            }
        );
        assert_eq!(
            classify(&Ty::String, "contains", 1, IntrinsicMethodPhase::Semantic,),
            IntrinsicMethodDisposition::Supported {
                result: Ty::Bool,
                lowering: None,
            }
        );
        assert!(matches!(
            classify(
                &Ty::String,
                "contains",
                1,
                IntrinsicMethodPhase::Checked,
            ),
            IntrinsicMethodDisposition::ExplicitlyRejected(message)
                if message.contains("compile-time String")
        ));
        for receiver in [
            Ty::Int,
            Ty::Float,
            Ty::Bool,
            Ty::Tuple(vec![Ty::Int, Ty::Bool]),
            Ty::Struct("S".to_string()),
            Ty::Reference(Box::new(Ty::Int), false),
        ] {
            assert!(matches!(
                classify(
                    &receiver,
                    "missing",
                    0,
                    IntrinsicMethodPhase::Semantic,
                ),
                IntrinsicMethodDisposition::ExplicitlyRejected(message)
                    if message.contains("Unsupported intrinsic method call")
                        && message.contains("missing")
            ));
        }
        assert!(matches!(
            classify_intrinsic_method(
                &Ty::TypeParam("T".to_string()),
                "method",
                0,
                None,
                &[],
                &StructRegistry::default(),
                IntrinsicMethodPhase::Semantic,
                true,
            ),
            IntrinsicMethodDisposition::PreservedContext(message)
                if message.contains("generic/impl context")
        ));
    }
}
