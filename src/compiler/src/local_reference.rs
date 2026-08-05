use crate::ast::{Expression, Parameter, Type};
use crate::ir::LogicalType;
use crate::types::{OwnershipState, Ty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalReferenceContract {
    pub(crate) pointee: Ty,
    pub(crate) mutable: bool,
}

impl LocalReferenceContract {
    pub(crate) fn reference_type(&self) -> Ty {
        Ty::Reference(Box::new(self.pointee.clone()), self.mutable)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct LocalReferenceSourceFacts {
    pub(crate) ty: Ty,
    pub(crate) mutable: bool,
    pub(crate) initialized: bool,
    pub(crate) local: bool,
    pub(crate) ownership: OwnershipState,
}

#[derive(Debug, Clone)]
pub(crate) struct MutableReferenceAssignmentFacts {
    pub(crate) ty: Ty,
    pub(crate) initialized: bool,
    pub(crate) local: bool,
    pub(crate) ownership: OwnershipState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LocalReferenceDisposition {
    Supported(LocalReferenceContract),
    ExplicitlyRejected(String),
    Preserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum MutableReferenceAssignmentDisposition {
    Supported(LocalReferenceContract),
    ExplicitlyRejected(String),
    Preserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceTransportTypeContract {
    pub(crate) ty: Ty,
    pub(crate) logical_type: LogicalType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceFunctionContract {
    pub(crate) name: String,
    pub(crate) parameters: Vec<(String, ReferenceTransportTypeContract)>,
    pub(crate) result: ReferenceTransportTypeContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceFunctionDisposition {
    Supported(ReferenceFunctionContract),
    ExplicitlyRejected(String),
    Preserved,
}

fn scalar_contract(ty: &Ty) -> Option<LocalReferenceContract> {
    matches!(ty, Ty::Int | Ty::Float | Ty::Bool).then(|| LocalReferenceContract {
        pointee: ty.clone(),
        mutable: false,
    })
}

fn scalar_reference_contract(ty: &Ty, mutable: bool) -> Option<LocalReferenceContract> {
    scalar_contract(ty).map(|mut contract| {
        contract.mutable = mutable;
        contract
    })
}

fn scalar_transport_type(annotation: &Type) -> Option<ReferenceTransportTypeContract> {
    let (ty, logical_type) = match annotation {
        Type::Named(name) if matches!(name.as_str(), "int" | "i32") => (Ty::Int, LogicalType::Int),
        Type::Named(name) if matches!(name.as_str(), "float" | "f64") => {
            (Ty::Float, LogicalType::Float)
        }
        Type::Named(name) if name == "bool" => (Ty::Bool, LogicalType::Bool),
        _ => return None,
    };
    Some(ReferenceTransportTypeContract { ty, logical_type })
}

fn reference_transport_type(
    annotation: &Type,
) -> Result<Option<ReferenceTransportTypeContract>, &'static str> {
    let Type::Reference(pointee, mutable) = annotation else {
        return Ok(None);
    };
    if *mutable {
        return Err("mutable reference parameters are not supported by CORE-053");
    }
    let Some(pointee) = scalar_transport_type(pointee) else {
        return Err("immutable reference parameters support only Int, Float, or Bool pointees");
    };
    Ok(Some(ReferenceTransportTypeContract {
        ty: Ty::Reference(Box::new(pointee.ty), false),
        logical_type: LogicalType::ImmutableReference {
            pointee: Box::new(pointee.logical_type),
        },
    }))
}

pub(crate) fn classify_reference_function(
    name: &str,
    parameters: &[Parameter],
    return_type: Option<&Type>,
    type_params: &[String],
) -> ReferenceFunctionDisposition {
    let mentions_reference = parameters
        .iter()
        .any(|parameter| matches!(parameter.param_type, Type::Reference(_, _)))
        || return_type.is_some_and(|result| matches!(result, Type::Reference(_, _)));
    if !mentions_reference {
        return ReferenceFunctionDisposition::Preserved;
    }
    if return_type.is_some_and(|result| matches!(result, Type::Reference(_, _))) {
        return ReferenceFunctionDisposition::ExplicitlyRejected(
            "reference results require lifetime semantics and are not supported by CORE-053"
                .to_string(),
        );
    }
    if !type_params.is_empty() {
        return ReferenceFunctionDisposition::ExplicitlyRejected(
            "generic reference transport functions are not supported by CORE-053".to_string(),
        );
    }
    if name == "main" {
        return ReferenceFunctionDisposition::ExplicitlyRejected(
            "process entry cannot use reference parameters".to_string(),
        );
    }

    let mut resolved_parameters = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let contract = match reference_transport_type(&parameter.param_type) {
            Ok(Some(contract)) => contract,
            Err(diagnostic) => {
                return ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic.to_string());
            }
            Ok(None) => match scalar_transport_type(&parameter.param_type) {
                Some(contract) => contract,
                None => {
                    return ReferenceFunctionDisposition::ExplicitlyRejected(format!(
                        "reference transport function `{name}` parameter `{}` is not an admitted scalar or immutable scalar-reference type",
                        parameter.name
                    ));
                }
            },
        };
        resolved_parameters.push((parameter.name.clone(), contract));
    }

    let result = match return_type {
        Some(annotation) => match scalar_transport_type(annotation) {
            Some(contract) => contract,
            None => {
                return ReferenceFunctionDisposition::ExplicitlyRejected(format!(
                    "reference transport function `{name}` return type is not an admitted scalar or Void type"
                ));
            }
        },
        None => ReferenceTransportTypeContract {
            ty: Ty::Void,
            logical_type: LogicalType::Void,
        },
    };

    ReferenceFunctionDisposition::Supported(ReferenceFunctionContract {
        name: name.to_string(),
        parameters: resolved_parameters,
        result,
    })
}

pub(crate) fn classify_local_borrow(
    expression: &Expression,
    mutable: bool,
    facts: Option<&LocalReferenceSourceFacts>,
) -> LocalReferenceDisposition {
    let Expression::Identifier(name) = expression else {
        let qualifier = if mutable { "mutable " } else { "immutable " };
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "a local {qualifier}scalar borrow requires an identifier place"
        ));
    };
    let Some(facts) = facts else {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "local scalar borrow source `{name}` is not an initialized local binding"
        ));
    };
    if !facts.local || !facts.initialized {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "local scalar borrow source `{name}` is not an initialized local binding"
        ));
    }
    let Some(contract) = scalar_reference_contract(&facts.ty, mutable) else {
        let qualifier = if mutable { "mutable " } else { "immutable " };
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "local {qualifier}references support only Int, Float, or Bool pointees"
        ));
    };
    if mutable && !facts.mutable {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "mutable scalar borrow source `{name}` must be declared mutable"
        ));
    }
    let conflict = match (&facts.ownership, mutable) {
        (OwnershipState::Moved, _) => Some(format!("cannot borrow `{name}` because it was moved")),
        (OwnershipState::MutablyBorrowed, true) => Some(format!(
            "cannot borrow `{name}` as mutable because it is already borrowed as mutable"
        )),
        (OwnershipState::MutablyBorrowed, false) => Some(format!(
            "cannot borrow `{name}` as immutable because it is also borrowed as mutable"
        )),
        (OwnershipState::ImmutablyBorrowed(_), true) => Some(format!(
            "cannot borrow `{name}` as mutable because it is also borrowed as immutable"
        )),
        _ => None,
    };
    conflict.map_or(
        LocalReferenceDisposition::Supported(contract),
        LocalReferenceDisposition::ExplicitlyRejected,
    )
}

