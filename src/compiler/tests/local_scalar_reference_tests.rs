use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_RELATIVE: &str = "examples/local_scalar_references/main.aero";
const EXAMPLE_MODULE_RELATIVE: &str = "examples/local_scalar_references/data.aero";
const WORKFLOW_RELATIVE: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 127;

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
            "aero-local-scalar-references-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create local scalar-reference workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-local-scalar-references-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn analyzed_ast(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(ast).map(|(_, ast)| ast)
}

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed_ast(source)?)
        .map_err(|error| error.to_string())?;
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .map_err(|error| error.to_string())?;
    Ok((checked, llvm))
}

fn expect_success(label: &str, source: &str, required: &[&str]) -> Vec<String> {
    match compile_program(source, CompilerOptions::default()) {
        Err(error) => vec![format!("{label}: expected success, got {error}")],
        Ok(llvm) => required
            .iter()
            .filter(|fragment| !llvm.contains(**fragment))
            .map(|fragment| format!("{label}: LLVM missing {fragment:?}\n{llvm}"))
            .collect(),
    }
}

fn expect_rejection(label: &str, source: &str, expected: &str) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: unsupported reference source compiled:\n{llvm}"
        )),
        Err(error) if error.contains(expected) => None,
        Err(error) => Some(format!(
            "{label}: expected diagnostic containing {expected:?}, got {error:?}"
        )),
    }
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

fn positive_source() -> &'static str {
    r#"
fn read_parameter(value: int) -> int {
    let reference = &value;
    return *reference;
}

