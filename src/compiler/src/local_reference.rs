use crate::ast::{Expression, Type};
use crate::types::Ty;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalReferenceContract {
    pub(crate) pointee: Ty,
}

impl LocalReferenceContract {
    pub(crate) fn reference_type(&self) -> Ty {
        Ty::Reference(Box::new(self.pointee.clone()), false)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocalReferenceRejectKind {
    Mutable,
    NonIdentifierPlace,
    UnsupportedPointee,
    NonReferenceDereference,
}

impl LocalReferenceRejectKind {
    pub(crate) fn diagnostic(self) -> &'static str {
        match self {
            Self::Mutable => "mutable references are not supported by CORE-048",
            Self::NonIdentifierPlace => {
                "a local immutable scalar borrow requires an identifier place"
            }
            Self::UnsupportedPointee => {
                "local immutable references support only Int, Float, or Bool pointees"
            }
            Self::NonReferenceDereference => "cannot dereference a non-reference value",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LocalReferenceDisposition {
    Supported(LocalReferenceContract),
    ExplicitlyRejected(LocalReferenceRejectKind),
    Preserved,
}

fn scalar_contract(ty: &Ty) -> Option<LocalReferenceContract> {
    matches!(ty, Ty::Int | Ty::Float | Ty::Bool).then(|| LocalReferenceContract {
        pointee: ty.clone(),
    })
}

pub(crate) fn classify_local_borrow(
    expression: &Expression,
    mutable: bool,
    pointee: &Ty,
) -> LocalReferenceDisposition {
    if mutable {
        return LocalReferenceDisposition::ExplicitlyRejected(LocalReferenceRejectKind::Mutable);
    }
    if !matches!(expression, Expression::Identifier(_)) {
        return LocalReferenceDisposition::ExplicitlyRejected(
            LocalReferenceRejectKind::NonIdentifierPlace,
        );
    }
    scalar_contract(pointee).map_or(
        LocalReferenceDisposition::ExplicitlyRejected(LocalReferenceRejectKind::UnsupportedPointee),
        LocalReferenceDisposition::Supported,
    )
}

pub(crate) fn classify_local_dereference(operand: &Ty) -> LocalReferenceDisposition {
    match operand {
        Ty::Reference(_, true) => {
            LocalReferenceDisposition::ExplicitlyRejected(LocalReferenceRejectKind::Mutable)
        }
        Ty::Reference(pointee, false) => scalar_contract(pointee).map_or(
            LocalReferenceDisposition::ExplicitlyRejected(
                LocalReferenceRejectKind::UnsupportedPointee,
            ),
            LocalReferenceDisposition::Supported,
        ),
        _ => LocalReferenceDisposition::ExplicitlyRejected(
            LocalReferenceRejectKind::NonReferenceDereference,
        ),
    }
}

pub(crate) fn classify_local_reference_annotation(
    annotation: &Type,
    initialized: bool,
) -> LocalReferenceDisposition {
    let Type::Reference(inner, mutable) = annotation else {
        return LocalReferenceDisposition::Preserved;
    };
    if !initialized {
        return LocalReferenceDisposition::Preserved;
    }
    if *mutable {
        return LocalReferenceDisposition::ExplicitlyRejected(LocalReferenceRejectKind::Mutable);
    }
    let pointee = match inner.as_ref() {
        Type::Named(name) if matches!(name.as_str(), "int" | "i32") => Ty::Int,
        Type::Named(name) if matches!(name.as_str(), "float" | "f64") => Ty::Float,
        Type::Named(name) if name == "bool" => Ty::Bool,
        _ => {
            return LocalReferenceDisposition::ExplicitlyRejected(
                LocalReferenceRejectKind::UnsupportedPointee,
            );
        }
    };
    LocalReferenceDisposition::Supported(LocalReferenceContract { pointee })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_partitions_supported_rejected_and_preserved_reference_shapes() {
        for pointee in [Ty::Int, Ty::Float, Ty::Bool] {
            let disposition = classify_local_borrow(
                &Expression::Identifier("value".to_string()),
                false,
                &pointee,
            );
            assert_eq!(
                disposition,
                LocalReferenceDisposition::Supported(LocalReferenceContract { pointee })
            );
        }
        assert_eq!(
            classify_local_borrow(&Expression::Identifier("value".to_string()), true, &Ty::Int),
            LocalReferenceDisposition::ExplicitlyRejected(LocalReferenceRejectKind::Mutable)
        );
        assert_eq!(
            classify_local_borrow(&Expression::IntegerLiteral(1), false, &Ty::Int),
            LocalReferenceDisposition::ExplicitlyRejected(
                LocalReferenceRejectKind::NonIdentifierPlace
            )
        );
        assert_eq!(
            classify_local_dereference(&Ty::Reference(Box::new(Ty::String), false)),
            LocalReferenceDisposition::ExplicitlyRejected(
                LocalReferenceRejectKind::UnsupportedPointee
            )
        );
        assert_eq!(
            classify_local_reference_annotation(
                &Type::Reference(Box::new(Type::Named("int".to_string())), false),
                false
            ),
            LocalReferenceDisposition::Preserved
        );
    }
}
