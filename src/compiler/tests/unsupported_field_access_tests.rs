use compiler::ast::{AstNode, ComparisonOp, Expression, Statement};
use compiler::{
    CompilerOptions, compile_program, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_INNER_DIAGNOSTIC: &str = "Field access expressions are not supported.";
const TUPLE_DIAGNOSTIC: &str = "Tuple expressions are not supported.";
const VOID_RECEIVER_DIAGNOSTIC: &str = "Error: void function `sink` cannot be used as a value.";

const ADJACENT_CONTROLS_SOURCE: &str = r#"
struct Point { x: int, y: int }

fn main() {
    let point = Point { x: 7, y: 9 };
    let grouped: int = (7);
    let added: int = grouped + 2;
    let compared = added > grouped;
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
            "aero-unsupported-field-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh field-access test workspace");
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
            .is_some_and(|name| name.starts_with("aero-unsupported-field-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn public_field_rejection_failure(label: &str, source: &str) -> Option<String> {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => Some(format!(
            "{label}: compile_program unwound instead of returning the field diagnostic"
        )),
        Ok(Ok(llvm)) => Some(format!(
            "{label}: unexpectedly accepted; zero marker={}, field GEP marker={}, receiver call marker={}",
            llvm.contains("0x0000000000000000") || llvm.contains("ret i32 0"),
            llvm.contains("GetFieldPtr") || llvm.contains("getelementptr inbounds %struct"),
            llvm.contains("call i32 @make")
        )),
        Ok(Err(error)) => {
            let Some(inner) = error.strip_prefix("Semantic Analysis Error: ") else {
                return Some(format!(
                    "{label}: unexpected public error category: {error}"
                ));
            };
            (inner != EXPECTED_INNER_DIAGNOSTIC).then(|| {
                format!("{label}: expected `{EXPECTED_INNER_DIAGNOSTIC}`, received `{inner}`")
            })
        }
    }
}

fn assert_public_semantic_error(source: &str, expected: &str) {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => panic!("compile_program unwound instead of returning `{expected}`"),
        Ok(Ok(llvm)) => panic!("compile_program unexpectedly accepted source:\n{llvm}"),
        Ok(Err(error)) => {
            let inner = error
                .strip_prefix("Semantic Analysis Error: ")
                .unwrap_or_else(|| panic!("unexpected public error category: {error}"));
            assert_eq!(inner, expected);
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

fn cli_field_rejection_failures(
    label: &str,
    output: &Output,
    artifact: Option<&Path>,
) -> Vec<String> {
    let diagnostics = combined_output(output);
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!("{label}: field source exited zero: {diagnostics}"));
    }
    if diagnostics.to_ascii_lowercase().contains("panicked") {
        failures.push(format!(
            "{label}: field source reached a panic: {diagnostics}"
        ));
    }
    if !diagnostics.contains(EXPECTED_INNER_DIAGNOSTIC) {
        failures.push(format!(
            "{label}: diagnostic `{diagnostics}` missing `{EXPECTED_INNER_DIAGNOSTIC}`"
        ));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        failures.push(format!(
            "{label}: field source created requested artifact {}",
            artifact.display()
        ));
    }
    failures
}

#[test]
fn parser_retains_field_access_and_distinguishes_method_and_tuple_projection() {
    let tokens = try_tokenize_with_locations(
        "let field = base.value; let method = base.value(); let tuple = (1, 2).0;",
        None,
    )
    .expect("dot-expression controls should lex");
    let ast = parse_with_locations(tokens).expect("dot-expression controls should parse");
    assert_eq!(ast.len(), 3, "unexpected dot-expression AST: {ast:?}");

    let AstNode::Statement(Statement::Let {
        value: Some(Expression::FieldAccess { object, field }),
        ..
    }) = &ast[0]
    else {
        panic!("expected named field access: {:?}", ast[0]);
    };
    assert_eq!(field, "value");
    assert!(matches!(object.as_ref(), Expression::Identifier(name) if name == "base"));

    let AstNode::Statement(Statement::Let {
        value:
            Some(Expression::MethodCall {
                object,
                method,
                arguments,
            }),
        ..
    }) = &ast[1]
    else {
        panic!("expected method call: {:?}", ast[1]);
    };
    assert_eq!(method, "value");
    assert!(arguments.is_empty());
    assert!(matches!(object.as_ref(), Expression::Identifier(name) if name == "base"));

    let AstNode::Statement(Statement::Let {
        value: Some(Expression::TupleIndex { object, index }),
        ..
    }) = &ast[2]
    else {
        panic!("expected tuple projection: {:?}", ast[2]);
    };
    assert_eq!(*index, 0);
    assert!(matches!(object.as_ref(), Expression::TupleLiteral(elements) if elements.len() == 2));
}

