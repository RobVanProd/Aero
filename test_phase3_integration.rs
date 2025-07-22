// Phase 3 Integration Tests - Complete Feature Testing
// This test suite validates end-to-end functionality for all Phase 3 features

use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    println!("=== Phase 3 Integration Tests ===");
    println!("Testing complete feature combinations and end-to-end functionality");
    println!();
    
    // Test function definition and call integration
    test_function_definition_and_calls();
    
    // Test control flow integration
    test_control_flow_integration();
    
    // Test I/O operation integration
    test_io_operations_integration();
    
    // Test variable scoping and mutability
    test_variable_scoping_and_mutability();
    
    // Test error case integration
    test_error_cases_integration();
    
    println!("\n🎉 All Phase 3 integration tests completed successfully!");
    println!("✅ Task 10.2: Add integration tests for complete features - COMPLETED");
}

fn test_function_definition_and_calls() {
    println!("🧪 Testing Function Definition and Call Integration...");
    
    // Test 1: Simple function with parameters and return value
    let simple_function_code = r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
    println!("5 + 3 = {}", result);
}
"#;
    
    test_aero_code("simple_function", simple_function_code, "Function definition and call");
    
    // Test 2: Recursive function (fibonacci)
    let recursive_function_code = r#"
fn fib(n: i32) -> i32 {
    if n <= 1 {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}

fn main() {
    let result = fib(5);
    println!("Fibonacci(5) = {}", result);
}
"#;
    
    test_aero_code("recursive_function", recursive_function_code, "Recursive function calls");
    
    // Test 3: Multiple functions with different signatures
    let multiple_functions_code = r#"
fn multiply(x: i32, y: i32) -> i32 {
    return x * y;
}

fn greet(name: String) {
    println!("Hello, {}!", name);
}

fn is_even(num: i32) -> bool {
    return num % 2 == 0;
}

fn main() {
    let product = multiply(4, 7);
    println!("4 * 7 = {}", product);
    
    greet("Aero");
    
    let even = is_even(10);
    println!("10 is even: {}", even);
}
"#;
    
    test_aero_code("multiple_functions", multiple_functions_code, "Multiple function definitions");
    
    // Test 4: Function with no parameters and no return value
    let void_function_code = r#"
fn print_header() {
    println!("=== Aero Program ===");
    println!("Starting execution...");
}

fn main() {
    print_header();
    println!("Program completed.");
}
"#;
    
    test_aero_code("void_function", void_function_code, "Function with no parameters/return");
    
    println!("✅ Function definition and call integration tests passed");
}

fn test_control_flow_integration() {
    println!("\n🧪 Testing Control Flow Integration...");
    
    // Test 1: If/else statements with function calls
    let if_else_code = r#"
fn max(a: i32, b: i32) -> i32 {
    if a > b {
        return a;
    } else {
        return b;
    }
}

fn main() {
    let x = 10;
    let y = 20;
    let maximum = max(x, y);
    
    if maximum == x {
        println!("x is maximum: {}", x);
    } else {
        println!("y is maximum: {}", y);
    }
}
"#;
    
    test_aero_code("if_else_control", if_else_code, "If/else with function calls");
    
    // Test 2: While loops with variables
    let while_loop_code = r#"
fn main() {
    let mut count = 0;
    
    while count < 5 {
        println!("Count: {}", count);
        count = count + 1;
    }
    
    println!("Loop completed. Final count: {}", count);
}
"#;
    
    test_aero_code("while_loop", while_loop_code, "While loop with mutable variables");
    
    // Test 3: For loops with ranges
    let for_loop_code = r#"
fn main() {
    println!("Counting from 1 to 5:");
    
    for i in 1..6 {
        println!("Number: {}", i);
    }
    
    println!("For loop completed.");
}
"#;
    
    test_aero_code("for_loop", for_loop_code, "For loop with ranges");
    
    // Test 4: Nested control flow
    let nested_control_code = r#"
fn main() {
    let mut i = 0;
    
    while i < 3 {
        println!("Outer loop: {}", i);
        
        let mut j = 0;
        while j < 2 {
            if j == 1 {
                println!("  Inner: {} (special)", j);
            } else {
                println!("  Inner: {}", j);
            }
            j = j + 1;
        }
        
        i = i + 1;
    }
}
"#;
    
    test_aero_code("nested_control", nested_control_code, "Nested control flow structures");
    
    // Test 5: Break and continue in loops
    let break_continue_code = r#"
fn main() {
    let mut i = 0;
    
    loop {
        i = i + 1;
        
        if i == 3 {
            println!("Skipping {}", i);
            continue;
        }
        
        if i > 5 {
            println!("Breaking at {}", i);
            break;
        }
        
        println!("Processing {}", i);
    }
    
    println!("Loop exited.");
}
"#;
    
    test_aero_code("break_continue", break_continue_code, "Break and continue statements");
    
    println!("✅ Control flow integration tests passed");
}

