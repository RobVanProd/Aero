use crate::ast::{Expression, Parameter, Type};
use crate::ir::LogicalType;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceTransportTypeContract {
    pub(crate) ty: Ty,
    pub(crate) logical_type: LogicalType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceFunctionContract {
    pub(crate) name: String,
    pub(crate) parameters: Vec<(String, ReferenceTransportTypeContract)>,
    pub(crate) result: ReferenceTransportTypeContract,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ReferenceFunctionDisposition {
    Supported(ReferenceFunctionContract),
    ExplicitlyRejected(String),
    Preserved,
}

fn scalar_contract(ty: &Ty) -> Option<LocalReferenceContract> {
    matches!(ty, Ty::Int | Ty::Float | Ty::Bool).then(|| LocalReferenceContract {
        pointee: ty.clone(),
    })
}

fn scalar_transport_type(annotation: &Type) -> Option<ReferenceTransportTypeContract> {
    let (ty, logical_type) = match annotation {
        Type::Named(name) if matches!(name.as_str(), "int" | "i32") => (Ty::Int, LogicalType::Int),
        Type::Named(name) if matches!(name.as_str(), "float" | "f64") => {
            (Ty::Float, LogicalType::Float)
        }
        Type::Named(name) if name == "bool" => (Ty::Bool, LogicalType::Bool),
        _ => return None,
    };
    Some(ReferenceTransportTypeContract { ty, logical_type })
}

fn reference_transport_type(
    annotation: &Type,
) -> Result<Option<ReferenceTransportTypeContract>, &'static str> {
    let Type::Reference(pointee, mutable) = annotation else {
        return Ok(None);
    };
    if *mutable {
        return Err("mutable reference parameters are not supported by CORE-053");
    }
    let Some(pointee) = scalar_transport_type(pointee) else {
        return Err("immutable reference parameters support only Int, Float, or Bool pointees");
    };
    Ok(Some(ReferenceTransportTypeContract {
        ty: Ty::Reference(Box::new(pointee.ty), false),
        logical_type: LogicalType::ImmutableReference {
            pointee: Box::new(pointee.logical_type),
        },
    }))
}

pub(crate) fn classify_reference_function(
    name: &str,
    parameters: &[Parameter],
    return_type: Option<&Type>,
    type_params: &[String],
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

    let mut resolved_parameters = Vec::with_capacity(parameters.len());
    for parameter in parameters {
        let contract = match reference_transport_type(&parameter.param_type) {
            Ok(Some(contract)) => contract,
            Err(diagnostic) => {
                return ReferenceFunctionDisposition::ExplicitlyRejected(diagnostic.to_string());
            }
            Ok(None) => match scalar_transport_type(&parameter.param_type) {
                Some(contract) => contract,
                None => {
                    return ReferenceFunctionDisposition::ExplicitlyRejected(format!(
                        "reference transport function `{name}` parameter `{}` is not an admitted scalar or immutable scalar-reference type",
                        parameter.name
                    ));
                }
            },
        };
        resolved_parameters.push((parameter.name.clone(), contract));
    }

    let result = match return_type {
        Some(annotation) => match scalar_transport_type(annotation) {
            Some(contract) => contract,
            None => {
                return ReferenceFunctionDisposition::ExplicitlyRejected(format!(
                    "reference transport function `{name}` return type is not an admitted scalar or Void type"
                ));
            }
        },
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
                &[]
            ),
            ReferenceFunctionDisposition::ExplicitlyRejected(message)
                if message.contains("support only Int, Float, or Bool")
        ));
        assert_eq!(
            classify_reference_function(
                "plain",
                &[parameter("value", Type::Named("int".to_string()))],
                Some(&Type::Named("int".to_string())),
                &[]
            ),
            ReferenceFunctionDisposition::Preserved
        );
    }
}
