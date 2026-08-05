use crate::ast::{Expression, Parameter, Type};
use crate::copy_place_contract::{
    CopyPlaceDisposition, CopyPlaceExecutionContext, classify_copy_place_annotation,
    classify_copy_place_type,
};
use crate::ir::LogicalType;
use crate::struct_contract::StructRegistry;
use crate::types::{OwnershipState, Ty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalReferenceContract {
    pub(crate) pointee: Ty,
    pub(crate) logical_pointee: LogicalType,
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

impl ReferenceFunctionContract {
    pub(crate) fn mutable_parameter_pointee(&self) -> Option<&Ty> {
        self.parameters.iter().find_map(|(_, parameter)| {
            let Ty::Reference(pointee, true) = &parameter.ty else {
                return None;
            };
            Some(pointee.as_ref())
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceFunctionDisposition {
    Supported(ReferenceFunctionContract),
    ExplicitlyRejected(String),
    Preserved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceCallDisposition {
    Supported(ReferenceCallContract),
    ExplicitlyRejected(String),
    Preserved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReferenceCallSourceMode {
    DirectOwnerBorrow,
    MutableReferenceIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceCallContract {
    pub(crate) reference: LocalReferenceContract,
    pub(crate) source_mode: ReferenceCallSourceMode,
}

impl ReferenceCallContract {
    pub(crate) fn reference_type(&self) -> Ty {
        self.reference.reference_type()
    }
}

#[cfg(test)]
fn scalar_contract(ty: &Ty) -> Option<LocalReferenceContract> {
    let logical_pointee = match ty {
        Ty::Int => LogicalType::Int,
        Ty::Float => LogicalType::Float,
        Ty::Bool => LogicalType::Bool,
        _ => return None,
    };
    Some(LocalReferenceContract {
        pointee: ty.clone(),
        logical_pointee,
        mutable: false,
    })
}

#[cfg(test)]
fn scalar_reference_contract(ty: &Ty, mutable: bool) -> Option<LocalReferenceContract> {
    scalar_contract(ty).map(|mut contract| {
        contract.mutable = mutable;
        contract
    })
}

fn reference_transport_type(
    annotation: &Type,
    registry: &StructRegistry,
) -> Result<Option<ReferenceTransportTypeContract>, &'static str> {
    let Type::Reference(pointee, mutable) = annotation else {
        return Ok(None);
    };
    let context = if *mutable {
        CopyPlaceExecutionContext::AdmittedMutableReference
    } else {
        CopyPlaceExecutionContext::AdmittedImmutableReference
    };
    let pointee = match classify_copy_place_annotation(pointee, registry, context) {
        CopyPlaceDisposition::Supported(contract) => ReferenceTransportTypeContract {
            ty: contract.ty,
            logical_type: contract.logical_type,
        },
        CopyPlaceDisposition::ExplicitlyRejected(_) if *mutable => {
            return Err("mutable reference parameter pointee is not admitted Copy-data");
        }
        CopyPlaceDisposition::ExplicitlyRejected(_) => {
            return Err("immutable reference parameter pointee is not admitted Copy-data");
        }
        CopyPlaceDisposition::Preserved => unreachable!("reference context is admitted"),
    };
    Ok(Some(ReferenceTransportTypeContract {
        ty: Ty::Reference(Box::new(pointee.ty), *mutable),
        logical_type: if *mutable {
            LogicalType::MutableReference {
                pointee: Box::new(pointee.logical_type),
            }
        } else {
            LogicalType::ImmutableReference {
                pointee: Box::new(pointee.logical_type),
            }
        },
    }))
}

pub(crate) fn classify_reference_function(
    name: &str,
    parameters: &[Parameter],
    return_type: Option<&Type>,
    type_params: &[String],
    registry: &StructRegistry,
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

    let mutable_parameters = parameters
        .iter()
        .filter(|parameter| matches!(parameter.param_type, Type::Reference(_, true)))
        .count();
    if mutable_parameters > 0
        && (parameters.len() != 1 || !matches!(parameters[0].param_type, Type::Reference(_, true)))
    {
        return ReferenceFunctionDisposition::ExplicitlyRejected(
            "mutable reference transport functions require exactly one mutable-reference parameter"
                .to_string(),
        );
    }

    let mut resolved_parameters = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let contract = match reference_transport_type(&parameter.param_type, registry) {
            Ok(Some(contract)) => contract,
            Err(diagnostic) => {
                return ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic.to_string());
            }
            Ok(None) => {
                let contract = match classify_copy_place_annotation(
                    &parameter.param_type,
                    registry,
                    CopyPlaceExecutionContext::AdmittedImmutableReference,
                ) {
                    CopyPlaceDisposition::Supported(contract) => {
                        Some(ReferenceTransportTypeContract {
                            ty: contract.ty,
                            logical_type: contract.logical_type,
                        })
                    }
                    CopyPlaceDisposition::ExplicitlyRejected(_)
                    | CopyPlaceDisposition::Preserved => None,
                };
                match contract {
                    Some(contract) => contract,
                    None => {
                        return ReferenceFunctionDisposition::ExplicitlyRejected(format!(
                            "reference transport function `{name}` parameter `{}` is not admitted Copy-data",
                            parameter.name
                        ));
                    }
                }
            }
        };
        resolved_parameters.push((parameter.name.clone(), contract));
    }

    let result = match return_type {
        Some(annotation) => {
            let contract = match classify_copy_place_annotation(
                annotation,
                registry,
                CopyPlaceExecutionContext::AdmittedImmutableReference,
            ) {
                CopyPlaceDisposition::Supported(contract) => Some(ReferenceTransportTypeContract {
                    ty: contract.ty,
                    logical_type: contract.logical_type,
                }),
                CopyPlaceDisposition::ExplicitlyRejected(_) | CopyPlaceDisposition::Preserved => {
                    None
                }
            };
            match contract {
                Some(contract) => contract,
                None => {
                    return ReferenceFunctionDisposition::ExplicitlyRejected(format!(
                        "reference transport function `{name}` return type is not admitted Copy-data or Void"
                    ));
                }
            }
        }
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

pub(crate) fn classify_reference_call(
    contract: &ReferenceFunctionContract,
    arguments: &[Expression],
    facts: Option<&LocalReferenceSourceFacts>,
    registry: &StructRegistry,
) -> ReferenceCallDisposition {
    let Some(expected_pointee) = contract.mutable_parameter_pointee() else {
        return ReferenceCallDisposition::Preserved;
    };
    if arguments.len() != 1 {
        return ReferenceCallDisposition::ExplicitlyRejected(
            "mutable reference call requires exactly one mutable-reference identifier or direct `&mut` local owner argument".to_string(),
        );
    }
    let Some((source_mode, source)) = reference_call_source_topology(arguments) else {
        return ReferenceCallDisposition::ExplicitlyRejected(
            "mutable reference call requires a mutable-reference identifier or direct `&mut` local owner argument".to_string(),
        );
    };
    if source_mode == ReferenceCallSourceMode::MutableReferenceIdentifier {
        let Expression::Identifier(name) = source else {
            unreachable!("shared mutable-reference call topology retained its identifier")
        };
        let Some(facts) = facts else {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call argument `{name}` is not an initialized local binding"
            ));
        };
        if !facts.initialized {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "Error: Use of uninitialized variable `{name}`."
            ));
        }
        if !facts.local {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call argument `{name}` is not an initialized local binding"
            ));
        }
        let Ty::Reference(actual_pointee, true) = &facts.ty else {
            return ReferenceCallDisposition::ExplicitlyRejected(
                "mutable reference call requires a mutable-reference identifier or direct `&mut` local owner argument".to_string(),
            );
        };
        let reference = match classify_copy_place_type(
            actual_pointee,
            registry,
            CopyPlaceExecutionContext::AdmittedMutableReference,
        ) {
            CopyPlaceDisposition::Supported(contract) => LocalReferenceContract {
                pointee: contract.ty,
                logical_pointee: contract.logical_type,
                mutable: true,
            },
            CopyPlaceDisposition::ExplicitlyRejected(message) => {
                return ReferenceCallDisposition::ExplicitlyRejected(message);
            }
            CopyPlaceDisposition::Preserved => unreachable!("mutable call context is admitted"),
        };
        match facts.ownership {
            OwnershipState::Owned => {}
            OwnershipState::Moved => {
                return ReferenceCallDisposition::ExplicitlyRejected(format!(
                    "cannot reborrow moved mutable reference `{name}`"
                ));
            }
            OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
                return ReferenceCallDisposition::ExplicitlyRejected(format!(
                    "mutable reference call argument `{name}` has an invalid ownership state"
                ));
            }
        }
        return if reference.pointee == *expected_pointee {
            ReferenceCallDisposition::Supported(ReferenceCallContract {
                reference,
                source_mode,
            })
        } else {
            ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call pointee mismatch: expected {expected_pointee}, actual {}",
                reference.pointee
            ))
        };
    }

    match classify_local_borrow(source, true, facts, registry) {
        LocalReferenceDisposition::Supported(contract) if contract.pointee == *expected_pointee => {
            ReferenceCallDisposition::Supported(ReferenceCallContract {
                reference: contract,
                source_mode,
            })
        }
        LocalReferenceDisposition::Supported(contract) => {
            ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call pointee mismatch: expected {expected_pointee}, actual {}",
                contract.pointee
            ))
        }
        LocalReferenceDisposition::ExplicitlyRejected(message) => {
            ReferenceCallDisposition::ExplicitlyRejected(message)
        }
        LocalReferenceDisposition::Preserved => unreachable!(
            "direct mutable call borrow is fully classified by the local reference contract"
        ),
    }
}

