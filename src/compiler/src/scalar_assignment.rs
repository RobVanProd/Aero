use crate::ast::Expression;
use crate::types::{OwnershipState, Ty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScalarAssignmentContract {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, Clone)]
pub(crate) struct ScalarAssignmentTargetFacts {
    pub ty: Ty,
    pub mutable: bool,
    pub initialized: bool,
    pub local: bool,
    pub ownership: OwnershipState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ScalarAssignmentDisposition {
    Supported(ScalarAssignmentContract),
    ExplicitlyRejected(String),
    PreserveExistingBehavior,
}

pub(crate) fn classify_scalar_assignment(
    target: Option<&Expression>,
    facts: Option<&ScalarAssignmentTargetFacts>,
    rhs: &Ty,
    inside_admitted_function: bool,
) -> ScalarAssignmentDisposition {
    let Some(target) = target else {
        return ScalarAssignmentDisposition::PreserveExistingBehavior;
    };

    if !inside_admitted_function {
        return ScalarAssignmentDisposition::ExplicitlyRejected(
            "scalar reassignment is supported only inside admitted function bodies".to_string(),
        );
    }

    let Expression::Identifier(name) = target else {
        return ScalarAssignmentDisposition::ExplicitlyRejected(
            "assignment target must be a local identifier".to_string(),
        );
    };

    let Some(facts) = facts else {
        return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` is not an initialized local binding"
        ));
    };

    if !facts.initialized {
        return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` must already be initialized"
        ));
    }

    if !matches!(facts.ty, Ty::Int | Ty::Float | Ty::Bool) {
        return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` supports only Int, Float, or Bool"
        ));
    }

    if !facts.local || !facts.mutable {
        return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment target `{name}` must be a mutable local scalar binding"
        ));
    }

    match facts.ownership {
        OwnershipState::Owned => {}
        OwnershipState::Moved => {
            return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to moved value `{name}`"
            ));
        }
        OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
            return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to `{name}` while it is borrowed"
            ));
        }
    }

    if facts.ty != *rhs {
        return ScalarAssignmentDisposition::ExplicitlyRejected(format!(
            "assignment to `{name}` type mismatch: expected {}, actual {rhs}",
            facts.ty
        ));
    }

    ScalarAssignmentDisposition::Supported(ScalarAssignmentContract {
        name: name.clone(),
        ty: facts.ty.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(ty: Ty) -> ScalarAssignmentTargetFacts {
        ScalarAssignmentTargetFacts {
            ty,
            mutable: true,
            initialized: true,
            local: true,
            ownership: OwnershipState::Owned,
        }
    }

    #[test]
    fn classifier_closes_the_whole_scalar_assignment_topology() {
        let target = Expression::Identifier("value".to_string());
        assert!(matches!(
            classify_scalar_assignment(None, None, &Ty::Int, true),
            ScalarAssignmentDisposition::PreserveExistingBehavior
        ));
        assert!(matches!(
            classify_scalar_assignment(Some(&target), Some(&facts(Ty::Int)), &Ty::Int, true),
            ScalarAssignmentDisposition::Supported(ScalarAssignmentContract { name, ty: Ty::Int })
                if name == "value"
        ));

        for ty in [Ty::Float, Ty::Bool] {
            assert!(matches!(
                classify_scalar_assignment(Some(&target), Some(&facts(ty.clone())), &ty, true),
                ScalarAssignmentDisposition::Supported(_)
            ));
        }

        let unsupported_target = Expression::Deref(Box::new(target.clone()));
        assert!(matches!(
            classify_scalar_assignment(
                Some(&unsupported_target),
                Some(&facts(Ty::Int)),
                &Ty::Int,
                true
            ),
            ScalarAssignmentDisposition::ExplicitlyRejected(message)
                if message == "assignment target must be a local identifier"
        ));

        let cases = [
            (
                ScalarAssignmentTargetFacts {
                    mutable: false,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "must be a mutable local scalar binding",
            ),
            (
                ScalarAssignmentTargetFacts {
                    initialized: false,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "must already be initialized",
            ),
            (
                ScalarAssignmentTargetFacts {
                    ownership: OwnershipState::ImmutablyBorrowed(1),
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "while it is borrowed",
            ),
            (
                ScalarAssignmentTargetFacts {
                    ownership: OwnershipState::Moved,
                    ..facts(Ty::Int)
                },
                Ty::Int,
                "moved value",
            ),
            (
                facts(Ty::String),
                Ty::String,
                "supports only Int, Float, or Bool",
            ),
            (facts(Ty::Int), Ty::Float, "type mismatch"),
        ];
        for (target_facts, rhs, expected) in cases {
            assert!(matches!(
                classify_scalar_assignment(Some(&target), Some(&target_facts), &rhs, true),
                ScalarAssignmentDisposition::ExplicitlyRejected(message)
                    if message.contains(expected)
            ));
        }
    }
}
