pub mod accelerator;
pub mod ast;
mod binding_annotation;
mod builtin_carrier_contract;
mod closure_contract;
mod code_generator;
mod compatibility;
pub mod conformance;
mod const_contract;
mod copy_data_layout;
mod copy_place_contract;
mod copydata_trait_dispatch;
mod enum_match_contract;
pub mod errors;
mod fixed_array_method;
mod function_call_contract;
mod generic_enum_contract;
mod generic_function_contract;
mod generic_struct_contract;
pub mod gpu;
pub mod graph_compiler;
mod ir;
mod ir_generator;
mod ir_verifier;
mod language_profile;
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
mod resolved_profile_shape;
mod scalar_assignment;
pub mod semantic_analyzer;
mod specialization_contract;
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
pub use language_profile::LanguageProfile;
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
use std::time::{Duration, Instant};

#[cfg(test)]
mod checked_ir_contract_test;

#[cfg(test)]
mod error_test;

/// Compiler options for benchmarking.
///
/// Optimization, debug-info, and target overrides remain unsupported. The typed
/// language-profile selection is consumed before semantic analysis.
#[derive(Debug, Clone, Default)]
pub struct CompilerOptions {
    pub optimize: bool,
    pub debug_info: bool,
    pub target: String,
    pub language_profile: LanguageProfile,
}

fn validate_compiler_options(options: &CompilerOptions) -> Result<(), String> {
    if options.optimize || options.debug_info || !options.target.is_empty() {
        return Err("Unsupported CompilerOptions: optimize, debug_info, and target behavior is not implemented; language_profile is the only supported nondefault option".to_string());
    }

    Ok(())
}

fn render_ir_generation_error(error: IrGenerationError) -> String {
    match error {
        IrGenerationError::Admission(message) => format!("IR Generation Error: {message}"),
        IrGenerationError::Verification(error) => error.to_string(),
    }
}

/// Phase timings retained for the compiler-service CLI without duplicating the
/// checked-program pipeline.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckedProgramTimings {
    pub lexing: Duration,
    pub parsing: Duration,
    pub direct_modules: Duration,
    pub semantics: Duration,
    pub checked_ir: Duration,
}

/// Exact result of the canonical source-to-checked-IR pipeline.
///
/// This type is public only so the package's CLI binary can consume the library-owned
/// authority. It is not a stable package or language API.
#[doc(hidden)]
pub struct CheckedProgram {
    checked_ir: CheckedIr,
    language_profile: LanguageProfile,
    _resolved_profile: resolved_profile_shape::ResolvedProfileProgram,
    semantic_message: String,
    direct_module_cache_material: Option<Vec<u8>>,
    timings: CheckedProgramTimings,
}

impl std::fmt::Debug for CheckedProgram {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CheckedProgram")
            .field("checked_ir", &self.checked_ir)
            .field("language_profile", &self.language_profile)
            .field("semantic_message", &self.semantic_message)
            .field(
                "direct_module_cache_material",
                &self.direct_module_cache_material,
            )
            .field("timings", &self.timings)
            .finish()
    }
}

impl CheckedProgram {
    #[doc(hidden)]
    pub fn semantic_message(&self) -> &str {
        &self.semantic_message
    }

    #[doc(hidden)]
    pub fn direct_module_cache_material(&self) -> Option<&[u8]> {
        self.direct_module_cache_material.as_deref()
    }

    #[doc(hidden)]
    pub fn timings(&self) -> CheckedProgramTimings {
        self.timings
    }

    /// Emit LLVM using the physical lane paired with this validated program.
    ///
    /// The profile is deliberately not accepted as a separate argument: callers
    /// cannot pair checked IR admitted under one source profile with another
    /// profile's backend representation.
    #[doc(hidden)]
    pub fn try_generate_llvm(self) -> Result<String, CodeGenerationError> {
        code_generator::try_generate_code_with_profile(self.checked_ir, self.language_profile)
    }
}

/// Canonical compiler-service authority for lexing through verified checked IR.
///
/// The callback observes resolved direct modules for CLI progress reporting only. It
/// cannot alter their source, AST, cache identity, or admission result.
#[doc(hidden)]
pub fn prepare_checked_program_for_compiler_service(
    source: &str,
    filename: Option<String>,
    entry_file: Option<&str>,
) -> Result<CheckedProgram, String> {
    prepare_checked_program_with_module_observer(source, filename, entry_file, |_, _| {})
}

/// Canonical compiler-service authority with read-only direct-module observation.
#[doc(hidden)]
pub fn prepare_checked_program_with_module_observer(
    source: &str,
    filename: Option<String>,
    entry_file: Option<&str>,
    on_resolved: impl FnMut(&str, &Path),
) -> Result<CheckedProgram, String> {
    prepare_checked_program_with_module_observer_and_profile(
        source,
        filename,
        entry_file,
        LanguageProfile::Experimental,
        on_resolved,
    )
}

