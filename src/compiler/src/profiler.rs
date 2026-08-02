use crate::ast::AstNode;
use crate::code_generator;
use crate::ir_generator::IrGenerator;
use crate::lexer;
use crate::llvm_verifier::{LlvmVerificationMode, verify_llvm_module};
use crate::module_resolver;
use crate::parser;
use crate::semantic_analyzer::SemanticAnalyzer;
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct StageProfile {
    pub name: String,
    pub duration_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CompilationProfile {
    pub stages: Vec<StageProfile>,
    pub total_ms: f64,
    pub llvm_verification: String,
}

pub fn profile_compilation(
    source_code: &str,
    input_file: &str,
) -> Result<CompilationProfile, String> {
    let total_start = Instant::now();
    let mut stages = Vec::new();

    let lex_start = Instant::now();
    let tokens = lexer::try_tokenize_with_locations(source_code, Some(input_file.to_string()))
        .map_err(|err| format!("Lex error: {}", err))?;
    push_stage(&mut stages, "lexing", lex_start.elapsed());

    let parse_start = Instant::now();
    let mut ast =
        parser::parse_with_locations(tokens).map_err(|err| format!("Parse error: {}", err))?;
    push_stage(&mut stages, "parsing", parse_start.elapsed());

    let module_start = Instant::now();
    resolve_modules(input_file, &mut ast)?;
    push_stage(&mut stages, "module_resolution", module_start.elapsed());

    let semantic_start = Instant::now();
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed_ast) = analyzer
        .analyze(ast)
        .map_err(|err| format!("Semantic analysis failed: {}", err))?;
    push_stage(&mut stages, "semantic_analysis", semantic_start.elapsed());

    let ir_start = Instant::now();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen
        .try_generate_ir(analyzed_ast)
        .map_err(|error| match error {
            crate::ir_generator::IrGenerationError::Admission(message) => {
                format!("IR Generation Error: {message}")
            }
            crate::ir_generator::IrGenerationError::Verification(error) => error.to_string(),
        })?;
    push_stage(&mut stages, "ir_generation", ir_start.elapsed());

    let codegen_start = Instant::now();
    let llvm_ir = code_generator::try_generate_code(ir).map_err(|error| match error {
        code_generator::CodeGenerationError::IrVerification(error) => error.to_string(),
        other => format!("Code Generation Error: {other}"),
    })?;
    push_stage(&mut stages, "code_generation", codegen_start.elapsed());

    let verification_start = Instant::now();
    let llvm_verification = verify_llvm_module(&llvm_ir, LlvmVerificationMode::PreferExternal)
        .map_err(|error| error.to_string())?
        .to_string();
    push_stage(
        &mut stages,
        "llvm_verification",
        verification_start.elapsed(),
    );

    Ok(CompilationProfile {
        stages,
        total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
        llvm_verification,
    })
}

pub fn write_trace_file(profile: &CompilationProfile, output_path: &str) -> Result<(), String> {
    let mut trace_events = Vec::new();
    let mut current_ts_us = 0.0_f64;

    for stage in &profile.stages {
        let duration_us = stage.duration_ms * 1000.0;
        trace_events.push(json!({
            "name": stage.name,
            "cat": "aero.compiler",
            "ph": "X",
            "pid": 1,
            "tid": 1,
            "ts": current_ts_us,
            "dur": duration_us
        }));
        current_ts_us += duration_us;
    }

    let payload = json!({
        "displayTimeUnit": "ms",
        "llvmVerification": profile.llvm_verification,
        "traceEvents": trace_events
    });

    fs::write(
        output_path,
        serde_json::to_string_pretty(&payload)
            .map_err(|err| format!("failed to serialize trace: {}", err))?,
    )
    .map_err(|err| format!("failed to write trace file {}: {}", output_path, err))
}

pub fn print_profile(profile: &CompilationProfile) {
    println!("Compilation profile:");
    println!("LLVM verification: {}", profile.llvm_verification);
    for stage in &profile.stages {
        println!("  {:<20} {:>8.3} ms", stage.name, stage.duration_ms);
    }
    println!("  {:<20} {:>8.3} ms", "total", profile.total_ms);
}

fn resolve_modules(input_file: &str, ast: &mut Vec<AstNode>) -> Result<(), String> {
    let modules = module_resolver::collect_direct_modules(ast, Some(input_file))?;
    for module in modules {
        ast.extend(module.ast);
    }
    Ok(())
}

fn push_stage(stages: &mut Vec<StageProfile>, name: &str, elapsed: std::time::Duration) {
    stages.push(StageProfile {
        name: name.to_string(),
        duration_ms: elapsed.as_secs_f64() * 1000.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_collects_expected_stages() {
        let _environment = crate::LLVM_VERIFIER_TEST_ENVIRONMENT_LOCK
            .lock()
            .expect("lock verifier test environment");
        let source = "fn main() { let x = 1; println!(\"{}\", x); }";
        let profile = profile_compilation(source, "main.aero").expect("profile should succeed");

        let names = profile
            .stages
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"lexing"));
        assert!(names.contains(&"parsing"));
        assert!(names.contains(&"module_resolution"));
        assert!(names.contains(&"semantic_analysis"));
        assert!(names.contains(&"ir_generation"));
        assert!(names.contains(&"code_generation"));
        assert!(profile.total_ms >= 0.0);
    }
}
