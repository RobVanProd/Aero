use crate::ast::AstNode;
use crate::code_generator;
use crate::ir_generator::IrGenerator;
use crate::lexer;
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
}

pub fn profile_compilation(
    source_code: &str,
    input_file: &str,
) -> Result<CompilationProfile, String> {
    let total_start = Instant::now();
    let mut stages = Vec::new();

    let lex_start = Instant::now();
    let tokens = lexer::tokenize(source_code);
    push_stage(&mut stages, "lexing", lex_start.elapsed());

    let parse_start = Instant::now();
    let mut ast = parser::parse(tokens);
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
    let ir = ir_gen.generate_ir(analyzed_ast);
    push_stage(&mut stages, "ir_generation", ir_start.elapsed());

    let codegen_start = Instant::now();
    let _llvm_ir = code_generator::generate_code(ir);
    push_stage(&mut stages, "code_generation", codegen_start.elapsed());

    Ok(CompilationProfile {
        stages,
        total_ms: total_start.elapsed().as_secs_f64() * 1000.0,
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
    for stage in &profile.stages {
        println!("  {:<20} {:>8.3} ms", stage.name, stage.duration_ms);
    }
    println!("  {:<20} {:>8.3} ms", "total", profile.total_ms);
}

fn resolve_modules(input_file: &str, ast: &mut Vec<AstNode>) -> Result<(), String> {
    let mut resolver = module_resolver::ModuleResolver::new(input_file);
    let mut module_asts = Vec::new();

    for node in ast.iter() {
        if let AstNode::Statement(crate::ast::Statement::ModDecl { name, is_public: _ }) = node {
            let resolved = resolver
                .resolve(name)
                .map_err(|err| format!("Module resolution failed for `{}`: {}", name, err))?;
            let mod_tokens = lexer::tokenize(&resolved.source);
            let mod_ast = parser::parse(mod_tokens);
            module_asts.extend(mod_ast);
        }
    }

    ast.extend(module_asts);
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