/// Canonical compiler-service authority with a typed language-profile selection.
///
/// Profile classification occurs after fatal parsing and before module resolution,
/// semantic analysis, checked IR, cache lookup, or backend work.
#[doc(hidden)]
pub fn prepare_checked_program_with_module_observer_and_profile(
    source: &str,
    filename: Option<String>,
    entry_file: Option<&str>,
    language_profile: LanguageProfile,
    mut on_resolved: impl FnMut(&str, &Path),
) -> Result<CheckedProgram, String> {
    let lexing_start = Instant::now();
    let tokens = try_tokenize_with_locations(source, filename)
        .map_err(|err| format!("Lex error: {}", err))?;
    let lexing = lexing_start.elapsed();

    let parsing_start = Instant::now();
    let mut ast = parse_with_locations(tokens).map_err(|err| format!("Parse error: {}", err))?;
    let parsing = parsing_start.elapsed();

    language_profile::validate_language_profile(&ast, language_profile)?;

    let direct_modules_start = Instant::now();
    let direct_module_cache_material =
        collect_direct_modules_for_compiler_service(&mut ast, entry_file, |name, path| {
            on_resolved(name, path)
        })?;
    let direct_modules = direct_modules_start.elapsed();

    let semantics_start = Instant::now();
    let mut semantic_analyzer = SemanticAnalyzer::new();
    let (semantic_message, analyzed_ast, resolved_profile) = semantic_analyzer
        .analyze_with_resolved_profile(ast)
        .map_err(|err| format!("Semantic Analysis Error: {}", err))?;
    let semantics = semantics_start.elapsed();

    let checked_ir_start = Instant::now();
    let mut ir_generator = IrGenerator::new();
    let checked_ir = ir_generator
        .try_generate_ir(analyzed_ast)
        .map_err(render_ir_generation_error)?;
    let checked_ir_time = checked_ir_start.elapsed();

    Ok(CheckedProgram {
        checked_ir,
        language_profile,
        _resolved_profile: resolved_profile,
        semantic_message,
        direct_module_cache_material,
        timings: CheckedProgramTimings {
            lexing,
            parsing,
            direct_modules,
            semantics,
            checked_ir: checked_ir_time,
        },
    })
}

fn compile_source(
    source: &str,
    filename: Option<String>,
    entry_file: Option<&str>,
    language_profile: LanguageProfile,
) -> Result<String, String> {
    let program = prepare_checked_program_with_module_observer_and_profile(
        source,
        filename,
        entry_file,
        language_profile,
        |_, _| {},
    )?;

    // Checked code generation re-verifies the private IR before LLVM emission.
    let llvm_code = program.try_generate_llvm().map_err(|error| match error {
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
    *ast = const_contract::normalize_primitive_consts(std::mem::take(ast))?;
    let direct_modules = module_resolver::collect_direct_modules(ast, entry_file)?;
    if direct_modules.is_empty() {
        return Ok(None);
    }

    let mut cache_material = Vec::new();
    cache_material.extend_from_slice(&(direct_modules.len() as u64).to_be_bytes());
    for module in direct_modules {
        let module_ast = const_contract::normalize_primitive_consts(module.ast)?;
        on_resolved(&module.name, &module.file_path);
        push_direct_module_cache_frame(&mut cache_material, "name", module.name.as_bytes());
        push_direct_module_cache_frame(
            &mut cache_material,
            "candidate",
            module.candidate.as_bytes(),
        );
        push_direct_module_cache_frame(&mut cache_material, "source", module.source.as_bytes());
        ast.extend(module_ast);
    }

    Ok(Some(cache_material))
}

/// Compile exact Aero source text through the checked library pipeline.
///
/// This source-only API cannot resolve `mod` declarations because it has no
/// entry-file directory. Optimization, debug-info, and target overrides are unsupported.
pub fn compile_program(source: &str, options: CompilerOptions) -> Result<String, String> {
    validate_compiler_options(&options)?;
    compile_source(source, None, None, options.language_profile)
}

/// Check exact Aero source text through semantic analysis and verified checked IR.
///
/// This source-only API cannot resolve `mod` declarations because it has no entry-file
/// directory. It never generates LLVM or writes filesystem artifacts. Only
/// optimization, debug-info, and target overrides are unsupported.
pub fn check_program(source: &str, options: CompilerOptions) -> Result<(), String> {
    validate_compiler_options(&options)?;
    prepare_checked_program_with_module_observer_and_profile(
        source,
        None,
        None,
        options.language_profile,
        |_, _| {},
    )
    .map(|_| ())
}

/// Compile an Aero root file through the checked library pipeline.
///
/// The file path supplies located root diagnostics and the base directory for
/// the existing direct `mod name;` compatibility contract. The returned LLVM is
/// kept in memory; this function does not write an artifact or run external tools.
/// Optimization, debug-info, and target overrides are unsupported.
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

    compile_source(
        &source,
        Some(filename.clone()),
        Some(&filename),
        options.language_profile,
    )
}

/// Check an Aero root file through semantic analysis and verified checked IR.
///
/// The file path supplies located diagnostics and the base directory for the existing
/// direct `mod name;` compatibility contract. This function never generates LLVM or
/// writes filesystem artifacts. Optimization, debug-info, and target overrides are unsupported.
pub fn check_file(path: impl AsRef<Path>, options: CompilerOptions) -> Result<(), String> {
    validate_compiler_options(&options)?;

    let path = path.as_ref();
    let source = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "Could not read Aero source file `{}`: {error}",
            path.display()
        )
    })?;
    let filename = path.to_string_lossy().into_owned();

    prepare_checked_program_with_module_observer_and_profile(
        &source,
        Some(filename.clone()),
        Some(&filename),
        options.language_profile,
        |_, _| {},
    )
    .map(|_| ())
}
