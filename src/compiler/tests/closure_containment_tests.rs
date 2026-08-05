use compiler::ast::{AstNode, Expression, Statement, Type};
#[allow(deprecated)]
use compiler::{
    CompilerOptions, IrGenerationError, IrGenerator, SemanticAnalyzer, compile_program,
    generate_code, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const UNSUPPORTED_CLOSURE: &str =
    "Error: closure expressions are parsed but unsupported in executable code at 1:20.";

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "aero-closure-containment-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create closure-containment workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-closure-containment-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn located_diagnostic(source: &str, filename: &str) -> String {
    let offset = source.find('|').expect("fixture must contain a closure");
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit_once('\n')
        .map_or(before.len(), |(_, tail)| tail.len())
        + 1;
    format!(
        "Error: closure expressions are parsed but unsupported in executable code at {filename}:{line}:{column}."
    )
}

fn parsed(source: &str, filename: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, Some(filename.to_string()))
        .expect("closure fixture must lex");
    parse_with_locations(tokens).expect("closure syntax must remain parsed")
}

fn semantic_error(source: &str, filename: &str) -> String {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .analyze(parsed(source, filename))
        .expect_err("executable closure must fail semantics")
}

fn run_cli(workspace: &TestWorkspace, arguments: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    for argument in arguments {
        command.arg(argument);
    }
    command
        .current_dir(&workspace.root)
        .output()
        .expect("run Aero CLI")
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn closure_expression_fails_closed_instead_of_fabricating_int() {
    let source = "fn main() -> int { |x: int| 7 }";
    let tokens =
        try_tokenize_with_locations(source, Some("closure_false_success.aero".to_string()))
            .expect("closure fixture must lex");
    let ast = parse_with_locations(tokens).expect("closure syntax must remain parsed");
    let mut analyzer = SemanticAnalyzer::new();
    let mut failures = Vec::new();

    match analyzer.analyze(ast) {
        Ok(_) => failures.push(
            "semantic analysis accepted an int-returning closure expression, proving that the \
             closure was fabricated as Ty::Int"
                .to_string(),
        ),
        Err(error) => {
            let expected = "Error: closure expressions are parsed but unsupported in executable code at closure_false_success.aero:1:20.";
            if error != expected {
                failures.push(format!(
                    "semantic diagnostic was `{error}` instead of `{expected}`"
                ));
            }
        }
    }

    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => failures.push(format!(
            "trusted compilation reached LLVM instead of rejecting the closure; generated closure symbol: {}",
            llvm.contains("@__closure_")
        )),
        Err(error) => {
            let expected = format!("Semantic Analysis Error: {UNSUPPORTED_CLOSURE}");
            if error != expected {
                failures.push(format!(
                    "public diagnostic was `{error}` instead of `{expected}`"
                ));
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn parser_retains_closure_syntax_parameters_body_and_opening_location() {
    let source = "fn main() { let closure = |value: float| value; }";
    let ast = parsed(source, "parser_retention.aero");
    let AstNode::Statement(Statement::Function { body, .. }) = &ast[0] else {
        panic!("expected parsed function")
    };
    let Statement::Let {
        value:
            Some(Expression::Closure {
                params,
                body,
                location,
            }),
        ..
    } = &body.statements[0]
    else {
        panic!("expected retained closure binding")
    };

    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "value");
    assert!(matches!(
        &params[0].param_type,
        Type::Named(name) if name == "float"
    ));
    assert!(matches!(body.as_ref(), Expression::Identifier(name) if name == "value"));
    assert_eq!(location.filename.as_deref(), Some("parser_retention.aero"));
    assert_eq!((location.line, location.column), (1, 27));
}

#[test]
fn semantic_rejection_covers_every_frozen_closure_topology() {
    let cases = [
        (
            "inferred binding",
            "fn main() { let closure = |value| value; }",
        ),
        (
            "explicit binding annotation",
            "fn main() { let closure: int = |value: int| value; }",
        ),
        (
            "comparison",
            "fn main() -> bool { (|value: int| value) == 1 }",
        ),
        (
            "function argument",
            "fn consume(value: int) -> int { value } fn main() -> int { consume(|value: int| value) }",
        ),
        ("function return", "fn main() -> int { |value: int| value }"),
        (
            "array storage",
            "fn main() { let closures = [|value: int| value, |value: int| value + 1]; }",
        ),
        (
            "struct storage",
            "struct Cell { value: int } fn main() { let cell = Cell { value: |value: int| value }; }",
        ),
        (
            "capture",
            "fn main() { let base = 4; let closure = |value: int| value + base; }",
        ),
        (
            "closure binding call",
            "fn main() -> int { let closure = |value: int| value + 1; closure(6) }",
        ),
        (
            "unsupported parameter annotation does not activate inference",
            "fn main() { let closure = |value: Mystery| value; }",
        ),
        (
            "outer closure wins over body semantics",
            "fn main() { let closure = |value: int| missing_value; }",
        ),
    ];

    let mut failures = Vec::new();
    for (label, source) in cases {
        let filename = format!("{}.aero", label.replace(' ', "_"));
        let expected = located_diagnostic(source, &filename);
        let actual = semantic_error(source, &filename);
        if actual != expected {
            failures.push(format!("{label}: expected `{expected}`, got `{actual}`"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn checked_admission_independently_rejects_unanalyzed_closure_ast() {
    let source = "fn main() { let closure = |value: int| value; }";
    let expected = located_diagnostic(source, "unchecked_ast.aero");
    match IrGenerator::new().try_generate_ir(parsed(source, "unchecked_ast.aero")) {
        Err(IrGenerationError::Admission(message)) => assert_eq!(message, expected),
        Err(IrGenerationError::Verification(error)) => {
            panic!("closure reached verification instead of admission: {error}")
        }
        Ok(_) => panic!("unanalyzed closure AST reached checked IR"),
    }
}

#[test]
#[allow(deprecated)]
fn unchecked_lowering_quarantines_closure_without_type_layout_or_symbol() {
    let source = "fn main() { let closure = |value: Mystery| value; }";
    let raw = IrGenerator::new().generate_ir(parsed(source, "raw_quarantine.aero"));
    assert!(
        raw.keys().all(|name| !name.starts_with("__closure_")),
        "raw IR manufactured a closure symbol: {:?}",
        raw.keys().collect::<Vec<_>>()
    );
    let llvm = generate_code(raw);
    assert!(!llvm.contains("__closure_"), "{llvm}");
    assert!(!llvm.contains("Mystery"), "{llvm}");
}

#[test]
fn check_build_and_run_reject_with_the_same_located_cause_and_no_artifact() {
    let workspace = TestWorkspace::new();
    let source_path = workspace.path("closure_cli.aero");
    let artifact = workspace.path("closure_cli.ll");
    let source =
        "fn main() -> int {\n    let closure = |value: int| value + 1;\n    closure(6)\n}\n";
    fs::write(&source_path, source).expect("write closure CLI fixture");
    let expected = located_diagnostic(source, &source_path.display().to_string());

    let commands = [
        (
            "check",
            run_cli(&workspace, &[Path::new("check"), &source_path]),
        ),
        (
            "build",
            run_cli(
                &workspace,
                &[Path::new("build"), &source_path, Path::new("-o"), &artifact],
            ),
        ),
        (
            "run",
            run_cli(&workspace, &[Path::new("run"), &source_path]),
        ),
    ];

    let mut failures = Vec::new();
    for (label, output) in commands {
        let diagnostics = output_text(&output);
        if output.status.success() {
            failures.push(format!("{label} unexpectedly succeeded: {diagnostics}"));
        }
        if !diagnostics.contains(&expected) {
            failures.push(format!(
                "{label} omitted shared located cause `{expected}`: {diagnostics}"
            ));
        }
        if diagnostics.contains("__closure_") {
            failures.push(format!(
                "{label} exposed a manufactured closure symbol: {diagnostics}"
            ));
        }
    }
    if artifact.exists() {
        failures.push(format!("build left artifact {}", artifact.display()));
    }
    let run_directory = workspace.root.join(".aero").join("aero-run");
    if run_directory.exists()
        && fs::read_dir(&run_directory)
            .expect("read run artifact directory")
            .next()
            .is_some()
    {
        failures.push(format!(
            "run left executable artifacts in {}",
            run_directory.display()
        ));
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn direct_module_closure_is_rejected_at_its_own_source_location() {
    let workspace = TestWorkspace::new();
    let root = workspace.path("main.aero");
    let module = workspace.path("callbacks.aero");
    let artifact = workspace.path("module_closure.ll");
    let module_source = "fn module_value() -> int {\n    let callback = |value: int| value + 1;\n    callback(4)\n}\n";
    fs::write(&root, "mod callbacks; fn main() -> int { 0 }").expect("write direct-module root");
    fs::write(&module, module_source).expect("write direct-module closure");
    let expected = located_diagnostic(module_source, &module.display().to_string());

    let commands = [
        ("check", run_cli(&workspace, &[Path::new("check"), &root])),
        (
            "build",
            run_cli(
                &workspace,
                &[Path::new("build"), &root, Path::new("-o"), &artifact],
            ),
        ),
    ];
    let mut failures = Vec::new();
    for (label, output) in commands {
        let diagnostics = output_text(&output);
        if output.status.success() || !diagnostics.contains(&expected) {
            failures.push(format!(
                "module {label} did not reject at `{expected}`: {diagnostics}"
            ));
        }
    }
    if artifact.exists() {
        failures.push(format!("module build left {}", artifact.display()));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
