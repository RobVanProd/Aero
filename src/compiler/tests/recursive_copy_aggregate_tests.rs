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

const EXAMPLE_ROOT: &str = "examples/recursive_copy_aggregates/main.aero";
const EXAMPLE_MODULE: &str = "examples/recursive_copy_aggregates/shapes.aero";
const EXPECTED_EXIT: i32 = 109;

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
            "aero-recursive-copy-aggregate-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create recursive Copy-aggregate workspace");
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
            .is_some_and(|name| name.starts_with("aero-recursive-copy-aggregate-"));
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

fn expect_rejection(label: &str, source: &str) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded recursive Copy-data source compiled:\n{llvm}"
        )),
        Err(_) => None,
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
struct Cell { state: bool, meta: (int, bool), flags: [bool; 2] }
struct Frame {
    primary: Cell,
    cell_list: [Cell; 2],
    cells: [(Cell, [bool; 2]); 2],
    matrix: [[int; 2]; 2],
    mix: ((int, bool), [float; 2]),
    empty: [[bool; 2]; 0]
}
struct Envelope { frame: Frame }
struct Outer { envelope: Envelope }

fn make_cell(value: int) -> Cell {
    Cell { state: value > 0, meta: (value, value > 0), flags: [value > 0, value < 0] }
}
fn make_frame(seed: int) -> Frame {
    let first = make_cell(seed);
    let second = make_cell(seed + 1);
    Frame {
        primary: first,
        cell_list: [first, second],
        cells: [(first, [1 < 2, 1 > 2]), (second, [1 > 2, 1 < 2])],
        matrix: [[seed, seed + 1], [seed + 2, seed + 3]],
        mix: ((seed, seed > 0), [1.5, 2.5]),
        empty: []
    }
}
fn later(value: Frame) -> Frame { value }
fn forward(value: Frame) -> Frame { later(value) }
fn observe(value: &Frame) -> Frame { *value }
fn replace(value: &mut Frame) -> Frame {
    *value = make_frame(9);
    *value
}
fn recurse(value: Frame, remaining: int) -> Frame {
    if remaining > 0 { return recurse(value, remaining - 1); }
    value
}
fn select_cell(values: [Cell; 2], index: int) -> Cell { values[index] }
fn select_row(values: [[int; 2]; 2], index: int) -> [int; 2] { values[index] }
fn mixed(value: ((int, bool), [float; 2]), rows: [(Cell, [bool; 2]); 2])
    -> (((int, bool), [float; 2]), [(Cell, [bool; 2]); 2]) {
    (value, rows)
}

fn main() -> int {
    let original = make_frame(5);
    let copied: Frame = original;
    let forwarded = forward(copied);
    let observed = observe(&forwarded);
    let mut changed = recurse(observed, 2);
    let replaced = replace(&mut changed);
    changed = make_frame(7);
    let nested = mixed(changed.mix, changed.cells);
    let deep = Outer { envelope: Envelope { frame: changed } };
    let empty: [[bool; 2]; 0] = [];
    let exact_matrix: [[int; 2]; 2] = original.matrix;
    let exact_tuple: ((int, bool), [float; 2]) = original.mix;
    let exact_cells: [(Cell, [bool; 2]); 2] = original.cells;
    let selected = select_cell(original.cell_list, 1);
    let selected_row = select_row(observed.matrix, 1);
    if original.primary.flags[0]
        && original.cell_list[1].meta.0 == 6
        && forwarded.cells[1].0.meta.0 == 6
        && observed.matrix[1][1] == 8
        && (replaced.mix.0).0 == 9
        && ((nested.0).0).1
        && nested.1[1].1[1]
        && deep.envelope.frame.cells[0].0.state
        && deep.envelope.frame.mix.1[1] > 2.0
        && selected.meta.0 == 6
        && selected_row[1] == 8
        && exact_matrix[1][1] == 8
        && (exact_tuple.0).0 == 5
        && exact_cells[1].0.meta.0 == 6 {
        return 109;
    }
    1
}
"#
}

