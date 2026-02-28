// Test file for collection and string semantic validation
// This tests the implementation of task 8.3: Add collection and string semantic validation

use std::process::Command;
use std::fs;

fn main() {
    println!("=== Testing Collection and String Semantic Validation ===");
    
    // Test 1: Array literal validation
    test_array_literals();
    
    // Test 2: Array access validation
    test_array_access();
    
    // Test 3: Vec macro validation
    test_vec_macro();
    
    // Test 4: Vec method validation
    test_vec_methods();
    
    // Test 5: String method validation
    test_string_methods();
    
    // Test 6: Format macro validation
    test_format_macro();
    
    // Test 7: String concatenation validation
    test_string_concatenation();
    
    // Test 8: Collection bounds checking
    test_bounds_checking();
    
    println!("\n=== All Collection and String Semantic Validation Tests Completed ===");
}

fn test_array_literals() {
    println!("\n--- Testing Array Literal Validation ---");
    
    // Test 1: Valid homogeneous array
    let valid_array = r#"
fn main() {
    let arr = [1, 2, 3, 4, 5];
    println!("Array: {:?}", arr);
}
"#;
    test_aero_code("valid_array_literal", valid_array, true);
    
    // Test 2: Invalid heterogeneous array
    let invalid_array = r#"
fn main() {
    let arr = [1, 2.5, 3, 4];
    println!("Array: {:?}", arr);
}
"#;
    test_aero_code("invalid_heterogeneous_array", invalid_array, false);
    
    // Test 3: Empty array (should fail without type annotation)
    let empty_array = r#"
fn main() {
    let arr = [];
    println!("Array: {:?}", arr);
}
"#;
    test_aero_code("empty_array_no_type", empty_array, false);
}

fn test_array_access() {
    println!("\n--- Testing Array Access Validation ---");
    
    // Test 1: Valid array access
    let valid_access = r#"
fn main() {
    let arr = [1, 2, 3, 4, 5];
    let first = arr[0];
    println!("First element: {}", first);
}
"#;
    test_aero_code("valid_array_access", valid_access, true);
    
    // Test 2: Invalid index type
    let invalid_index = r#"
fn main() {
    let arr = [1, 2, 3, 4, 5];
    let first = arr[2.5];
    println!("Element: {}", first);
}
"#;
    test_aero_code("invalid_array_index_type", invalid_index, false);
    
    // Test 3: Static bounds checking - out of bounds
    let out_of_bounds = r#"
fn main() {
    let arr = [1, 2, 3];
    let element = arr[5];
    println!("Element: {}", element);
}
"#;
    test_aero_code("array_out_of_bounds", out_of_bounds, false);
    
    // Test 4: Negative index
    let negative_index = r#"
fn main() {
    let arr = [1, 2, 3];
    let element = arr[-1];
    println!("Element: {}", element);
}
"#;
    test_aero_code("array_negative_index", negative_index, false);
}

fn test_vec_macro() {
    println!("\n--- Testing Vec Macro Validation ---");
    
    // Test 1: Valid Vec macro
    let valid_vec = r#"
fn main() {
    let vec = vec![1, 2, 3, 4, 5];
    println!("Vec: {:?}", vec);
}
"#;
    test_aero_code("valid_vec_macro", valid_vec, true);
    
    // Test 2: Invalid heterogeneous Vec
    let invalid_vec = r#"
fn main() {
    let vec = vec![1, 2.5, 3, true];
    println!("Vec: {:?}", vec);
}
"#;
    test_aero_code("invalid_heterogeneous_vec", invalid_vec, false);
    
    // Test 3: Empty Vec (should fail without type annotation)
    let empty_vec = r#"
fn main() {
    let vec = vec![];
    println!("Vec: {:?}", vec);
}
"#;
    test_aero_code("empty_vec_no_type", empty_vec, false);
}

