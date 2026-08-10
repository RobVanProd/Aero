use crate::ast::Expression;
use crate::copy_place_contract::{
    CopyPlaceDisposition, CopyPlaceExecutionContext, classify_copy_place_type,
};
use crate::enum_match_contract::EnumRegistry;
use crate::ir::LogicalType;
use crate::struct_contract::{
    StructContract, StructExecutionContext, StructFieldContract, StructRegistry,
};
use crate::tuple_contract::{
    CopyTupleContract, TupleContractDisposition, TupleExecutionContext, classify_tuple_projection,
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CopyProjectionStep {
    StructField {
        receiver: StructContract,
        field_index: usize,
        field: StructFieldContract,
    },
    TupleElement {
        receiver: CopyTupleContract,
        index: usize,
        element: Ty,
    },
    ArrayElement {
        receiver: Ty,
        index: usize,
        element: Ty,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectedCopyDataAssignmentContract {
    pub(crate) root_name: String,
    pub(crate) root_type: Ty,
    pub(crate) leaf_type: Ty,
    pub(crate) leaf_logical_type: LogicalType,
    pub(crate) path: Vec<CopyProjectionStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectedCopyDataAssignmentDisposition {
    Supported(ProjectedCopyDataAssignmentContract),
    ExplicitlyRejected(String),
    PreserveExistingBehavior,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum UnresolvedProjectionStep {
    StructField(String),
    TupleElement(usize),
    ArrayElement(usize),
}

fn collect_static_projection_path<'a>(
    target: &'a Expression,
    path: &mut Vec<UnresolvedProjectionStep>,
) -> Result<&'a str, String> {
    match target {
        Expression::Identifier(name) => Ok(name),
        Expression::FieldAccess { object, field } => {
            let root = collect_static_projection_path(object, path)?;
            path.push(UnresolvedProjectionStep::StructField(field.clone()));
            Ok(root)
        }
        Expression::TupleIndex { object, index } => {
            let root = collect_static_projection_path(object, path)?;
            path.push(UnresolvedProjectionStep::TupleElement(*index));
            Ok(root)
        }
        Expression::IndexAccess { object, index } => {
            let Expression::IntegerLiteral(index) = index.as_ref() else {
                return Err(
                    "projected CopyData assignment array indexes require a compile-time integer literal"
                        .to_string(),
                );
            };
            let index = usize::try_from(*index).map_err(|_| {
                "projected CopyData assignment array indexes require a nonnegative compile-time integer literal"
                    .to_string()
            })?;
            let root = collect_static_projection_path(object, path)?;
            path.push(UnresolvedProjectionStep::ArrayElement(index));
            Ok(root)
        }
        _ => {
            Err("projected CopyData assignment requires a direct local identifier root".to_string())
        }
    }
}

pub(crate) fn classify_projected_copydata_assignment<F>(
    target: &Expression,
    rhs: &Ty,
    inside_admitted_function: bool,
    structs: &StructRegistry,
    mut facts_for_root: F,
) -> ProjectedCopyDataAssignmentDisposition
where
    F: FnMut(&str) -> Option<OwnedPlaceAssignmentTargetFacts>,
{
    if !matches!(
        target,
        Expression::FieldAccess { .. }
            | Expression::TupleIndex { .. }
            | Expression::IndexAccess { .. }
    ) {
        return ProjectedCopyDataAssignmentDisposition::PreserveExistingBehavior;
    }
    if !inside_admitted_function {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(
            "projected CopyData assignment is supported only inside admitted function bodies"
                .to_string(),
        );
    }

    let mut unresolved = Vec::new();
    let root_name = match collect_static_projection_path(target, &mut unresolved) {
        Ok(root) => root.to_string(),
        Err(message) => {
            return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(message);
        }
    };
    debug_assert!(!unresolved.is_empty());

    let Some(facts) = facts_for_root(&root_name) else {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
            "projected assignment root `{root_name}` is not an initialized local binding"
        ));
    };
    if !facts.initialized {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
            "projected assignment root `{root_name}` must already be initialized"
        ));
    }
    if !facts.local || !facts.mutable {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
            "projected assignment root `{root_name}` must be a mutable local owned binding"
        ));
    }
    match facts.ownership {
        OwnershipState::Owned => {}
        OwnershipState::Moved => {
            return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign through moved value `{root_name}`"
            ));
        }
        OwnershipState::MaybeMoved => {
            return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(
                crate::ownership_flow::maybe_moved_diagnostic(&root_name),
            );
        }
        OwnershipState::ImmutablyBorrowed(_) | OwnershipState::MutablyBorrowed => {
            return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
                "cannot assign to a projection of `{root_name}` while it is borrowed"
            ));
        }
    }
    if !matches!(
        classify_copy_place_type(
            &facts.ty,
            structs,
            CopyPlaceExecutionContext::AdmittedOwnedAssignment,
        ),
        CopyPlaceDisposition::Supported(_)
    ) {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
            "projected assignment root `{root_name}` is not admitted CopyData"
        ));
    }

    let mut current = facts.ty.clone();
    let mut path = Vec::with_capacity(unresolved.len());
    for step in unresolved {
        match step {
            UnresolvedProjectionStep::StructField(field_name) => {
                let (receiver, field_index, field) = match structs.resolve_field(
                    &current,
                    &field_name,
                    StructExecutionContext::AdmittedFunction,
                ) {
                    Ok(resolved) => resolved,
                    Err(error) => {
                        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(
                            error.diagnostic(),
                        );
                    }
                };
                current = field.ty();
                path.push(CopyProjectionStep::StructField {
                    receiver,
                    field_index,
                    field,
                });
            }
            UnresolvedProjectionStep::TupleElement(index) => {
                let projection = match classify_tuple_projection(
                    &current,
                    index,
                    structs,
                    TupleExecutionContext::AdmittedFunction,
                ) {
                    TupleContractDisposition::Supported(contract) => contract,
                    TupleContractDisposition::ExplicitlyRejected(message) => {
                        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(message);
                    }
                    TupleContractDisposition::Preserved => {
                        unreachable!("projected assignment uses admitted tuple context")
                    }
                };
                current = projection.element.clone();
                path.push(CopyProjectionStep::TupleElement {
                    receiver: projection.tuple,
                    index,
                    element: projection.element,
                });
            }
            UnresolvedProjectionStep::ArrayElement(index) => {
                let Ty::Array(element, count) = &current else {
                    return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
                        "projected assignment array selector requires an admitted fixed array, found {current}"
                    ));
                };
                if index >= *count {
                    return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
                        "projected CopyData array index {index} is outside 0..{count}"
                    ));
                }
                let receiver = current.clone();
                let element = element.as_ref().clone();
                current = element.clone();
                path.push(CopyProjectionStep::ArrayElement {
                    receiver,
                    index,
                    element,
                });
            }
        }
    }

    let Some(leaf) = structs.resolve_copy_type(&current) else {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
            "projected assignment leaf type {current} is not admitted CopyData"
        ));
    };
    if current != *rhs {
        return ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(format!(
            "projected assignment type mismatch: expected {current}, actual {rhs}"
        ));
    }

    ProjectedCopyDataAssignmentDisposition::Supported(ProjectedCopyDataAssignmentContract {
        root_name,
        root_type: facts.ty,
        leaf_type: current,
        leaf_logical_type: leaf.logical_type,
        path,
    })
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

    fn id(name: &str) -> Expression {
        Expression::Identifier(name.to_string())
    }

    fn field(object: Expression, name: &str) -> Expression {
        Expression::FieldAccess {
            object: Box::new(object),
            field: name.to_string(),
        }
    }

    fn tuple_element(object: Expression, index: usize) -> Expression {
        Expression::TupleIndex {
            object: Box::new(object),
            index,
        }
    }

    fn array_element(object: Expression, index: Expression) -> Expression {
        Expression::IndexAccess {
            object: Box::new(object),
            index: Box::new(index),
        }
    }

    #[test]
    fn projected_assignment_classifier_exhausts_static_copydata_paths_and_boundaries() {
        let (structs, _) = registries();
        let root_type = Ty::Tuple(vec![Ty::Struct("Frame".to_string()), Ty::Bool]);
        let supported_targets = [
            (
                field(id("root"), "bias"),
                Ty::Struct("Frame".to_string()),
                Ty::Int,
                1,
            ),
            (tuple_element(id("root"), 1), root_type.clone(), Ty::Bool, 1),
            (
                array_element(id("root"), Expression::IntegerLiteral(1)),
                Ty::Array(Box::new(Ty::Int), 2),
                Ty::Int,
                1,
            ),
            (
                field(
                    array_element(
                        field(tuple_element(id("root"), 0), "rows"),
                        Expression::IntegerLiteral(1),
                    ),
                    "x",
                ),
                root_type.clone(),
                Ty::Int,
                4,
            ),
        ];
        for (target, root_ty, leaf_ty, path_len) in supported_targets {
            assert!(matches!(
                classify_projected_copydata_assignment(
                    &target,
                    &leaf_ty,
                    true,
                    &structs,
                    |name| (name == "root").then(|| facts(root_ty.clone())),
                ),
                ProjectedCopyDataAssignmentDisposition::Supported(
                    ProjectedCopyDataAssignmentContract {
                        root_name,
                        root_type: actual_root,
                        leaf_type: actual_leaf,
                        path,
                        ..
                    }
                ) if root_name == "root"
                    && actual_root == root_ty
                    && actual_leaf == leaf_ty
                    && path.len() == path_len
            ));
        }

        let mixed = field(
            array_element(
                field(tuple_element(id("root"), 0), "rows"),
                Expression::IntegerLiteral(1),
            ),
            "x",
        );
        let ProjectedCopyDataAssignmentDisposition::Supported(contract) =
            classify_projected_copydata_assignment(&mixed, &Ty::Int, true, &structs, |name| {
                (name == "root").then(|| facts(root_type.clone()))
            })
        else {
            panic!("arbitrarily nested mixed projection must be admitted");
        };
        assert!(matches!(
            contract.path.as_slice(),
            [
                CopyProjectionStep::TupleElement { index: 0, .. },
                CopyProjectionStep::StructField { field_index: 1, .. },
                CopyProjectionStep::ArrayElement { index: 1, .. },
                CopyProjectionStep::StructField { field_index: 0, .. },
            ]
        ));
        assert_eq!(contract.leaf_logical_type, LogicalType::Int);

        assert!(matches!(
            classify_projected_copydata_assignment(&id("root"), &Ty::Int, true, &structs, |_| None),
            ProjectedCopyDataAssignmentDisposition::PreserveExistingBehavior
        ));

        let simple = field(id("root"), "bias");
        let root_is_frame =
            |name: &str| (name == "root").then(|| facts(Ty::Struct("Frame".to_string())));
        let rejection_cases = [
            (
                classify_projected_copydata_assignment(
                    &simple,
                    &Ty::Int,
                    false,
                    &structs,
                    root_is_frame,
                ),
                "only inside admitted function bodies",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| None),
                "not an initialized local binding",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| {
                    Some(OwnedPlaceAssignmentTargetFacts {
                        initialized: false,
                        ..facts(Ty::Struct("Frame".to_string()))
                    })
                }),
                "must already be initialized",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| {
                    Some(OwnedPlaceAssignmentTargetFacts {
                        mutable: false,
                        ..facts(Ty::Struct("Frame".to_string()))
                    })
                }),
                "must be a mutable local owned binding",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| {
                    Some(OwnedPlaceAssignmentTargetFacts {
                        local: false,
                        ..facts(Ty::Struct("Frame".to_string()))
                    })
                }),
                "must be a mutable local owned binding",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| {
                    Some(OwnedPlaceAssignmentTargetFacts {
                        ownership: OwnershipState::Moved,
                        ..facts(Ty::Struct("Frame".to_string()))
                    })
                }),
                "moved value",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| {
                    Some(OwnedPlaceAssignmentTargetFacts {
                        ownership: OwnershipState::MaybeMoved,
                        ..facts(Ty::Struct("Frame".to_string()))
                    })
                }),
                "may have been moved",
            ),
            (
                classify_projected_copydata_assignment(&simple, &Ty::Int, true, &structs, |_| {
                    Some(OwnedPlaceAssignmentTargetFacts {
                        ownership: OwnershipState::ImmutablyBorrowed(1),
                        ..facts(Ty::Struct("Frame".to_string()))
                    })
                }),
                "while it is borrowed",
            ),
            (
                classify_projected_copydata_assignment(
                    &field(id("root"), "missing"),
                    &Ty::Int,
                    true,
                    &structs,
                    root_is_frame,
                ),
                "has no field",
            ),
            (
                classify_projected_copydata_assignment(
                    &tuple_element(id("root"), 2),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| Some(facts(Ty::Tuple(vec![Ty::Int, Ty::Bool]))),
                ),
                "outside",
            ),
            (
                classify_projected_copydata_assignment(
                    &array_element(id("root"), Expression::IntegerLiteral(2)),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| Some(facts(Ty::Array(Box::new(Ty::Int), 2))),
                ),
                "outside 0..2",
            ),
            (
                classify_projected_copydata_assignment(
                    &array_element(id("root"), Expression::IntegerLiteral(0)),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| Some(facts(Ty::Array(Box::new(Ty::Int), 0))),
                ),
                "outside 0..0",
            ),
            (
                classify_projected_copydata_assignment(
                    &array_element(id("root"), Expression::IntegerLiteral(-1)),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| Some(facts(Ty::Array(Box::new(Ty::Int), 2))),
                ),
                "nonnegative",
            ),
            (
                classify_projected_copydata_assignment(
                    &array_element(id("root"), id("index")),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| Some(facts(Ty::Array(Box::new(Ty::Int), 2))),
                ),
                "compile-time integer literal",
            ),
            (
                classify_projected_copydata_assignment(
                    &field(
                        Expression::StructLiteral {
                            name: "Frame".to_string(),
                            fields: Vec::new(),
                        },
                        "bias",
                    ),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| None,
                ),
                "direct local identifier root",
            ),
            (
                classify_projected_copydata_assignment(
                    &field(id("root"), "bias"),
                    &Ty::Float,
                    true,
                    &structs,
                    root_is_frame,
                ),
                "type mismatch",
            ),
            (
                classify_projected_copydata_assignment(
                    &field(id("root"), "bias"),
                    &Ty::Int,
                    true,
                    &structs,
                    |_| Some(facts(Ty::String)),
                ),
                "not admitted CopyData",
            ),
        ];
        for (actual, expected) in rejection_cases {
            assert!(
                matches!(
                    actual,
                    ProjectedCopyDataAssignmentDisposition::ExplicitlyRejected(ref message)
                        if message.contains(expected)
                ),
                "expected rejection containing `{expected}`, got {actual:?}"
            );
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
