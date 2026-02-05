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

    pub fn unknown() -> Self {
        SourceLocation {
            line: 0,
            column: 0,
            filename: None,
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

    pub fn with_location(message: &str, location: SourceLocation) -> Self {
        ErrorSuggestion {
            message: message.to_string(),
            replacement: None,
            location: Some(location),
        }
    }
}

/// Context information for errors
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub function_name: Option<String>,
    pub variable_definitions: Vec<(String, SourceLocation)>,
    pub available_functions: Vec<String>,
    pub current_scope_variables: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        ErrorContext {
            function_name: None,
            variable_definitions: Vec::new(),
            available_functions: Vec::new(),
            current_scope_variables: Vec::new(),
        }
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ErrorContext {
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

    pub fn with_suggestions(mut self, suggestions: Vec<ErrorSuggestion>) -> Self {
        self.suggestions = suggestions;
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
                write!(
                    f,
                    "\n  available variables: {}",
                    context.current_scope_variables.join(", ")
                )?;
            }

            if !context.available_functions.is_empty() {
                write!(
                    f,
                    "\n  available functions: {}",
                    context.available_functions.join(", ")
                )?;
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

/// Comprehensive error types for the Aero compiler
#[derive(Debug)]
pub enum CompilerError {
    // Lexer errors
    UnexpectedCharacter {
        character: char,
        location: SourceLocation,
    },
    UnterminatedString {
        location: SourceLocation,
    },
    InvalidNumber {
        text: String,
        location: SourceLocation,
    },

    // Parser errors
    UnexpectedToken {
        expected: String,
        found: String,
        location: SourceLocation,
    },
    UnexpectedEndOfInput {
        expected: String,
        location: SourceLocation,
    },
    InvalidSyntax {
        message: String,
        location: SourceLocation,
    },

    // Function errors
    FunctionRedefinition {
        name: String,
        location: SourceLocation,
        previous_location: Option<SourceLocation>,
    },
    UndefinedFunction {
        name: String,
        location: SourceLocation,
    },
    ArityMismatch {
        function_name: String,
        expected: usize,
        actual: usize,
        location: SourceLocation,
    },
    ParameterTypeMismatch {
        function_name: String,
        parameter_name: String,
        expected: String,
        actual: String,
        location: SourceLocation,
    },
    ReturnTypeMismatch {
        function_name: String,
        expected: String,
        actual: String,
        location: SourceLocation,
    },

    // Control flow errors
    BreakOutsideLoop {
        location: SourceLocation,
    },
    ContinueOutsideLoop {
        location: SourceLocation,
    },
    UnreachableCode {
        location: SourceLocation,
    },
    InvalidConditionType {
        expected: String,
        actual: String,
        location: SourceLocation,
    },

    // Variable errors
    UndefinedVariable {
        name: String,
        location: SourceLocation,
    },
    VariableRedefinition {
        name: String,
        location: SourceLocation,
        previous_location: Option<SourceLocation>,
    },
    ImmutableAssignment {
        name: String,
        location: SourceLocation,
    },
    UninitializedVariable {
        name: String,
        location: SourceLocation,
    },

    // Type errors
    TypeMismatch {
        expected: String,
        actual: String,
        location: SourceLocation,
    },
    IncompatibleTypes {
        left: String,
        right: String,
        operation: String,
        location: SourceLocation,
    },
    InvalidTypeAnnotation {
        type_name: String,
        location: SourceLocation,
    },

    // I/O errors
    InvalidFormatString {
        format: String,
        location: SourceLocation,
    },
    FormatArgumentMismatch {
        expected: usize,
        actual: usize,
        location: SourceLocation,
    },
    InvalidFormatSpecifier {
        specifier: String,
        location: SourceLocation,
    },

    // General semantic errors
    InvalidOperation {
        operation: String,
        operand_type: String,
        location: SourceLocation,
    },
    ScopeError {
        message: String,
        location: SourceLocation,
    },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::UnexpectedCharacter {
                character,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Unexpected character '{}'",
                    location, character
                )
            }
            CompilerError::UnterminatedString { location } => {
                write!(f, "Error at {}: Unterminated string literal", location)
            }
            CompilerError::InvalidNumber { text, location } => {
                write!(f, "Error at {}: Invalid number format '{}'", location, text)
            }
            CompilerError::UnexpectedToken {
                expected,
                found,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Expected {}, found {}",
                    location, expected, found
                )
            }
            CompilerError::UnexpectedEndOfInput { expected, location } => {
                write!(
                    f,
                    "Error at {}: Unexpected end of input, expected {}",
                    location, expected
                )
            }
            CompilerError::InvalidSyntax { message, location } => {
                write!(f, "Syntax error at {}: {}", location, message)
            }
            CompilerError::FunctionRedefinition {
                name,
                location,
                previous_location,
            } => match previous_location {
                Some(prev) => write!(
                    f,
                    "Error at {}: Function '{}' redefined (previously defined at {})",
                    location, name, prev
                ),
                None => write!(f, "Error at {}: Function '{}' redefined", location, name),
            },
            CompilerError::UndefinedFunction { name, location } => {
                write!(f, "Error at {}: Undefined function '{}'", location, name)
            }
            CompilerError::ArityMismatch {
                function_name,
                expected,
                actual,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Function '{}' expects {} arguments, got {}",
                    location, function_name, expected, actual
                )
            }
            CompilerError::ParameterTypeMismatch {
                function_name,
                parameter_name,
                expected,
                actual,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Parameter '{}' of function '{}' expects type {}, got {}",
                    location, parameter_name, function_name, expected, actual
                )
            }
            CompilerError::ReturnTypeMismatch {
                function_name,
                expected,
                actual,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Function '{}' expects return type {}, got {}",
                    location, function_name, expected, actual
                )
            }
            CompilerError::BreakOutsideLoop { location } => {
                write!(
                    f,
                    "Error at {}: 'break' statement outside of loop",
                    location
                )
            }
            CompilerError::ContinueOutsideLoop { location } => {
                write!(
                    f,
                    "Error at {}: 'continue' statement outside of loop",
                    location
                )
            }
            CompilerError::UnreachableCode { location } => {
                write!(f, "Warning at {}: Unreachable code detected", location)
            }
            CompilerError::InvalidConditionType {
                expected,
                actual,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Condition must be of type {}, got {}",
                    location, expected, actual
                )
            }
            CompilerError::UndefinedVariable { name, location } => {
                write!(f, "Error at {}: Undefined variable '{}'", location, name)
            }
            CompilerError::VariableRedefinition {
                name,
                location,
                previous_location,
            } => match previous_location {
                Some(prev) => write!(
                    f,
                    "Error at {}: Variable '{}' redefined (previously defined at {})",
                    location, name, prev
                ),
                None => write!(f, "Error at {}: Variable '{}' redefined", location, name),
            },
            CompilerError::ImmutableAssignment { name, location } => {
                write!(
                    f,
                    "Error at {}: Cannot assign to immutable variable '{}'",
                    location, name
                )
            }
            CompilerError::UninitializedVariable { name, location } => {
                write!(
                    f,
                    "Error at {}: Use of uninitialized variable '{}'",
                    location, name
                )
            }
            CompilerError::TypeMismatch {
                expected,
                actual,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Type mismatch, expected {}, got {}",
                    location, expected, actual
                )
            }
            CompilerError::IncompatibleTypes {
                left,
                right,
                operation,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Incompatible types {} and {} for operation '{}'",
                    location, left, right, operation
                )
            }
            CompilerError::InvalidTypeAnnotation {
                type_name,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Invalid type annotation '{}'",
                    location, type_name
                )
            }
            CompilerError::InvalidFormatString { format, location } => {
                write!(
                    f,
                    "Error at {}: Invalid format string '{}'",
                    location, format
                )
            }
            CompilerError::FormatArgumentMismatch {
                expected,
                actual,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Format string expects {} arguments, got {}",
                    location, expected, actual
                )
            }
            CompilerError::InvalidFormatSpecifier {
                specifier,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Invalid format specifier '{}'",
                    location, specifier
                )
            }
            CompilerError::InvalidOperation {
                operation,
                operand_type,
                location,
            } => {
                write!(
                    f,
                    "Error at {}: Invalid operation '{}' for type {}",
                    location, operation, operand_type
                )
            }
            CompilerError::ScopeError { message, location } => {
                write!(f, "Error at {}: {}", location, message)
            }
        }
    }
}

