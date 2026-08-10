use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/conditional_enum_ownership/main.aero";
const EXAMPLE_MODULE: &str = "examples/conditional_enum_ownership/flows.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 137;

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
            "aero-conditional-enum-ownership-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create conditional enum ownership workspace");
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
            .is_some_and(|name| name.starts_with("aero-conditional-enum-ownership-"));
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

fn phase_branch(flag: bool, value: Phase) -> int {
    if flag { return phase_score(value); }
    else { return phase_score(value); }
}

fn measure_branch(flag: bool, value: Measure) -> int {
    if flag { return measure_score(value); }
    else { return measure_score(value); }
}

fn payload_branch(flag: bool, value: Payload) -> int {
    if flag { return score(value); }
    else { return score(value); }
}

fn returning_then(flag: bool, value: Payload) -> int {
    if flag { return score(value); }
    score(value)
}

fn replace_both(flag: bool, value: Payload) -> int {
    let mut target: Payload = Payload::Idle;
    if flag { target = value; }
    else { target = value; }
    score(target)
}

fn replace_constructors(flag: bool) -> int {
    let mut target = Payload::Idle;
    if flag { target = Payload::Pair((7, 1 < 2)); }
    else { target = produce_pair(8); }
    score(target)
}

fn nested(which: int, value: Payload) -> int {
    if which == 0 { return score(value); }
    else if which == 1 { return score(value); }
    else { return score(value); }
}

fn unchanged_no_else(flag: bool, value: Payload) -> int {
    if flag { let local = 1; }
    score(value)
}

fn shadowed(flag: bool, value: Payload) -> int {
    if flag {
        let value = Payload::Pair((9, 1 < 2));
        return score(value);
    }
    else { return score(value); }
}

fn independent(flag: bool, left: Payload, right: Payload) -> int {
    if flag { let moved = left; }
    else { let moved = right; }
    0
}

fn main() -> int {
    let exact: Payload = Payload::Pair((19, 1 < 2));
    let total = phase_branch(1 < 2, Phase::Hot)
        + measure_branch(1 > 2, Measure::Flag(1 < 2))
        + payload_branch(1 < 2, Payload::Cells([make_cell(59), make_cell(64)]))
        + replace_both(1 < 2, exact)
        + nested(1, Payload::Pair((23, 1 < 2)))
        + returning_then(1 < 2, Payload::Pair((23, 1 < 2)));
    if total == 137 { return 137; }
    1
}
"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded conditional enum ownership compiled:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => None,
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
        )),
    }
}

