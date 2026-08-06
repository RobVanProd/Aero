use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const CURRENT_RESULT_DIAGNOSTIC: &str =
    "enum match arms must return one identical admitted CopyData or owned enum value";
const EXPECTED_EXIT: i32 = 223;
const EXAMPLE_ROOT: &str = "examples/copydata_match_results/main.aero";
const EXAMPLE_MODULE: &str = "examples/copydata_match_results/values.aero";
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
            "aero-copydata-match-result-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CopyData Match result workspace");
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
            .is_some_and(|name| name.starts_with("aero-copydata-match-result-"));
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

fn analyzed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    SemanticAnalyzer::new()
        .analyze(ast)
        .map(|(_, analyzed)| analyzed)
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

fn expect_success(label: &str, source: &str, failures: &mut Vec<String>) {
    if let Err(error) = compile_program(source, CompilerOptions::default()) {
        failures.push(format!("{label}: unexpectedly failed: {error}"));
    }
}

fn expect_rejection(label: &str, source: &str, required: &[&str], failures: &mut Vec<String>) {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => failures.push(format!("{label}: unexpectedly compiled:\n{llvm}")),
        Err(error) => {
            for marker in required {
                if !error.contains(marker) {
                    failures.push(format!(
                        "{label}: diagnostic {error:?} omitted required marker {marker:?}"
                    ));
                }
            }
        }
    }
}

fn common_prelude(body: &str) -> String {
    format!(
        r#"
enum Pick {{ Left, Right }}
enum Owned {{ Empty, Number(int) }}
struct Cell {{ value: int, ready: bool }}
struct Frame {{
    cell: Cell,
    rows: [[int; 2]; 2],
    mixed: ((char, bool), [float; 2]),
    empty: [int; 0]
}}

fn make_cell(value: int) -> Cell {{
    Cell {{ value: value, ready: value > 0 }}
}}

fn make_empty() -> [int; 0] {{
    let empty: [int; 0] = [];
    empty
}}

fn make_frame(value: int) -> Frame {{
    Frame {{
        cell: make_cell(value),
        rows: [[value, value + 1], [value + 2, value + 3]],
        mixed: (('x', value > 0), [1.5, 2.5]),
        empty: make_empty()
    }}
}}

fn owned_score(value: Owned) -> int {{
    match value {{
        Owned::Empty => 0,
        Owned::Number(number) => number
    }}
}}

{body}
"#
    )
}

fn complete_source() -> String {
    common_prelude(
        r#"
enum Arrays { Values([int; 2]), Empty }

fn choose_array(value: Arrays) -> [int; 2] {
    match value {
        Arrays::Values(items) => items,
        Arrays::Empty => [3, 4]
    }
}

fn choose_empty(pick: Pick) -> [int; 0] {
    match pick {
        Pick::Left => make_empty(),
        Pick::Right => make_empty()
    }
}

fn choose_tuple(pick: Pick, value: ((char, bool), [float; 2]))
    -> ((char, bool), [float; 2]) {
    match pick {
        Pick::Left => value,
        Pick::Right => (('z', 1 < 2), [3.5, 4.5])
    }
}

fn choose_struct(pick: Pick, value: Frame) -> Frame {
    match pick {
        Pick::Left => value,
        Pick::Right => make_frame(7)
    }
}

fn nested(pick: Pick, value: Frame) -> Frame {
    match pick {
        Pick::Left => match Pick::Right {
            Pick::Left => value,
            Pick::Right => make_frame(11)
        },
        Pick::Right => make_frame(13)
    }
}

fn main() -> int {
    let array = choose_array(Arrays::Values([223, 2]));
    let empty = choose_empty(Pick::Right);
    let tuple = choose_tuple(Pick::Left, (('q', 1 < 2), [5.5, 6.5]));
    let structure = choose_struct(Pick::Left, make_frame(17));
    let recursive = nested(Pick::Left, make_frame(19));
    if empty.len() == 0
        && (tuple.0).1
        && structure.rows[1][1] == 20
        && recursive.cell.value == 11 {
        return array[0];
    }
    1
}
"#,
    )
}

