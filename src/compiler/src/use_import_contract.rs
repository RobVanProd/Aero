use crate::ast::ImportSyntax;
use crate::errors::SourceLocation;

pub(crate) const UNSUPPORTED_USE_IMPORT_MESSAGE: &str = "use declarations are parsed but unsupported because name-resolution semantics are not implemented";
pub(crate) const UNSUPPORTED_FOUNDING_IMPORT_MESSAGE: &str = "import declarations are parsed but unsupported because name-resolution semantics are not implemented";

pub(crate) fn unsupported_name_import_diagnostic(
    syntax: ImportSyntax,
    location: &SourceLocation,
) -> String {
    let message = match syntax {
        ImportSyntax::RustLikeUse => UNSUPPORTED_USE_IMPORT_MESSAGE,
        ImportSyntax::FoundingDottedImport => UNSUPPORTED_FOUNDING_IMPORT_MESSAGE,
    };
    format!("Error: {message} at {location}.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_is_deterministic_with_and_without_a_filename() {
        assert_eq!(
            unsupported_name_import_diagnostic(
                ImportSyntax::RustLikeUse,
                &SourceLocation::new(3, 8),
            ),
            "Error: use declarations are parsed but unsupported because name-resolution semantics are not implemented at 3:8."
        );
        assert_eq!(
            unsupported_name_import_diagnostic(
                ImportSyntax::RustLikeUse,
                &SourceLocation::with_filename(5, 13, "module.aero".to_string()),
            ),
            "Error: use declarations are parsed but unsupported because name-resolution semantics are not implemented at module.aero:5:13."
        );
        assert_eq!(
            unsupported_name_import_diagnostic(
                ImportSyntax::FoundingDottedImport,
                &SourceLocation::with_filename(2, 4, "legacy.aero".to_string()),
            ),
            "Error: import declarations are parsed but unsupported because name-resolution semantics are not implemented at legacy.aero:2:4."
        );
    }
}
