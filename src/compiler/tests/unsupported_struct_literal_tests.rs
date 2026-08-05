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

const EXPECTED_INNER_DIAGNOSTIC: &str = "Struct construction expressions are not supported.";
const FIELD_DIAGNOSTIC: &str = "Field access expressions are not supported.";
const MATCH_DIAGNOSTIC: &str = "Match expressions are not supported.";
const VOID_DIAGNOSTIC: &str = "Error: void function `sink` cannot be used as a value.";

const PUBLIC_DECLARATION_AND_SCALAR_CONTROL_SOURCE: &str = r#"
struct Point { x: int, y: int }
struct Container<T> { value: T }
enum Choice { Empty, Number(int) }

fn add(left: int, right: int) -> int {
    return left + right;
}

fn main() {
    let message = "ready";
    let sum: int = add(6, 2);
    let values = [11, 22];
    let selected: int = values[1];
    for item in values.iter() {
        let copied: int = item;
    }
}
"#;

const ENUM_CONSTRUCTION_SYNTAX_CONTROL_SOURCE: &str = r#"
enum Choice { Empty, Number(int) }

fn main() {
    let choice = Choice::Number(11);
    let option = Some(7);
    let result = Ok(9);
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
            "aero-unsupported-struct-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh StructLiteral test workspace");
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
            .is_some_and(|name| name.starts_with("aero-unsupported-struct-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn llvm_function_body<'a>(llvm: &'a str, signature: &str) -> Option<&'a str> {
    let start = llvm.find(signature)?;
    let remainder = &llvm[start..];
    let open = remainder.find('{')?;
    let body = &remainder[open + 1..];
    let close = body.find('}')?;
    Some(&body[..close])
}

fn red_llvm_evidence(source: &str, llvm: &str) -> String {
    let main_is_empty = llvm_function_body(llvm, "define i32 @main(").is_some_and(|body| {
        !body.lines().any(|line| {
            let line = line.trim();
            line.starts_with('%')
                || line.starts_with("store ")
                || line.starts_with("call ")
                || line.starts_with("ret ")
                || line.starts_with("br ")
        })
    });
    let defines_mark = source.contains("fn mark(");
    let dropped_mark_call =
        defines_mark && !llvm.contains("call i32 @mark(") && !llvm.contains("call double @mark(");
    let fabricated_zero = llvm.contains("0x0000000000000000") || llvm.contains("ret i32 0");
    let aggregate_text = llvm.contains("Point")
        || llvm.contains("Ghost")
        || llvm.contains("Inner")
        || llvm.contains("Outer");
    format!(
        "fabricated_zero={fabricated_zero}, empty_main={main_is_empty}, dropped_mark_call={dropped_mark_call}, aggregate_text={aggregate_text}, llvm_bytes={}",
        llvm.len()
    )
}

fn public_struct_rejection_failure(label: &str, source: &str) -> Option<String> {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => Some(format!(
            "{label}: compile_program unwound instead of returning '{EXPECTED_INNER_DIAGNOSTIC}'"
        )),
        Ok(Ok(llvm)) => Some(format!(
            "{label}: unexpectedly accepted StructLiteral; red lowering evidence: {}",
            red_llvm_evidence(source, &llvm)
        )),
        Ok(Err(error)) => {
            let Some(inner) = error.strip_prefix("Semantic Analysis Error: ") else {
                return Some(format!(
                    "{label}: unexpected public error category: {error}"
                ));
            };
            (inner != EXPECTED_INNER_DIAGNOSTIC).then(|| {
                format!("{label}: expected '{EXPECTED_INNER_DIAGNOSTIC}', received '{inner}'")
            })
        }
    }
}

fn assert_public_semantic_error(source: &str, expected: &str) {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => panic!("compile_program unwound instead of returning '{expected}'"),
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

fn strip_cli_colors(output: &str) -> String {
    output
        .replace("\x1b[1;31m", "")
        .replace("\x1b[1;32m", "")
        .replace("\x1b[1;34m", "")
        .replace("\x1b[1;36m", "")
        .replace("\x1b[0m", "")
}

fn cli_struct_rejection_failures(
    label: &str,
    output: &Output,
    expected_line: &str,
    artifact: Option<&Path>,
    source: &str,
) -> Vec<String> {
    let diagnostics = strip_cli_colors(&combined_output(output));
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!(
            "{label}: StructLiteral source exited zero: {diagnostics}"
        ));
    }
    if diagnostics.to_ascii_lowercase().contains("panicked") {
        failures.push(format!(
            "{label}: StructLiteral source reached a panic: {diagnostics}"
        ));
    }
    if !diagnostics.lines().any(|line| line.trim() == expected_line) {
        failures.push(format!(
            "{label}: expected exact CLI line '{expected_line}', received '{diagnostics}'"
        ));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        let evidence = fs::read_to_string(artifact)
            .map(|llvm| red_llvm_evidence(source, &llvm))
            .unwrap_or_else(|error| format!("artifact unreadable: {error}"));
        failures.push(format!(
            "{label}: StructLiteral source created requested artifact {}; red artifact evidence: {evidence}",
            artifact.display()
        ));
    }
    failures
}

