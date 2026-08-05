#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StaticStringPredicateKind {
    IsEmpty,
    Contains,
    StartsWith,
    EndsWith,
}

impl StaticStringPredicateKind {
    pub(crate) fn method(self) -> &'static str {
        match self {
            Self::IsEmpty => "is_empty",
            Self::Contains => "contains",
            Self::StartsWith => "starts_with",
            Self::EndsWith => "ends_with",
        }
    }

    fn expected_arguments(self) -> usize {
        match self {
            Self::IsEmpty => 0,
            Self::Contains | Self::StartsWith | Self::EndsWith => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StaticStringPredicateDisposition {
    StaticBool(bool),
    WrongArity {
        kind: StaticStringPredicateKind,
        expected: usize,
        actual: usize,
    },
    PreserveExistingBehavior,
}

pub(crate) fn classify_static_string_predicate(
    receiver: Option<&str>,
    kind: StaticStringPredicateKind,
    arguments: &[Option<&str>],
) -> StaticStringPredicateDisposition {
    let Some(receiver) = receiver else {
        return StaticStringPredicateDisposition::PreserveExistingBehavior;
    };
    let expected = kind.expected_arguments();
    if arguments.len() != expected {
        return StaticStringPredicateDisposition::WrongArity {
            kind,
            expected,
            actual: arguments.len(),
        };
    }

    let value = match kind {
        StaticStringPredicateKind::IsEmpty => receiver.is_empty(),
        StaticStringPredicateKind::Contains => {
            let Some(needle) = arguments[0] else {
                return StaticStringPredicateDisposition::PreserveExistingBehavior;
            };
            receiver.contains(needle)
        }
        StaticStringPredicateKind::StartsWith => {
            let Some(prefix) = arguments[0] else {
                return StaticStringPredicateDisposition::PreserveExistingBehavior;
            };
            receiver.starts_with(prefix)
        }
        StaticStringPredicateKind::EndsWith => {
            let Some(suffix) = arguments[0] else {
                return StaticStringPredicateDisposition::PreserveExistingBehavior;
            };
            receiver.ends_with(suffix)
        }
    };
    StaticStringPredicateDisposition::StaticBool(value)
}

#[cfg(test)]
mod tests {
    use super::{
        StaticStringPredicateDisposition, StaticStringPredicateKind,
        classify_static_string_predicate,
    };

    #[test]
    fn classifier_closes_arity_trust_and_content_product() {
        use StaticStringPredicateDisposition::{PreserveExistingBehavior, StaticBool, WrongArity};
        use StaticStringPredicateKind::{Contains, EndsWith, IsEmpty, StartsWith};

        for (receiver, expected) in [("", true), ("Aero", false), ("🚀", false)] {
            assert_eq!(
                classify_static_string_predicate(Some(receiver), IsEmpty, &[]),
                StaticBool(expected)
            );
        }

        for (kind, receiver, argument, expected) in [
            (Contains, "", "", true),
            (Contains, "", "a", false),
            (Contains, "abc", "", true),
            (Contains, "abcabc", "bca", true),
            (Contains, "abc", "abcd", false),
            (Contains, "é🚀中", "🚀", true),
            (Contains, "é", "e\u{301}", false),
            (Contains, "\n\t\r\\\"\0\\q", "\\q", true),
            (StartsWith, "", "", true),
            (StartsWith, "", "a", false),
            (StartsWith, "abc", "ab", true),
            (StartsWith, "abc", "bc", false),
            (StartsWith, "é🚀中", "é🚀", true),
            (EndsWith, "", "", true),
            (EndsWith, "", "a", false),
            (EndsWith, "abc", "bc", true),
            (EndsWith, "abc", "ab", false),
            (EndsWith, "é🚀中", "🚀中", true),
        ] {
            assert_eq!(
                classify_static_string_predicate(Some(receiver), kind, &[Some(argument)]),
                StaticBool(expected),
                "{}({argument:?}) on {receiver:?}",
                kind.method()
            );
        }

        for (kind, expected) in [(IsEmpty, 0), (Contains, 1), (StartsWith, 1), (EndsWith, 1)] {
            for actual in [0, 1, 2, 3] {
                if actual == expected {
                    continue;
                }
                let arguments = vec![Some("a"); actual];
                assert_eq!(
                    classify_static_string_predicate(Some("a"), kind, &arguments),
                    WrongArity {
                        kind,
                        expected,
                        actual,
                    }
                );
            }
        }

        for (kind, arguments) in [
            (IsEmpty, Vec::new()),
            (Contains, vec![Some("a")]),
            (StartsWith, vec![Some("a")]),
            (EndsWith, vec![Some("a")]),
        ] {
            assert_eq!(
                classify_static_string_predicate(None, kind, &arguments),
                PreserveExistingBehavior
            );
        }

        for kind in [Contains, StartsWith, EndsWith] {
            assert_eq!(
                classify_static_string_predicate(Some("a"), kind, &[None]),
                PreserveExistingBehavior
            );
        }
    }
}
