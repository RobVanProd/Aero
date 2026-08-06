use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 203;
const EXAMPLE_ROOT: &str = "examples/owned_enum_match_results/main.aero";
const EXAMPLE_MODULE: &str = "examples/owned_enum_match_results/values.aero";
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
            "aero-owned-enum-match-result-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create owned-enum Match result workspace");
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
            .is_some_and(|name| name.starts_with("aero-owned-enum-match-result-"));
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

fn tracked_root_source() -> &'static str {
    "mod values;\n\nfn main() -> int {\n    let result = exercise();\n    if result == 203 { return 203; }\n    1\n}\n"
}

fn tracked_module_source() -> &'static str {
    r#"enum Input {
    Empty,
    Count(int)
}

enum Output {
    Empty,
    Count(int)
}

fn make_empty() -> Output { Output::Empty }

fn translate(input: Input) -> Output {
    match input {
        Input::Empty => make_empty(),
        Input::Count(number) => match Input::Count(number) {
            Input::Empty => Output::Empty,
            Input::Count(inner) => Output::Count(inner)
        }
    }
}

fn score(value: Output) -> int {
    match value {
        Output::Empty => 0,
        Output::Count(number) => number
    }
}

fn exercise() -> int {
    let mut result = translate(Input::Count(1));
    let consumed = score(result);
    result = translate(Input::Count(203));
    score(result)
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

fn complete_source() -> &'static str {
    r#"
struct Cell { value: int, flags: [bool; 2] }

enum Input {
    Empty,
    Count(int),
    Pair(int, bool),
    Mark(char),
    CellValue(Cell),
    Cells([Cell; 2]),
    Matrix([[int; 2]; 2])
}

enum Output {
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

fn make_count() -> Output { Output::Count(17) }

fn make_pair(value: int, flag: bool) -> Output { Output::Pair(value, flag) }

fn bool_score(value: bool, score: int) -> int {
    if value { return score; }
    0
}

fn char_score(value: char, score: int) -> int {
    if value == 'x' { return score; }
    0
}

fn score(value: Output) -> int {
    match value {
        Output::Empty => 0,
        Output::Count(number) => number,
        Output::Pair(number, flag) => number + bool_score(flag, 2),
        Output::Mark(glyph) => char_score(glyph, 3),
        Output::CellValue(item) => item.value,
        Output::Cells(items) => items[1].value,
        Output::Matrix(matrix) => matrix[1][0],
        Output::Mixed(number, flag, glyph) =>
            number + bool_score(flag, 2) + char_score(glyph, 3)
    }
}

fn constructor_result(input: Input) -> Output {
    match input {
        Input::Empty => Output::Empty,
        Input::Count(number) => Output::Count(number),
        Input::Pair(number, flag) => Output::Pair(number, flag),
        Input::Mark(glyph) => Output::Mark(glyph),
        Input::CellValue(item) => Output::CellValue(item),
        Input::Cells(items) => Output::Cells(items),
        Input::Matrix(matrix) => Output::Matrix(matrix)
    }
}

fn call_result(input: Input) -> Output {
    match input {
        Input::Empty => make_count(),
        Input::Count(number) => make_pair(number, number > 0),
        Input::Pair(number, flag) => make_pair(number, flag),
        Input::Mark(glyph) => make_count(),
        Input::CellValue(item) => make_pair(item.value, 1 < 2),
        Input::Cells(items) => make_pair(items[0].value, 1 < 2),
        Input::Matrix(matrix) => make_pair(matrix[0][1], 1 > 2)
    }
}

fn nested_result(input: Input) -> Output {
    match input {
        Input::Empty => match Input::Count(1) {
            Input::Empty => Output::Empty,
            Input::Count(number) => Output::Count(number),
            Input::Pair(number, flag) => Output::Pair(number, flag),
            Input::Mark(glyph) => Output::Mark(glyph),
            Input::CellValue(item) => Output::CellValue(item),
            Input::Cells(items) => Output::Cells(items),
            Input::Matrix(matrix) => Output::Matrix(matrix)
        },
        Input::Count(number) => Output::Count(number),
        Input::Pair(number, flag) => Output::Pair(number, flag),
        Input::Mark(glyph) => Output::Mark(glyph),
        Input::CellValue(item) => Output::CellValue(item),
        Input::Cells(items) => Output::Cells(items),
        Input::Matrix(matrix) => Output::Matrix(matrix)
    }
}

fn direct_call() -> int {
    score(match Input::Pair(20, 1 < 2) {
        Input::Empty => Output::Empty,
        Input::Count(number) => Output::Count(number),
        Input::Pair(number, flag) => Output::Pair(number, flag),
        Input::Mark(glyph) => Output::Mark(glyph),
        Input::CellValue(item) => Output::CellValue(item),
        Input::Cells(items) => Output::Cells(items),
        Input::Matrix(matrix) => Output::Matrix(matrix)
    })
}

fn binding_and_rematch() -> int {
    let result: Output = match Input::Mark('x') {
        Input::Empty => Output::Empty,
        Input::Count(number) => Output::Count(number),
        Input::Pair(number, flag) => Output::Pair(number, flag),
        Input::Mark(glyph) => Output::Mark(glyph),
        Input::CellValue(item) => Output::CellValue(item),
        Input::Cells(items) => Output::Cells(items),
        Input::Matrix(matrix) => Output::Matrix(matrix)
    };
    score(result)
}

fn replacement() -> int {
    let mut result = Output::Empty;
    result = constructor_result(Input::Count(52));
    score(result)
}

fn moved_reinitialization() -> int {
    let mut result = Output::Count(1);
    let before = score(result);
    result = match Input::Count(24) {
        Input::Empty => Output::Empty,
        Input::Count(number) => Output::Count(number),
        Input::Pair(number, flag) => Output::Pair(number, flag),
        Input::Mark(glyph) => Output::Mark(glyph),
        Input::CellValue(item) => Output::CellValue(item),
        Input::Cells(items) => Output::Cells(items),
        Input::Matrix(matrix) => Output::Matrix(matrix)
    };
    before + score(result)
}

fn maybe_moved_reinitialization(flag: bool) -> int {
    let mut result = Output::Count(1);
    if flag {
        let consumed = score(result);
    }
    result = call_result(Input::Count(25));
    score(result)
}

fn returning_arm(flag: bool) -> Output {
    if flag {
        return match Input::Count(26) {
            Input::Empty => Output::Empty,
            Input::Count(number) => Output::Count(number),
            Input::Pair(number, inner) => Output::Pair(number, inner),
            Input::Mark(glyph) => Output::Mark(glyph),
            Input::CellValue(item) => Output::CellValue(item),
            Input::Cells(items) => Output::Cells(items),
            Input::Matrix(matrix) => Output::Matrix(matrix)
        };
    }
    Output::Empty
}

fn main() -> int {
    let inferred = constructor_result(Input::CellValue(cell(1)));
    let total = score(inferred)
        + score(call_result(Input::Count(17)))
        + score(nested_result(Input::Empty))
        + direct_call()
        + binding_and_rematch()
        + replacement()
        + moved_reinitialization()
        + maybe_moved_reinitialization(1 < 2)
        + maybe_moved_reinitialization(1 > 2)
        + score(returning_arm(1 < 2));
    if total == 203 { return 203; }
    1
}
"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded owned-enum Match result topology compiled:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => None,
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
        )),
    }
}