#[test]
fn parser_retains_named_and_generic_struct_construction_shape() {
    let source = r#"
struct Point { x: int, y: int }
struct Container<T> { value: T }
fn main() {
    let point = Point { x: 7, y: 9 };
    let wrapped: Container<int> = Container { value: 11 };
}
"#;
    let tokens = try_tokenize_with_locations(source, None).expect("Struct controls should lex");
    let ast = parse_with_locations(tokens).expect("Struct controls should parse");
    assert_eq!(ast.len(), 3, "unexpected Struct control AST: {ast:?}");

    let AstNode::Statement(Statement::Function { body, .. }) = &ast[2] else {
        panic!("expected main function: {:?}", ast[2]);
    };
    let [
        Statement::Let {
            value: Some(Expression::StructLiteral { name, fields }),
            ..
        },
        Statement::Let {
            type_annotation: Some(compiler::ast::Type::Generic(type_name, type_args)),
            value:
                Some(Expression::StructLiteral {
                    name: generic_name,
                    fields: generic_fields,
                }),
            ..
        },
    ] = body.statements.as_slice()
    else {
        panic!("unexpected Struct control body: {body:?}");
    };
    assert_eq!(name, "Point");
    assert_eq!(
        fields
            .iter()
            .map(|(field, _)| field.as_str())
            .collect::<Vec<_>>(),
        ["x", "y"]
    );
    assert_eq!(type_name, "Container");
    assert_eq!(type_args.len(), 1);
    assert_eq!(generic_name, "Container");
    assert_eq!(generic_fields.len(), 1);
}

