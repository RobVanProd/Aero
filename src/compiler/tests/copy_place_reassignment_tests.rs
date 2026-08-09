use compiler::ast::{AstNode, Expression, Statement};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/copy_place_reassignment/main.aero";
const EXAMPLE_MODULE: &str = "examples/copy_place_reassignment/updates.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 83;

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
            "aero-copy-place-reassignment-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create Copy-place reassignment workspace");
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
            .is_some_and(|name| name.starts_with("aero-copy-place-reassignment-"));
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
            "{label}: unsupported Copy-place reassignment compiled:\n{llvm}"
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

fn make_leaf(seed: int) -> Leaf { Leaf { x: seed, y: seed + 1 } }
fn replace_leaf(start: Leaf) -> Leaf {
    let mut value = start;
    value = Leaf { x: 7, y: 8 };
    value
}
fn replace_frame(start: Frame) -> Frame {
    let mut value: Frame = start;
    let leaf = Leaf { x: 9, y: 10 };
    let rows = [leaf, leaf];
    value = Frame { leaf: leaf, rows: rows, bias: 11 };
    value
}
fn replace_envelope(start: Envelope) -> Envelope {
    let mut value = start;
    let leaf = Leaf { x: 12, y: 13 };
    let rows = [leaf, leaf];
    let frame = Frame { leaf: leaf, rows: rows, bias: 14 };
    value = Envelope { frame: frame };
    value
}
fn recurse(value: Leaf, count: int) -> Leaf {
    let mut next = value;
    if count > 0 {
        next = make_leaf(count);
        next = recurse(next, count - 1);
    }
    next
}
fn copy_from_shared(source: Leaf) -> Leaf {
    let mut target = Leaf { x: 0, y: 0 };
    { let view = &source; target = *view; }
    target
}
fn copy_from_mutable() -> Leaf {
    let mut source = Leaf { x: 21, y: 22 };
    let mut target = Leaf { x: 0, y: 0 };
    { let view = &mut source; target = *view; }
    target
}

fn main() -> int {
    let mut scalar = 1;
    scalar = 2;
    let mut ratio: float = 1.5;
    ratio = 2.5;
    let mut ready = 1 == 2;
    ready = 1 < 2;

    let seed = Leaf { x: 1, y: 2 };
    let mut leaf = seed;
    leaf = Leaf { x: 3, y: 4 };
    leaf = make_leaf(5);
    let leaf_copy = leaf;

    let rows = [seed, seed];
    let frame_seed = Frame { leaf: seed, rows: rows, bias: 0 };
    let mut frame: Frame = frame_seed;
    frame = Frame { leaf: leaf, rows: rows, bias: 6 };
    frame = replace_frame(frame);
    let envelope_seed = Envelope { frame: frame };
    let mut envelope = envelope_seed;
    envelope = replace_envelope(envelope);

    let mut tuple = (1, 1.5, 1 == 2);
    tuple = (15, 3.5, 1 < 2);
    let tuple_copy = tuple;
    let mut values = [1, 2, 3];
    values = [16, 17, 18];
    let values_copy = values;
    let mut ratios = [1.5, 2.5];
    ratios = [3.5, 4.5];
    let mut leaf_rows = [seed, seed];
    let row_value = Leaf { x: 19, y: 20 };
    leaf_rows = [row_value, row_value];
    let row_copy = leaf_rows;

    let mut empty_numbers: [int; 0] = [];
    let empty_number_replacement: [int; 0] = [];
    empty_numbers = empty_number_replacement;
    let mut empty_ratios: [float; 0] = [];
    let empty_ratio_replacement: [float; 0] = [];
    empty_ratios = empty_ratio_replacement;
    let mut empty_rows: [Leaf; 0] = [];
    let empty_row_replacement: [Leaf; 0] = [];
    empty_rows = empty_row_replacement;

    let mut selected = Leaf { x: 0, y: 0 };
    if ready { selected = Leaf { x: 1, y: 2 }; }
    else { selected = Leaf { x: 30, y: 31 }; }
    let mut looped = Leaf { x: 0, y: 4 };
    let mut step = 0;
    while step < 3 {
        looped = Leaf { x: looped.x + 1, y: looped.y };
        step = step + 1;
    }
    let mut total = 0;
    for item in values { total = total + item; }

    let mut shadow = Leaf { x: 1, y: 1 };
    { let mut shadow = Leaf { x: 2, y: 2 }; shadow = Leaf { x: 3, y: 3 }; }
    shadow = Leaf { x: 4, y: 4 };

    let mut after_borrow = Leaf { x: 1, y: 1 };
    { let alias = &mut after_borrow; *alias = Leaf { x: 2, y: 2 }; }
    after_borrow = Leaf { x: 5, y: 5 };
    let shared_copy = copy_from_shared(after_borrow);
    let mutable_copy = copy_from_mutable();
    let recursive = recurse(seed, 2);

    if scalar == 2 && ratio > 2.0 && ready && leaf_copy.y == 6
        && frame.bias == 11 && envelope.frame.bias == 14 && tuple_copy.2
        && tuple.0 == 15 && values_copy[2] == 18 && ratios[1] > 4.0
        && row_copy[0].x == 19 && leaf_rows[1].y == 20
        && empty_numbers.len() == 0 && empty_ratios.len() == 0
        && empty_rows.len() == 0 && selected.y == 2 && looped.x == 3
        && total == 51 && shadow.x == 4 && after_borrow.y == 5
        && shared_copy.x == 5 && mutable_copy.y == 22 && recursive.y == 2 {
        return replace_leaf(seed).x + replace_frame(frame_seed).bias;
    }
    1
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod updates;

enum Mode { Off, On }
struct Point { x: int, y: int }
struct Frame { point: Point, rows: [Point; 2], bias: int }

fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 10 } }