#[test]
fn fresh_owned_enum_match_result_class_is_complete_checked_and_executable() {
    let source = complete_source();
    let mut failures = Vec::new();

    if let Err(error) = parsed(source) {
        failures.push(format!(
            "owned-enum Match result syntax was not retained: {error}"
        ));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("complete owned-enum Match result failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedEnumMatchResultPlaceAlloca",
                "dispatch_schema",
                "CheckedOwnedPlaceAssignment",
                "CheckedEnumDispatch",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!("checked result IR missing {marker:?}:\n{debug}"));
                }
            }
            if debug.matches("CheckedEnumMatchResultPlaceAlloca").count() < 8 {
                failures.push(format!(
                    "checked IR omitted owned Match result places:\n{debug}"
                ));
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "owned-enum Match result LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "owned-enum Match result LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic owned-enum Match result compilation failed: {error}"
                )),
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
        (Ok(actual_root), Ok(actual_module)) => {
            if actual_root != tracked_root_source() {
                failures.push(format!(
                    "tracked owned-enum Match result root drifted at {}",
                    tracked_root.display()
                ));
            }
            if actual_module != tracked_module_source() {
                failures.push(format!(
                    "tracked owned-enum Match result module drifted at {}",
                    tracked_module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked owned-enum Match result pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("@translate(")
                && llvm.contains("@exercise(")
                && llvm.contains("ret i32 203") => {}
        Ok(llvm) => failures.push(format!(
            "direct-module owned-enum Match result LLVM omitted exact execution evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!(
            "direct-module owned-enum Match result compilation failed: {error}"
        )),
    }

    let tracked_workspace = TestWorkspace::new("tracked-example");
    let tracked_output = tracked_workspace.path("owned_enum_match_results.ll");
    let check = run_cli(&tracked_workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "tracked owned-enum Match result CLI check failed: {}",
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
            "tracked owned-enum Match result CLI build failed (artifact={}): {}",
            tracked_output.is_file(),
            output_text(&build)
        ));
    }

    let invalid_workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_output = invalid_workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum I { A } enum O { X } fn main() { let existing = O::X; let result = match I::A { I::A => existing }; }",
    )
    .expect("write excluded owned-enum Match result source");
    let check = run_cli(&invalid_workspace, &[Path::new("check"), &invalid]);
    if check.status.success() {
        failures.push(format!(
            "excluded owned-enum Match result CLI check succeeded: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &invalid_workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_output,
        ],
    );
    if build.status.success() || invalid_output.exists() {
        failures.push(format!(
            "excluded owned-enum Match result CLI build did not fail without an artifact: {}",
            output_text(&build)
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for owned-enum Match result integration anchors");
    for anchor in [
        "Test fresh owned enum Match result integration example",
        "cargo run -- check ../../examples/owned_enum_match_results/main.aero",
        "cargo run -- run ../../examples/owned_enum_match_results/main.aero",
        "opt-22 -passes=verify -disable-output ../../owned_enum_match_results.ll",
        "llc-22 -verify-machineinstrs ../../owned_enum_match_results.ll",
        "clang-22 -no-pie ../../owned_enum_match_results.o -o ../../owned_enum_match_results",
        "Expected exit code 203",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "stable/nightly workflow must contain exactly one {anchor:?} anchor"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-074 fresh owned-enum Match result failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}

#[test]
fn excluded_owned_enum_match_result_origins_fail_closed() {
    let mut failures = Vec::new();
    for (label, source, expected) in [
        (
            "identifier result",
            "enum I { A } enum O { X } fn main() { let existing = O::X; let result = match I::A { I::A => existing }; }",
            vec!["owned enum match result arm 1"],
        ),
        (
            "owned call argument",
            "enum I { A } enum O { X } fn forward(value: O) -> O { value } fn main() { let existing = O::X; let result = match I::A { I::A => forward(existing) }; }",
            vec!["owned enum match result arm 1"],
        ),
        (
            "nested external scrutinee",
            "enum I { A } enum O { X } fn main() { let inner = I::A; let result = match I::A { I::A => match inner { I::A => O::X } }; }",
            vec!["owned enum match result arm 1"],
        ),
        (
            "different enum results",
            "enum I { A, B } enum O { X } enum P { X } fn main() { let result = match I::A { I::A => O::X, I::B => P::X }; }",
            vec!["same type", "expected"],
        ),
        (
            "array results",
            "enum I { A } fn main() { let result = match I::A { I::A => [1, 2] }; }",
            vec![
                "must return Int, Float, Bool, Char, or one fresh admitted enum",
                "not admitted",
            ],
        ),
        (
            "struct results",
            "enum I { A } struct Cell { value: int } fn main() { let result = match I::A { I::A => Cell { value: 1 } }; }",
            vec![
                "must return Int, Float, Bool, Char, or one fresh admitted enum",
                "not admitted",
            ],
        ),
        (
            "borrowed result",
            "enum I { A } enum O { X } fn main() { let value = O::X; let result = match I::A { I::A => &value }; }",
            vec!["reference", "must return"],
        ),
        (
            "non-exhaustive patterns",
            "enum I { A, B } enum O { X } fn main() { let result = match I::A { I::A => O::X }; }",
            vec!["cover every declared variant"],
        ),
        (
            "foreign pattern",
            "enum I { A } enum J { A } enum O { X } fn main() { let result = match I::A { J::A => O::X }; }",
            vec!["enum match arm names `J`, expected `I`"],
        ),
        (
            "scrutinee reuse",
            "enum I { A } enum O { X } fn main() { let input = I::A; let result = match input { I::A => O::X }; let reused = input; }",
            vec!["moved value", "Use of moved"],
        ),
        (
            "loop reinitialization",
            "enum I { A } enum O { X } fn take(value: O) -> int { match value { O::X => 1 } } fn main() { let mut value = O::X; loop { let consumed = take(value); value = match I::A { I::A => O::X }; break; } }",
            vec!["reinitialization", "loop"],
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, &expected) {
            failures.push(failure);
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-074 exclusion failures:\n{}",
        failures.join("\n---\n")
    );
}

#[test]
fn deprecated_raw_generation_cannot_activate_checked_match_result_identity() {
    let source = complete_source();
    let raw = IrGenerator::new().generate_ir(parsed(source).expect("complete source parses"));
    let debug = format!("{raw:#?}");
    assert!(
        !debug.contains("CheckedEnumMatchResultPlaceAlloca"),
        "deprecated raw generation activated checked Match result identity:\n{debug}"
    );
}