pub(crate) fn classify_local_dereference(operand: &Ty) -> LocalReferenceDisposition {
    match operand {
        Ty::Reference(pointee, mutable) => scalar_reference_contract(pointee, *mutable).map_or(
            LocalReferenceDisposition::ExplicitlyRejected(
                "local references support only Int, Float, or Bool pointees".to_string(),
            ),
            LocalReferenceDisposition::Supported,
        ),
        _ => LocalReferenceDisposition::ExplicitlyRejected(
            "cannot dereference a non-reference value".to_string(),
        ),
    }
}

pub(crate) fn classify_local_reference_annotation(
    annotation: &Type,
    initialized: bool,
) -> LocalReferenceDisposition {
    let Type::Reference(inner, mutable) = annotation else {
        return LocalReferenceDisposition::Preserved;
    };
    if !initialized {
        return LocalReferenceDisposition::Preserved;
    }
    let pointee = match inner.as_ref() {
        Type::Named(name) if matches!(name.as_str(), "int" | "i32") => Ty::Int,
        Type::Named(name) if matches!(name.as_str(), "float" | "f64") => Ty::Float,
        Type::Named(name) if name == "bool" => Ty::Bool,
        _ => {
            return LocalReferenceDisposition::ExplicitlyRejected(format!(
                "local {}references support only Int, Float, or Bool pointees",
                if *mutable { "mutable " } else { "immutable " }
            ));
        }
    };
    LocalReferenceDisposition::Supported(LocalReferenceContract {
        pointee,
        mutable: *mutable,
    })
}

