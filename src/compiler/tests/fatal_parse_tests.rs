use compiler::{CompilerOptions, compile_program};
use std::fs;
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
            "aero_{test_name}_{}_{}_{}",
            std::process::id(),
            timestamp,
            sequence
        ));
        fs::create_dir(&root).expect("create fresh test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn write_malformed_source(workspace: &TestWorkspace) -> PathBuf {
    let source_path = workspace.path("malformed.aero");
    fs::write(&source_path, "let = ;").expect("write malformed Aero source");
    source_path
}

fn run_aero(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.current_dir(&workspace.root);
    for arg in args {
        command.arg(arg);
    }
    command.output().expect("run Aero CLI")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_located_parse_error(output: &Output, filename: &str) {
    let diagnostics = combined_output(output);
    assert!(
        diagnostics.contains("Parse error"),
        "missing parse category: {diagnostics}"
    );
    assert!(
        diagnostics.contains("Expected variable name, found Assign"),
        "missing expected/found context: {diagnostics}"
    );
    assert!(
        diagnostics.contains(&format!("{filename}:1:5")),
        "missing located filename/line/column: {diagnostics}"
    );
}

#[test]
fn compile_program_rejects_malformed_syntax() {
    let error = compile_program("let = ;", CompilerOptions::default())
        .expect_err("malformed syntax must not compile");

    assert!(error.contains("Parse error"), "unexpected error: {error}");
    assert!(
        error.contains("Expected variable name, found Assign"),
        "unexpected error: {error}"
    );
    assert!(error.contains("1:5"), "unexpected error: {error}");
}

#[test]
fn compile_program_still_accepts_a_valid_minimal_program() {
    let result = compile_program("let value = 1;", CompilerOptions::default());
    assert!(result.is_ok(), "valid source should compile: {result:?}");
}

#[test]
fn build_rejects_malformed_syntax_without_writing_llvm() {
    let workspace = TestWorkspace::new("fatal_parse_build");
    let source_path = write_malformed_source(&workspace);
    let output_path = workspace.path("fresh-output.ll");
    assert!(!output_path.exists(), "output path must begin fresh");

    let output = run_aero(
        &workspace,
        &[
            Path::new("build"),
            &source_path,
            Path::new("-o"),
            &output_path,
        ],
    );

    assert!(!output.status.success(), "malformed build must fail");
    assert_located_parse_error(&output, "malformed.aero");
    assert!(
        !output_path.exists(),
        "failed build must not create LLVM IR"
    );
    let diagnostics = combined_output(&output);
    assert!(
        !diagnostics.contains("compilation process completed successfully"),
        "failed build printed a success message: {diagnostics}"
    );
}

#[test]
fn check_rejects_malformed_syntax() {
    let workspace = TestWorkspace::new("fatal_parse_check");
    let source_path = write_malformed_source(&workspace);

    let output = run_aero(&workspace, &[Path::new("check"), &source_path]);

    assert!(!output.status.success(), "malformed check must fail");
    assert_located_parse_error(&output, "malformed.aero");
}

#[test]
fn run_rejects_malformed_syntax_before_native_tooling() {
    let workspace = TestWorkspace::new("fatal_parse_run");
    let source_path = write_malformed_source(&workspace);

    let output = run_aero(&workspace, &[Path::new("run"), &source_path]);

    assert!(!output.status.success(), "malformed run must fail");
    assert_located_parse_error(&output, "malformed.aero");

    let diagnostics = combined_output(&output);
    for native_tool_error in [
        "Error executing clang",
        "Error executing llc",
        "Error running clang",
        "Error running llc",
    ] {
        assert!(
            !diagnostics.contains(native_tool_error),
            "malformed run reached native tooling: {diagnostics}"
        );
    }

    let run_root = workspace.path("target").join("aero-run");
    if run_root.exists() {
        assert!(
            fs::read_dir(&run_root)
                .expect("read run artifact root")
                .next()
                .is_none(),
            "malformed run left a per-invocation directory under {}",
            run_root.display()
        );
    }
}

#[test]
fn build_rejects_malformed_imported_module_without_writing_llvm() {
    let workspace = TestWorkspace::new("fatal_parse_module");
    let source_path = workspace.path("main.aero");
    let module_path = workspace.path("broken.aero");
    let output_path = workspace.path("module-output.ll");
    fs::write(&source_path, "mod broken;").expect("write root Aero source");
    fs::write(&module_path, "let = ;").expect("write malformed module");
    assert!(!output_path.exists(), "output path must begin fresh");

    let output = run_aero(
        &workspace,
        &[
            Path::new("build"),
            &source_path,
            Path::new("-o"),
            &output_path,
        ],
    );

    assert!(!output.status.success(), "malformed module build must fail");
    assert_located_parse_error(&output, "broken.aero");
    assert!(
        !output_path.exists(),
        "malformed module build must not create LLVM IR"
    );
}

#[test]
fn profile_rejects_malformed_syntax_without_writing_trace() {
    let workspace = TestWorkspace::new("fatal_parse_profile");
    let source_path = write_malformed_source(&workspace);
    let trace_path = workspace.path("fresh-trace.json");
    assert!(!trace_path.exists(), "trace path must begin fresh");

    let output = run_aero(
        &workspace,
        &[
            Path::new("profile"),
            &source_path,
            Path::new("-o"),
            &trace_path,
        ],
    );

    assert!(!output.status.success(), "malformed profile must fail");
    assert_located_parse_error(&output, "malformed.aero");
    assert!(
        !trace_path.exists(),
        "malformed profile must not create a trace"
    );
}

#[test]
fn profile_rejects_malformed_imported_module_without_writing_trace() {
    let workspace = TestWorkspace::new("fatal_parse_profile_module");
    let source_path = workspace.path("main.aero");
    let module_path = workspace.path("broken.aero");
    let trace_path = workspace.path("module-trace.json");
    fs::write(&source_path, "mod broken;").expect("write profile root source");
    fs::write(&module_path, "let = ;").expect("write malformed profile module");
    assert!(!trace_path.exists(), "trace path must begin fresh");

    let output = run_aero(
        &workspace,
        &[
            Path::new("profile"),
            &source_path,
            Path::new("-o"),
            &trace_path,
        ],
    );

    assert!(
        !output.status.success(),
        "malformed profile module must fail"
    );
    assert_located_parse_error(&output, "broken.aero");
    assert!(
        !trace_path.exists(),
        "malformed profile module must not create a trace"
    );
}

#[test]
fn discovered_test_rejects_malformed_syntax_with_failure_status() {
    let workspace = TestWorkspace::new("fatal_parse_test_command");
    let source_path = workspace.path("malformed_test.aero");
    fs::write(&source_path, "let = ;").expect("write malformed discovered test");

    let output = run_aero(&workspace, &[Path::new("test")]);

    assert!(
        !output.status.success(),
        "malformed discovered test must fail the command"
    );
    assert_located_parse_error(&output, "malformed_test.aero");
}
