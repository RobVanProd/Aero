use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FunctionCallUse {
    Value,
    Discarded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionCallParameter {
    pub(crate) name: Option<String>,
    pub(crate) ty: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FunctionCallTarget {
    Admitted {
        parameters: Option<Vec<FunctionCallParameter>>,
        result: Ty,
    },
    Callable {
        result: Ty,
    },
    DeclaredUnadmitted,
    Missing,
    PreservedContext {
        diagnostic: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FunctionCallFacts {
    pub(crate) name: String,
    pub(crate) target: FunctionCallTarget,
    pub(crate) arguments: Vec<Ty>,
    pub(crate) use_context: FunctionCallUse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AcceptedFunctionCall {
    pub(crate) result: Ty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FunctionCallDisposition {
    Supported(AcceptedFunctionCall),
    ExplicitlyRejected(String),
    PreservedContext(String),
}

pub(crate) fn unsupported_function_call_diagnostic(name: &str, detail: &str) -> String {
    format!("Unsupported function call `{name}`: {detail}")
}

fn reject(name: &str, detail: String) -> FunctionCallDisposition {
    FunctionCallDisposition::ExplicitlyRejected(unsupported_function_call_diagnostic(name, &detail))
}

fn classify_admitted_call(
    name: &str,
    parameters: Option<Vec<FunctionCallParameter>>,
    result: Ty,
    arguments: Vec<Ty>,
    use_context: FunctionCallUse,
) -> FunctionCallDisposition {
    if result == Ty::Void && use_context == FunctionCallUse::Value {
        return reject(
            name,
            format!("Error: void function `{name}` cannot be used as a value."),
        );
    }

    if let Some(parameters) = parameters {
        if parameters.len() != arguments.len() {
            return reject(
                name,
                format!(
                    "Error: Function `{name}` arity mismatch: expected {}, actual {}.",
                    parameters.len(),
                    arguments.len()
                ),
            );
        }

        for (index, (parameter, actual)) in parameters.iter().zip(&arguments).enumerate() {
            if parameter.ty == *actual {
                continue;
            }
            let detail = if let Some(parameter_name) = &parameter.name {
                format!(
                    "Error: Function `{name}` parameter `{parameter_name}` type mismatch: expected {}, actual {}.",
                    parameter.ty, actual
                )
            } else {
                format!(
                    "Error: Function `{name}` argument {} type mismatch: expected {}, actual {}.",
                    index + 1,
                    parameter.ty,
                    actual
                )
            };
            return reject(name, detail);
        }
    }

    FunctionCallDisposition::Supported(AcceptedFunctionCall { result })
}

pub(crate) fn classify_function_call(facts: FunctionCallFacts) -> FunctionCallDisposition {
    match facts.target {
        FunctionCallTarget::Admitted { parameters, result } => classify_admitted_call(
            &facts.name,
            parameters,
            result,
            facts.arguments,
            facts.use_context,
        ),
        FunctionCallTarget::Callable { result } => classify_admitted_call(
            &facts.name,
            None,
            result,
            facts.arguments,
            facts.use_context,
        ),
        FunctionCallTarget::DeclaredUnadmitted => reject(
            &facts.name,
            format!(
                "Error: Function `{}` has no admitted executable signature.",
                facts.name
            ),
        ),
        FunctionCallTarget::Missing => reject(
            &facts.name,
            format!("Error: Function `{}` is not defined.", facts.name),
        ),
        FunctionCallTarget::PreservedContext { diagnostic } => {
            FunctionCallDisposition::PreservedContext(diagnostic)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(target: FunctionCallTarget, arguments: Vec<Ty>) -> FunctionCallFacts {
        FunctionCallFacts {
            name: "probe".to_string(),
            target,
            arguments,
            use_context: FunctionCallUse::Value,
        }
    }

    #[test]
    fn classifies_the_complete_function_call_product() {
        let admitted = FunctionCallTarget::Admitted {
            parameters: Some(vec![FunctionCallParameter {
                name: Some("value".to_string()),
                ty: Ty::Int,
            }]),
            result: Ty::Bool,
        };
        assert!(matches!(
            classify_function_call(facts(admitted.clone(), vec![Ty::Int])),
            FunctionCallDisposition::Supported(AcceptedFunctionCall { result: Ty::Bool })
        ));
        assert!(matches!(
            classify_function_call(facts(admitted.clone(), vec![])),
            FunctionCallDisposition::ExplicitlyRejected(message)
                if message.contains("arity mismatch")
        ));
        assert!(matches!(
            classify_function_call(facts(admitted, vec![Ty::Float])),
            FunctionCallDisposition::ExplicitlyRejected(message)
                if message.contains("parameter `value` type mismatch")
        ));
        assert!(matches!(
            classify_function_call(facts(FunctionCallTarget::Missing, vec![])),
            FunctionCallDisposition::ExplicitlyRejected(message)
                if message.contains("is not defined")
        ));
        assert!(matches!(
            classify_function_call(facts(FunctionCallTarget::DeclaredUnadmitted, vec![])),
            FunctionCallDisposition::ExplicitlyRejected(message)
                if message.contains("no admitted executable signature")
        ));
        assert!(matches!(
            classify_function_call(facts(
                FunctionCallTarget::PreservedContext {
                    diagnostic: "constructor context".to_string(),
                },
                vec![],
            )),
            FunctionCallDisposition::PreservedContext(message)
                if message == "constructor context"
        ));
        assert!(matches!(
            classify_function_call(facts(
                FunctionCallTarget::Callable { result: Ty::Int },
                vec![Ty::Bool],
            )),
            FunctionCallDisposition::Supported(AcceptedFunctionCall { result: Ty::Int })
        ));
    }

    #[test]
    fn void_is_supported_only_when_discarded() {
        let target = FunctionCallTarget::Admitted {
            parameters: Some(vec![]),
            result: Ty::Void,
        };
        assert!(matches!(
            classify_function_call(facts(target.clone(), vec![])),
            FunctionCallDisposition::ExplicitlyRejected(message)
                if message.contains("cannot be used as a value")
        ));
        assert!(matches!(
            classify_function_call(FunctionCallFacts {
                name: "probe".to_string(),
                target,
                arguments: vec![],
                use_context: FunctionCallUse::Discarded,
            }),
            FunctionCallDisposition::Supported(AcceptedFunctionCall { result: Ty::Void })
        ));
    }
}