#[test]
fn recursive_copydata_match_result_universe_is_one_checked_executable_class() {
    let mut failures = Vec::new();

    for (label, body) in [
        (
            "fixed array payload and constructor",
            r#"
enum Arrays { Values([int; 2]), Empty }
fn choose(value: Arrays) -> [int; 2] {
    match value { Arrays::Values(items) => items, Arrays::Empty => [3, 4] }
}
fn main() -> int { choose(Arrays::Values([223, 2]))[0] }
"#,
        ),
        (
            "zero-length recursive fixed array",
            r#"
fn choose(pick: Pick) -> [int; 0] {
    match pick { Pick::Left => make_empty(), Pick::Right => make_empty() }
}
fn main() -> int { if choose(Pick::Left).len() == 0 { return 223; } 1 }
"#,
        ),
        (
            "heterogeneous recursive tuple",
            r#"
fn choose(pick: Pick) -> ((char, bool), [float; 2]) {
    match pick {
        Pick::Left => (('x', 1 < 2), [1.5, 2.5]),
        Pick::Right => (('y', 1 > 2), [3.5, 4.5])
    }
}
fn main() -> int { if (choose(Pick::Left).0).1 { return 223; } 1 }
"#,
        ),
        (
            "finite named struct and nested Match",
            r#"
fn choose(pick: Pick, value: Frame) -> Frame {
    match pick {
        Pick::Left => match Pick::Right {
            Pick::Left => value,
            Pick::Right => make_frame(223)
        },
        Pick::Right => value
    }
}
fn main() -> int { choose(Pick::Left, make_frame(1)).cell.value }
"#,
        ),
    ] {
        expect_success(label, &common_prelude(body), &mut failures);
    }

    let complete = complete_source();
    expect_success(
        "complete recursive CopyData result product",
        &complete,
        &mut failures,
    );
    match checked_ir_and_llvm(&complete) {
        Err(error) => failures.push(format!("complete checked product failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            if !debug.contains("CheckedMatchResultPlaceAlloca") {
                failures.push(format!(
                    "checked IR omitted unified Match result place:\n{debug}"
                ));
            }
            if debug.contains("CheckedEnumMatchResultPlaceAlloca") {
                failures.push(format!(
                    "checked IR retained enum-only result place:\n{debug}"
                ));
            }
            for marker in [
                "[2 x double]",
                "[0 x double]",
                "%aero.struct.Frame",
                "{ { i32, i1 }, [2 x double] }",
            ] {
                if !llvm.contains(marker) {
                    failures.push(format!("LLVM omitted {marker:?}:\n{llvm}"));
                }
            }
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
            if root_source.contains("if result == 223")
                && module_source.contains("fn choose_array")
                && module_source.contains("fn choose_tuple")
                && module_source.contains("fn choose_frame")
                && module_source.contains("Pick::Right => make_empty()") => {}
        (Ok(root_source), Ok(module_source)) => failures.push(format!(
            "tracked CORE-076 example drifted:\nROOT\n{root_source}\nMODULE\n{module_source}"
        )),
        (root_result, module_result) => failures.push(format!(
            "tracked CORE-076 example is missing: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }
    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("@choose_array(")
                && llvm.contains("@choose_tuple(")
                && llvm.contains("@choose_frame(")
                && llvm.contains("ret i32 223") => {}
        Ok(llvm) => failures.push(format!(
            "tracked CopyData Match result LLVM omitted exact evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!(
            "tracked CopyData Match result example did not compile: {error}"
        )),
    }

    let tracked_workspace = TestWorkspace::new("tracked-example");
    let tracked_output = tracked_workspace.path("copydata_match_results.ll");
    let check = run_cli(&tracked_workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "tracked CopyData Match result CLI check failed: {}",
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
            "tracked CopyData Match result CLI build failed (artifact={}): {}",
            tracked_output.is_file(),
            output_text(&build)
        ));
    }

    let invalid_workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_output = invalid_workspace.path("invalid.ll");
    fs::write(
        &invalid,
        r#"enum Pick { Left, Right } fn main() { let value = match Pick::Left { Pick::Left => "a", Pick::Right => "b" }; }"#,
    )
    .expect("write excluded Match result source");
    let invalid_check = run_cli(&invalid_workspace, &[Path::new("check"), &invalid]);
    if invalid_check.status.success()
        || !output_text(&invalid_check).contains(CURRENT_RESULT_DIAGNOSTIC)
    {
        failures.push(format!(
            "excluded Match result CLI check did not fail closed: {}",
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
            "excluded Match result CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for CopyData Match result integration anchors");
    for anchor in [
        "Test unified CopyData Match result integration example",
        "cargo run -- check ../../examples/copydata_match_results/main.aero",
        "cargo run -- run ../../examples/copydata_match_results/main.aero",
        "opt-22 -passes=verify -disable-output ../../copydata_match_results.ll",
        "llc-22 -verify-machineinstrs ../../copydata_match_results.ll",
        "clang-22 -no-pie ../../copydata_match_results.o -o ../../copydata_match_results",
        "unified CopyData Match result example passed with exit code 223",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "stable/nightly workflow must contain exactly one {anchor:?} anchor"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-076 complete result-class failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}

#[test]
fn copydata_result_contexts_compose_and_noncopy_results_stay_closed() {
    let mut failures = Vec::new();

    let contexts = common_prelude(
        r#"
fn identity(value: Frame) -> Frame { value }
fn read(value: &Frame) -> Frame { *value }
fn score(value: Frame) -> int { value.cell.value }
fn choose(pick: Pick, value: Frame) -> Frame {
    match pick { Pick::Left => value, Pick::Right => make_frame(7) }
}
fn direct_return(pick: Pick) -> Frame {
    match pick { Pick::Left => make_frame(11), Pick::Right => make_frame(13) }
}
fn main() -> int {
    let inferred = choose(Pick::Left, make_frame(223));
    let exact: Frame = match Pick::Right {
        Pick::Left => inferred,
        Pick::Right => make_frame(17)
    };
    let mut replaced = make_frame(19);
    replaced = match Pick::Left {
        Pick::Left => exact,
        Pick::Right => direct_return(Pick::Right)
    };
    let projected = (match Pick::Left {
        Pick::Left => replaced.rows,
        Pick::Right => [[1, 2], [3, 4]]
    })[0][0];
    let tuple_item = (match Pick::Right {
        Pick::Left => (1, 1 < 2),
        Pick::Right => (2, 1 < 2)
    }).0;
    let called = score(match Pick::Left {
        Pick::Left => identity(replaced),
        Pick::Right => read(&replaced)
    });
    if projected == 17 && tuple_item == 2 && called == 17 { return inferred.cell.value; }
    1
}
"#,
    );
    expect_success(
        "binding/projection/call/return/reassignment/reference contexts",
        &contexts,
        &mut failures,
    );

    expect_success(
        "accepted scalar and owned-enum results remain executable",
        &common_prelude(
            r#"
fn owned(pick: Pick) -> Owned {
    match pick { Pick::Left => Owned::Number(223), Pick::Right => Owned::Empty }
}
fn scalar(pick: Pick) -> char {
    match pick { Pick::Left => 'x', Pick::Right => 'y' }
}
fn main() -> int { if scalar(Pick::Left) == 'x' { return owned_score(owned(Pick::Left)); } 1 }
"#,
        ),
        &mut failures,
    );

    for (label, body, required) in [
        (
            "compile-time String result",
            r#"fn main() { let value = match Pick::Left { Pick::Left => "a", Pick::Right => "b" }; }"#,
            vec![CURRENT_RESULT_DIAGNOSTIC],
        ),
        (
            "reference result",
            "fn main() { let left = 1; let right = 2; let value = match Pick::Left { Pick::Left => &left, Pick::Right => &right }; }",
            vec![CURRENT_RESULT_DIAGNOSTIC],
        ),
        (
            "unit tuple result",
            "fn main() { let value = match Pick::Left { Pick::Left => (), Pick::Right => () }; }",
            vec!["tuple"],
        ),
        (
            "different arm result types",
            "fn main() { let value = match Pick::Left { Pick::Left => [1, 2], Pick::Right => [1, 2, 3] }; }",
            vec!["mismatch"],
        ),
    ] {
        expect_rejection(label, &common_prelude(body), &required, &mut failures);
    }

    assert!(
        failures.is_empty(),
        "CORE-076 context/containment failures:\n{}",
        failures.join("\n---\n")
    );
}
