pub mod accelerator;
pub mod ast;
mod binding_annotation;
mod closure_contract;
mod code_generator;
pub mod conformance;
mod copy_place_contract;
mod enum_match_contract;
pub mod errors;
mod fixed_array_method;
mod function_call_contract;
pub mod gpu;
pub mod graph_compiler;
mod ir;
mod ir_generator;
mod ir_verifier;
pub mod lexer;
mod llvm_verifier;
mod local_reference;
mod method_call_contract;
pub mod module_resolver;
mod ownership_flow;
pub mod parser;
pub mod quantization;
pub mod registry;
mod scalar_assignment;
pub mod semantic_analyzer;
mod static_string_equality;
mod static_string_method;
mod static_string_predicate;
pub mod stdlib;
mod struct_contract;
mod tuple_contract;
pub mod types;

pub use code_generator::{CodeGenerationError, CodeGenerator, generate_code, try_generate_code};
pub use ir::{CheckedIr, IrMetadata, LogicalType};
pub use ir_generator::{IrGenerationError, IrGenerator};
pub use ir_verifier::IrVerificationError;
pub use lexer::{
    LocatedToken, Token, tokenize, tokenize_with_locations, try_tokenize_with_locations,
};
pub use llvm_verifier::{
    LlvmVerificationError, LlvmVerificationMode, LlvmVerificationStatus, verify_llvm_module,
};
pub use parser::{Parser, parse, parse_with_locations};
pub use semantic_analyzer::SemanticAnalyzer;

#[cfg(test)]
mod checked_ir_contract_test;

#[cfg(test)]
mod error_test;

/// Compiler options for benchmarking.
///
/// Only [`CompilerOptions::default`] is currently supported.
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub optimize: bool,
    pub debug_info: bool,
    pub target: String,
}

/// Main compilation function for benchmarking.
pub fn compile_program(source: &str, options: CompilerOptions) -> Result<String, String> {
    if options.optimize || options.debug_info || !options.target.is_empty() {
        return Err("Unsupported CompilerOptions: only CompilerOptions::default() is supported; optimize, debug_info, and target behavior is not implemented".to_string());
    }

    // Lexical analysis
    let tokens =
        try_tokenize_with_locations(source, None).map_err(|err| format!("Lex error: {}", err))?;

    // Parsing
    let ast = parse_with_locations(tokens).map_err(|err| format!("Parse error: {}", err))?;

    // Source-only compilation has no directory from which to resolve `mod` files.
    module_resolver::collect_direct_modules(&ast, None)?;

    // Semantic analysis
    let mut semantic_analyzer = SemanticAnalyzer::new();
    let (_analyzed_result, analyzed_ast) = match semantic_analyzer.analyze(ast.clone()) {
        Ok((msg, typed_ast)) => (msg, typed_ast),
        Err(err) => return Err(format!("Semantic Analysis Error: {}", err)),
    };

    // Checked IR admission and mandatory in-process verification.
    let mut ir_generator = IrGenerator::new();
    let ir = ir_generator
        .try_generate_ir(analyzed_ast)
        .map_err(|error| match error {
            IrGenerationError::Admission(message) => {
                format!("IR Generation Error: {message}")
            }
            IrGenerationError::Verification(error) => error.to_string(),
        })?;

    // Checked code generation re-verifies the private IR before LLVM emission.
    let llvm_code = try_generate_code(ir).map_err(|error| match error {
        CodeGenerationError::IrVerification(error) => error.to_string(),
        other => format!("Code Generation Error: {other}"),
    })?;

    Ok(llvm_code)
}
