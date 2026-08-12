use crate::ast::{Expression, Parameter, Type};
use crate::copy_place_contract::{
    CopyPlaceDisposition, CopyPlaceExecutionContext, classify_copy_place_annotation,
    classify_copy_place_type,
};
use crate::enum_match_contract::EnumRegistry;
use crate::ir::LogicalType;
use crate::scalar_assignment::{
    OwnedPlaceAssignmentTargetFacts, ProjectedCopyDataPlaceContract,
    ProjectedCopyDataPlaceDisposition, ProjectedCopyDataPlaceUse,
    classify_projected_copydata_place, projected_copydata_place_array_selectors,
};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReferencePointeeContext {
    Immutable,
    Mutable,
}

pub(crate) fn classify_reference_pointee_annotation(
    annotation: &Type,
    context: ReferencePointeeContext,
    structs: &StructRegistry,
    enums: &EnumRegistry,
) -> Result<ReferenceTransportTypeContract, String> {
    match enums.reference_annotation_type(annotation) {
        Ok(Some(contract)) => {
            return Ok(ReferenceTransportTypeContract {
                ty: contract.ty,
                logical_type: contract.logical_type,
            });
        }
        Err(error) => return Err(error.diagnostic()),
        Ok(None) => {}
    }
    let copy_context = match context {
        ReferencePointeeContext::Immutable => CopyPlaceExecutionContext::AdmittedImmutableReference,
        ReferencePointeeContext::Mutable => CopyPlaceExecutionContext::AdmittedMutableReference,
    };
    match classify_copy_place_annotation(annotation, structs, copy_context) {
        CopyPlaceDisposition::Supported(contract) => Ok(ReferenceTransportTypeContract {
            ty: contract.ty,
            logical_type: contract.logical_type,
        }),
        CopyPlaceDisposition::ExplicitlyRejected(message) => Err(message),
        CopyPlaceDisposition::Preserved => unreachable!("reference pointee context is admitted"),
    }
}

