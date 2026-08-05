use compiler::ast::{AstNode, Statement, Type, VariantDeclKind};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/recursive_copy_payload_enum/main.aero";
const EXAMPLE_MODULE: &str = "examples/recursive_copy_payload_enum/payloads.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 113;

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
            "aero-recursive-copy-payload-enum-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create recursive CopyData payload enum workspace");
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
            .is_some_and(|name| name.starts_with("aero-recursive-copy-payload-enum-"));
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
            "{label}: excluded recursive CopyData payload enum compiled:\n{llvm}"
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
struct Cell { value: int, flags: [bool; 2] }
struct Frame {
    cells: [Cell; 2],
    meta: ((int, bool), [float; 2]),
    empty: [bool; 0]
}

enum Payload {
    Idle,
    Flag(bool),
    Empty([bool; 0]),
    Flags([bool; 2]),
    Matrix([[int; 2]; 2]),
    Pair((int, bool)),
    Rows([(int, bool); 2]),
    CellValue(Cell),
    Cells([Cell; 2]),
    Wrapped((Cell, [bool; 2])),
    FrameValue(Frame)
}

fn make_cell(value: int) -> Cell {
    Cell { value: value, flags: [value > 0, value < 0] }
}

fn make_frame(value: int) -> Frame {
    Frame {
        cells: [make_cell(value), make_cell(value + 1)],
        meta: ((value + 2, value > 0), [1.5, 2.5]),
        empty: []
    }
}

fn forward(value: Payload) -> Payload { value }

fn produce() -> Payload { Payload::Flag(1 < 2) }

fn bool_score(value: bool, score: int) -> int {
    if value { return score; }
    0
}

fn score(value: Payload) -> int {
    match value {
        Payload::Idle => 1,
        Payload::Flag(flag) => bool_score(flag, 2),
        Payload::Empty(items) => 3,
        Payload::Flags(flags) => bool_score(flags[0], 4),
        Payload::Matrix(matrix) => matrix[1][0],
        Payload::Pair(pair) => pair.0,
        Payload::Rows(rows) => rows[1].0,
        Payload::CellValue(cell) => cell.value,
        Payload::Cells(cells) => cells[1].value,
        Payload::Wrapped(wrapped) => bool_score(wrapped.1[0], wrapped.0.value),
        Payload::FrameValue(frame) => frame.cells[1].value + (frame.meta.0).0
    }
}

fn main() -> int {
    let original: Payload = Payload::Pair((9, 1 < 2));
    let moved = forward(original);
    let empty: [bool; 0] = [];
    let total = score(Payload::Idle)
        + score(produce())
        + score(Payload::Empty(empty))
        + score(Payload::Flags([1 < 2, 1 > 2]))
        + score(Payload::Matrix([[5, 6], [7, 8]]))
        + score(moved)
        + score(Payload::Rows([(10, 1 < 2), (11, 1 > 2)]))
        + score(forward(Payload::CellValue(make_cell(12))))
        + score(Payload::Cells([make_cell(13), make_cell(14)]))
        + score(Payload::Wrapped((make_cell(15), [1 < 2, 1 > 2])))
        + score(Payload::FrameValue(make_frame(16)));
    if total == 113 { return 113; }
    1
}
"#
}