#[test]
fn conditional_enum_ownership_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!(
            "conditional enum ownership syntax was not retained: {error}"
        ));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("complete conditional enum flow failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedEnumParameter",
                "CheckedMutableOwnedPlaceAlloca",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumDispatch",
                "if_then_",
                "if_else_",
                "if_end_",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked conditional enum IR missing {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "conditional enum LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            if !llvm.contains("br i1")
                || !llvm.contains("switch i32")
                || !llvm.contains("load ")
                || !llvm.contains("store ")
            {
                failures.push(format!(
                    "conditional enum LLVM lost CFG/load/store/dispatch evidence:\n{llvm}"
                ));
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "conditional enum LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic conditional enum compilation failed: {error}"
                )),
            }
        }
    }

    let reinitialized_join = "enum E { A, B } fn main() { let mut target = E::A; if 1 < 2 { let moved = target; } target = E::B; }";
    if let Err(error) = checked_ir_and_llvm(reinitialized_join) {
        failures.push(format!(
            "CORE-073 whole-place write did not close the prior maybe-moved target: {error}"
        ));
    }

    for (label, source, expected) in [
        (
            "no-else partial move then Match",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() -> int { let value = E::A; if 1 < 2 { let moved = value; } take(value) }",
            vec!["may have been moved"],
        ),
        (
            "one fallthrough arm moves before call",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() -> int { let value = E::A; if 1 < 2 { let moved = value; } else { let keep = 1; } take(value) }",
            vec!["may have been moved"],
        ),
        (
            "both fallthrough arms move before return",
            "enum E { A, B } fn bad() -> E { let value = E::A; if 1 < 2 { let moved = value; } else { let moved = value; } value } fn main() -> int { match bad() { E::A => 1, E::B => 2 } }",
            vec!["moved value `value`", "Use of moved value"],
        ),
        (
            "nested partial move",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() -> int { let value = E::A; if 1 < 2 { if 2 < 3 { let moved = value; } } take(value) }",
            vec!["may have been moved"],
        ),
        (
            "maybe-moved borrow",
            "enum E { A } fn main() { let value = E::A; if 1 < 2 { let moved = value; } let alias = &value; }",
            vec!["may have been moved"],
        ),
        (
            "maybe-moved assignment source",
            "enum E { A, B } fn main() { let source = E::A; let mut target = E::B; if 1 < 2 { let moved = source; } target = source; }",
            vec!["may have been moved"],
        ),
        (
            "loop-carried conditional move",
            "enum E { A, B } fn main() { let value = E::A; let mut step = 0; while step < 2 { if step == 0 { let moved = value; } step = step + 1; } }",
            vec!["may have been moved", "moved value"],
        ),
        (
            "loop condition consumption without reinitialization",
            "enum E { A, B } fn main() -> int { let value = E::A; while match value { E::A => 1 < 2, E::B => 2 < 1 } { return 1; } match value { E::A => 2, E::B => 3 } }",
            vec!["Use of moved value", "moved value"],
        ),
        (
            "enum field storage remains excluded",
            "enum E { A } struct Boxed { value: E } fn main() { let value = Boxed { value: E::A }; }",
            vec!["not an admitted", "unsupported", "Struct construction"],
        ),
        (
            "enum array storage remains excluded",
            "enum E { A } fn main() { let values = [E::A]; }",
            vec!["not admitted", "array"],
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, &expected) {
            failures.push(failure);
        }
    }

    let raw = IrGenerator::new()
        .generate_ir(parsed("fn main() -> int { 0 }").expect("raw compatibility sentinel parses"));
    let raw_debug = format!("{raw:#?}");
    for marker in [
        "CheckedEnumParameter",
        "CheckedMutableOwnedPlaceAlloca",
        "CheckedOwnedPlaceAssignment",
        "CheckedEnumDispatch",
    ] {
        if raw_debug.contains(marker) {
            failures.push(format!(
                "raw generation activated checked conditional identity {marker}:\n{raw_debug}"
            ));
        }
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() -> int { let value = E::A; if 1 < 2 { let moved = value; } take(value) }",
    )
    .expect("write invalid conditional enum ownership source");
    let invalid_check = run_cli(&workspace, &[Path::new("check"), &invalid]);
    if invalid_check.status.success() {
        failures.push(format!(
            "invalid conditional enum CLI check succeeded: {}",
            output_text(&invalid_check)
        ));
    }
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
            "invalid conditional enum CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for path in [&tracked_root, &tracked_module] {
        if !path.is_file() {
            failures.push(format!(
                "tracked conditional enum example missing: {}",
                path.display()
            ));
        }
    }
    if tracked_root.is_file() && tracked_module.is_file() {
        let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
        if !check.status.success() {
            failures.push(format!(
                "tracked conditional enum direct-module check failed: {}",
                output_text(&check)
            ));
        }
        let output = workspace.path("conditional-enum-ownership.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
        );
        if !build.status.success() || !output.is_file() {
            failures.push(format!(
                "tracked conditional enum direct-module build failed (artifact={}): {}",
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
                "Test conditional enum ownership integration example",
                "examples/conditional_enum_ownership/main.aero",
                "opt-22 -passes=verify -disable-output ../../conditional_enum_ownership.ll",
                "llc-22 -verify-machineinstrs ../../conditional_enum_ownership.ll",
                "clang-22 -no-pie ../../conditional_enum_ownership.o -o ../../conditional_enum_ownership",
                "Expected exit code 137",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-065 conditional enum ownership failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
