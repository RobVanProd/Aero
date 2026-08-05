use compiler::ast::{AstNode, Statement, Type};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/mutable_copy_place_references/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_copy_place_references/mutations.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 59;

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
            "aero-mutable-copy-place-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create mutable Copy-place test workspace");
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
            .is_some_and(|name| name.starts_with("aero-mutable-copy-place-"));
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
            "{label}: unsupported mutable Copy-place source compiled:\n{llvm}"
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

fn complete_source() -> &'static str {
    r#"
struct Leaf { x: int, y: int }
struct Frame { leaf: Leaf, rows: [Leaf; 2], bias: int }
struct Envelope { frame: Frame }

fn replace_leaf(value: &mut Leaf) -> Leaf {
    *value = Leaf { x: 7, y: 8 };
    *value
}
fn replace_frame(value: &mut Frame) -> Frame {
    let leaf = Leaf { x: 9, y: 10 };
    let rows = [leaf, leaf];
    *value = Frame { leaf: leaf, rows: rows, bias: 11 };
    *value
}
fn replace_envelope(value: &mut Envelope) -> int {
    let leaf = Leaf { x: 12, y: 13 };
    let rows = [leaf, leaf];
    let frame = Frame { leaf: leaf, rows: rows, bias: 14 };
    *value = Envelope { frame: frame };
    (*value).frame.rows[1].y + (*value).frame.bias
}
fn replace_tuple(value: &mut (int, float, bool)) -> (int, float, bool) {
    *value = (15, 3.5, 1 < 2);
    *value
}
fn replace_values(value: &mut [int; 3]) -> [int; 3] {
    *value = [16, 17, 18];
    *value
}
fn replace_rows(value: &mut [Leaf; 2]) -> [Leaf; 2] {
    let leaf = Leaf { x: 19, y: 20 };
    *value = [leaf, leaf];
    *value
}
fn forward(value: &mut Frame) -> int { replace_frame(value).rows[1].y }
fn recurse(value: &mut Leaf) -> Leaf {
    let current = *value;
    if current.x < 3 {
        *value = Leaf { x: current.x + 1, y: current.y };
        recurse(value);
    }
    *value
}
fn observe(value: &mut Frame) { let copied = *value; *value = copied; }

fn main() -> int {
    let mut scalar = 1;
    { let alias = &mut scalar; *alias = 2; }
    let mut ratio = 1.5;
    { let alias = &mut ratio; *alias = 2.5; }
    let mut ready = 1 == 2;
    { let alias = &mut ready; *alias = 1 < 2; }

    let mut leaf = Leaf { x: 1, y: 2 };
    let leaf_copy = replace_leaf(&mut leaf);
    let seed = Leaf { x: 0, y: 1 };
    let seed_rows = [seed, seed];
    let mut frame = Frame { leaf: seed, rows: seed_rows, bias: 2 };
    let mut forwarded = 0;
    { let alias: &mut Frame = &mut frame; forwarded = forward(alias); }
    observe(&mut frame);
    let mut envelope = Envelope { frame: frame };
    let envelope_score = replace_envelope(&mut envelope);
    let mut tuple = (1, 1.5, 1 == 2);
    let tuple_copy = replace_tuple(&mut tuple);
    let mut values = [1, 2, 3];
    let values_copy = replace_values(&mut values);
    let mut rows = [seed, seed];
    let rows_copy = replace_rows(&mut rows);
    let mut empty_numbers: [int; 0] = [];
    let empty_number_replacement: [int; 0] = [];
    { let alias = &mut empty_numbers; *alias = empty_number_replacement; }
    let mut empty_rows: [Leaf; 0] = [];
    let empty_row_replacement: [Leaf; 0] = [];
    { let alias = &mut empty_rows; *alias = empty_row_replacement; }
    let mut counter = Leaf { x: 0, y: 4 };
    let counted = recurse(&mut counter);

    if ready && ratio > 2.0 && scalar == 2 && leaf_copy.x == 7
        && forwarded == 10 && envelope_score == 27 && tuple_copy.2
        && tuple.0 == 15 && values_copy[2] == 18 && values.len() == 3
        && rows_copy[0].x == 19 && rows[1].y == 20
        && empty_numbers.len() == 0 && empty_rows.len() == 0
        && counted.x == 3 && counter.y == 4 {
        return leaf.y + frame.bias + envelope.frame.leaf.x;
    }
    1
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod mutations;

enum Mode { Off, On }
struct Point { x: int, y: int }
struct Frame { point: Point, rows: [Point; 2], bias: int }

fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 10 } }

