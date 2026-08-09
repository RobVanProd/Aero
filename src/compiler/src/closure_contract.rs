use crate::errors::SourceLocation;

pub(crate) const UNSUPPORTED_CLOSURE_MESSAGE: &str =
    "closure expressions are parsed but unsupported in executable code";

pub(crate) fn unsupported_closure_diagnostic(location: &SourceLocation) -> String {
    format!("Error: {UNSUPPORTED_CLOSURE_MESSAGE} at {location}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_is_deterministic_with_and_without_a_filename() {
        assert_eq!(
            unsupported_closure_diagnostic(&SourceLocation::new(3, 8)),
            "Error: closure expressions are parsed but unsupported in executable code at 3:8."
        );
        assert_eq!(
            unsupported_closure_diagnostic(&SourceLocation::with_filename(
                5,
                13,
                "module.aero".to_string(),
            )),
            "Error: closure expressions are parsed but unsupported in executable code at module.aero:5:13."
        );
    }
}
