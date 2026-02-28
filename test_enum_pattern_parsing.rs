use std::path::Path;
use std::process::Command;

fn main() {
    println!("Testing enum and pattern parsing implementation...");
    
    // Test 1: Simple enum definition
    test_simple_enum();
    
    // Test 2: Generic enum definition
    test_generic_enum();
    
    // Test 3: Enum with struct variants
    test_enum_struct_variants();
    
    // Test 4: Simple match expression
    test_simple_match();
    
    // Test 5: Match with enum patterns
    test_match_enum_patterns();
    
    // Test 6: Match with guards
    test_match_guards();
    
    // Test 7: Pattern types
    test_pattern_types();
    
    println!("All enum and pattern parsing tests completed!");
}

fn test_simple_enum() {
    println!("Test 1: Simple enum definition");
    let code = r#"
enum Color {
    Red,
    Green,
    Blue,
}
"#;
    
    test_aero_code(code, "simple_enum");
}

fn test_generic_enum() {
    println!("Test 2: Generic enum definition");
    let code = r#"
enum Option<T> {
    Some(T),
    None,
}
"#;
    
    test_aero_code(code, "generic_enum");
}

fn test_enum_struct_variants() {
    println!("Test 3: Enum with struct variants");
    let code = r#"
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Point,
}
"#;
    
    test_aero_code(code, "enum_struct_variants");
}

fn test_simple_match() {
    println!("Test 4: Simple match expression");
    let code = r#"
fn main() {
    let color = Red;
    let result = match color {
        Red => 1,
        Green => 2,
        Blue => 3,
    };
}
"#;
    
    test_aero_code(code, "simple_match");
}

fn test_match_enum_patterns() {
    println!("Test 5: Match with enum patterns");
    let code = r#"
fn main() {
    let option = Some(42);
    let result = match option {
        Some(x) => x,
        None => 0,
    };
}
"#;
    
    test_aero_code(code, "match_enum_patterns");
}

fn test_match_guards() {
    println!("Test 6: Match with guards");
    let code = r#"
fn main() {
    let x = 5;
    let result = match x {
        n if n > 0 => n,
        _ => 0,
    };
}
"#;
    
    test_aero_code(code, "match_guards");
}

fn test_pattern_types() {
    println!("Test 7: Various pattern types");
    let code = r#"
fn main() {
    let tuple = (1, 2, 3);
    let result = match tuple {
        (x, y, _) => x + y,
    };
    
    let range_test = 5;
    let range_result = match range_test {
        1..=10 => "in range",
        _ => "out of range",
    };
    
    let or_test = Red;
    let or_result = match or_test {
        Red | Green | Blue => "primary color",
    };
}
"#;
    
    test_aero_code(code, "pattern_types");
}

fn test_aero_code(code: &str, test_name: &str) {
    // Write the test code to a temporary file
    let filename = format!("{}.aero", test_name);
    std::fs::write(&filename, code).expect("Failed to write test file");
    
    // Try to compile with the Aero compiler
    let output = Command::new("cargo")
        .args(&["run", "--", &filename])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✓ {} - Parsing successful", test_name);
            } else {
                println!("  ✗ {} - Parsing failed", test_name);
                println!("    stdout: {}", String::from_utf8_lossy(&result.stdout));
                println!("    stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("  ✗ {} - Failed to run compiler: {}", test_name, e);
        }
    }
    
    // Clean up the test file
    let _ = std::fs::remove_file(&filename);
}