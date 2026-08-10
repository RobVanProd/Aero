use compiler::ast::{AstNode, Expression, Statement};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/projected_copydata_assignment/main.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-projected-copydata-assignment-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create projected assignment test workspace");
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
            .is_some_and(|name| name.starts_with("aero-projected-copydata-assignment-"));
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

fn parsed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn complete_source() -> &'static str {
    r#"
struct Cell { value: int, ready: bool }
struct Packet { pair: (Cell, [int; 3]), rows: [Cell; 2] }
struct Envelope { packet: Packet }
enum Pick { Left, Right }

fn make_cell(value: int) -> Cell { Cell { value: value, ready: value > 0 } }

fn main() -> int {
    let seed = Cell { value: 1, ready: 1 < 2 };
    let pair = (seed, [2, 3, 4]);
    let rows = [seed, seed];
    let packet = Packet { pair: pair, rows: rows };
    let mut envelope: Envelope = Envelope { packet: packet };

    envelope.packet.pair.0.value = 10;
    (envelope.packet.pair.1)[0] = 11;
    envelope.packet.rows[1] = make_cell(12);
    envelope.packet.rows[0].ready = 2 < 3;
    envelope.packet.rows[0].value = match Pick::Left {
        Pick::Left => 13,
        Pick::Right => 1
    };

    let mut tuple = ([1, 2], Cell { value: 3, ready: 1 == 2 });
    (tuple.0)[1] = 14;
    tuple.1 = make_cell(15);
    tuple.1.ready = 3 > 2;

    let mut matrix = [[1, 2], [3, 4]];
    matrix[1][0] = 21;

    if tuple.1.ready {
        envelope.packet.pair.0.value = 17;
    } else {
        envelope.packet.pair.0.value = 1;
    }
    let mut step = 0;
    while step < 1 {
        envelope.packet.pair.1[2] = 18;
        step = step + 1;
    }

    {
        let view = &envelope;
        let observed = (*view).packet.rows[1].value;
    }
    envelope.packet.rows[1].value = 19;

    envelope.packet.pair.0.value
        + envelope.packet.pair.1[0]
        + envelope.packet.rows[0].value
        + tuple.0[1]
        + tuple.1.value
        + matrix[1][0]
        + envelope.packet.pair.1[2]
        - envelope.packet.rows[1].value
}
"#
}

fn semantic_result(source: &str) -> Result<Vec<AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed_ast(source)?)
        .map(|(_, ast)| ast)
}

fn checked_result(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(parsed_ast(source)?)
        .map_err(|error| error.to_string())?;
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .map_err(|error| error.to_string())?;
    Ok((checked, llvm))
}

fn expect_rejection(label: &str, source: &str, expected: &str) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: unsupported projected assignment compiled:\n{llvm}"
        )),
        Err(error) if error.contains(expected) => None,
        Err(error) => Some(format!(
            "{label}: expected diagnostic containing {expected:?}, got {error:?}"
        )),
    }
}