fn reference_call_source_topology(
    arguments: &[Expression],
) -> Option<(ReferenceCallSourceMode, &Expression)> {
    let [argument] = arguments else {
        return None;
    };
    match argument {
        Expression::Borrow {
            expr,
            mutable: true,
        } => Some((ReferenceCallSourceMode::DirectOwnerBorrow, expr.as_ref())),
        Expression::Identifier(_) => Some((
            ReferenceCallSourceMode::MutableReferenceIdentifier,
            argument,
        )),
        _ => None,
    }
}

pub(crate) fn reference_call_fact_subject(arguments: &[Expression]) -> Option<&Expression> {
    reference_call_source_topology(arguments).map(|(_, expression)| expression)
}

pub(crate) fn reference_call_source_mode(
    arguments: &[Expression],
) -> Option<ReferenceCallSourceMode> {
    reference_call_source_topology(arguments).map(|(mode, _)| mode)
}

pub(crate) fn classify_local_borrow(
    expression: &Expression,
    mutable: bool,
    facts: Option<&LocalReferenceSourceFacts>,
    registry: &StructRegistry,
) -> LocalReferenceDisposition {
    let Expression::Identifier(name) = expression else {
        let qualifier = if mutable { "mutable " } else { "immutable " };
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "a local {qualifier}Copy-data borrow requires an identifier place"
        ));
    };
    let Some(facts) = facts else {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "local Copy-data borrow source `{name}` is not an initialized local binding"
        ));
    };
    if !facts.initialized {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "Error: Use of uninitialized variable `{name}`."
        ));
    }
    if !facts.local {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "local Copy-data borrow source `{name}` is not an initialized local binding"
        ));
    }
    let context = if mutable {
        CopyPlaceExecutionContext::AdmittedMutableReference
    } else {
        CopyPlaceExecutionContext::AdmittedImmutableReference
    };
    let contract = match classify_copy_place_type(&facts.ty, registry, context) {
        CopyPlaceDisposition::Supported(contract) => LocalReferenceContract {
            pointee: contract.ty,
            logical_pointee: contract.logical_type,
            mutable,
        },
        CopyPlaceDisposition::ExplicitlyRejected(message) => {
            return LocalReferenceDisposition::ExplicitlyRejected(message);
        }
        CopyPlaceDisposition::Preserved => unreachable!("borrow context is admitted"),
    };
    if mutable && !facts.mutable {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "mutable borrow source `{name}` must be declared mutable"
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

pub(crate) fn classify_local_dereference(
    operand: &Ty,
    registry: &StructRegistry,
) -> LocalReferenceDisposition {
    match operand {
        Ty::Reference(pointee, true) => match classify_copy_place_type(
            pointee,
            registry,
            CopyPlaceExecutionContext::AdmittedMutableReference,
        ) {
            CopyPlaceDisposition::Supported(contract) => {
                LocalReferenceDisposition::Supported(LocalReferenceContract {
                    pointee: contract.ty,
                    logical_pointee: contract.logical_type,
                    mutable: true,
                })
            }
            CopyPlaceDisposition::ExplicitlyRejected(message) => {
                LocalReferenceDisposition::ExplicitlyRejected(message)
            }
            CopyPlaceDisposition::Preserved => unreachable!("dereference context is admitted"),
        },
        Ty::Reference(pointee, false) => match classify_copy_place_type(
            pointee,
            registry,
            CopyPlaceExecutionContext::AdmittedImmutableReference,
        ) {
            CopyPlaceDisposition::Supported(contract) => {
                LocalReferenceDisposition::Supported(LocalReferenceContract {
                    pointee: contract.ty,
                    logical_pointee: contract.logical_type,
                    mutable: false,
                })
            }
            CopyPlaceDisposition::ExplicitlyRejected(message) => {
                LocalReferenceDisposition::ExplicitlyRejected(message)
            }
            CopyPlaceDisposition::Preserved => unreachable!("dereference context is admitted"),
        },
        _ => LocalReferenceDisposition::ExplicitlyRejected(
            "cannot dereference a non-reference value".to_string(),
        ),
    }
}

pub(crate) fn classify_local_reference_annotation(
    annotation: &Type,
    initialized: bool,
    registry: &StructRegistry,
) -> LocalReferenceDisposition {
    let Type::Reference(inner, mutable) = annotation else {
        return LocalReferenceDisposition::Preserved;
    };
    if !initialized {
        return LocalReferenceDisposition::Preserved;
    }
    if *mutable {
        return match classify_copy_place_annotation(
            inner,
            registry,
            CopyPlaceExecutionContext::AdmittedMutableReference,
        ) {
            CopyPlaceDisposition::Supported(contract) => {
                LocalReferenceDisposition::Supported(LocalReferenceContract {
                    pointee: contract.ty,
                    logical_pointee: contract.logical_type,
                    mutable: true,
                })
            }
            CopyPlaceDisposition::ExplicitlyRejected(message) => {
                LocalReferenceDisposition::ExplicitlyRejected(message)
            }
            CopyPlaceDisposition::Preserved => {
                unreachable!("reference annotation context is admitted")
            }
        };
    }
    match classify_copy_place_annotation(
        inner,
        registry,
        CopyPlaceExecutionContext::AdmittedImmutableReference,
    ) {
        CopyPlaceDisposition::Supported(contract) => {
            LocalReferenceDisposition::Supported(LocalReferenceContract {
                pointee: contract.ty,
                logical_pointee: contract.logical_type,
                mutable: false,
            })
        }
        CopyPlaceDisposition::ExplicitlyRejected(message) => {
            LocalReferenceDisposition::ExplicitlyRejected(message)
        }
        CopyPlaceDisposition::Preserved => unreachable!("reference annotation context is admitted"),
    }
}

pub(crate) fn classify_mutable_reference_assignment(
    target: &Expression,
    facts: Option<&MutableReferenceAssignmentFacts>,
    rhs: &Ty,
    inside_admitted_function: bool,
    registry: &StructRegistry,
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
            match classify_copy_place_type(
                pointee,
                registry,
                CopyPlaceExecutionContext::AdmittedMutableReference,
            ) {
                CopyPlaceDisposition::Supported(contract) => LocalReferenceContract {
                    pointee: contract.ty,
                    logical_pointee: contract.logical_type,
                    mutable: true,
                },
                CopyPlaceDisposition::ExplicitlyRejected(message) => {
                    return MutableReferenceAssignmentDisposition::ExplicitlyRejected(message);
                }
                CopyPlaceDisposition::Preserved => {
                    unreachable!("mutable assignment context is admitted")
                }
            }
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
        let registry = StructRegistry::default();
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
                &registry,
            );
            assert_eq!(
                disposition,
                LocalReferenceDisposition::Supported(
                    scalar_reference_contract(&pointee, false).expect("scalar contract"),
                )
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
                Some(&mutable_facts),
                &registry,
            ),
            LocalReferenceDisposition::Supported(LocalReferenceContract {
                pointee: Ty::Int,
                mutable: true,
                ..
            })
        ));
        assert!(matches!(
            classify_local_borrow(
                &Expression::IntegerLiteral(1),
                false,
                None,
                &registry,
            ),
            LocalReferenceDisposition::ExplicitlyRejected(message)
                if message.contains("identifier place")
        ));
        assert!(matches!(
            classify_local_dereference(
                &Ty::Reference(Box::new(Ty::String), false),
                &registry,
            ),
            LocalReferenceDisposition::ExplicitlyRejected(message)
                if message.contains("admitted Copy-data")
        ));
        assert_eq!(
            classify_local_reference_annotation(
                &Type::Reference(Box::new(Type::Named("int".to_string())), false),
                false,
                &registry,
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
            &registry,
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
                &[],
                &registry,
            ),
            ReferenceFunctionDisposition::ExplicitlyRejected(message)
                if message.contains("admitted Copy-data")
        ));
        assert_eq!(
            classify_reference_function(
                "plain",
                &[parameter("value", Type::Named("int".to_string()))],
                Some(&Type::Named("int".to_string())),
                &[],
                &registry,
            ),
            ReferenceFunctionDisposition::Preserved
        );
    }

    #[test]
    fn mutable_call_classifier_partitions_direct_identifier_and_rejected_topologies() {
        let registry = StructRegistry::default();
        let parameter = Parameter {
            name: "value".to_string(),
            param_type: Type::Reference(Box::new(Type::Named("int".to_string())), true),
        };
        let ReferenceFunctionDisposition::Supported(function) = classify_reference_function(
            "write",
            &[parameter],
            Some(&Type::Named("int".to_string())),
            &[],
            &registry,
        ) else {
            panic!("sole mutable scalar-reference function must be supported")
        };
        let direct = vec![Expression::Borrow {
            expr: Box::new(Expression::Identifier("owner".to_string())),
            mutable: true,
        }];
        let owner = LocalReferenceSourceFacts {
            ty: Ty::Int,
            mutable: true,
            initialized: true,
            local: true,
            ownership: OwnershipState::Owned,
        };
        assert!(matches!(
            classify_reference_call(&function, &direct, Some(&owner), &registry),
            ReferenceCallDisposition::Supported(ReferenceCallContract {
                source_mode: ReferenceCallSourceMode::DirectOwnerBorrow,
                ..
            })
        ));
        assert_eq!(
            reference_call_source_mode(&direct),
            Some(ReferenceCallSourceMode::DirectOwnerBorrow)
        );

        let identifier = vec![Expression::Identifier("alias".to_string())];
        let alias = LocalReferenceSourceFacts {
            ty: Ty::Reference(Box::new(Ty::Int), true),
            mutable: false,
            initialized: true,
            local: true,
            ownership: OwnershipState::Owned,
        };
        assert!(matches!(
            classify_reference_call(&function, &identifier, Some(&alias), &registry),
            ReferenceCallDisposition::Supported(ReferenceCallContract {
                source_mode: ReferenceCallSourceMode::MutableReferenceIdentifier,
                ..
            })
        ));
        assert!(matches!(
            reference_call_fact_subject(&identifier),
            Some(Expression::Identifier(name)) if name == "alias"
        ));

        let immutable_alias = LocalReferenceSourceFacts {
            ty: Ty::Reference(Box::new(Ty::Int), false),
            ..alias.clone()
        };
        assert!(matches!(
            classify_reference_call(
                &function,
                &identifier,
                Some(&immutable_alias),
                &registry,
            ),
            ReferenceCallDisposition::ExplicitlyRejected(message)
                if message.contains("mutable-reference identifier")
        ));
        let moved_alias = LocalReferenceSourceFacts {
            ownership: OwnershipState::Moved,
            ..alias
        };
        assert!(matches!(
            classify_reference_call(&function, &identifier, Some(&moved_alias), &registry),
            ReferenceCallDisposition::ExplicitlyRejected(message)
                if message.contains("moved mutable reference")
        ));
        assert!(matches!(
            classify_reference_call(
                &function,
                &[Expression::IntegerLiteral(1)],
                Some(&owner),
                &registry,
            ),
            ReferenceCallDisposition::ExplicitlyRejected(message)
                if message.contains("mutable-reference identifier")
        ));
        assert!(matches!(
            classify_reference_call(&function, &[], None, &registry),
            ReferenceCallDisposition::ExplicitlyRejected(message)
                if message.contains("exactly one")
        ));
    }
}
