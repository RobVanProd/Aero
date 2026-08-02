use compiler::{CompilerOptions, compile_program};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "aero-function-contract-{test_name}-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create function-contract workspace");
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
            .is_some_and(|name| name.starts_with("aero-function-contract-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn analyze_source(source: &str) -> Result<(), String> {
    compile_program(source, CompilerOptions::default()).map(|_| ())
}

fn assert_semantic_rejections(cases: &[(&str, &str, &[&str])]) {
    let mut failures = Vec::new();
    for (name, source, expected_fragments) in cases {
        match analyze_source(source) {
            Ok(()) => failures.push(format!("{name}: unexpectedly accepted")),
            Err(error) => {
                for fragment in *expected_fragments {
                    if !error.contains(fragment) {
                        failures.push(format!("{name}: diagnostic `{error}` missing `{fragment}`"));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn run_aero(workspace: &TestWorkspace, args: &[&Path]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .args(args)
        .current_dir(&workspace.root)
        .output()
        .expect("run aero")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_failed_build(output: &Output, artifact: &Path, fragments: &[&str]) {
    let diagnostics = combined_output(output);
    assert!(
        !output.status.success(),
        "invalid contract build succeeded: {diagnostics}"
    );
    assert!(
        !artifact.exists(),
        "invalid contract build created {}",
        artifact.display()
    );
    assert!(diagnostics.contains("Semantic Analysis Error"));
    for fragment in fragments {
        assert!(
            diagnostics.contains(fragment),
            "diagnostic `{diagnostics}` missing `{fragment}`"
        );
    }
}

fn llvm_function<'a>(llvm: &'a str, signature_prefix: &str) -> &'a str {
    let start = llvm.find(signature_prefix).expect("function definition");
    let tail = &llvm[start..];
    let end = tail.find("\n}\n").expect("function terminator") + 3;
    &tail[..end]
}

#[test]
fn rejects_undefined_duplicate_and_invalid_numeric_calls() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "undefined function",
            "fn main() { missing(1); }",
            &["missing"],
        ),
        (
            "duplicate function",
            "fn duplicate() {} fn duplicate() {}",
            &["duplicate"],
        ),
        (
            "too few arguments",
            "fn takes_one(value: i32) -> i32 { return value; } fn main() { takes_one(); }",
            &["takes_one", "expected 1", "actual 0"],
        ),
        (
            "too many arguments",
            "fn takes_one(value: i32) -> i32 { return value; } fn main() { takes_one(1, 2); }",
            &["takes_one", "expected 1", "actual 2"],
        ),
        (
            "float passed to int",
            "fn takes_int(value: i32) -> i32 { return value; } fn main() { takes_int(1.0); }",
            &["takes_int", "expected int", "actual float"],
        ),
        (
            "int passed to float",
            "fn takes_float(value: f64) -> f64 { return value; } fn main() { takes_float(1); }",
            &["takes_float", "expected float", "actual int"],
        ),
    ];

    assert_semantic_rejections(cases);
}

#[test]
fn rejects_invalid_explicit_tail_missing_and_void_returns() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "explicit return mismatch",
            "fn explicit_bad() -> i32 { return 1.0; }",
            &["explicit_bad", "expected int", "actual float"],
        ),
        (
            "tail return mismatch",
            "fn tail_bad() -> f64 { 1 }",
            &["tail_bad", "expected float", "actual int"],
        ),
        (
            "missing return",
            "fn missing_return() -> i32 {}",
            &["missing_return", "return int"],
        ),
        (
            "bare non-void return",
            "fn bare_return() -> i32 { return; }",
            &["bare_return", "expected int", "actual void"],
        ),
        (
            "value return from void",
            "fn value_return() { return 1; }",
            &["value_return", "expected void", "actual int"],
        ),
        (
            "void call used as value",
            "fn notify() {} fn main() { let value = notify(); }",
            &["notify", "void", "value"],
        ),
    ];

    assert_semantic_rejections(cases);
}

