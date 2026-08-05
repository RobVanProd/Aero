use crate::ast::{Block, Statement};
use crate::types::{OwnershipState, Ty};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ConditionalOwnershipArm {
    pub(crate) state: OwnershipState,
    pub(crate) reaches_merge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum OwnershipFlowDisposition {
    Joined(Option<OwnershipState>),
    ExplicitlyRejected(String),
    PreserveExistingBehavior,
}

pub(crate) fn maybe_moved_diagnostic(name: &str) -> String {
    format!("enum owner `{name}` may have been moved on another control-flow path")
}

pub(crate) fn statement_definitely_returns(statement: &Statement) -> bool {
    match statement {
        Statement::Return(_) => true,
        Statement::Block(block) => block_definitely_returns(block),
        Statement::If {
            then_block,
            else_block: Some(else_statement),
            ..
        } => block_definitely_returns(then_block) && statement_definitely_returns(else_statement),
        _ => false,
    }
}

pub(crate) fn block_definitely_returns(block: &Block) -> bool {
    block.statements.iter().any(statement_definitely_returns)
}

fn joined_state(states: &[OwnershipState]) -> Result<Option<OwnershipState>, ()> {
    if states.is_empty() {
        return Ok(None);
    }
    if states.iter().any(|state| {
        matches!(
            state,
            OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed
        )
    }) {
        return Err(());
    }
    if states
        .iter()
        .all(|state| matches!(state, OwnershipState::Owned))
    {
        return Ok(Some(OwnershipState::Owned));
    }
    if states
        .iter()
        .all(|state| matches!(state, OwnershipState::Moved))
    {
        return Ok(Some(OwnershipState::Moved));
    }
    Ok(Some(OwnershipState::MaybeMoved))
}

pub(crate) fn classify_conditional_ownership(
    name: &str,
    ty: &Ty,
    entry: &OwnershipState,
    arms: &[ConditionalOwnershipArm],
    inside_loop: bool,
) -> OwnershipFlowDisposition {
    if !matches!(ty, Ty::Enum(_)) {
        return OwnershipFlowDisposition::PreserveExistingBehavior;
    }
    let reachable = arms
        .iter()
        .filter(|arm| arm.reaches_merge)
        .map(|arm| arm.state.clone())
        .collect::<Vec<_>>();
    let joined = match joined_state(&reachable) {
        Ok(joined) => joined,
        Err(()) => {
            return OwnershipFlowDisposition::ExplicitlyRejected(format!(
                "conditional enum owner `{name}` has an unsupported borrowed state at its control-flow join"
            ));
        }
    };
    if inside_loop && joined.as_ref().is_some_and(|state| state != entry) {
        return OwnershipFlowDisposition::ExplicitlyRejected(format!(
            "conditional ownership change for enum owner `{name}` inside a loop is not admitted; loop backedge ownership requires a fixed-point proof"
        ));
    }
    OwnershipFlowDisposition::Joined(joined)
}

pub(crate) fn classify_loop_ownership(
    name: &str,
    ty: &Ty,
    entry: &OwnershipState,
    backedge: &OwnershipState,
    reaches_backedge: bool,
) -> OwnershipFlowDisposition {
    if !matches!(ty, Ty::Enum(_)) {
        return OwnershipFlowDisposition::PreserveExistingBehavior;
    }
    if reaches_backedge && backedge != entry {
        return OwnershipFlowDisposition::ExplicitlyRejected(format!(
            "ownership change for enum owner `{name}` across a loop backedge is not admitted; loop ownership requires a fixed-point proof"
        ));
    }
    OwnershipFlowDisposition::Joined(Some(entry.clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arm(state: OwnershipState, reaches_merge: bool) -> ConditionalOwnershipArm {
        ConditionalOwnershipArm {
            state,
            reaches_merge,
        }
    }

    #[test]
    fn exhaustively_joins_enum_fallthrough_states_and_rejects_loop_changes() {
        let ty = Ty::Enum("E".to_string());
        for (left, right, expected) in [
            (
                OwnershipState::Owned,
                OwnershipState::Owned,
                OwnershipState::Owned,
            ),
            (
                OwnershipState::Moved,
                OwnershipState::Moved,
                OwnershipState::Moved,
            ),
            (
                OwnershipState::Owned,
                OwnershipState::Moved,
                OwnershipState::MaybeMoved,
            ),
            (
                OwnershipState::Moved,
                OwnershipState::Owned,
                OwnershipState::MaybeMoved,
            ),
            (
                OwnershipState::MaybeMoved,
                OwnershipState::MaybeMoved,
                OwnershipState::MaybeMoved,
            ),
        ] {
            assert_eq!(
                classify_conditional_ownership(
                    "value",
                    &ty,
                    &OwnershipState::Owned,
                    &[arm(left, true), arm(right, true)],
                    false,
                ),
                OwnershipFlowDisposition::Joined(Some(expected))
            );
        }
        assert_eq!(
            classify_conditional_ownership(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[
                    arm(OwnershipState::Moved, false),
                    arm(OwnershipState::Owned, true),
                ],
                false,
            ),
            OwnershipFlowDisposition::Joined(Some(OwnershipState::Owned))
        );
        assert!(matches!(
            classify_conditional_ownership(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[
                    arm(OwnershipState::Moved, true),
                    arm(OwnershipState::Owned, true),
                ],
                true,
            ),
            OwnershipFlowDisposition::ExplicitlyRejected(_)
        ));
        assert!(matches!(
            classify_loop_ownership(
                "value",
                &ty,
                &OwnershipState::Owned,
                &OwnershipState::Moved,
                true,
            ),
            OwnershipFlowDisposition::ExplicitlyRejected(_)
        ));
        assert_eq!(
            classify_conditional_ownership(
                "value",
                &Ty::Int,
                &OwnershipState::Owned,
                &[arm(OwnershipState::Moved, true)],
                false,
            ),
            OwnershipFlowDisposition::PreserveExistingBehavior
        );
    }
}
