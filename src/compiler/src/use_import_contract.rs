use crate::errors::SourceLocation;

pub(crate) const UNSUPPORTED_USE_IMPORT_MESSAGE: &str = "use declarations are parsed but unsupported because name-resolution semantics are not implemented";

pub(crate) fn unsupported_use_import_diagnostic(location: &SourceLocation) -> String {
    format!("Error: {UNSUPPORTED_USE_IMPORT_MESSAGE} at {location}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_is_deterministic_with_and_without_a_filename() {
        assert_eq!(
            unsupported_use_import_diagnostic(&SourceLocation::new(3, 8)),
            "Error: use declarations are parsed but unsupported because name-resolution semantics are not implemented at 3:8."
        );
        assert_eq!(
            unsupported_use_import_diagnostic(&SourceLocation::with_filename(
                5,
                13,
                "module.aero".to_string(),
            )),
            "Error: use declarations are parsed but unsupported because name-resolution semantics are not implemented at module.aero:5:13."
        );
    }
}
