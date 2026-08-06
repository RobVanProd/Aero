use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 227;
const EXAMPLE_ROOT: &str = "examples/balanced_loop_enum_ownership/main.aero";
const EXAMPLE_MODULE: &str = "examples/balanced_loop_enum_ownership/loops.aero";
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
            "aero-balanced-loop-enum-ownership-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create balanced-loop ownership workspace");
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
            .is_some_and(|name| name.starts_with("aero-balanced-loop-enum-ownership-"));
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

fn checked_admission_without_semantics(source: &str) -> Result<compiler::CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(parsed(source)?)
        .map_err(|error| error.to_string())
}

fn complete_source() -> &'static str {
    r#"
struct Cell { value: int, flags: [bool; 2] }

enum Switch { Left, Right }

enum Packet {
    Empty,
    Count(int),
    Pair(int, bool),
    CellValue(Cell),
    Cells([Cell; 2]),
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
        Packet::CellValue(item) => item.value,
        Packet::Cells(items) => items[1].value,
        Packet::Mixed(number, flag, glyph) =>
            number + bool_score(flag, 2) + char_score(glyph, 3)
    }
}

fn produce(value: int) -> Packet { Packet::Count(value) }

fn choose(input: Switch, value: int) -> Packet {
    match input {
        Switch::Left => Packet::Pair(value, value > 0),
        Switch::Right => Packet::CellValue(cell(value))
    }
}

fn while_fallthrough() -> int {
    let mut owner = Packet::Empty;
    let mut step = 0;
    let mut total = 0;
    while step < 3 {
        total = total + score(owner);
        owner = produce(step + 1);
        step = step + 1;
    }
    total + score(owner)
}

fn while_continue() -> int {
    let mut owner: Packet = Packet::Cells([cell(1), cell(2)]);
    let mut step = 0;
    let mut total = 0;
    while step < 2 {
        total = total + score(owner);
        owner = choose(Switch::Left, step + 4);
        step = step + 1;
        continue;
    }
    total + score(owner)
}

fn while_break(flag: bool) -> int {
    let mut owner = Packet::Pair(5, flag);
    let mut total = 0;
    while 1 < 2 {
        total = total + score(owner);
        owner = Packet::Mixed(6, flag, 'λ');
        break;
    }
    total + score(owner)
}

fn for_fallthrough() -> int {
    let mut owner = Packet::CellValue(cell(7));
    let mut total = 0;
    for item in [1, 2] {
        total = total + score(owner);
        owner = produce(item + 7);
    }
    total + score(owner)
}

fn for_continue_and_break() -> int {
    let mut owner = Packet::Count(9);
    let mut total = 0;
    for item in [1, 2, 3] {
        total = total + score(owner);
        owner = choose(Switch::Right, item + 9);
        if item < 2 { continue; }
        break;
    }
    total + score(owner)
}

fn loop_break() -> int {
    let mut owner = Packet::Empty;
    let mut total = 0;
    loop {
        total = total + score(owner);
        owner = Packet::Cells([cell(11), cell(12)]);
        break;
    }
    total + score(owner)
}

fn conditional_paths(flag: bool) -> int {
    let mut owner = Packet::Count(13);
    let mut step = 0;
    let mut total = 0;
    while step < 2 {
        if flag {
            total = total + score(owner);
            owner = Packet::Pair(step + 13, flag);
        } else {
            let moved = owner;
            total = total + score(moved);
            owner = choose(Switch::Right, step + 15);
        }
        step = step + 1;
    }
    total + score(owner)
}

fn returning_path(flag: bool) -> int {
    let mut owner = Packet::Count(17);
    while 1 < 2 {
        if flag { return score(owner); }
        let moved = owner;
        let before = score(moved);
        owner = produce(before + 1);
        break;
    }
    score(owner)
}

fn repeated_cycles() -> int {
    let mut owner = Packet::Count(18);
    let mut step = 0;
    let mut total = 0;
    while step < 1 {
        total = total + score(owner);
        owner = Packet::Count(19);
        total = total + score(owner);
        owner = Packet::Count(20);
        step = step + 1;
    }
    total + score(owner)
}

fn nested_loops() -> int {
    let mut outer = Packet::Count(21);
    let mut outer_step = 0;
    let mut total = 0;
    while outer_step < 1 {
        total = total + score(outer);
        outer = Packet::Count(22);
        let mut inner = Packet::Count(23);
        loop {
            total = total + score(inner);
            inner = Packet::Count(24);
            break;
        }
        total = total + score(inner);
        outer_step = outer_step + 1;
    }
    total + score(outer)
}

fn main() -> int {
    while_fallthrough()
        + while_continue()
        + while_break(1 < 2)
        + for_fallthrough()
        + for_continue_and_break()
        + loop_break()
        + conditional_paths(1 < 2)
        + conditional_paths(1 > 2)
        + returning_path(1 > 2)
        + repeated_cycles()
        + nested_loops()
}
"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match checked_ir_and_llvm(source) {
        Ok((_, llvm)) => Some(format!(
            "{label}: excluded balanced-loop ownership topology compiled:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => None,
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
        )),
    }
}

