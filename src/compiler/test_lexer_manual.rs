// Manual test for lexer functionality
use crate::lexer::{tokenize, Token};

fn main() {
    // Test I/O macros
    let source1 = r#"print!("Hello") println!("World")"#;
    let tokens1 = tokenize(source1);
    println!("I/O Macros test:");
    for token in &tokens1 {
        println!("  {:?}", token);
    }
    
    // Test comparison operators
    let source2 = "== != < > <= >=";
    let tokens2 = tokenize(source2);
    println!("\nComparison operators test:");
    for token in &tokens2 {
        println!("  {:?}", token);
    }
    
    // Test logical operators
    let source3 = "&& || !";
    let tokens3 = tokenize(source3);
    println!("\nLogical operators test:");
    for token in &tokens3 {
        println!("  {:?}", token);
    }
    
    // Test complex expression
    let source4 = "if x >= 5 && y != 10 || !flag { println!(\"Result: {}\", x + y); }";
    let tokens4 = tokenize(source4);
    println!("\nComplex expression test:");
    for token in &tokens4 {
        println!("  {:?}", token);
    }
}