// Standalone test for enhanced error reporting system
use std::fmt;

/// Represents a location in source code
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub filename: Option<String>,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize) -> Self {
        SourceLocation {
            line,
            column,
            filename: None,
        }
    }

    pub fn with_filename(line: usize, column: usize, filename: String) -> Self {
        SourceLocation {
            line,
            column,
            filename: Some(filename),
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.filename {
            Some(filename) => write!(f, "{}:{}:{}", filename, self.line, self.column),
            None => write!(f, "{}:{}", self.line, self.column),
        }
    }
}

/// Suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct ErrorSuggestion {
    pub message: String,
    pub replacement: Option<String>,
    pub location: Option<SourceLocation>,
}

impl ErrorSuggestion {
    pub fn new(message: &str) -> Self {
        ErrorSuggestion {
            message: message.to_string(),
            replacement: None,
            location: None,
        }
    }

    pub fn with_replacement(message: &str, replacement: &str) -> Self {
        ErrorSuggestion {
            message: message.to_string(),
            replacement: Some(replacement.to_string()),
            location: None,
        }
    }
}

/// Context information for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub function_name: Option<String>,
    pub current_scope_variables: Vec<String>,
    pub available_functions: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        ErrorContext {
            function_name: None,
            current_scope_variables: Vec::new(),
            available_functions: Vec::new(),
        }
    }

    pub fn in_function(mut self, name: String) -> Self {
        self.function_name = Some(name);
        self
    }

    pub fn with_variables(mut self, vars: Vec<String>) -> Self {
        self.current_scope_variables = vars;
        self
    }

    pub fn with_functions(mut self, funcs: Vec<String>) -> Self {
        self.available_functions = funcs;
        self
    }
}

/// Basic compiler error types
#[derive(Debug)]
pub enum CompilerError {
    UndefinedVariable { 
        name: String, 
        location: SourceLocation 
    },
    UndefinedFunction { 
        name: String, 
        location: SourceLocation 
    },
    TypeMismatch { 
        expected: String, 
        actual: String, 
        location: SourceLocation 
    },
    BreakOutsideLoop { 
        location: SourceLocation 
    },
    ImmutableAssignment { 
        name: String, 
        location: SourceLocation 
    },
    InvalidSyntax { 
        message: String, 
        location: SourceLocation 
    },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::UndefinedVariable { name, location } => {
                write!(f, "Error at {}: Undefined variable '{}'", location, name)
            }
            CompilerError::UndefinedFunction { name, location } => {
                write!(f, "Error at {}: Undefined function '{}'", location, name)
            }
            CompilerError::TypeMismatch { expected, actual, location } => {
                write!(f, "Error at {}: Type mismatch, expected {}, got {}", location, expected, actual)
            }
            CompilerError::BreakOutsideLoop { location } => {
                write!(f, "Error at {}: 'break' statement outside of loop", location)
            }
            CompilerError::ImmutableAssignment { name, location } => {
                write!(f, "Error at {}: Cannot assign to immutable variable '{}'", location, name)
            }
            CompilerError::InvalidSyntax { message, location } => {
                write!(f, "Syntax error at {}: {}", location, message)
            }
        }
    }
}

/// Enhanced error with suggestions and context
#[derive(Debug)]
pub struct EnhancedError {
    pub error: CompilerError,
    pub suggestions: Vec<ErrorSuggestion>,
    pub context: Option<ErrorContext>,
}

impl EnhancedError {
    pub fn new(error: CompilerError) -> Self {
        EnhancedError {
            error,
            suggestions: Vec::new(),
            context: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: ErrorSuggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display the main error
        write!(f, "{}", self.error)?;

        // Add context information
        if let Some(context) = &self.context {
            if let Some(func_name) = &context.function_name {
                write!(f, "\n  in function '{}'", func_name)?;
            }
            
            if !context.current_scope_variables.is_empty() {
                write!(f, "\n  available variables: {}", context.current_scope_variables.join(", "))?;
            }
            
            if !context.available_functions.is_empty() {
                write!(f, "\n  available functions: {}", context.available_functions.join(", "))?;
            }
        }

        // Add suggestions
        for suggestion in &self.suggestions {
            write!(f, "\n  help: {}", suggestion.message)?;
            if let Some(replacement) = &suggestion.replacement {
                write!(f, " (try: {})", replacement)?;
            }
        }

        Ok(())
    }
}

/// Find similar names using simple string distance
pub fn find_similar_names(target: &str, candidates: &[String]) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    for candidate in candidates {
        if levenshtein_distance(target, candidate) <= 2 && target.len() > 2 {
            suggestions.push(candidate.clone());
        }
    }
    
    // Sort by similarity (shorter distance first)
    suggestions.sort_by_key(|s| levenshtein_distance(target, s));
    suggestions.truncate(3); // Limit to 3 suggestions
    
    suggestions
}

/// Calculate Levenshtein distance between two strings
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    
    if len1 == 0 { return len2; }
    if len2 == 0 { return len1; }
    
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
    
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }
    
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i-1] == s2_chars[j-1] { 0 } else { 1 };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i-1][j] + 1,      // deletion
                    matrix[i][j-1] + 1       // insertion
                ),
                matrix[i-1][j-1] + cost      // substitution
            );
        }
    }
    
    matrix[len1][len2]
}

