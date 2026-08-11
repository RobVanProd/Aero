use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/owned_enum_reassignment/main.aero";
const EXAMPLE_MODULE: &str = "examples/owned_enum_reassignment/values.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 131;

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
            "aero-owned-enum-reassignment-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create owned enum reassignment workspace");
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
            .is_some_and(|name| name.starts_with("aero-owned-enum-reassignment-"));
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

fn parsed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source)?)
        .map(|(_, ast)| ast)
}

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed(source)?)
        .map_err(|error| error.to_string())?;
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .map_err(|error| error.to_string())?;
    Ok((checked, llvm))
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

fn complete_source() -> &'static str {
    r#"
struct Cell { value: int, flags: [bool; 2] }

enum Phase { Cold, Warm, Hot }
enum Measure { Count(int), Ratio(float), Flag(bool) }
enum Payload {
    Idle,
    Flag(bool),
    Pair((int, bool)),
    CellValue(Cell),
    Cells([Cell; 2]),
    Matrix([[int; 2]; 2])
}

fn make_cell(value: int) -> Cell {
    Cell { value: value, flags: [value > 0, value < 0] }
}

fn produce_pair(value: int) -> Payload {
    Payload::Pair((value, value > 0))
}

fn bool_score(value: bool, score: int) -> int {
    if value { return score; }
    0
}

fn phase_score(value: Phase) -> int {
    match value {
        Phase::Cold => 1,
        Phase::Warm => 2,
        Phase::Hot => 3
    }
}

fn measure_score(value: Measure) -> int {
    match value {
        Measure::Count(count) => count,
        Measure::Ratio(ratio) => 4,
        Measure::Flag(flag) => bool_score(flag, 5)
    }
}

fn score(value: Payload) -> int {
    match value {
        Payload::Idle => 1,
        Payload::Flag(flag) => bool_score(flag, 2),
        Payload::Pair(pair) => pair.0,
        Payload::CellValue(cell) => cell.value,
        Payload::Cells(cells) => cells[1].value,
        Payload::Matrix(matrix) => matrix[1][0]
    }
}

fn finish(value: Payload) -> int { score(value) }

fn main() -> int {
    let mut phase = Phase::Cold;
    phase = Phase::Hot;

    let mut measure: Measure = Measure::Count(1);
    measure = Measure::Flag(1 < 2);

    let mut repeated = Payload::Idle;
    repeated = Payload::Pair((11, 1 < 2));
    repeated = Payload::CellValue(make_cell(17));

    let mut called: Payload = Payload::Idle;
    called = produce_pair(19);

    let mut nested = Payload::Idle;
    {
        nested = Payload::Matrix([[20, 21], [23, 24]]);
    }

    let source = Payload::Cells([make_cell(59), make_cell(64)]);
    let mut moved = Payload::Idle;
    moved = source;

    let total = phase_score(phase)
        + measure_score(measure)
        + score(repeated)
        + score(called)
        + score(nested)
        + finish(moved);
    if total == 131 { return 131; }
    1
}
"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded owned-enum reassignment compiled:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => None,
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
        )),
    }
}

