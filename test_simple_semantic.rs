// Simple test to verify semantic analyzer compiles
use std::process::Command;

fn main() {
    println!("Testing semantic analyzer compilation...");
    
    let output = Command::new("cargo")
        .args(&["check", "--manifest-path", "Aero/src/compiler/Cargo.toml"])
        .output()
        .expect("Failed to execute cargo check");
    
    if output.status.success() {
        println!("✓ Semantic analyzer compiles successfully");
    } else {
        println!("✗ Semantic analyzer compilation failed:");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
}