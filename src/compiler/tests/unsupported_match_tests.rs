use compiler::ast::{AstNode, Expression, Pattern, Statement};
use compiler::{
    CompilerOptions, compile_program, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_INNER_DIAGNOSTIC: &str = "Match expressions are not supported.";
const FIELD_DIAGNOSTIC: &str = "Field access expressions are not supported.";
const TUPLE_DIAGNOSTIC: &str = "Tuple expressions are not supported.";
const VOID_DIAGNOSTIC: &str = "Error: void function `sink` cannot be used as a value.";

const NUMERIC_CONTROL_SOURCE: &str = r#"
fn add(left: int, right: int) -> int {
    return left + right;
}

fn main() {
    let sum: int = add(6, 2);
    if sum > 3 {
        let smaller: int = sum - 1;
    } else {
        let larger: int = sum + 1;
    }
    while sum < 0 {
        let doubled: int = sum * 2;
        break;
    }
    let values = [11, 22];
    let selected: int = values[1];
}
"#;

const SYNTAX_CONTROL_SOURCE: &str = r#"
struct Point { x: int, y: int }
enum Choice { Empty, Number(int) }

fn main() {
    let message = "ready";
    let point = Point { x: 7, y: 9 };
    let choice = Choice::Number(11);
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
            "aero-unsupported-match-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh Match test workspace");
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
            .is_some_and(|name| name.starts_with("aero-unsupported-match-"));
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
    let dropped_mark_call = defines_mark && !llvm.contains("call i32 @mark(");
    let hides_zero_division = source.contains("/ 0");
    let suppressed_zero_division = hides_zero_division && !llvm.contains("fdiv double");
    let fabricated_zero = llvm.contains("0x0000000000000000") || llvm.contains("ret i32 0");
    format!(
        "fabricated_zero={fabricated_zero}, empty_main={main_is_empty}, dropped_mark_call={dropped_mark_call}, suppressed_zero_division={suppressed_zero_division}, fdiv={}, llvm_bytes={}",
        llvm.contains("fdiv double"),
        llvm.len()
    )
}

fn public_match_rejection_failure(label: &str, source: &str) -> Option<String> {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => Some(format!(
            "{label}: compile_program unwound instead of returning `{EXPECTED_INNER_DIAGNOSTIC}`"
        )),
        Ok(Ok(llvm)) => Some(format!(
            "{label}: unexpectedly accepted Match; red lowering evidence: {}",
            red_llvm_evidence(source, &llvm)
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

fn strip_cli_colors(output: &str) -> String {
    output
        .replace("\x1b[1;31m", "")
        .replace("\x1b[1;32m", "")
        .replace("\x1b[1;34m", "")
        .replace("\x1b[1;36m", "")
        .replace("\x1b[0m", "")
}

fn cli_match_rejection_failures(
    label: &str,
    output: &Output,
    expected_line: &str,
    artifact: Option<&Path>,
    source: &str,
) -> Vec<String> {
    let diagnostics = strip_cli_colors(&combined_output(output));
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!("{label}: Match source exited zero: {diagnostics}"));
    }
    if diagnostics.to_ascii_lowercase().contains("panicked") {
        failures.push(format!(
            "{label}: Match source reached a panic: {diagnostics}"
        ));
    }
    if !diagnostics.lines().any(|line| line.trim() == expected_line) {
        failures.push(format!(
            "{label}: expected exact CLI line `{expected_line}`, received `{diagnostics}`"
        ));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        let evidence = fs::read_to_string(artifact)
            .map(|llvm| red_llvm_evidence(source, &llvm))
            .unwrap_or_else(|error| format!("artifact unreadable: {error}"));
        failures.push(format!(
            "{label}: Match source created requested artifact {}; red artifact evidence: {evidence}",
            artifact.display()
        ));
    }
    failures
}

#[test]
fn parser_retains_three_arm_match_shape_and_patterns() {
    let tokens = try_tokenize_with_locations(
        "let result = match value { 1 => 10, 2 => 20, _ => 0 };",
        None,
    )
    .expect("three-arm Match should lex");
    let ast = parse_with_locations(tokens).expect("three-arm Match should parse");
    assert_eq!(ast.len(), 1, "unexpected Match AST: {ast:?}");

    let AstNode::Statement(Statement::Let {
        value: Some(Expression::Match { expr, arms }),
        ..
    }) = &ast[0]
    else {
        panic!("expected Match binding: {:?}", ast[0]);
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(name) if name == "value"));
    assert_eq!(arms.len(), 3);
    assert!(matches!(
        &arms[0].pattern,
        Pattern::Literal(Expression::IntegerLiteral(1))
    ));
    assert!(matches!(arms[0].body, Expression::IntegerLiteral(10)));
    assert!(matches!(
        &arms[1].pattern,
        Pattern::Literal(Expression::IntegerLiteral(2))
    ));
    assert!(matches!(arms[1].body, Expression::IntegerLiteral(20)));
    assert!(matches!(arms[2].pattern, Pattern::Wildcard));
    assert!(matches!(arms[2].body, Expression::IntegerLiteral(0)));
}

