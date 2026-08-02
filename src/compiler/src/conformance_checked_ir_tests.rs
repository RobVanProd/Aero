use super::{conformance, run_conformance_command_with_report};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-conformance-checked-ir-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create conformance test workspace");
        Self { root }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn checked_ir_failure_keeps_a_complete_report_and_returns_a_nonzero_command_result() {
    let workspace = TestWorkspace::new();
    let report_path = workspace.root.join("conformance.json");
    let report = conformance::run_conformance_suite_with_checked_ir_checker(|_typed_ir| {
        Err("IR Verification Error: injected checked-IR failure".to_string())
    });

    assert_eq!(report.total_cases, 3, "conformance cases were truncated");
    for name in [
        "basic_arithmetic_well_typed",
        "undefined_variable_rejected",
        "function_call_well_typed",
    ] {
        assert!(
            report.case_results.iter().any(|case| case.name == name),
            "checked-IR failure removed conformance case `{name}`"
        );
    }
    for name in [
        "lexer_determinism",
        "parser_determinism",
        "ir_generation_determinism",
        "lowering_pipeline_determinism",
    ] {
        assert!(
            report
                .mechanized_checks
                .iter()
                .any(|check| check.name == name),
            "checked-IR failure removed mechanized check `{name}`"
        );
    }
    assert!(
        report
            .mechanized_checks
            .iter()
            .any(|check| !check.passed && check.details.contains("IR Verification Error:")),
        "injected checked-IR failure was not recorded as a failed mechanized result"
    );
    assert_eq!(
        report.passed_cases + report.failed_cases,
        report.total_cases,
        "case totals do not describe the complete report"
    );
    assert_eq!(
        report.passed_mechanized_checks + report.failed_mechanized_checks,
        report.total_mechanized_checks,
        "mechanized totals do not describe the complete report"
    );

    let command = run_conformance_command_with_report(report, Some(&report_path))
        .expect("serialize and publish requested conformance failure report");
    assert_ne!(
        command.exit_code, 0,
        "failed conformance returned exit zero"
    );
    assert!(
        command.stdout.contains("IR Verification Error:"),
        "command output omitted checked-IR failure: {}",
        command.stdout
    );
    let json = fs::read_to_string(&report_path).expect("read requested conformance failure report");
    for required in [
        "basic_arithmetic_well_typed",
        "undefined_variable_rejected",
        "function_call_well_typed",
        "lexer_determinism",
        "parser_determinism",
        "ir_generation_determinism",
        "lowering_pipeline_determinism",
        "IR Verification Error:",
    ] {
        assert!(
            json.contains(required),
            "published failure report omitted `{required}`: {json}"
        );
    }
}
