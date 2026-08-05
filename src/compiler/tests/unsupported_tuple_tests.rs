use compiler::ast::{AstNode, Expression, Statement};
use compiler::{
    CompilerOptions, compile_program, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const PRESERVED_TUPLE_DIAGNOSTIC: &str = "Tuple expressions are not supported.";

const ADJACENT_CONTROLS_SOURCE: &str = r#"
fn main() {
    let grouped: int = (7);
    let added: int = grouped + 1;
    let values = [11, 22];
    let selected: int = values[1];
}
"#;

const ITER_CONTROL_SOURCE: &str = r#"
fn main() {
    let values = [1, 2];
    for item in values.iter() {
        let copied: int = item;
    }
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
            "aero-unsupported-tuple-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh tuple test workspace");
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
            .is_some_and(|name| name.starts_with("aero-unsupported-tuple-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn assert_public_tuple_rejection(source: &str, expected: &str) {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => panic!("compile_program unwound instead of returning the tuple diagnostic"),
        Ok(Ok(llvm)) => panic!(
            "compile_program unexpectedly accepted tuple; fabricated-zero evidence: {}\n{llvm}",
            llvm.contains("0x0000000000000000") || llvm.contains("ret i32 0")
        ),
        Ok(Err(error)) => assert!(
            error.contains(expected),
            "diagnostic {error:?} missing {expected:?}"
        ),
    }
}

fn assert_public_tuple_success(source: &str, expected_llvm: &str) {
    let llvm = compile_program(source, CompilerOptions::default())
        .unwrap_or_else(|error| panic!("admitted flat tuple source failed: {error}"));
    assert!(
        llvm.contains(expected_llvm),
        "tuple LLVM missing {expected_llvm:?}:\n{llvm}"
    );
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

fn assert_cli_tuple_rejection(output: &Output, artifact: Option<&Path>, expected: &str) {
    let diagnostics = combined_output(output);
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!("tuple source exited zero: {diagnostics}"));
    }
    if diagnostics.to_ascii_lowercase().contains("panicked") {
        failures.push(format!("tuple source reached a panic: {diagnostics}"));
    }
    if !diagnostics.contains(expected) {
        failures.push(format!("diagnostic `{diagnostics}` missing `{expected}`"));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        failures.push(format!(
            "tuple source created requested artifact {}",
            artifact.display()
        ));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn parser_preserves_tuple_literal_and_projection() {
    let tokens =
        try_tokenize_with_locations("let literal = (1, 2); let projection = (3, 4).1;", None)
            .expect("tuple source should lex");
    let ast = parse_with_locations(tokens).expect("tuple source should parse");
    assert_eq!(ast.len(), 2, "unexpected tuple AST: {ast:?}");

    let AstNode::Statement(Statement::Let {
        value: Some(Expression::TupleLiteral(elements)),
        ..
    }) = &ast[0]
    else {
        panic!("expected tuple literal: {:?}", ast[0]);
    };
    assert_eq!(elements.len(), 2);

    let AstNode::Statement(Statement::Let {
        value: Some(Expression::TupleIndex { object, index }),
        ..
    }) = &ast[1]
    else {
        panic!("expected tuple projection: {:?}", ast[1]);
    };
    assert_eq!(*index, 1);
    assert!(matches!(object.as_ref(), Expression::TupleLiteral(elements) if elements.len() == 2));
}

#[test]
fn public_compile_preserves_grouped_scalar_numeric_array_and_index_controls() {
    let llvm = compile_program(ADJACENT_CONTROLS_SOURCE, CompilerOptions::default())
        .expect("grouping, arithmetic, arrays, and indexing should remain executable");
    for marker in [
        "fadd double",
        "alloca [2 x double]",
        "getelementptr inbounds [2 x double]",
    ] {
        assert!(llvm.contains(marker), "LLVM missing `{marker}`:\n{llvm}");
    }
}

#[test]
fn cli_check_and_build_preserve_array_iter_control() {
    let workspace = TestWorkspace::new("iter-control");
    let source = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&source, ITER_CONTROL_SOURCE).expect("write iterator control source");

    let check = run_check(&workspace, &source);
    assert!(
        check.status.success(),
        "array iterator check failed: {}",
        combined_output(&check)
    );

    let build = run_build(&workspace, &source, &artifact);
    assert!(
        build.status.success(),
        "array iterator build failed: {}",
        combined_output(&build)
    );
    let llvm = fs::read_to_string(&artifact).expect("read iterator control artifact");
    assert!(
        llvm.contains("getelementptr inbounds [2 x double]"),
        "{llvm}"
    );
    assert!(llvm.contains("icmp slt"), "{llvm}");
}

#[test]
fn public_compile_rejects_direct_tuple_literal_without_unwinding() {
    assert_public_tuple_rejection(
        "fn main() { let value: int = (11, 22); }",
        "tuple binding annotation mismatch: expected int, actual (int, int)",
    );
}

#[test]
fn public_compile_accepts_flat_tuple_projection() {
    assert_public_tuple_success(
        "fn main() { let value: int = (11, 22).1; }",
        "getelementptr inbounds { double, double }",
    );
}

#[test]
fn public_compile_rejects_tuple_explicit_return_without_unwinding() {
    assert_public_tuple_rejection(
        "fn tuple_value() -> int { return (11, 22); } fn main() { let value: int = tuple_value(); }",
        "return type mismatch: expected int, actual (int, int)",
    );
}

#[test]
fn public_compile_rejects_tuple_tail_return_without_unwinding() {
    assert_public_tuple_rejection(
        "fn tuple_value() -> int { (11, 22) } fn main() { let value: int = tuple_value(); }",
        "return type mismatch: expected int, actual (int, int)",
    );
}

#[test]
fn public_compile_accepts_discarded_flat_tuple() {
    assert_public_tuple_success("fn main() { (11, 22); }", "alloca { double, double }");
}

#[test]
fn public_compile_rejects_root_tuple_without_unwinding() {
    assert_public_tuple_rejection("let value: int = (11, 22);", PRESERVED_TUPLE_DIAGNOSTIC);
}

#[test]
fn tuple_children_are_analyzed_before_flat_tuple_classification() {
    assert_public_tuple_rejection(
        "fn main() { let value: int = (missing, 22); }",
        "Use of undeclared variable `missing`",
    );
}

#[test]
fn public_compile_rejects_tuple_in_non_first_array_element_without_unwinding() {
    assert_public_tuple_rejection(
        "fn main() { let values = [1, (2, 3)]; }",
        "array element type mismatch: expected int, actual (int, int)",
    );
}

#[test]
fn public_compile_rejects_tuple_in_closure_body_without_unwinding() {
    assert_public_tuple_rejection(
        "fn main() { let make_tuple = |value: int| (value, 2); let result: int = make_tuple(1); }",
        PRESERVED_TUPLE_DIAGNOSTIC,
    );
}

#[test]
fn cli_check_rejects_root_tuple() {
    let workspace = TestWorkspace::new("root-check");
    let source = workspace.path("main.aero");
    fs::write(&source, "fn main() { let value: int = (11, 22); }")
        .expect("write root tuple source");

    let output = run_check(&workspace, &source);
    assert_cli_tuple_rejection(
        &output,
        None,
        "tuple binding annotation mismatch: expected int, actual (int, int)",
    );
}

#[test]
fn cli_build_accepts_root_flat_tuple_projection() {
    let workspace = TestWorkspace::new("root-build");
    let source = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&source, "fn main() { let value: int = (11, 22).1; }")
        .expect("write root tuple source");

    let output = run_build(&workspace, &source, &artifact);
    assert!(
        output.status.success() && artifact.is_file(),
        "flat tuple projection build failed: {}",
        combined_output(&output)
    );
    let llvm = fs::read_to_string(&artifact).expect("read tuple projection artifact");
    assert!(
        llvm.contains("getelementptr inbounds { double, double }"),
        "{llvm}"
    );
}

#[test]
fn cli_check_accepts_direct_module_flat_tuple_projection() {
    let workspace = TestWorkspace::new("module-check");
    let root = workspace.path("main.aero");
    let module = workspace.path("tuple_values.aero");
    fs::write(&root, "mod tuple_values; fn main() {}").expect("write direct-module root source");
    fs::write(&module, "fn tuple_value() -> int { return (11, 22).1; }")
        .expect("write direct module with tuple");

    let output = run_check(&workspace, &root);
    assert!(
        output.status.success(),
        "direct-module flat tuple check failed: {}",
        combined_output(&output)
    );
}

#[test]
fn cli_build_rejects_direct_module_tuple_without_panicking_or_artifact() {
    let workspace = TestWorkspace::new("module-build");
    let root = workspace.path("main.aero");
    let module = workspace.path("tuple_values.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&root, "mod tuple_values; fn main() {}").expect("write direct-module root source");
    fs::write(&module, "fn tuple_value() -> int { return (11, 22); }")
        .expect("write direct module with tuple");

    let output = run_build(&workspace, &root, &artifact);
    assert_cli_tuple_rejection(
        &output,
        Some(&artifact),
        "return type mismatch: expected int, actual (int, int)",
    );
}
