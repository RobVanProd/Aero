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

const EXAMPLE_ROOT: &str = "examples/copy_place_references/main.aero";
const EXAMPLE_MODULE: &str = "examples/copy_place_references/borrows.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 37;

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
            "aero-copy-place-references-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create Copy-place reference workspace");
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
            .is_some_and(|name| name.starts_with("aero-copy-place-references-"));
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
            "{label}: unsupported Copy-place reference source compiled:\n{llvm}"
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

fn complete_local_source() -> &'static str {
    r#"
struct Leaf { x: int, y: int }
struct Frame { leaf: Leaf, rows: [Leaf; 2], bias: int }
struct Envelope { frame: Frame }

fn read_copy_places(
    point: &Leaf,
    envelope: &Envelope,
    tuple: &(int, float, bool),
    values: &[int; 3],
    rows: &[Leaf; 2],
    scalar: &int,
    bias: int
) -> int {
    let point_copy: Leaf = *point;
    let envelope_copy = *envelope;
    let tuple_copy: (int, float, bool) = *tuple;
    let values_copy: [int; 3] = *values;
    let rows_copy = *rows;
    if tuple_copy.2 && tuple_copy.1 > 2.0 && values_copy.len() == 3
        && rows_copy[0].x == *scalar {
        return point_copy.x + envelope_copy.frame.rows[1].y
            + tuple_copy.0 + values_copy[2] + bias;
    }
    0
}

fn copy_leaf(value: &Leaf) -> Leaf { *value }
fn copy_tuple(value: &(int, float, bool)) -> (int, float, bool) { *value }
fn copy_values(value: &[int; 3]) -> [int; 3] { *value }
fn observe(value: &Frame) { let copy = *value; let seen = copy.bias; }
fn forward(value: &Leaf) -> int { copy_leaf(value).x }
fn recurse(value: &Leaf, depth: int) -> int {
    if depth == 0 { return (*value).x; }
    recurse(value, depth - 1)
}

fn main() -> int {
    let leaf = Leaf { x: 5, y: 7 };
    let rows = [leaf, leaf];
    let frame = Frame { leaf: leaf, rows: rows, bias: 1 };
    let envelope = Envelope { frame: frame };
    let tuple = (3, 2.5, 1 < 2);
    let values = [4, 6, 8];
    let scalar = 5;
    let leaf_ref: &Leaf = &leaf;
    let leaf_alias = leaf_ref;
    let first = &leaf;
    let second = &leaf;
    let envelope_ref: &Envelope = &envelope;
    let tuple_ref: &(int, float, bool) = &tuple;
    let values_ref: &[int; 3] = &values;
    let rows_ref: &[Leaf; 2] = &rows;
    let scalar_ref = &scalar;
    observe(&frame);
    let total = read_copy_places(
        leaf_alias, envelope_ref, tuple_ref, values_ref, rows_ref, scalar_ref, 1
    );
    let copied_tuple = copy_tuple(tuple_ref);
    let copied_values = copy_values(values_ref);
    if total == 24 && copied_tuple.0 == 3 && copied_values[1] == 6
        && forward(first) == 5 && recurse(second, 2) == 5
        && leaf.x == 5 && (*leaf_ref).y == 7 {
        return total;
    }
    1
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod borrows;

enum Mode { Off, On }
struct Point { x: int, y: int }
struct Frame { point: Point, rows: [Point; 2], bias: int }
struct Envelope { frame: Frame }

fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 7 } }

