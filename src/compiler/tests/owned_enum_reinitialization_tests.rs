use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 199;
const EXAMPLE_ROOT: &str = "examples/owned_enum_reinitialization/main.aero";
const EXAMPLE_MODULE: &str = "examples/owned_enum_reinitialization/values.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";

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
            "aero-owned-enum-reinitialization-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create owned-enum reinitialization workspace");
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
            .is_some_and(|name| name.starts_with("aero-owned-enum-reinitialization-"));
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

fn tracked_root_source() -> &'static str {
    r#"mod values;

fn main() -> int {
    let result = reinitialize(1 < 2);
    if result == 199 { return 199; }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"enum Packet {
    Empty,
    Count(int),
    Pair(int, bool),
    Mark(char)
}

fn bool_score(value: bool, score: int) -> int {
    if value { return score; }
    0
}

fn char_score(value: char, score: int) -> int {
    if value == 'λ' { return score; }
    0
}

fn score(value: Packet) -> int {
    match value {
        Packet::Empty => 0,
        Packet::Count(number) => number,
        Packet::Pair(number, flag) => number + bool_score(flag, 2),
        Packet::Mark(glyph) => char_score(glyph, 3)
    }
}

fn reinitialize(flag: bool) -> int {
    let mut value: Packet = Packet::Pair(1, 1 < 2);
    if flag {
        let consumed = score(value);
    }
    value = Packet::Count(199);
    score(value)
}
"#
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

enum Packet {
    Empty,
    Count(int),
    Pair(int, bool),
    Mark(char),
    CellValue(Cell),
    Cells([Cell; 2]),
    Matrix([[int; 2]; 2]),
    Mixed(int, bool, char)
}

fn cell(value: int) -> Cell {
    Cell { value: value, flags: [value > 0, value < 0] }
}

fn bool_score(value: bool, score: int) -> int {
    if value { return score; }
    0
}

fn char_score(value: char, score: int) -> int {
    if value == 'λ' { return score; }
    0
}

fn score(value: Packet) -> int {
    match value {
        Packet::Empty => 0,
        Packet::Count(number) => number,
        Packet::Pair(number, flag) => number + bool_score(flag, 2),
        Packet::Mark(glyph) => char_score(glyph, 3),
        Packet::CellValue(item) => item.value,
        Packet::Cells(items) => items[1].value,
        Packet::Matrix(matrix) => matrix[1][0],
        Packet::Mixed(number, flag, glyph) =>
            number + bool_score(flag, 2) + char_score(glyph, 3)
    }
}

fn produce(value: int) -> Packet { Packet::Count(value) }

fn alias_reinitialization() -> int {
    let mut value = Packet::Count(1);
    let moved = value;
    let before = score(moved);
    value = Packet::Pair(10, 1 < 2);
    before + score(value)
}

fn call_reinitialization() -> int {
    let mut value: Packet = Packet::Cells([cell(5), cell(6)]);
    let before = score(value);
    value = produce(20);
    before + score(value)
}

fn match_reinitialization() -> int {
    let mut value = Packet::CellValue(cell(4));
    let before = match value {
        Packet::Empty => 0,
        Packet::Count(number) => number,
        Packet::Pair(number, flag) => number,
        Packet::Mark(glyph) => char_score(glyph, 3),
        Packet::CellValue(item) => item.value,
        Packet::Cells(items) => items[1].value,
        Packet::Matrix(matrix) => matrix[1][0],
        Packet::Mixed(number, flag, glyph) => number
    };
    value = Packet::Mark('λ');
    before + score(value)
}

fn assignment_reinitialization() -> int {
    let mut source = Packet::Matrix([[1, 2], [8, 9]]);
    let mut sink = Packet::Empty;
    sink = source;
    source = Packet::Mixed(30, 1 < 2, 'λ');
    score(sink) + score(source)
}

fn maybe_moved_reinitialization(flag: bool) -> int {
    let mut value = Packet::Count(41);
    if flag {
        let consumed = score(value);
    }
    value = Packet::Count(5);
    score(value)
}

