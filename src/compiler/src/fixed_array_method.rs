use crate::struct_contract::StructRegistry;
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedArrayKind {
    Numeric,
    CopyStruct,
}

impl FixedArrayKind {
    pub(crate) fn diagnostic_subject(self) -> &'static str {
        match self {
            Self::Numeric => "fixed numeric array",
            Self::CopyStruct => "fixed Copy-struct array",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedArrayLenDisposition {
    StaticLength { kind: FixedArrayKind, count: i32 },
    WrongArity { kind: FixedArrayKind, actual: usize },
    LengthOutsideIntRange { kind: FixedArrayKind, count: usize },
    PreserveExistingBehavior,
}

pub(crate) fn classify_fixed_array_len(
    receiver: &Ty,
    method: &str,
    argument_count: usize,
    structs: &StructRegistry,
) -> FixedArrayLenDisposition {
    let Ty::Array(element, count) = receiver else {
        return FixedArrayLenDisposition::PreserveExistingBehavior;
    };
    if method != "len" {
        return FixedArrayLenDisposition::PreserveExistingBehavior;
    }
    let kind = if structs.is_copy_struct_ty(element) {
        FixedArrayKind::CopyStruct
    } else if matches!(element.as_ref(), Ty::Int | Ty::Float) {
        FixedArrayKind::Numeric
    } else {
        return FixedArrayLenDisposition::PreserveExistingBehavior;
    };
    if argument_count != 0 {
        return FixedArrayLenDisposition::WrongArity {
            kind,
            actual: argument_count,
        };
    }
    match i32::try_from(*count) {
        Ok(count) => FixedArrayLenDisposition::StaticLength { kind, count },
        Err(_) => FixedArrayLenDisposition::LengthOutsideIntRange {
            kind,
            count: *count,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedArrayKind, FixedArrayLenDisposition, classify_fixed_array_len};
    use crate::ast::{AstNode, FieldDecl, Statement, Type};
    use crate::struct_contract::StructRegistry;
    use crate::types::Ty;

    fn structs() -> StructRegistry {
        StructRegistry::from_top_level_ast(&[AstNode::Statement(Statement::StructDef {
            name: "Value".to_string(),
            fields: vec![FieldDecl {
                name: "number".to_string(),
                field_type: Type::Named("int".to_string()),
            }],
            type_params: vec![],
        })])
    }

    fn array(element: Ty, count: usize) -> Ty {
        Ty::Array(Box::new(element), count)
    }

    #[test]
    fn classifier_closes_the_normalized_type_method_arity_count_product() {
        for (label, receiver, count) in [
            ("Int zero", array(Ty::Int, 0), 0),
            ("Int positive", array(Ty::Int, 7), 7),
            ("Int maximum", array(Ty::Int, i32::MAX as usize), i32::MAX),
            ("Float zero", array(Ty::Float, 0), 0),
            ("Float positive", array(Ty::Float, 11), 11),
            (
                "Float maximum",
                array(Ty::Float, i32::MAX as usize),
                i32::MAX,
            ),
        ] {
            assert_eq!(
                classify_fixed_array_len(&receiver, "len", 0, &structs()),
                FixedArrayLenDisposition::StaticLength {
                    kind: FixedArrayKind::Numeric,
                    count,
                },
                "{label}"
            );
        }

        for actual in [1, 2, usize::MAX] {
            assert_eq!(
                classify_fixed_array_len(&array(Ty::Int, 3), "len", actual, &structs(),),
                FixedArrayLenDisposition::WrongArity {
                    kind: FixedArrayKind::Numeric,
                    actual,
                }
            );
        }

        for (label, receiver, count) in [
            (
                "Copy struct zero",
                array(Ty::Struct("Value".to_string()), 0),
                0,
            ),
            (
                "Copy struct positive",
                array(Ty::Struct("Value".to_string()), 9),
                9,
            ),
            (
                "Copy struct maximum",
                array(Ty::Struct("Value".to_string()), i32::MAX as usize),
                i32::MAX,
            ),
        ] {
            assert_eq!(
                classify_fixed_array_len(&receiver, "len", 0, &structs()),
                FixedArrayLenDisposition::StaticLength {
                    kind: FixedArrayKind::CopyStruct,
                    count,
                },
                "{label}"
            );
        }

        assert_eq!(
            classify_fixed_array_len(
                &array(Ty::Struct("Value".to_string()), 3),
                "len",
                2,
                &structs(),
            ),
            FixedArrayLenDisposition::WrongArity {
                kind: FixedArrayKind::CopyStruct,
                actual: 2,
            }
        );

        let outside = i32::MAX as usize + 1;
        assert_eq!(
            classify_fixed_array_len(&array(Ty::Float, outside), "len", 0, &structs(),),
            FixedArrayLenDisposition::LengthOutsideIntRange {
                kind: FixedArrayKind::Numeric,
                count: outside,
            }
        );
        assert_eq!(
            classify_fixed_array_len(
                &array(Ty::Struct("Value".to_string()), outside),
                "len",
                0,
                &structs(),
            ),
            FixedArrayLenDisposition::LengthOutsideIntRange {
                kind: FixedArrayKind::CopyStruct,
                count: outside,
            }
        );

        for (label, receiver, method, arity) in [
            ("scalar", Ty::Int, "len", 0),
            ("String", Ty::String, "len", 0),
            ("Vec", Ty::Vec(Box::new(Ty::Int)), "len", 0),
            ("Bool array", array(Ty::Bool, 3), "len", 0),
            ("nested array", array(array(Ty::Int, 1), 2), "len", 0),
            ("wrong method", array(Ty::Int, 3), "Len", 0),
            ("iter", array(Ty::Float, 3), "iter", 0),
            ("unknown arity", array(Ty::Int, 3), "missing", 2),
        ] {
            assert_eq!(
                classify_fixed_array_len(&receiver, method, arity, &structs()),
                FixedArrayLenDisposition::PreserveExistingBehavior,
                "{label}"
            );
        }
    }
}
