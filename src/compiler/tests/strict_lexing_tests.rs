use compiler::{CompilerOptions, Token, compile_program, tokenize};
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

fn assert_lex_error(output: &Output, filename: &str, column: usize, detail: &str) {
    let diagnostics = combined_output(output);
    assert!(
        !output.status.success(),
        "lexically invalid command must fail: {diagnostics}"
    );
    assert!(
        diagnostics.contains("Lex error"),
        "missing lexical category: {diagnostics}"
    );
    assert!(
        diagnostics.contains(detail),
        "missing lexical detail `{detail}`: {diagnostics}"
    );
    assert!(
        diagnostics.contains(&format!("{filename}:1:{column}")),
        "missing located filename/line/column: {diagnostics}"
    );
}

#[test]
fn compile_program_rejects_each_lexical_corruption() {
    let cases = [
        ("let value = 1@;", "Unexpected character '@'", "1:14"),
        (
            "let value = 9223372036854775808;",
            "Invalid number format '9223372036854775808'",
            "1:13",
        ),
        ("let value = 1e+;", "Invalid number format '1e+'", "1:13"),
        (
            "let value = 1e9999;",
            "Invalid number format '1e9999'",
            "1:13",
        ),
        (
            "let value = \"unterminated",
            "Unterminated string literal",
            "1:13",
        ),
        (
            "let value = f\"unterminated",
            "Unterminated string literal",
            "1:13",
        ),
        ("println!(f\"{1@}\");", "Unexpected character '@'", "1:2"),
    ];

    for (source, detail, location) in cases {
        let error = compile_program(source, CompilerOptions::default())
            .expect_err("lexically invalid source must not compile");
        assert!(
            error.contains("Lex error") || source.contains("{1@}"),
            "missing lexical category for `{source}`: {error}"
        );
        assert!(
            error.contains(detail),
            "missing lexical detail for `{source}`: {error}"
        );
        assert!(
            error.contains(location),
            "missing lexical location for `{source}`: {error}"
        );
    }
}

#[test]
fn legacy_tokenize_retains_recovery_behavior() {
    let unexpected = tokenize("let value = 1@;");
    assert!(unexpected.contains(&Token::IntegerLiteral(1)));

    let overflow = tokenize("let value = 9223372036854775808;");
    assert!(overflow.contains(&Token::IntegerLiteral(0)));

    let unterminated = tokenize("let value = \"unterminated");
    assert!(unterminated.contains(&Token::StringLiteral("unterminated".to_string())));
}

#[test]
fn build_rejects_unexpected_character_without_writing_llvm() {
    let workspace = TestWorkspace::new("strict_lex_build_root");
    let source_path = workspace.path("unexpected.aero");
    let output_path = workspace.path("unexpected.ll");
    fs::write(&source_path, "let value = 1@;").expect("write invalid root source");

    let output = run_aero(
        &workspace,
        &[
            Path::new("build"),
            &source_path,
            Path::new("-o"),
            &output_path,
        ],
    );

    assert_lex_error(&output, "unexpected.aero", 14, "Unexpected character '@'");
    assert!(!output_path.exists(), "failed build wrote LLVM IR");
}

#[test]
fn build_rejects_overflow_in_direct_module_without_writing_llvm() {
    let workspace = TestWorkspace::new("strict_lex_build_module");
    let source_path = workspace.path("main.aero");
    let module_path = workspace.path("broken.aero");
    let output_path = workspace.path("module.ll");
    fs::write(&source_path, "mod broken;").expect("write module root source");
    fs::write(&module_path, "let value = 9223372036854775808;")
        .expect("write invalid direct module");

    let output = run_aero(
        &workspace,
        &[
            Path::new("build"),
            &source_path,
            Path::new("-o"),
            &output_path,
        ],
    );

    assert_lex_error(
        &output,
        "broken.aero",
        13,
        "Invalid number format '9223372036854775808'",
    );
    assert!(!output_path.exists(), "failed module build wrote LLVM IR");
}

#[test]
fn check_rejects_malformed_exponent_with_failure_status() {
    let workspace = TestWorkspace::new("strict_lex_check");
    let source_path = workspace.path("exponent.aero");
    fs::write(&source_path, "let value = 1e+;").expect("write invalid check source");

    let output = run_aero(&workspace, &[Path::new("check"), &source_path]);

    assert_lex_error(&output, "exponent.aero", 13, "Invalid number format '1e+'");
}

