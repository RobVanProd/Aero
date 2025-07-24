// Manual test for struct parsing functionality
// This file tests the struct parsing implementation without relying on the test framework

use std::process::Command;
use std::fs;

fn main() {
    println!("Testing struct parsing functionality manually...");
    
    // Test 1: Basic struct definition
    test_basic_struct();
    
    // Test 2: Tuple struct definition
    test_tuple_struct();
    
    // Test 3: Generic struct definition
    test_generic_struct();
    
    // Test 4: Struct with public fields
    test_public_fields_struct();
    
    println!("All manual struct parsing tests completed!");
}

fn test_basic_struct() {
    println!("Testing basic struct definition...");
    
    let test_code = r#"
struct Point {
    x: i32,
    y: i32
}

fn main() {
    let p = Point { x: 10, y: 20 };
    println!("Point: ({}, {})", p.x, p.y);
}
"#;
    
    // Write test code to a file
    fs::write("test_basic_struct.aero", test_code).expect("Failed to write test file");
    
    // Try to compile it (this will test the parsing)
    let output = Command::new("cargo")
        .args(&["run", "--", "test_basic_struct.aero"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✓ Basic struct parsing test passed");
            } else {
                println!("✗ Basic struct parsing test failed");
                println!("stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = fs::remove_file("test_basic_struct.aero");
}

fn test_tuple_struct() {
    println!("Testing tuple struct definition...");
    
    let test_code = r#"
struct Point(i32, i32);

fn main() {
    let p = Point(10, 20);
    println!("Tuple Point created");
}
"#;
    
    // Write test code to a file
    fs::write("test_tuple_struct.aero", test_code).expect("Failed to write test file");
    
    // Try to compile it (this will test the parsing)
    let output = Command::new("cargo")
        .args(&["run", "--", "test_tuple_struct.aero"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✓ Tuple struct parsing test passed");
            } else {
                println!("✗ Tuple struct parsing test failed");
                println!("stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = fs::remove_file("test_tuple_struct.aero");
}

fn test_generic_struct() {
    println!("Testing generic struct definition...");
    
    let test_code = r#"
struct Container<T> {
    value: T
}

fn main() {
    let c = Container { value: 42 };
    println!("Container created");
}
"#;
    
    // Write test code to a file
    fs::write("test_generic_struct.aero", test_code).expect("Failed to write test file");
    
    // Try to compile it (this will test the parsing)
    let output = Command::new("cargo")
        .args(&["run", "--", "test_generic_struct.aero"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✓ Generic struct parsing test passed");
            } else {
                println!("✗ Generic struct parsing test failed");
                println!("stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = fs::remove_file("test_generic_struct.aero");
}

fn test_public_fields_struct() {
    println!("Testing struct with public fields...");
    
    let test_code = r#"
struct Person {
    pub name: String,
    age: i32,
    pub email: String
}

fn main() {
    let person = Person { 
        name: "Alice", 
        age: 30, 
        email: "alice@example.com" 
    };
    println!("Person created");
}
"#;
    
    // Write test code to a file
    fs::write("test_public_struct.aero", test_code).expect("Failed to write test file");
    
    // Try to compile it (this will test the parsing)
    let output = Command::new("cargo")
        .args(&["run", "--", "test_public_struct.aero"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✓ Public fields struct parsing test passed");
            } else {
                println!("✗ Public fields struct parsing test failed");
                println!("stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = fs::remove_file("test_public_struct.aero");
}