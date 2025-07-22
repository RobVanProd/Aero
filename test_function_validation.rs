// Test function validation and enhanced error reporting
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
pub struct ErrorContext {
    pub function_signature: Option<String>,
    pub available_functions: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        ErrorContext {
            function_signature: None,
            available_functions: Vec::new(),
        }
    }

    pub fn with_signature(mut self, signature: String) -> Self {
        self.function_signature = Some(signature);
        self
    }

    pub fn with_functions(mut self, funcs: Vec<String>) -> Self {
        self.available_functions = funcs;
        self
    }
}

#[derive(Debug)]
pub enum CompilerError {
    UndefinedFunction { 
        name: String, 
        location: SourceLocation 
    },
    ArityMismatch { 
        function_name: String,
        expected: usize, 
        actual: usize, 
        location: SourceLocation 
    },
    ParameterTypeMismatch { 
        function_name: String,
        parameter_name: String,
        expected: String, 
        actual: String, 
        location: SourceLocation 
    },
    ReturnTypeMismatch { 
        function_name: String,
        expected: String, 
        actual: String, 
        location: SourceLocation 
    },
    FunctionRedefinition { 
        name: String, 
        location: SourceLocation,
        previous_location: Option<SourceLocation>,
    },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::UndefinedFunction { name, location } => {
                write!(f, "Error at {}: Undefined function '{}'", location, name)
            }
            CompilerError::ArityMismatch { function_name, expected, actual, location } => {
                write!(f, "Error at {}: Function '{}' expects {} arguments, got {}", location, function_name, expected, actual)
            }
            CompilerError::ParameterTypeMismatch { function_name, parameter_name, expected, actual, location } => {
                write!(f, "Error at {}: Parameter '{}' of function '{}' expects type {}, got {}", location, parameter_name, function_name, expected, actual)
            }
            CompilerError::ReturnTypeMismatch { function_name, expected, actual, location } => {
                write!(f, "Error at {}: Function '{}' expects return type {}, got {}", location, function_name, expected, actual)
            }
            CompilerError::FunctionRedefinition { name, location, previous_location } => {
                match previous_location {
                    Some(prev) => write!(f, "Error at {}: Function '{}' redefined (previously defined at {})", location, name, prev),
                    None => write!(f, "Error at {}: Function '{}' redefined", location, name),
                }
            }
        }
    }
}

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

    /// Create enhanced undefined function error with suggestions
    pub fn undefined_function_with_suggestions(
        name: &str, 
        location: SourceLocation, 
        available_funcs: Vec<String>
    ) -> Self {
        let error = CompilerError::UndefinedFunction {
            name: name.to_string(),
            location,
        };
        let mut enhanced = EnhancedError::new(error);
        
        // Find similar function names for suggestions
        let suggestions = find_similar_names(name, &available_funcs);
        for suggestion in suggestions {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::with_replacement(
                    &format!("Did you mean '{}'?", suggestion),
                    &suggestion
                )
            );
        }
        
        enhanced = enhanced.with_context(
            ErrorContext::new().with_functions(available_funcs)
        );
        
        enhanced
    }

    /// Create enhanced arity mismatch error with function signature
    pub fn arity_mismatch_with_signature(
        function_name: &str,
        expected: usize,
        actual: usize,
        location: SourceLocation,
        signature: &str
    ) -> Self {
        let error = CompilerError::ArityMismatch {
            function_name: function_name.to_string(),
            expected,
            actual,
            location,
        };
        let mut enhanced = EnhancedError::new(error);
        
        enhanced = enhanced.with_context(
            ErrorContext::new().with_signature(signature.to_string())
        );
        
        if actual < expected {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::new(&format!("Add {} more argument(s)", expected - actual))
            );
        } else if actual > expected {
            enhanced = enhanced.with_suggestion(
                ErrorSuggestion::new(&format!("Remove {} argument(s)", actual - expected))
            );
        }
        
        enhanced
    }

    /// Create enhanced parameter type mismatch error
    pub fn parameter_type_mismatch_with_suggestions(
        function_name: &str,
        parameter_name: &str,
        expected: &str,
        actual: &str,
        location: SourceLocation
    ) -> Self {
        let error = CompilerError::ParameterTypeMismatch {
            function_name: function_name.to_string(),
            parameter_name: parameter_name.to_string(),
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

impl fmt::Display for EnhancedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;
        
        if let Some(context) = &self.context {
            if let Some(signature) = &context.function_signature {
                write!(f, "\n  function signature: {}", signature)?;
            }
            
            if !context.available_functions.is_empty() {
                write!(f, "\n  available functions: {}", context.available_functions.join(", "))?;
            }
        }
        
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
    
    suggestions.sort_by_key(|s| levenshtein_distance(target, s));
    suggestions.truncate(3);
    
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
                    matrix[i-1][j] + 1,
                    matrix[i][j-1] + 1
                ),
                matrix[i-1][j-1] + cost
            );
        }
    }
    
    matrix[len1][len2]
}

