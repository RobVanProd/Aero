use crate::ast::ComparisonOp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StaticStringEqualityDisposition {
    StaticBool(bool),
    PreserveExistingBehavior,
}

pub(crate) fn classify_static_string_equality(
    left: Option<&str>,
    op: &ComparisonOp,
    right: Option<&str>,
) -> StaticStringEqualityDisposition {
    let (Some(left), Some(right)) = (left, right) else {
        return StaticStringEqualityDisposition::PreserveExistingBehavior;
    };

    match op {
        ComparisonOp::Equal => StaticStringEqualityDisposition::StaticBool(left == right),
        ComparisonOp::NotEqual => StaticStringEqualityDisposition::StaticBool(left != right),
        ComparisonOp::LessThan
        | ComparisonOp::GreaterThan
        | ComparisonOp::LessEqual
        | ComparisonOp::GreaterEqual => StaticStringEqualityDisposition::PreserveExistingBehavior,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifier_closes_exact_operator_presence_and_content_product() {
        use StaticStringEqualityDisposition::{PreserveExistingBehavior, StaticBool};

        for (left, right, equal) in [
            ("", "", true),
            ("", "a", false),
            ("Aero", "Aero", true),
            ("Aero", "aero", false),
            ("\n\t\r\\\"\0", "\n\t\r\\\"\0", true),
            ("\\q", "\\q", true),
            ("é🚀中", "é🚀中", true),
            ("é", "é", false),
            ("👩‍🚀", "👩‍🚀", true),
        ] {
            assert_eq!(
                classify_static_string_equality(Some(left), &ComparisonOp::Equal, Some(right)),
                StaticBool(equal)
            );
            assert_eq!(
                classify_static_string_equality(Some(left), &ComparisonOp::NotEqual, Some(right)),
                StaticBool(!equal)
            );
        }

        for op in [ComparisonOp::Equal, ComparisonOp::NotEqual] {
            assert_eq!(
                classify_static_string_equality(None, &op, Some("a")),
                PreserveExistingBehavior
            );
            assert_eq!(
                classify_static_string_equality(Some("a"), &op, None),
                PreserveExistingBehavior
            );
            assert_eq!(
                classify_static_string_equality(None, &op, None),
                PreserveExistingBehavior
            );
        }

        for op in [
            ComparisonOp::LessThan,
            ComparisonOp::GreaterThan,
            ComparisonOp::LessEqual,
            ComparisonOp::GreaterEqual,
        ] {
            assert_eq!(
                classify_static_string_equality(Some("a"), &op, Some("b")),
                PreserveExistingBehavior
            );
        }
    }
}
