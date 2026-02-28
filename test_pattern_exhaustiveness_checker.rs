// test_pattern_exhaustiveness_checker.rs
// Comprehensive test for pattern exhaustiveness checking functionality

use std::process::Command;

fn main() {
    println!("=== Pattern Exhaustiveness Checker Test ===");
    
    // Test 1: Basic enum exhaustiveness
    test_enum_exhaustiveness();
    
    // Test 2: Missing pattern detection
    test_missing_pattern_detection();
    
    // Test 3: Unreachable pattern detection
    test_unreachable_pattern_detection();
    
    // Test 4: Wildcard pattern handling
    test_wildcard_pattern_handling();
    
    // Test 5: Or-pattern exhaustiveness
    test_or_pattern_exhaustiveness();
    
    // Test 6: Boolean exhaustiveness
    test_boolean_exhaustiveness();
    
    // Test 7: Pattern compilation
    test_pattern_compilation();
    
    // Test 8: Binding extraction
    test_binding_extraction();
    
    // Test 9: Irrefutable patterns
    test_irrefutable_patterns();
    
    // Test 10: Complex nested patterns
    test_complex_nested_patterns();
    
    println!("\n=== All Pattern Exhaustiveness Tests Completed ===");
}

fn test_enum_exhaustiveness() {
    println!("\n--- Test 1: Basic Enum Exhaustiveness ---");
    
    let test_code = r#"
        // Test complete enum pattern matching
        enum Color {
            Red,
            Green,
            Blue,
        }
        
        fn test_color(color: Color) -> int {
            match color {
                Color::Red => 1,
                Color::Green => 2,
                Color::Blue => 3,
            }
        }
        
        fn main() {
            let c = Color::Red;
            let result = test_color(c);
            println!("Color result: {}", result);
        }
    "#;
    
    run_test("enum_exhaustiveness", test_code, true);
}

fn test_missing_pattern_detection() {
    println!("\n--- Test 2: Missing Pattern Detection ---");
    
    let test_code = r#"
        // Test incomplete enum pattern matching - should fail
        enum Color {
            Red,
            Green,
            Blue,
        }
        
        fn test_color(color: Color) -> int {
            match color {
                Color::Red => 1,
                Color::Green => 2,
                // Missing Color::Blue - should be detected
            }
        }
        
        fn main() {
            let c = Color::Red;
            let result = test_color(c);
            println!("Color result: {}", result);
        }
    "#;
    
    // This should fail compilation due to missing pattern
    run_test("missing_pattern", test_code, false);
}

fn test_unreachable_pattern_detection() {
    println!("\n--- Test 3: Unreachable Pattern Detection ---");
    
    let test_code = r#"
        // Test unreachable pattern detection
        enum Color {
            Red,
            Green,
            Blue,
        }
        
        fn test_color(color: Color) -> int {
            match color {
                Color::Red => 1,
                _ => 0,  // Wildcard catches all remaining
                Color::Green => 2,  // This should be unreachable
            }
        }
        
        fn main() {
            let c = Color::Red;
            let result = test_color(c);
            println!("Color result: {}", result);
        }
    "#;
    
    // This should generate warnings about unreachable patterns
    run_test("unreachable_pattern", test_code, true);
}

fn test_wildcard_pattern_handling() {
    println!("\n--- Test 4: Wildcard Pattern Handling ---");
    
    let test_code = r#"
        // Test wildcard pattern making match exhaustive
        enum Color {
            Red,
            Green,
            Blue,
        }
        
        fn test_color(color: Color) -> int {
            match color {
                Color::Red => 1,
                _ => 0,  // Wildcard should make this exhaustive
            }
        }
        
        fn main() {
            let c = Color::Green;
            let result = test_color(c);
            println!("Color result: {}", result);
        }
    "#;
    
    run_test("wildcard_pattern", test_code, true);
}

fn test_or_pattern_exhaustiveness() {
    println!("\n--- Test 5: Or-Pattern Exhaustiveness ---");
    
    let test_code = r#"
        // Test or-patterns in exhaustiveness checking
        enum Color {
            Red,
            Green,
            Blue,
            Yellow,
        }
        
        fn test_color(color: Color) -> int {
            match color {
                Color::Red | Color::Green => 1,  // Or-pattern
                Color::Blue | Color::Yellow => 2,  // Or-pattern
            }
        }
        
        fn main() {
            let c = Color::Red;
            let result = test_color(c);
            println!("Color result: {}", result);
        }
    "#;
    
    run_test("or_pattern_exhaustiveness", test_code, true);
}

fn test_boolean_exhaustiveness() {
    println!("\n--- Test 6: Boolean Exhaustiveness ---");
    
    let test_code = r#"
        // Test boolean pattern exhaustiveness
        fn test_bool(flag: bool) -> int {
            match flag {
                true => 1,
                false => 0,
            }
        }
        
        fn main() {
            let result1 = test_bool(true);
            let result2 = test_bool(false);
            println!("Bool results: {} {}", result1, result2);
        }
    "#;
    
    run_test("boolean_exhaustiveness", test_code, true);
}

