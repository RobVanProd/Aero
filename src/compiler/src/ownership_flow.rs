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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoopOwnershipKind {
    While,
    For,
    Loop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LoopOwnershipSummary {
    pub(crate) header: OwnershipState,
    pub(crate) exit: Option<OwnershipState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LoopOwnershipDisposition {
    FixedPoint(LoopOwnershipSummary),
    ExplicitlyRejected(String),
    PreserveExistingBehavior,
}

pub(crate) const LOOP_OWNERSHIP_FIXED_POINT_LIMIT: usize = 4;

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
    _entry: &OwnershipState,
    arms: &[ConditionalOwnershipArm],
    _inside_loop: bool,
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
    kind: LoopOwnershipKind,
    initial_header: &OwnershipState,
    edges: &[LoopOwnershipEdge],
) -> LoopOwnershipDisposition {
    if !matches!(ty, Ty::Enum(_)) {
        return LoopOwnershipDisposition::PreserveExistingBehavior;
    }

    let condition_count = edges
        .iter()
        .filter(|edge| edge.kind == LoopOwnershipEdgeKind::Condition)
        .count();
    let iterable_count = edges
        .iter()
        .filter(|edge| edge.kind == LoopOwnershipEdgeKind::Iterable)
        .count();
    let topology_is_valid = match kind {
        LoopOwnershipKind::While => condition_count == 1 && iterable_count == 0,
        LoopOwnershipKind::For => condition_count == 0 && iterable_count == 1,
        LoopOwnershipKind::Loop => condition_count == 0 && iterable_count == 0,
    };
    if !topology_is_valid {
        return LoopOwnershipDisposition::ExplicitlyRejected(format!(
            "enum owner `{name}` has an invalid loop ownership edge topology"
        ));
    }

    if matches!(
        initial_header,
        OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed
    ) {
        return LoopOwnershipDisposition::ExplicitlyRejected(format!(
            "enum owner `{name}` has an unsupported borrowed state at the loop header"
        ));
    }
    if let Some(edge) = edges.iter().find(|edge| {
        matches!(
            edge.state,
            OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed
        )
    }) {
        return LoopOwnershipDisposition::ExplicitlyRejected(format!(
            "enum owner `{name}` has an unsupported borrowed state on the {}",
            edge.kind.diagnostic_name()
        ));
    }

    let mut header_states = vec![initial_header.clone()];
    header_states.extend(
        edges
            .iter()
            .filter(|edge| {
                matches!(
                    edge.kind,
                    LoopOwnershipEdgeKind::Fallthrough | LoopOwnershipEdgeKind::Continue
                )
            })
            .map(|edge| edge.state.clone()),
    );
    let header = joined_state(&header_states)
        .expect("borrowed loop header states were rejected")
        .expect("the initial loop header state is always reachable");

    let mut exit_states = edges
        .iter()
        .filter(|edge| edge.kind == LoopOwnershipEdgeKind::Break)
        .map(|edge| edge.state.clone())
        .collect::<Vec<_>>();
    match kind {
        LoopOwnershipKind::While => exit_states.extend(
            edges
                .iter()
                .filter(|edge| edge.kind == LoopOwnershipEdgeKind::Condition)
                .map(|edge| edge.state.clone()),
        ),
        LoopOwnershipKind::For => exit_states.push(header.clone()),
        LoopOwnershipKind::Loop => {}
    }
    let exit = joined_state(&exit_states).expect("borrowed loop exit states were rejected");

    LoopOwnershipDisposition::FixedPoint(LoopOwnershipSummary { header, exit })
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
    fn exhaustively_joins_enum_fallthrough_states_inside_and_outside_loops() {
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
        assert_eq!(
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
            OwnershipFlowDisposition::Joined(Some(OwnershipState::MaybeMoved))
        );
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
        assert_eq!(
            classify_owned_consumption_paths(
                "value",
                &ty,
                &OwnershipState::Owned,
                &[vec!["value".to_string()], Vec::new()],
                true,
            ),
            OwnershipFlowDisposition::Joined(Some(OwnershipState::MaybeMoved))
        );
    }

    #[test]
    fn loop_fixed_point_classifier_covers_every_loop_and_edge_topology() {
        let ty = Ty::Enum("E".to_string());
        let while_edges = [
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Condition,
                state: OwnershipState::Owned,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Fallthrough,
                state: OwnershipState::Moved,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Continue,
                state: OwnershipState::Owned,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Break,
                state: OwnershipState::Moved,
            },
        ];
        assert_eq!(
            classify_loop_ownership(
                "value",
                &ty,
                LoopOwnershipKind::While,
                &OwnershipState::Owned,
                &while_edges,
            ),
            LoopOwnershipDisposition::FixedPoint(LoopOwnershipSummary {
                header: OwnershipState::MaybeMoved,
                exit: Some(OwnershipState::MaybeMoved),
            })
        );

        let for_edges = [
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Iterable,
                state: OwnershipState::Moved,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Fallthrough,
                state: OwnershipState::Owned,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Continue,
                state: OwnershipState::Moved,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Break,
                state: OwnershipState::Owned,
            },
        ];
        assert_eq!(
            classify_loop_ownership(
                "value",
                &ty,
                LoopOwnershipKind::For,
                &OwnershipState::Moved,
                &for_edges,
            ),
            LoopOwnershipDisposition::FixedPoint(LoopOwnershipSummary {
                header: OwnershipState::MaybeMoved,
                exit: Some(OwnershipState::MaybeMoved),
            })
        );

        let loop_edges = [
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Fallthrough,
                state: OwnershipState::Owned,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Continue,
                state: OwnershipState::Moved,
            },
            LoopOwnershipEdge {
                kind: LoopOwnershipEdgeKind::Break,
                state: OwnershipState::Owned,
            },
        ];
        assert_eq!(
            classify_loop_ownership(
                "value",
                &ty,
                LoopOwnershipKind::Loop,
                &OwnershipState::Moved,
                &loop_edges,
            ),
            LoopOwnershipDisposition::FixedPoint(LoopOwnershipSummary {
                header: OwnershipState::MaybeMoved,
                exit: Some(OwnershipState::Owned),
            })
        );
        assert_eq!(
            classify_loop_ownership(
                "value",
                &ty,
                LoopOwnershipKind::Loop,
                &OwnershipState::Moved,
                &[LoopOwnershipEdge {
                    kind: LoopOwnershipEdgeKind::Continue,
                    state: OwnershipState::Moved,
                }],
            ),
            LoopOwnershipDisposition::FixedPoint(LoopOwnershipSummary {
                header: OwnershipState::Moved,
                exit: None,
            })
        );

        for (kind, edges) in [
            (
                LoopOwnershipKind::While,
                vec![LoopOwnershipEdge {
                    kind: LoopOwnershipEdgeKind::Condition,
                    state: OwnershipState::MutablyBorrowed,
                }],
            ),
            (
                LoopOwnershipKind::For,
                vec![LoopOwnershipEdge {
                    kind: LoopOwnershipEdgeKind::Iterable,
                    state: OwnershipState::ImmutablyBorrowed(1),
                }],
            ),
            (
                LoopOwnershipKind::Loop,
                vec![LoopOwnershipEdge {
                    kind: LoopOwnershipEdgeKind::Break,
                    state: OwnershipState::MutablyBorrowed,
                }],
            ),
        ] {
            assert!(matches!(
                classify_loop_ownership(
                    "value",
                    &ty,
                    kind,
                    &OwnershipState::Owned,
                    &edges,
                ),
                LoopOwnershipDisposition::ExplicitlyRejected(message)
                    if message.contains("borrowed state")
            ));
        }

        assert!(matches!(
            classify_loop_ownership(
                "value",
                &ty,
                LoopOwnershipKind::While,
                &OwnershipState::Owned,
                &[],
            ),
            LoopOwnershipDisposition::ExplicitlyRejected(message)
                if message.contains("invalid loop ownership edge topology")
        ));
        assert_eq!(
            classify_loop_ownership(
                "value",
                &Ty::Int,
                LoopOwnershipKind::Loop,
                &OwnershipState::Owned,
                &loop_edges,
            ),
            LoopOwnershipDisposition::PreserveExistingBehavior
        );
    }
}