#[test]
fn parser_retains_custom_option_and_result_construction_syntax() {
    let tokens = try_tokenize_with_locations(ENUM_CONSTRUCTION_SYNTAX_CONTROL_SOURCE, None)
        .expect("Enum/Option/Result controls should lex");
    let ast = parse_with_locations(tokens).expect("Enum/Option/Result controls should parse");
    assert_eq!(ast.len(), 2, "unexpected Enum control AST: {ast:?}");
    let AstNode::Statement(Statement::Function { body, .. }) = &ast[1] else {
        panic!("expected main function: {:?}", ast[1]);
    };
    let variants = body
        .statements
        .iter()
        .filter_map(|statement| match statement {
            Statement::Let {
                value:
                    Some(Expression::EnumVariant {
                        enum_name,
                        variant,
                        data: Some(_),
                    }),
                ..
            } => Some((enum_name.as_str(), variant.as_str())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [("Choice", "Number"), ("Option", "Some"), ("Result", "Ok")]
    );
}

#[test]
fn public_compile_preserves_declarations_numeric_function_array_index_and_iter_controls() {
    let llvm = compile_program(
        PUBLIC_DECLARATION_AND_SCALAR_CONTROL_SOURCE,
        CompilerOptions::default(),
    )
    .expect("declaration-only and adjacent controls should compile");
    for marker in [
        "define i32 @add(i32 %aero.arg.left, i32 %aero.arg.right)",
        "call i32 @add(i32 6, i32 2)",
        "fadd double",
        "alloca [2 x double]",
        "getelementptr inbounds [2 x double]",
        "icmp slt",
    ] {
        assert!(llvm.contains(marker), "LLVM missing '{marker}':\n{llvm}");
    }
    assert!(
        !llvm.contains("Point") && !llvm.contains("Choice"),
        "declarations emitted runtime IR:\n{llvm}"
    );
}

#[test]
fn public_compile_rejects_ordinary_struct_construction_matrix_without_unwinding() {
    let cases = [
        ("unknown-name", "fn main() { let point = Ghost { x: 7 }; }"),
        (
            "missing-field",
            "struct Point { x: int, y: int } fn main() { let point = Point { x: 7 }; }",
        ),
        (
            "extra-field",
            "struct Point { x: int } fn main() { let point = Point { x: 7, y: 9 }; }",
        ),
        (
            "duplicate-field",
            "struct Point { x: int } fn main() { let point = Point { x: 7, x: 9 }; }",
        ),
        ("root", "struct Point { x: int } Point { x: 7 };"),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| public_struct_rejection_failure(label, source))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn public_compile_rejects_closure_parent_without_unwinding() {
    let cases = [(
        "closure-body",
        "struct Point { x: int } fn main() { let make = |x: int| Point { x: x }; }",
    )];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| public_struct_rejection_failure(label, source))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn public_compile_rejects_default_trait_and_nested_declaration_containers() {
    let cases = [
        (
            "default-trait-body",
            "struct Point { x: int } trait Make { fn make(&self) -> Point { return Point { x: 7 }; } } fn main() {}",
        ),
        (
            "nested-function",
            "struct Point { x: int } fn outer() { fn inner() -> Point { return Point { x: 7 }; } } fn main() {}",
        ),
        (
            "impl-method",
            "struct Point { x: int } impl Point { fn make() -> Point { return Point { x: 7 }; } } fn main() {}",
        ),
        (
            "nested-trait-declaration",
            "struct Point { x: int } fn outer() { trait Make { fn make(&self) -> Point { return Point { x: 7 }; } } } fn main() {}",
        ),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| public_struct_rejection_failure(label, source))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn established_child_diagnostics_and_field_source_order_retain_precedence() {
    for (source, expected) in [
        (
            "struct Point { x: int } fn main() { let point = Point { x: (1, 2) }; }",
            "struct `Point` field `x` type mismatch: expected int, actual (int, int)",
        ),
        (
            "struct Point { x: int } fn main() { let point = Point { x: 7.field }; }",
            FIELD_DIAGNOSTIC,
        ),
        (
            "struct Point { x: int } fn main() { let point = Point { x: match 1 { 1 => 7, _ => 9 } }; }",
            MATCH_DIAGNOSTIC,
        ),
        (
            "struct Point { x: int } fn sink() {} fn main() { let point = Point { x: sink() }; }",
            VOID_DIAGNOSTIC,
        ),
        (
            "struct Pair { first: int, second: int } fn main() { let value = Pair { first: 7.field, second: (1, 2) }; }",
            FIELD_DIAGNOSTIC,
        ),
        (
            "struct Pair { first: int, second: int } fn main() { let value = Pair { first: (1, 2), second: 7.field }; }",
            FIELD_DIAGNOSTIC,
        ),
        (
            "struct Inner { x: int } struct Outer { first: int, second: Inner } fn main() { let value = Outer { first: 7.field, second: Inner { x: 1 } }; }",
            FIELD_DIAGNOSTIC,
        ),
        (
            "struct Inner { x: int } struct Outer { first: int, second: Inner } fn main() { let value = Outer { first: (1, 2), second: Inner { x: 1 } }; }",
            "struct `Outer` field `first` type mismatch: expected int, actual (int, int)",
        ),
        (
            "struct Point { x: int } fn main() { let values = (Point { x: 7 }, 1); }",
            "flat Copy tuple element 1 has unsupported type Point; expected Int, Float, or Bool",
        ),
    ] {
        assert_public_semantic_error(source, expected);
    }
}

#[test]
fn admitted_children_retain_written_source_order_inside_preserved_outer_shapes() {
    let cases = [
        (
            "admitted-nested-Copy-first",
            "struct Inner { x: int } struct Outer { first: Inner, second: int } fn main() { let value = Outer { first: Inner { x: 1 }, second: 7.field }; }",
            FIELD_DIAGNOSTIC,
        ),
        (
            "modulo-field-child",
            "struct Point { x: int } fn main() { let point = Point { x: 9 % 4 }; }",
            "Binary operator `%` is not supported.",
        ),
        (
            "undeclared-field-child",
            "struct Point { x: int } fn main() { let point = Point { x: missing }; }",
            "Error: Use of undeclared variable `missing`.",
        ),
    ];

    for (_, source, expected) in cases {
        assert_public_semantic_error(source, expected);
    }
}

#[test]
fn cli_check_and_build_reject_root_struct_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("root");
    let source_path = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    let source = r#"
struct Point { x: int, y: int }
fn mark() -> int { return 9; }
fn main() { let point = Point { x: mark() }; }
"#;
    fs::write(&source_path, source).expect("write root StructLiteral source");

    let check = run_check(&workspace, &source_path);
    let build = run_build(&workspace, &source_path, &artifact);
    let mut failures = cli_struct_rejection_failures(
        "root check",
        &check,
        "error: Struct construction expressions are not supported.",
        None,
        source,
    );
    failures.extend(cli_struct_rejection_failures(
        "root build",
        &build,
        "error: Semantic Analysis Error: Struct construction expressions are not supported.",
        Some(&artifact),
        source,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn cli_check_and_build_reject_direct_module_struct_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("module");
    let root = workspace.path("main.aero");
    let module = workspace.path("shapes.aero");
    let artifact = workspace.path("program.ll");
    let module_source = r#"
struct Point { x: int, y: int }
fn mark() -> int { return 9; }
fn make() -> Point { return Point { x: mark() }; }
"#;
    fs::write(&root, "mod shapes; fn main() {}").expect("write direct-module root source");
    fs::write(&module, module_source).expect("write direct module StructLiteral source");

    let check = run_check(&workspace, &root);
    let build = run_build(&workspace, &root, &artifact);
    let mut failures = cli_struct_rejection_failures(
        "module check",
        &check,
        "error: Struct construction expressions are not supported.",
        None,
        module_source,
    );
    failures.extend(cli_struct_rejection_failures(
        "module build",
        &build,
        "error: Semantic Analysis Error: Struct construction expressions are not supported.",
        Some(&artifact),
        module_source,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
