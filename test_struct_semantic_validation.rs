// Test file for struct semantic validation
// Tests for task 8.1: Add struct semantic validation

use std::process::Command;

fn compile_aero_code(code: &str) -> Result<String, String> {
    // Write test code to a temporary file
    std::fs::write("temp_test.aero", code).map_err(|e| format!("Failed to write test file: {}", e))?;
    
    // Run the Aero compiler
    let output = Command::new("cargo")
        .args(&["run", "--", "temp_test.aero"])
        .current_dir("src/compiler")
        .output()
        .map_err(|e| format!("Failed to run compiler: {}", e))?;
    
    // Clean up
    let _ = std::fs::remove_file("temp_test.aero");
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn test_struct_definition_validation() {
    println!("Testing struct definition validation...");
    
    // Test valid struct definition
    let valid_struct = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    println!("Struct defined successfully");
}
"#;
    
    match compile_aero_code(valid_struct) {
        Ok(_) => println!("✓ Valid struct definition accepted"),
        Err(e) => panic!("Valid struct definition rejected: {}", e),
    }
    
    // Test duplicate field names
    let duplicate_fields = r#"
struct Point {
    x: int,
    x: int,
}

fn main() {
    println!("This should fail");
}
"#;
    
    match compile_aero_code(duplicate_fields) {
        Ok(_) => panic!("Duplicate field names should be rejected"),
        Err(e) => {
            if e.contains("Duplicate field") {
                println!("✓ Duplicate field names correctly rejected");
            } else {
                panic!("Wrong error for duplicate fields: {}", e);
            }
        }
    }
}

fn test_struct_instantiation_validation() {
    println!("Testing struct instantiation validation...");
    
    // Test valid struct instantiation
    let valid_instantiation = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println!("Point created successfully");
}
"#;
    
    match compile_aero_code(valid_instantiation) {
        Ok(_) => println!("✓ Valid struct instantiation accepted"),
        Err(e) => panic!("Valid struct instantiation rejected: {}", e),
    }
    
    // Test missing field in instantiation
    let missing_field = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10 };
    println!("This should fail");
}
"#;
    
    match compile_aero_code(missing_field) {
        Ok(_) => panic!("Missing field in instantiation should be rejected"),
        Err(e) => {
            if e.contains("Missing field") {
                println!("✓ Missing field in instantiation correctly rejected");
            } else {
                panic!("Wrong error for missing field: {}", e);
            }
        }
    }
    
    // Test unknown field in instantiation
    let unknown_field = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10, y: 20, z: 30 };
    println!("This should fail");
}
"#;
    
    match compile_aero_code(unknown_field) {
        Ok(_) => panic!("Unknown field in instantiation should be rejected"),
        Err(e) => {
            if e.contains("Unknown field") {
                println!("✓ Unknown field in instantiation correctly rejected");
            } else {
                panic!("Wrong error for unknown field: {}", e);
            }
        }
    }
    
    // Test type mismatch in field
    let type_mismatch = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10.5, y: 20 };
    println!("This should fail");
}
"#;
    
    match compile_aero_code(type_mismatch) {
        Ok(_) => panic!("Type mismatch in field should be rejected"),
        Err(e) => {
            if e.contains("Type mismatch") {
                println!("✓ Type mismatch in field correctly rejected");
            } else {
                panic!("Wrong error for type mismatch: {}", e);
            }
        }
    }
}

fn test_field_access_validation() {
    println!("Testing field access validation...");
    
    // Test valid field access
    let valid_access = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let x_val = p.x;
    println!("Field access successful: {}", x_val);
}
"#;
    
    match compile_aero_code(valid_access) {
        Ok(_) => println!("✓ Valid field access accepted"),
        Err(e) => panic!("Valid field access rejected: {}", e),
    }
    
    // Test access to non-existent field
    let invalid_field = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let z_val = p.z;
    println!("This should fail");
}
"#;
    
    match compile_aero_code(invalid_field) {
        Ok(_) => panic!("Access to non-existent field should be rejected"),
        Err(e) => {
            if e.contains("Field") && e.contains("not found") {
                println!("✓ Access to non-existent field correctly rejected");
            } else {
                panic!("Wrong error for non-existent field: {}", e);
            }
        }
    }
    
    // Test field access on non-struct type
    let non_struct_access = r#"
fn main() {
    let x = 42;
    let val = x.field;
    println!("This should fail");
}
"#;
    
    match compile_aero_code(non_struct_access) {
        Ok(_) => panic!("Field access on non-struct should be rejected"),
        Err(e) => {
            if e.contains("Cannot access field") && e.contains("non-struct") {
                println!("✓ Field access on non-struct correctly rejected");
            } else {
                panic!("Wrong error for non-struct field access: {}", e);
            }
        }
    }
}

fn test_struct_with_base_expression() {
    println!("Testing struct instantiation with base expression...");
    
    // Test valid base expression
    let valid_base = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p1 = Point { x: 10, y: 20 };
    let p2 = Point { x: 30, ..p1 };
    println!("Base expression successful");
}
"#;
    
    match compile_aero_code(valid_base) {
        Ok(_) => println!("✓ Valid base expression accepted"),
        Err(e) => panic!("Valid base expression rejected: {}", e),
    }
    
    // Test invalid base expression type
    let invalid_base_type = r#"
struct Point {
    x: int,
    y: int,
}

struct Circle {
    radius: int,
}