fn all_arms_moved_reinitialization(flag: bool) -> int {
    let mut value = Packet::Count(1);
    if flag {
        let left = score(value);
    } else {
        let right = score(value);
    }
    value = Packet::Pair(6, 1 > 2);
    score(value)
}

fn condition_reinitialization() -> int {
    let mut value = Packet::Mark('λ');
    if match value {
        Packet::Empty => 1 > 2,
        Packet::Count(number) => number > 0,
        Packet::Pair(number, flag) => flag,
        Packet::Mark(glyph) => glyph == 'λ',
        Packet::CellValue(item) => item.value > 0,
        Packet::Cells(items) => items[0].value > 0,
        Packet::Matrix(matrix) => matrix[0][0] > 0,
        Packet::Mixed(number, flag, glyph) => flag
    } {
        let selected = 1;
    }
    value = Packet::Count(7);
    score(value)
}

fn returning_arm_reinitialization(flag: bool) -> int {
    let mut value = Packet::Count(9);
    if flag {
        return score(value);
    } else {
        let moved = value;
        let consumed = score(moved);
    }
    value = Packet::Count(8);
    score(value)
}

fn nested_reinitialization() -> int {
    let mut value = Packet::Count(1);
    {
        let moved = value;
        let consumed = score(moved);
        value = Packet::Count(9);
    }
    score(value)
}

fn repeated_reinitialization() -> int {
    let mut value = Packet::Count(1);
    let first = score(value);
    value = Packet::Count(10);
    let second = score(value);
    value = Packet::Count(11);
    score(value)
}

fn pre_loop_reinitialization() -> int {
    let mut value = Packet::Count(1);
    let consumed = score(value);
    value = Packet::Count(59);
    let mut step = 0;
    while step < 1 { step = step + 1; }
    score(value)
}

fn main() -> int {
    let total = alias_reinitialization()
        + call_reinitialization()
        + match_reinitialization()
        + assignment_reinitialization()
        + maybe_moved_reinitialization(1 < 2)
        + maybe_moved_reinitialization(1 > 2)
        + all_arms_moved_reinitialization(1 < 2)
        + condition_reinitialization()
        + returning_arm_reinitialization(1 > 2)
        + nested_reinitialization()
        + repeated_reinitialization()
        + pre_loop_reinitialization();
    if total == 199 { return 199; }
    1
}
"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded enum reinitialization topology compiled:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => None,
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
        )),
    }
}

