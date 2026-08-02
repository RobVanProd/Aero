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

fn run_aero(args: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    for arg in args {
        command.arg(arg);
    }
    command.output().expect("run Aero CLI")
}

#[test]
fn compile_program_rejects_malformed_syntax() {
    let error = compile_program("let = ;", CompilerOptions::default())
        .expect_err("malformed syntax must not compile");

    assert!(error.contains("Parse error"), "unexpected error: {error}");
    assert!(error.contains("Expected"), "unexpected error: {error}");
    assert!(error.contains("found"), "unexpected error: {error}");
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

    let output = run_aero(&[
        Path::new("build"),
        &source_path,
        Path::new("-o"),
        &output_path,
    ]);

    assert!(!output.status.success(), "malformed build must fail");
    assert!(
        !output_path.exists(),
        "failed build must not create LLVM IR"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("compilation process completed successfully"),
        "failed build printed a success message: {stdout}"
    );
}

#[test]
fn check_rejects_malformed_syntax() {
    let workspace = TestWorkspace::new("fatal_parse_check");
    let source_path = write_malformed_source(&workspace);

    let output = run_aero(&[Path::new("check"), &source_path]);

    assert!(!output.status.success(), "malformed check must fail");
}

#[test]
fn run_rejects_malformed_syntax_before_native_tooling() {
    let workspace = TestWorkspace::new("fatal_parse_run");
    let source_path = write_malformed_source(&workspace);

    let output = run_aero(&[Path::new("run"), &source_path]);

    assert!(!output.status.success(), "malformed run must fail");
}
