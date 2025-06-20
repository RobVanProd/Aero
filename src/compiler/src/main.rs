mod ast;
mod lexer;
mod parser;
mod semantic_analyzer;
mod ir;
mod ir_generator;
mod code_generator;

use crate::semantic_analyzer::SemanticAnalyzer;
use crate::ir_generator::IrGenerator;

fn main() {
    let source_code = "return 15;"; // Simplified test case
    println!("Compiling: \"{}\"", source_code);

    // Lexing
    let tokens = lexer::tokenize(source_code);
    println!("Tokens: {:?}", tokens);

    // Parsing
    let ast = parser::parse(tokens);
    println!("AST: {:?}", ast);

    // Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new();
    let analyzed_ast = match analyzer.analyze(ast.clone()) {
        Ok(msg) => {
            println!("Semantic Analysis Result: {}", msg);
            ast
        },
        Err(err) => {
            eprintln!("Semantic Analysis Error: {}", err);
            return;
        }
    };

    // IR Generation
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate_ir(analyzed_ast);
    println!("IR: {:?}", ir);

    // Code Generation
    let llvm_ir = code_generator::generate_code(ir);
    println!("LLVM IR:\n{}", llvm_ir);

    println!("Compilation process simulated successfully.");
}


