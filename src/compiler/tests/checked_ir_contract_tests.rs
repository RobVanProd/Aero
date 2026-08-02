use compiler::ast::AstNode;
use compiler::{
    CodeGenerationError, CodeGenerator, CompilerOptions, IrGenerationError, IrGenerator,
    SemanticAnalyzer, compile_program, generate_code, parse_with_locations, try_generate_code,
    try_tokenize_with_locations,
};
use std::ffi::OsStr;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-checked-ir-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create checked-IR test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-checked-ir-"));
        if self.root.starts_with(std::env::temp_dir()) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn analyzed_ast(source: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("source must lex");
    let ast = parse_with_locations(tokens).expect("source must parse");
    let mut analyzer = SemanticAnalyzer::new();
    let (_message, typed_ast) = analyzer.analyze(ast).expect("source must pass semantics");
    typed_ast
}

#[test]
fn checked_apis_are_additive_and_keep_structured_error_types() {
    let ast = analyzed_ast("fn main() { let value: int = 1 + 2; }");

    let mut checked_ir_generator = IrGenerator::new();
    let checked_ir: Result<_, IrGenerationError> =
        checked_ir_generator.try_generate_ir(ast.clone());
    let checked_ir = checked_ir.expect("valid source must pass checked IR generation");

    let mut legacy_ir_generator = IrGenerator::new();
    let legacy_ir = legacy_ir_generator.generate_ir(ast);

    let free_checked: Result<String, CodeGenerationError> = try_generate_code(checked_ir.clone());
    let mut method_generator = CodeGenerator::new();
    let method_checked: Result<String, CodeGenerationError> =
        method_generator.try_generate_code(checked_ir.clone());

    for (label, llvm) in [
        ("free checked", free_checked.expect("free checked codegen")),
        (
            "method checked",
            method_checked.expect("method checked codegen"),
        ),
        ("free legacy", generate_code(legacy_ir.clone())),
        (
            "method legacy",
            CodeGenerator::new().generate_code(legacy_ir),
        ),
    ] {
        assert!(!llvm.trim().is_empty(), "{label} returned empty IR");
        assert!(
            !llvm.contains("IR Generation Error:")
                && !llvm.contains("IR Verification Error:")
                && !llvm.contains("Code Generation Error:"),
            "{label} embedded an error as IR: {llvm}"
        );
    }
}

#[test]
fn checked_ir_generation_returns_errors_instead_of_unwinding_or_partial_ir() {
    let cases = [
        (
            "string comparison",
            "fn main() { let compared = \"left\" == \"right\"; }",
        ),
        (
            "constant integer division by zero",
            "fn main() { let divided: int = 1 / 0; }",
        ),
    ];

    for (label, source) in cases {
        let ast = analyzed_ast(source);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            let mut generator = IrGenerator::new();
            let result: Result<_, IrGenerationError> = generator.try_generate_ir(ast);
            result
        }));
        let result = outcome.unwrap_or_else(|_| panic!("{label}: checked IR generation unwound"));
        assert!(
            result.is_err(),
            "{label}: checked IR returned a partial map"
        );

        let public = catch_unwind(AssertUnwindSafe(|| {
            compile_program(source, CompilerOptions::default())
        }));
        let public = public.unwrap_or_else(|_| panic!("{label}: compile_program unwound"));
        let error = match public {
            Ok(llvm) => {
                panic!("{label}: compile_program returned LLVM instead of an error:\n{llvm}")
            }
            Err(error) => error,
        };
        assert!(
            error.starts_with("IR Generation Error:"),
            "{label}: expected exact IR Generation phase identity: {error}"
        );
    }
}

#[derive(Clone, Copy)]
enum FakeVerifierBehavior {
    Accept,
    Reject,
}