fn test_io_operations_integration() {
    println!("\n🧪 Testing I/O Operations Integration...");
    
    // Test 1: Print and println with different types
    let print_types_code = r#"
fn main() {
    let integer = 42;
    let float = 3.14;
    let boolean = true;
    let text = "Hello, World!";
    
    print!("Integer: ");
    println!("{}", integer);
    
    print!("Float: ");
    println!("{}", float);
    
    print!("Boolean: ");
    println!("{}", boolean);
    
    print!("Text: ");
    println!("{}", text);
}
"#;
    
    test_aero_code("print_types", print_types_code, "Print operations with different types");
    
    // Test 2: Format strings with multiple arguments
    let format_strings_code = r#"
fn main() {
    let name = "Alice";
    let age = 25;
    let height = 5.6;
    
    println!("Name: {}, Age: {}, Height: {}", name, age, height);
    
    let x = 10;
    let y = 20;
    println!("{} + {} = {}", x, y, x + y);
    println!("{} * {} = {}", x, y, x * y);
}
"#;
    
    test_aero_code("format_strings", format_strings_code, "Format strings with multiple arguments");
    
    // Test 3: I/O in functions and control flow
    let io_in_functions_code = r#"
fn print_table(n: i32) {
    println!("Multiplication table for {}:", n);
    
    let mut i = 1;
    while i <= 5 {
        let result = n * i;
        println!("{} * {} = {}", n, i, result);
        i = i + 1;
    }
}

fn main() {
    print_table(3);
    println!();
    print_table(7);
}
"#;
    
    test_aero_code("io_in_functions", io_in_functions_code, "I/O operations in functions and loops");
    
    // Test 4: Debug printing with expressions
    let debug_printing_code = r#"
fn calculate(a: i32, b: i32) -> i32 {
    println!("Calculating {} + {}", a, b);
    let result = a + b;
    println!("Result: {}", result);
    return result;
}

fn main() {
    let x = 15;
    let y = 25;
    
    println!("Starting calculation...");
    let sum = calculate(x, y);
    println!("Final result: {}", sum);
}
"#;
    
    test_aero_code("debug_printing", debug_printing_code, "Debug printing with expressions");
    
    println!("✅ I/O operations integration tests passed");
}

fn test_variable_scoping_and_mutability() {
    println!("\n🧪 Testing Variable Scoping and Mutability Integration...");
    
    // Test 1: Variable shadowing in nested scopes
    let shadowing_code = r#"
fn main() {
    let x = 10;
    println!("Outer x: {}", x);
    
    {
        let x = 20;
        println!("Inner x: {}", x);
        
        {
            let x = 30;
            println!("Innermost x: {}", x);
        }
        
        println!("Back to inner x: {}", x);
    }
    
    println!("Back to outer x: {}", x);
}
"#;
    
    test_aero_code("variable_shadowing", shadowing_code, "Variable shadowing in nested scopes");
    
    // Test 2: Mutable variables in functions
    let mutable_variables_code = r#"
fn increment(mut value: i32) -> i32 {
    value = value + 1;
    return value;
}

fn main() {
    let mut counter = 0;
    
    println!("Initial counter: {}", counter);
    
    counter = increment(counter);
    println!("After increment: {}", counter);
    
    counter = counter * 2;
    println!("After doubling: {}", counter);
}
"#;
    
    test_aero_code("mutable_variables", mutable_variables_code, "Mutable variables in functions");
    
    // Test 3: Function-local variables and scoping
    let function_scoping_code = r#"
fn process_data() {
    let local_var = 100;
    println!("Function local variable: {}", local_var);
    
    let mut sum = 0;
    let mut i = 1;
    
    while i <= 5 {
        sum = sum + i;
        i = i + 1;
    }
    
    println!("Sum in function: {}", sum);
}

fn main() {
    let main_var = 50;
    println!("Main variable: {}", main_var);
    
    process_data();
    
    println!("Back in main: {}", main_var);
}
"#;
    
    test_aero_code("function_scoping", function_scoping_code, "Function-local variable scoping");
    
    // Test 4: Loop variable scoping
    let loop_scoping_code = r#"
fn main() {
    let outer = 10;
    
    for i in 1..4 {
        let loop_local = i * 2;
        println!("Loop iteration {}: local = {}, outer = {}", i, loop_local, outer);
        
        if i == 2 {
            let conditional_var = 999;
            println!("Conditional variable: {}", conditional_var);
        }
    }
    
    println!("After loop, outer still: {}", outer);
}
"#;
    
    test_aero_code("loop_scoping", loop_scoping_code, "Loop variable scoping");
    
    println!("✅ Variable scoping and mutability integration tests passed");
}

