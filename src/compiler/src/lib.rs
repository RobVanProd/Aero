pub mod ast;
mod code_generator;
pub mod errors;
pub mod graph_compiler;
mod ir;
mod ir_generator;
pub mod lexer;
pub mod module_resolver;
pub mod parser;
pub mod quantization;
pub mod registry;
pub mod semantic_analyzer;
pub mod stdlib;
pub mod types;

pub use code_generator::{CodeGenerator, generate_code};
pub use ir_generator::IrGenerator;
pub use lexer::{LocatedToken, Token, tokenize, tokenize_with_locations};
pub use parser::{Parser, parse, parse_with_locations};
pub use semantic_analyzer::SemanticAnalyzer;

#[cfg(test)]
mod error_test;

/// Compiler options for benchmarking
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub optimize: bool,
    pub debug_info: bool,
    pub target: String,
}

/// Main compilation function for benchmarking
pub fn compile_program(source: &str, _options: CompilerOptions) -> Result<String, String> {
    // Lexical analysis
    let tokens = tokenize(source);

    // Parsing
    let ast = parse(tokens);

    // Semantic analysis
    let mut semantic_analyzer = SemanticAnalyzer::new();
    let (_analyzed_result, analyzed_ast) = match semantic_analyzer.analyze(ast.clone()) {
        Ok((msg, typed_ast)) => (msg, typed_ast),
        Err(err) => return Err(format!("Semantic Analysis Error: {}", err)),
    };

    // IR generation
    let mut ir_generator = IrGenerator::new();
    let ir = ir_generator.generate_ir(analyzed_ast);

    // Code generation
    let llvm_code = generate_code(ir);

    Ok(llvm_code)
}
