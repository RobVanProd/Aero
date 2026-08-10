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

const EXAMPLE_ROOT: &str = "examples/scalar_reassignment/main.aero";
const EXAMPLE_MODULE: &str = "examples/scalar_reassignment/mutation.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 227;

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
            "aero-scalar-reassignment-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create scalar-reassignment workspace");
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
            .is_some_and(|name| name.starts_with("aero-scalar-reassignment-"));
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
            "{label}: unsupported scalar reassignment compiled:\n{llvm}"
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
fn once(value: int) -> int { value + 1 }
fn mutate(start: int) -> int {
    let mut value: int = start;
    value = once(value);
    value = value + 2;
    let mut ratio: float = 1.5;
    ratio = ratio + 0.5;
    let mut ready: bool = value > 0;
    ready = ratio > 1.0;
    {
        let mut value = 10;
        value = value + 1;
        if value != 11 { return 1; }
    }
    let mut index = 0;
    while index < 3 {
        value = value + index;
        index = index + 1;
    }
    for item in [1, 2] {
        value = value + item;
    }
    if ready {
        value = value + 4;
    } else {
        value = 0;
    }
    value
}
fn main() -> int { mutate(1) }
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod mutation;

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
    let mutated = mutation_score(100, 4, 2.0, 1 == 1);
    let signal = signal_score(Signal::Count(30));
    let mode = mode_score(Mode::On);
    if packet.ready && mutated == 161 && "aero".len() == 4 {
        return mutated + read(&base) + signal + mode + rows.len() + packet.value + "aero".len();
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn accumulate(start: int, limit: int) -> int {
    let mut total = start;
    let mut index = 0;
    while index < limit {
        total = total + index;
        index = index + 1;
    }
    total
}
fn choose(value: int, ready: bool) -> int {
    let mut result = value;
    if ready { result = result + 20; } else { result = result + 1; }
    result
}
fn ratio_score(start: float) -> int {
    let mut ratio = start;
    ratio = ratio + 0.5;
    if ratio > 2.0 { return 30; }
    0
}
fn bool_score(input: bool) -> int {
    let mut ready = 1 == 2;
    ready = input;
    if ready { return 5; }
    0
}
fn mutation_score(start: int, limit: int, ratio: float, ready: bool) -> int {
    choose(accumulate(start, limit), ready) + ratio_score(ratio) + bool_score(ready)
}
"#
}