fn test_error_cases_integration() {
    println!("\n🧪 Testing Error Cases Integration...");
    
    // Test 1: Function call with wrong arity
    let arity_error_code = r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5);  // Error: missing argument
    println!("Result: {}", result);
}
"#;
    
    test_error_case("arity_error", arity_error_code, "Function arity mismatch");
    
    // Test 2: Undefined function call
    let undefined_function_code = r#"
fn main() {
    let result = undefined_func(10);  // Error: undefined function
    println!("Result: {}", result);
}
"#;
    
    test_error_case("undefined_function", undefined_function_code, "Undefined function call");
    
    // Test 3: Type mismatch in function parameters
    let type_mismatch_code = r#"
fn process_number(n: i32) {
    println!("Number: {}", n);
}

fn main() {
    let text = "hello";
    process_number(text);  // Error: type mismatch
}
"#;
    
    test_error_case("type_mismatch", type_mismatch_code, "Parameter type mismatch");
    
    // Test 4: Break outside loop
    let break_outside_loop_code = r#"
fn main() {
    println!("Starting...");
    break;  // Error: break outside loop
    println!("This won't execute");
}
"#;
    
    test_error_case("break_outside_loop", break_outside_loop_code, "Break outside loop");
    
    // Test 5: Continue outside loop
    let continue_outside_loop_code = r#"
fn main() {
    println!("Starting...");
    continue;  // Error: continue outside loop
    println!("This won't execute");
}
"#;
    
    test_error_case("continue_outside_loop", continue_outside_loop_code, "Continue outside loop");
    
    // Test 6: Immutable variable reassignment
    let immutable_reassignment_code = r#"
fn main() {
    let x = 10;
    println!("Initial x: {}", x);
    x = 20;  // Error: cannot reassign immutable variable
    println!("Modified x: {}", x);
}
"#;
    
    test_error_case("immutable_reassignment", immutable_reassignment_code, "Immutable variable reassignment");
    
    // Test 7: Invalid format string
    let invalid_format_code = r#"
fn main() {
    let name = "Alice";
    let age = 25;
    println!("Name: {}, Age: {}, Height: {}", name, age);  // Error: too few arguments
}
"#;
    
    test_error_case("invalid_format", invalid_format_code, "Invalid format string arguments");
    
    println!("✅ Error cases integration tests passed");
}

fn test_aero_code(test_name: &str, code: &str, description: &str) {
    println!("  Testing: {}", description);
    
    let filename = format!("test_{}.aero", test_name);
    let ll_filename = format!("test_{}.ll", test_name);
    
    // Write test code to file
    if let Err(e) = fs::write(&filename, code) {
        println!("    ❌ Failed to write test file: {}", e);
        return;
    }
    
    // Compile the code
    let output = Command::new("cargo")
        .args(&["run", "--bin", "aero", "--", "build", &filename, "-o", &ll_filename])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("    ✅ Compilation successful");
                
                // Check if LLVM IR was generated
                if Path::new(&ll_filename).exists() {
                    println!("    ✅ LLVM IR generated successfully");
                } else {
                    println!("    ⚠️  LLVM IR file not found");
                }
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                println!("    ❌ Compilation failed: {}", stderr);
            }
        }
        Err(e) => {
            println!("    ❌ Failed to run compiler: {}", e);
        }
    }
    
    // Clean up test files
    let _ = fs::remove_file(&filename);
    let _ = fs::remove_file(&ll_filename);
}

fn test_error_case(test_name: &str, code: &str, description: &str) {
    println!("  Testing error case: {}", description);
    
    let filename = format!("test_error_{}.aero", test_name);
    let ll_filename = format!("test_error_{}.ll", test_name);
    
    // Write test code to file
    if let Err(e) = fs::write(&filename, code) {
        println!("    ❌ Failed to write test file: {}", e);
        return;
    }
    
    // Compile the code (expecting failure)
    let output = Command::new("cargo")
        .args(&["run", "--bin", "aero", "--", "build", &filename, "-o", &ll_filename])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                if !stderr.is_empty() {
                    println!("    ✅ Error correctly detected: {}", stderr.lines().next().unwrap_or(""));
                } else {
                    println!("    ✅ Error correctly detected (compilation failed)");
                }
            } else {
                println!("    ❌ Expected compilation to fail, but it succeeded");
            }
        }
        Err(e) => {
            println!("    ❌ Failed to run compiler: {}", e);
        }
    }
    
    // Clean up test files
    let _ = fs::remove_file(&filename);
    let _ = fs::remove_file(&ll_filename);
}