#[test]
fn public_compile_preserves_numeric_function_control_flow_array_and_index_controls() {
    let llvm = compile_program(NUMERIC_CONTROL_SOURCE, CompilerOptions::default())
        .expect("numeric, function, if/while, array, and index controls should compile");
    for marker in [
        "define i32 @add(i32 %left, i32 %right)",
        "call i32 @add(i32 6, i32 2)",
        "fadd double",
        "fsub double",
        "fmul double",
        "icmp sgt",
        "icmp slt",
        "alloca [2 x double]",
        "getelementptr inbounds [2 x double]",
    ] {
        assert!(llvm.contains(marker), "LLVM missing `{marker}`:\n{llvm}");
    }
    assert!(
        llvm.matches("br i1").count() >= 2,
        "if/while conditional branches missing:\n{llvm}"
    );
}

#[test]
fn parser_and_public_compile_preserve_string_struct_and_enum_syntax_controls() {
    let tokens = try_tokenize_with_locations(SYNTAX_CONTROL_SOURCE, None)
        .expect("string/struct/enum controls should lex");
    let ast = parse_with_locations(tokens).expect("string/struct/enum controls should parse");
    assert!(matches!(
        &ast[0],
        AstNode::Statement(Statement::StructDef { name, fields, .. })
            if name == "Point" && fields.len() == 2
    ));
    assert!(matches!(
        &ast[1],
        AstNode::Statement(Statement::EnumDef { name, variants, .. })
            if name == "Choice" && variants.len() == 2
    ));
    let AstNode::Statement(Statement::Function { body, .. }) = &ast[2] else {
        panic!("expected main function: {:?}", ast[2]);
    };
    assert!(body.statements.iter().any(|statement| matches!(
        statement,
        Statement::Let { value: Some(Expression::StringLiteral(value)), .. } if value == "ready"
    )));
    assert!(body.statements.iter().any(|statement| matches!(
        statement,
        Statement::Let { value: Some(Expression::StructLiteral { name, fields }), .. }
            if name == "Point" && fields.len() == 2
    )));
    assert!(body.statements.iter().any(|statement| matches!(
        statement,
        Statement::Let {
            value: Some(Expression::EnumVariant { enum_name, variant, data: Some(_) }),
            ..
        } if enum_name == "Choice" && variant == "Number"
    )));

    compile_program(SYNTAX_CONTROL_SOURCE, CompilerOptions::default())
        .expect("string/struct/enum controls should retain pre-task behavior");
}