impl std::error::Error for CompilerError {}

/// Result type for compiler operations
#[allow(clippy::result_large_err)]
pub type CompilerResult<T> = Result<T, CompilerError>;

/// Collection of multiple enhanced compiler errors
#[derive(Debug)]
pub struct CompilerErrors {
    pub errors: Vec<EnhancedError>,
}

impl CompilerErrors {
    pub fn new() -> Self {
        CompilerErrors { errors: Vec::new() }
    }
}

impl Default for CompilerErrors {
    fn default() -> Self {
        Self::new()
    }
}

impl CompilerErrors {
    pub fn add(&mut self, error: CompilerError) {
        self.errors.push(EnhancedError::new(error));
    }

    pub fn add_enhanced(&mut self, error: EnhancedError) {
        self.errors.push(error);
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn len(&self) -> usize {
        self.errors.len()
    }

    pub fn into_result<T>(self, value: T) -> Result<T, CompilerErrors> {
        if self.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl fmt::Display for CompilerErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
                writeln!(f)?; // Extra line between errors for readability
            }
            write!(f, "{}", error)?;
        }
        Ok(())
    }
}

impl std::error::Error for CompilerErrors {}

/// Helper functions for creating common errors
impl CompilerError {
    pub fn unexpected_token(expected: &str, found: &str, location: SourceLocation) -> Self {
        CompilerError::UnexpectedToken {
            expected: expected.to_string(),
            found: found.to_string(),
            location,
        }
    }

    pub fn undefined_variable(name: &str, location: SourceLocation) -> Self {
        CompilerError::UndefinedVariable {
            name: name.to_string(),
            location,
        }
    }

