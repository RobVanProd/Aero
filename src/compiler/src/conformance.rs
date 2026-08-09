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
    run_conformance_suite_with_checked_ir_checker(|checked_ir| {
        crate::ir_verifier::verify_checked_ir(&checked_ir)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
}

pub fn run_conformance_suite_with_checked_ir_checker<F>(checked_ir_checker: F) -> ConformanceReport
where
    F: Fn(crate::ir::CheckedIr) -> Result<(), String>,
{
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

    let case_results = cases
        .into_iter()
        .map(|(name, source, expected_success)| {
            evaluate_conformance_case(name, source, expected_success, &checked_ir_checker)
        })
        .collect::<Vec<_>>();

    let mechanized_checks = run_mechanized_semantics_checks_with_checker(&checked_ir_checker);

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

fn evaluate_conformance_case<F>(
    name: &str,
    source: &str,
    expected_success: bool,
    checked_ir_checker: &F,
) -> ConformanceCaseResult
where
    F: Fn(crate::ir::CheckedIr) -> Result<(), String>,
{
    let ast = match parse_conformance_source(source, name) {
        Ok(ast) => ast,
        Err(details) => {
            return ConformanceCaseResult {
                name: name.to_string(),
                expected_success,
                passed: false,
                details,
            };
        }
    };

    let mut analyzer = SemanticAnalyzer::new();
    let outcome = analyzer
        .analyze(ast)
        .map_err(|error| error.to_string())
        .and_then(|(message, typed_ast)| {
            let mut generator = IrGenerator::new();
            let checked_ir = generator
                .try_generate_ir(typed_ast)
                .map_err(render_ir_generation_error)?;
            checked_ir_checker(checked_ir)?;
            Ok(message)
        });
    let passed = outcome.is_ok() == expected_success;
    let details = match outcome {
        Ok(msg) => format!("success: {msg}; checked IR admitted and verified"),
        Err(err) => format!("error: {}", err),
    };

    ConformanceCaseResult {
        name: name.to_string(),
        expected_success,
        passed,
        details,
    }
}

fn parse_conformance_source(source: &str, name: &str) -> Result<Vec<crate::ast::AstNode>, String> {
    let filename = format!("<conformance:{name}>");
    let tokens = lexer::try_tokenize_with_locations(source, Some(filename))
        .map_err(|err| format!("Lex error: {}", err))?;
    parser::parse_with_locations(tokens).map_err(|err| format!("Parse error: {}", err))
}

pub fn run_mechanized_semantics_checks() -> Vec<MechanizedCheckResult> {
    run_mechanized_semantics_checks_with_checker(&|checked_ir| {
        crate::ir_verifier::verify_checked_ir(&checked_ir)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
}

fn run_mechanized_semantics_checks_with_checker<F>(
    checked_ir_checker: &F,
) -> Vec<MechanizedCheckResult>
where
    F: Fn(crate::ir::CheckedIr) -> Result<(), String>,
{
    let source = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = a + b;
}
"#;

    let mut results = run_frontend_mechanized_checks_with_checker(source, checked_ir_checker);

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

fn run_frontend_mechanized_checks(source: &str) -> Vec<MechanizedCheckResult> {
    run_frontend_mechanized_checks_with_checker(source, &|checked_ir| {
        crate::ir_verifier::verify_checked_ir(&checked_ir)
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
}

fn run_frontend_mechanized_checks_with_checker<F>(
    source: &str,
    checked_ir_checker: &F,
) -> Vec<MechanizedCheckResult>
where
    F: Fn(crate::ir::CheckedIr) -> Result<(), String>,
{
    let mut results = Vec::new();

    let tokens_a =
        lexer::try_tokenize_with_locations(source, Some("<conformance:mechanized>".to_string()));
    let tokens_b =
        lexer::try_tokenize_with_locations(source, Some("<conformance:mechanized>".to_string()));
    let (tokens_a, tokens_b) = match (tokens_a, tokens_b) {
        (Ok(tokens_a), Ok(tokens_b)) => (tokens_a, tokens_b),
        (Err(err), _) | (_, Err(err)) => {
            return failed_frontend_checks(format!("Lex error: {}", err));
        }
    };

    results.push(MechanizedCheckResult {
        name: "lexer_determinism".to_string(),
        passed: tokens_a == tokens_b,
        details: if tokens_a == tokens_b {
            "Tokenization is deterministic.".to_string()
        } else {
            "Tokenization produced unstable streams for identical input.".to_string()
        },
    });

    let ast_a = parser::parse_with_locations(tokens_a);
    let ast_b = parser::parse_with_locations(tokens_b);
    let (ast_a, ast_b) = match (ast_a, ast_b) {
        (Ok(ast_a), Ok(ast_b)) => (ast_a, ast_b),
        (Err(err), _) | (_, Err(err)) => {
            let details = format!("Parse error: {}", err);
            results.push(MechanizedCheckResult {
                name: "parser_determinism".to_string(),
                passed: false,
                details: details.clone(),
            });
            results.push(MechanizedCheckResult {
                name: "ir_generation_determinism".to_string(),
                passed: false,
                details,
            });
            return results;
        }
    };

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
            let ir_a = ir_gen_a.try_generate_ir(typed_ast.clone());
            let mut ir_gen_b = IrGenerator::new();
            let ir_b = ir_gen_b.try_generate_ir(typed_ast);
            let (ir_a, ir_b) = match (ir_a, ir_b) {
                (Ok(ir_a), Ok(ir_b)) => (ir_a, ir_b),
                (Err(error), _) | (_, Err(error)) => {
                    results.push(MechanizedCheckResult {
                        name: "ir_generation_determinism".to_string(),
                        passed: false,
                        details: render_ir_generation_error(error),
                    });
                    return results;
                }
            };
            if let Err(details) =
                checked_ir_checker(ir_a.clone()).and_then(|_| checked_ir_checker(ir_b.clone()))
            {
                results.push(MechanizedCheckResult {
                    name: "ir_generation_determinism".to_string(),
                    passed: false,
                    details,
                });
                return results;
            }
            let ir_det = ir_a == ir_b;
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

    results
}

fn render_ir_generation_error(error: crate::ir_generator::IrGenerationError) -> String {
    match error {
        crate::ir_generator::IrGenerationError::Admission(message) => {
            format!("IR Generation Error: {message}")
        }
        crate::ir_generator::IrGenerationError::Verification(error) => error.to_string(),
    }
}

fn failed_frontend_checks(details: String) -> Vec<MechanizedCheckResult> {
    [
        "lexer_determinism",
        "parser_determinism",
        "ir_generation_determinism",
    ]
    .into_iter()
    .map(|name| MechanizedCheckResult {
        name: name.to_string(),
        passed: false,
        details: details.clone(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conformance_suite_runs_and_reports() {
        let report = run_conformance_suite();
        assert_eq!(report.total_cases, 3);
        assert_eq!(report.passed_cases, 3);
        assert_eq!(report.failed_cases, 0);
        assert_eq!(report.total_mechanized_checks, 4);
        assert_eq!(report.passed_mechanized_checks, 4);
        assert_eq!(report.failed_mechanized_checks, 0);
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

    #[test]
    fn invalid_source_becomes_explicit_failed_cases_and_checks() {
        let lexical_source = "fn main() { let value = 1@; }";
        let lexical_case = evaluate_conformance_case(
            "invalid_lexical_source",
            lexical_source,
            true,
            &|_checked_ir| Ok(()),
        );
        assert!(!lexical_case.passed);
        assert!(lexical_case.details.contains("Lex error"));

        let parse_source = "fn main() { let = ; }";
        let parse_case =
            evaluate_conformance_case("invalid_parse_source", parse_source, true, &|_checked_ir| {
                Ok(())
            });
        assert!(!parse_case.passed);
        assert!(parse_case.details.contains("Parse error"));

        let lexical_checks = run_frontend_mechanized_checks(lexical_source);
        assert_eq!(lexical_checks.len(), 3);
        assert!(lexical_checks.iter().all(|check| !check.passed));
        assert!(
            lexical_checks
                .iter()
                .all(|check| check.details.contains("Lex error"))
        );

        let parse_checks = run_frontend_mechanized_checks(parse_source);
        assert_eq!(parse_checks.len(), 3);
        assert!(parse_checks[0].passed);
        assert!(!parse_checks[1].passed);
        assert!(parse_checks[1].details.contains("Parse error"));
        assert!(!parse_checks[2].passed);
        assert!(parse_checks[2].details.contains("Parse error"));
    }
}