fn main() -> int {
    let mut point = Point { x: 1, y: 2 };
    point = Point { x: 3, y: 4 };
    let mut viewed = 0;
    { let view = &point; viewed = (*view).x + (*view).y; }

    let mut borrowed = Point { x: 1, y: 1 };
    { let alias = &mut borrowed; *alias = Point { x: 2, y: 2 }; }
    borrowed = Point { x: 4, y: 5 };

    let mut empty: [int; 0] = [];
    let empty_replacement: [int; 0] = [];
    empty = empty_replacement;

    if empty.len() == 0 {
        return update_frame() + update_tuple() + update_values() + update_rows()
            + cfg_score() + viewed + borrowed.x + borrowed.y
            + "aero".len() + mode_score(Mode::On);
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn update_frame() -> int {
    let seed = Point { x: 0, y: 0 };
    let rows = [seed, seed];
    let mut frame = Frame { point: seed, rows: rows, bias: 0 };
    let point = Point { x: 5, y: 6 };
    let updated_rows = [point, point];
    frame = Frame { point: point, rows: updated_rows, bias: 7 };
    frame.point.x + frame.point.y + frame.rows[1].x + frame.bias
}

fn update_tuple() -> int {
    let mut tuple = (0, 0.5, 1 == 2);
    tuple = (8, 2.5, 1 < 2);
    if tuple.2 { return tuple.0 + 1; }
    0
}

fn update_values() -> int {
    let mut values = [0, 0, 0];
    values = [2, 4, 6];
    values[2]
}

fn update_rows() -> int {
    let seed = Point { x: 0, y: 0 };
    let mut rows = [seed, seed];
    let point = Point { x: 7, y: 8 };
    rows = [point, point];
    rows[1].y
}

fn cfg_score() -> int {
    let mut selected = Point { x: 0, y: 0 };
    if 1 < 2 { selected = Point { x: 1, y: 2 }; }
    else { selected = Point { x: 20, y: 21 }; }
    let mut step = 0;
    while step < 2 {
        selected = Point { x: selected.x + 1, y: selected.y + 1 };
        step = step + 1;
    }
    selected.x + selected.y
}
"#
}

#[test]
fn copy_place_reassignment_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row = Row { value: 2 }; let mut pair = (1, 1 == 2); pair = (2, 1 < 2); let mut values = [1, 2]; values = [3, 4]; row.value + pair.0 + values[0] }";
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!("parser retention failed: {error}")),
        Ok(ast) => {
            let targets = ast
                .iter()
                .find_map(|node| match node {
                    AstNode::Statement(Statement::Function { name, body, .. })
                        if name == "main" =>
                    {
                        Some(
                            body.statements
                                .iter()
                                .filter_map(|statement| match statement {
                                    Statement::Assignment {
                                        target: Expression::Identifier(name),
                                        ..
                                    } => Some(name.as_str()),
                                    _ => None,
                                })
                                .collect::<Vec<_>>(),
                        )
                    }
                    _ => None,
                })
                .unwrap_or_default();
            if targets != ["row", "pair", "values"] {
                failures.push(format!(
                    "parser changed direct Copy-place assignment topology: {targets:?}\n{ast:#?}"
                ));
            }
        }
    }

    failures.extend(expect_success(
        "complete Copy-place assignment product",
        complete_source(),
        &[
            "store %aero.struct.Leaf",
            "store %aero.struct.Frame",
            "store %aero.struct.Envelope",
            "store { double, double, i1 }",
            "store [3 x double]",
            "store [2 x %aero.struct.Leaf]",
            "while_start",
        ],
    ));

    for (label, source, required) in [
        (
            "inferred and annotated structs",
            "struct Row { value: int } fn make(value: int) -> Row { Row { value: value } } fn main() -> int { let mut left = Row { value: 1 }; left = make(2); let mut right: Row = left; right = Row { value: 3 }; left.value + right.value }",
            vec!["store %aero.struct.Row"],
        ),
        (
            "flat tuple products",
            "fn main() -> int { let mut pair = (1, 1.5, 1 == 2); pair = (2, 2.5, 1 < 2); let copy = pair; if copy.2 { return copy.0; } 0 }",
            vec!["store { double, double, i1 }"],
        ),
        (
            "zero and nonzero numeric arrays",
            "fn main() -> int { let mut ints = [1, 2, 3]; ints = [3, 4, 5]; let mut floats = [1.5, 2.5]; floats = [3.5, 4.5]; let mut empty: [float; 0] = []; let replacement: [float; 0] = []; empty = replacement; ints[2] + empty.len() }",
            vec!["store [3 x double]", "store [2 x double]", "[0 x double]"],
        ),
        (
            "recursive Bool array assignment",
            "fn main() -> int { let mut values = [1 < 2, 2 < 3]; values = [2 < 1, 3 < 2]; if values[0] { return 1; } 0 }",
            vec!["store [2 x i1]"],
        ),
        (
            "recursive nested tuple assignment",
            "fn main() -> int { let mut value = ((1, 2), 1 < 2); value = ((3, 4), 2 < 3); if value.1 { return (value.0).1; } 0 }",
            vec!["store { { double, double }, i1 }"],
        ),
        (
            "zero and nonzero Copy-struct arrays",
            "struct Row { value: int } fn main() -> int { let seed = Row { value: 1 }; let mut rows = [seed, seed]; let next = Row { value: 2 }; rows = [next, next]; let mut empty: [Row; 0] = []; let replacement: [Row; 0] = []; empty = replacement; rows[1].value + empty.len() }",
            vec!["store [2 x %aero.struct.Row]", "[0 x %aero.struct.Row]"],
        ),
        (
            "finite acyclic recursive schema",
            "struct Leaf { value: int } struct Frame { leaf: Leaf, rows: [Leaf; 2] } struct Envelope { frame: Frame } fn main() -> int { let leaf = Leaf { value: 1 }; let rows = [leaf, leaf]; let frame = Frame { leaf: leaf, rows: rows }; let mut value = Envelope { frame: frame }; let next = Leaf { value: 7 }; let next_rows = [next, next]; let next_frame = Frame { leaf: next, rows: next_rows }; value = Envelope { frame: next_frame }; value.frame.rows[1].value }",
            vec!["store %aero.struct.Envelope"],
        ),
        (
            "shared and mutable reference Copy RHS",
            "struct Row { value: int } fn main() -> int { let source = Row { value: 3 }; let mut from_shared = Row { value: 0 }; { let view = &source; from_shared = *view; } let mut mutable_source = Row { value: 4 }; let mut from_mutable = Row { value: 0 }; { let view = &mut mutable_source; from_mutable = *view; } from_shared.value + from_mutable.value }",
            vec!["load %aero.struct.Row", "store %aero.struct.Row"],
        ),
        (
            "branch loop shadow and post-borrow owner",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 0 }; if 1 < 2 { row = Row { value: 1 }; } else { row = Row { value: 9 }; } let mut step = 0; while step < 2 { row = Row { value: row.value + 1 }; step = step + 1; } { let mut row = Row { value: 4 }; row = Row { value: 5 }; } { let alias = &mut row; *alias = Row { value: 6 }; } row = Row { value: 7 }; row.value }",
            vec!["while_start", "store %aero.struct.Row"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(complete_source()) {
        Err(error) => failures.push(format!("checked Copy-place assignment failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedMutableOwnedPlaceAlloca",
                "CheckedOwnedPlaceAssignment",
                "Struct {",
                "Array {",
                "Tuple {",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked IR omitted Copy-place assignment marker {marker:?}:\n{debug}"
                    ));
                }
            }
            for obsolete in ["CheckedMutableScalarAlloca", "CheckedScalarAssignment"] {
                if debug.contains(obsolete) {
                    failures.push(format!(
                        "checked IR retained obsolete split identity {obsolete}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["inttoptr", "ptrtoint", "bitcast"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "Copy-place assignment LLVM used forbidden {forbidden}:\n{llvm}"
                    ));
                }
            }
        }
    }

    let rejected = [
        (
            "immutable aggregate owner",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; row = Row { value: 2 }; row.value }",
            "mutable local owned binding",
        ),
        (
            "owned parameter target",
            "struct Row { value: int } fn bad(row: Row) -> Row { row = Row { value: 2 }; row } fn main() -> int { 0 }",
            "mutable local owned binding",
        ),
        (
            "String target",
            "fn main() -> int { let mut value = \"a\"; value = \"b\"; 0 }",
            "admitted Copy-data for owned assignment",
        ),
        (
            "non-Copy struct target",
            "struct Bad { text: String } fn main() -> int { let mut value = Bad { text: \"a\" }; value = Bad { text: \"b\" }; 0 }",
            "Struct construction expressions are not supported",
        ),
        (
            "field assignment target",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row.value = 2; row.value }",
            "assignment target must be a local identifier",
        ),
        (
            "index assignment target",
            "fn main() -> int { let mut values = [1, 2]; values[0] = 3; values[0] }",
            "assignment target must be a local identifier",
        ),
        (
            "tuple projection target",
            "fn main() -> int { let mut value = (1, 2); value.0 = 3; value.0 }",
            "assignment target must be a local identifier",
        ),
        (
            "struct schema mismatch",
            "struct Left { value: int } struct Right { value: int } fn main() -> int { let mut value = Left { value: 1 }; value = Right { value: 2 }; 0 }",
            "type mismatch",
        ),
        (
            "immutable borrow conflict",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let view = &row; row = Row { value: 2 }; view.value }",
            "while it is borrowed",
        ),
        (
            "mutable borrow conflict",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let view = &mut row; row = Row { value: 2 }; (*view).value }",
            "while it is borrowed",
        ),
        (
            "assignment expression value",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let copy = (row = Row { value: 2 }); 0 }",
            "Parse error",
        ),
        (
            "chained assignment",
            "struct Row { value: int } fn main() -> int { let mut left = Row { value: 1 }; let mut right = Row { value: 2 }; left = right = Row { value: 3 }; 0 }",
            "Parse error",
        ),
        (
            "compound assignment",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row += Row { value: 2 }; 0 }",
            "Parse error",
        ),
    ];
    for (label, source, expected) in rejected {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    failures.extend(expect_success(
        "separate mutable-reference dereference assignment remains admitted",
        "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; { let alias = &mut row; *alias = Row { value: 2 }; } row.value }",
        &["store %aero.struct.Row"],
    ));

    match analyzed_ast(
        "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row = Row { value: 2 }; row.value }",
    ) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedMutableOwnedPlaceAlloca")
                || debug.contains("CheckedOwnedPlaceAssignment")
            {
                failures.push(format!(
                    "deprecated raw generation activated checked Copy-place assignment identities:\n{debug}"
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
        .expect("read Rust workflow for Copy-place reassignment anchors");
    let lane_start = workflow
        .find("    - name: Test Copy-place reassignment integration example")
        .unwrap_or(workflow.len());
    let lane_tail = &workflow[lane_start..];
    let lane_end = lane_tail
        .find("\n    - name: Run tests")
        .unwrap_or(lane_tail.len());
    let lane = &lane_tail[..lane_end];
    for anchor in [
        "Test Copy-place reassignment integration example",
        "cargo run -- build ../../examples/copy_place_reassignment/main.aero -o ../../copy_place_reassignment.ll",
        "opt-22 -passes=verify -disable-output ../../copy_place_reassignment.ll",
        "llc-22 -verify-machineinstrs ../../copy_place_reassignment.ll -o /dev/null",
        "llc-22 -filetype=obj ../../copy_place_reassignment.ll -o ../../copy_place_reassignment.o",
        "clang-22 ../../copy_place_reassignment.o -o ../../copy_place_reassignment",
        "if [ $exit_code -ne 83 ]; then",
        "Copy-place reassignment example passed with exit code 83",
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
    let output_path = workspace.path("copy-place-reassignment.ll");
    let build = run_cli(
        &workspace,
        &[Path::new("build"), &example, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&build);
    if !build.status.success() || !output_path.is_file() {
        failures.push(format!(
            "tracked Copy-place reassignment example failed checked CLI build:\n{diagnostics}"
        ));
    }

    let invalid = workspace.path("invalid.aero");
    let invalid_output = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; row = Row { value: 2 }; 0 }",
    )
    .expect("write invalid Copy-place reassignment source");
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
        || !rejected_text.contains("mutable local owned binding")
    {
        failures.push(format!(
            "invalid Copy-place reassignment CLI hygiene failed (status={}, artifact={}):\n{}",
            rejected.status,
            invalid_output.exists(),
            rejected_text
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-061 Copy-place reassignment failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
