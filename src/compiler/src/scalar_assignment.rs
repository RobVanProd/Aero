use crate::ast::Expression;
use crate::copy_place_contract::{
    CopyPlaceDisposition, CopyPlaceExecutionContext, classify_copy_place_type,
};
use crate::enum_match_contract::EnumRegistry;
use crate::ir::LogicalType;
use crate::struct_contract::StructRegistry;
use crate::types::{OwnershipState, Ty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OwnedPlaceAssignmentContract {
    pub name: String,
    pub ty: Ty,
    pub logical_type: LogicalType,
    pub moved_source: Option<String>,
    pub transition: OwnedPlaceAssignmentTransition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnedPlaceAssignmentTransition {
    Replacement,
    ReinitializeMoved,
    ReinitializeMaybeMoved,
}

impl OwnedPlaceAssignmentTransition {
    pub(crate) fn resulting_ownership(&self) -> OwnershipState {
        OwnershipState::Owned
    }
}

#[derive(Debug, Clone)]
pub(crate) struct OwnedPlaceAssignmentTargetFacts {
    pub ty: Ty,
    pub mutable: bool,
    pub initialized: bool,
    pub local: bool,
    pub ownership: OwnershipState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnedPlaceAssignmentDisposition {
    Supported(OwnedPlaceAssignmentContract),
    ExplicitlyRejected(String),
    PreserveExistingBehavior,
}

pub(crate) fn resolve_owned_place_logical_type(
    ty: &Ty,
    structs: &StructRegistry,
    enums: &EnumRegistry,
) -> Result<LogicalType, String> {
    if let Ty::Enum(enum_name) = ty {
        return enums
            .owned_place_logical_type(enum_name)
            .map_err(|error| error.diagnostic());
    }
    match classify_copy_place_type(
        ty,
        structs,
        CopyPlaceExecutionContext::AdmittedOwnedAssignment,
    ) {
        CopyPlaceDisposition::Supported(contract) => Ok(contract.logical_type),
        CopyPlaceDisposition::ExplicitlyRejected(message) => Err(message),
        CopyPlaceDisposition::Preserved => unreachable!("owned assignment context is admitted"),
    }
}

pub(crate) fn classify_owned_place_assignment(
    target: Option<&Expression>,
    facts: Option<&OwnedPlaceAssignmentTargetFacts>,
    rhs_expression: Option<&Expression>,
    rhs: &Ty,
    inside_admitted_function: bool,
    _inside_loop: bool,
    structs: &StructRegistry,
    enums: &EnumRegistry,
) -> OwnedPlaceAssignmentDisposition {
    let Some(target) = target else {
        return OwnedPlaceAssignmentDisposition::PreserveExistingBehavior;
    };

    if !inside_admitted_function {
        return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(
            "owned-place reassignment is supported only inside admitted function bodies"
                .to_string(),
        );
    }

    let Expression::Identifier(name) = target else {
        return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(
            "assignment target must be a local identifier".to_string(),
        );
    };

    let Some(facts) = facts else {
        return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` is not an initialized local binding"
        ));
    };

    if !facts.initialized {
        return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` must already be initialized"
        ));
    }

    let logical_type = match resolve_owned_place_logical_type(&facts.ty, structs, enums) {
        Ok(logical_type) => logical_type,
        Err(message) => {
            return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(message);
        }
    };

    if !facts.local || !facts.mutable {
        return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` must be a mutable local owned binding"
        ));
    }

    if facts.ty != *rhs {
        return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment to `{name}` type mismatch: expected {}, actual {rhs}",
            facts.ty
        ));
    }

    let transition = match facts.ownership {
        OwnershipState::Owned => OwnedPlaceAssignmentTransition::Replacement,
        OwnershipState::Moved if matches!(facts.ty, Ty::Enum(_)) => {
            OwnedPlaceAssignmentTransition::ReinitializeMoved
        }
        OwnershipState::MaybeMoved if matches!(facts.ty, Ty::Enum(_)) => {
            OwnedPlaceAssignmentTransition::ReinitializeMaybeMoved
        }
        OwnershipState::Moved => {
            return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to moved value `{name}`"
            ));
        }
        OwnershipState::MaybeMoved => {
            return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(
                crate::ownership_flow::maybe_moved_diagnostic(name),
            );
        }
        OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
            return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to `{name}` while it is borrowed"
            ));
        }
    };

    let moved_source = if matches!(facts.ty, Ty::Enum(_)) {
        match rhs_expression {
            Some(Expression::Identifier(source)) if source == name => {
                return OwnedPlaceAssignmentDisposition::ExplicitlyRejected(format!(
                    "direct enum self-replacement of `{name}` is not admitted"
                ));
            }
            Some(Expression::Identifier(source)) => Some(source.clone()),
            _ => None,
        }
    } else {
        None
    };

    OwnedPlaceAssignmentDisposition::Supported(OwnedPlaceAssignmentContract {
        name: name.clone(),
        ty: facts.ty.clone(),
        logical_type,
        moved_source,
        transition,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::tokenize, parser::parse};

    fn registries() -> (StructRegistry, EnumRegistry) {
        let ast = parse(tokenize(
            "struct Leaf { x: int, y: float } \
             struct Frame { leaf: Leaf, rows: [Leaf; 2], bias: int } \
             struct Envelope { frame: Frame } \
             struct Bad { text: String } \
             enum E { Empty, Value(Envelope) }",
        ));
        let structs = StructRegistry::from_top_level_ast(&ast);
        let enums = EnumRegistry::from_top_level_ast(&ast, &structs);
        (structs, enums)
    }

    fn facts(ty: Ty) -> OwnedPlaceAssignmentTargetFacts {
        OwnedPlaceAssignmentTargetFacts {
            ty,
            mutable: true,
            initialized: true,
            local: true,
            ownership: OwnershipState::Owned,
        }
    }

    #[test]
    fn classifier_closes_the_whole_owned_place_assignment_topology() {
        let (structs, enums) = registries();
        let target = Expression::Identifier("value".to_string());
        assert!(matches!(
            classify_owned_place_assignment(
                None,
                None,
                None,
                &Ty::Int,
                true,
                false,
                &structs,
                &enums,
            ),
            OwnedPlaceAssignmentDisposition::PreserveExistingBehavior
        ));

        for (ty, logical_type) in [
            (Ty::Int, LogicalType::Int),
            (Ty::Float, LogicalType::Float),
            (Ty::Bool, LogicalType::Bool),
            (
                Ty::Tuple(vec![Ty::Int, Ty::Float, Ty::Bool]),
                LogicalType::Tuple {
                    elements: vec![LogicalType::Int, LogicalType::Float, LogicalType::Bool],
                },
            ),
            (
                Ty::Array(Box::new(Ty::Int), 0),
                LogicalType::Array {
                    element: Box::new(LogicalType::Int),
                    count: 0,
                },
            ),
            (
                Ty::Array(Box::new(Ty::Struct("Leaf".to_string())), 2),
                LogicalType::Array {
                    element: Box::new(LogicalType::Struct {
                        name: "Leaf".to_string(),
                        fields: vec![LogicalType::Int, LogicalType::Float],
                    }),
                    count: 2,
                },
            ),
            (
                Ty::Struct("Envelope".to_string()),
                structs
                    .resolve_copy_type(&Ty::Struct("Envelope".to_string()))
                    .expect("Envelope is admitted Copy-data")
                    .logical_type,
            ),
            (
                Ty::Enum("E".to_string()),
                enums
                    .owned_place_logical_type("E")
                    .expect("E is an admitted owned enum"),
            ),
        ] {
            assert!(matches!(
                classify_owned_place_assignment(
                    Some(&target),
                    Some(&facts(ty.clone())),
                    None,
                    &ty,
                    true,
                    false,
                    &structs,
                    &enums,
                ),
                OwnedPlaceAssignmentDisposition::Supported(OwnedPlaceAssignmentContract {
                    name,
                    ty: actual,
                    logical_type: actual_logical,
                    moved_source: None,
                    transition: OwnedPlaceAssignmentTransition::Replacement,
                }) if name == "value" && actual == ty && actual_logical == logical_type
            ));
        }

        let source = Expression::Identifier("source".to_string());
        assert!(matches!(
            classify_owned_place_assignment(
                Some(&target),
                Some(&facts(Ty::Enum("E".to_string()))),
                Some(&source),
                &Ty::Enum("E".to_string()),
                true,
                false,
                &structs,
                &enums,
            ),
            OwnedPlaceAssignmentDisposition::Supported(OwnedPlaceAssignmentContract {
                moved_source: Some(name),
                ..
            }) if name == "source"
        ));
        assert!(matches!(
            classify_owned_place_assignment(
                Some(&target),
                Some(&facts(Ty::Enum("E".to_string()))),
                Some(&target),
                &Ty::Enum("E".to_string()),
                true,
                false,
                &structs,
                &enums,
            ),
            OwnedPlaceAssignmentDisposition::ExplicitlyRejected(message)
                if message.contains("self-replacement")
        ));

        let unsupported_target = Expression::Deref(Box::new(target.clone()));
        assert!(matches!(
            classify_owned_place_assignment(
                Some(&unsupported_target),
                Some(&facts(Ty::Int)),
                None,
                &Ty::Int,
                true,
                false,
                &structs,
                &enums,
            ),
            OwnedPlaceAssignmentDisposition::ExplicitlyRejected(message)
                if message == "assignment target must be a local identifier"
        ));

        let cases = [
            (
                OwnedPlaceAssignmentTargetFacts {
                    mutable: false,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "must be a mutable local owned binding",
            ),
            (
                OwnedPlaceAssignmentTargetFacts {
                    initialized: false,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "must already be initialized",
            ),
            (
                OwnedPlaceAssignmentTargetFacts {
                    ownership: OwnershipState::ImmutablyBorrowed(1),
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "while it is borrowed",
            ),
            (
                OwnedPlaceAssignmentTargetFacts {
                    ownership: OwnershipState::Moved,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "moved value",
            ),
            (
                facts(Ty::String),
                Ty::String,
                "not admitted Copy-data for owned assignment",
            ),
            (facts(Ty::Int), Ty::Float, "type mismatch"),
        ];
        for (target_facts, rhs, expected) in cases {
            assert!(matches!(
                classify_owned_place_assignment(
                    Some(&target),
                    Some(&target_facts),
                    None,
                    &rhs,
                    true,
                    false,
                    &structs,
                    &enums,
                ),
                OwnedPlaceAssignmentDisposition::ExplicitlyRejected(message)
                    if message.contains(expected)
            ));
        }

        for (ownership, expected) in [
            (
                OwnershipState::Moved,
                OwnedPlaceAssignmentTransition::ReinitializeMoved,
            ),
            (
                OwnershipState::MaybeMoved,
                OwnedPlaceAssignmentTransition::ReinitializeMaybeMoved,
            ),
        ] {
            let target_facts = OwnedPlaceAssignmentTargetFacts {
                ownership,
                ..facts(Ty::Enum("E".to_string()))
            };
            assert!(matches!(
                classify_owned_place_assignment(
                    Some(&target),
                    Some(&target_facts),
                    None,
                    &Ty::Enum("E".to_string()),
                    true,
                    false,
                    &structs,
                    &enums,
                ),
                OwnedPlaceAssignmentDisposition::Supported(OwnedPlaceAssignmentContract {
                    transition,
                    ..
                }) if transition == expected
            ));
            assert!(matches!(
                classify_owned_place_assignment(
                    Some(&target),
                    Some(&target_facts),
                    None,
                    &Ty::Enum("E".to_string()),
                    true,
                    true,
                    &structs,
                    &enums,
                ),
                OwnedPlaceAssignmentDisposition::Supported(OwnedPlaceAssignmentContract {
                    transition,
                    ..
                }) if transition == expected
            ));
        }
    }
}