fn main() -> int {
    let point = Point { x: 5, y: 7 };
    let rows = [point, point];
    let frame = Frame { point: point, rows: rows, bias: 1 };
    let envelope = Envelope { frame: frame };
    let tuple = (3, 2.5, 1 < 2);
    let values = [4, 6, 8];
    let scalar = 5;
    let point_ref: &Point = &point;
    let point_alias = point_ref;
    let envelope_ref: &Envelope = &envelope;
    let tuple_ref: &(int, float, bool) = &tuple;
    let values_ref: &[int; 3] = &values;
    let rows_ref: &[Point; 2] = &rows;
    let scalar_ref = &scalar;
    let borrowed = module_score(
        point_alias, envelope_ref, tuple_ref, values_ref, rows_ref, scalar_ref, 1
    );
    let copied = copy_point(point_ref);
    if copied.x == 5 && point.x == 5 && (*point_ref).y == 7 {
        return borrowed + mode_score(Mode::On) + "aero".len() + (*envelope_ref).frame.rows.len();
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn copy_point(value: &Point) -> Point { *value }

fn module_score(
    point: &Point,
    envelope: &Envelope,
    tuple: &(int, float, bool),
    values: &[int; 3],
    rows: &[Point; 2],
    scalar: &int,
    bias: int
) -> int {
    let point_copy = *point;
    let envelope_copy = *envelope;
    let tuple_copy = *tuple;
    let values_copy = *values;
    let rows_copy = *rows;
    if tuple_copy.2 && tuple_copy.1 > 2.0 && values_copy.len() == 3
        && rows_copy[0].x == *scalar {
        return point_copy.x + envelope_copy.frame.rows[1].y
            + tuple_copy.0 + values_copy[2] + bias;
    }
    0
}
"#
}

#[test]
fn immutable_copy_place_reference_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = "struct Row { value: int } fn read(row: &Row, values: &[int; 3], pair: &(int, bool)) -> Row { *row } fn main() -> int { 0 }";
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!("parser retention failed: {error}")),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::Function { parameters, .. })) = ast.get(1)
            else {
                failures.push(format!(
                    "parser omitted Copy-place reference signature: {ast:?}"
                ));
                return assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
            };
            let retained = matches!(
                &parameters[0].param_type,
                Type::Reference(inner, false)
                    if matches!(inner.as_ref(), Type::Named(name) if name == "Row")
            ) && matches!(
                &parameters[1].param_type,
                Type::Reference(inner, false)
                    if matches!(inner.as_ref(), Type::Array(element, 3)
                        if matches!(element.as_ref(), Type::Named(name) if name == "int"))
            ) && matches!(
                &parameters[2].param_type,
                Type::Reference(inner, false)
                    if matches!(inner.as_ref(), Type::Tuple(elements) if elements.len() == 2)
            );
            if !retained {
                failures.push(format!(
                    "parser changed aggregate-reference annotation topology: {parameters:?}"
                ));
            }
        }
    }

    failures.extend(expect_success(
        "complete local and transported Copy-place reference product",
        complete_local_source(),
        &[
            "%aero.struct.Leaf = type",
            "%aero.struct.Frame = type",
            "%aero.struct.Envelope = type",
            "@read_copy_places(%aero.struct.Leaf*",
            "[3 x double]*",
            "[2 x %aero.struct.Leaf]*",
            "{ double, double, i1 }*",
            "call i32 @read_copy_places(%aero.struct.Leaf* %ptr",
            "load %aero.struct.Leaf, %aero.struct.Leaf*",
            "getelementptr inbounds %aero.struct.Envelope",
        ],
    ));

    for (label, source, required) in [
        (
            "direct struct borrow and owned result",
            "struct Row { value: int } fn copy(value: &Row) -> Row { *value } fn main() -> int { let row = Row { value: 9 }; let copied = copy(&row); copied.value }",
            vec![
                "define %aero.struct.Row @copy(%aero.struct.Row*",
                "ret %aero.struct.Row",
            ],
        ),
        (
            "numeric array reference and owned result",
            "fn copy(value: &[int; 3]) -> [int; 3] { *value } fn main() -> int { let values = [2, 4, 6]; let copied = copy(&values); copied[2] }",
            vec![
                "define [3 x double] @copy([3 x double]*",
                "ret [3 x double]",
            ],
        ),
        (
            "Copy-struct array reference and projection",
            "struct Row { value: int } fn read(value: &[Row; 2]) -> int { let copy = *value; copy[1].value } fn main() -> int { let row = Row { value: 11 }; let rows = [row, row]; read(&rows) }",
            vec![
                "define i32 @read([2 x %aero.struct.Row]*",
                "load [2 x %aero.struct.Row]",
            ],
        ),
        (
            "flat tuple reference and owned result",
            "fn copy(value: &(int, float, bool)) -> (int, float, bool) { *value } fn main() -> int { let value = (12, 1.5, 1 < 2); let copied = copy(&value); if copied.2 { return copied.0; } 0 }",
            vec!["define { double, double, i1 } @copy({ double, double, i1 }*"],
        ),
        (
            "recursive nested tuple reference",
            "fn read(value: &((int, int), bool)) -> int { let copy = *value; if copy.1 { return (copy.0).1; } 0 } fn main() -> int { let value = ((3, 7), 1 < 2); read(&value) }",
            vec!["define i32 @read({ { double, double }, i1 }*"],
        ),
        (
            "recursive Bool array reference",
            "fn read(value: &[bool; 2]) -> int { let copy = *value; if copy[1] { return 1; } 0 } fn main() -> int { let value = [1 > 2, 1 < 2]; read(&value) }",
            vec!["define i32 @read([2 x i1]*"],
        ),
        (
            "recursive tuple array reference",
            "fn read(value: &[(int, int); 2]) -> int { let copy = *value; (copy[1]).0 } fn main() -> int { let value = [(1, 2), (3, 4)]; read(&value) }",
            vec!["define i32 @read([2 x { double, double }]*"],
        ),
        (
            "recursive tuple-field struct reference",
            "struct PairBox { pair: (int, int) } fn read(value: &PairBox) -> int { let copy = *value; (copy.pair).1 } fn main() -> int { let value = PairBox { pair: (3, 9) }; read(&value) }",
            vec!["define i32 @read(%aero.struct.PairBox*"],
        ),
        (
            "arbitrary reference count order and forwarding",
            "struct Row { value: int } fn inner(left: &Row, bias: int, right: &Row) -> int { (*left).value + (*right).value + bias } fn outer(value: &Row) -> int { let alias = value; inner(alias, 1, value) } fn main() -> int { let row = Row { value: 3 }; let first = &row; let second = &row; outer(first) + (*second).value }",
            vec!["@inner(%aero.struct.Row*", "@outer(%aero.struct.Row*"],
        ),
        (
            "branch loop and terminating recursion",
            "struct Row { value: int } fn walk(value: &Row, depth: int) -> int { if depth == 0 { return (*value).value; } walk(value, depth - 1) } fn main() -> int { let row = Row { value: 4 }; let reference = &row; let mut total = 0; let mut depth = 0; while depth < 2 { total = total + walk(reference, depth); depth = depth + 1; } total }",
            vec!["call i32 @walk(%aero.struct.Row*", "br i1"],
        ),
        (
            "Void aggregate-reference consumer",
            "struct Row { value: int } fn observe(value: &Row) { let copy = *value; let seen = copy.value; } fn main() -> int { let row = Row { value: 5 }; observe(&row); row.value }",
            vec![
                "define void @observe(%aero.struct.Row*",
                "call void @observe(%aero.struct.Row*",
            ],
        ),
        (
            "explicit generic CopyData struct reference",
            "struct Box<T> { value: T } fn read(value: &Box<int>) -> int { let copy = *value; copy.value } fn main() -> int { let value: Box<int> = Box { value: 9 }; read(&value) }",
            vec!["define i32 @read(%\"aero.struct.Box<int>\"*"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(complete_local_source()) {
        Err(error) => failures.push(format!(
            "checked Copy-place reference IR/LLVM failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            if debug.matches("CheckedImmutableBorrow").count() < 9 {
                failures.push(format!(
                    "checked IR retained too few Copy-place borrow identities:\n{debug}"
                ));
            }
            if debug.matches("CheckedImmutableReferenceParameter").count() < 12 {
                failures.push(format!(
                    "checked IR retained too few aggregate-reference parameter identities:\n{debug}"
                ));
            }
            for marker in [
                "ImmutableReference { pointee: Struct",
                "ImmutableReference { pointee: Array",
                "ImmutableReference { pointee: Tuple",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked IR omitted logical reference marker {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["inttoptr", "ptrtoint", "bitcast"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "aggregate reference LLVM used forbidden {forbidden}:\n{llvm}"
                    ));
                }
            }
        }
    }

    let rejected = [
        (
            "String pointee",
            "fn bad(value: &String) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "free enum pointee dereference",
            "enum Mode { Off, On } fn bad(value: &Mode) -> Mode { *value } fn main() -> int { 0 }",
            "not admitted Copy-data",
        ),
        (
            "nested reference pointee",
            "fn main() -> int { let value = 1; let first = &value; let second = &first; 0 }",
            "admitted Copy-data",
        ),
        (
            "unit tuple pointee",
            "fn bad(value: &()) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "unary tuple pointee",
            "fn bad(value: &(int,)) -> int { 0 } fn main() -> int { 0 }",
            "Expected type",
        ),
        (
            "String tuple pointee",
            "fn bad(value: &(int, String)) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "empty struct pointee",
            "struct Empty {} fn bad(value: &Empty) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "String-field struct pointee",
            "struct Bad { text: String } fn bad(value: &Bad) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "cyclic struct pointee",
            "struct Node { next: Node } fn bad(value: &Node) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "borrowed literal",
            "fn main() -> int { let reference = &1; *reference }",
            "identifier place",
        ),
        (
            "borrowed computation",
            "fn main() -> int { let value = 1; let reference = &(value + 1); *reference }",
            "identifier place",
        ),
        (
            "borrowed field",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let reference = &row.value; *reference }",
            "identifier place",
        ),
        (
            "borrowed index",
            "fn main() -> int { let values = [1, 2]; let reference = &values[0]; *reference }",
            "identifier place",
        ),
        (
            "borrowed dereference",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let first = &row; let second = &*first; 0 }",
            "identifier place",
        ),
        (
            "borrowed tuple projection",
            "fn main() -> int { let pair = (1, 2); let reference = &pair.0; *reference }",
            "identifier place",
        ),
        (
            "reference result escape",
            "struct Row { value: int } fn bad(value: &Row) -> &Row { value } fn main() -> int { 0 }",
            "reference results require lifetime semantics",
        ),
        (
            "process entry reference parameter",
            "struct Row { value: int } fn main(value: &Row) -> int { (*value).value }",
            "process entry cannot use reference parameters",
        ),
        (
            "generic reference function",
            "fn bad<T>(value: &T) -> int { 0 } fn main() -> int { 0 }",
            "generic reference transport functions",
        ),
        (
            "enum mixed reference signature",
            "enum Mode { Off, On } struct Row { value: int } fn bad(row: &Row, mode: Mode) -> int { (*row).value } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "exact aggregate reference annotation mismatch",
            "struct Left { value: int } struct Right { value: int } fn main() -> int { let value = Left { value: 1 }; let reference: &Right = &value; 0 }",
            "type annotation mismatch",
        ),
    ];
    for (label, source, expected) in rejected {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    match analyzed_ast(
        "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let reference = &row; (*reference).value }",
    ) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedImmutableBorrow")
                || debug.contains("CheckedImmutableReferenceParameter")
            {
                failures.push(format!(
                    "deprecated raw generation activated checked aggregate references:\n{debug}"
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
        .expect("read Rust workflow for Copy-place reference integration anchors");
    let lane_start = workflow
        .find("    - name: Test immutable Copy-place reference integration example")
        .unwrap_or(workflow.len());
    let lane_tail = &workflow[lane_start..];
    let lane_end = lane_tail
        .find("\n    - name: Run tests")
        .unwrap_or(lane_tail.len());
    let lane = &lane_tail[..lane_end];
    for anchor in [
        "Test immutable Copy-place reference integration example",
        "cargo run -- build ../../examples/copy_place_references/main.aero -o ../../copy_place_references.ll",
        "opt-22 -passes=verify -disable-output ../../copy_place_references.ll",
        "llc-22 -verify-machineinstrs ../../copy_place_references.ll -o /dev/null",
        "llc-22 -filetype=obj ../../copy_place_references.ll -o ../../copy_place_references.o",
        "clang-22 ../../copy_place_references.o -o ../../copy_place_references",
        "if [ $exit_code -ne 37 ]; then",
        "immutable Copy-place reference example passed with exit code 37",
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
    let output_path = workspace.path("copy-place-references.ll");
    let build = run_cli(
        &workspace,
        &[Path::new("build"), &example, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&build);
    if !build.status.success() || !output_path.is_file() {
        failures.push(format!(
            "tracked Copy-place reference example failed checked CLI build:\n{diagnostics}"
        ));
    }

    let invalid = workspace.path("invalid.aero");
    let invalid_output = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let reference = &row.value; *reference }",
    )
    .expect("write invalid Copy-place reference source");
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
            "invalid Copy-place reference CLI hygiene failed (status={}, artifact={}):\n{}",
            rejected.status,
            invalid_output.exists(),
            rejected_text
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-059 immutable Copy-place reference failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
