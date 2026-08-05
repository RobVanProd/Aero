#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StaticStringLenDisposition {
    StaticLength(i32),
    WrongArity { actual: usize },
    LengthOutsideIntRange { count: usize },
    PreserveExistingBehavior,
}

pub(crate) fn classify_static_string_len(
    receiver: Option<&str>,
    argument_count: usize,
) -> StaticStringLenDisposition {
    let Some(receiver) = receiver else {
        return StaticStringLenDisposition::PreserveExistingBehavior;
    };
    if argument_count != 0 {
        return StaticStringLenDisposition::WrongArity {
            actual: argument_count,
        };
    }
    classify_static_string_len_count(receiver.chars().count())
}

fn classify_static_string_len_count(count: usize) -> StaticStringLenDisposition {
    match i32::try_from(count) {
        Ok(count) => StaticStringLenDisposition::StaticLength(count),
        Err(_) => StaticStringLenDisposition::LengthOutsideIntRange { count },
    }
}

#[cfg(test)]
mod tests {
    use super::{
        StaticStringLenDisposition, classify_static_string_len, classify_static_string_len_count,
    };

    #[test]
    fn classifier_closes_literal_method_arity_and_count_product() {
        for (label, receiver, expected) in [
            ("empty", "", 0),
            ("ASCII", "Aero", 4),
            ("decoded escapes", "\n\t\r\\\"\0", 6),
            ("unknown escape text", "\\q", 2),
            ("Unicode scalars", "é🚀中", 3),
            ("combining scalars", "e\u{301}", 2),
            ("joined emoji scalars", "👩‍🚀", 3),
            ("mixed", "Aé🚀中z", 5),
        ] {
            assert_eq!(
                classify_static_string_len(Some(receiver), 0),
                StaticStringLenDisposition::StaticLength(expected),
                "{label}"
            );
        }

        for actual in [1, 2, usize::MAX] {
            assert_eq!(
                classify_static_string_len(Some("Aero"), actual),
                StaticStringLenDisposition::WrongArity { actual }
            );
        }

        assert_eq!(
            classify_static_string_len_count(i32::MAX as usize),
            StaticStringLenDisposition::StaticLength(i32::MAX)
        );
        let outside = i32::MAX as usize + 1;
        assert_eq!(
            classify_static_string_len_count(outside),
            StaticStringLenDisposition::LengthOutsideIntRange { count: outside }
        );

        for (label, receiver, arity) in [
            ("untrusted value", None, 0),
            ("untrusted wrong arity", None, 1),
        ] {
            assert_eq!(
                classify_static_string_len(receiver, arity),
                StaticStringLenDisposition::PreserveExistingBehavior,
                "{label}"
            );
        }
    }
}
