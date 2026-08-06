use crate::ast::{Block, Statement};
use crate::types::{OwnershipState, Ty};
use std::collections::BTreeSet;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoopOwnershipEdgeKind {
    Condition,
    Iterable,
    Fallthrough,
    Continue,
    Break,
}

impl LoopOwnershipEdgeKind {
    fn diagnostic_name(self) -> &'static str {
        match self {
            Self::Condition => "condition",
            Self::Iterable => "iterable",
            Self::Fallthrough => "fallthrough backedge",
            Self::Continue => "continue backedge",
            Self::Break => "break exit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoopOwnershipEdge {
    pub(crate) kind: LoopOwnershipEdgeKind,
    pub(crate) state: OwnershipState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoopControlSnapshots<S> {
    pub(crate) breaks: Vec<S>,
    pub(crate) continues: Vec<S>,
}

impl<S> Default for LoopControlSnapshots<S> {
    fn default() -> Self {
        Self {
            breaks: Vec::new(),
            continues: Vec::new(),
        }
    }
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

pub(crate) fn statement_reaches_merge(statement: &Statement, inside_loop: bool) -> bool {
    match statement {
        Statement::Return(_) => false,
        Statement::Break | Statement::Continue if inside_loop => false,
        Statement::Block(block) => block_reaches_merge(block, inside_loop),
        Statement::If {
            then_block,
            else_block: Some(else_statement),
            ..
        } => {
            block_reaches_merge(then_block, inside_loop)
                || statement_reaches_merge(else_statement, inside_loop)
        }
        _ => true,
    }
}

pub(crate) fn block_reaches_merge(block: &Block, inside_loop: bool) -> bool {
    block
        .statements
        .iter()
        .all(|statement| statement_reaches_merge(statement, inside_loop))
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

pub(crate) fn classify_owned_consumption_paths(
    name: &str,
    ty: &Ty,
    entry: &OwnershipState,
    paths: &[Vec<String>],
    inside_loop: bool,
) -> OwnershipFlowDisposition {
    if !matches!(ty, Ty::Enum(_)) {
        return OwnershipFlowDisposition::PreserveExistingBehavior;
    }
    let mut arms = Vec::with_capacity(paths.len());
    for path in paths {
        let mut seen = BTreeSet::new();
        for consumed in path {
            if !seen.insert(consumed.as_str()) {
                return OwnershipFlowDisposition::ExplicitlyRejected(format!(
                    "enum `{consumed}` is consumed more than once on one Match result path"
                ));
            }
        }
        let consumed = seen.contains(name);
        if consumed && !matches!(entry, OwnershipState::Owned) {
            return OwnershipFlowDisposition::ExplicitlyRejected(match entry {
                OwnershipState::Moved => {
                    format!("Error: Use of moved value `{name}`. Value was previously moved.")
                }
                OwnershipState::MaybeMoved => maybe_moved_diagnostic(name),
                OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
                    format!("enum owner `{name}` cannot move while it is borrowed")
                }
                OwnershipState::Owned => unreachable!("consumed owner was checked as non-owned"),
            });
        }
        arms.push(ConditionalOwnershipArm {
            state: if consumed {
                OwnershipState::Moved
            } else {
                entry.clone()
            },
            reaches_merge: true,
        });
    }
    classify_conditional_ownership(name, ty, entry, &arms, inside_loop)
}

pub(crate) fn classify_loop_ownership(
    name: &str,
    ty: &Ty,
    entry: &OwnershipState,
    edges: &[LoopOwnershipEdge],
) -> OwnershipFlowDisposition {
    if !matches!(ty, Ty::Enum(_)) {
        return OwnershipFlowDisposition::PreserveExistingBehavior;
    }
    if let Some(edge) = edges.iter().find(|edge| edge.state != *entry) {
        let edge_name = edge.kind.diagnostic_name();
        if !matches!(entry, OwnershipState::Owned) {
            return OwnershipFlowDisposition::ExplicitlyRejected(format!(
                "balanced loop ownership for enum owner `{name}` requires an Owned entry before changing state on the {edge_name}"
            ));
        }
        return OwnershipFlowDisposition::ExplicitlyRejected(format!(
            "balanced loop ownership for enum owner `{name}` is not restored to Owned on the {edge_name}"
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
                &[LoopOwnershipEdge {
                    kind: LoopOwnershipEdgeKind::Continue,
                    state: OwnershipState::Moved,
                }],
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

    #[test]
    fn owned_consumption_paths_distinguish_exclusive_partial_and_duplicate_moves() {
        let ty = Ty::Enum("E".to_string());
        assert_eq!(
            classify_owned_consumption_paths(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[vec!["value".to_string()], vec!["value".to_string()]],
                false,
            ),
            OwnershipFlowDisposition::Joined(Some(OwnershipState::Moved))
        );
        assert_eq!(
            classify_owned_consumption_paths(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[vec!["value".to_string()], Vec::new()],
                false,
            ),
            OwnershipFlowDisposition::Joined(Some(OwnershipState::MaybeMoved))
        );
        assert!(matches!(
            classify_owned_consumption_paths(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[vec!["value".to_string(), "value".to_string()]],
                false,
            ),
            OwnershipFlowDisposition::ExplicitlyRejected(message)
                if message.contains("one Match result path")
        ));
        assert!(matches!(
            classify_owned_consumption_paths(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[vec!["value".to_string()], Vec::new()],
                true,
            ),
            OwnershipFlowDisposition::ExplicitlyRejected(_)
        ));
    }

    #[test]
    fn balanced_loop_edges_are_complete_and_require_exact_entry_restoration() {
        let ty = Ty::Enum("E".to_string());
        let kinds = [
            LoopOwnershipEdgeKind::Condition,
            LoopOwnershipEdgeKind::Iterable,
            LoopOwnershipEdgeKind::Fallthrough,
            LoopOwnershipEdgeKind::Continue,
            LoopOwnershipEdgeKind::Break,
        ];
        let owned_edges = kinds
            .iter()
            .map(|kind| LoopOwnershipEdge {
                kind: *kind,
                state: OwnershipState::Owned,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            classify_loop_ownership("value", &ty, &OwnershipState::Owned, &owned_edges),
            OwnershipFlowDisposition::Joined(Some(OwnershipState::Owned))
        );

        for kind in kinds {
            let changed = [LoopOwnershipEdge {
                kind,
                state: OwnershipState::Moved,
            }];
            assert!(matches!(
                classify_loop_ownership("value", &ty, &OwnershipState::Owned, &changed),
                OwnershipFlowDisposition::ExplicitlyRejected(message)
                    if message.contains(kind.diagnostic_name())
                        && message.contains("not restored to Owned")
            ));
        }

        let unchanged_moved = [LoopOwnershipEdge {
            kind: LoopOwnershipEdgeKind::Fallthrough,
            state: OwnershipState::Moved,
        }];
        assert_eq!(
            classify_loop_ownership("value", &ty, &OwnershipState::Moved, &unchanged_moved,),
            OwnershipFlowDisposition::Joined(Some(OwnershipState::Moved))
        );
        let acquired = [LoopOwnershipEdge {
            kind: LoopOwnershipEdgeKind::Break,
            state: OwnershipState::Owned,
        }];
        assert!(matches!(
            classify_loop_ownership("value", &ty, &OwnershipState::Moved, &acquired),
            OwnershipFlowDisposition::ExplicitlyRejected(message)
                if message.contains("requires an Owned entry")
        ));
        assert_eq!(
            classify_loop_ownership("value", &Ty::Int, &OwnershipState::Owned, &unchanged_moved,),
            OwnershipFlowDisposition::PreserveExistingBehavior
        );
    }
}
