use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/loop_local_enum_ownership/main.aero";
const EXAMPLE_MODULE: &str = "examples/loop_local_enum_ownership/loops.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 149;

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
            "aero-loop-local-enum-ownership-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create loop-local enum ownership workspace");
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
            .is_some_and(|name| name.starts_with("aero-loop-local-enum-ownership-"));
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
struct Leaf { value: int, active: bool }
struct Branch { leaf: Leaf, trail: [Leaf; 2] }
struct Deep { branch: Branch, pair: (int, bool), branches: [Branch; 2] }

enum Packet {
    Empty,
    Count(int),
    Flag(bool),
    Pair((int, bool)),
    BranchValue(Branch),
    Branches([Branch; 2]),
    DeepValue(Deep),
    Matrix([[int; 2]; 2])
}

fn leaf(value: int) -> Leaf {
    Leaf { value: value, active: value > 0 }
}

fn branch(value: int) -> Branch {
    Branch { leaf: leaf(value), trail: [leaf(value + 1), leaf(value + 2)] }
}

fn deep(value: int) -> Deep {
    Deep {
        branch: branch(value),
        pair: (value + 3, value > 0),
        branches: [branch(value + 4), branch(value + 5)]
    }
}

fn bool_score(value: bool, score: int) -> int {
    if value { return score; }
    0
}

fn score(packet: Packet) -> int {
    match packet {
        Packet::Empty => 0,
        Packet::Count(count) => count,
        Packet::Flag(flag) => bool_score(flag, 1),
        Packet::Pair(pair) => pair.0,
        Packet::BranchValue(value) => value.leaf.value,
        Packet::Branches(values) => values[1].leaf.value,
        Packet::DeepValue(value) => value.branch.leaf.value,
        Packet::Matrix(value) => value[1][0]
    }
}

fn produce(value: int) -> Packet {
    if value == 0 { return Packet::Empty; }
    Packet::Count(value)
}

fn while_fallthrough(limit: int) -> int {
    let mut step = 0;
    let mut total = 0;
    while step < limit {
        let exact: Packet = produce(step + 1);
        let inferred = exact;
        total = total + score(inferred);
        step = step + 1;
    }
    total
}

fn while_condition() -> int {
    while match produce(0) {
        Packet::Empty => 1 > 2,
        Packet::Count(value) => value < 0,
        Packet::Flag(value) => value,
        Packet::Pair(value) => value.1,
        Packet::BranchValue(value) => value.leaf.value < 0,
        Packet::Branches(value) => value[0].leaf.value < 0,
        Packet::DeepValue(value) => value.branch.leaf.value < 0,
        Packet::Matrix(value) => value[0][0] < 0
    } {
        return 1;
    }
    7
}

fn for_continue() -> int {
    let mut total = 0;
    for item in [1, 2, 3] {
        let fresh = Packet::Pair((item, item > 0));
        total = total + score(fresh);
        continue;
    }
    total
}

fn loop_break() -> int {
    let mut total = 0;
    loop {
        let mut fresh: Packet = Packet::BranchValue(branch(3));
        fresh = produce(4);
        total = total + score(fresh);
        break;
    }
    total
}

fn nested_loops() -> int {
    let mut outer = 0;
    let mut total = 0;
    while outer < 1 {
        loop {
            let fresh = Packet::DeepValue(deep(5));
            total = total + score(fresh);
            break;
        }
        outer = outer + 1;
    }
    total
}

fn return_fresh(flag: bool) -> Packet {
    while flag {
        let fresh = Packet::Matrix([[1, 2], [8, 9]]);
        return fresh;
    }
    Packet::Empty
}

fn schema_product() -> int {
    let mut total = 0;
    let mut step = 0;
    while step < 1 {
        total = total + score(Packet::Empty);
        total = total + score(Packet::Count(1));
        total = total + score(Packet::Flag(1 < 2));
        total = total + score(Packet::Pair((2, 1 < 2)));
        total = total + score(Packet::BranchValue(branch(3)));
        total = total + score(Packet::Branches([branch(3), branch(4)]));
        total = total + score(Packet::DeepValue(deep(5)));
        total = total + score(Packet::Matrix([[5, 6], [7, 8]]));
        step = step + 1;
    }
    total
}

fn main() -> int {
    let total = while_fallthrough(3)
        + while_condition()
        + for_continue()
        + loop_break()
        + nested_loops()
        + score(return_fresh(1 < 2))
        + schema_product();
    if total == 59 { return 149; }
    1
}
"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded loop ownership topology compiled:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => None,
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
        )),
    }
}