#[test]
fn recursive_copy_aggregate_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    let source = complete_source();
    match parsed_ast(source) {
        Err(error) => failures.push(format!(
            "recursive aggregate syntax was not retained: {error}"
        )),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            for marker in [
                "TupleLiteral",
                "ArrayLiteral",
                "StructLiteral",
                "TupleIndex",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!("parser AST missing {marker}:\n{debug}"));
                }
            }
        }
    }

    failures.extend(expect_success(
        "complete recursive constructor product and execution contexts",
        source,
        &[
            "[2 x i1]",
            "[2 x [2 x double]]",
            "{ %aero.struct.Cell, [2 x i1] }",
            "{ { double, i1 }, [2 x double] }",
            "define %aero.struct.Frame @forward",
            "call %aero.struct.Frame @replace",
        ],
    ));

    for (label, source, required) in [
        (
            "Bool arrays including zero length",
            "fn main() -> int { let flags = [1 < 2, 1 > 2]; let none: [bool; 0] = []; if flags[0] { return 11; } 1 }",
            vec!["[2 x i1]", "[0 x i1]"],
        ),
        (
            "every immediate recursive constructor pair",
            "struct S { x: int } struct C { a: [[int; 1]; 2], b: [(int, bool); 2], c: [S; 2], d: ([int; 2], (bool, int), S), e: S } fn main() -> int { let s = S { x: 3 }; let c = C { a: [[1], [2]], b: [(3, 1 < 2), (4, 1 > 2)], c: [s, s], d: ([5, 6], (1 < 2, 7), s), e: s }; if c.b[0].1 && (c.d.1).0 { return c.a[1][0] + c.c[1].x + c.d.0[1] + c.d.2.x + c.e.x; } 1 }",
            vec!["[2 x [1 x double]]", "[2 x { double, i1 }]"],
        ),
        (
            "recursive function transport and whole Copy references",
            "struct R { value: ((int, bool), [[float; 1]; 2]) } fn id(value: R) -> R { value } fn read(value: &R) -> R { *value } fn write(value: &mut R) -> R { *value = R { value: ((8, 1 < 2), [[1.5], [2.5]]) }; *value } fn main() -> int { let mut value = id(R { value: ((3, 1 < 2), [[0.5], [1.5]]) }); let before = read(&value); let after = write(&mut value); if (before.value.0).1 && after.value.1[1][0] > 2.0 { return (value.value.0).0; } 1 }",
            vec!["define %aero.struct.R @id", "[2 x [1 x double]]"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("recursive checked IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in ["Checked", "Tuple", "Array", "Struct"] {
                if !debug.contains(marker) {
                    failures.push(format!("checked IR missing {marker}:\n{debug}"));
                }
            }
            if llvm.contains("inttoptr") || llvm.contains("ptrtoint") || llvm.contains("bitcast") {
                failures.push(format!(
                    "recursive aggregate LLVM used a forbidden cast:\n{llvm}"
                ));
            }
        }
    }

    for (label, source) in [
        ("unit tuple", "fn main() -> int { let value = (); 0 }"),
        (
            "heterogeneous recursive array",
            "fn main() -> int { let value = [(1, 1 < 2), (2, 3.0)]; 0 }",
        ),
        (
            "recursive array annotation mismatch",
            "fn main() -> int { let value: [[int; 2]; 1] = [[1]]; 0 }",
        ),
        (
            "String leaf under tuple and array",
            "fn main() -> int { let value = [((1, \"x\"), 2); 2]; 0 }",
        ),
        (
            "self-recursive named struct",
            "struct Loop { next: Loop } fn consume(value: Loop) -> int { 0 } fn main() -> int { 0 }",
        ),
        (
            "mutually recursive named structs through zero array",
            "struct A { values: [B; 0] } struct B { value: A } fn consume(value: A) -> int { 0 } fn main() -> int { 0 }",
        ),
        (
            "empty named struct",
            "struct Empty {} fn consume(value: Empty) -> int { 0 } fn main() -> int { 0 }",
        ),
        (
            "duplicate named struct",
            "struct Duplicate { value: int } struct Duplicate { value: int } fn consume(value: Duplicate) -> int { 0 } fn main() -> int { 0 }",
        ),
        (
            "unresolved nested named struct",
            "struct MissingLeaf { values: [Unknown; 1] } fn consume(value: MissingLeaf) -> int { 0 } fn main() -> int { 0 }",
        ),
        (
            "recursive constant out-of-bounds index",
            "fn main() -> int { let value = [[1]]; return value[1][0]; }",
        ),
        (
            "aggregate comparison",
            "fn main() -> int { if (1, 2) == (1, 2) { return 1; } 0 }",
        ),
        (
            "projected mutable borrow",
            "struct S { values: [[int; 1]; 1] } fn main() -> int { let mut value = S { values: [[1]] }; let alias = &mut value.values[0]; 0 }",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source) {
            failures.push(failure);
        }
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for path in [&tracked_root, &tracked_module] {
        if !path.is_file() {
            failures.push(format!(
                "tracked recursive example is missing: {}",
                path.display()
            ));
        }
    }

    if tracked_root.is_file() && tracked_module.is_file() {
        let workspace = TestWorkspace::new("tracked-example");
        let output = workspace.path("recursive-copy-aggregate.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output.is_file() {
            failures.push(format!(
                "tracked recursive Copy-aggregate example failed checked build (artifact={}):\n{diagnostics}",
                output.is_file()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-062 recursive Copy-aggregate failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