fn main() {
    let c = Circle { radius: 5 };
    let p = Point { x: 10, ..c };
    println!("This should fail");
}
"#;
    
    match compile_aero_code(invalid_base_type) {
        Ok(_) => panic!("Invalid base expression type should be rejected"),
        Err(e) => {
            if e.contains("Base expression") && e.contains("must be of type") {
                println!("✓ Invalid base expression type correctly rejected");
            } else {
                panic!("Wrong error for invalid base type: {}", e);
            }
        }
    }
}

fn test_method_call_validation() {
    println!("Testing method call validation...");
    
    // Test valid method call
    let valid_method = r#"
struct Point {
    x: int,
    y: int,
}

impl Point {
    fn distance(&self) -> float {
        return (self.x * self.x + self.y * self.y) as float;
    }
}

fn main() {
    let p = Point { x: 3, y: 4 };
    let dist = p.distance();
    println!("Distance: {}", dist);
}
"#;
    
    match compile_aero_code(valid_method) {
        Ok(_) => println!("✓ Valid method call accepted"),
        Err(e) => panic!("Valid method call rejected: {}", e),
    }
    
    // Test method call on undefined method
    let undefined_method = r#"
struct Point {
    x: int,
    y: int,
}

fn main() {
    let p = Point { x: 3, y: 4 };
    let result = p.undefined_method();
    println!("This should fail");
}
"#;
    
    match compile_aero_code(undefined_method) {
        Ok(_) => panic!("Undefined method call should be rejected"),
        Err(e) => {
            if e.contains("Method") && e.contains("not found") {
                println!("✓ Undefined method call correctly rejected");
            } else {
                panic!("Wrong error for undefined method: {}", e);
            }
        }
    }
    
    // Test method call with wrong argument count
    let wrong_arg_count = r#"
struct Point {
    x: int,
    y: int,
}

impl Point {
    fn set_x(&mut self, new_x: int) {
        self.x = new_x;
    }
}

fn main() {
    let mut p = Point { x: 3, y: 4 };
    p.set_x(10, 20);
    println!("This should fail");
}
"#;
    
    match compile_aero_code(wrong_arg_count) {
        Ok(_) => panic!("Wrong argument count should be rejected"),
        Err(e) => {
            if e.contains("expects") && e.contains("arguments") {
                println!("✓ Wrong argument count correctly rejected");
            } else {
                panic!("Wrong error for argument count: {}", e);
            }
        }
    }
    
    // Test method call with wrong argument type
    let wrong_arg_type = r#"
struct Point {
    x: int,
    y: int,
}

impl Point {
    fn set_x(&mut self, new_x: int) {
        self.x = new_x;
    }
}

fn main() {
    let mut p = Point { x: 3, y: 4 };
    p.set_x(10.5);
    println!("This should fail");
}
"#;
    
    match compile_aero_code(wrong_arg_type) {
        Ok(_) => panic!("Wrong argument type should be rejected"),
        Err(e) => {
            if e.contains("expects argument") && e.contains("to be of type") {
                println!("✓ Wrong argument type correctly rejected");
            } else {
                panic!("Wrong error for argument type: {}", e);
            }
        }
    }
}

fn test_undefined_struct_type() {
    println!("Testing undefined struct type validation...");
    
    // Test instantiation of undefined struct
    let undefined_struct = r#"
fn main() {
    let p = UndefinedStruct { x: 10, y: 20 };
    println!("This should fail");
}
"#;
    
    match compile_aero_code(undefined_struct) {
        Ok(_) => panic!("Undefined struct type should be rejected"),
        Err(e) => {
            if e.contains("Undefined struct type") {
                println!("✓ Undefined struct type correctly rejected");
            } else {
                panic!("Wrong error for undefined struct: {}", e);
            }
        }
    }
}

fn test_complex_struct_scenarios() {
    println!("Testing complex struct scenarios...");
    
    // Test nested struct access
    let nested_struct = r#"
struct Point {
    x: int,
    y: int,
}

struct Rectangle {
    top_left: Point,
    bottom_right: Point,
}

fn main() {
    let rect = Rectangle {
        top_left: Point { x: 0, y: 0 },
        bottom_right: Point { x: 10, y: 10 },
    };
    let width = rect.bottom_right.x - rect.top_left.x;
    println!("Width: {}", width);
}
"#;
    
    match compile_aero_code(nested_struct) {
        Ok(_) => println!("✓ Nested struct access accepted"),
        Err(e) => panic!("Nested struct access rejected: {}", e),
    }
    
    // Test struct with multiple field types
    let multi_type_struct = r#"
struct Person {
    name: int,
    age: int,
    height: float,
    is_student: bool,
}

fn main() {
    let person = Person {
        name: 42,
        age: 25,
        height: 5.9,
        is_student: true,
    };
    println!("Person created");
}
"#;
    
    match compile_aero_code(multi_type_struct) {
        Ok(_) => println!("✓ Multi-type struct accepted"),
        Err(e) => panic!("Multi-type struct rejected: {}", e),
    }
}

fn main() {
    println!("Running struct semantic validation tests...\n");
    
    test_struct_definition_validation();
    println!();
    
    test_struct_instantiation_validation();
    println!();
    
    test_field_access_validation();
    println!();
    
    test_struct_with_base_expression();
    println!();
    
    test_method_call_validation();
    println!();
    
    test_undefined_struct_type();
    println!();
    
    test_complex_struct_scenarios();
    println!();
    
    println!("All struct semantic validation tests completed!");
}