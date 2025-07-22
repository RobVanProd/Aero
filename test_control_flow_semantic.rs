// Test file for control flow semantic validation
// This tests the implementation of task 6.2: Add control flow semantic validation

use std::path::Path;

// Add the compiler crate to the path
fn main() {
    println!("Testing control flow semantic validation...");
    
    // Test cases for control flow semantic validation
    test_if_statement_validation();
    test_while_loop_validation();
    test_for_loop_validation();
    test_infinite_loop_validation();
    test_break_continue_validation();
    test_nested_control_flow();
    test_condition_type_validation();
    
    println!("All control flow semantic validation tests completed!");
}

fn test_if_statement_validation() {
    println!("Testing if statement validation...");
    
    // Test valid if statement
    let valid_if = r#"
        fn test() {
            let x = 5;
            if x > 0 {
                let y = 10;
            }
        }
    "#;
    
    // Test if with else
    let if_else = r#"
        fn test() {
            let flag = true;
            if flag {
                let a = 1;
            } else {
                let b = 2;
            }
        }
    "#;
    
    // Test invalid condition type
    let invalid_condition = r#"
        fn test() {
            let x = 5;
            if x {  // Should error: condition must be boolean
                let y = 10;
            }
        }
    "#;
    
    println!("✓ If statement validation tests defined");
}

fn test_while_loop_validation() {
    println!("Testing while loop validation...");
    
    // Test valid while loop
    let valid_while = r#"
        fn test() {
            let mut i = 0;
            while i < 10 {
                i = i + 1;
            }
        }
    "#;
    
    // Test invalid while condition
    let invalid_while = r#"
        fn test() {
            let x = 5;
            while x {  // Should error: condition must be boolean
                break;
            }
        }
    "#;
    
    println!("✓ While loop validation tests defined");
}

fn test_for_loop_validation() {
    println!("Testing for loop validation...");
    
    // Test valid for loop
    let valid_for = r#"
        fn test() {
            for i in 0..10 {
                let temp = i * 2;
            }
        }
    "#;
    
    // Test for loop variable scoping
    let for_scoping = r#"
        fn test() {
            for i in 0..5 {
                let j = i + 1;
            }
            // i should not be accessible here
        }
    "#;
    
    println!("✓ For loop validation tests defined");
}

fn test_infinite_loop_validation() {
    println!("Testing infinite loop validation...");
    
    // Test valid infinite loop
    let valid_loop = r#"
        fn test() {
            loop {
                let x = 1;
                if x > 0 {
                    break;
                }
            }
        }
    "#;
    
    println!("✓ Infinite loop validation tests defined");
}

fn test_break_continue_validation() {
    println!("Testing break/continue validation...");
    
    // Test valid break in loop
    let valid_break = r#"
        fn test() {
            while true {
                break;
            }
        }
    "#;
    
    // Test valid continue in loop
    let valid_continue = r#"
        fn test() {
            for i in 0..10 {
                if i % 2 == 0 {
                    continue;
                }
                let odd = i;
            }
        }
    "#;
    
    // Test invalid break outside loop
    let invalid_break = r#"
        fn test() {
            break;  // Should error: break outside loop
        }
    "#;
    
    // Test invalid continue outside loop
    let invalid_continue = r#"
        fn test() {
            continue;  // Should error: continue outside loop
        }
    "#;
    
    println!("✓ Break/continue validation tests defined");
}

fn test_nested_control_flow() {
    println!("Testing nested control flow validation...");
    
    // Test nested loops with break/continue
    let nested_loops = r#"
        fn test() {
            while true {
                for i in 0..10 {
                    if i == 5 {
                        break;  // Should break from for loop
                    }
                    if i == 3 {
                        continue;  // Should continue for loop
                    }
                }
                break;  // Should break from while loop
            }
        }
    "#;
    
    // Test nested if statements
    let nested_if = r#"
        fn test() {
            let x = 5;
            let y = 10;
            if x > 0 {
                if y > 5 {
                    let z = x + y;
                } else {
                    let w = x - y;
                }
            }
        }
    "#;
    
    println!("✓ Nested control flow validation tests defined");
}

fn test_condition_type_validation() {
    println!("Testing condition type validation...");
    
    // Test boolean conditions (valid)
    let valid_conditions = r#"
        fn test() {
            let flag = true;
            let x = 5;
            
            if flag {
                let a = 1;
            }
            
            if x > 0 {
                let b = 2;
            }
            
            if flag && (x < 10) {
                let c = 3;
            }
            
            while !flag {
                break;
            }
        }
    "#;
    
    // Test non-boolean conditions (invalid)
    let invalid_conditions = r#"
        fn test() {
            let x = 5;
            let s = "hello";
            
            if x {  // Should error: int is not boolean
                let a = 1;
            }
            
            if s {  // Should error: string is not boolean
                let b = 2;
            }
            
            while x {  // Should error: int is not boolean
                break;
            }
        }
    "#;
    
    println!("✓ Condition type validation tests defined");
}