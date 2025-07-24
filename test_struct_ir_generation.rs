// Test file for struct IR generation
use std::process::Command;

fn main() {
    println!("Testing struct IR generation...");
    
    // Test 1: Basic struct definition and instantiation
    test_basic_struct();
    
    // Test 2: Struct field access
    test_struct_field_access();
    
    // Test 3: Tuple struct
    test_tuple_struct();
    
    println!("All struct IR generation tests completed!");
}

fn test_basic_struct() {
    println!("Test 1: Basic struct definition and instantiation");
    
    let test_code = r#"
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println!("Point created");
}
"#;
    
    std::fs::write("test_basic_struct.aero", test_code).expect("Failed to write test file");
    
    let output = Command::new("cargo")
        .args(&["run", "--", "test_basic_struct.aero", "--emit-ir"])
        .current_dir("src/compiler")
        .output()
        .expect("Failed to execute compiler");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT:\n{}", stdout);
    if !stderr.is_empty() {
        println!("STDERR:\n{}", stderr);
    }
    
    // Check if struct definition IR was generated
    assert!(stdout.contains("StructDef") || stderr.contains("StructDef"), 
           "Expected StructDef instruction in IR output");
    
    // Check if struct initialization IR was generated
    assert!(stdout.contains("StructInit") || stderr.contains("StructInit"), 
           "Expected StructInit instruction in IR output");
    
    println!("✓ Basic struct test passed\n");
}

fn test_struct_field_access() {
    println!("Test 2: Struct field access");
    
    let test_code = r#"
struct Point {
    x: i32,
    y: i32,
}

fn main() {
    let p = Point { x: 10, y: 20 };
    let x_val = p.x;
    println!("X value: {}", x_val);
}
"#;
    
    std::fs::write("test_struct_field_access.aero", test_code).expect("Failed to write test file");
    
    let output = Command::new("cargo")
        .args(&["run", "--", "test_struct_field_access.aero", "--emit-ir"])
        .current_dir("src/compiler")
        .output()
        .expect("Failed to execute compiler");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT:\n{}", stdout);
    if !stderr.is_empty() {
        println!("STDERR:\n{}", stderr);
    }
    
    // Check if field access IR was generated
    assert!(stdout.contains("FieldAccess") || stderr.contains("FieldAccess"), 
           "Expected FieldAccess instruction in IR output");
    
    println!("✓ Struct field access test passed\n");
}

fn test_tuple_struct() {
    println!("Test 3: Tuple struct");
    
    let test_code = r#"
struct Color(i32, i32, i32);

fn main() {
    let red = Color(255, 0, 0);
    println!("Color created");
}
"#;
    
    std::fs::write("test_tuple_struct.aero", test_code).expect("Failed to write test file");
    
    let output = Command::new("cargo")
        .args(&["run", "--", "test_tuple_struct.aero", "--emit-ir"])
        .current_dir("src/compiler")
        .output()
        .expect("Failed to execute compiler");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    println!("STDOUT:\n{}", stdout);
    if !stderr.is_empty() {
        println!("STDERR:\n{}", stderr);
    }
    
    // Check if tuple struct definition IR was generated
    assert!(stdout.contains("StructDef") || stderr.contains("StructDef"), 
           "Expected StructDef instruction for tuple struct in IR output");
    
    println!("✓ Tuple struct test passed\n");
}