use std::process::Command;

fn main() {
    println!("Testing Phase 4 lexer tokens...");
    
    // Test FatArrow token
    test_fat_arrow();
    
    // Test enum tokens
    test_enum_tokens();
    
    // Test match tokens
    test_match_tokens();
    
    println!("Lexer tests completed!");
}

fn test_fat_arrow() {
    println!("Testing FatArrow token...");
    
    let test_code = r#"
fn test() {
    match x {
        1 => "one",
        2 => "two",
    }
}
"#;
    
    // Write test code to file
    std::fs::write("test_fat_arrow.aero", test_code).expect("Failed to write test file");
    
    // Try to compile
    let output = Command::new("cargo")
        .args(&["run", "--", "build", "test_fat_arrow.aero", "-o", "test_fat_arrow.ll"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✓ FatArrow token test passed");
            } else {
                println!("  ✗ FatArrow token test failed");
                println!("    stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("  ✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = std::fs::remove_file("test_fat_arrow.aero");
    let _ = std::fs::remove_file("test_fat_arrow.ll");
}

fn test_enum_tokens() {
    println!("Testing enum tokens...");
    
    let test_code = r#"
enum Color {
    Red,
    Green,
    Blue,
}
"#;
    
    // Write test code to file
    std::fs::write("test_enum.aero", test_code).expect("Failed to write test file");
    
    // Try to compile
    let output = Command::new("cargo")
        .args(&["run", "--", "build", "test_enum.aero", "-o", "test_enum.ll"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✓ Enum token test passed");
            } else {
                println!("  ✗ Enum token test failed");
                println!("    stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("  ✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = std::fs::remove_file("test_enum.aero");
    let _ = std::fs::remove_file("test_enum.ll");
}

fn test_match_tokens() {
    println!("Testing match tokens...");
    
    let test_code = r#"
fn test() {
    match value {
        _ => 0,
    }
}
"#;
    
    // Write test code to file
    std::fs::write("test_match.aero", test_code).expect("Failed to write test file");
    
    // Try to compile
    let output = Command::new("cargo")
        .args(&["run", "--", "build", "test_match.aero", "-o", "test_match.ll"])
        .current_dir("src/compiler")
        .output();
    
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("  ✓ Match token test passed");
            } else {
                println!("  ✗ Match token test failed");
                println!("    stderr: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("  ✗ Failed to run test: {}", e);
        }
    }
    
    // Clean up
    let _ = std::fs::remove_file("test_match.aero");
    let _ = std::fs::remove_file("test_match.ll");
}