#[test]
fn recursive_copydata_payload_enum_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    match parsed_ast(source) {
        Err(error) => failures.push(format!(
            "recursive payload enum syntax was not retained: {error}"
        )),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::EnumDef { variants, .. })) = ast
                .iter()
                .find(|node| matches!(node, AstNode::Statement(Statement::EnumDef { .. })))
            else {
                failures.push(format!("parser omitted recursive payload enum: {ast:#?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            let expected = [
                ("Idle", None),
                ("Flag", Some("bool")),
                ("Empty", Some("array")),
                ("Flags", Some("array")),
                ("Matrix", Some("array")),
                ("Pair", Some("tuple")),
                ("Rows", Some("array")),
                ("CellValue", Some("Cell")),
                ("Cells", Some("array")),
                ("Wrapped", Some("tuple")),
                ("FrameValue", Some("Frame")),
            ];
            if variants.len() != expected.len() {
                failures.push(format!(
                    "parser changed recursive payload variant count: {variants:#?}"
                ));
            }
            for (variant, (name, shape)) in variants.iter().zip(expected) {
                let retained = match (&variant.kind, shape) {
                    (VariantDeclKind::Unit, None) => true,
                    (VariantDeclKind::Tuple(types), Some("array")) => {
                        matches!(types.as_slice(), [Type::Array(_, _)])
                    }
                    (VariantDeclKind::Tuple(types), Some("tuple")) => {
                        matches!(types.as_slice(), [Type::Tuple(_)])
                    }
                    (VariantDeclKind::Tuple(types), Some(expected)) => {
                        matches!(types.as_slice(), [Type::Named(actual)] if actual == expected)
                    }
                    _ => false,
                };
                if variant.name != name || !retained {
                    failures.push(format!(
                        "parser changed recursive payload declaration for {name}: {variant:#?}"
                    ));
                }
            }
        }
    }

    failures.extend(expect_success(
        "complete recursive CopyData enum payload class",
        source,
        &[
            "switch i32",
            "[2 x i1]",
            "[2 x [2 x double]]",
            "{ double, i1 }",
            "%aero.struct.Cell",
            "%aero.struct.Frame",
            "define i32 @score",
            "define",
        ],
    ));

    for (label, source, required) in [
        (
            "every immediate recursive constructor pair",
            "struct S { x: int } enum E { AA([[int; 1]; 2]), AT([(int, bool); 2]), AS([S; 2]), TA(([int; 2], bool)), TT(((int, bool), [float; 1])), TS((S, int)), SA(S) } fn main() -> int { let value = E::AS([S { x: 1 }, S { x: 2 }]); return match value { E::AA(x) => x[1][0], E::AT(x) => x[1].0, E::AS(x) => x[1].x, E::TA(x) => (x.0)[0], E::TT(x) => (x.0).0, E::TS(x) => (x.0).x, E::SA(x) => x.x }; }",
            vec![
                "[2 x [1 x double]]",
                "[2 x { double, i1 }]",
                "%aero.struct.S",
            ],
        ),
        (
            "aggregate payload internal transport",
            "struct Boxed { values: [(int, bool); 2] } enum E { Value(Boxed) } fn id(value: E) -> E { value } fn main() -> int { let value = id(E::Value(Boxed { values: [(3, 1 < 2), (4, 1 > 2)] })); return match value { E::Value(inner) => inner.values[0].0 }; }",
            vec!["define", "%aero.struct.Boxed", "switch i32"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("recursive payload checked IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedEnumVariant",
                "CheckedEnumPayload",
                "CheckedEnumDispatch",
                "CheckedEnumParameter",
                "Array {",
                "Tuple {",
                "Struct {",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked recursive enum IR missing {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in [
                "EnumConstruct",
                "EnumVariantData",
                "EnumDiscriminant",
                "inttoptr",
                "ptrtoint",
                "bitcast",
            ] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "recursive enum LLVM contains forbidden {forbidden}:\n{llvm}"
                    ));
                }
            }
        }
    }

    for (label, source) in [
        (
            "String leaf under recursive payload",
            "enum E { Value([(int, String); 1]) } fn main() { let value = E::Value([(1, \"x\")]); }",
        ),
        (
            "reference leaf under recursive payload",
            "enum E { Value((&int, bool)) } fn main() { let value = E::Value((1, 1 < 2)); }",
        ),
        (
            "enum leaf under recursive payload",
            "enum Inner { Unit } enum Outer { Value((Inner, int)) } fn main() { let value = Outer::Value((Inner::Unit, 1)); }",
        ),
        (
            "unit tuple payload",
            "enum E { Value(()) } fn main() { let value = E::Value(()); }",
        ),
        (
            "unary tuple payload",
            "enum E { Value((int,)) } fn main() { let value = E::Value((1,)); }",
        ),
        (
            "multi-field enum variant",
            "enum E { Value(int, bool) } fn main() { let value = E::Value((1, 1 < 2)); }",
        ),
        (
            "struct enum variant",
            "enum E { Value { x: int } } fn main() { let value = E::Value; }",
        ),
        (
            "generic enum",
            "enum E<T> { Value(T) } fn main() { let value = E::Value(1); }",
        ),
        (
            "cyclic struct payload",
            "struct Loop { next: Loop } enum E { Value(Loop) } fn consume(value: E) -> int { 0 } fn main() -> int { 0 }",
        ),
        (
            "wrong recursive payload type",
            "enum E { Value([int; 2]) } fn main() { let value = E::Value([1]); }",
        ),
        (
            "aggregate Match result remains excluded",
            "enum E { Value([int; 2]) } fn main() -> int { let result = match E::Value([1, 2]) { E::Value(items) => items }; return result[0]; }",
        ),
        (
            "enum stored in aggregate remains excluded",
            "enum E { Value([int; 2]) } struct S { value: E } fn consume(value: S) -> int { 0 } fn main() -> int { 0 }",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source) {
            failures.push(failure);
        }
    }

    let moved_source = "enum E { Value([int; 2]) } fn take(value: E) -> int { match value { E::Value(items) => items[0] } } fn main() -> int { let original = E::Value([1, 2]); let first = take(original); let second = take(original); first + second }";
    match compile_program(moved_source, CompilerOptions::default()) {
        Err(error) if error.contains("moved") => {}
        result => failures.push(format!(
            "whole aggregate-payload enum move was not enforced: {result:?}"
        )),
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { Value([int; 2]) } fn main() { let value = E::Value([1]); }",
    )
    .expect("write invalid recursive payload source");
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
            "invalid recursive payload CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for path in [&tracked_root, &tracked_module] {
        if !path.is_file() {
            failures.push(format!(
                "tracked recursive payload example is missing: {}",
                path.display()
            ));
        }
    }
    if tracked_root.is_file() && tracked_module.is_file() {
        let output = workspace.path("recursive-copy-payload-enum.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
        );
        if !build.status.success() || !output.is_file() {
            failures.push(format!(
                "tracked recursive payload enum failed checked build (artifact={}): {}",
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
                "Test recursive CopyData payload enum integration example",
                "examples/recursive_copy_payload_enum/main.aero",
                "opt-22 -passes=verify -disable-output ../../recursive_copy_payload_enum.ll",
                "llc-22 -verify-machineinstrs ../../recursive_copy_payload_enum.ll",
                "clang-22 -no-pie ../../recursive_copy_payload_enum.o -o ../../recursive_copy_payload_enum",
                "Expected exit code 113",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-063 recursive CopyData payload enum failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