fn test_vec_methods() {
    println!("\n--- Testing Vec Method Validation ---");
    
    // Test 1: Valid Vec methods
    let valid_vec_methods = r#"
fn main() {
    let mut vec = vec![1, 2, 3];
    vec.push(4);
    let len = vec.len();
    let is_empty = vec.is_empty();
    println!("Length: {}, Empty: {}", len, is_empty);
}
"#;
    test_aero_code("valid_vec_methods", valid_vec_methods, true);
    
    // Test 2: Invalid push argument type
    let invalid_push = r#"
fn main() {
    let mut vec = vec![1, 2, 3];
    vec.push("hello");
    println!("Vec: {:?}", vec);
}
"#;
    test_aero_code("invalid_vec_push_type", invalid_push, false);
    
    // Test 3: Invalid method argument count
    let invalid_args = r#"
fn main() {
    let vec = vec![1, 2, 3];
    let len = vec.len(5);
    println!("Length: {}", len);
}
"#;
    test_aero_code("invalid_vec_method_args", invalid_args, false);
    
    // Test 4: Vec get method
    let vec_get = r#"
fn main() {
    let vec = vec![1, 2, 3];
    let element = vec.get(1);
    println!("Element: {:?}", element);
}
"#;
    test_aero_code("vec_get_method", vec_get, true);
    
    // Test 5: Invalid get index type
    let invalid_get = r#"
fn main() {
    let vec = vec![1, 2, 3];
    let element = vec.get(1.5);
    println!("Element: {:?}", element);
}
"#;
    test_aero_code("invalid_vec_get_type", invalid_get, false);
}

fn test_string_methods() {
    println!("\n--- Testing String Method Validation ---");
    
    // Test 1: Valid String methods
    let valid_string_methods = r#"
fn main() {
    let mut s = String::new();
    s.push_str("Hello");
    let len = s.len();
    let is_empty = s.is_empty();
    println!("Length: {}, Empty: {}", len, is_empty);
}
"#;
    test_aero_code("valid_string_methods", valid_string_methods, true);
    
    // Test 2: String contains method
    let string_contains = r#"
fn main() {
    let s = "Hello, World!";
    let contains = s.contains("World");
    println!("Contains 'World': {}", contains);
}
"#;
    test_aero_code("string_contains", string_contains, true);
    
    // Test 3: Invalid contains argument type
    let invalid_contains = r#"
fn main() {
    let s = "Hello, World!";
    let contains = s.contains(42);
    println!("Contains: {}", contains);
}
"#;
    test_aero_code("invalid_string_contains_type", invalid_contains, false);
    
    // Test 4: String transformation methods
    let string_transform = r#"
fn main() {
    let s = "Hello, World!";
    let upper = s.to_uppercase();
    let lower = s.to_lowercase();
    let trimmed = s.trim();
    println!("Upper: {}, Lower: {}, Trimmed: {}", upper, lower, trimmed);
}
"#;
    test_aero_code("string_transform_methods", string_transform, true);
    
    // Test 5: String replace method
    let string_replace = r#"
fn main() {
    let s = "Hello, World!";
    let replaced = s.replace("World", "Aero");
    println!("Replaced: {}", replaced);
}
"#;
    test_aero_code("string_replace", string_replace, true);
    
    // Test 6: Invalid replace argument types
    let invalid_replace = r#"
fn main() {
    let s = "Hello, World!";
    let replaced = s.replace(42, true);
    println!("Replaced: {}", replaced);
}
"#;
    test_aero_code("invalid_string_replace_types", invalid_replace, false);
}

fn test_format_macro() {
    println!("\n--- Testing Format Macro Validation ---");
    
    // Test 1: Valid format macro
    let valid_format = r#"
fn main() {
    let name = "Aero";
    let version = 1;
    let formatted = format!("Language: {}, Version: {}", name, version);
    println!("{}", formatted);
}
"#;
    test_aero_code("valid_format_macro", valid_format, true);
    
    // Test 2: Mismatched placeholder count
    let mismatched_placeholders = r#"
fn main() {
    let name = "Aero";
    let formatted = format!("Language: {}, Version: {}", name);
    println!("{}", formatted);
}
"#;
    test_aero_code("mismatched_format_placeholders", mismatched_placeholders, false);
    
    // Test 3: Too many arguments
    let too_many_args = r#"
fn main() {
    let name = "Aero";
    let version = 1;
    let extra = true;
    let formatted = format!("Language: {}", name, version, extra);
    println!("{}", formatted);
}
"#;
    test_aero_code("format_too_many_args", too_many_args, false);
    
    // Test 4: Non-printable type in format
    let non_printable = r#"
struct Point { x: i32, y: i32 }

fn main() {
    let p = Point { x: 1, y: 2 };
    let formatted = format!("Point: {}", p);
    println!("{}", formatted);
}
"#;
    test_aero_code("format_non_printable", non_printable, true); // Structs are assumed printable for now
}

