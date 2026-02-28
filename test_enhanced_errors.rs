// Standalone test for enhanced error reporting system
// This tests the enhanced error functionality independently

use std::path::Path;

// Include the errors module directly
#[path = "src/compiler/src/errors.rs"]
mod errors;

use errors::{CompilerError, SourceLocation, EnhancedError, ErrorSuggestion, ErrorContext, CompilerErrors};

fn main() {
    println!("Testing Enhanced Error Reporting System");
    println!("========================================");
    
    test_basic_error_creation();
    test_enhanced_error_with_suggestions();
    test_undefined_variable_with_suggestions();
    test_undefined_function_with_suggestions();
    test_type_mismatch_with_suggestions();
    test_syntax_error_with_suggestions();
    test_control_flow_error_with_context();
    test_mutability_error_with_suggestions();
    test_multi_error_reporting();
    test_levenshtein_distance();
    test_error_context_display();
    
    println!("\n✅ All enhanced error tests passed!");
}

fn test_basic_error_creation() {
    println!("\n🧪 Testing basic error creation...");
    
    let location = SourceLocation::new(10, 25);
    let error = CompilerError::undefined_variable("x", location);
    let error_msg = format!("{}", error);
    
    assert!(error_msg.contains("10:25"));
    assert!(error_msg.contains("Undefined variable 'x'"));
    println!("✅ Basic error creation works");
}

fn test_enhanced_error_with_suggestions() {
    println!("\n🧪 Testing enhanced error with suggestions...");
    
    let location = SourceLocation::new(3, 5);
    let error = CompilerError::undefined_variable("z", location);
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

fn test_undefined_function_with_suggestions() {
    println!("\n🧪 Testing undefined function with suggestions...");
    
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
    println!("✅ Undefined function with suggestions works");
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

fn test_syntax_error_with_suggestions() {
    println!("\n🧪 Testing syntax error with suggestions...");
    
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
    
    println!("✅ Syntax error with suggestions works");
}

fn test_control_flow_error_with_context() {
    println!("\n🧪 Testing control flow error with context...");
    
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
    
    println!("✅ Control flow error with context works");
}

fn test_mutability_error_with_suggestions() {
    println!("\n🧪 Testing mutability error with suggestions...");
    
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
    
    println!("✅ Mutability error with suggestions works");
}

fn test_multi_error_reporting() {
    println!("\n🧪 Testing multi-error reporting...");
    
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
    
    println!("✅ Multi-error reporting works");
}

fn test_levenshtein_distance() {
    println!("\n🧪 Testing Levenshtein distance algorithm...");
    
    let candidates = vec![
        "count".to_string(),
        "counter".to_string(), 
        "value".to_string(),
        "variable".to_string()
    ];
    
    let suggestions = errors::find_similar_names("cout", &candidates);
    assert!(suggestions.contains(&"count".to_string()));
    
    let suggestions = errors::find_similar_names("countr", &candidates);
    assert!(suggestions.contains(&"counter".to_string()));
    
    // Test that very different names are not suggested
    let suggestions = errors::find_similar_names("xyz", &candidates);
    assert!(suggestions.is_empty());
    
    println!("✅ Levenshtein distance algorithm works");
}

fn test_error_context_display() {
    println!("\n🧪 Testing error context display...");
    
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
    
    println!("✅ Error context display works");
}