fn main() {
    println!("Testing Function Validation and Enhanced Error Reporting");
    println!("========================================================");
    
    test_undefined_function_with_suggestions();
    test_arity_mismatch_with_signature();
    test_parameter_type_mismatch();
    test_function_redefinition();
    
    println!("\n✅ All function validation tests passed!");
}

fn test_undefined_function_with_suggestions() {
    println!("\n🧪 Testing undefined function with suggestions...");
    
    let location = SourceLocation::new(8, 15);
    let available_funcs = vec!["print".to_string(), "println".to_string(), "calculate".to_string()];
    
    let enhanced = EnhancedError::undefined_function_with_suggestions(
        "pint", 
        location, 
        available_funcs
    );
    
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Undefined function 'pint'"));
    assert!(error_msg.contains("Did you mean 'print'?"));
    assert!(error_msg.contains("available functions: print, println, calculate"));
    
    println!("✅ Undefined function with suggestions works");
}

fn test_arity_mismatch_with_signature() {
    println!("\n🧪 Testing arity mismatch with function signature...");
    
    let location = SourceLocation::new(12, 20);
    
    let enhanced = EnhancedError::arity_mismatch_with_signature(
        "add",
        2,
        3,
        location,
        "fn add(a: i32, b: i32) -> i32"
    );
    
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Function 'add' expects 2 arguments, got 3"));
    assert!(error_msg.contains("function signature: fn add(a: i32, b: i32) -> i32"));
    assert!(error_msg.contains("Remove 1 argument(s)"));
    
    println!("✅ Arity mismatch with signature works");
}

fn test_parameter_type_mismatch() {
    println!("\n🧪 Testing parameter type mismatch...");
    
    let location = SourceLocation::new(15, 25);
    
    let enhanced = EnhancedError::parameter_type_mismatch_with_suggestions(
        "multiply",
        "factor",
        "i32",
        "f64",
        location
    );
    
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Parameter 'factor' of function 'multiply' expects type i32, got f64"));
    assert!(error_msg.contains("Convert to integer"));
    assert!(error_msg.contains("try: value as i32"));
    
    println!("✅ Parameter type mismatch works");
}

fn test_function_redefinition() {
    println!("\n🧪 Testing function redefinition error...");
    
    let location = SourceLocation::new(20, 5);
    let previous_location = Some(SourceLocation::new(10, 5));
    
    let error = CompilerError::FunctionRedefinition {
        name: "calculate".to_string(),
        location,
        previous_location,
    };
    
    let enhanced = EnhancedError::new(error)
        .with_suggestion(ErrorSuggestion::new("Functions can only be defined once"))
        .with_suggestion(ErrorSuggestion::new("Consider using a different function name"));
    
    let error_msg = format!("{}", enhanced);
    assert!(error_msg.contains("Function 'calculate' redefined"));
    assert!(error_msg.contains("previously defined at 10:5"));
    assert!(error_msg.contains("Functions can only be defined once"));
    assert!(error_msg.contains("Consider using a different function name"));
    
    println!("✅ Function redefinition error works");
}