use compiler::ast::{AstNode, Statement};
use compiler::{
    CompilerOptions, IrGenerationError, IrGenerator, SemanticAnalyzer, compile_file,
    compile_program, parse_with_locations, try_tokenize_with_locations,
};
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
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-use-import-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create use-import test workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create use-import fixture directory");
        }
        fs::write(&path, contents).expect("write use-import fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-use-import-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn parsed(source: &str, filename: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, Some(filename.to_string()))
        .expect("use fixture must lex");
    parse_with_locations(tokens).expect("valid use syntax must remain parsed")
}

fn located_diagnostic(source: &str, filename: &str) -> String {
    let offset = source.find("use").expect("fixture must contain use");
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit_once('\n')
        .map_or(before.len(), |(_, tail)| tail.len())
        + 1;
    format!(
        "Error: use declarations are parsed but unsupported because name-resolution semantics are not implemented at {filename}:{line}:{column}."
    )
}

fn semantic_error(source: &str, filename: &str) -> String {
    SemanticAnalyzer::new()
        .analyze(parsed(source, filename))
        .expect_err("use declaration must fail semantics")
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
fn parser_retains_direct_alias_and_glob_use_shapes() {
    let source = "use alpha::beta; use gamma::delta as renamed; use tools::*; fn main() {}";
    let ast = parsed(source, "parser_retention.aero");
    let expected = [
        (vec!["alpha", "beta"], None, 1),
        (vec!["gamma", "delta"], Some("renamed"), 18),
        (vec!["tools", "*"], None, 47),
    ];

    for (node, (expected_path, expected_alias, expected_column)) in ast.iter().zip(expected) {
        let AstNode::Statement(Statement::UseImport {
            path,
            alias,
            location,
        }) = node
        else {
            panic!("expected retained use declaration, got {node:?}")
        };
        assert_eq!(
            path,
            &expected_path
                .into_iter()
                .map(str::to_string)
                .collect::<Vec<_>>()
        );
        assert_eq!(alias.as_deref(), expected_alias);
        assert_eq!(location.line, 1);
        assert_eq!(location.column, expected_column);
        assert_eq!(location.filename.as_deref(), Some("parser_retention.aero"));
    }
}

#[test]
fn semantic_analysis_rejects_every_frozen_use_placement_with_one_cause() {
    let cases = [
        ("top-level direct", "use package::value; fn main() {}"),
        (
            "top-level alias",
            "use package::value as renamed; fn main() {}",
        ),
        ("top-level glob", "use package::*; fn main() {}"),
        ("function-local", "fn main() {\n    use package::value;\n}"),
        (
            "nested block",
            "fn main() {\n    {\n        use package::value;\n    }\n}",
        ),
        (
            "trait default",
            "trait Example {\n    fn value() {\n        use package::value;\n    }\n}\nfn main() {}",
        ),
        (
            "impl method",
            "struct Example {}\nimpl Example {\n    fn value() {\n        use package::value;\n    }\n}\nfn main() {}",
        ),
    ];

    let mut failures = Vec::new();
    for (label, source) in cases {
        let filename = format!("{}.aero", label.replace(' ', "_"));
        let expected = located_diagnostic(source, &filename);
        match SemanticAnalyzer::new().analyze(parsed(source, &filename)) {
            Ok(_) => failures.push(format!(
                "{label}: semantic analysis silently erased the use declaration"
            )),
            Err(actual) if actual != expected => {
                failures.push(format!("{label}: expected `{expected}`, got `{actual}`"))
            }
            Err(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn source_and_file_library_routes_reject_before_llvm_return() {
    let source = "use package::value; fn main() -> int { 7 }";
    let expected_source = located_diagnostic(source, "source_only.aero");
    let source_error = semantic_error(source, "source_only.aero");
    assert_eq!(source_error, expected_source);
    assert_eq!(
        compile_program(source, CompilerOptions::default())
            .expect_err("source-only compilation must reject use"),
        format!(
            "Semantic Analysis Error: {}",
            located_diagnostic(source, "").replace("at :", "at ")
        )
    );

    let workspace = TestWorkspace::new("library");
    let root = workspace.write("main.aero", source);
    let expected_file = located_diagnostic(source, &root.display().to_string());
    let file_error = compile_file(&root, CompilerOptions::default())
        .expect_err("file compilation must reject use");
    assert_eq!(
        file_error,
        format!("Semantic Analysis Error: {expected_file}")
    );
}

#[test]
fn checked_admission_independently_rejects_unanalyzed_use_ast() {
    let source = "fn main() { use package::value; }";
    let expected = located_diagnostic(source, "unchecked_use.aero");
    match IrGenerator::new().try_generate_ir(parsed(source, "unchecked_use.aero")) {
        Err(IrGenerationError::Admission(actual)) => assert_eq!(actual, expected),
        Err(IrGenerationError::Verification(error)) => {
            panic!("use reached verification instead of admission: {error}")
        }
        Ok(_) => panic!("unanalyzed use declaration reached checked IR"),
    }
}

#[test]
fn direct_module_use_is_rejected_at_the_module_location() {
    let workspace = TestWorkspace::new("direct-module");
    let root = workspace.write("main.aero", "mod helper; fn main() -> int { helper() }");
    let module_source = "use package::value; fn helper() -> int { 17 }";
    let module = workspace.write("helper.aero", module_source);
    let expected = located_diagnostic(module_source, &module.display().to_string());

    let error = compile_file(&root, CompilerOptions::default())
        .expect_err("direct-module use must reject file compilation");
    assert_eq!(error, format!("Semantic Analysis Error: {expected}"));
}

#[test]
fn check_build_and_run_fail_without_requested_or_native_artifacts() {
    let workspace = TestWorkspace::new("cli");
    let source = "fn main() {\n    use package::value;\n}\n";
    let root = workspace.write("main.aero", source);
    let artifact = workspace.path("main.ll");
    let expected = located_diagnostic(source, &root.display().to_string());

    let commands = [
        ("check", run_cli(&workspace, &[Path::new("check"), &root])),
        (
            "build",
            run_cli(
                &workspace,
                &[Path::new("build"), &root, Path::new("-o"), &artifact],
            ),
        ),
        ("run", run_cli(&workspace, &[Path::new("run"), &root])),
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
    }
    if artifact.exists() {
        failures.push(format!(
            "build left requested artifact {}",
            artifact.display()
        ));
    }
    let run_root = workspace.root.join(".aero").join("aero-run");
    if run_root.exists()
        && fs::read_dir(&run_root)
            .expect("read run artifact directory")
            .next()
            .is_some()
    {
        failures.push(format!(
            "run left native artifacts in {}",
            run_root.display()
        ));
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn malformed_use_and_unimplemented_import_text_keep_existing_parse_boundaries() {
    let malformed_use = compile_program("use package::; fn main() {}", CompilerOptions::default())
        .expect_err("malformed use must remain a parse failure");
    assert!(malformed_use.starts_with("Parse error:"), "{malformed_use}");

    let import_text = compile_program(
        "import package.value; fn main() {}",
        CompilerOptions::default(),
    )
    .expect_err("founding import grammar is not implemented by this task");
    assert!(import_text.starts_with("Parse error:"), "{import_text}");
}

#[test]
fn ordinary_source_and_direct_modules_remain_accepted() {
    let source = "fn main() -> int { 11 }";
    compile_program(source, CompilerOptions::default())
        .expect("module-free source must remain accepted");

    let workspace = TestWorkspace::new("positive-controls");
    let root = workspace.write("main.aero", "mod helper; fn main() -> int { helper() }");
    workspace.write("helper.aero", "fn helper() -> int { 13 }");
    compile_file(&root, CompilerOptions::default())
        .expect("ordinary direct module must remain accepted");
}