#[test]
fn accepts_forward_recursive_numeric_calls_tail_returns_and_void_statements() {
    let source = r#"
fn main() {
    let whole = countdown(2);
    let fraction = half(whole) + 0.5;
    notify();
}

fn countdown(value: i32) -> i32 {
    if value < 1 {
        return value;
    } else {
        return countdown(value - 1);
    }
}

fn half(value: i32) -> f64 {
    0.5
}

fn notify() {
    return;
}
"#;

    analyze_source(source).expect("valid numeric contracts should analyze");
}

#[test]
fn public_llvm_preserves_numeric_call_types_and_discards_void_results() {
    let source = r#"
fn twice(value: i32) -> i32 {
    return value + value;
}

fn identity_float(value: f64) -> f64 {
    return value;
}

fn notify() {}

fn main() {
    let doubled = twice(2);
    let adjusted = identity_float(1.5) + 0.5;
    notify();
}
"#;
    let llvm = compile_program(source, CompilerOptions::default()).expect("compile valid source");
    let main = llvm_function(&llvm, "define i32 @main(");

    assert!(main.contains("call i32 @twice(i32 2)"), "{main}");
    assert!(
        main.contains("call double @identity_float(double "),
        "{main}"
    );
    assert_eq!(
        main.matches("fadd double").count(),
        1,
        "float call result was spuriously promoted: {main}"
    );
    assert!(main.contains("  call void @notify()"), "{main}");
    assert!(
        !main
            .lines()
            .any(|line| line.contains("= call void @notify()")),
        "void call unexpectedly assigned a result: {main}"
    );
}

#[test]
fn direct_module_numeric_contract_is_visible_before_root_analysis() {
    let workspace = TestWorkspace::new("module-visible");
    let root = workspace.path("main.aero");
    let module = workspace.path("math.aero");
    fs::write(&root, "mod math; fn main() { let result = twice(2); }").expect("write module root");
    fs::write(
        &module,
        "fn twice(value: i32) -> i32 { return value + value; }",
    )
    .expect("write numeric module");

    let output = run_aero(&workspace, &[Path::new("check"), &root]);
    assert!(
        output.status.success(),
        "forward module signature was not visible: {}",
        combined_output(&output)
    );
}

#[test]
fn direct_module_wrong_arity_fails_before_artifact_output() {
    let workspace = TestWorkspace::new("module-arity");
    let root = workspace.path("main.aero");
    let module = workspace.path("math.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&root, "mod math; fn main() { twice(); }").expect("write module root");
    fs::write(
        &module,
        "fn twice(value: i32) -> i32 { return value + value; }",
    )
    .expect("write numeric module");

    let output = run_aero(
        &workspace,
        &[Path::new("build"), &root, Path::new("-o"), &artifact],
    );
    assert_failed_build(&output, &artifact, &["twice", "expected 1", "actual 0"]);
}

#[test]
fn direct_module_undefined_call_fails_before_artifact_output() {
    let workspace = TestWorkspace::new("module-undefined");
    let root = workspace.path("main.aero");
    let module = workspace.path("helper.aero");
    let artifact = workspace.path("program.ll");
    fs::write(&root, "mod helper; fn main() { missing(); }").expect("write module root");
    fs::write(&module, "fn helper() {}").expect("write helper module");

    let output = run_aero(
        &workspace,
        &[Path::new("build"), &root, Path::new("-o"), &artifact],
    );
    assert_failed_build(&output, &artifact, &["missing"]);
}

#[test]
fn duplicate_root_and_module_function_fails_before_artifact_output() {
    let workspace = TestWorkspace::new("module-duplicate");
    let root = workspace.path("main.aero");
    let module = workspace.path("helper.aero");
    let artifact = workspace.path("program.ll");
    fs::write(
        &root,
        "mod helper; fn shared() -> i32 { return 1; } fn main() { shared(); }",
    )
    .expect("write module root");
    fs::write(&module, "fn shared() -> i32 { return 2; }").expect("write helper module");

    let output = run_aero(
        &workspace,
        &[Path::new("build"), &root, Path::new("-o"), &artifact],
    );
    assert_failed_build(&output, &artifact, &["shared"]);
}
