use compiler::{CompilerOptions, compile_program};
use std::fs;
use std::panic::catch_unwind;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

const ALTERNATE_ARM_SCOPE_LEAK_SOURCE: &str = r#"
fn main() {
    if 1 < 2 {
        let then_only: int = 1;
    } else {
        let observed: int = then_only;
    }
}
"#;

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

fn assert_scope_rejections(cases: &[(&str, &str, &str)]) {
    let mut failures = Vec::new();
    for (case_name, source, leaked_name) in cases {
        match catch_unwind(|| compile(source)) {
            Err(_) => failures.push(format!("{case_name}: compile_program unwound")),
            Ok(Ok(_)) => failures.push(format!("{case_name}: unexpectedly accepted")),
            Ok(Err(error)) => {
                let expected = format!("Use of undeclared variable `{leaked_name}`.");
                if !error.contains(&expected) {
                    failures.push(format!(
                        "{case_name}: diagnostic `{error}` missing `{expected}`"
                    ));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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

fn llvm_function<'a>(llvm: &'a str, signature_prefix: &str) -> &'a str {
    let start = llvm.find(signature_prefix).expect("function definition");
    let tail = &llvm[start..];
    let end = tail.find("\n}\n").expect("function terminator") + 3;
    &tail[..end]
}

fn scalar_slots_before_call<'a>(function: &'a str, callee: &str) -> Vec<&'a str> {
    let call_marker = format!("call i32 @{callee}(");
    function
        .lines()
        .take_while(|line| !line.contains(&call_marker))
        .filter_map(|line| {
            line.trim()
                .split_once(" = alloca double")
                .map(|(slot, _)| slot)
        })
        .collect()
}

fn pointer_loaded_for_i32_call<'a>(function: &'a str, callee: &str) -> &'a str {
    let call_marker = format!("call i32 @{callee}(");
    let call_line = function
        .lines()
        .find(|line| line.contains(&call_marker))
        .unwrap_or_else(|| panic!("missing downstream `{callee}` call in:\n{function}"));
    let call_tail = call_line.split_once(&call_marker).expect("call marker").1;
    let call_argument = call_tail
        .split_once(')')
        .expect("call argument terminator")
        .0
        .strip_prefix("i32 ")
        .expect("eligible i32 call argument");

    let cast_prefix = format!("{call_argument} = fptosi double ");
    let cast_line = function
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with(&cast_prefix))
        .unwrap_or_else(|| {
            panic!("call argument `{call_argument}` was not produced by fptosi in:\n{function}")
        });
    let loaded_value = cast_line
        .strip_prefix(&cast_prefix)
        .expect("cast prefix")
        .strip_suffix(" to i32")
        .expect("i32 cast suffix");

    let load_prefix = format!("{loaded_value} = load double, double* ");
    let load_line = function
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with(&load_prefix))
        .unwrap_or_else(|| {
            panic!("cast input `{loaded_value}` was not produced by a load in:\n{function}")
        });
    load_line
        .strip_prefix(&load_prefix)
        .expect("load prefix")
        .split_once(',')
        .expect("load alignment suffix")
        .0
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