#[test]
fn owned_enum_reassignment_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!(
            "owned-enum reassignment syntax was not retained: {error}"
        ));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("complete owned-enum reassignment failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedMutableOwnedPlaceAlloca",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumVariant",
                "CheckedEnumDispatch",
                "Enum {",
                "Array {",
                "Tuple {",
                "Struct {",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked owned-enum IR missing {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "owned-enum LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            if !llvm.contains("load ") || !llvm.contains("store ") || !llvm.contains("switch i32") {
                failures.push(format!(
                    "owned-enum LLVM lost typed load/store/dispatch evidence:\n{llvm}"
                ));
            }
        }
    }

    if let Err(error) = compile_program(
        "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() -> int { let mut target = E::A; loop { let score = take(target); if 1 < 2 { target = E::B; } break; } 0 }",
        CompilerOptions::default(),
    ) {
        failures.push(format!(
            "consumed-target reassignment on an always-terminating loop path unexpectedly failed: {error}"
        ));
    }

    for (label, source, expected) in [
        (
            "immutable target",
            "enum E { A, B } fn main() { let value = E::A; value = E::B; }",
            vec!["mutable local owned binding"],
        ),
        (
            "uninitialized target",
            "enum E { A, B } fn main() { let mut value: E; value = E::B; }",
            vec!["must already be initialized", "uninitialized"],
        ),
        (
            "wrong enum identity",
            "enum E { A } enum F { A } fn main() { let mut value = E::A; value = F::A; }",
            vec!["type mismatch"],
        ),
        (
            "direct self replacement",
            "enum E { A } fn main() { let mut value = E::A; value = value; }",
            vec!["self-replacement"],
        ),
        (
            "reuse moved assignment source",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() -> int { let source = E::B; let mut target = E::A; target = source; take(source) }",
            vec!["moved"],
        ),
        (
            "assignment while immutable enum loan remains live",
            "enum E { A } fn main() { let mut value = E::A; let alias = &value; value = E::A; }",
            vec!["borrowed"],
        ),
        (
            "enum field storage remains excluded",
            "enum E { A } struct Boxed { value: E } fn main() { let value = Boxed { value: E::A }; }",
            vec![
                "Struct construction expressions",
                "not an admitted",
                "unsupported",
            ],
        ),
        (
            "enum array storage remains excluded",
            "enum E { A } fn main() { let values = [E::A]; }",
            vec!["not admitted", "array"],
        ),
        (
            "inferred generic enum remains excluded",
            "enum E<T> { A(T) } fn main() { let mut value = E::A(1); value = E::A(2); }",
            vec!["requires an exact expected E<...> type"],
        ),
        (
            "multi-field constructor requires exact arity",
            "enum E { A(int, bool) } fn main() { let mut value = E::A((1, 1 < 2)); }",
            vec!["requires 2 positional field", "actual 1"],
        ),
        (
            "unsupported payload leaf remains excluded",
            "enum E { A(String) } fn main() { let mut value = E::A(\"a\"); value = E::A(\"b\"); }",
            vec!["not an admitted", "unsupported"],
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, &expected) {
            failures.push(failure);
        }
    }

    let raw = IrGenerator::new().generate_ir(parsed(source).expect("complete source parses"));
    let raw_debug = format!("{raw:#?}");
    for marker in [
        "CheckedMutableOwnedPlaceAlloca",
        "CheckedOwnedPlaceAssignment",
    ] {
        if raw_debug.contains(marker) {
            failures.push(format!(
                "deprecated raw generation activated checked identity {marker}:\n{raw_debug}"
            ));
        }
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { A } fn main() { let value = E::A; value = E::A; }",
    )
    .expect("write invalid owned-enum reassignment source");
    let invalid_build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    if invalid_build.status.success() || invalid_artifact.exists() {
        failures.push(format!(
            "invalid owned-enum CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for path in [&tracked_root, &tracked_module] {
        if !path.is_file() {
            failures.push(format!(
                "tracked owned-enum example missing: {}",
                path.display()
            ));
        }
    }
    if tracked_root.is_file() && tracked_module.is_file() {
        let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
        if !check.status.success() {
            failures.push(format!(
                "tracked owned-enum direct-module check failed: {}",
                output_text(&check)
            ));
        }
        let output = workspace.path("owned-enum-reassignment.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
        );
        if !build.status.success() || !output.is_file() {
            failures.push(format!(
                "tracked owned-enum direct-module build failed (artifact={}): {}",
                output.is_file(),
                output_text(&build)
            ));
        }
    }

    let workflow_path = root.join(WORKFLOW);
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not read {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test owned enum reassignment integration example",
                "examples/owned_enum_reassignment/main.aero",
                "opt-22 -passes=verify -disable-output ../../owned_enum_reassignment.ll",
                "llc-22 -verify-machineinstrs ../../owned_enum_reassignment.ll",
                "clang-22 -no-pie ../../owned_enum_reassignment.o -o ../../owned_enum_reassignment",
                "Expected exit code 131",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-064 owned enum reassignment failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
