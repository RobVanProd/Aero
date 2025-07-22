use crate::errors::{CompilerError, SourceLocation, EnhancedError, ErrorSuggestion, ErrorContext, CompilerErrors};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_location_creation() {
        let loc = SourceLocation::new(10, 25);
        assert_eq!(loc.line, 10);
        assert_eq!(loc.column, 25);
        assert_eq!(loc.filename, None);
    }

    #[test]
    fn test_source_location_with_filename() {
        let loc = SourceLocation::with_filename(5, 12, "test.aero".to_string());
        assert_eq!(loc.line, 5);
        assert_eq!(loc.column, 12);
        assert_eq!(loc.filename, Some("test.aero".to_string()));
    }

    #[test]
    fn test_source_location_display() {
        let loc_with_file = SourceLocation::with_filename(10, 25, "example.aero".to_string());
        assert_eq!(format!("{}", loc_with_file), "example.aero:10:25");
        
        let loc_no_file = SourceLocation::new(5, 12);
        assert_eq!(format!("{}", loc_no_file), "5:12");
    }

    #[test]
    fn test_compiler_error_creation() {
        let location = SourceLocation::new(1, 5);
        let error = CompilerError::unexpected_token("identifier", "number", location.clone());
        
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("1:5"));
        assert!(error_msg.contains("Expected identifier"));
        assert!(error_msg.contains("found number"));
    }

    #[test]
    fn test_undefined_variable_error() {
        let location = SourceLocation::with_filename(3, 10, "main.aero".to_string());
        let error = CompilerError::undefined_variable("x", location);
        
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("main.aero:3:10"));
        assert!(error_msg.contains("Undefined variable 'x'"));
    }

    #[test]
    fn test_type_mismatch_error() {
        let location = SourceLocation::new(2, 15);
        let error = CompilerError::type_mismatch("i32", "f64", location);
        
        let error_msg = format!("{}", error);
        assert!(error_msg.contains("2:15"));
        assert!(error_msg.contains("Type mismatch"));
        assert!(error_msg.contains("expected i32"));
        assert!(error_msg.contains("got f64"));
    }

    #[test]
    fn test_function_errors() {
        let location = SourceLocation::new(5, 1);
        
        let arity_error = CompilerError::ArityMismatch {
            function_name: "add".to_string(),
            expected: 2,
            actual: 3,
            location: location.clone(),
        };
        
        let error_msg = format!("{}", arity_error);
        assert!(error_msg.contains("5:1"));
        assert!(error_msg.contains("Function 'add' expects 2 arguments, got 3"));
    }

    #[test]
    fn test_control_flow_errors() {
        let location = SourceLocation::new(8, 5);
        
        let break_error = CompilerError::BreakOutsideLoop { location: location.clone() };
        let error_msg = format!("{}", break_error);
        assert!(error_msg.contains("8:5"));
        assert!(error_msg.contains("'break' statement outside of loop"));
        
        let continue_error = CompilerError::ContinueOutsideLoop { location };
        let error_msg = format!("{}", continue_error);
        assert!(error_msg.contains("8:5"));
        assert!(error_msg.contains("'continue' statement outside of loop"));
    }

    #[test]
    fn test_io_errors() {
        let location = SourceLocation::new(12, 8);
        
        let format_error = CompilerError::FormatArgumentMismatch {
            expected: 2,
            actual: 1,
            location,
        };
        
        let error_msg = format!("{}", format_error);
        assert!(error_msg.contains("12:8"));
        assert!(error_msg.contains("Format string expects 2 arguments, got 1"));
    }

    // Enhanced Error System Tests

    #[test]
    fn test_error_suggestion_creation() {
        let suggestion = ErrorSuggestion::new("Try using a different approach");
        assert_eq!(suggestion.message, "Try using a different approach");
        assert_eq!(suggestion.replacement, None);
        assert_eq!(suggestion.location, None);

        let suggestion_with_replacement = ErrorSuggestion::with_replacement(
            "Use comparison operator", 
            "=="
        );
        assert_eq!(suggestion_with_replacement.message, "Use comparison operator");
        assert_eq!(suggestion_with_replacement.replacement, Some("==".to_string()));

        let location = SourceLocation::new(5, 10);
        let suggestion_with_location = ErrorSuggestion::with_location(
            "Variable defined here", 
            location.clone()
        );
        assert_eq!(suggestion_with_location.message, "Variable defined here");
        assert_eq!(suggestion_with_location.location, Some(location));
    }

    #[test]
    fn test_error_context_creation() {
        let context = ErrorContext::new()
            .in_function("main".to_string())
            .with_variables(vec!["x".to_string(), "y".to_string()])
            .with_functions(vec!["add".to_string(), "subtract".to_string()]);

        assert_eq!(context.function_name, Some("main".to_string()));
        assert_eq!(context.current_scope_variables, vec!["x", "y"]);
        assert_eq!(context.available_functions, vec!["add", "subtract"]);
    }

    #[test]
    fn test_enhanced_error_creation() {
        let location = SourceLocation::new(3, 5);
        let error = CompilerError::undefined_variable("z", location);
        let enhanced = EnhancedError::new(error)
            .with_suggestion(ErrorSuggestion::new("Check variable name"))
            .with_context(ErrorContext::new().with_variables(vec!["x".to_string(), "y".to_string()]));

        assert_eq!(enhanced.suggestions.len(), 1);
        assert!(enhanced.context.is_some());
        
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("Undefined variable 'z'"));
        assert!(error_msg.contains("help: Check variable name"));
        assert!(error_msg.contains("available variables: x, y"));
    }

    #[test]
    fn test_undefined_variable_with_suggestions() {
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
    }

    #[test]
    fn test_undefined_function_with_suggestions() {
        let location = SourceLocation::new(8, 15);
        let available_funcs = vec!["print".to_string(), "println".to_string(), "printf".to_string()];
        
        let enhanced = EnhancedError::undefined_function_with_suggestions(
            "pint", 
            location, 
            available_funcs
        );
        
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("Undefined function 'pint'"));
        assert!(error_msg.contains("Did you mean 'print'?"));
        assert!(error_msg.contains("available functions: print, println, printf"));
    }

    #[test]
    fn test_type_mismatch_with_suggestions() {
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
    }

    #[test]
    fn test_syntax_error_with_suggestions() {
        let location = SourceLocation::new(7, 12);
        
        // Test assignment vs comparison suggestion
        let enhanced = EnhancedError::syntax_error_with_suggestions(
            "Invalid syntax", 
            location.clone(), 
            "="
        );
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("Syntax error"));
        assert!(error_msg.contains("Use '==' for comparison"));
        assert!(error_msg.contains("try: =="));

        // Test brace matching suggestion
        let enhanced = EnhancedError::syntax_error_with_suggestions(
            "Unmatched brace", 
            location, 
            "{"
        );
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("Make sure to close this brace with '}'"));
    }

    #[test]
    fn test_control_flow_error_with_context() {
        let location = SourceLocation::new(15, 8);
        let error = CompilerError::BreakOutsideLoop { location };
        
        let enhanced = EnhancedError::control_flow_error_with_context(
            error, 
            Some("main".to_string())
        );
        
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("'break' statement outside of loop"));
        assert!(error_msg.contains("in function 'main'"));
        assert!(error_msg.contains("'break' can only be used inside loops"));
    }

    #[test]
    fn test_mutability_error_with_suggestions() {
        let location = SourceLocation::new(12, 5);
        let def_location = Some(SourceLocation::new(10, 8));
        
        let enhanced = EnhancedError::mutability_error_with_suggestions(
            "x", 
            location, 
            def_location
        );
        
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("Cannot assign to immutable variable 'x'"));
        assert!(error_msg.contains("Declare the variable as mutable"));
        assert!(error_msg.contains("try: let mut x = ..."));
        assert!(error_msg.contains("Variable 'x' was defined here"));
    }

    #[test]
    fn test_multi_error_reporting() {
        let mut errors = CompilerErrors::new();
        
        let location1 = SourceLocation::new(5, 10);
        let error1 = CompilerError::undefined_variable("x", location1);
        errors.add(error1);
        
        let location2 = SourceLocation::new(8, 15);
        let error2 = CompilerError::undefined_function("foo", location2);
        errors.add(error2);
        
        let enhanced_error = EnhancedError::type_mismatch_with_suggestions(
            "i32", 
            "f64", 
            SourceLocation::new(12, 20)
        );
        errors.add_enhanced(enhanced_error);
        
        assert_eq!(errors.len(), 3);
        assert!(!errors.is_empty());
        
        let error_msg = format!("{}", errors);
        assert!(error_msg.contains("Undefined variable 'x'"));
        assert!(error_msg.contains("Undefined function 'foo'"));
        assert!(error_msg.contains("Type mismatch"));
        assert!(error_msg.contains("Convert to integer"));
        
        // Check that errors are separated by blank lines
        let lines: Vec<&str> = error_msg.lines().collect();
        let blank_lines = lines.iter().filter(|line| line.trim().is_empty()).count();
        assert!(blank_lines >= 2); // At least 2 blank lines between 3 errors
    }

    #[test]
    fn test_levenshtein_distance() {
        use crate::errors::find_similar_names;
        
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
    }

    #[test]
    fn test_error_context_display() {
        let location = SourceLocation::new(5, 10);
        let error = CompilerError::undefined_variable("x", location);
        
        let context = ErrorContext::new()
            .in_function("calculate".to_string())
            .with_variables(vec!["a".to_string(), "b".to_string(), "result".to_string()])
            .with_functions(vec!["add".to_string(), "multiply".to_string()]);
        
        let enhanced = EnhancedError::new(error).with_context(context);
        let error_msg = format!("{}", enhanced);
        
        assert!(error_msg.contains("in function 'calculate'"));
        assert!(error_msg.contains("available variables: a, b, result"));
        assert!(error_msg.contains("available functions: add, multiply"));
    }

    #[test]
    fn test_empty_suggestions_and_context() {
        let location = SourceLocation::new(3, 5);
        let enhanced = EnhancedError::undefined_variable_with_suggestions(
            "x", 
            location, 
            vec![] // No available variables
        );
        
        let error_msg = format!("{}", enhanced);
        assert!(error_msg.contains("No variables are currently in scope"));
        assert!(!error_msg.contains("available variables:"));
    }

    #[test]
    fn test_compiler_errors_into_result() {
        let mut errors = CompilerErrors::new();
        let result = errors.into_result("success");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        
        let mut errors = CompilerErrors::new();
        errors.add(CompilerError::undefined_variable("x", SourceLocation::new(1, 1)));
        let result = errors.into_result("success");
        assert!(result.is_err());
    }
}