    pub fn type_mismatch(expected: &str, actual: &str, location: SourceLocation) -> Self {
        CompilerError::TypeMismatch {
            expected: expected.to_string(),
            actual: actual.to_string(),
            location,
        }
    }

    pub fn undefined_function(name: &str, location: SourceLocation) -> Self {
        CompilerError::UndefinedFunction {
            name: name.to_string(),
            location,
        }
    }
}

/// Enhanced error creation helpers with suggestions and context
impl EnhancedError {
    /// Create an enhanced undefined variable error with suggestions
    pub fn undefined_variable_with_suggestions(
        name: &str,
        location: SourceLocation,
        available_vars: Vec<String>,
    ) -> Self {
        let error = CompilerError::undefined_variable(name, location);
        let mut enhanced = EnhancedError::new(error);

        // Find similar variable names for suggestions
        let suggestions = find_similar_names(name, &available_vars);
        for suggestion in suggestions {
            enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
                &format!("Did you mean '{}'?", suggestion),
                &suggestion,
            ));
        }

        if available_vars.is_empty() {
            enhanced = enhanced
                .with_suggestion(ErrorSuggestion::new("No variables are currently in scope"));
        } else {
            enhanced = enhanced.with_context(ErrorContext::new().with_variables(available_vars));
        }

        enhanced
    }

    /// Create an enhanced undefined function error with suggestions
    pub fn undefined_function_with_suggestions(
        name: &str,
        location: SourceLocation,
        available_funcs: Vec<String>,
    ) -> Self {
        let error = CompilerError::undefined_function(name, location);
        let mut enhanced = EnhancedError::new(error);

        // Find similar function names for suggestions
        let suggestions = find_similar_names(name, &available_funcs);
        for suggestion in suggestions {
            enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
                &format!("Did you mean '{}'?", suggestion),
                &suggestion,
            ));
        }

        enhanced = enhanced.with_context(ErrorContext::new().with_functions(available_funcs));

        enhanced
    }

    /// Create an enhanced type mismatch error with conversion suggestions
    pub fn type_mismatch_with_suggestions(
        expected: &str,
        actual: &str,
        location: SourceLocation,
    ) -> Self {
        let error = CompilerError::type_mismatch(expected, actual, location);
        let mut enhanced = EnhancedError::new(error);

        // Add type conversion suggestions
        match (expected, actual) {
            ("i32", "f64") => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
                    "Convert to integer",
                    "value as i32",
                ));
            }
            ("f64", "i32") => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
                    "Convert to float",
                    "value as f64",
                ));
            }
            ("bool", _) => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::new(
                    "Use a comparison operator like ==, !=, <, >, <=, >=",
                ));
            }
            ("String", _) => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
                    "Convert to string",
                    "value.to_string()",
                ));
            }
            _ => {}
        }

        enhanced
    }

    /// Create an enhanced syntax error with common fix suggestions
    pub fn syntax_error_with_suggestions(
        message: &str,
        location: SourceLocation,
        found_token: &str,
    ) -> Self {
        let error = CompilerError::InvalidSyntax {
            message: message.to_string(),
            location,
        };
        let mut enhanced = EnhancedError::new(error);

        // Add common syntax fix suggestions
        match found_token {
            "=" => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
                    "Use '==' for comparison",
                    "==",
                ));
            }
            ";" => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::new(
                    "Remove unnecessary semicolon or add expression before it",
                ));
            }
            "{" => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::new(
                    "Make sure to close this brace with '}'",
                ));
            }
            "(" => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::new(
                    "Make sure to close this parenthesis with ')'",
                ));
            }
            _ => {}
        }

        enhanced
    }

    /// Create an enhanced control flow error with context
    pub fn control_flow_error_with_context(
        error: CompilerError,
        current_function: Option<String>,
    ) -> Self {
        let mut enhanced = EnhancedError::new(error);

        if let Some(func_name) = current_function {
            enhanced = enhanced.with_context(ErrorContext::new().in_function(func_name));
        }

        match &enhanced.error {
            CompilerError::BreakOutsideLoop { .. } => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::new(
                    "'break' can only be used inside loops (while, for, loop)",
                ));
            }
            CompilerError::ContinueOutsideLoop { .. } => {
                enhanced = enhanced.with_suggestion(ErrorSuggestion::new(
                    "'continue' can only be used inside loops (while, for, loop)",
                ));
            }
            _ => {}
        }

        enhanced
    }

    /// Create an enhanced mutability error with suggestions
    pub fn mutability_error_with_suggestions(
        name: &str,
        location: SourceLocation,
        definition_location: Option<SourceLocation>,
    ) -> Self {
        let error = CompilerError::ImmutableAssignment {
            name: name.to_string(),
            location,
        };
        let mut enhanced = EnhancedError::new(error);

        enhanced = enhanced.with_suggestion(ErrorSuggestion::with_replacement(
            "Declare the variable as mutable",
            &format!("let mut {} = ...", name),
        ));

        if let Some(def_loc) = definition_location {
            enhanced = enhanced.with_suggestion(ErrorSuggestion::with_location(
                &format!("Variable '{}' was defined here", name),
                def_loc,
            ));
        }

        enhanced
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

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}