#[test]
fn mutable_local_scalar_reassignment_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
fn main() -> int {
    let mut value = 1;
    let mut row = 2;
    let mut items = [1, 2];
    let alias = &value;
    value = 3;
    row.value = 4;
    items[0] = 5;
    *alias = 6;
    value
}
"#;
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!("parser assignment retention failed: {error}")),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            if debug.matches("Assignment").count() != 4
                || !debug.contains("FieldAccess")
                || !debug.contains("IndexAccess")
                || !debug.contains("Deref")
            {
                failures.push(format!(
                    "parser changed assignment target topology or count:\n{debug}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "complete scalar reassignment composition",
            positive_source(),
            vec![
                "define i32 @mutate(i32 %aero.arg.start)",
                "call i32 @once(i32",
                "while_start",
                "store double",
                "store i1",
            ],
        ),
        (
            "inferred Int sequential reassignment",
            "fn main() -> int { let mut value = 1; value = 2; value = value + 3; value }",
            vec![
                "alloca double",
                "store double 0x4000000000000000",
                "load double",
            ],
        ),
        (
            "annotated Float reassignment",
            "fn main() -> int { let mut value: float = 1.0; value = value + 2.0; if value == 3.0 { return 9; } 0 }",
            vec!["alloca double", "store double", "fcmp"],
        ),
        (
            "annotated Bool reassignment",
            "fn main() -> int { let mut ready: bool = 1 == 2; ready = 2 > 1; if ready { return 8; } 0 }",
            vec!["alloca i1", "store i1", "load i1"],
        ),
        (
            "nearest shadowed mutable binding",
            "fn main() -> int { let mut value = 1; { let mut value = 4; value = 7; if value != 7 { return 1; } } value = 9; value }",
            vec![
                "alloca double",
                "store double 0x401C000000000000",
                "store double 0x4022000000000000",
            ],
        ),
        (
            "selected branch reassignment",
            "fn main() -> int { let mut value = 1; if value == 1 { value = 12; } else { value = 2; } value }",
            vec!["if_then", "if_else", "store double 0x4028000000000000"],
        ),
        (
            "while-carried reassignment",
            "fn main() -> int { let mut value = 0; while value < 4 { value = value + 1; } value }",
            vec!["while_start", "while_body", "store double"],
        ),
        (
            "compiler-bounded for-body reassignment",
            "fn main() -> int { let mut total = 0; for item in [1, 2, 3] { total = total + item; } total }",
            vec!["store double", "load double"],
        ),
        (
            "superseding Copy-data array reassignment",
            "fn main() -> int { let mut value = [1, 2]; value = [3, 4]; value[1] }",
            vec!["alloca [2 x double]", "store [2 x double]"],
        ),
        (
            "superseding Copy-data struct reassignment",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row = Row { value: 2 }; row.value }",
            vec!["alloca %aero.struct.Row", "store %aero.struct.Row"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked reassignment IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            if debug.matches("CheckedMutableOwnedPlaceAlloca").count() < 5 {
                failures.push(format!(
                    "checked IR lost mutable scalar place declarations:\n{debug}"
                ));
            }
            if debug.matches("CheckedOwnedPlaceAssignment").count() < 9
                || !debug.contains("ty: Int")
                || !debug.contains("ty: Float")
                || !debug.contains("ty: Bool")
            {
                failures.push(format!(
                    "checked IR lost exact scalar reassignment metadata:\n{debug}"
                ));
            }
            if llvm.matches("call i32 @once(").count() != 1 {
                failures.push(format!(
                    "reassignment RHS call was not emitted exactly once:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "immutable local",
            "fn main() -> int { let value = 1; value = 2; value }",
            "assignment target `value` must be a mutable local owned binding",
        ),
        (
            "immutable parameter",
            "fn change(value: int) -> int { value = 2; value } fn main() -> int { change(1) }",
            "assignment target `value` must be a mutable local owned binding",
        ),
        (
            "unknown target",
            "fn main() -> int { missing = 2; 0 }",
            "assignment target `missing` is not an initialized local binding",
        ),
        (
            "uninitialized target",
            "fn main() -> int { let mut value: int; value = 2; value }",
            "assignment target `value` must already be initialized",
        ),
        (
            "wrong Float RHS",
            "fn main() -> int { let mut value = 1; value = 2.0; value }",
            "assignment to `value` type mismatch: expected int, actual float",
        ),
        (
            "wrong Bool RHS",
            "fn main() -> int { let mut ready = 1 == 1; ready = 1; 0 }",
            "assignment to `ready` type mismatch: expected bool, actual int",
        ),
        (
            "String target",
            "fn main() -> int { let mut value = \"a\"; value = \"b\"; 0 }",
            "type String is not admitted Copy-data for owned assignment",
        ),
        (
            "enum target",
            "enum Mode { Off, On } fn main() -> int { let mode = Mode::Off; mode = Mode::On; 0 }",
            "assignment target `mode` must be a mutable local owned binding",
        ),
        (
            "immutably borrowed target",
            "fn main() -> int { let mut value = 1; let alias = &value; value = 2; *alias }",
            "cannot assign to `value` while it is borrowed",
        ),
        (
            "field target",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; row.value = 2; 0 }",
            "projected assignment root `row` must be a mutable local owned binding",
        ),
        (
            "index target",
            "fn main() -> int { let items = [1, 2]; items[0] = 3; 0 }",
            "projected assignment root `items` must be a mutable local owned binding",
        ),
        (
            "immutable dereference target",
            "fn main() -> int { let mut value = 1; let alias = &value; *alias = 2; value }",
            "assignment through an immutable reference is not supported",
        ),
        (
            "top-level assignment",
            "let mut value = 1; value = 2; fn main() -> int { value }",
            "owned-place reassignment is supported only inside admitted function bodies",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    for (label, source) in [
        (
            "compound assignment remains contained",
            "fn main() -> int { let mut value = 1; value += 2; value }",
        ),
        (
            "chained assignment remains contained",
            "fn main() -> int { let mut left = 1; let mut right = 2; left = right = 3; left }",
        ),
        (
            "assignment value remains contained",
            "fn main() -> int { let mut value = 1; let result = (value = 2); result }",
        ),
    ] {
        if compile_program(source, CompilerOptions::default()).is_ok() {
            failures.push(format!("{label}: unsupported assignment syntax compiled"));
        }
    }

    match analyzed_ast(positive_source()) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:#?}");
            if debug.contains("CheckedMutableOwnedPlaceAlloca")
                || debug.contains("CheckedOwnedPlaceAssignment")
            {
                failures.push(format!(
                    "deprecated raw generation activated checked reassignment:\n{debug}"
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
                    "tracked root scalar-reassignment example drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!(
                    "tracked scalar-reassignment module drifted at {}",
                    module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked scalar-reassignment example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for scalar-reassignment integration anchor");
    for anchor in [
        "Test mutable scalar reassignment integration example",
        "examples/scalar_reassignment/main.aero",
        "cargo run -- build ../../examples/scalar_reassignment/main.aero -o ../../scalar_reassignment.ll",
        "if [ $exit_code -ne 227 ]; then",
        "mutable scalar reassignment example passed with exit code 227",
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
        let output_path = workspace.path("scalar_reassignment.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked scalar-reassignment example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = workspace.path("invalid.aero");
    let output_path = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn main() -> int { let value = 1; value = 2; value }",
    )
    .expect("write invalid scalar-reassignment source");
    let output = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&output);
    if output.status.success()
        || output_path.exists()
        || !diagnostics.contains("assignment target `value` must be a mutable local owned binding")
    {
        failures.push(format!(
            "invalid scalar-reassignment CLI hygiene failed (status={}, artifact={}):\n{}",
            output.status,
            output_path.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-054 mutable local scalar reassignment failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
