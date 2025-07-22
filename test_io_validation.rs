// Test I/O validation and format string error reporting
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        SourceLocation { line, column }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Debug)]
pub struct ErrorSuggestion {
    pub message: String,
    pub replacement: Option<String>,
}

impl ErrorSuggestion {
    pub fn new(message: &str) -> Self {
        ErrorSuggestion {
            message: message.to_string(),
            replacement: None,
        }
    }

    pub fn with_replacement(message: &str, replacement: &str) -> Self {
        ErrorSuggestion {
            message: message.to_string(),
            replacement: Some(replacement.to_string()),
        }
    }
}

#[derive(Debug)]
pub enum CompilerError {
    FormatArgumentMismatch { 
        expected: usize, 
        actual: usize, 
        location: SourceLocation 
    },
    InvalidFormatString { 
        format: String, 
        location: SourceLocation 
    },
    InvalidFormatSpecifier { 
        specifier: String, 
        location: SourceLocation 
    },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::FormatArgumentMismatch { expected, actual, location } => {
                write!(f, "Error at {}: Format string expects {} arguments, got {}", location, expected, actual)
            }
            CompilerError::InvalidFormatString { format, location } => {
                write!(f, "Error at {}: Invalid format string '{}'", location, format)
            }
            CompilerError::InvalidFormatSpecifier { specifier, location } => {
                write!(f, "Error at {}: Invalid format specifier '{}'", location, specifier)
            }
        }
    }
}

#[derive(Debug)]
pub struct EnhancedError {
    pub error: CompilerError,
    pub suggestions: Vec<ErrorSuggestion>,
}

impl EnhancedError {
    pub fn new(error: CompilerError) -> Self {
        EnhancedError {
            error,
            suggestions: Vec::new(),
        }
    }

    pub fn with_suggestion(mut self, suggestion: ErrorSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    /// Create enhanced I/O error with format string suggestions
    pub fn format_error_with_suggestions(
        expected: usize,
        actual: usize,
        location: SourceLocation,
        format_string: &str
    ) -> Self {
        let error = CompilerError::FormatArgumentMismatch { expected, actual, location };
        let mut enhanced = EnhancedError::new(error);
        
        if actual < expected {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::new(&format!("Add {} more argument(s) to match the format string", expected - actual))
            );
        } else if actual > expected {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::new(&format!("Remove {} argument(s) or add more placeholders to the format string", actual - expected))
            );
        }
        
        // Count placeholders in format string
        let placeholder_count = format_string.matches("{}").count();
        if placeholder_count != expected {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::with_replacement(
                    &format!("Format string has {} placeholders but expects {}", placeholder_count, expected),
                    &format!("Check format string: \"{}\"", format_string)
                )
            );
        }
        
        enhanced
    }

    /// Create enhanced invalid format specifier error
    pub fn invalid_format_specifier_with_suggestions(
        specifier: &str,
        location: SourceLocation
    ) -> Self {
        let error = CompilerError::InvalidFormatSpecifier {
            specifier: specifier.to_string(),
            location,
        };
        let mut enhanced = EnhancedError::new(error);
        
        // Suggest common format specifiers
        match specifier {
            "{d}" | "{i}" => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement("Use '{}' for general formatting", "{}")
                );
            }
            "{s}" => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement("Use '{}' for string formatting", "{}")
                );
            }
            "{f}" => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement("Use '{}' for float formatting", "{}")
                );
            }
            _ if specifier.starts_with("{") && specifier.ends_with("}") => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement("Use simple '{}' placeholder", "{}")
                );
            }
            _ => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::new("Valid format specifiers: '{}' for general formatting")
                );
            }
        }
        
        enhanced
    }
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        
        for suggestion in &self.suggestions {
            write!(f, "\n  help: {}", suggestion.message)?;
            if let Some(replacement) = &suggestion.replacement {
                write!(f, " (try: {})", replacement)?;
            }
        }
        
        Ok(())
    }
}

fn main() {
    println!("Testing I/O Validation and Format String Error Reporting");
    println!("========================================================");
    
    test_format_argument_mismatch();
    test_invalid_format_specifier();
    test_format_string_validation();
    
    println!("\n✅ All I/O validation tests passed!");
}

fn test_format_argument_mismatch() {
    println!("\n🧪 Testing format argument mismatch errors...");
    
    let location = SourceLocation::new(10, 15);
    
    // Test too few arguments
    let enhanced = EnhancedError::format_error_with_suggestions(
        3, 1, location.clone(), "Hello {} {} {}"
    );
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Format string expects 3 arguments, got 1"));
    assert!(error_msg.contains("Add 2 more argument(s)"));
    
    // Test too many arguments
    let enhanced = EnhancedError::format_error_with_suggestions(
        1, 3, location, "Hello {}"
    );
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Format string expects 1 arguments, got 3"));
    assert!(error_msg.contains("Remove 2 argument(s)"));
    
    println!("✅ Format argument mismatch errors work");
}

fn test_invalid_format_specifier() {
    println!("\n🧪 Testing invalid format specifier errors...");
    
    let location = SourceLocation::new(5, 20);
    
    // Test {d} specifier
    let enhanced = EnhancedError::invalid_format_specifier_with_suggestions(
        "{d}", location.clone()
    );
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Invalid format specifier '{d}'"));
    assert!(error_msg.contains("Use '{}' for general formatting"));
    
    // Test {s} specifier
    let enhanced = EnhancedError::invalid_format_specifier_with_suggestions(
        "{s}", location.clone()
    );
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Use '{}' for string formatting"));
    
    // Test unknown specifier
    let enhanced = EnhancedError::invalid_format_specifier_with_suggestions(
        "{xyz}", location
    );
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Use simple '{}' placeholder"));
    
    println!("✅ Invalid format specifier errors work");
}

fn test_format_string_validation() {
    println!("\n🧪 Testing format string validation...");
    
    let location = SourceLocation::new(8, 12);
    let error = CompilerError::InvalidFormatString {
        format: "Hello {unclosed".to_string(),
        location,
    };
    
    let enhanced = EnhancedError::new(error)
        .with_suggestion(ErrorSuggestion::new("Format string has unclosed placeholder"))
        .with_suggestion(ErrorSuggestion::with_replacement(
            "Close the placeholder", 
            "Hello {}"
        ));
    
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Invalid format string 'Hello {unclosed'"));
    assert!(error_msg.contains("Format string has unclosed placeholder"));
    assert!(error_msg.contains("try: Hello {}"));
    
    println!("✅ Format string validation works");
}