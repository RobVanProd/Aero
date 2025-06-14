mod ast;
mod lexer;
mod parser;
mod semantic_analyzer;
mod ir_generator;
mod code_generator;

use crate::semantic_analyzer::SemanticAnalyzer;

fn main() {
    let source_code = "let x = 10; let y = x + 5.0; ";
    println!("Compiling: \"{}\"", source_code);

    // Lexing
    let tokens = lexer::tokenize(source_code);
    println!("Tokens: {:?}", tokens);

    // Parsing
    let ast = parser::parse(tokens);
    println!("AST: {:?}", ast);

    // Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    match analyzer.analyze(ast) {
        Ok(msg) => println!("Semantic Analysis Result: {}", msg),
        Err(err) => eprintln!("Semantic Analysis Error: {}", err),
    }

    // IR Generation
    // let ir = ir_generator::generate_ir(analyzed_ast);
    // println!("IR: {}", ir);

    // Code Generation
    // let llvm_ir = code_generator::generate_code(ir);
    // println!("LLVM IR: {}", llvm_ir);

    println!("Compilation process simulated successfully.");
}


