use compiler::{CompilerOptions, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestWorkspace {
    root: PathBuf,
    nonce: u128,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "aero-numeric-let-annotation-{test_name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create numeric-let-annotation workspace");
        Self { root, nonce }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    fn unique_source(&self, source: &str) -> String {
        // Keep each fixture distinct so its diagnostics and artifacts are
        // attributable to one CLI invocation.
        format!(
            "{source}\n// numeric-let-annotation test nonce {}\n",
            self.nonce
        )
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-numeric-let-annotation-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn compile(source: &str) -> Result<String, String> {
    compile_program(source, CompilerOptions::default())
}

fn assert_binding_rejections(cases: &[(&str, &str, &str, &str)]) {
    let mut failures = Vec::new();
    for (case_name, source, expected, actual) in cases {
        match compile(source) {
            Ok(_) => failures.push(format!("{case_name}: unexpectedly accepted")),
            Err(error) => {
                for fragment in [
                    "Variable `value` type annotation mismatch",
                    &format!("expected {expected}"),
                    &format!("actual {actual}"),
                ] {
                    if !error.contains(fragment) {
                        failures.push(format!(
                            "{case_name}: diagnostic `{error}` missing `{fragment}`"
                        ));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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

fn assert_cli_rejection(output: &Output, artifact: Option<&Path>, expected: &str, actual: &str) {
    let diagnostics = combined_output(output);
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!("invalid annotation exited zero: {diagnostics}"));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        failures.push(format!(
            "invalid annotation created requested artifact {}",
            artifact.display()
        ));
    }
    for fragment in [
        "Variable `value` type annotation mismatch",
        &format!("expected {expected}"),
        &format!("actual {actual}"),
    ] {
        if !diagnostics.contains(fragment) {
            failures.push(format!("diagnostic `{diagnostics}` missing `{fragment}`"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn accepts_exact_alias_literals_identifiers_and_mutable_binding() {
    let source = r#"
fn main() {
    let int_literal: int = 1;
    let i32_literal: i32 = 2;
    let float_literal: float = 1.5;
    let f64_literal: f64 = 2.5;

    let int_identifier: int = i32_literal;
    let i32_identifier: i32 = int_literal;
    let float_identifier: float = f64_literal;
    let f64_identifier: f64 = float_literal;
    let mut mutable_value: i32 = int_identifier;
}
"#;

    compile(source).expect("exact numeric aliases and mutable binding should compile");
}

#[test]
fn accepts_exact_numeric_calls_mixed_float_and_nested_shadowing() {
    let source = r#"
fn identity_int(value: i32) -> i32 {
    return value;
}

fn identity_float(value: f64) -> f64 {
    return value;
}

fn main() {
    let called_int: int = identity_int(7);
    let called_float: float = identity_float(1.5);
    let mixed: f64 = called_int + 0.5;

    let value: int = 1;
    {
        let value: float = 2.5;
        let inner: f64 = value;
    }
    let outer: i32 = value;
}
"#;

    compile(source).expect("existing inference should satisfy exact numeric annotations");
}

#[test]
fn public_llvm_preserves_numeric_abi_casts_and_scalar_slot_lowering() {
    let source = r#"
fn identity_int(value: i32) -> i32 {
    return value;
}

fn identity_float(value: f64) -> f64 {
    return value;
}

fn main() {
    let integer: int = identity_int(7);
    let fractional: f64 = identity_float(1.5);
    let mixed: float = integer + fractional;
}
"#;
    let llvm = compile(source).expect("valid annotated program should produce public LLVM");

    assert!(
        llvm.contains("define i32 @identity_int(i32 %value)"),
        "{llvm}"
    );
    assert!(
        llvm.contains("define double @identity_float(double %value)"),
        "{llvm}"
    );
    assert!(llvm.contains("call i32 @identity_int(i32 7)"), "{llvm}");
    assert!(
        llvm.contains("call double @identity_float(double "),
        "{llvm}"
    );
    assert!(llvm.contains("sitofp i32"), "{llvm}");
    assert!(llvm.contains("alloca double"), "{llvm}");
    assert!(
        !llvm.contains("alloca i32"),
        "numeric annotation unexpectedly changed existing scalar slot lowering: {llvm}"
    );
}

#[test]
fn rejects_all_literal_alias_mismatches() {
    let cases = [
        ("int from float", "let value: int = 1.5;", "int", "float"),
        ("i32 from float", "let value: i32 = 1.5;", "int", "float"),
        ("float from int", "let value: float = 1;", "float", "int"),
        ("f64 from int", "let value: f64 = 1;", "float", "int"),
    ];

    assert_binding_rejections(&cases);
}

#[test]
fn rejects_numeric_expression_and_identifier_mismatches() {
    let cases = [
        (
            "mixed expression into int",
            "let value: int = 1 + 0.5;",
            "int",
            "float",
        ),
        (
            "integer expression into float",
            "let value: float = 1 + 2;",
            "float",
            "int",
        ),
        (
            "float identifier into int",
            "let source = 1.5; let value: i32 = source;",
            "int",
            "float",
        ),
        (
            "int identifier into float",
            "let source = 1; let value: f64 = source;",
            "float",
            "int",
        ),
    ];

    assert_binding_rejections(&cases);
}

#[test]
fn rejects_numeric_function_result_mismatches_in_both_directions() {
    let cases = [
        (
            "float function into int",
            "fn ratio() -> float { return 1.5; } fn main() { let value: i32 = ratio(); }",
            "int",
            "float",
        ),
        (
            "int function into float",
            "fn one() -> int { return 1; } fn main() { let value: f64 = one(); }",
            "float",
            "int",
        ),
    ];

    assert_binding_rejections(&cases);
}

#[test]
fn rejects_function_local_and_nested_binding_mismatches() {
    let cases = [
        (
            "function local",
            "fn main() { let value: int = 1.5; }",
            "int",
            "float",
        ),
        (
            "nested block",
            "fn main() { let outer = 1; { let value: float = outer; } }",
            "float",
            "int",
        ),
    ];

    assert_binding_rejections(&cases);
}

#[test]
fn preserves_same_scope_duplicate_diagnostic_precedence() {
    let error = compile("let value: int = 1; let value: float = 1;")
        .expect_err("same-scope duplicate must remain rejected before annotation validation");

    assert!(error.contains("Variable `value` is already defined in this scope"));
    assert!(
        !error.contains("type annotation mismatch"),
        "annotation mismatch displaced duplicate diagnostic: {error}"
    );
}

#[test]
fn cli_check_rejects_root_mismatch() {
    let workspace = TestWorkspace::new("root-check");
    let root = workspace.path("main.aero");
    fs::write(
        &root,
        workspace.unique_source("fn main() { let value: int = 1.5; }"),
    )
    .expect("write root source");

    let output = run_check(&workspace, &root);
    assert_cli_rejection(&output, None, "int", "float");
}

#[test]
fn cli_build_rejects_root_mismatch_without_artifact() {
    let workspace = TestWorkspace::new("root-build");
    let root = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(
        &root,
        workspace.unique_source("fn main() { let value: float = 1; }"),
    )
    .expect("write root source");

    let output = run_build(&workspace, &root, &artifact);
    assert_cli_rejection(&output, Some(&artifact), "float", "int");
}

#[test]
fn cli_check_rejects_direct_module_mismatch() {
    let workspace = TestWorkspace::new("module-check");
    let root = workspace.path("main.aero");
    let module = workspace.path("invalid.aero");
    fs::write(&root, workspace.unique_source("mod invalid; fn main() {}"))
        .expect("write module root");
    fs::write(&module, "fn helper() { let value: i32 = 1.5; }").expect("write invalid module");

    let output = run_check(&workspace, &root);
    assert_cli_rejection(&output, None, "int", "float");
}

#[test]
fn cli_build_rejects_direct_module_mismatch_without_artifact() {
    let workspace = TestWorkspace::new("module-build");
    let root = workspace.path("main.aero");
    let module = workspace.path("invalid.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&root, workspace.unique_source("mod invalid; fn main() {}"))
        .expect("write module root");
    fs::write(&module, "fn helper() { let value: f64 = 1; }").expect("write invalid module");

    let output = run_build(&workspace, &root, &artifact);
    assert_cli_rejection(&output, Some(&artifact), "float", "int");
}
