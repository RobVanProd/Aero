use crate::struct_contract::StructRegistry;
use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedArrayKind {
    Numeric,
    CopyStruct,
    RecursiveCopyData,
}

impl FixedArrayKind {
    pub(crate) fn diagnostic_subject(self) -> &'static str {
        match self {
            Self::Numeric => "fixed numeric array",
            Self::CopyStruct => "fixed Copy-struct array",
            Self::RecursiveCopyData => "fixed recursive CopyData array",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedArrayQueryKind {
    Length,
    IsEmpty,
}

impl FixedArrayQueryKind {
    pub(crate) fn method(self) -> &'static str {
        match self {
            Self::Length => "len",
            Self::IsEmpty => "is_empty",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedArrayQueryValue {
    Length(i32),
    IsEmpty(bool),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedArrayQueryDisposition {
    StaticValue {
        kind: FixedArrayKind,
        value: FixedArrayQueryValue,
    },
    WrongArity {
        kind: FixedArrayKind,
        query: FixedArrayQueryKind,
        actual: usize,
    },
    CountOutsideIntRange {
        kind: FixedArrayKind,
        query: FixedArrayQueryKind,
        count: usize,
    },
    PreserveExistingBehavior,
}

pub(crate) fn classify_fixed_array_query(
    receiver: &Ty,
    query: FixedArrayQueryKind,
    argument_count: usize,
    structs: &StructRegistry,
) -> FixedArrayQueryDisposition {
    let Ty::Array(element, count) = receiver else {
        return FixedArrayQueryDisposition::PreserveExistingBehavior;
    };
    let kind = if matches!(element.as_ref(), Ty::Int | Ty::Float) {
        FixedArrayKind::Numeric
    } else if structs.is_copy_struct_ty(element) {
        FixedArrayKind::CopyStruct
    } else if structs.resolve_copy_type(element).is_some() {
        FixedArrayKind::RecursiveCopyData
    } else {
        return FixedArrayQueryDisposition::PreserveExistingBehavior;
    };
    if argument_count != 0 {
        return FixedArrayQueryDisposition::WrongArity {
            kind,
            query,
            actual: argument_count,
        };
    }
    let Ok(int_count) = i32::try_from(*count) else {
        return FixedArrayQueryDisposition::CountOutsideIntRange {
            kind,
            query,
            count: *count,
        };
    };
    let value = match query {
        FixedArrayQueryKind::Length => FixedArrayQueryValue::Length(int_count),
        FixedArrayQueryKind::IsEmpty => FixedArrayQueryValue::IsEmpty(*count == 0),
    };
    FixedArrayQueryDisposition::StaticValue { kind, value }
}

#[cfg(test)]
mod tests {
    use super::{
        FixedArrayKind, FixedArrayQueryDisposition, FixedArrayQueryKind, FixedArrayQueryValue,
        classify_fixed_array_query,
    };
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
    fn classifier_closes_recursive_copydata_query_arity_and_count_product() {
        let registry = structs();
        for (label, receiver, kind, count) in [
            ("Int", array(Ty::Int, 0), FixedArrayKind::Numeric, 0),
            ("Float", array(Ty::Float, 7), FixedArrayKind::Numeric, 7),
            (
                "Bool",
                array(Ty::Bool, 1),
                FixedArrayKind::RecursiveCopyData,
                1,
            ),
            (
                "tuple",
                array(Ty::Tuple(vec![Ty::Int, Ty::Bool]), 2),
                FixedArrayKind::RecursiveCopyData,
                2,
            ),
            (
                "nested array",
                array(array(Ty::Int, 2), 3),
                FixedArrayKind::RecursiveCopyData,
                3,
            ),
            (
                "Copy struct",
                array(Ty::Struct("Value".to_string()), 4),
                FixedArrayKind::CopyStruct,
                4,
            ),
        ] {
            assert_eq!(
                classify_fixed_array_query(&receiver, FixedArrayQueryKind::Length, 0, &registry,),
                FixedArrayQueryDisposition::StaticValue {
                    kind,
                    value: FixedArrayQueryValue::Length(count),
                },
                "{label} length"
            );
            assert_eq!(
                classify_fixed_array_query(&receiver, FixedArrayQueryKind::IsEmpty, 0, &registry,),
                FixedArrayQueryDisposition::StaticValue {
                    kind,
                    value: FixedArrayQueryValue::IsEmpty(count == 0),
                },
                "{label} emptiness"
            );
        }

        for query in [FixedArrayQueryKind::Length, FixedArrayQueryKind::IsEmpty] {
            for actual in [1, 2, usize::MAX] {
                assert_eq!(
                    classify_fixed_array_query(&array(Ty::Bool, 3), query, actual, &registry),
                    FixedArrayQueryDisposition::WrongArity {
                        kind: FixedArrayKind::RecursiveCopyData,
                        query,
                        actual,
                    }
                );
            }
            let outside = i32::MAX as usize + 1;
            assert_eq!(
                classify_fixed_array_query(&array(Ty::Int, outside), query, 0, &registry),
                FixedArrayQueryDisposition::CountOutsideIntRange {
                    kind: FixedArrayKind::Numeric,
                    query,
                    count: outside,
                }
            );
        }

        for receiver in [
            Ty::Int,
            Ty::String,
            Ty::Vec(Box::new(Ty::Int)),
            array(Ty::String, 2),
            array(Ty::Reference(Box::new(Ty::Int), false), 2),
            array(Ty::Fn("f".to_string()), 2),
        ] {
            assert_eq!(
                classify_fixed_array_query(&receiver, FixedArrayQueryKind::Length, 0, &registry,),
                FixedArrayQueryDisposition::PreserveExistingBehavior
            );
        }
    }
}
