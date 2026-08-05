use compiler::ast::AstNode;
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/mutable_reference_parameter_transport/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_reference_parameter_transport/mutators.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 251;

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
            "aero-mutable-reference-parameter-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create mutable-reference parameter test workspace");
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
            .is_some_and(|name| name.starts_with("aero-mutable-reference-parameter-"));
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

fn parsed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed_ast(source)?)
        .map(|(_, ast)| ast)
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
            "{label}: unsupported mutable-reference parameter form compiled:\n{llvm}"
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
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn tracked_root_source() -> &'static str {
    r#"mod mutators;

enum Signal { Idle, Count(int), Ready(bool) }
enum Mode { Off, On }
struct Packet { value: int, ready: bool }

fn read(value: &int) -> int { *value }
fn signal_score(value: Signal) -> int {
    match value {
        Signal::Idle => 1,
        Signal::Count(inner) => inner,
        Signal::Ready(inner) => 3
    }
}
fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 6 } }

fn main() -> int {
    let base = 20;
    let packet = Packet { value: 4, ready: 1 == 1 };
    let rows = [packet, packet];

    let mut direct = 130;
    { let alias = &mut direct; *alias = *alias + 8; }
    let raised = raise(&mut direct);

    let mut ratio = 2.0;
    let adjusted_ratio = adjust_ratio(&mut ratio);
    let mut ready = 1 == 2;
    set_ready(&mut ready);

    let mut ratio_score = 0;
    if adjusted_ratio > 2.0 { ratio_score = 30; }
    let mut ready_score = 0;
    if ready { ready_score = 5; }
    let signal = signal_score(Signal::Count(30));
    let mode = mode_score(Mode::On);
    if packet.ready && raised == 150 && read(&direct) == 150 && "aero".len() == 4 {
        return raised + ratio_score + ready_score + read(&base) + signal + mode + rows.len() + packet.value + "aero".len();
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn raise(value: &mut int) -> int {
    let mut index = 0;
    while index < 3 {
        *value = *value + index;
        index = index + 1;
    }
    if *value > 0 { *value = *value + 9; } else { *value = 0; }
    *value
}

fn adjust_ratio(value: &mut float) -> float {
    *value = *value + 0.5;
    *value
}

fn set_ready(value: &mut bool) {
    *value = 1 == 1;
}
"#
}

#[test]
fn direct_call_mutable_scalar_reference_parameter_class_is_complete() {
    let mut failures = Vec::new();

    let parser_source = r#"
fn bump(value: &mut int) -> int { *value = *value + 1; *value }
fn main() -> int { let mut value = 1; bump(&mut value) }
"#;
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!(
            "parser mutable parameter retention failed: {error}"
        )),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            if !debug.contains("Reference")
                || debug.matches("mutable: true").count() < 2
                || !debug.contains("Deref")
                || !debug.contains("Assignment")
            {
                failures.push(format!(
                    "parser lost mutable parameter/call/dereference topology:\n{debug}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "Int mutable parameter write and result",
            "fn bump(value: &mut int) -> int { *value = *value + 1; *value } fn main() -> int { let mut value = 4; bump(&mut value) + value }",
            vec!["double*", "call i32 @bump", "store double"],
        ),
        (
            "Float mutable parameter",
            "fn adjust(value: &mut float) -> float { *value = *value + 0.5; *value } fn main() -> int { let mut value = 1.5; let result = adjust(&mut value); if result == 2.0 && value == 2.0 { return 7; } 1 }",
            vec![
                "define double @adjust(double*",
                "call double @adjust(double*",
            ],
        ),
        (
            "Bool mutable parameter Void result",
            "fn set(value: &mut bool) { *value = 1 == 1; } fn main() -> int { let mut value = 1 == 2; set(&mut value); if value { return 9; } 1 }",
            vec!["define void @set(i1*", "call void @set(i1*", "store i1"],
        ),
        (
            "repeated calls and owner reuse",
            "fn bump(value: &mut int) { *value = *value + 1; } fn main() -> int { let mut value = 1; bump(&mut value); value = value + 2; bump(&mut value); value }",
            vec!["call void @bump"],
        ),
        (
            "callee branch and loop CFG",
            "fn advance(value: &mut int) -> int { let mut i = 0; while i < 3 { *value = *value + i; i = i + 1; } if *value > 0 { *value = *value + 4; } else { *value = 0; } *value } fn main() -> int { let mut value = 2; advance(&mut value) }",
            vec!["while_start", "if_then", "store double"],
        ),
        (
            "forward-declared mutable function",
            "fn main() -> int { let mut value = 3; change(&mut value) } fn change(value: &mut int) -> int { *value = 8; *value }",
            vec!["call i32 @change(double*"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    let checked_source = "fn bump(value: &mut int) -> int { *value = *value + 1; *value } fn main() -> int { let mut value = 4; let result = bump(&mut value); value + result }";
    match checked_ir_and_llvm(checked_source) {
        Err(error) => failures.push(format!("checked mutable parameter case failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for identity in [
                "MutableReference",
                "CheckedMutableReferenceParameter",
                "CheckedMutableBorrow",
                "CheckedMutableDereferenceAssignment",
                "CheckedMutableBorrowEnd",
            ] {
                if !debug.contains(identity) {
                    failures.push(format!("checked IR missing {identity}:\n{debug}"));
                }
            }
            for anchor in [
                "define i32 @bump(double*",
                "call i32 @bump(double*",
                "store double",
                "load double",
            ] {
                if !llvm.contains(anchor) {
                    failures.push(format!(
                        "mutable parameter LLVM missing {anchor:?}:\n{llvm}"
                    ));
                }
            }
            if llvm.contains("inttoptr") || llvm.contains("ptrtoint") {
                failures.push(format!(
                    "mutable parameter lowering used pointer/integer conversion:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "stored alias argument",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; let alias = &mut value; bump(alias) }",
            "mutable reference calls require a direct `&mut` local owner argument",
        ),
        (
            "forwarded mutable parameter",
            "fn inner(value: &mut int) -> int { *value } fn outer(value: &mut int) -> int { inner(value) } fn main() -> int { let mut value = 1; outer(&mut value) }",
            "mutable reference calls require a direct `&mut` local owner argument",
        ),
        (
            "two mutable parameters",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { 0 }",
            "exactly one mutable scalar-reference parameter",
        ),
        (
            "mixed mutable and scalar parameters",
            "fn bad(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { 0 }",
            "exactly one mutable scalar-reference parameter",
        ),
        (
            "mixed mutable and immutable references",
            "fn bad(value: &mut int, other: &int) -> int { *value + *other } fn main() -> int { 0 }",
            "exactly one mutable scalar-reference parameter",
        ),
        (
            "mutable reference result",
            "fn bad(value: &mut int) -> &mut int { value } fn main() -> int { 0 }",
            "reference results require lifetime semantics",
        ),
        (
            "main mutable parameter",
            "fn main(value: &mut int) -> int { *value }",
            "process entry cannot use reference parameters",
        ),
        (
            "generic mutable parameter",
            "fn bad<T>(value: &mut int) -> int { *value } fn main() -> int { 0 }",
            "generic reference transport functions are not supported",
        ),
        (
            "immutable owner",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let value = 1; bump(&mut value) }",
            "must be declared mutable",
        ),
        (
            "uninitialized owner",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value: int; bump(&mut value) }",
            "uninitialized variable `value`",
        ),
        (
            "borrowed owner",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; let shared = &value; bump(&mut value) + *shared }",
            "also borrowed as immutable",
        ),
        (
            "temporary argument",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { bump(&mut (1 + 2)) }",
            "requires an identifier place",
        ),
        (
            "immutable borrow argument",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; bump(&value) }",
            "direct `&mut` local owner argument",
        ),
        (
            "scalar argument",
            "fn bump(value: &mut int) -> int { *value } fn main() -> int { let mut value = 1; bump(value) }",
            "direct `&mut` local owner argument",
        ),
        (
            "String mutable parameter",
            "fn bad(value: &mut String) -> int { 0 } fn main() -> int { 0 }",
            "mutable reference parameters support only Int, Float, or Bool pointees",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    for source in [
        "fn read(value: &int) -> int { *value } fn main() -> int { let value = 7; read(&value) }",
        "fn main() -> int { let mut value = 1; let alias = &mut value; *alias = 2; *alias }",
    ] {
        if let Err(error) = compile_program(source, CompilerOptions::default()) {
            failures.push(format!(
                "accepted reference compatibility regressed: {error}"
            ));
        }
    }

    match analyzed_ast(checked_source) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:#?}");
            if debug.contains("CheckedMutableReferenceParameter")
                || debug.contains("CheckedMutableBorrowEnd")
            {
                failures.push(format!(
                    "deprecated raw generation activated mutable parameter identities:\n{debug}"
                ));
            }
        }
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_ROOT);
    let module = root.join(EXAMPLE_MODULE);
    match (fs::read_to_string(&example), fs::read_to_string(&module)) {
        (Ok(actual), Ok(module_actual)) => {
            if actual != tracked_root_source() {
                failures.push(format!(
                    "tracked root example drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!("tracked module drifted at {}", module.display()));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked mutable parameter example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for mutable parameter integration anchor");
    for anchor in [
        "Test mutable scalar-reference parameter transport integration example",
        "examples/mutable_reference_parameter_transport/main.aero",
        "cargo run -- build ../../examples/mutable_reference_parameter_transport/main.aero -o ../../mutable_reference_parameter_transport.ll",
        "if [ $exit_code -ne 251 ]; then",
        "mutable scalar-reference parameter transport example passed with exit code 251",
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
        let output_path = workspace.path("mutable_reference_parameter_transport.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked mutable parameter example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = workspace.path("invalid.aero");
    let output_path = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn bump(value: &mut int) -> int { *value } fn main() -> int { let value = 1; bump(&mut value) }",
    )
    .expect("write invalid mutable parameter source");
    let output = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&output);
    if output.status.success()
        || output_path.exists()
        || !diagnostics.contains("must be declared mutable")
    {
        failures.push(format!(
            "invalid mutable parameter CLI hygiene failed (status={}, artifact={}):\n{}",
            output.status,
            output_path.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-056 mutable scalar-reference parameter failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