fn test_pattern_compilation() {
    println!("\n--- Test 7: Pattern Compilation ---");
    
    let test_code = r#"
        // Test pattern compilation with various pattern types
        enum Option<T> {
            Some(T),
            None,
        }
        
        fn test_option(opt: Option<int>) -> int {
            match opt {
                Option::Some(x) => x,  // Pattern with binding
                Option::None => 0,     // Simple pattern
            }
        }
        
        fn main() {
            let some_val = Option::Some(42);
            let none_val = Option::None;
            
            let result1 = test_option(some_val);
            let result2 = test_option(none_val);
            
            println!("Option results: {} {}", result1, result2);
        }
    "#;
    
    run_test("pattern_compilation", test_code, true);
}

fn test_binding_extraction() {
    println!("\n--- Test 8: Binding Extraction ---");
    
    let test_code = r#"
        // Test binding extraction from patterns
        struct Point {
            x: int,
            y: int,
        }
        
        fn test_point(p: Point) -> int {
            match p {
                Point { x, y } => x + y,  // Struct pattern with bindings
            }
        }
        
        fn main() {
            let point = Point { x: 10, y: 20 };
            let result = test_point(point);
            println!("Point result: {}", result);
        }
    "#;
    
    run_test("binding_extraction", test_code, true);
}

fn test_irrefutable_patterns() {
    println!("\n--- Test 9: Irrefutable Patterns ---");
    
    let test_code = r#"
        // Test irrefutable patterns in let statements
        struct Point {
            x: int,
            y: int,
        }
        
        fn main() {
            let point = Point { x: 10, y: 20 };
            
            // Irrefutable pattern in let statement
            let Point { x, y } = point;
            
            println!("Extracted coordinates: {} {}", x, y);
        }
    "#;
    
    run_test("irrefutable_patterns", test_code, true);
}

fn test_complex_nested_patterns() {
    println!("\n--- Test 10: Complex Nested Patterns ---");
    
    let test_code = r#"
        // Test complex nested pattern matching
        enum Result<T, E> {
            Ok(T),
            Err(E),
        }
        
        enum Option<T> {
            Some(T),
            None,
        }
        
        fn test_nested(result: Result<Option<int>, int>) -> int {
            match result {
                Result::Ok(Option::Some(x)) => x,      // Nested enum patterns
                Result::Ok(Option::None) => 0,         // Nested enum patterns
                Result::Err(e) => e,                   // Simple enum pattern
            }
        }
        
        fn main() {
            let success = Result::Ok(Option::Some(42));
            let empty = Result::Ok(Option::None);
            let error = Result::Err(-1);
            
            let result1 = test_nested(success);
            let result2 = test_nested(empty);
            let result3 = test_nested(error);
            
            println!("Nested results: {} {} {}", result1, result2, result3);
        }
    "#;
    
    run_test("complex_nested_patterns", test_code, true);
}

fn run_test(test_name: &str, code: &str, should_succeed: bool) {
    println!("Running test: {}", test_name);
    
    // Write test code to file
    let filename = format!("{}.aero", test_name);
    std::fs::write(&filename, code).expect("Failed to write test file");
    
    // Try to compile the test
    let output = Command::new("cargo")
        .args(&["run", "--", "build", &filename, "-o", &format!("{}.ll", test_name)])
        .current_dir("src/compiler")
        .output()
        .expect("Failed to execute compiler");
    
    let success = output.status.success();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    if should_succeed {
        if success {
            println!("✓ Test {} passed (compilation succeeded)", test_name);
            
            // Check for pattern-related messages in output
            if stdout.contains("exhaustive") || stdout.contains("pattern") {
                println!("  Pattern analysis output: {}", stdout.lines().find(|l| l.contains("pattern") || l.contains("exhaustive")).unwrap_or(""));
            }
        } else {
            println!("✗ Test {} failed (compilation should have succeeded)", test_name);
            println!("  Error: {}", stderr);
        }
    } else {
        if success {
            println!("✗ Test {} failed (compilation should have failed)", test_name);
        } else {
            println!("✓ Test {} passed (compilation correctly failed)", test_name);
            
            // Check for expected error messages
            if stderr.contains("exhaustive") || stderr.contains("missing") || stderr.contains("pattern") {
                println!("  Expected error found: {}", stderr.lines().find(|l| l.contains("pattern") || l.contains("exhaustive") || l.contains("missing")).unwrap_or(""));
            }
        }
    }
    
    // Clean up test files
    let _ = std::fs::remove_file(&filename);
    let _ = std::fs::remove_file(&format!("{}.ll", test_name));
    
    println!();
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_pattern_matcher_creation() {
        // This would test the PatternMatcher creation
        // In a real implementation, we'd import and test the actual PatternMatcher
        println!("Testing PatternMatcher creation");
    }
    
    #[test]
    fn test_exhaustiveness_analysis() {
        // This would test the exhaustiveness analysis logic
        println!("Testing exhaustiveness analysis");
    }
    
    #[test]
    fn test_pattern_compilation_logic() {
        // This would test the pattern compilation logic
        println!("Testing pattern compilation logic");
    }
    
    #[test]
    fn test_binding_extraction_logic() {
        // This would test the binding extraction logic
        println!("Testing binding extraction logic");
    }
}