#[test]
fn fresh_loop_local_enum_ownership_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!(
            "fresh loop-local enum syntax was not retained: {error}"
        ));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("complete loop-local enum flow failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "while_start_",
                "while_body_",
                "for_start_",
                "for_body_",
                "for_continue_",
                "loop_start_",
                "CheckedEnumVariant",
                "CheckedMutableOwnedPlaceAlloca",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumDispatch",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked loop-local enum IR missing {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "loop-local enum LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            if !llvm.contains("br i1")
                || !llvm.contains("switch i32")
                || !llvm.contains("load ")
                || !llvm.contains("store ")
            {
                failures.push(format!(
                    "loop-local enum LLVM lost CFG/load/store/dispatch evidence:\n{llvm}"
                ));
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "loop-local enum LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic loop-local enum compilation failed: {error}"
                )),
            }
        }
    }

    for (label, source, expected) in [
        (
            "double consumption in one while iteration",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() -> int { let mut step = 0; while step < 1 { let fresh = E::A; take(fresh); take(fresh); step = step + 1; } 0 }",
            vec!["moved value `fresh`", "Use of moved value"],
        ),
        (
            "outer owner consumed on while backedge",
            "enum E { A } fn main() { let outer = E::A; let mut step = 0; while step < 1 { let moved = outer; step = step + 1; } }",
            vec!["loop", "fixed-point", "backedge"],
        ),
        (
            "outer owner consumed before for continue",
            "enum E { A } fn main() { let outer = E::A; for item in [1] { let moved = outer; continue; } }",
            vec!["loop", "fixed-point", "backedge"],
        ),
        (
            "outer owner consumed before loop break remains quarantined",
            "enum E { A } fn main() { let outer = E::A; loop { let moved = outer; break; } }",
            vec!["moved", "loop", "backedge", "not admitted"],
        ),
        (
            "fresh local self replacement",
            "enum E { A } fn main() { loop { let mut fresh = E::A; fresh = fresh; break; } }",
            vec!["self-replacement", "itself", "same owner", "cannot replace"],
        ),
        (
            "textually unreachable second consumption remains fail closed",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() -> int { loop { let fresh = E::A; take(fresh); continue; take(fresh); } }",
            vec!["moved value `fresh`", "Use of moved value"],
        ),
        (
            "enum borrowing remains excluded",
            "enum E { A } fn main() { while 1 < 2 { let fresh = E::A; let alias = &fresh; break; } }",
            vec!["not admitted Copy-data", "reference"],
        ),
        (
            "enum array storage remains excluded",
            "enum E { A } fn main() { while 1 < 2 { let values = [E::A]; break; } }",
            vec!["not admitted", "array"],
        ),
        (
            "break outside loop",
            "fn main() { break; }",
            vec![
                "Break statement outside of loop",
                "only admitted inside loops",
            ],
        ),
        (
            "continue outside loop",
            "fn main() { continue; }",
            vec![
                "Continue statement outside of loop",
                "only admitted inside loops",
            ],
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, &expected) {
            failures.push(failure);
        }
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { A } fn main() { let outer = E::A; for item in [1] { let moved = outer; continue; } }",
    )
    .expect("write invalid loop-local enum ownership source");
    let invalid_check = run_cli(&workspace, &[Path::new("check"), &invalid]);
    if invalid_check.status.success() {
        failures.push(format!(
            "invalid loop ownership CLI check succeeded: {}",
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
            "invalid loop ownership CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for path in [&tracked_root, &tracked_module] {
        if !path.is_file() {
            failures.push(format!(
                "tracked loop-local enum example missing: {}",
                path.display()
            ));
        }
    }
    if tracked_root.is_file() && tracked_module.is_file() {
        let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
        if !check.status.success() {
            failures.push(format!(
                "tracked loop-local enum direct-module check failed: {}",
                output_text(&check)
            ));
        }
        let output = workspace.path("loop-local-enum-ownership.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
        );
        if !build.status.success() || !output.is_file() {
            failures.push(format!(
                "tracked loop-local enum direct-module build failed (artifact={}): {}",
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
                "Test loop-local enum ownership integration example",
                "examples/loop_local_enum_ownership/main.aero",
                "opt-22 -passes=verify -disable-output ../../loop_local_enum_ownership.ll",
                "llc-22 -verify-machineinstrs ../../loop_local_enum_ownership.ll",
                "clang-22 -no-pie ../../loop_local_enum_ownership.o -o ../../loop_local_enum_ownership",
                "Expected exit code 149",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-066 loop-local enum ownership failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
