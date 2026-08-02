use compiler::ast::{AstNode, BinaryOp, Expression, Statement};
use compiler::{
    CompilerOptions, compile_program, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_INNER_DIAGNOSTIC: &str = "Binary operator `%` is not supported.";

const ADJACENT_ARITHMETIC_SOURCE: &str = r#"
fn main() {
    let left: int = 12;
    let right: int = 3;
    let added: int = left + right;
    let subtracted: int = left - right;
    let multiplied: int = left * right;
    let divided: int = left / right;
}
"#;

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
            "aero-unsupported-modulo-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh modulo test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-unsupported-modulo-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn assert_public_modulo_rejection(source: &str) {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => panic!("compile_program unwound instead of returning the modulo diagnostic"),
        Ok(Ok(_)) => panic!("compile_program unexpectedly accepted modulo"),
        Ok(Err(error)) => {
            let inner = error
                .strip_prefix("Semantic Analysis Error: ")
                .unwrap_or_else(|| panic!("unexpected public error category: {error}"));
            assert_eq!(inner, EXPECTED_INNER_DIAGNOSTIC);
        }
    }
}

fn run_check(workspace: &TestWorkspace, input: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("check")
        .arg(input)
        .current_dir(&workspace.root)
        .output()
        .expect("run aero check")
}