pub(crate) fn classify_mutable_reference_assignment(
    target: &Expression,
    facts: Option<&MutableReferenceAssignmentFacts>,
    rhs: &Ty,
    inside_admitted_function: bool,
) -> MutableReferenceAssignmentDisposition {
    let Expression::Deref(reference) = target else {
        return MutableReferenceAssignmentDisposition::Preserved;
    };
    if !inside_admitted_function {
        return MutableReferenceAssignmentDisposition::ExplicitlyRejected(
            "mutable reference assignment is supported only inside admitted function bodies"
                .to_string(),
        );
    }
    let Expression::Identifier(name) = reference.as_ref() else {
        return MutableReferenceAssignmentDisposition::ExplicitlyRejected(
            "mutable reference assignment requires a local reference identifier".to_string(),
        );
    };
    let Some(facts) = facts else {
        return MutableReferenceAssignmentDisposition::ExplicitlyRejected(format!(
            "mutable reference assignment target `{name}` is not an initialized local binding"
        ));
    };
    if !facts.local || !facts.initialized {
        return MutableReferenceAssignmentDisposition::ExplicitlyRejected(format!(
            "mutable reference assignment target `{name}` is not an initialized local binding"
        ));
    }
    match facts.ownership {
        OwnershipState::Owned => {}
        OwnershipState::Moved => {
            return MutableReferenceAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign through moved mutable reference `{name}`"
            ));
        }
        OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
            return MutableReferenceAssignmentDisposition::ExplicitlyRejected(format!(
                "mutable reference alias `{name}` has an invalid ownership state"
            ));
        }
    }
    let contract = match &facts.ty {
        Ty::Reference(_, false) => {
            return MutableReferenceAssignmentDisposition::ExplicitlyRejected(
                "assignment through an immutable reference is not supported".to_string(),
            );
        }
        Ty::Reference(pointee, true) => {
            let Some(contract) = scalar_reference_contract(pointee, true) else {
                return MutableReferenceAssignmentDisposition::ExplicitlyRejected(
                    "local mutable references support only Int, Float, or Bool pointees"
                        .to_string(),
                );
            };
            contract
        }
        _ => {
            return MutableReferenceAssignmentDisposition::ExplicitlyRejected(
                "mutable reference assignment requires a mutable reference target".to_string(),
            );
        }
    };
    if contract.pointee != *rhs {
        return MutableReferenceAssignmentDisposition::ExplicitlyRejected(format!(
            "mutable reference assignment type mismatch: expected {}, actual {rhs}",
            contract.pointee
        ));
    }
    MutableReferenceAssignmentDisposition::Supported(contract)
}

