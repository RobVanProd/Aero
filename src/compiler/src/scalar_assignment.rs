use crate::ast::Expression;
use crate::copy_place_contract::{
    CopyPlaceDisposition, CopyPlaceExecutionContext, classify_copy_place_type,
};
use crate::ir::LogicalType;
use crate::struct_contract::StructRegistry;
use crate::types::{OwnershipState, Ty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyPlaceAssignmentContract {
    pub name: String,
    pub ty: Ty,
    pub logical_type: LogicalType,
}

#[derive(Debug, Clone)]
pub(crate) struct CopyPlaceAssignmentTargetFacts {
    pub ty: Ty,
    pub mutable: bool,
    pub initialized: bool,
    pub local: bool,
    pub ownership: OwnershipState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CopyPlaceAssignmentDisposition {
    Supported(CopyPlaceAssignmentContract),
    ExplicitlyRejected(String),
    PreserveExistingBehavior,
}

pub(crate) fn classify_copy_place_assignment(
    target: Option<&Expression>,
    facts: Option<&CopyPlaceAssignmentTargetFacts>,
    rhs: &Ty,
    inside_admitted_function: bool,
    registry: &StructRegistry,
) -> CopyPlaceAssignmentDisposition {
    let Some(target) = target else {
        return CopyPlaceAssignmentDisposition::PreserveExistingBehavior;
    };

    if !inside_admitted_function {
        return CopyPlaceAssignmentDisposition::ExplicitlyRejected(
            "Copy-place reassignment is supported only inside admitted function bodies".to_string(),
        );
    }

    let Expression::Identifier(name) = target else {
        return CopyPlaceAssignmentDisposition::ExplicitlyRejected(
            "assignment target must be a local identifier".to_string(),
        );
    };

    let Some(facts) = facts else {
        return CopyPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` is not an initialized local binding"
        ));
    };

    if !facts.initialized {
        return CopyPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` must already be initialized"
        ));
    }

    let copy_place = match classify_copy_place_type(
        &facts.ty,
        registry,
        CopyPlaceExecutionContext::AdmittedOwnedAssignment,
    ) {
        CopyPlaceDisposition::Supported(contract) => contract,
        CopyPlaceDisposition::ExplicitlyRejected(message) => {
            return CopyPlaceAssignmentDisposition::ExplicitlyRejected(message);
        }
        CopyPlaceDisposition::Preserved => unreachable!("owned assignment context is admitted"),
    };

    if !facts.local || !facts.mutable {
        return CopyPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` must be a mutable local Copy-data binding"
        ));
    }

    match facts.ownership {
        OwnershipState::Owned => {}
        OwnershipState::Moved => {
            return CopyPlaceAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to moved value `{name}`"
            ));
        }
        OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
            return CopyPlaceAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to `{name}` while it is borrowed"
            ));
        }
    }

    if facts.ty != *rhs {
        return CopyPlaceAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment to `{name}` type mismatch: expected {}, actual {rhs}",
            facts.ty
        ));
    }

    CopyPlaceAssignmentDisposition::Supported(CopyPlaceAssignmentContract {
        name: name.clone(),
        ty: facts.ty.clone(),
        logical_type: copy_place.logical_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{lexer::tokenize, parser::parse};

    fn registry() -> StructRegistry {
        let ast = parse(tokenize(
            "struct Leaf { x: int, y: float } \
             struct Frame { leaf: Leaf, rows: [Leaf; 2], bias: int } \
             struct Envelope { frame: Frame } \
             struct Bad { text: String }",
        ));
        StructRegistry::from_top_level_ast(&ast)
    }

    fn facts(ty: Ty) -> CopyPlaceAssignmentTargetFacts {
        CopyPlaceAssignmentTargetFacts {
            ty,
            mutable: true,
            initialized: true,
            local: true,
            ownership: OwnershipState::Owned,
        }
    }

    #[test]
    fn classifier_closes_the_whole_copy_place_assignment_topology() {
        let registry = registry();
        let target = Expression::Identifier("value".to_string());
        assert!(matches!(
            classify_copy_place_assignment(None, None, &Ty::Int, true, &registry),
            CopyPlaceAssignmentDisposition::PreserveExistingBehavior
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
                registry
                    .resolve_copy_type(&Ty::Struct("Envelope".to_string()))
                    .expect("Envelope is admitted Copy-data")
                    .logical_type,
            ),
        ] {
            assert!(matches!(
                classify_copy_place_assignment(
                    Some(&target),
                    Some(&facts(ty.clone())),
                    &ty,
                    true,
                    &registry,
                ),
                CopyPlaceAssignmentDisposition::Supported(CopyPlaceAssignmentContract {
                    name,
                    ty: actual,
                    logical_type: actual_logical,
                }) if name == "value" && actual == ty && actual_logical == logical_type
            ));
        }

        let unsupported_target = Expression::Deref(Box::new(target.clone()));
        assert!(matches!(
            classify_copy_place_assignment(
                Some(&unsupported_target),
                Some(&facts(Ty::Int)),
                &Ty::Int,
                true,
                &registry,
            ),
            CopyPlaceAssignmentDisposition::ExplicitlyRejected(message)
                if message == "assignment target must be a local identifier"
        ));

        let cases = [
            (
                CopyPlaceAssignmentTargetFacts {
                    mutable: false,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "must be a mutable local Copy-data binding",
            ),
            (
                CopyPlaceAssignmentTargetFacts {
                    initialized: false,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "must already be initialized",
            ),
            (
                CopyPlaceAssignmentTargetFacts {
                    ownership: OwnershipState::ImmutablyBorrowed(1),
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "while it is borrowed",
            ),
            (
                CopyPlaceAssignmentTargetFacts {
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
                classify_copy_place_assignment(
                    Some(&target),
                    Some(&target_facts),
                    &rhs,
                    true,
                    &registry,
                ),
                CopyPlaceAssignmentDisposition::ExplicitlyRejected(message)
                    if message.contains(expected)
            ));
        }
    }
}
