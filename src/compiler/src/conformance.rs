use crate::accelerator::AcceleratorBackend;
use crate::graph_compiler::{self, GraphCompilationConfig};
use crate::ir_generator::IrGenerator;
use crate::lexer;
use crate::parser;
use crate::quantization::{self, QuantizationConfig, QuantizationMode};
use crate::semantic_analyzer::SemanticAnalyzer;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConformanceCaseResult {
    pub name: String,
    pub expected_success: bool,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MechanizedCheckResult {
    pub name: String,
    pub passed: bool,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConformanceReport {
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub total_mechanized_checks: usize,
    pub passed_mechanized_checks: usize,
    pub failed_mechanized_checks: usize,
    pub case_results: Vec<ConformanceCaseResult>,
    pub mechanized_checks: Vec<MechanizedCheckResult>,
}

pub fn run_conformance_suite() -> ConformanceReport {
    let cases = vec![
        (
            "basic_arithmetic_well_typed",
            r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
}
"#,
            true,
        ),
        (
            "undefined_variable_rejected",
            r#"
fn main() {
    let x = y + 1;
}
"#,
            false,
        ),
        (
            "function_call_well_typed",
            r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
fn main() {
    let n = add(2, 3);
}
"#,
            true,
        ),
    ];

    let mut case_results = Vec::new();
    for (name, source, expected_success) in cases {
        let mut analyzer = SemanticAnalyzer::new();
        let tokens = lexer::tokenize(source);
        let ast = parser::parse(tokens);
        let outcome = analyzer.analyze(ast);
        let passed = outcome.is_ok() == expected_success;
        let details = match outcome {
            Ok((msg, _)) => format!("success: {}", msg),
            Err(err) => format!("error: {}", err),
        };
        case_results.push(ConformanceCaseResult {
            name: name.to_string(),
            expected_success,
            passed,
            details,
        });
    }

    let mechanized_checks = run_mechanized_semantics_checks();

    let total_cases = case_results.len();
    let passed_cases = case_results.iter().filter(|r| r.passed).count();
    let failed_cases = total_cases.saturating_sub(passed_cases);
    let total_mechanized_checks = mechanized_checks.len();
    let passed_mechanized_checks = mechanized_checks.iter().filter(|r| r.passed).count();
    let failed_mechanized_checks = total_mechanized_checks.saturating_sub(passed_mechanized_checks);

    ConformanceReport {
        total_cases,
        passed_cases,
        failed_cases,
        total_mechanized_checks,
        passed_mechanized_checks,
        failed_mechanized_checks,
        case_results,
        mechanized_checks,
    }
}

pub fn run_mechanized_semantics_checks() -> Vec<MechanizedCheckResult> {
    let mut results = Vec::new();
    let source = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
}
"#;

    let tokens_a = lexer::tokenize(source);
    let tokens_b = lexer::tokenize(source);
    results.push(MechanizedCheckResult {
        name: "lexer_determinism".to_string(),
        passed: tokens_a == tokens_b,
        details: if tokens_a == tokens_b {
            "Tokenization is deterministic.".to_string()
        } else {
            "Tokenization produced unstable streams for identical input.".to_string()
        },
    });

    let ast_a = parser::parse(tokens_a.clone());
    let ast_b = parser::parse(tokens_b.clone());
    let ast_det = format!("{:?}", ast_a) == format!("{:?}", ast_b);
    results.push(MechanizedCheckResult {
        name: "parser_determinism".to_string(),
        passed: ast_det,
        details: if ast_det {
            "Parsing is deterministic for identical token streams.".to_string()
        } else {
            "Parser AST output diverged for identical token streams.".to_string()
        },
    });

    let mut analyzer = SemanticAnalyzer::new();
    let analyzed = analyzer.analyze(ast_a.clone());
    match analyzed {
        Ok((_msg, typed_ast)) => {
            let mut ir_gen_a = IrGenerator::new();
            let ir_a = ir_gen_a.generate_ir(typed_ast.clone());
            let mut ir_gen_b = IrGenerator::new();
            let ir_b = ir_gen_b.generate_ir(typed_ast);
            let ir_det = format!("{:?}", ir_a) == format!("{:?}", ir_b);
            results.push(MechanizedCheckResult {
                name: "ir_generation_determinism".to_string(),
                passed: ir_det,
                details: if ir_det {
                    "IR generation is deterministic for identical typed AST input.".to_string()
                } else {
                    "IR generation produced divergent output for identical typed AST input."
                        .to_string()
                },
            });
        }
        Err(err) => results.push(MechanizedCheckResult {
            name: "ir_generation_determinism".to_string(),
            passed: false,
            details: format!("semantic analysis failed before IR check: {}", err),
        }),
    }

    let llvm = "define i32 @main() {\nentry:\n  %0 = fadd double %a, %b\n  %1 = fmul double %0, %c\n  ret i32 0\n}\n";
    let graph_config = GraphCompilationConfig {
        backend: AcceleratorBackend::Rocm,
        gpu_arch: Some("gfx1101".to_string()),
        executable_fusion: true,
    };
    let (graph_a, _) =
        graph_compiler::apply_advanced_graph_compilation_with_config(llvm, &graph_config);
    let (graph_b, _) =
        graph_compiler::apply_advanced_graph_compilation_with_config(llvm, &graph_config);
    let mut q_config = QuantizationConfig::new(QuantizationMode::Int8);
    q_config.backend = AcceleratorBackend::Rocm;
    let (q_a, _) = quantization::apply_quantization_interface(&graph_a, &q_config);
    let (q_b, _) = quantization::apply_quantization_interface(&graph_b, &q_config);
    let lowering_det = q_a == q_b;
    results.push(MechanizedCheckResult {
        name: "lowering_pipeline_determinism".to_string(),
        passed: lowering_det,
        details: if lowering_det {
            "Graph fusion + quantization lowering pipeline is deterministic.".to_string()
        } else {
            "Graph fusion + quantization lowering output diverged.".to_string()
        },
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_suite_runs_and_reports() {
        let report = run_conformance_suite();
        assert!(report.total_cases >= 3);
        assert!(report.total_mechanized_checks >= 4);
        assert!(report.passed_cases >= 2);
    }

    #[test]
    fn mechanized_checks_include_pipeline_determinism() {
        let checks = run_mechanized_semantics_checks();
        assert!(
            checks
                .iter()
                .any(|check| check.name == "lowering_pipeline_determinism")
        );
    }
}
