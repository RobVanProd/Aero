#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StaticStringPredicateDisposition {
    StaticBool(bool),
    WrongArity {
        method: &'static str,
        expected: usize,
        actual: usize,
    },
    PreserveExistingBehavior,
}

pub(crate) fn classify_static_string_predicate(
    receiver: Option<&str>,
    method: &str,
    arguments: &[Option<&str>],
) -> StaticStringPredicateDisposition {
    let (method, expected) = match method {
        "is_empty" => ("is_empty", 0),
        "contains" => ("contains", 1),
        "starts_with" => ("starts_with", 1),
        "ends_with" => ("ends_with", 1),
        _ => return StaticStringPredicateDisposition::PreserveExistingBehavior,
    };
    let Some(receiver) = receiver else {
        return StaticStringPredicateDisposition::PreserveExistingBehavior;
    };
    if arguments.len() != expected {
        return StaticStringPredicateDisposition::WrongArity {
            method,
            expected,
            actual: arguments.len(),
        };
    }

    let value = match method {
        "is_empty" => receiver.is_empty(),
        "contains" => {
            let Some(needle) = arguments[0] else {
                return StaticStringPredicateDisposition::PreserveExistingBehavior;
            };
            receiver.contains(needle)
        }
        "starts_with" => {
            let Some(prefix) = arguments[0] else {
                return StaticStringPredicateDisposition::PreserveExistingBehavior;
            };
            receiver.starts_with(prefix)
        }
        "ends_with" => {
            let Some(suffix) = arguments[0] else {
                return StaticStringPredicateDisposition::PreserveExistingBehavior;
            };
            receiver.ends_with(suffix)
        }
        _ => unreachable!("method names were normalized above"),
    };
    StaticStringPredicateDisposition::StaticBool(value)
}

#[cfg(test)]
mod tests {
    use super::{StaticStringPredicateDisposition, classify_static_string_predicate};

    #[test]
    fn classifier_closes_method_arity_trust_and_content_product() {
        use StaticStringPredicateDisposition::{PreserveExistingBehavior, StaticBool, WrongArity};

        for (receiver, expected) in [("", true), ("Aero", false), ("🚀", false)] {
            assert_eq!(
                classify_static_string_predicate(Some(receiver), "is_empty", &[]),
                StaticBool(expected)
            );
        }

        for (method, receiver, argument, expected) in [
            ("contains", "", "", true),
            ("contains", "", "a", false),
            ("contains", "abc", "", true),
            ("contains", "abcabc", "bca", true),
            ("contains", "abc", "abcd", false),
            ("contains", "é🚀中", "🚀", true),
            ("contains", "é", "é", false),
            ("contains", "\n\t\r\\\"\0\\q", "\\q", true),
            ("starts_with", "", "", true),
            ("starts_with", "", "a", false),
            ("starts_with", "abc", "ab", true),
            ("starts_with", "abc", "bc", false),
            ("starts_with", "é🚀中", "é🚀", true),
            ("ends_with", "", "", true),
            ("ends_with", "", "a", false),
            ("ends_with", "abc", "bc", true),
            ("ends_with", "abc", "ab", false),
            ("ends_with", "é🚀中", "🚀中", true),
        ] {
            assert_eq!(
                classify_static_string_predicate(Some(receiver), method, &[Some(argument)]),
                StaticBool(expected),
                "{method}({argument:?}) on {receiver:?}"
            );
        }

        for (method, expected) in [
            ("is_empty", 0),
            ("contains", 1),
            ("starts_with", 1),
            ("ends_with", 1),
        ] {
            for actual in [0, 1, 2, 3] {
                if actual == expected {
                    continue;
                }
                let arguments = vec![Some("a"); actual];
                let disposition = classify_static_string_predicate(Some("a"), method, &arguments);
                assert_eq!(
                    disposition,
                    WrongArity {
                        method,
                        expected,
                        actual,
                    }
                );
            }
        }

        for (method, arguments) in [
            ("is_empty", Vec::new()),
            ("contains", vec![Some("a")]),
            ("starts_with", vec![Some("a")]),
            ("ends_with", vec![Some("a")]),
        ] {
            assert_eq!(
                classify_static_string_predicate(None, method, &arguments),
                PreserveExistingBehavior
            );
        }

        for (method, arguments) in [
            ("is_empty", vec![Some("a")]),
            ("contains", Vec::new()),
            ("starts_with", vec![Some("a"), Some("b")]),
            ("ends_with", vec![Some("a"), Some("b")]),
        ] {
            assert_eq!(
                classify_static_string_predicate(None, method, &arguments),
                PreserveExistingBehavior
            );
        }

        for method in ["contains", "starts_with", "ends_with"] {
            assert_eq!(
                classify_static_string_predicate(Some("a"), method, &[None]),
                PreserveExistingBehavior
            );
        }

        for method in ["Contains", "IS_EMPTY", "len", "trim", "missing"] {
            assert_eq!(
                classify_static_string_predicate(Some("a"), method, &[Some("a")]),
                PreserveExistingBehavior
            );
        }
    }
}