pub(crate) fn classify_mutable_reference_binding(
    value: &Expression,
    ty: &Ty,
) -> Result<(), String> {
    if matches!(ty, Ty::Reference(_, true))
        && !matches!(value, Expression::Borrow { mutable: true, .. })
    {
        return Err(
            "mutable reference aliases cannot be copied or relocated by CORE-055".to_string(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_partitions_supported_rejected_and_preserved_reference_shapes() {
        for pointee in [Ty::Int, Ty::Float, Ty::Bool] {
            let facts = LocalReferenceSourceFacts {
                ty: pointee.clone(),
                mutable: true,
                initialized: true,
                local: true,
                ownership: OwnershipState::Owned,
            };
            let disposition = classify_local_borrow(
                &Expression::Identifier("value".to_string()),
                false,
                Some(&facts),
            );
            assert_eq!(
                disposition,
                LocalReferenceDisposition::Supported(LocalReferenceContract {
                    pointee,
                    mutable: false,
                })
            );
        }
        let mutable_facts = LocalReferenceSourceFacts {
            ty: Ty::Int,
            mutable: true,
            initialized: true,
            local: true,
            ownership: OwnershipState::Owned,
        };
        assert!(matches!(
            classify_local_borrow(
                &Expression::Identifier("value".to_string()),
                true,
                Some(&mutable_facts)
            ),
            LocalReferenceDisposition::Supported(LocalReferenceContract {
                pointee: Ty::Int,
                mutable: true
            })
        ));
        assert!(matches!(
            classify_local_borrow(&Expression::IntegerLiteral(1), false, None),
            LocalReferenceDisposition::ExplicitlyRejected(message)
                if message.contains("identifier place")
        ));
        assert!(matches!(
            classify_local_dereference(&Ty::Reference(Box::new(Ty::String), false)),
            LocalReferenceDisposition::ExplicitlyRejected(message)
                if message.contains("support only Int, Float, or Bool")
        ));
        assert_eq!(
            classify_local_reference_annotation(
                &Type::Reference(Box::new(Type::Named("int".to_string())), false),
                false
            ),
            LocalReferenceDisposition::Preserved
        );

        let parameter = |name: &str, param_type: Type| Parameter {
            name: name.to_string(),
            param_type,
        };
        let parameters = vec![
            parameter(
                "left",
                Type::Reference(Box::new(Type::Named("int".to_string())), false),
            ),
            parameter("bias", Type::Named("int".to_string())),
            parameter(
                "ready",
                Type::Reference(Box::new(Type::Named("bool".to_string())), false),
            ),
        ];
        let ReferenceFunctionDisposition::Supported(contract) = classify_reference_function(
            "read",
            &parameters,
            Some(&Type::Named("int".to_string())),
            &[],
        ) else {
            panic!("reference-bearing scalar signature must be supported")
        };
        assert_eq!(contract.name, "read");
        assert_eq!(contract.parameters[0].1.ty.to_string(), "&int");
        assert_eq!(
            contract.parameters[2].1.logical_type,
            LogicalType::ImmutableReference {
                pointee: Box::new(LogicalType::Bool)
            }
        );
        assert_eq!(contract.result.logical_type, LogicalType::Int);

        assert!(matches!(
            classify_reference_function(
                "bad",
                &[parameter(
                    "value",
                    Type::Reference(Box::new(Type::Named("String".to_string())), false)
                )],
                Some(&Type::Named("int".to_string())),
                &[]
            ),
            ReferenceFunctionDisposition::ExplicitlyRejected(message)
                if message.contains("support only Int, Float, or Bool")
        ));
        assert_eq!(
            classify_reference_function(
                "plain",
                &[parameter("value", Type::Named("int".to_string()))],
                Some(&Type::Named("int".to_string())),
                &[]
            ),
            ReferenceFunctionDisposition::Preserved
        );
    }
}
