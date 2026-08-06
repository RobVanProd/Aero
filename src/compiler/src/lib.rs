pub mod accelerator;
pub mod ast;
mod binding_annotation;
mod closure_contract;
mod code_generator;
mod compatibility;
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
mod optimizations;
mod ownership_flow;
pub mod parser;
mod performance_optimizations;
mod primitive_contract;
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
mod use_import_contract;

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
pub use performance_optimizations::PerformanceOptimizer;
pub use semantic_analyzer::SemanticAnalyzer;

#[doc(hidden)]
pub const LIVE_REGISTRY_DISABLED_FOR_COMPILER_SERVICE: &str = registry::LIVE_REGISTRY_DISABLED;

#[doc(hidden)]
pub fn guard_live_registry_transport_for_compiler_service() -> Result<(), String> {
    registry::live_registry_transport_guard()
}

use std::path::Path;

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

fn validate_compiler_options(options: &CompilerOptions) -> Result<(), String> {
    if options.optimize || options.debug_info || !options.target.is_empty() {
        return Err("Unsupported CompilerOptions: only CompilerOptions::default() is supported; optimize, debug_info, and target behavior is not implemented".to_string());
    }

    Ok(())
}

fn compile_source(
    source: &str,
    filename: Option<String>,
    entry_file: Option<&str>,
) -> Result<String, String> {
    // Lexical analysis
    let tokens = try_tokenize_with_locations(source, filename)
        .map_err(|err| format!("Lex error: {}", err))?;

    // Parsing
    let mut ast = parse_with_locations(tokens).map_err(|err| format!("Parse error: {}", err))?;

    // File-aware compilation resolves only the existing direct-module compatibility
    // contract. Source-only compilation has no directory from which to resolve files.
    collect_direct_modules_for_compiler_service(&mut ast, entry_file, |_, _| {})?;

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

fn push_direct_module_cache_frame(bytes: &mut Vec<u8>, label: &str, payload: &[u8]) {
    bytes.extend_from_slice(label.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    bytes.extend_from_slice(payload);
}

/// Internal compiler-service bridge used by the CLI-specific crate.
///
/// This is deliberately hidden from generated documentation and is not a stable
/// language or package API. It keeps direct-module parsing, AST identity, and cache
/// material owned by the library instead of exposing resolver representations.
#[doc(hidden)]
pub fn collect_direct_modules_for_compiler_service(
    ast: &mut Vec<ast::AstNode>,
    entry_file: Option<&str>,
    mut on_resolved: impl FnMut(&str, &Path),
) -> Result<Option<Vec<u8>>, String> {
    let direct_modules = module_resolver::collect_direct_modules(ast, entry_file)?;
    if direct_modules.is_empty() {
        return Ok(None);
    }

    let mut cache_material = Vec::new();
    cache_material.extend_from_slice(&(direct_modules.len() as u64).to_be_bytes());
    for module in direct_modules {
        on_resolved(&module.name, &module.file_path);
        push_direct_module_cache_frame(&mut cache_material, "name", module.name.as_bytes());
        push_direct_module_cache_frame(
            &mut cache_material,
            "candidate",
            module.candidate.as_bytes(),
        );
        push_direct_module_cache_frame(&mut cache_material, "source", module.source.as_bytes());
        ast.extend(module.ast);
    }

    Ok(Some(cache_material))
}

/// Compile exact Aero source text through the checked library pipeline.
///
/// This source-only API cannot resolve `mod` declarations because it has no
/// entry-file directory. Only [`CompilerOptions::default`] is supported.
pub fn compile_program(source: &str, options: CompilerOptions) -> Result<String, String> {
    validate_compiler_options(&options)?;
    compile_source(source, None, None)
}

/// Compile an Aero root file through the checked library pipeline.
///
/// The file path supplies located root diagnostics and the base directory for
/// the existing direct `mod name;` compatibility contract. The returned LLVM is
/// kept in memory; this function does not write an artifact or run external tools.
/// Only [`CompilerOptions::default`] is supported.
pub fn compile_file(path: impl AsRef<Path>, options: CompilerOptions) -> Result<String, String> {
    validate_compiler_options(&options)?;

    let path = path.as_ref();
    let source = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "Could not read Aero source file `{}`: {error}",
            path.display()
        )
    })?;
    let filename = path.to_string_lossy().into_owned();

    compile_source(&source, Some(filename.clone()), Some(&filename))
}