fn main() -> int {
    let mut point = Point { x: 1, y: 2 };
    {
        let alias: &mut Point = &mut point;
        *alias = Point { x: 3, y: 4 };
    }

    let seed = Point { x: 0, y: 0 };
    let seed_rows = [seed, seed];
    let mut frame = Frame { point: seed, rows: seed_rows, bias: 0 };
    let mut frame_score = 0;
    {
        let frame_alias = &mut frame;
        frame_score = forward_frame(frame_alias);
    }

    let mut tuple = (0, 0.5, 1 == 2);
    let tuple_copy = set_tuple(&mut tuple);
    let mut values = [0, 0, 0];
    let values_copy = set_values(&mut values);
    let mut rows = [seed, seed];
    let rows_copy = set_rows(&mut rows);
    let mut empty: [int; 0] = [];
    let empty_replacement: [int; 0] = [];
    { let empty_alias = &mut empty; *empty_alias = empty_replacement; }

    if tuple_copy.2 && tuple_copy.1 > 2.0 && frame.bias == 9
        && values[1] == 4 && rows[0].y == 5 && empty.len() == 0 {
        return frame_score + tuple_copy.0 + values_copy[2] + rows_copy[1].x
            + point.x + point.y + "aero".len() + mode_score(Mode::On);
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn set_frame(value: &mut Frame) -> Frame {
    let point = Point { x: 7, y: 8 };
    let rows = [point, point];
    *value = Frame { point: point, rows: rows, bias: 9 };
    *value
}

fn forward_frame(value: &mut Frame) -> int {
    let copied = set_frame(value);
    copied.point.x + copied.rows[1].y + copied.bias
}

fn set_tuple(value: &mut (int, float, bool)) -> (int, float, bool) {
    *value = (5, 2.5, 1 < 2);
    *value
}

fn set_values(value: &mut [int; 3]) -> [int; 3] {
    *value = [2, 4, 6];
    *value
}

fn set_rows(value: &mut [Point; 2]) -> [Point; 2] {
    let point = Point { x: 3, y: 5 };
    *value = [point, point];
    *value
}
"#
}

#[test]
fn mutable_copy_place_reference_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = "struct Row { value: int } fn replace(row: &mut Row, values: &mut [int; 3], pair: &mut (int, bool)) -> Row { *row } fn main() -> int { 0 }";
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!("parser retention failed: {error}")),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::Function { parameters, .. })) = ast.get(1)
            else {
                failures.push(format!(
                    "parser omitted mutable Copy-place signature: {ast:?}"
                ));
                return assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
            };
            let retained = matches!(
                &parameters[0].param_type,
                Type::Reference(inner, true)
                    if matches!(inner.as_ref(), Type::Named(name) if name == "Row")
            ) && matches!(
                &parameters[1].param_type,
                Type::Reference(inner, true)
                    if matches!(inner.as_ref(), Type::Array(element, 3)
                        if matches!(element.as_ref(), Type::Named(name) if name == "int"))
            ) && matches!(
                &parameters[2].param_type,
                Type::Reference(inner, true)
                    if matches!(inner.as_ref(), Type::Tuple(elements) if elements.len() == 2)
            );
            if !retained {
                failures.push(format!(
                    "parser changed mutable Copy-place annotation topology: {parameters:?}"
                ));
            }
        }
    }

    failures.extend(expect_success(
        "complete local, call, child-reborrow, CFG, and recursive product",
        complete_source(),
        &[
            "define %aero.struct.Leaf @replace_leaf(%aero.struct.Leaf*",
            "define %aero.struct.Frame @replace_frame(%aero.struct.Frame*",
            "define { double, double, i1 } @replace_tuple({ double, double, i1 }*",
            "define [3 x double] @replace_values([3 x double]*",
            "define [2 x %aero.struct.Leaf] @replace_rows([2 x %aero.struct.Leaf]*",
            "call %aero.struct.Frame @replace_frame(%aero.struct.Frame* %ptr",
            "store %aero.struct.Leaf",
            "store %aero.struct.Frame",
            "store { double, double, i1 }",
            "store [3 x double]",
        ],
    ));

    for (label, source, required) in [
        (
            "inferred struct alias and whole replacement",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; { let alias = &mut row; *alias = Row { value: 9 }; let copy = *alias; } row.value }",
            vec!["store %aero.struct.Row", "load %aero.struct.Row"],
        ),
        (
            "annotated flat tuple alias",
            "fn main() -> int { let mut pair = (1, 1.5, 1 == 2); { let alias: &mut (int, float, bool) = &mut pair; *alias = (6, 2.5, 1 < 2); } if pair.2 { return pair.0; } 1 }",
            vec!["store { double, double, i1 }"],
        ),
        (
            "zero numeric and Copy-struct arrays",
            "struct Row { value: int } fn main() -> int { let mut numbers: [int; 0] = []; let replacement: [int; 0] = []; { let alias = &mut numbers; *alias = replacement; } let mut rows: [Row; 0] = []; let row_replacement: [Row; 0] = []; { let alias = &mut rows; *alias = row_replacement; } numbers.len() + rows.len() }",
            vec!["[0 x double]", "[0 x %aero.struct.Row]"],
        ),
        (
            "nearest shadowed aggregate aliases",
            "struct Row { value: int } fn main() -> int { let mut left = Row { value: 1 }; let mut right = Row { value: 2 }; { let alias = &mut left; *alias = Row { value: 3 }; { let alias = &mut right; *alias = Row { value: 4 }; } } left.value + right.value }",
            vec!["store %aero.struct.Row"],
        ),
        (
            "loop and whole-place mutation",
            "struct Counter { value: int } fn climb(value: &mut Counter) -> Counter { while (*value).value < 3 { let current = *value; *value = Counter { value: current.value + 1 }; } *value } fn main() -> int { let mut value = Counter { value: 0 }; climb(&mut value).value }",
            vec!["while_start", "store %aero.struct.Counter"],
        ),
        (
            "Void mutable aggregate consumer",
            "struct Row { value: int } fn set(value: &mut Row) { *value = Row { value: 5 }; } fn main() -> int { let mut row = Row { value: 1 }; set(&mut row); row.value }",
            vec![
                "define void @set(%aero.struct.Row*",
                "call void @set(%aero.struct.Row*",
            ],
        ),
        (
            "recursive Bool array mutable pointee",
            "fn replace(value: &mut [bool; 2]) -> [bool; 2] { *value = [2 < 1, 1 < 2]; *value } fn main() -> int { let mut value = [1 < 2, 2 < 1]; let copy = replace(&mut value); if copy[1] { return 1; } 0 }",
            vec!["define [2 x i1] @replace([2 x i1]*", "store [2 x i1]"],
        ),
        (
            "recursive nested tuple mutable pointee",
            "fn replace(value: &mut ((int, int), bool)) -> ((int, int), bool) { *value = ((4, 5), 1 < 2); *value } fn main() -> int { let mut value = ((1, 2), 2 < 1); let copy = replace(&mut value); if copy.1 { return (copy.0).1; } 0 }",
            vec![
                "define { { double, double }, i1 } @replace({ { double, double }, i1 }*",
                "store { { double, double }, i1 }",
            ],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(complete_source()) {
        Err(error) => failures.push(format!(
            "checked mutable Copy-place IR/LLVM failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedMutableCopyPlaceAlloca",
                "CheckedMutableReferenceParameter",
                "CheckedMutableBorrow",
                "CheckedMutableDereferenceAssignment",
                "CheckedMutableBorrowEnd",
                "MutableReference {",
                "Struct {",
                "Array {",
                "Tuple {",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked IR omitted mutable Copy-place marker {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["inttoptr", "ptrtoint", "bitcast"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "mutable Copy-place LLVM used forbidden {forbidden}:\n{llvm}"
                    ));
                }
            }
        }
    }

    let rejected = [
        (
            "immutable aggregate owner",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let alias = &mut row; 0 }",
            "must be declared mutable",
        ),
        (
            "String pointee",
            "fn bad(value: &mut String) { } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "enum pointee",
            "enum Mode { Off, On } fn bad(value: &mut Mode) { } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "non-Copy struct pointee",
            "struct Bad { text: String } fn bad(value: &mut Bad) { } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "projected field origin",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let alias = &mut row.value; 0 }",
            "identifier place",
        ),
        (
            "projected index origin",
            "fn main() -> int { let mut values = [1, 2]; let alias = &mut values[0]; 0 }",
            "identifier place",
        ),
        (
            "literal origin",
            "fn main() -> int { let alias = &mut 1; 0 }",
            "identifier place",
        ),
        (
            "immutable reference assignment",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let alias = &row; *alias = Row { value: 2 }; 0 }",
            "immutable reference",
        ),
        (
            "whole replacement mismatch",
            "struct Left { value: int } struct Right { value: int } fn main() -> int { let mut value = Left { value: 1 }; let alias = &mut value; *alias = Right { value: 2 }; 0 }",
            "type mismatch",
        ),
        (
            "second mutable loan",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let first = &mut row; let second = &mut row; 0 }",
            "already borrowed as mutable",
        ),
        (
            "immutable loan conflict",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let first = &row; let second = &mut row; 0 }",
            "also borrowed as immutable",
        ),
        (
            "mixed mutable signature",
            "struct Row { value: int } fn bad(value: &mut Row, bias: int) { } fn main() -> int { 0 }",
            "exactly one mutable",
        ),
        (
            "multiple mutable signature",
            "struct Row { value: int } fn bad(left: &mut Row, right: &mut Row) { } fn main() -> int { 0 }",
            "exactly one mutable",
        ),
        (
            "reference result escape",
            "struct Row { value: int } fn bad(value: &mut Row) -> &mut Row { value } fn main() -> int { 0 }",
            "reference results require lifetime semantics",
        ),
    ];
    for (label, source, expected) in rejected {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    match analyzed_ast(
        "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; { let alias = &mut row; *alias = Row { value: 2 }; } row.value }",
    ) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedMutableCopyPlaceAlloca")
                || debug.contains("CheckedMutableBorrowEnd")
                || debug.contains("CheckedMutableReferenceParameter")
            {
                failures.push(format!(
                    "deprecated raw generation activated checked mutable Copy-place identities:\n{debug}"
                ));
            }
        }
    }

    let root = repository_root();
    for (label, relative, expected) in [
        ("tracked root", EXAMPLE_ROOT, tracked_root_source()),
        ("tracked module", EXAMPLE_MODULE, tracked_module_source()),
    ] {
        let path = root.join(relative);
        match fs::read_to_string(&path) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!(
                "{label} {} drifted from frozen source:\n{actual}",
                path.display()
            )),
            Err(error) => failures.push(format!(
                "{label} {} is unavailable: {error}",
                path.display()
            )),
        }
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for mutable Copy-place integration anchors");
    let lane_start = workflow
        .find("    - name: Test mutable Copy-place reference integration example")
        .unwrap_or(workflow.len());
    let lane_tail = &workflow[lane_start..];
    let lane_end = lane_tail
        .find("\n    - name: Run tests")
        .unwrap_or(lane_tail.len());
    let lane = &lane_tail[..lane_end];
    for anchor in [
        "Test mutable Copy-place reference integration example",
        "cargo run -- build ../../examples/mutable_copy_place_references/main.aero -o ../../mutable_copy_place_references.ll",
        "opt-22 -passes=verify -disable-output ../../mutable_copy_place_references.ll",
        "llc-22 -verify-machineinstrs ../../mutable_copy_place_references.ll -o /dev/null",
        "llc-22 -filetype=obj ../../mutable_copy_place_references.ll -o ../../mutable_copy_place_references.o",
        "clang-22 ../../mutable_copy_place_references.o -o ../../mutable_copy_place_references",
        "if [ $exit_code -ne 59 ]; then",
        "mutable Copy-place reference example passed with exit code 59",
    ] {
        let count = lane.matches(anchor).count();
        if count != 1 {
            failures.push(format!(
                "workflow anchor {anchor:?} occurs {count} times instead of once"
            ));
        }
    }

    let example = root.join(EXAMPLE_ROOT);
    let workspace = TestWorkspace::new("tracked-example");
    let output_path = workspace.path("mutable-copy-place.ll");
    let build = run_cli(
        &workspace,
        &[Path::new("build"), &example, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&build);
    if !build.status.success() || !output_path.is_file() {
        failures.push(format!(
            "tracked mutable Copy-place example failed checked CLI build:\n{diagnostics}"
        ));
    }

    let invalid = workspace.path("invalid.aero");
    let invalid_output = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let alias = &mut row.value; 0 }",
    )
    .expect("write invalid mutable Copy-place source");
    let rejected = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_output,
        ],
    );
    let rejected_text = output_text(&rejected);
    if rejected.status.success()
        || invalid_output.exists()
        || !rejected_text.contains("identifier place")
    {
        failures.push(format!(
            "invalid mutable Copy-place CLI hygiene failed (status={}, artifact={}):\n{}",
            rejected.status,
            invalid_output.exists(),
            rejected_text
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-060 mutable Copy-place reference failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