#[test]
fn balanced_loop_owned_enum_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!(
            "balanced loop-owned enum syntax was not retained: {error}"
        ));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!(
            "complete balanced loop-owned enum class failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "while_start_",
                "for_start_",
                "for_continue_",
                "loop_start_",
                "CheckedMutableOwnedPlaceAlloca",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumDispatch",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked balanced-loop IR missing {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "balanced-loop LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "balanced-loop LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => {
                    failures.push(format!("second balanced-loop compilation failed: {error}"))
                }
            }
        }
    }

    if let Err(error) = checked_admission_without_semantics(source) {
        failures.push(format!(
            "independent checked admission rejected the balanced class: {error}"
        ));
    }

    for (label, source, expected) in [
        (
            "missing fallthrough restoration",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; let mut step = 0; while step < 1 { let used = take(owner); step = step + 1; } }",
            vec!["may have been moved", "moved value"],
        ),
        (
            "missing continue restoration",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; while 1 < 2 { let used = take(owner); continue; } }",
            vec!["may have been moved", "moved value"],
        ),
        (
            "restoration after continue is unreachable",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; while 1 < 2 { let used = take(owner); continue; owner = E::A; } }",
            vec!["may have been moved", "moved value"],
        ),
        (
            "only one conditional path restores",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() { let mut owner = E::A; let mut step = 0; while step < 1 { if step < 1 { let used = take(owner); owner = E::B; } else { let used = take(owner); } step = step + 1; } }",
            vec!["may have been moved", "moved value"],
        ),
        (
            "same-path double consumption",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; while 1 < 2 { let first = take(owner); let second = take(owner); owner = E::A; break; } }",
            vec!["moved value `owner`", "Use of moved value"],
        ),
        (
            "direct self replacement",
            "enum E { A } fn main() { let mut owner = E::A; while 1 < 2 { owner = owner; break; } }",
            vec!["self-replacement", "same owner", "cannot replace"],
        ),
        (
            "enum aggregate storage remains excluded",
            "enum E { A } fn main() { let mut owner = E::A; while 1 < 2 { let stored = [owner]; break; } }",
            vec!["not admitted", "array"],
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, &expected) {
            failures.push(failure);
        }
    }

    for (label, source) in [(
        "direct admission missing continue restoration",
        "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; while 1 < 2 { let used = take(owner); continue; } }",
    )] {
        if checked_admission_without_semantics(source).is_ok() {
            failures.push(format!(
                "{label}: independent checked admission accepted an unbalanced edge"
            ));
        }
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    match (
        fs::read_to_string(&tracked_root),
        fs::read_to_string(&tracked_module),
    ) {
        (Ok(root_source), Ok(module_source))
            if root_source.contains("if result == 227")
                && module_source.contains("fn balanced_while")
                && module_source.contains("fn balanced_for")
                && module_source.contains("fn balanced_loop")
                && module_source.contains("fn balanced_transfers")
                && module_source.contains("if total == 383 { return 227; }") => {}
        (Ok(root_source), Ok(module_source)) => failures.push(format!(
            "tracked CORE-077 example drifted:\nROOT\n{root_source}\nMODULE\n{module_source}"
        )),
        (root_result, module_result) => failures.push(format!(
            "tracked CORE-077 example is missing: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }
    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("@balanced_while(")
                && llvm.contains("@balanced_for(")
                && llvm.contains("@balanced_loop(")
                && llvm.contains("@balanced_transfers(")
                && llvm.contains("ret i32 227") => {}
        Ok(llvm) => failures.push(format!(
            "tracked balanced-loop LLVM omitted exact evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!(
            "tracked balanced-loop example did not compile: {error}"
        )),
    }

    let tracked_workspace = TestWorkspace::new("tracked-example");
    let tracked_output = tracked_workspace.path("balanced_loop_enum_ownership.ll");
    let check = run_cli(&tracked_workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "tracked balanced-loop CLI check failed: {}",
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
            "tracked balanced-loop CLI build failed (artifact={}): {}",
            tracked_output.is_file(),
            output_text(&build)
        ));
    }

    let invalid_workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_output = invalid_workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; while 1 < 2 { let used = take(owner); continue; owner = E::A; } }",
    )
    .expect("write invalid balanced-loop ownership source");
    let invalid_check = run_cli(&invalid_workspace, &[Path::new("check"), &invalid]);
    if invalid_check.status.success()
        || !output_text(&invalid_check).contains("may have been moved")
    {
        failures.push(format!(
            "unbalanced continue CLI check did not fail closed: {}",
            output_text(&invalid_check)
        ));
    }
    let invalid_build = run_cli(
        &invalid_workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_output,
        ],
    );
    if invalid_build.status.success() || invalid_output.exists() {
        failures.push(format!(
            "unbalanced continue CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for balanced-loop ownership anchors");
    for anchor in [
        "Test balanced loop enum ownership integration example",
        "cargo run -- check ../../examples/balanced_loop_enum_ownership/main.aero",
        "cargo run -- run ../../examples/balanced_loop_enum_ownership/main.aero",
        "opt-22 -passes=verify -disable-output ../../balanced_loop_enum_ownership.ll",
        "llc-22 -verify-machineinstrs ../../balanced_loop_enum_ownership.ll",
        "clang-22 -no-pie ../../balanced_loop_enum_ownership.o -o ../../balanced_loop_enum_ownership",
        "balanced loop enum ownership example passed with exit code 227",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "stable/nightly workflow must contain exactly one {anchor:?} anchor"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-077 balanced loop-owned enum failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