fn test_string_concatenation() {
    println!("\n--- Testing String Concatenation Validation ---");
    
    // Test 1: Valid string concatenation
    let valid_concat = r#"
fn main() {
    let s1 = "Hello, ";
    let s2 = "World!";
    let result = s1 + s2;
    println!("{}", result);
}
"#;
    test_aero_code("valid_string_concat", valid_concat, true);
    
    // Test 2: String concatenation with other types
    let mixed_concat = r#"
fn main() {
    let s = "Number: ";
    let n = 42;
    let result = s + n;
    println!("{}", result);
}
"#;
    test_aero_code("mixed_string_concat", mixed_concat, true); // Should work with type coercion
    
    // Test 3: String comparison
    let string_comparison = r#"
fn main() {
    let s1 = "hello";
    let s2 = "world";
    let equal = s1 == s2;
    let less = s1 < s2;
    println!("Equal: {}, Less: {}", equal, less);
}
"#;
    test_aero_code("string_comparison", string_comparison, true);
}

fn test_bounds_checking() {
    println!("\n--- Testing Collection Bounds Checking ---");
    
    // Test 1: Array bounds checking with literal index
    let array_bounds = r#"
fn main() {
    let arr = [1, 2, 3];
    let element = arr[10];
    println!("Element: {}", element);
}
"#;
    test_aero_code("array_bounds_literal", array_bounds, false);
    
    // Test 2: Vec access (runtime bounds checking)
    let vec_access = r#"
fn main() {
    let vec = vec![1, 2, 3];
    let element = vec[1];
    println!("Element: {}", element);
}
"#;
    test_aero_code("vec_runtime_access", vec_access, true); // Should compile, runtime check
    
    // Test 3: Array method bounds checking
    let array_method_bounds = r#"
fn main() {
    let arr = [1, 2, 3];
    let element = arr.get(10);
    println!("Element: {:?}", element);
}
"#;
    test_aero_code("array_method_bounds", array_method_bounds, true); // get returns Option
}

fn test_aero_code(test_name: &str, code: &str, should_succeed: bool) {
    let filename = format!("{}.aero", test_name);
    
    // Write the test code to a file
    if let Err(e) = fs::write(&filename, code) {
        println!("❌ Failed to write test file {}: {}", filename, e);
        return;
    }
    
    // Compile the code
    let output = Command::new("cargo")
        .args(&["run", "--", &filename])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            let success = result.status.success();
            let stderr = String::from_utf8_lossy(&result.stderr);
            let stdout = String::from_utf8_lossy(&result.stdout);
            
            if should_succeed {
                if success {
                    println!("✅ {}: Compilation succeeded as expected", test_name);
                } else {
                    println!("❌ {}: Expected success but compilation failed", test_name);
                    println!("   Error: {}", stderr);
                }
            } else {
                if success {
                    println!("❌ {}: Expected failure but compilation succeeded", test_name);
                    println!("   Output: {}", stdout);
                } else {
                    println!("✅ {}: Compilation failed as expected", test_name);
                    if stderr.contains("Array index") || 
                       stderr.contains("Type mismatch") || 
                       stderr.contains("expects") ||
                       stderr.contains("Format string") ||
                       stderr.contains("out of bounds") {
                        println!("   ✓ Error message indicates proper semantic validation");
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ {}: Failed to run compiler: {}", test_name, e);
        }
    }
    
    // Clean up the test file
    let _ = fs::remove_file(&filename);
}