fn write_fake_llvm_tool(
    workspace: &TestWorkspace,
    name: &str,
    behavior: FakeVerifierBehavior,
) -> PathBuf {
    #[cfg(windows)]
    let path = workspace.path(&format!("{name}.cmd"));
    #[cfg(not(windows))]
    let path = workspace.path(name);

    let exit_code = match behavior {
        FakeVerifierBehavior::Accept => 0,
        FakeVerifierBehavior::Reject => 42,
    };

    #[cfg(windows)]
    let script = format!(
        r#"@echo off
>>"%AERO_CHECK_FAKE_LOG%" echo invoked %*
if "%~1"=="--version" (
  echo LLVM version 22.1.0
  exit /b 0
)
exit /b {exit_code}
"#
    );
    #[cfg(not(windows))]
    let script = format!(
        r#"#!/bin/sh
printf '%s\n' "invoked $*" >> "$AERO_CHECK_FAKE_LOG"
if [ "${{1:-}}" = "--version" ]; then
  echo "LLVM version 22.1.0"
  exit 0
fi
exit {exit_code}
"#
    );

    fs::write(&path, script).expect("write fake LLVM verifier");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&path)
            .expect("stat fake LLVM verifier")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make fake LLVM verifier executable");
    }
    path
}

fn workspace_snapshot(workspace: &TestWorkspace) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(&workspace.root)
        .expect("read checked-IR workspace")
        .map(|entry| entry.expect("read checked-IR workspace entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn assert_workspace_unchanged(workspace: &TestWorkspace, before: &[PathBuf], operation: &str) {
    let after = workspace_snapshot(workspace);
    assert_eq!(
        after, before,
        "{operation} created an artifact; before={before:?}, after={after:?}"
    );
}

fn run_check_with_verifier(
    workspace: &TestWorkspace,
    source_path: &Path,
    opt_path: &Path,
    llvm_as_path: &Path,
    invocation_log: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("check")
        .arg(source_path)
        .current_dir(&workspace.root)
        .env("AERO_REQUIRE_LLVM_VERIFIER", "true")
        .env("AERO_LLVM_OPT", opt_path)
        .env("AERO_LLVM_AS", llvm_as_path)
        .env("AERO_CHECK_FAKE_LOG", invocation_log)
        .env("PATH", &workspace.root)
        .output()
        .expect("run aero check")
}

fn diagnostics(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
    .replace("\x1b[1;31m", "")
    .replace("\x1b[1;32m", "")
    .replace("\x1b[1;34m", "")
    .replace("\x1b[1;36m", "")
    .replace("\x1b[0m", "")
}

fn stable_check_diagnostics(output: &Output) -> String {
    diagnostics(output)
        .lines()
        .map(|line| {
            if line.contains("Checking") && line.ends_with(')') {
                line.rsplit_once(" (").map_or(line, |(stable, _)| stable)
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_only_source_file_exists(workspace: &TestWorkspace, source_path: &Path) {
    let mut unexpected = fs::read_dir(&workspace.root)
        .expect("read check workspace")
        .map(|entry| entry.expect("workspace entry").path())
        .filter(|path| path != source_path)
        .collect::<Vec<_>>();
    unexpected.sort();
    assert!(
        unexpected.is_empty(),
        "aero check created unexpected artifacts: {unexpected:?}"
    );
}

#[test]
fn check_is_identical_with_missing_accepting_or_rejecting_llvm_tools() {
    for (label, source, expected_success) in [
        ("valid", "fn main() { let value: int = 1 + 2; }", true),
        (
            "string-comparison",
            "fn main() { let compared = \"left\" == \"right\"; }",
            false,
        ),
        (
            "integer-divide-by-zero",
            "fn main() { let divided: int = 1 / 0; }",
            false,
        ),
    ] {
        let workspace = TestWorkspace::new(&format!("check-tool-independent-{label}"));
        let source_path = workspace.path("case.aero");
        fs::write(&source_path, source).expect("write check source");
        let accepting =
            write_fake_llvm_tool(&workspace, "accepting-opt", FakeVerifierBehavior::Accept);
        let rejecting =
            write_fake_llvm_tool(&workspace, "rejecting-opt", FakeVerifierBehavior::Reject);
        let missing = workspace.path("missing-opt-22");

        let configurations = [
            ("missing", missing.as_path(), missing.as_path()),
            ("accepting", accepting.as_path(), accepting.as_path()),
            ("rejecting", rejecting.as_path(), rejecting.as_path()),
        ];
        let mut observations = Vec::new();

        for (configuration, opt, llvm_as) in configurations {
            let invocation_log = workspace.path(&format!("{configuration}-invocations.log"));
            let before = workspace_snapshot(&workspace);
            let output =
                run_check_with_verifier(&workspace, &source_path, opt, llvm_as, &invocation_log);
            let diagnostic = stable_check_diagnostics(&output);
            assert_eq!(
                output.status.success(),
                expected_success,
                "{label}/{configuration}: unexpected check status: {diagnostic}"
            );
            assert!(
                !invocation_log.exists(),
                "{label}/{configuration}: check invoked an external LLVM tool"
            );
            assert_workspace_unchanged(
                &workspace,
                &before,
                &format!("{label}/{configuration} check"),
            );
            assert!(
                !diagnostic.to_ascii_lowercase().contains("panicked at"),
                "{label}/{configuration}: check unwound: {diagnostic}"
            );
            observations.push((
                configuration,
                output.status.code(),
                output.status.success(),
                diagnostic,
            ));
        }

        let reference = &observations[0];
        for observation in &observations[1..] {
            assert_eq!(
                (observation.1, observation.2, &observation.3),
                (reference.1, reference.2, &reference.3),
                "{label}: check changed with LLVM configuration; reference={reference:?}, observed={observation:?}"
            );
        }
        if !expected_success {
            assert!(
                reference.3.contains("error: IR Generation Error:"),
                "{label}: missing exact IR Generation phase identity: {}",
                reference.3
            );
        }
    }
}

#[test]
fn invalid_ir_generation_stops_build_before_backend_or_publication() {
    let workspace = TestWorkspace::new("build-ir-ordering");
    let source_path = workspace.path("invalid.aero");
    let artifact = workspace.path("must-not-exist.ll");
    fs::write(
        &source_path,
        "fn main() { let compared = \"left\" == \"right\"; }",
    )
    .expect("write invalid build source");
    let rejecting = write_fake_llvm_tool(&workspace, "rejecting-opt", FakeVerifierBehavior::Reject);
    let invocation_log = workspace.path("build-verifier-invocations.log");
    let before = workspace_snapshot(&workspace);

    let output = Command::new(env!("CARGO_BIN_EXE_aero"))
        .args([OsStr::new("build"), source_path.as_os_str()])
        .arg("-o")
        .arg(&artifact)
        .current_dir(&workspace.root)
        .env("AERO_REQUIRE_LLVM_VERIFIER", "true")
        .env("AERO_LLVM_OPT", &rejecting)
        .env("AERO_LLVM_AS", &rejecting)
        .env("AERO_CHECK_FAKE_LOG", &invocation_log)
        .env("PATH", &workspace.root)
        .output()
        .expect("run invalid aero build");
    let diagnostic = diagnostics(&output);

    assert!(
        !output.status.success(),
        "invalid checked IR build succeeded: {diagnostic}"
    );
    assert!(
        diagnostic.contains("error: IR Generation Error:"),
        "invalid build lost exact IR Generation identity: {diagnostic}"
    );
    assert!(
        !invocation_log.exists(),
        "invalid build reached the external LLVM verifier"
    );
    assert!(
        !diagnostic.contains("Advanced graph compilation")
            && !diagnostic.contains("Optimized code generation"),
        "invalid build reached graph/backend processing: {diagnostic}"
    );
    assert!(!artifact.exists(), "invalid build published LLVM output");
    assert_workspace_unchanged(&workspace, &before, "invalid checked-IR build");
}

#[test]
fn check_preserves_existing_raw_semantic_diagnostic_text() {
    let workspace = TestWorkspace::new("check-semantic-text");
    let source_path = workspace.path("semantic.aero");
    fs::write(
        &source_path,
        "struct Point { x: int } fn main() { let point = Point { x: 1 }; }",
    )
    .expect("write semantic check source");
    let missing = workspace.path("missing-opt-22");
    let invocation_log = workspace.path("semantic-check-invocations.log");
    let output = run_check_with_verifier(
        &workspace,
        &source_path,
        &missing,
        &missing,
        &invocation_log,
    );
    let diagnostic = diagnostics(&output);
    assert!(
        !output.status.success(),
        "semantic check succeeded: {diagnostic}"
    );
    assert!(
        diagnostic
            .lines()
            .any(|line| line.trim() == "error: Struct construction expressions are not supported."),
        "check changed its existing semantic text: {diagnostic}"
    );
    assert!(
        !diagnostic.contains("Semantic Analysis Error:"),
        "check gained the build-only semantic prefix: {diagnostic}"
    );
    assert_only_source_file_exists(&workspace, &source_path);
}