#[test]
fn discovered_test_rejects_non_finite_float_with_failure_status() {
    let workspace = TestWorkspace::new("strict_lex_test");
    let source_path = workspace.path("float_test.aero");
    fs::write(&source_path, "let value = 1e9999;").expect("write invalid discovered test");

    let output = run_aero(&workspace, &[Path::new("test")]);

    assert_lex_error(
        &output,
        "float_test.aero",
        13,
        "Invalid number format '1e9999'",
    );
}

#[test]
fn check_and_discovered_test_reject_lexical_error_in_direct_module() {
    let check_workspace = TestWorkspace::new("strict_lex_check_module");
    let check_root = check_workspace.path("main.aero");
    let check_module = check_workspace.path("broken.aero");
    fs::write(&check_root, "mod broken;").expect("write check module root");
    fs::write(&check_module, "let value = 1@;").expect("write invalid check module");

    let check_output = run_aero(&check_workspace, &[Path::new("check"), &check_root]);
    assert_lex_error(&check_output, "broken.aero", 14, "Unexpected character '@'");

    let test_workspace = TestWorkspace::new("strict_lex_test_module");
    let test_root = test_workspace.path("module_test.aero");
    let test_module = test_workspace.path("broken.aero");
    fs::write(&test_root, "mod broken;").expect("write discovered test module root");
    fs::write(&test_module, "let value = 1@;").expect("write invalid discovered test module");

    let test_output = run_aero(&test_workspace, &[Path::new("test")]);
    assert_lex_error(&test_output, "broken.aero", 14, "Unexpected character '@'");
}

#[test]
fn run_rejects_lexical_error_before_native_tools_and_cleans_nonce_directory() {
    let workspace = TestWorkspace::new("strict_lex_run");
    let source_path = workspace.path("run-invalid.aero");
    fs::write(&source_path, "let value = 1@;").expect("write invalid run source");

    let output = run_aero(&workspace, &[Path::new("run"), &source_path]);

    assert_lex_error(&output, "run-invalid.aero", 14, "Unexpected character '@'");
    let diagnostics = combined_output(&output);
    for native_tool_marker in ["clang", "llc", "Program executed successfully"] {
        assert!(
            !diagnostics.contains(native_tool_marker),
            "lexically invalid run reached native tooling: {diagnostics}"
        );
    }

    let run_root = workspace.path("target").join("aero-run");
    if run_root.exists() {
        assert!(
            fs::read_dir(&run_root)
                .expect("read run artifact root")
                .next()
                .is_none(),
            "failed run left a per-invocation nonce directory"
        );
    }
}

#[test]
fn profile_rejects_unterminated_f_string_without_writing_trace() {
    let workspace = TestWorkspace::new("strict_lex_profile");
    let source_path = workspace.path("profile-invalid.aero");
    let trace_path = workspace.path("profile.json");
    fs::write(&source_path, "let value = f\"unterminated").expect("write invalid profile source");

    let output = run_aero(
        &workspace,
        &[
            Path::new("profile"),
            &source_path,
            Path::new("-o"),
            &trace_path,
        ],
    );

    assert_lex_error(
        &output,
        "profile-invalid.aero",
        13,
        "Unterminated string literal",
    );
    assert!(!trace_path.exists(), "failed profile wrote a trace");

    let module_workspace = TestWorkspace::new("strict_lex_profile_module");
    let module_root = module_workspace.path("main.aero");
    let module_source = module_workspace.path("broken.aero");
    let module_trace = module_workspace.path("module-profile.json");
    fs::write(&module_root, "mod broken;").expect("write profile module root");
    fs::write(&module_source, "let value = 1e9999;").expect("write invalid profile module");

    let module_output = run_aero(
        &module_workspace,
        &[
            Path::new("profile"),
            &module_root,
            Path::new("-o"),
            &module_trace,
        ],
    );
    assert_lex_error(
        &module_output,
        "broken.aero",
        13,
        "Invalid number format '1e9999'",
    );
    assert!(
        !module_trace.exists(),
        "failed module profile wrote a trace"
    );
}

#[test]
fn doc_rejects_unterminated_string_without_writing_markdown() {
    let workspace = TestWorkspace::new("strict_lex_doc");
    let source_path = workspace.path("doc-invalid.aero");
    let markdown_path = workspace.path("doc.md");
    fs::write(&source_path, "let value = \"unterminated").expect("write invalid doc source");

    let output = run_aero(
        &workspace,
        &[
            Path::new("doc"),
            &source_path,
            Path::new("-o"),
            &markdown_path,
        ],
    );

    assert_lex_error(
        &output,
        "doc-invalid.aero",
        13,
        "Unterminated string literal",
    );
    assert!(!markdown_path.exists(), "failed doc wrote Markdown");
}