pub(crate) fn classify_reference_pointee_type(
    ty: &Ty,
    context: ReferencePointeeContext,
    structs: &StructRegistry,
    enums: &EnumRegistry,
) -> Result<ReferenceTransportTypeContract, String> {
    match enums.reference_pointee_type(ty) {
        Ok(Some(contract)) => {
            return Ok(ReferenceTransportTypeContract {
                ty: contract.ty,
                logical_type: contract.logical_type,
            });
        }
        Err(error) => return Err(error.diagnostic()),
        Ok(None) => {}
    }
    let copy_context = match context {
        ReferencePointeeContext::Immutable => CopyPlaceExecutionContext::AdmittedImmutableReference,
        ReferencePointeeContext::Mutable => CopyPlaceExecutionContext::AdmittedMutableReference,
    };
    match classify_copy_place_type(ty, structs, copy_context) {
        CopyPlaceDisposition::Supported(contract) => Ok(ReferenceTransportTypeContract {
            ty: contract.ty,
            logical_type: contract.logical_type,
        }),
        CopyPlaceDisposition::ExplicitlyRejected(message) => Err(message),
        CopyPlaceDisposition::Preserved => unreachable!("reference pointee context is admitted"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceFunctionContract {
    pub(crate) name: String,
    pub(crate) parameters: Vec<(String, ReferenceTransportTypeContract)>,
    pub(crate) result: ReferenceTransportTypeContract,
}

impl ReferenceFunctionContract {
    pub(crate) fn reference_parameters(&self) -> Vec<(usize, &Ty)> {
        self.parameters
            .iter()
            .enumerate()
            .filter_map(|(index, (_, parameter))| {
                let Ty::Reference(pointee, _) = &parameter.ty else {
                    return None;
                };
                Some((index, pointee.as_ref()))
            })
            .collect()
    }

    pub(crate) fn mutable_parameters(&self) -> Vec<(usize, &Ty)> {
        self.parameters
            .iter()
            .enumerate()
            .filter_map(|(index, (_, parameter))| {
                let Ty::Reference(pointee, true) = &parameter.ty else {
                    return None;
                };
                Some((index, pointee.as_ref()))
            })
            .collect()
    }
}

pub(crate) fn admitted_reference_parameter_topology(
    reference_parameters: usize,
    mutable_parameters: usize,
) -> bool {
    mutable_parameters <= reference_parameters
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
    ProjectedOwnerBorrow,
    MutableReferenceIdentifier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceCallArgumentContract {
    pub(crate) reference: LocalReferenceContract,
    pub(crate) source_mode: ReferenceCallSourceMode,
    pub(crate) reference_parameter_index: usize,
    pub(crate) projected: Option<ProjectedCopyDataPlaceContract>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceCallContract {
    pub(crate) reference_arguments: Vec<ReferenceCallArgumentContract>,
}

impl ReferenceCallContract {
    pub(crate) fn reference_argument(
        &self,
        parameter_index: usize,
    ) -> Option<&ReferenceCallArgumentContract> {
        self.reference_arguments
            .iter()
            .find(|argument| argument.reference_parameter_index == parameter_index)
    }

    pub(crate) fn is_mutable_parameter(&self, parameter_index: usize) -> bool {
        self.reference_argument(parameter_index)
            .is_some_and(|argument| argument.reference.mutable)
    }

    pub(crate) fn reference_type(&self, parameter_index: usize) -> Option<Ty> {
        self.reference_argument(parameter_index)
            .map(|argument| argument.reference.reference_type())
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
    structs: &StructRegistry,
    enums: &EnumRegistry,
) -> Result<Option<ReferenceTransportTypeContract>, String> {
    let Type::Reference(pointee, mutable) = annotation else {
        return Ok(None);
    };
    let context = if *mutable {
        ReferencePointeeContext::Mutable
    } else {
        ReferencePointeeContext::Immutable
    };
    let pointee = classify_reference_pointee_annotation(pointee, context, structs, enums).map_err(
        |message| {
            if enums.annotation_mentions_declared_enum(pointee) {
                message
            } else if *mutable {
                "mutable reference parameter pointee is not admitted Copy-data".to_string()
            } else {
                "immutable reference parameter pointee is not admitted Copy-data".to_string()
            }
        },
    )?;
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

#[cfg(test)]
pub(crate) fn classify_reference_function(
    name: &str,
    parameters: &[Parameter],
    return_type: Option<&Type>,
    type_params: &[String],
    registry: &StructRegistry,
) -> ReferenceFunctionDisposition {
    classify_reference_function_with_enums(
        name,
        parameters,
        return_type,
        type_params,
        registry,
        &EnumRegistry::default(),
    )
}

pub(crate) fn classify_reference_function_with_enums(
    name: &str,
    parameters: &[Parameter],
    return_type: Option<&Type>,
    type_params: &[String],
    registry: &StructRegistry,
    enums: &EnumRegistry,
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

    let reference_parameters = parameters
        .iter()
        .filter(|parameter| matches!(parameter.param_type, Type::Reference(_, _)))
        .count();
    let mutable_parameters = parameters
        .iter()
        .filter(|parameter| matches!(parameter.param_type, Type::Reference(_, true)))
        .count();
    if !admitted_reference_parameter_topology(reference_parameters, mutable_parameters) {
        return ReferenceFunctionDisposition::ExplicitlyRejected(
            "reference transport functions support at most one mutable-reference parameter; simultaneous mutable-reference parameters are not supported".to_string(),
        );
    }

    let mut resolved_parameters = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let contract = match reference_transport_type(&parameter.param_type, registry, enums) {
            Ok(Some(contract)) => contract,
            Err(diagnostic) => {
                return ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic);
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

#[cfg(test)]
pub(crate) fn classify_reference_call(
    contract: &ReferenceFunctionContract,
    arguments: &[Expression],
    facts: Option<&LocalReferenceSourceFacts>,
    registry: &StructRegistry,
) -> ReferenceCallDisposition {
    classify_reference_call_with_enums(
        contract,
        arguments,
        |_| facts.cloned(),
        |_| Ok(Ty::Int),
        registry,
        &EnumRegistry::default(),
    )
}

pub(crate) fn classify_reference_call_with_enums<F, S>(
    contract: &ReferenceFunctionContract,
    arguments: &[Expression],
    mut facts_for_subject: F,
    mut selector_type_for_subject: S,
    registry: &StructRegistry,
    enums: &EnumRegistry,
) -> ReferenceCallDisposition
where
    F: FnMut(&Expression) -> Option<LocalReferenceSourceFacts>,
    S: FnMut(&Expression) -> Result<Ty, String>,
{
    let mutable_parameters = contract.mutable_parameters();
    let reference_parameters = contract.reference_parameters();
    let has_projected_argument = reference_parameters.iter().any(|(index, _)| {
        matches!(
            arguments.get(*index),
            Some(Expression::Borrow { expr, .. })
                if matches!(
                    expr.as_ref(),
                    Expression::FieldAccess { .. }
                        | Expression::TupleIndex { .. }
                        | Expression::IndexAccess { .. }
                )
        )
    });
    if mutable_parameters.is_empty() && !has_projected_argument {
        return ReferenceCallDisposition::Preserved;
    }
    if arguments.len() != contract.parameters.len() {
        if mutable_parameters.is_empty() {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "projected reference call requires exactly {} arguments",
                contract.parameters.len()
            ));
        }
        if contract.parameters.len() == 1 {
            return ReferenceCallDisposition::ExplicitlyRejected(
                "mutable reference call requires exactly one mutable-reference identifier or direct `&mut` local owner argument".to_string(),
            );
        }
        let positions = mutable_parameters
            .iter()
            .map(|(index, _)| (index + 1).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return ReferenceCallDisposition::ExplicitlyRejected(format!(
            "mutable reference call requires exactly {} arguments with mutable references at positions {positions}",
            contract.parameters.len(),
        ));
    }

    let mutable_parameter_indices = mutable_parameters
        .iter()
        .map(|(index, _)| *index)
        .collect::<Vec<_>>();
    let mut source_names = Vec::with_capacity(mutable_parameters.len());
    let mut reference_arguments = Vec::with_capacity(reference_parameters.len());
    for (reference_parameter_index, expected_pointee) in mutable_parameters {
        let Some((source_mode, source)) =
            reference_call_source_topology_at(contract, arguments, reference_parameter_index)
        else {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call argument at position {} requires a mutable-reference identifier or direct `&mut` local owner argument",
                reference_parameter_index + 1
            ));
        };
        let argument = match if source_mode == ReferenceCallSourceMode::ProjectedOwnerBorrow {
            classify_projected_reference_call_argument(
                source,
                true,
                reference_parameter_index,
                expected_pointee,
                &mut facts_for_subject,
                &mut selector_type_for_subject,
                registry,
            )
        } else {
            let facts = facts_for_subject(source);
            classify_mutable_reference_call_argument(
                source,
                source_mode,
                reference_parameter_index,
                expected_pointee,
                facts.as_ref(),
                registry,
                enums,
            )
        } {
            Ok(argument) => argument,
            Err(message) => return ReferenceCallDisposition::ExplicitlyRejected(message),
        };
        let source_name = if let Some(projected) = &argument.projected {
            projected.root_name.as_str()
        } else {
            let Expression::Identifier(source_name) = source else {
                unreachable!("admitted direct mutable call source retains an identifier place")
            };
            source_name.as_str()
        };
        if source_names.iter().any(|prior| prior == source_name) {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call requires pairwise-distinct source identities; `{source_name}` is used by more than one mutable argument"
            ));
        }
        if arguments.iter().enumerate().any(|(index, argument)| {
            !mutable_parameter_indices.contains(&index)
                && expression_mentions_identifier(argument, source_name)
        }) {
            return ReferenceCallDisposition::ExplicitlyRejected(format!(
                "mutable reference call non-mutable arguments must be independent of reference source `{source_name}`"
            ));
        }
        source_names.push(source_name.to_string());
        reference_arguments.push(argument);
    }

    for (reference_parameter_index, expected_pointee) in reference_parameters {
        if mutable_parameter_indices.contains(&reference_parameter_index) {
            continue;
        }
        let Some(Expression::Borrow {
            expr,
            mutable: false,
        }) = arguments.get(reference_parameter_index)
        else {
            continue;
        };
        if !matches!(
            expr.as_ref(),
            Expression::FieldAccess { .. }
                | Expression::TupleIndex { .. }
                | Expression::IndexAccess { .. }
        ) {
            continue;
        }
        let argument = match classify_projected_reference_call_argument(
            expr,
            false,
            reference_parameter_index,
            expected_pointee,
            &mut facts_for_subject,
            &mut selector_type_for_subject,
            registry,
        ) {
            Ok(argument) => argument,
            Err(message) => return ReferenceCallDisposition::ExplicitlyRejected(message),
        };
        reference_arguments.push(argument);
    }

    ReferenceCallDisposition::Supported(ReferenceCallContract {
        reference_arguments,
    })
}

fn classify_projected_reference_call_argument<F, S>(
    source: &Expression,
    mutable: bool,
    reference_parameter_index: usize,
    expected_pointee: &Ty,
    facts_for_subject: &mut F,
    selector_type_for_subject: &mut S,
    registry: &StructRegistry,
) -> Result<ReferenceCallArgumentContract, String>
where
    F: FnMut(&Expression) -> Option<LocalReferenceSourceFacts>,
    S: FnMut(&Expression) -> Result<Ty, String>,
{
    let selectors = projected_copydata_place_array_selectors(source)?
        .expect("projected reference call retained a projected place");
    let selector_types = selectors
        .into_iter()
        .map(&mut *selector_type_for_subject)
        .collect::<Result<Vec<_>, _>>()?;
    let use_context = if mutable {
        ProjectedCopyDataPlaceUse::MutableCallLoan
    } else {
        ProjectedCopyDataPlaceUse::ImmutableCallLoan
    };
    let projected = match classify_projected_copydata_place(
        source,
        expected_pointee,
        &selector_types,
        true,
        registry,
        use_context,
        |root| {
            facts_for_subject(&Expression::Identifier(root.to_string())).map(|facts| {
                OwnedPlaceAssignmentTargetFacts {
                    ty: facts.ty,
                    mutable: facts.mutable,
                    initialized: facts.initialized,
                    local: facts.local,
                    ownership: facts.ownership,
                }
            })
        },
    ) {
        ProjectedCopyDataPlaceDisposition::Supported(contract) => contract,
        ProjectedCopyDataPlaceDisposition::ExplicitlyRejected(message) => return Err(message),
        ProjectedCopyDataPlaceDisposition::PreserveExistingBehavior => {
            unreachable!("projected reference-call topology was already identified")
        }
    };
    Ok(ReferenceCallArgumentContract {
        reference: LocalReferenceContract {
            pointee: projected.leaf_type.clone(),
            logical_pointee: projected.leaf_logical_type.clone(),
            mutable,
        },
        source_mode: ReferenceCallSourceMode::ProjectedOwnerBorrow,
        reference_parameter_index,
        projected: Some(projected),
    })
}

fn classify_mutable_reference_call_argument(
    source: &Expression,
    source_mode: ReferenceCallSourceMode,
    reference_parameter_index: usize,
    expected_pointee: &Ty,
    facts: Option<&LocalReferenceSourceFacts>,
    registry: &StructRegistry,
    enums: &EnumRegistry,
) -> Result<ReferenceCallArgumentContract, String> {
    let source_name = match source {
        Expression::Identifier(name) => Some(name.as_str()),
        _ => None,
    };
    if source_mode == ReferenceCallSourceMode::MutableReferenceIdentifier {
        let Some(name) = source_name else {
            unreachable!("shared mutable-reference identifier topology retained its identifier")
        };
        let Some(facts) = facts else {
            return Err(format!(
                "mutable reference call argument `{name}` is not an initialized local binding"
            ));
        };
        if !facts.initialized {
            return Err(format!("Error: Use of uninitialized variable `{name}`."));
        }
        if !facts.local {
            return Err(format!(
                "mutable reference call argument `{name}` is not an initialized local binding"
            ));
        }
        let Ty::Reference(actual_pointee, true) = &facts.ty else {
            return Err(
                "mutable reference call requires a mutable-reference identifier or direct `&mut` local owner argument".to_string(),
            );
        };
        let reference = match classify_reference_pointee_type(
            actual_pointee,
            ReferencePointeeContext::Mutable,
            registry,
            enums,
        ) {
            Ok(contract) => LocalReferenceContract {
                pointee: contract.ty,
                logical_pointee: contract.logical_type,
                mutable: true,
            },
            Err(message) => {
                return Err(message);
            }
        };
        match facts.ownership {
            OwnershipState::Owned => {}
            OwnershipState::Moved => {
                return Err(format!("cannot reborrow moved mutable reference `{name}`"));
            }
            OwnershipState::MaybeMoved => {
                return Err(crate::ownership_flow::maybe_moved_diagnostic(name));
            }
            OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
                return Err(format!(
                    "mutable reference call argument `{name}` has an invalid ownership state"
                ));
            }
        }
        return if reference.pointee == *expected_pointee {
            Ok(ReferenceCallArgumentContract {
                reference,
                source_mode,
                reference_parameter_index,
                projected: None,
            })
        } else {
            Err(format!(
                "mutable reference call pointee mismatch: expected {expected_pointee}, actual {}",
                reference.pointee
            ))
        };
    }

    match classify_local_borrow_with_enums(source, true, facts, registry, enums) {
        LocalReferenceDisposition::Supported(reference)
            if reference.pointee == *expected_pointee =>
        {
            Ok(ReferenceCallArgumentContract {
                reference,
                source_mode,
                reference_parameter_index,
                projected: None,
            })
        }
        LocalReferenceDisposition::Supported(contract) => Err(format!(
            "mutable reference call pointee mismatch: expected {expected_pointee}, actual {}",
            contract.pointee
        )),
        LocalReferenceDisposition::ExplicitlyRejected(message) => Err(message),
        LocalReferenceDisposition::Preserved => unreachable!(
            "direct mutable call borrow is fully classified by the local reference contract"
        ),
    }
}

fn reference_call_source_topology_at<'a>(
    contract: &ReferenceFunctionContract,
    arguments: &'a [Expression],
    reference_parameter_index: usize,
) -> Option<(ReferenceCallSourceMode, &'a Expression)> {
    if !contract
        .mutable_parameters()
        .iter()
        .any(|(index, _)| *index == reference_parameter_index)
    {
        return None;
    }
    let argument = arguments.get(reference_parameter_index)?;
    match argument {
        Expression::Borrow {
            expr,
            mutable: true,
        } => Some((
            if matches!(expr.as_ref(), Expression::Identifier(_)) {
                ReferenceCallSourceMode::DirectOwnerBorrow
            } else if matches!(
                expr.as_ref(),
                Expression::FieldAccess { .. }
                    | Expression::TupleIndex { .. }
                    | Expression::IndexAccess { .. }
            ) {
                ReferenceCallSourceMode::ProjectedOwnerBorrow
            } else {
                ReferenceCallSourceMode::DirectOwnerBorrow
            },
            expr.as_ref(),
        )),
        Expression::Identifier(_) => Some((
            ReferenceCallSourceMode::MutableReferenceIdentifier,
            argument,
        )),
        _ => None,
    }
}

pub(crate) fn reference_call_source_modes(
    contract: &ReferenceFunctionContract,
    arguments: &[Expression],
) -> Vec<(usize, ReferenceCallSourceMode)> {
    contract
        .mutable_parameters()
        .into_iter()
        .filter_map(|(index, _)| {
            reference_call_source_topology_at(contract, arguments, index)
                .map(|(mode, _)| (index, mode))
        })
        .collect()
}

fn expression_mentions_identifier(expression: &Expression, target: &str) -> bool {
    match expression {
        Expression::Identifier(name) => name == target,
        Expression::Binary { left, right, .. }
        | Expression::Comparison { left, right, .. }
        | Expression::Logical { left, right, .. } => {
            expression_mentions_identifier(left, target)
                || expression_mentions_identifier(right, target)
        }
        Expression::FunctionCall { arguments, .. }
        | Expression::Print { arguments, .. }
        | Expression::Println { arguments, .. }
        | Expression::TupleLiteral(arguments)
        | Expression::ArrayLiteral(arguments) => arguments
            .iter()
            .any(|argument| expression_mentions_identifier(argument, target)),
        Expression::MethodCall {
            object, arguments, ..
        } => {
            expression_mentions_identifier(object, target)
                || arguments
                    .iter()
                    .any(|argument| expression_mentions_identifier(argument, target))
        }
        Expression::Unary { operand, .. }
        | Expression::FieldAccess {
            object: operand, ..
        }
        | Expression::TupleIndex {
            object: operand, ..
        }
        | Expression::Borrow { expr: operand, .. }
        | Expression::Deref(operand)
        | Expression::Closure { body: operand, .. }
        | Expression::ArrayRepeat { value: operand, .. } => {
            expression_mentions_identifier(operand, target)
        }
        Expression::IndexAccess { object, index } => {
            expression_mentions_identifier(object, target)
                || expression_mentions_identifier(index, target)
        }
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expression_mentions_identifier(value, target)),
        Expression::EnumVariant { data, .. } => data.as_ref().is_some_and(|fields| {
            fields
                .iter()
                .any(|value| expression_mentions_identifier(value, target))
        }),
        Expression::Match { expr, arms } => {
            expression_mentions_identifier(expr, target)
                || arms
                    .iter()
                    .any(|arm| expression_mentions_identifier(&arm.body, target))
        }
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::CharacterLiteral(_)
        | Expression::StringLiteral(_) => false,
    }
}

#[cfg(test)]
pub(crate) fn classify_local_borrow(
    expression: &Expression,
    mutable: bool,
    facts: Option<&LocalReferenceSourceFacts>,
    registry: &StructRegistry,
) -> LocalReferenceDisposition {
    classify_local_borrow_with_enums(
        expression,
        mutable,
        facts,
        registry,
        &EnumRegistry::default(),
    )
}

pub(crate) fn classify_local_borrow_with_enums(
    expression: &Expression,
    mutable: bool,
    facts: Option<&LocalReferenceSourceFacts>,
    registry: &StructRegistry,
    enums: &EnumRegistry,
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
        ReferencePointeeContext::Mutable
    } else {
        ReferencePointeeContext::Immutable
    };
    let contract = match classify_reference_pointee_type(&facts.ty, context, registry, enums) {
        Ok(contract) => LocalReferenceContract {
            pointee: contract.ty,
            logical_pointee: contract.logical_type,
            mutable,
        },
        Err(message) => {
            return LocalReferenceDisposition::ExplicitlyRejected(message);
        }
    };
    if mutable && !facts.mutable {
        return LocalReferenceDisposition::ExplicitlyRejected(format!(
            "mutable borrow source `{name}` must be declared mutable"
        ));
    }
    let conflict = match (&facts.ownership, mutable) {
        (OwnershipState::Moved, _) => Some(format!("cannot borrow `{name}` because it was moved")),
        (OwnershipState::MaybeMoved, _) => {
            Some(crate::ownership_flow::maybe_moved_diagnostic(name))
        }
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

pub(crate) fn classify_enum_match_dereference(
    reference: &Expression,
    operand: &Ty,
    enums: &EnumRegistry,
) -> LocalReferenceDisposition {
    let Expression::Identifier(_) = reference else {
        return LocalReferenceDisposition::ExplicitlyRejected(
            "enum Match dereference requires an identifier reference".to_string(),
        );
    };
    let Ty::Reference(pointee, mutable) = operand else {
        return LocalReferenceDisposition::Preserved;
    };
    let enum_contract = match enums.reference_pointee_type(pointee) {
        Ok(Some(contract)) => contract,
        Ok(None) => return LocalReferenceDisposition::Preserved,
        Err(error) => {
            return LocalReferenceDisposition::ExplicitlyRejected(error.diagnostic());
        }
    };
    LocalReferenceDisposition::Supported(LocalReferenceContract {
        pointee: enum_contract.ty,
        logical_pointee: enum_contract.logical_type,
        mutable: *mutable,
    })
}

pub(crate) fn validate_enum_reference_match_result(
    scrutinee: &Expression,
    result: &Ty,
    structs: &StructRegistry,
) -> Result<(), String> {
    if !matches!(scrutinee, Expression::Deref(_)) || matches!(result, Ty::Void) {
        return Ok(());
    }
    match classify_copy_place_type(
        result,
        structs,
        CopyPlaceExecutionContext::AdmittedImmutableReference,
    ) {
        CopyPlaceDisposition::Supported(_) => Ok(()),
        CopyPlaceDisposition::ExplicitlyRejected(_) | CopyPlaceDisposition::Preserved => Err(
            "enum Match through a reference must produce admitted Copy-data or Void".to_string(),
        ),
    }
}

#[cfg(test)]
pub(crate) fn classify_local_reference_annotation(
    annotation: &Type,
    initialized: bool,
    registry: &StructRegistry,
) -> LocalReferenceDisposition {
    classify_local_reference_annotation_with_enums(
        annotation,
        initialized,
        registry,
        &EnumRegistry::default(),
    )
}

pub(crate) fn classify_local_reference_annotation_with_enums(
    annotation: &Type,
    initialized: bool,
    registry: &StructRegistry,
    enums: &EnumRegistry,
) -> LocalReferenceDisposition {
    let Type::Reference(inner, mutable) = annotation else {
        return LocalReferenceDisposition::Preserved;
    };
    if !initialized {
        return LocalReferenceDisposition::Preserved;
    }
    let context = if *mutable {
        ReferencePointeeContext::Mutable
    } else {
        ReferencePointeeContext::Immutable
    };
    match classify_reference_pointee_annotation(inner, context, registry, enums) {
        Ok(contract) => LocalReferenceDisposition::Supported(LocalReferenceContract {
            pointee: contract.ty,
            logical_pointee: contract.logical_type,
            mutable: *mutable,
        }),
        Err(message) => LocalReferenceDisposition::ExplicitlyRejected(message),
    }
}

#[cfg(test)]
pub(crate) fn classify_mutable_reference_assignment(
    target: &Expression,
    facts: Option<&MutableReferenceAssignmentFacts>,
    rhs: &Ty,
    inside_admitted_function: bool,
    registry: &StructRegistry,
) -> MutableReferenceAssignmentDisposition {
    classify_mutable_reference_assignment_with_enums(
        target,
        facts,
        rhs,
        inside_admitted_function,
        registry,
        &EnumRegistry::default(),
    )
}

pub(crate) fn classify_mutable_reference_assignment_with_enums(
    target: &Expression,
    facts: Option<&MutableReferenceAssignmentFacts>,
    rhs: &Ty,
    inside_admitted_function: bool,
    registry: &StructRegistry,
    enums: &EnumRegistry,
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
        OwnershipState::MaybeMoved => {
            return MutableReferenceAssignmentDisposition::ExplicitlyRejected(
                crate::ownership_flow::maybe_moved_diagnostic(name),
            );
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
            match classify_reference_pointee_type(
                pointee,
                ReferencePointeeContext::Mutable,
                registry,
                enums,
            ) {
                Ok(contract) => LocalReferenceContract {
                    pointee: contract.ty,
                    logical_pointee: contract.logical_type,
                    mutable: true,
                },
                Err(message) => {
                    return MutableReferenceAssignmentDisposition::ExplicitlyRejected(message);
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
    use crate::ast::{AstNode, Statement, VariantDecl, VariantDeclKind};

    fn enum_registry() -> EnumRegistry {
        EnumRegistry::from_top_level_ast(
            &[AstNode::Statement(Statement::EnumDef {
                name: "State".to_string(),
                variants: vec![
                    VariantDecl {
                        name: "Idle".to_string(),
                        kind: VariantDeclKind::Unit,
                    },
                    VariantDecl {
                        name: "Count".to_string(),
                        kind: VariantDeclKind::Tuple(vec![Type::Named("int".to_string())]),
                    },
                ],
                type_params: Vec::new(),
                trait_bounds: Vec::new(),
            })],
            &StructRegistry::default(),
        )
    }

    #[test]
    fn shared_pointee_classifier_admits_exact_enum_reference_transport() {
        let structs = StructRegistry::default();
        let enums = enum_registry();
        let expected_logical = LogicalType::Enum {
            name: "State".to_string(),
            variants: vec![
                crate::ir::EnumVariantSchema {
                    name: "Idle".to_string(),
                    payload: None,
                },
                crate::ir::EnumVariantSchema {
                    name: "Count".to_string(),
                    payload: Some(LogicalType::Int),
                },
            ],
        };

        for context in [
            ReferencePointeeContext::Immutable,
            ReferencePointeeContext::Mutable,
        ] {
            for contract in [
                classify_reference_pointee_annotation(
                    &Type::Named("State".to_string()),
                    context,
                    &structs,
                    &enums,
                )
                .expect("admitted enum annotation"),
                classify_reference_pointee_type(
                    &Ty::Enum("State".to_string()),
                    context,
                    &structs,
                    &enums,
                )
                .expect("admitted enum type"),
            ] {
                assert_eq!(contract.ty, Ty::Enum("State".to_string()));
                assert_eq!(contract.logical_type, expected_logical);
            }
        }

        for context in [
            ReferencePointeeContext::Immutable,
            ReferencePointeeContext::Mutable,
        ] {
            let scalar = classify_reference_pointee_type(&Ty::Int, context, &structs, &enums)
                .expect("CORE-052/055 scalar reference behavior remains admitted");
            assert_eq!(scalar.ty, Ty::Int);
            assert_eq!(scalar.logical_type, LogicalType::Int);
        }
    }

    #[test]
    fn classifier_partitions_supported_rejected_and_preserved_reference_shapes() {
        for reference_count in 0..=6 {
            for mutable_count in 0..=6 {
                assert_eq!(
                    admitted_reference_parameter_topology(reference_count, mutable_count),
                    mutable_count <= reference_count,
                    "topology drifted for {reference_count} references and {mutable_count} mutable references"
                );
            }
        }

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
            ReferenceCallDisposition::Supported(ReferenceCallContract { reference_arguments })
                if reference_arguments.len() == 1
                    && reference_arguments[0].source_mode
                        == ReferenceCallSourceMode::DirectOwnerBorrow
        ));
        assert_eq!(
            reference_call_source_modes(&function, &direct),
            vec![(0, ReferenceCallSourceMode::DirectOwnerBorrow)]
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
            ReferenceCallDisposition::Supported(ReferenceCallContract { reference_arguments })
                if reference_arguments.len() == 1
                    && reference_arguments[0].source_mode
                        == ReferenceCallSourceMode::MutableReferenceIdentifier
        ));
        assert_eq!(
            reference_call_source_modes(&function, &identifier),
            vec![(0, ReferenceCallSourceMode::MutableReferenceIdentifier)]
        );

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