#[test]
fn acyclic_owned_enum_reinitialization_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!(
            "owned-enum reinitialization syntax was not retained: {error}"
        ));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!(
            "complete acyclic owned-enum reinitialization failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedMutableOwnedPlaceAlloca",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumVariant",
                "CheckedEnumDispatch",
                "if_then_",
                "if_else_",
                "if_end_",
                "while_start_",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked reinitialization IR missing {marker:?}:\n{debug}"
                    ));
                }
            }
            if debug.matches("CheckedOwnedPlaceAssignment").count() < 12 {
                failures.push(format!(
                    "checked IR omitted whole-place reinitialization writes:\n{debug}"
                ));
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "reinitialization LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "reinitialization LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic reinitialization compilation failed: {error}"
                )),
            }
        }
    }

    for (label, source, expected) in [
        (
            "immutable moved target",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() { let value = E::A; let used = take(value); value = E::B; }",
            vec!["mutable local owned binding"],
        ),
        (
            "originally uninitialized target",
            "enum E { A } fn main() { let mut value: E; value = E::A; }",
            vec!["must already be initialized", "uninitialized"],
        ),
        (
            "wrong enum identity",
            "enum E { A } enum F { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; let used = take(value); value = F::A; }",
            vec!["type mismatch"],
        ),
        (
            "moved target self use",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; let used = take(value); value = value; }",
            vec!["moved value `value`", "use of moved value"],
        ),
        (
            "maybe-moved RHS source",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let source = E::A; if 1 < 2 { let used = take(source); } let mut target = E::A; target = source; }",
            vec!["may have been moved"],
        ),
        (
            "read before reinitialization",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; let first = take(value); let second = take(value); value = E::A; }",
            vec!["moved value `value`", "Use of moved value"],
        ),
        (
            "borrowed enum remains excluded",
            "enum E { A } fn main() { let mut value = E::A; let alias = &value; value = E::A; }",
            vec!["not admitted Copy-data", "reference"],
        ),
        (
            "while backedge without reinitialization",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; let mut step = 0; while step < 1 { let used = take(value); step = step + 1; } }",
            vec!["not restored", "backedge"],
        ),
        (
            "for backedge without reinitialization",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; for item in [1] { let used = take(value); } }",
            vec!["not restored", "backedge"],
        ),
        (
            "loop continue without reinitialization",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; loop { let used = take(value); continue; } }",
            vec!["not restored", "continue backedge"],
        ),
        (
            "enum array storage remains excluded",
            "enum E { A } fn main() { let values = [E::A]; }",
            vec!["not admitted", "array"],
        ),
        (
            "enum struct storage remains excluded",
            "enum E { A } struct Boxed { value: E } fn main() { let value = Boxed { value: E::A }; }",
            vec![
                "not an admitted",
                "unsupported",
                "Struct construction expressions",
            ],
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

    let workspace = TestWorkspace::new("cli-hygiene");
    let invalid = workspace.path("invalid.aero");
    let artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut value = E::A; loop { let used = take(value); break; } }",
    )
    .expect("write invalid unbalanced-loop source");
    let check = run_cli(&workspace, &[Path::new("check"), &invalid]);
    if check.status.success() {
        failures.push(format!(
            "unbalanced-loop CLI check succeeded: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &artifact],
    );
    if build.status.success() || artifact.exists() {
        failures.push(format!(
            "unbalanced-loop CLI build did not fail without an artifact: {}",
            output_text(&build)
        ));
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    match (
        fs::read_to_string(&tracked_root),
        fs::read_to_string(&tracked_module),
    ) {
        (Ok(actual_root), Ok(actual_module)) => {
            if actual_root != tracked_root_source() {
                failures.push(format!(
                    "tracked reinitialization root drifted at {}",
                    tracked_root.display()
                ));
            }
            if actual_module != tracked_module_source() {
                failures.push(format!(
                    "tracked reinitialization module drifted at {}",
                    tracked_module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked reinitialization example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("define i32 @reinitialize(i1 %aero.arg.flag)")
                && llvm.contains("store { i32, double, { double, i1 }, i32 }") => {}
        Ok(llvm) => failures.push(format!(
            "direct-module reinitialization LLVM omitted exact function/store evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!(
            "direct-module reinitialization compilation failed: {error}"
        )),
    }

    let tracked_workspace = TestWorkspace::new("tracked-example");
    let tracked_output = tracked_workspace.path("owned_enum_reinitialization.ll");
    let check = run_cli(&tracked_workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "tracked reinitialization CLI check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &tracked_workspace,
        &[
            Path::new("build"),
            &tracked_root,
            Path::new("-o"),
            &tracked_output,
        ],
    );
    if !build.status.success() || !tracked_output.is_file() {
        failures.push(format!(
            "tracked reinitialization CLI build failed (artifact={}): {}",
            tracked_output.is_file(),
            output_text(&build)
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for reinitialization integration anchors");
    for anchor in [
        "Test acyclic owned enum reinitialization integration example",
        "cargo run -- check ../../examples/owned_enum_reinitialization/main.aero",
        "cargo run -- run ../../examples/owned_enum_reinitialization/main.aero",
        "opt-22 -passes=verify -disable-output ../../owned_enum_reinitialization.ll",
        "llc-22 -verify-machineinstrs ../../owned_enum_reinitialization.ll",
        "clang-22 -no-pie ../../owned_enum_reinitialization.o -o ../../owned_enum_reinitialization",
        "Expected exit code 199",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "stable/nightly workflow must contain exactly one {anchor:?} anchor"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-073 acyclic owned-enum reinitialization failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