/// Enhanced error creation helpers
impl EnhancedError {
    /// Create an enhanced undefined variable error with suggestions
    pub fn undefined_variable_with_suggestions(
        name: &str, 
        location: SourceLocation, 
        available_vars: Vec<String>
    ) -> Self {
        let error = CompilerError::UndefinedVariable {
            name: name.to_string(),
            location,
        };
        let mut enhanced = EnhancedError::new(error);
        
        // Find similar variable names for suggestions
        let suggestions = find_similar_names(name, &available_vars);
        for suggestion in suggestions {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::with_replacement(
                    &format!("Did you mean '{}'?", suggestion),
                    &suggestion
                )
            );
        }
        
        if available_vars.is_empty() {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::new("No variables are currently in scope")
            );
        } else {
            enhanced = enhanced.with_context(
                ErrorContext::new().with_variables(available_vars)
            );
        }
        
        enhanced
    }

    /// Create an enhanced type mismatch error with conversion suggestions
    pub fn type_mismatch_with_suggestions(
        expected: &str, 
        actual: &str, 
        location: SourceLocation
    ) -> Self {
        let error = CompilerError::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
            location,
        };
        let mut enhanced = EnhancedError::new(error);
        
        // Add type conversion suggestions
        match (expected, actual) {
            ("i32", "f64") => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement(
                        "Convert to integer",
                        "value as i32"
                    )
                );
            }
            ("f64", "i32") => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement(
                        "Convert to float",
                        "value as f64"
                    )
                );
            }
            ("bool", _) => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::new("Use a comparison operator like ==, !=, <, >, <=, >=")
                );
            }
            ("String", _) => {
                enhanced = enhanced.with_suggestion(
                    ErrorSuggestion::with_replacement(
                        "Convert to string",
                        "value.to_string()"
                    )
                );
            }
            _ => {}
        }
        
        enhanced
    }
}

fn main() {
    println!("Testing Enhanced Error Reporting System");
    println!("========================================");
    
    test_basic_error_creation();
    test_enhanced_error_with_suggestions();
    test_undefined_variable_with_suggestions();
    test_type_mismatch_with_suggestions();
    test_levenshtein_distance();
    
    println!("\n✅ All enhanced error tests passed!");
}

fn test_basic_error_creation() {
    println!("\n🧪 Testing basic error creation...");
    
    let location = SourceLocation::new(10, 25);
    let error = CompilerError::UndefinedVariable {
        name: "x".to_string(),
        location,
    };
    let error_msg = format!("{}", error);
    
    assert!(error_msg.contains("10:25"));
    assert!(error_msg.contains("Undefined variable 'x'"));
    println!("✅ Basic error creation works");
}

fn test_enhanced_error_with_suggestions() {
    println!("\n🧪 Testing enhanced error with suggestions...");
    
    let location = SourceLocation::new(3, 5);
    let error = CompilerError::UndefinedVariable {
        name: "z".to_string(),
        location,
    };
    let enhanced = EnhancedError::new(error)
        .with_suggestion(ErrorSuggestion::new("Check variable name"))
        .with_context(ErrorContext::new().with_variables(vec!["x".to_string(), "y".to_string()]));

    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Undefined variable 'z'"));
    assert!(error_msg.contains("help: Check variable name"));
    assert!(error_msg.contains("available variables: x, y"));
    println!("✅ Enhanced error with suggestions works");
}

fn test_undefined_variable_with_suggestions() {
    println!("\n🧪 Testing undefined variable with suggestions...");
    
    let location = SourceLocation::new(5, 10);
    let available_vars = vec!["count".to_string(), "counter".to_string(), "value".to_string()];
    
    let enhanced = EnhancedError::undefined_variable_with_suggestions(
        "cout", 
        location, 
        available_vars
    );
    
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Undefined variable 'cout'"));
    assert!(error_msg.contains("Did you mean 'count'?"));
    assert!(error_msg.contains("available variables: count, counter, value"));
    println!("✅ Undefined variable with suggestions works");
}

fn test_type_mismatch_with_suggestions() {
    println!("\n🧪 Testing type mismatch with suggestions...");
    
    let location = SourceLocation::new(10, 20);
    
    // Test i32 to f64 conversion suggestion
    let enhanced = EnhancedError::type_mismatch_with_suggestions("i32", "f64", location.clone());
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Type mismatch, expected i32, got f64"));
    assert!(error_msg.contains("Convert to integer"));
    assert!(error_msg.contains("try: value as i32"));

    // Test bool type suggestion
    let enhanced = EnhancedError::type_mismatch_with_suggestions("bool", "i32", location.clone());
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Use a comparison operator"));

    // Test String conversion suggestion
    let enhanced = EnhancedError::type_mismatch_with_suggestions("String", "i32", location);
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Convert to string"));
    assert!(error_msg.contains("try: value.to_string()"));
    
    println!("✅ Type mismatch with suggestions works");
}

fn test_levenshtein_distance() {
    println!("\n🧪 Testing Levenshtein distance algorithm...");
    
    let candidates = vec![
        "count".to_string(),
        "counter".to_string(), 
        "value".to_string(),
        "variable".to_string()
    ];
    
    let suggestions = find_similar_names("cout", &candidates);
    assert!(suggestions.contains(&"count".to_string()));
    
    let suggestions = find_similar_names("countr", &candidates);
    assert!(suggestions.contains(&"counter".to_string()));
    
    // Test that very different names are not suggested
    let suggestions = find_similar_names("xyz", &candidates);
    assert!(suggestions.is_empty());
    
    println!("✅ Levenshtein distance algorithm works");
}