mod ast;
mod lexer;
mod parser;
mod semantic_analyzer;
mod ir;
mod ir_generator;
mod code_generator;
mod types;
pub mod errors;
mod pattern_matcher;

pub use lexer::{tokenize, tokenize_with_locations, Token, LocatedToken};
pub use parser::{parse, parse_with_locations, Parser};
pub use semantic_analyzer::SemanticAnalyzer;
pub use ir_generator::IrGenerator;
pub use code_generator::{generate_code, CodeGenerator};

#[cfg(test)]
mod error_test;

#[cfg(test)]
mod ast_generic_collection_test;

#[cfg(test)]
mod types_enum_test;



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