#[test]
fn cli_check_and_build_preserve_zero_argument_array_iter_control() {
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
fn established_child_diagnostics_retain_match_precedence() {
    assert_public_semantic_error(
        "fn main() { let value: int = match (1, 2) { _ => 0 }; }",
        TUPLE_DIAGNOSTIC,
    );
    assert_public_semantic_error(
        "fn main() { let value: int = match 1 { 1 => 7.field, _ => 0 }; }",
        FIELD_DIAGNOSTIC,
    );
    assert_public_semantic_error(
        "fn sink() {} fn main() { let value: int = match 1 { 1 => sink(), _ => 0 }; }",
        VOID_DIAGNOSTIC,
    );
    assert_public_semantic_error(
        "fn main() { let value: int = (match 1 { 1 => 7, _ => 0 }, 2); }",
        TUPLE_DIAGNOSTIC,
    );
}

#[test]
fn public_compile_rejects_match_matrix_without_unwinding() {
    let cases = [
        (
            "minimal",
            "fn main() { let value: int = match 1 { 1 => 11, _ => 22 }; }",
        ),
        (
            "bound-scrutinee",
            "fn main() { let key: int = 1; let value: int = match key { 1 => 11, _ => 22 }; }",
        ),
        (
            "undeclared-scrutinee",
            "fn main() { let value: int = match missing { _ => 22 }; }",
        ),
        (
            "call-scrutinee",
            "fn mark() -> int { 1 } fn main() { let value: int = match mark() { 1 => 11, _ => 22 }; }",
        ),
        (
            "string-scrutinee",
            "fn main() { let value: int = match \"ready\" { _ => 22 }; }",
        ),
        (
            "option-scrutinee",
            "fn main() { let value: int = match Option::Some(7) { _ => 22 }; }",
        ),
        (
            "call-arm",
            "fn mark() -> int { 11 } fn main() { let value: int = match 1 { 1 => mark(), _ => 22 }; }",
        ),
        (
            "undeclared-arm",
            "fn main() { let value: int = match 1 { 1 => missing, _ => 22 }; }",
        ),
        (
            "multiple-bodies",
            "fn main() { let value: int = match 2 { 1 => 11, 2 => 22, _ => 33 }; }",
        ),
        ("root-expression", "match 1 { 1 => 11, _ => 22 };"),
        (
            "top-level-binding",
            "let value: int = match 1 { 1 => 11, _ => 22 };",
        ),
        ("discarded", "fn main() { match 1 { 1 => 11, _ => 22 }; }"),
        (
            "explicit-return",
            "fn project() -> int { return match 1 { 1 => 11, _ => 22 }; } fn main() { let value: int = project(); }",
        ),
        (
            "tail-return",
            "fn project() -> int { match 1 { 1 => 11, _ => 22 } } fn main() { let value: int = project(); }",
        ),
        (
            "binary-parent",
            "fn main() { let value: int = 1 + match 1 { 1 => 11, _ => 22 }; }",
        ),
        (
            "non-first-array-element",
            "fn main() { let values = [7, match 1 { 1 => 11, _ => 22 }]; }",
        ),
        (
            "call-argument",
            "fn identity(value: int) -> int { value } fn main() { let value: int = identity(match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "closure-body",
            "fn main() { let project = |key: int| match key { 1 => 11, _ => 22 }; let value: int = project(1); }",
        ),
        (
            "nested-match",
            "fn main() { let value: int = match match 1 { 1 => 2, _ => 3 } { 2 => 11, _ => 22 }; }",
        ),
        (
            "hidden-zero-division-scrutinee",
            "fn main() { let value: int = match 5 / 0 { _ => 22 }; }",
        ),
        (
            "hidden-zero-division-arm",
            "fn main() { let value: int = match 1 { 1 => 5 / 0, _ => 22 }; }",
        ),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| public_match_rejection_failure(label, source))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn public_compile_rejects_match_in_recursive_parent_matrix_without_unwinding() {
    let cases = [
        (
            "unary-parent",
            "fn main() { let value: int = -(match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "comparison-parent",
            "fn main() { let value = (match 1 { 1 => 11, _ => 22 }) > 0; }",
        ),
        (
            "array-repeat-parent",
            "fn main() { let values = [match 1 { 1 => 11, _ => 22 }; 2]; }",
        ),
        (
            "index-parent",
            "fn main() { let value: int = [11, 22][match 1 { 1 => 0, _ => 1 }]; }",
        ),
        (
            "method-receiver",
            "fn main() { let value: int = (match 1 { 1 => 11, _ => 22 }).unknown(); }",
        ),
        (
            "method-argument",
            "fn main() { let values = [1, 2]; let value: int = values.unknown(match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "print-argument",
            "fn main() { println!(\"{}\", match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "struct-field",
            "struct Point { x: int } fn main() { let point = Point { x: match 1 { 1 => 11, _ => 22 } }; }",
        ),
        (
            "enum-payload",
            "fn main() { let option = Some(match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "borrow-parent",
            "fn main() { let value = &(match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "deref-parent",
            "fn main() { let value: int = *&(match 1 { 1 => 11, _ => 22 }); }",
        ),
        (
            "field-receiver",
            "fn main() { let value: int = (match 1 { 1 => 11, _ => 22 }).field; }",
        ),
        (
            "if-condition",
            "fn main() { if match 1 { 1 => 1 < 2, _ => 2 < 1 } { let value: int = 1; } }",
        ),
        (
            "while-condition",
            "fn main() { while match 1 { 1 => 1 < 2, _ => 2 < 1 } { break; } }",
        ),
        (
            "for-iterable",
            "fn main() { for item in match 1 { 1 => [11], _ => [22] } { let value: int = item; } }",
        ),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| public_match_rejection_failure(label, source))
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn cli_check_and_build_reject_root_match_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("root");
    let source_path = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    let source = "fn mark() -> int { 1 } match mark() { 1 => 11, _ => 22 };";
    fs::write(&source_path, source).expect("write root Match source");

    let check = run_check(&workspace, &source_path);
    let build = run_build(&workspace, &source_path, &artifact);
    let mut failures = cli_match_rejection_failures(
        "root check",
        &check,
        "error: Match expressions are not supported.",
        None,
        source,
    );
    failures.extend(cli_match_rejection_failures(
        "root build",
        &build,
        "error: Semantic Analysis Error: Match expressions are not supported.",
        Some(&artifact),
        source,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn cli_check_and_build_reject_direct_module_match_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("module");
    let root = workspace.path("main.aero");
    let module = workspace.path("matcher.aero");
    let artifact = workspace.path("program.ll");
    let module_source =
        "fn mark() -> int { 1 } fn project() -> int { return match mark() { 1 => 11, _ => 22 }; }";
    fs::write(&root, "mod matcher; fn main() {}").expect("write direct-module root source");
    fs::write(&module, module_source).expect("write direct module with Match");

    let check = run_check(&workspace, &root);
    let build = run_build(&workspace, &root, &artifact);
    let mut failures = cli_match_rejection_failures(
        "module check",
        &check,
        "error: Match expressions are not supported.",
        None,
        module_source,
    );
    failures.extend(cli_match_rejection_failures(
        "module build",
        &build,
        "error: Semantic Analysis Error: Match expressions are not supported.",
        Some(&artifact),
        module_source,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn trait_default_error_failure(label: &str, source: &str, expected: &str) -> Option<String> {
    match catch_unwind(|| compile_program(source, CompilerOptions::default())) {
        Err(_) => Some(format!(
            "{label}: compile_program unwound instead of returning `{expected}`"
        )),
        Ok(Ok(llvm)) => Some(format!(
            "{label}: unexpectedly accepted parser-retained trait default body; body traversal was skipped (dropped_mark_call={}, llvm_bytes={})",
            source.contains("fn mark(") && !llvm.contains("call i32 @mark("),
            llvm.len()
        )),
        Ok(Err(error)) => {
            let Some(inner) = error.strip_prefix("Semantic Analysis Error: ") else {
                return Some(format!(
                    "{label}: unexpected public error category: {error}"
                ));
            };
            (inner != expected)
                .then(|| format!("{label}: expected `{expected}`, received `{inner}`"))
        }
    }
}

#[test]
fn parser_retains_match_in_default_trait_body_and_required_signature() {
    let source = r#"
trait Selectable {
    fn select(&self) -> int {
        return match 2 { 1 => 11, 2 => 22, _ => 33 };
    }
    fn required(&self) -> int;
}
"#;
    let tokens = try_tokenize_with_locations(source, None).expect("trait default should lex");
    let ast = parse_with_locations(tokens).expect("trait default should parse");
    let [AstNode::Statement(Statement::TraitDef { name, methods, .. })] = ast.as_slice() else {
        panic!("expected one trait definition: {ast:?}");
    };
    assert_eq!(name, "Selectable");
    assert_eq!(methods.len(), 2);
    assert_eq!(methods[0].name, "select");
    let body = methods[0]
        .body
        .as_ref()
        .expect("default method body should be retained");
    let [Statement::Return(Some(Expression::Match { expr, arms }))] = body.statements.as_slice()
    else {
        panic!("expected returned Match in default body: {body:?}");
    };
    assert!(matches!(expr.as_ref(), Expression::IntegerLiteral(2)));
    assert_eq!(arms.len(), 3);
    assert!(body.expression.is_none());
    assert_eq!(methods[1].name, "required");
    assert!(methods[1].body.is_none());
}

#[test]
fn public_compile_preserves_trait_default_and_required_signature_controls_without_match() {
    let syntax_only_default = r#"
trait SyntaxOnlyDefault {
    fn project(&self) -> int {
        let value: int = unresolved_default_name;
        return value;
    }
}
fn main() {}
"#;
    compile_program(syntax_only_default, CompilerOptions::default()).expect(
        "default body without Match should retain syntax-only behavior, not activate name binding",
    );

    let required_signature = r#"
trait RequiredOnly {
    fn project(&self) -> int;
}
fn main() {}
"#;
    compile_program(required_signature, CompilerOptions::default())
        .expect("required trait signature without a body should remain accepted");
}

#[test]
fn public_compile_rejects_match_across_default_trait_body_traversal_without_unwinding() {
    let cases = [
        (
            "direct-return",
            "trait DirectReturn { fn choose(&self) -> int { return match 1 { 1 => 11, _ => 22 }; } } fn main() {}",
        ),
        (
            "tail-expression",
            "trait TailExpression { fn choose(&self) -> int { match 1 { 1 => 11, _ => 22 } } } fn main() {}",
        ),
        (
            "let-binding",
            "trait LetBinding { fn choose(&self) -> int { let selected: int = match 1 { 1 => 11, _ => 22 }; return selected; } } fn main() {}",
        ),
        (
            "discarded-expression",
            "trait Discarded { fn choose(&self) -> int { match 1 { 1 => 11, _ => 22 }; return 0; } } fn main() {}",
        ),
        (
            "if-condition",
            "trait IfCondition { fn choose(&self) -> int { if match 1 { 1 => 1 < 2, _ => 2 < 1 } { return 11; } return 22; } } fn main() {}",
        ),
        (
            "while-body",
            "trait WhileBody { fn choose(&self) -> int { while 1 < 2 { let selected: int = match 1 { 1 => 11, _ => 22 }; break; } return 0; } } fn main() {}",
        ),
        (
            "for-body",
            "trait ForBody { fn choose(&self) -> int { for item in [1, 2] { match item { 1 => 11, _ => 22 }; } return 0; } } fn main() {}",
        ),
        (
            "array-container",
            "trait ArrayContainer { fn choose(&self) -> int { let values = [7, match 1 { 1 => 11, _ => 22 }]; return 0; } } fn main() {}",
        ),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source)| {
            trait_default_error_failure(label, source, EXPECTED_INNER_DIAGNOSTIC)
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn default_trait_body_child_diagnostics_retain_precedence() {
    let cases = [
        (
            "tuple-scrutinee",
            "trait TupleChild { fn choose(&self) -> int { return match (1, 2) { _ => 0 }; } } fn main() {}",
            TUPLE_DIAGNOSTIC,
        ),
        (
            "field-arm",
            "trait FieldChild { fn choose(&self) -> int { return match 1 { 1 => 7.field, _ => 0 }; } } fn main() {}",
            FIELD_DIAGNOSTIC,
        ),
        (
            "void-arm",
            "fn sink() {} trait VoidChild { fn choose(&self) -> int { return match 1 { 1 => sink(), _ => 0 }; } } fn main() {}",
            VOID_DIAGNOSTIC,
        ),
    ];

    let failures = cases
        .iter()
        .filter_map(|(label, source, expected)| {
            trait_default_error_failure(label, source, expected)
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn cli_check_and_build_reject_match_in_root_trait_default_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("trait-default-root");
    let source_path = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    let source = r#"
fn mark() -> int { 1 }
trait RootDefault {
    fn choose(&self) -> int {
        return match mark() { 1 => 11, _ => 22 };
    }
}
fn main() {}
"#;
    fs::write(&source_path, source).expect("write root trait-default Match source");

    let check = run_check(&workspace, &source_path);
    let build = run_build(&workspace, &source_path, &artifact);
    let mut failures = cli_match_rejection_failures(
        "trait-default root check",
        &check,
        "error: Match expressions are not supported.",
        None,
        source,
    );
    failures.extend(cli_match_rejection_failures(
        "trait-default root build",
        &build,
        "error: Semantic Analysis Error: Match expressions are not supported.",
        Some(&artifact),
        source,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn cli_check_and_build_reject_match_in_module_trait_default_without_panic_or_artifact() {
    let workspace = TestWorkspace::new("trait-default-module");
    let root = workspace.path("main.aero");
    let module = workspace.path("defaults.aero");
    let artifact = workspace.path("program.ll");
    let module_source = r#"
fn mark() -> int { 1 }
trait ModuleDefault {
    fn choose(&self) -> int {
        return match mark() { 1 => 11, _ => 22 };
    }
}
"#;
    fs::write(&root, "mod defaults; fn main() {}").expect("write trait-default module root");
    fs::write(&module, module_source).expect("write module trait-default Match source");

    let check = run_check(&workspace, &root);
    let build = run_build(&workspace, &root, &artifact);
    let mut failures = cli_match_rejection_failures(
        "trait-default module check",
        &check,
        "error: Match expressions are not supported.",
        None,
        module_source,
    );
    failures.extend(cli_match_rejection_failures(
        "trait-default module build",
        &build,
        "error: Semantic Analysis Error: Match expressions are not supported.",
        Some(&artifact),
        module_source,
    ));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