#[test]
fn complete_projected_copydata_assignment_class_is_checked_and_executable() {
    let mut failures = Vec::new();

    match parsed_ast(complete_source()) {
        Err(error) => failures.push(format!(
            "parser rejected the complete target grammar: {error}"
        )),
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
                                    Statement::Assignment { target, .. } => Some(target),
                                    _ => None,
                                })
                                .collect::<Vec<_>>(),
                        )
                    }
                    _ => None,
                })
                .unwrap_or_default();
            let topologies = targets
                .iter()
                .map(|target| match target {
                    Expression::FieldAccess { .. } => "field",
                    Expression::IndexAccess { .. } => "index",
                    Expression::TupleIndex { .. } => "tuple",
                    other => panic!("unexpected projected target topology: {other:?}"),
                })
                .collect::<Vec<_>>();
            if !topologies.contains(&"field")
                || !topologies.contains(&"index")
                || !topologies.contains(&"tuple")
            {
                failures.push(format!(
                    "parser did not retain every terminal selector topology: {topologies:?}"
                ));
            }
        }
    }

    if let Err(error) = semantic_result(complete_source()) {
        failures.push(format!(
            "semantic route rejected the complete projected assignment class: {error}"
        ));
    }

    match checked_result(complete_source()) {
        Err(error) => failures.push(format!(
            "semantic-independent checked route rejected the class: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedStructFieldPtr",
                "CheckedTupleFieldPtr",
                "CheckedCopyStructArrayElementPtr",
                "CheckedOwnedPlaceAssignment",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!("checked IR omitted {marker}:\n{debug}"));
                }
            }
            for marker in [
                "getelementptr inbounds %aero.struct",
                "getelementptr inbounds {",
                "getelementptr inbounds [",
                "store double",
                "store i1",
            ] {
                if !llvm.contains(marker) {
                    failures.push(format!("LLVM omitted {marker:?}:\n{llvm}"));
                }
            }
            for forbidden in ["inttoptr", "ptrtoint", "bitcast"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "projected assignment LLVM used forbidden {forbidden}:\n{llvm}"
                    ));
                }
            }
        }
    }

    match compile_program(complete_source(), CompilerOptions::default()) {
        Err(error) => failures.push(format!("public compilation rejected the class: {error}")),
        Ok(llvm) if !llvm.contains("define i32 @main()") => {
            failures.push(format!("public LLVM omitted main:\n{llvm}"));
        }
        Ok(_) => {}
    }

    for (label, source, expected) in [
        (
            "immutable root",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; row.value = 2; row.value }",
            "mutable local owned binding",
        ),
        (
            "parameter root",
            "struct Row { value: int } fn bad(row: Row) -> int { row.value = 2; row.value } fn main() -> int { 0 }",
            "mutable local owned binding",
        ),
        (
            "borrowed root",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let view = &row; row.value = 2; (*view).value }",
            "while it is borrowed",
        ),
        (
            "unknown field",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row.missing = 2; row.value }",
            "has no field `missing`",
        ),
        (
            "tuple out of range",
            "fn main() -> int { let mut pair = (1, 2); pair.2 = 3; pair.0 }",
            "outside",
        ),
        (
            "array out of range",
            "fn main() -> int { let mut values = [1, 2]; values[2] = 3; values[0] }",
            "outside 0..2",
        ),
        (
            "zero array",
            "fn main() -> int { let mut values: [int; 0] = []; values[0] = 3; 0 }",
            "outside 0..0",
        ),
        (
            "dynamic index",
            "fn main() -> int { let mut values = [1, 2]; let index = 0; values[index] = 3; values[0] }",
            "compile-time integer literal",
        ),
        (
            "negative index",
            "fn main() -> int { let mut values = [1, 2]; values[-1] = 3; values[0] }",
            "compile-time integer literal",
        ),
        (
            "temporary root",
            "struct Row { value: int } fn main() -> int { (Row { value: 1 }).value = 2; 0 }",
            "direct local identifier root",
        ),
        (
            "call root",
            "struct Row { value: int } fn make() -> Row { Row { value: 1 } } fn main() -> int { make().value = 2; 0 }",
            "direct local identifier root",
        ),
        (
            "leaf mismatch",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; row.value = 2.5; row.value }",
            "type mismatch",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_ROOT);
    match compile_file(&example, CompilerOptions::default()) {
        Err(error) => failures.push(format!(
            "tracked direct-module specimen failed public compilation: {error}"
        )),
        Ok(llvm)
            if !llvm.contains("define i32 @main()")
                || !llvm.contains("define i32 @module_projection()") =>
        {
            failures.push(format!(
                "tracked direct-module specimen omitted linked functions:\n{llvm}"
            ));
        }
        Ok(_) => {}
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for projected assignment anchors");
    for anchor in [
        "Test projected CopyData assignment integration example",
        "cargo run -- check ../../examples/projected_copydata_assignment/main.aero",
        "cargo run -- run ../../examples/projected_copydata_assignment/main.aero",
        "cargo run -- build ../../examples/projected_copydata_assignment/main.aero -o ../../projected_copydata_assignment.ll",
        "opt-22 -passes=verify -disable-output ../../projected_copydata_assignment.ll",
        "llc-22 -verify-machineinstrs ../../projected_copydata_assignment.ll -o /dev/null",
        "llc-22 -filetype=obj ../../projected_copydata_assignment.ll -o ../../projected_copydata_assignment.o",
        "clang-22 -no-pie ../../projected_copydata_assignment.o -o ../../projected_copydata_assignment",
        "projected CopyData assignment example passed with exit code 90",
        "Test Windows projected CopyData assignment system specimen",
        "Windows projected CopyData assignment public run passed with exit code 90",
        "Windows projected CopyData assignment manual native execution passed with exit code 90",
    ] {
        let count = workflow.matches(anchor).count();
        if count != 1 {
            failures.push(format!(
                "workflow anchor {anchor:?} occurs {count} times instead of once"
            ));
        }
    }

    let workspace = TestWorkspace::new();
    let checked_output = workspace.path("projected.ll");
    let checked = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &example,
            Path::new("-o"),
            &checked_output,
        ],
    );
    if !checked.status.success() || !checked_output.is_file() {
        failures.push(format!(
            "tracked direct-module specimen failed checked CLI build:\n{}",
            output_text(&checked)
        ));
    }

    let invalid_source = workspace.path("invalid.aero");
    let invalid_output = workspace.path("invalid.ll");
    fs::write(
        &invalid_source,
        "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; row.value = 2; row.value }",
    )
    .expect("write invalid projected assignment source");
    let rejected = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid_source,
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
            "invalid projected assignment CLI hygiene failed (status={}, artifact={}):\n{}",
            rejected.status,
            invalid_output.exists(),
            rejected_text
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-090 projected CopyData assignment failures:\n{}",
        failures.join("\n\n")
    );
}