#[test]
fn public_compile_preserves_grouped_numeric_comparison_struct_and_array_controls() {
    let tokens = try_tokenize_with_locations(ADJACENT_CONTROLS_SOURCE, None)
        .expect("adjacent controls should lex");
    let ast = parse_with_locations(tokens).expect("adjacent controls should parse");
    assert!(matches!(
        &ast[0],
        AstNode::Statement(Statement::StructDef { name, .. }) if name == "Point"
    ));
    assert!(ast.iter().any(|node| matches!(
        node,
        AstNode::Statement(Statement::Function { body, .. })
            if body.statements.iter().any(|statement| matches!(
                statement,
                Statement::Let { value: Some(Expression::StructLiteral { name, .. }), .. }
                    if name == "Point"
            ))
    )));

    let llvm = compile_program(ADJACENT_CONTROLS_SOURCE, CompilerOptions::default())
        .expect("adjacent controls should retain their pre-task behavior");
    for marker in [
        "fadd double",
        "icmp sgt",
        "alloca [2 x double]",
        "getelementptr inbounds [2 x double]",
    ] {
        assert!(llvm.contains(marker), "LLVM missing `{marker}`:\n{llvm}");
    }

    let comparison_tokens = try_tokenize_with_locations("let value = 2 > 1;", None)
        .expect("numeric comparison should lex");
    let comparison_ast =
        parse_with_locations(comparison_tokens).expect("numeric comparison should parse");
    assert!(matches!(
        &comparison_ast[0],
        AstNode::Statement(Statement::Let {
            value: Some(Expression::Comparison {
                op: ComparisonOp::GreaterThan,
                ..
            }),
            ..
        })
    ));
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
fn tuple_receiver_diagnostic_retains_precedence() {
    assert_public_semantic_error(
        "fn main() { let value: int = (1, 2).field; }",
        TUPLE_DIAGNOSTIC,
    );
}

#[test]
fn void_call_receiver_diagnostic_retains_precedence() {
    assert_public_semantic_error(
        "fn sink() {} fn main() { let value: int = sink().field; }",
        VOID_RECEIVER_DIAGNOSTIC,
    );
}

#[test]
fn public_compile_rejects_field_access_matrix_without_unwinding() {
    let cases = [
        ("literal", "fn main() { let value: int = 7.field; }"),
        (
            "bound",
            "fn main() { let base: int = 7; let value: int = base.field; }",
        ),
        (
            "undeclared",
            "fn main() { let value: int = missing.field; }",
        ),
        (
            "call-receiver",
            "fn make() -> int { 7 } fn main() { let value: int = make().field; }",
        ),
        (
            "struct-literal-receiver",
            "struct Point { x: int } fn main() { let value: int = Point { x: 7 }.x; }",
        ),
        ("chained", "fn main() { let value: int = 7.first.second; }"),
        ("root", "7.field;"),
        ("discarded", "fn main() { 7.field; }"),
        (
            "explicit-return",
            "fn project() -> int { return 7.field; } fn main() { let value: int = project(); }",
        ),
        (
            "tail-return",
            "fn project() -> int { 7.field } fn main() { let value: int = project(); }",
        ),
        (
            "non-first-array-element",
            "fn main() { let values = [1, 7.field]; }",
        ),
        (
            "closure-body",
            "fn main() { let project = |value: int| value.field; let result: int = project(7); }",
        ),
        (
            "nested-call-argument",
            "fn identity(value: int) -> int { value } fn main() { let result: int = identity(7.field); }",
        ),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| public_field_rejection_failure(label, source))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn cli_check_and_build_reject_root_field_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("root");
    let source = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&source, "fn main() { let value: int = 7.field; }").expect("write root field source");

    let check = run_check(&workspace, &source);
    let build = run_build(&workspace, &source, &artifact);
    let mut failures = cli_field_rejection_failures("root check", &check, None);
    failures.extend(cli_field_rejection_failures(
        "root build",
        &build,
        Some(&artifact),
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn cli_check_and_build_reject_direct_module_field_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("module");
    let root = workspace.path("main.aero");
    let module = workspace.path("fields.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&root, "mod fields; fn main() {}").expect("write direct-module root source");
    fs::write(&module, "fn project() -> int { return missing.field; }")
        .expect("write direct module with undeclared receiver field access");

    let check = run_check(&workspace, &root);
    let build = run_build(&workspace, &root, &artifact);
    let mut failures = cli_field_rejection_failures("module check", &check, None);
    failures.extend(cli_field_rejection_failures(
        "module build",
        &build,
        Some(&artifact),
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
