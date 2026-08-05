use crate::ast::Type;
use crate::ir::LogicalType;
use crate::struct_contract::StructRegistry;
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CopyPlaceExecutionContext {
    AdmittedImmutableReference,
    AdmittedMutableReference,
    AdmittedOwnedAssignment,
    PreservedContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CopyPlaceContract {
    pub(crate) ty: Ty,
    pub(crate) logical_type: LogicalType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum CopyPlaceDisposition {
    Supported(CopyPlaceContract),
    ExplicitlyRejected(String),
    Preserved,
}

pub(crate) fn classify_copy_place_type(
    ty: &Ty,
    registry: &StructRegistry,
    context: CopyPlaceExecutionContext,
) -> CopyPlaceDisposition {
    let transport = match context {
        CopyPlaceExecutionContext::AdmittedImmutableReference => "immutable reference transport",
        CopyPlaceExecutionContext::AdmittedMutableReference => "mutable reference transport",
        CopyPlaceExecutionContext::AdmittedOwnedAssignment => "owned assignment",
        CopyPlaceExecutionContext::PreservedContext => return CopyPlaceDisposition::Preserved,
    };
    registry.resolve_copy_type(ty).map_or_else(
        || {
            CopyPlaceDisposition::ExplicitlyRejected(format!(
                "type {ty} is not admitted Copy-data for {transport}"
            ))
        },
        |contract| {
            CopyPlaceDisposition::Supported(CopyPlaceContract {
                ty: contract.ty,
                logical_type: contract.logical_type,
            })
        },
    )
}

pub(crate) fn classify_copy_place_annotation(
    annotation: &Type,
    registry: &StructRegistry,
    context: CopyPlaceExecutionContext,
) -> CopyPlaceDisposition {
    let transport = match context {
        CopyPlaceExecutionContext::AdmittedImmutableReference => "immutable reference transport",
        CopyPlaceExecutionContext::AdmittedMutableReference => "mutable reference transport",
        CopyPlaceExecutionContext::AdmittedOwnedAssignment => "owned assignment",
        CopyPlaceExecutionContext::PreservedContext => return CopyPlaceDisposition::Preserved,
    };
    registry.resolve_copy_annotation(annotation).map_or_else(
        || {
            CopyPlaceDisposition::ExplicitlyRejected(format!(
                "annotation is not admitted Copy-data for {transport}"
            ))
        },
        |contract| {
            CopyPlaceDisposition::Supported(CopyPlaceContract {
                ty: contract.ty,
                logical_type: contract.logical_type,
            })
        },
    )
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
             struct Bad { text: String } \
             struct Empty {}",
        ));
        StructRegistry::from_top_level_ast(&ast)
    }

    fn supported(ty: Ty, registry: &StructRegistry, context: CopyPlaceExecutionContext) -> bool {
        matches!(
            classify_copy_place_type(&ty, registry, context),
            CopyPlaceDisposition::Supported(_)
        )
    }

    #[test]
    fn classifier_is_exact_for_the_frozen_copy_place_universe() {
        let registry = registry();
        for context in [
            CopyPlaceExecutionContext::AdmittedImmutableReference,
            CopyPlaceExecutionContext::AdmittedMutableReference,
            CopyPlaceExecutionContext::AdmittedOwnedAssignment,
        ] {
            for ty in [Ty::Int, Ty::Float, Ty::Bool] {
                assert!(supported(ty, &registry, context));
            }
            for arity in 2..=6 {
                assert!(supported(
                    Ty::Tuple(vec![Ty::Int; arity]),
                    &registry,
                    context
                ));
            }
            for count in [0, 1, 3] {
                assert!(supported(
                    Ty::Array(Box::new(Ty::Int), count),
                    &registry,
                    context
                ));
                assert!(supported(
                    Ty::Array(Box::new(Ty::Float), count),
                    &registry,
                    context
                ));
                assert!(supported(
                    Ty::Array(Box::new(Ty::Struct("Leaf".to_string())), count),
                    &registry,
                    context
                ));
                assert!(supported(
                    Ty::Array(Box::new(Ty::Bool), count),
                    &registry,
                    context
                ));
                assert!(supported(
                    Ty::Array(Box::new(Ty::Array(Box::new(Ty::Int), 2)), count),
                    &registry,
                    context
                ));
                assert!(supported(
                    Ty::Array(Box::new(Ty::Tuple(vec![Ty::Int, Ty::Bool])), count),
                    &registry,
                    context
                ));
            }
            assert!(supported(
                Ty::Tuple(vec![
                    Ty::Array(Box::new(Ty::Bool), 2),
                    Ty::Struct("Leaf".to_string()),
                    Ty::Tuple(vec![Ty::Float, Ty::Int]),
                ]),
                &registry,
                context
            ));
            for name in ["Leaf", "Frame", "Envelope"] {
                assert!(supported(Ty::Struct(name.to_string()), &registry, context));
            }
        }

        let rejected = [
            Ty::String,
            Ty::Void,
            Ty::Enum("Mode".to_string()),
            Ty::Reference(Box::new(Ty::Int), false),
            Ty::TypeParam("T".to_string()),
            Ty::Option(Box::new(Ty::Int)),
            Ty::Result(Box::new(Ty::Int), Box::new(Ty::String)),
            Ty::Vec(Box::new(Ty::Int)),
            Ty::HashMap(Box::new(Ty::Int), Box::new(Ty::Int)),
            Ty::Fn("named".to_string()),
            Ty::Struct("Bad".to_string()),
            Ty::Struct("Empty".to_string()),
            Ty::Tuple(Vec::new()),
            Ty::Tuple(vec![Ty::Int]),
            Ty::Tuple(vec![Ty::Int, Ty::String]),
            Ty::Array(Box::new(Ty::String), 2),
        ];
        for context in [
            CopyPlaceExecutionContext::AdmittedImmutableReference,
            CopyPlaceExecutionContext::AdmittedMutableReference,
            CopyPlaceExecutionContext::AdmittedOwnedAssignment,
        ] {
            for ty in &rejected {
                assert!(
                    matches!(
                        classify_copy_place_type(ty, &registry, context),
                        CopyPlaceDisposition::ExplicitlyRejected(message)
                            if message.contains("admitted Copy-data")
                    ),
                    "unexpected classification for {ty} in {context:?}"
                );
            }
        }

        assert!(matches!(
            classify_copy_place_type(
                &Ty::Struct("Envelope".to_string()),
                &registry,
                CopyPlaceExecutionContext::PreservedContext,
            ),
            CopyPlaceDisposition::Preserved
        ));

        let annotations = [
            Type::Named("int".to_string()),
            Type::Named("float".to_string()),
            Type::Named("bool".to_string()),
            Type::Named("Envelope".to_string()),
            Type::Array(Box::new(Type::Named("int".to_string())), 3),
            Type::Array(Box::new(Type::Named("Leaf".to_string())), 2),
            Type::Array(Box::new(Type::Named("bool".to_string())), 2),
            Type::Array(
                Box::new(Type::Array(Box::new(Type::Named("int".to_string())), 2)),
                2,
            ),
            Type::Tuple(vec![
                Type::Array(Box::new(Type::Named("int".to_string())), 2),
                Type::Named("Leaf".to_string()),
            ]),
        ];
        for context in [
            CopyPlaceExecutionContext::AdmittedImmutableReference,
            CopyPlaceExecutionContext::AdmittedMutableReference,
            CopyPlaceExecutionContext::AdmittedOwnedAssignment,
        ] {
            for annotation in &annotations {
                assert!(matches!(
                    classify_copy_place_annotation(annotation, &registry, context),
                    CopyPlaceDisposition::Supported(_)
                ));
            }
        }
        let rejected_annotations = [
            Type::Named("String".to_string()),
            Type::Named("Bad".to_string()),
            Type::Array(Box::new(Type::Named("String".to_string())), 2),
            Type::Tuple(vec![Type::Named("int".to_string())]),
            Type::Reference(Box::new(Type::Named("int".to_string())), false),
        ];
        for context in [
            CopyPlaceExecutionContext::AdmittedImmutableReference,
            CopyPlaceExecutionContext::AdmittedMutableReference,
            CopyPlaceExecutionContext::AdmittedOwnedAssignment,
        ] {
            for annotation in &rejected_annotations {
                assert!(matches!(
                    classify_copy_place_annotation(annotation, &registry, context),
                    CopyPlaceDisposition::ExplicitlyRejected(_)
                ));
            }
        }
    }
}
