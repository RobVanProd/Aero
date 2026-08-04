use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FixedNumericArrayLenDisposition {
    StaticLength(i32),
    WrongArity { actual: usize },
    LengthOutsideIntRange { count: usize },
    PreserveExistingBehavior,
}

pub(crate) fn classify_fixed_numeric_array_len(
    receiver: &Ty,
    method: &str,
    argument_count: usize,
) -> FixedNumericArrayLenDisposition {
    let Ty::Array(element, count) = receiver else {
        return FixedNumericArrayLenDisposition::PreserveExistingBehavior;
    };
    if method != "len" || !matches!(element.as_ref(), Ty::Int | Ty::Float) {
        return FixedNumericArrayLenDisposition::PreserveExistingBehavior;
    }
    if argument_count != 0 {
        return FixedNumericArrayLenDisposition::WrongArity {
            actual: argument_count,
        };
    }
    match i32::try_from(*count) {
        Ok(count) => FixedNumericArrayLenDisposition::StaticLength(count),
        Err(_) => FixedNumericArrayLenDisposition::LengthOutsideIntRange { count: *count },
    }
}

#[cfg(test)]
mod tests {
    use super::{FixedNumericArrayLenDisposition, classify_fixed_numeric_array_len};
    use crate::types::Ty;

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
                classify_fixed_numeric_array_len(&receiver, "len", 0),
                FixedNumericArrayLenDisposition::StaticLength(count),
                "{label}"
            );
        }

        for actual in [1, 2, usize::MAX] {
            assert_eq!(
                classify_fixed_numeric_array_len(&array(Ty::Int, 3), "len", actual),
                FixedNumericArrayLenDisposition::WrongArity { actual }
            );
        }

        let outside = i32::MAX as usize + 1;
        assert_eq!(
            classify_fixed_numeric_array_len(&array(Ty::Float, outside), "len", 0),
            FixedNumericArrayLenDisposition::LengthOutsideIntRange { count: outside }
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
                classify_fixed_numeric_array_len(&receiver, method, arity),
                FixedNumericArrayLenDisposition::PreserveExistingBehavior,
                "{label}"
            );
        }
    }
}