fn assert_cli_scope_rejection(output: &Output, artifact: Option<&Path>, leaked_name: &str) {
    let diagnostics = combined_output(output);
    let mut failures = Vec::new();
    if output.status.success() {
        failures.push(format!("scope-invalid source exited zero: {diagnostics}"));
    }
    if diagnostics.to_ascii_lowercase().contains("panicked") {
        failures.push(format!("scope-invalid source panicked: {diagnostics}"));
    }
    if let Some(artifact) = artifact
        && artifact.exists()
    {
        failures.push(format!(
            "scope-invalid source created requested artifact {}",
            artifact.display()
        ));
    }
    let expected = format!("Use of undeclared variable `{leaked_name}`.");
    if !diagnostics.contains(&expected) {
        failures.push(format!("diagnostic `{diagnostics}` missing `{expected}`"));
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
fn public_llvm_restores_outer_numeric_binding_after_explicit_block_shadow() {
    let source = r#"
fn consume(value: i32) -> i32 {
    return value;
}

fn main() {
    let value: int = 1;
    {
        let value: float = 2.5;
    }
    let observed: i32 = consume(value);
}
"#;
    let llvm = compile(source).expect("valid explicit-block shadow should compile");
    let main = llvm_function(&llvm, "define i32 @main(");
    let slots = scalar_slots_before_call(main, "consume");
    assert!(
        slots.len() >= 2,
        "expected distinct outer and inner slots: {main}"
    );

    let loaded_slot = pointer_loaded_for_i32_call(main, "consume");
    assert_eq!(
        loaded_slot,
        slots[0],
        "post-block call loaded `{loaded_slot}` instead of outer slot `{}`; inner slots: {:?}\n{main}",
        slots[0],
        &slots[1..]
    );
}

#[test]
fn public_llvm_restores_outer_numeric_binding_after_both_if_arms_shadow() {
    let source = r#"
fn consume(value: i32) -> i32 {
    return value;
}

fn main() {
    let value: int = 1;
    if 1 < 2 {
        let value: float = 2.5;
    } else {
        let value: float = 3.5;
    }
    let observed: i32 = consume(value);
}
"#;
    let llvm = compile(source).expect("valid if-arm shadows should compile");
    let main = llvm_function(&llvm, "define i32 @main(");
    let slots = scalar_slots_before_call(main, "consume");
    assert!(
        slots.len() >= 3,
        "expected outer and both arm-local slots: {main}"
    );

    let loaded_slot = pointer_loaded_for_i32_call(main, "consume");
    assert_eq!(
        loaded_slot,
        slots[0],
        "post-if call loaded `{loaded_slot}` instead of outer slot `{}`; arm-local slots: {:?}\n{main}",
        slots[0],
        &slots[1..]
    );
}

#[test]
fn accepts_independent_same_name_declarations_in_both_if_arms() {
    let source = r#"
fn main() {
    if 1 < 2 {
        let arm_local: int = 1;
        let then_observed: int = arm_local;
    } else {
        let arm_local: int = 2;
        let else_observed: int = arm_local;
    }
}
"#;

    compile(source).expect("independent same-name declarations in if arms should compile");
}

#[test]
fn public_compile_rejects_all_cross_scope_uses_without_unwinding() {
    let cases = [
        (
            "then-arm binding used in else arm",
            ALTERNATE_ARM_SCOPE_LEAK_SOURCE,
            "then_only",
        ),
        (
            "explicit-block binding used afterward",
            r#"
fn main() {
    {
        let block_only: int = 1;
    }
    let observed: int = block_only;
}
"#,
            "block_only",
        ),
        (
            "while-body binding used afterward",
            r#"
fn main() {
    while 1 < 2 {
        let while_only: int = 1;
        break;
    }
    let observed: int = while_only;
}
"#,
            "while_only",
        ),
        (
            "for-body binding used afterward",
            r#"
fn main() {
    let values = [1];
    for item in values {
        let for_only: int = item;
    }
    let observed: int = for_only;
}
"#,
            "for_only",
        ),
        (
            "loop-body binding used afterward",
            r#"
fn main() {
    loop {
        let loop_only: int = 1;
        break;
    }
    let observed: int = loop_only;
}
"#,
            "loop_only",
        ),
        (
            "parameter used in later top-level function",
            r#"
fn first(function_parameter: int) -> int {
    return function_parameter;
}

fn second() -> int {
    return function_parameter;
}

fn main() {}
"#,
            "function_parameter",
        ),
    ];

    assert_scope_rejections(&cases);
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
fn cli_check_rejects_alternate_arm_scope_leak_without_panicking() {
    let workspace = TestWorkspace::new("alternate-arm-scope-check");
    let root = workspace.path("main.aero");
    fs::write(
        &root,
        workspace.unique_source(ALTERNATE_ARM_SCOPE_LEAK_SOURCE),
    )
    .expect("write alternate-arm scope-invalid source");

    let output = run_check(&workspace, &root);
    assert_cli_scope_rejection(&output, None, "then_only");
}

#[test]
fn cli_build_rejects_alternate_arm_scope_leak_without_panicking_or_artifact() {
    let workspace = TestWorkspace::new("alternate-arm-scope-build");
    let root = workspace.path("main.aero");
    let artifact = workspace.path("program.ll");
    fs::write(
        &root,
        workspace.unique_source(ALTERNATE_ARM_SCOPE_LEAK_SOURCE),
    )
    .expect("write alternate-arm scope-invalid source");

    let output = run_build(&workspace, &root, &artifact);
    assert_cli_scope_rejection(&output, Some(&artifact), "then_only");
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