fn main() -> int {
    let integer = 40;
    let integer_ref: &int = &integer;
    let integer_alias = integer_ref;
    let first = &integer;
    let second = &integer;
    let ratio = 1.5;
    let ratio_ref: &float = &ratio;
    let ready = integer > 0;
    let ready_ref: &bool = &ready;
    let immediate = *&integer;
    {
        let nested = integer_alias;
        let nested_value = *nested;
        println!("nested {}", nested_value);
    }
    if *ready_ref && *ratio_ref > 1.0 && *first + *second == 80
        && integer == 40 && immediate == 40 {
        return read_parameter(*integer_alias) + 87;
    }
    return 1;
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod data;

struct Envelope { row: ModuleRow, rows: [ModuleRow; 2] }
fn roundtrip(value: Envelope) -> Envelope { value }

fn main() -> int {
    let integer = module_value(module_make(20));
    let integer_ref: &int = &integer;
    let integer_alias = integer_ref;
    let ratio = 1.5;
    let ratio_ref: &float = &ratio;
    let ready = integer > 0;
    let ready_ref: &bool = &ready;
    let envelope = roundtrip(Envelope {
        rows: [module_make(*integer_alias), module_make(*integer_ref + 1)],
        row: module_make(*integer_ref)
    });
    if *ready_ref && *ratio_ref > 1.0 && envelope.rows[1].value == 21
        && module_scan(envelope.rows) == 2 && "aero".len() == 4 {
        return *integer_ref + 107;
    }
    return 1;
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"struct ModuleRow { value: int, ready: bool }
fn module_make(value: int) -> ModuleRow {
    ModuleRow { value: value, ready: value > 0 }
}
fn module_value(row: ModuleRow) -> int { row.value }
fn module_scan(rows: [ModuleRow; 2]) -> int {
    for row in rows.iter() {
        if !row.ready { return 0; }
    }
    rows.len()
}
"#
}

#[test]
fn local_immutable_scalar_reference_class_is_complete_and_executable() {
    let mut failures = expect_success(
        "all immutable local scalar-reference origins and consumers",
        positive_source(),
        &[
            "getelementptr inbounds double, double* %ptr",
            "getelementptr inbounds i1, i1* %ptr",
            "load double, double* %ptr",
            "load i1, i1* %ptr",
            "define i32 @read_parameter(i32 %aero.arg.value)",
        ],
    );

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!(
            "checked reference metadata/LLVM generation failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            let borrow_count = debug.matches("CheckedImmutableBorrow").count();
            if borrow_count < 7 {
                failures.push(format!(
                    "checked IR retained only {borrow_count} immutable borrow instructions:\n{debug}"
                ));
            }
            for logical in ["pointee: Int", "pointee: Float", "pointee: Bool"] {
                if !debug.contains(logical) {
                    failures.push(format!("checked IR missing {logical:?}:\n{debug}"));
                }
            }
            if llvm.contains("inttoptr") || llvm.contains("ptrtoint") {
                failures.push(format!(
                    "reference lowering used an integer/pointer escape:\n{llvm}"
                ));
            }
        }
    }

    let rejected = [
        (
            "mutable borrow",
            "fn main() { let mut x = 1; let r = &mut x; let y = *r; }",
            "mutable references are not supported",
        ),
        (
            "borrowed literal",
            "fn main() { let r = &1; let y = *r; }",
            "requires an identifier place",
        ),
        (
            "borrowed computation",
            "fn main() { let x = 1; let r = &(x + 1); let y = *r; }",
            "requires an identifier place",
        ),
        (
            "borrowed dereference",
            "fn main() { let x = 1; let r = &x; let rr = &*r; }",
            "requires an identifier place",
        ),
        (
            "borrowed field",
            "struct S { value: int } fn main() { let x = S { value: 1 }; let r = &x.value; }",
            "requires an identifier place",
        ),
        (
            "borrowed index",
            "fn main() { let x = [1, 2]; let r = &x[0]; }",
            "requires an identifier place",
        ),
        (
            "String pointee",
            "fn main() { let x = \"aero\"; let r = &x; }",
            "support only Int, Float, or Bool pointees",
        ),
        (
            "array pointee",
            "fn main() { let x = [1, 2]; let r = &x; }",
            "support only Int, Float, or Bool pointees",
        ),
        (
            "struct pointee",
            "struct S { x: int } fn main() { let x = S { x: 1 }; let r = &x; }",
            "support only Int, Float, or Bool pointees",
        ),
        (
            "nested reference pointee",
            "fn main() { let x = 1; let r = &x; let rr = &r; }",
            "support only Int, Float, or Bool pointees",
        ),
        (
            "exact annotation mismatch",
            "fn main() { let x = 1; let r: &float = &x; }",
            "type annotation mismatch",
        ),
        (
            "mutable annotation",
            "fn main() { let x = 1; let r: &mut int = &x; }",
            "mutable references are not supported",
        ),
        (
            "reference result ABI",
            "fn bad() -> &int { let x = 1; &x } fn main() -> int { 0 }",
            "reference results require lifetime semantics and are not supported by CORE-053",
        ),
        (
            "reference argument escape",
            "fn read(value: int) -> int { value } fn main() -> int { let x = 1; let r = &x; read(r) }",
            "parameter `value` type mismatch",
        ),
        (
            "undeclared source",
            "fn main() { let r = &missing; }",
            "undeclared variable `missing`",
        ),
        (
            "uninitialized source",
            "fn main() { let x: int; let r = &x; }",
            "uninitialized variable `x`",
        ),
        (
            "moved source",
            "fn main() { let x = \"aero\"; let moved = x; let r = &x; }",
            "Use of moved value `x`",
        ),
    ];
    for (label, source, expected) in rejected {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    match analyzed_ast("fn main() { let x = 1; let r = &x; let y = *r; }") {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedImmutableBorrow") {
                failures.push(format!(
                    "deprecated raw generation activated checked reference lowering:\n{debug}"
                ));
            }
        }
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_RELATIVE);
    let module = root.join(EXAMPLE_MODULE_RELATIVE);
    match (fs::read_to_string(&example), fs::read_to_string(&module)) {
        (Ok(actual), Ok(module_actual)) => {
            if actual != tracked_root_source() {
                failures.push(format!(
                    "tracked root reference example drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!(
                    "tracked reference module drifted at {}",
                    module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked reference example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW_RELATIVE))
        .expect("read Rust workflow for reference integration anchor");
    for anchor in [
        "Test local scalar-reference integration example",
        "examples/local_scalar_references/main.aero",
        "cargo run -- build ../../examples/local_scalar_references/main.aero -o ../../local_scalar_references.ll",
        "if [ $exit_code -ne 127 ]; then",
        "local scalar-reference example passed with exit code 127",
    ] {
        let count = workflow.matches(anchor).count();
        if count != 1 {
            failures.push(format!(
                "Rust workflow anchor {anchor:?} occurs {count} times instead of once"
            ));
        }
    }

    if example.is_file() && module.is_file() {
        let workspace = TestWorkspace::new("tracked-example");
        let output_path = workspace.path("reference.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked reference example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = workspace.path("invalid.aero");
    let output_path = workspace.path("invalid.ll");
    fs::write(&invalid, "fn main() { let r = &1; let y = *r; }")
        .expect("write invalid reference source");
    let output = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&output);
    if output.status.success()
        || output_path.exists()
        || !diagnostics.contains("requires an identifier place")
    {
        failures.push(format!(
            "invalid reference CLI hygiene failed (status={}, artifact={}):\n{}",
            output.status,
            output_path.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-048 local scalar-reference failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