fn run_build(workspace: &TestWorkspace, input: &Path, artifact: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(input)
        .arg("-o")
        .arg(artifact)
        .current_dir(&workspace.root)
        .output()
        .expect("run aero build")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_cli_modulo_rejection(output: &Output, artifact: Option<&Path>) {
    let diagnostics = combined_output(output);
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!("modulo source exited zero: {diagnostics}"));
    }
    if diagnostics.to_ascii_lowercase().contains("panicked") {
        failures.push(format!("modulo source reached a panic: {diagnostics}"));
    }
    if !diagnostics.contains(EXPECTED_INNER_DIAGNOSTIC) {
        failures.push(format!(
            "diagnostic `{diagnostics}` missing `{EXPECTED_INNER_DIAGNOSTIC}`"
        ));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        failures.push(format!(
            "modulo source created requested artifact {}",
            artifact.display()
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn parser_preserves_modulo_at_multiplicative_precedence() {
    let tokens = try_tokenize_with_locations("let value = 20 + 9 % 4 * 2;", None)
        .expect("modulo source should lex");
    let ast = parse_with_locations(tokens).expect("modulo source should parse");
    let AstNode::Statement(Statement::Let {
        value: Some(expression),
        ..
    }) = &ast[0]
    else {
        panic!("expected a top-level let statement: {ast:?}");
    };
    let Expression::Binary {
        op: BinaryOp::Add,
        left,
        right,
        ..
    } = expression
    else {
        panic!("expected addition at the expression root: {expression:?}");
    };
    assert!(matches!(left.as_ref(), Expression::IntegerLiteral(20)));
    let Expression::Binary {
        op: BinaryOp::Multiply,
        left,
        right,
        ..
    } = right.as_ref()
    else {
        panic!("expected multiplication on the right of addition: {right:?}");
    };
    assert!(matches!(right.as_ref(), Expression::IntegerLiteral(2)));
    let Expression::Binary {
        op: BinaryOp::Modulo,
        left,
        right,
        ..
    } = left.as_ref()
    else {
        panic!("expected modulo before multiplication: {left:?}");
    };
    assert!(matches!(left.as_ref(), Expression::IntegerLiteral(9)));
    assert!(matches!(right.as_ref(), Expression::IntegerLiteral(4)));
}

#[test]
fn public_compile_preserves_adjacent_arithmetic() {
    let llvm = compile_program(ADJACENT_ARITHMETIC_SOURCE, CompilerOptions::default())
        .expect("adjacent arithmetic should remain executable");
    for marker in ["fadd double", "fsub double", "fmul double", "fdiv double"] {
        assert!(llvm.contains(marker), "LLVM missing `{marker}`:\n{llvm}");
    }
}

#[test]
fn cli_check_and_build_preserve_adjacent_arithmetic() {
    let workspace = TestWorkspace::new("adjacent-controls");
    let source = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&source, ADJACENT_ARITHMETIC_SOURCE).expect("write adjacent arithmetic source");

    let check = run_check(&workspace, &source);
    assert!(
        check.status.success(),
        "adjacent arithmetic check failed: {}",
        combined_output(&check)
    );

    let build = run_build(&workspace, &source, &artifact);
    assert!(
        build.status.success(),
        "adjacent arithmetic build failed: {}",
        combined_output(&build)
    );
    let llvm = fs::read_to_string(&artifact).expect("read adjacent arithmetic artifact");
    for marker in ["fadd double", "fsub double", "fmul double", "fdiv double"] {
        assert!(llvm.contains(marker), "LLVM missing `{marker}`:\n{llvm}");
    }
}

#[test]
fn public_compile_rejects_integer_literal_modulo_without_unwinding() {
    assert_public_modulo_rejection("fn main() { let value: int = 5 % 2; }");
}

#[test]
fn public_compile_rejects_integer_identifier_modulo_in_function_without_unwinding() {
    assert_public_modulo_rejection(
        "fn remainder(left: int, right: int) -> int { return left % right; } fn main() { let value: int = remainder(5, 2); }",
    );
}

#[test]
fn public_compile_rejects_float_modulo_without_unwinding() {
    assert_public_modulo_rejection("fn main() { let value: float = 5.5 % 2.0; }");
}

#[test]
fn public_compile_rejects_mixed_modulo_without_unwinding() {
    assert_public_modulo_rejection("fn main() { let value: float = 5.5 % 2; }");
}

#[test]
fn public_compile_rejects_zero_rhs_modulo_without_unwinding() {
    assert_public_modulo_rejection("fn main() { let value: int = 5 % 0; }");
}

#[test]
fn public_compile_rejects_nested_modulo_comparison_without_unwinding() {
    assert_public_modulo_rejection("fn main() { let value = (5 % 2) == 1; }");
}

#[test]
fn public_compile_rejects_root_modulo_without_unwinding() {
    assert_public_modulo_rejection("let value: int = 5 % 2;");
}

#[test]
fn cli_check_rejects_root_modulo() {
    let workspace = TestWorkspace::new("root-check");
    let source = workspace.path("main.aero");
    fs::write(&source, "fn main() { let value: int = 5 % 2; }").expect("write root modulo source");

    let output = run_check(&workspace, &source);
    assert_cli_modulo_rejection(&output, None);
}

#[test]
fn cli_build_rejects_root_modulo_without_panicking_or_artifact() {
    let workspace = TestWorkspace::new("root-build");
    let source = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&source, "fn main() { let value: int = 5 % 2; }").expect("write root modulo source");

    let output = run_build(&workspace, &source, &artifact);
    assert_cli_modulo_rejection(&output, Some(&artifact));
}

#[test]
fn cli_check_rejects_direct_module_modulo() {
    let workspace = TestWorkspace::new("module-check");
    let root = workspace.path("main.aero");
    let module = workspace.path("arithmetic.aero");
    fs::write(&root, "mod arithmetic; fn main() {}").expect("write direct-module root source");
    fs::write(
        &module,
        "fn remainder(left: int, right: int) -> int { return left % right; }",
    )
    .expect("write direct module with modulo");

    let output = run_check(&workspace, &root);
    assert_cli_modulo_rejection(&output, None);
}

#[test]
fn cli_build_rejects_direct_module_modulo_without_panicking_or_artifact() {
    let workspace = TestWorkspace::new("module-build");
    let root = workspace.path("main.aero");
    let module = workspace.path("arithmetic.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&root, "mod arithmetic; fn main() {}").expect("write direct-module root source");
    fs::write(
        &module,
        "fn remainder(left: int, right: int) -> int { return left % right; }",
    )
    .expect("write direct module with modulo");

    let output = run_build(&workspace, &root, &artifact);
    assert_cli_modulo_rejection(&output, Some(&artifact));
}
