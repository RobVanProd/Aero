use compiler::ast::{AstNode, Statement, VariantDeclKind};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/multi_field_enum/main.aero";
const EXAMPLE_MODULE: &str = "examples/multi_field_enum/packets.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 193;

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
            "aero-multi-field-enum-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create multi-field enum workspace");
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
            .is_some_and(|name| name.starts_with("aero-multi-field-enum-"));
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

fn parsed(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed(source: &str) -> Result<Vec<AstNode>, String> {
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

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Option<String> {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Some(format!(
            "{label}: excluded multi-field enum program compiled:\n{llvm}"
        )),
        Err(error)
            if error.starts_with("Parse error:")
                || expected.is_empty()
                || expected.iter().any(|part| error.contains(part)) =>
        {
            None
        }
        Err(error) => Some(format!(
            "{label}: diagnostic {error:?} omitted every expected fragment {expected:?}"
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
struct Cell { value: int, flags: [bool; 2] }
struct Frame { cell: Cell, pair: (int, bool), rows: [[int; 2]; 2] }

enum Packet {
    Idle,
    Single(int),
    Wrapped((int, bool)),
    Pair(int, bool),
    Ordered(int, int),
    Rich(Cell, [bool; 0], (int, bool), [[int; 2]; 2], Frame)
}

fn make_cell(value: int) -> Cell {
    Cell { value: value, flags: [value > 0, value < 0] }
}

fn make_frame(value: int) -> Frame {
    Frame {
        cell: make_cell(value),
        pair: (value + 1, value > 0),
        rows: [[value + 2, value + 3], [value + 4, value + 5]]
    }
}

fn bool_add(flag: bool, value: int) -> int {
    if flag { return value + 1; }
    value
}

fn score(value: Packet) -> int {
    match value {
        Packet::Rich(cell, empty, pair, matrix, frame) =>
            cell.value + empty.len() + bool_add(pair.1, pair.0)
                + matrix[1][0] + frame.cell.value,
        Packet::Ordered(left, right) => left * 10 + right,
        Packet::Pair(left, flag) => bool_add(flag, left),
        Packet::Wrapped(pair) => bool_add(pair.1, pair.0),
        Packet::Single(single) => single,
        Packet::Idle => 1
    }
}

fn forward(value: Packet) -> Packet { value }

fn choose(flag: bool) -> Packet {
    if flag { return Packet::Pair(20, 1 > 2); }
    Packet::Single(20)
}

fn next(value: &mut int) -> int {
    *value = *value + 1;
    *value
}

fn main() -> int {
    let empty: [bool; 0] = [];
    let rich = Packet::Rich(
        make_cell(10),
        empty,
        (20, 1 < 2),
        [[1, 2], [30, 4]],
        make_frame(40)
    );
    let mut total = score(Packet::Idle)
        + score(Packet::Single(2))
        + score(Packet::Pair(3, 1 < 2))
        + score(forward(rich));

    let mut changed = Packet::Idle;
    changed = Packet::Pair(10, 1 < 2);
    total = total + score(changed) + score(choose(1 < 2));

    let mut step = 0;
    while step < 1 {
        let fresh = Packet::Pair(12, 1 < 2);
        total = total + score(fresh);
        step = step + 1;
    }
    for item in [15] {
        let fresh = Packet::Pair(item, 1 > 2);
        total = total + score(fresh);
    }
    loop {
        let fresh = Packet::Pair(25, 1 < 2);
        total = total + score(fresh);
        break;
    }

    let mut order = 0;
    let ordered = Packet::Ordered(next(&mut order), next(&mut order));
    if score(ordered) != 12 { return 1; }
    if total == 193 { return 193; }
    1
}
"#
}

#[test]
fn positional_multi_field_enum_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let declaration = parsed(
        "struct S { x: int } enum E { Unit, One((int, bool)), Two(int, bool), Many(S, [bool; 0], (int, bool)) } fn main() {}",
    );
    match declaration {
        Err(error) => failures.push(format!(
            "parser lost the founding positional type-list declaration grammar: {error}"
        )),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::EnumDef { variants, .. })) = ast
                .iter()
                .find(|node| matches!(node, AstNode::Statement(Statement::EnumDef { .. })))
            else {
                failures.push(format!("parser omitted the enum declaration: {ast:#?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            let arities = variants
                .iter()
                .map(|variant| match &variant.kind {
                    VariantDeclKind::Unit => 0,
                    VariantDeclKind::Tuple(fields) => fields.len(),
                    VariantDeclKind::Struct(_) => usize::MAX,
                })
                .collect::<Vec<_>>();
            if arities != [0, 1, 2, 3] {
                failures.push(format!(
                    "parser changed declaration-ordered positional arities: {arities:?}"
                ));
            }
        }
    }

    let source = complete_source();
    match parsed(source) {
        Err(error) => failures.push(format!(
            "multi-field constructor/pattern syntax was not retained: {error}"
        )),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            for marker in ["Packet", "Rich", "Ordered", "left", "right", "matrix"] {
                if !debug.contains(marker) {
                    failures.push(format!("multi-field AST omitted {marker:?}:\n{debug}"));
                }
            }
        }
    }

    failures.extend(expect_success(
        "complete positional multi-field recursive CopyData enum class",
        source,
        &[
            "switch i32",
            "%aero.struct.Cell",
            "%aero.struct.Frame",
            "[0 x i1]",
            "{ double, i1 }",
            "define i32 @score",
        ],
    ));

    for (label, source, required) in [
        (
            "two scalar fields and unary tuple remain distinct",
            "enum E { Packed((int, bool)), Separate(int, bool) } fn score(value: E) -> int { match value { E::Packed(pair) => pair.0, E::Separate(number, flag) => number } } fn main() -> int { score(E::Packed((1, 1 < 2))) + score(E::Separate(2, 1 > 2)) }",
            vec!["switch i32", "{ double, i1 }"],
        ),
        (
            "all recursive CopyData field shapes",
            "struct S { x: int, flags: [bool; 2] } enum E { Value([[int; 1]; 2], [(int, bool); 2], [S; 2], ([int; 2], bool), ((int, bool), [float; 1]), (S, int), S) } fn main() -> int { let value = E::Value([[1], [2]], [(3, 1 < 2), (4, 1 > 2)], [S { x: 5, flags: [1 < 2, 1 > 2] }, S { x: 6, flags: [1 > 2, 1 < 2] }], ([7, 8], 1 < 2), ((9, 1 < 2), [10.0]), (S { x: 11, flags: [1 < 2, 1 < 2] }, 12), S { x: 13, flags: [1 > 2, 1 < 2] }); match value { E::Value(aa, at, arrays, ta, tt, ts, item) => aa[1][0] + at[0].0 + arrays[1].x + (ta.0)[0] + (tt.0).0 + (ts.0).x + item.x } }",
            vec!["switch i32", "%aero.struct.S", "[2 x [1 x double]]"],
        ),
        (
            "declaration and source arm order are independent",
            "enum E { Unit, Pair(int, bool), Triple(int, int, int) } fn score(value: E) -> int { match value { E::Triple(a, b, c) => a + b + c, E::Unit => 0, E::Pair(number, flag) => number } } fn main() -> int { score(E::Triple(1, 2, 3)) }",
            vec!["switch i32"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!("multi-field checked IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in [
                "CheckedEnumVariant",
                "CheckedEnumVariantFields",
                "CheckedEnumPayload",
                "CheckedEnumField",
                "CheckedEnumDispatch",
                "CheckedEnumParameter",
                "CheckedOwnedPlaceAssignment",
                "Tuple {",
                "Struct {",
                "Array {",
            ] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "multi-field checked IR omitted {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "multi-field LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "multi-field LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic multi-field compilation failed: {error}"
                )),
            }
        }
    }

    for (label, source, expected) in [
        (
            "empty positional declaration",
            "enum E { Empty() } fn main() { let value = E::Empty; }",
            vec!["not an admitted", "not admitted", "unsupported", "empty"],
        ),
        (
            "unit constructor parentheses",
            "enum E { Unit } fn main() { let value = E::Unit(); }",
            vec![
                "does not accept",
                "Expected expression",
                "payload",
                "empty positional",
            ],
        ),
        (
            "missing field",
            "enum E { Pair(int, bool) } fn main() { let value = E::Pair(1); }",
            vec!["not an admitted", "requires", "arity", "field"],
        ),
        (
            "excess field",
            "enum E { Pair(int, bool) } fn main() { let value = E::Pair(1, 1 < 2, 3); }",
            vec!["Expected ')'", "arity", "field"],
        ),
        (
            "wrong field order",
            "enum E { Pair(int, bool) } fn main() { let value = E::Pair(1 < 2, 1); }",
            vec!["type mismatch", "expected"],
        ),
        (
            "unary tuple is not two fields",
            "enum E { Packed((int, bool)) } fn main() { let value = E::Packed(1, 1 < 2); }",
            vec!["Expected ')'", "arity", "field"],
        ),
        (
            "two fields are not one tuple",
            "enum E { Pair(int, bool) } fn main() { let value = E::Pair((1, 1 < 2)); }",
            vec![
                "not an admitted",
                "requires",
                "arity",
                "field",
                "type mismatch",
            ],
        ),
        (
            "String field",
            "enum E { Pair(int, String) } fn main() { let value = E::Pair(1, \"x\"); }",
            vec!["not an admitted", "not admitted", "unsupported"],
        ),
        (
            "reference field",
            "enum E { Pair(int, &int) } fn main() { let value = E::Pair(1, 2); }",
            vec!["not an admitted", "not admitted", "unsupported"],
        ),
        (
            "enum field",
            "enum Inner { Unit } enum Outer { Pair(int, Inner) } fn main() { let value = Outer::Pair(1, Inner::Unit); }",
            vec!["not an admitted", "not admitted", "unsupported"],
        ),
        (
            "generic multi-field enum",
            "enum E<T> { Pair(T, int) } fn main() { let value = E::Pair(1, 2); }",
            vec!["not an admitted", "not admitted", "unsupported"],
        ),
        (
            "named-field variant",
            "enum E { Pair { left: int, right: bool } } fn main() { let value = E::Pair; }",
            vec!["not an admitted", "not admitted", "unsupported"],
        ),
        (
            "missing pattern field",
            "enum E { Pair(int, bool) } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(value) => value } }",
            vec!["requires", "binding", "field"],
        ),
        (
            "excess pattern field",
            "enum E { Pair(int, bool) } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(a, b, c) => a } }",
            vec!["Expected ')'", "binding", "field"],
        ),
        (
            "wildcard field pattern",
            "enum E { Pair(int, bool) } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(value, _) => value } }",
            vec!["identifier", "binding"],
        ),
        (
            "literal field pattern",
            "enum E { Pair(int, bool) } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(1, flag) => 1 } }",
            vec!["identifier", "binding"],
        ),
        (
            "nested field pattern",
            "enum E { Pair((int, bool), int) } fn main() -> int { match E::Pair((1, 1 < 2), 2) { E::Pair((number, flag), other) => number } }",
            vec!["identifier", "binding"],
        ),
        (
            "duplicate field binder",
            "enum E { Pair(int, int) } fn main() -> int { match E::Pair(1, 2) { E::Pair(value, value) => value } }",
            vec!["duplicate", "binding"],
        ),
        (
            "field binder shadows consumed scrutinee",
            "enum E { Pair(int, bool) } fn main() -> int { let value = E::Pair(1, 1 < 2); match value { E::Pair(value, flag) => value } }",
            vec!["shadows consumed", "binding"],
        ),
        (
            "field binder leaks from arm",
            "enum E { Pair(int, bool) } fn main() -> int { let result = match E::Pair(1, 1 < 2) { E::Pair(value, flag) => value }; value }",
            vec!["not found", "Undefined", "undefined", "undeclared"],
        ),
        (
            "duplicate variant arm",
            "enum E { Pair(int, bool), Unit } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(a, b) => a, E::Pair(c, d) => c } }",
            vec!["duplicate", "cover"],
        ),
        (
            "incomplete variant coverage",
            "enum E { Pair(int, bool), Unit } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(a, b) => a } }",
            vec!["cover every", "exactly once"],
        ),
        (
            "foreign variant arm",
            "enum E { Pair(int, bool) } enum F { Pair(int, bool) } fn main() -> int { match E::Pair(1, 1 < 2) { F::Pair(a, b) => a } }",
            vec!["expected `E`", "names `F`", "foreign"],
        ),
        (
            "whole enum double consumption",
            "enum E { Pair(int, bool) } fn take(value: E) -> int { match value { E::Pair(a, b) => a } } fn main() -> int { let value = E::Pair(1, 1 < 2); take(value) + take(value) }",
            vec!["moved", "consumed"],
        ),
        (
            "enum array storage",
            "enum E { Pair(int, bool) } fn main() { let values = [E::Pair(1, 1 < 2)]; }",
            vec!["not admitted", "array"],
        ),
        (
            "enum struct-field storage",
            "enum E { Pair(int, bool) } struct S { value: E } fn main() { let value = S { value: E::Pair(1, 1 < 2) }; }",
            vec!["not admitted", "Struct construction", "unsupported"],
        ),
        (
            "enum borrowing",
            "enum E { Pair(int, bool) } fn main() { let value = E::Pair(1, 1 < 2); let alias = &value; }",
            vec!["not admitted Copy-data", "reference"],
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, &expected) {
            failures.push(failure);
        }
    }

    let raw = IrGenerator::new()
        .generate_ir(parsed("fn main() -> int { 0 }").expect("raw compatibility sentinel parses"));
    let raw_debug = format!("{raw:#?}");
    for marker in [
        "CheckedEnumVariant",
        "CheckedEnumVariantFields",
        "CheckedEnumPayload",
        "CheckedEnumField",
        "CheckedEnumDispatch",
    ] {
        if raw_debug.contains(marker) {
            failures.push(format!(
                "legacy raw generation activated checked identity {marker}:\n{raw_debug}"
            ));
        }
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { Pair(int, bool) } fn main() { let value = E::Pair(1); }",
    )
    .expect("write invalid multi-field enum source");
    for command in ["check", "run"] {
        let output = run_cli(&workspace, &[Path::new(command), &invalid]);
        if output.status.success() {
            failures.push(format!(
                "invalid multi-field enum CLI {command} succeeded: {}",
                output_text(&output)
            ));
        }
    }
    let build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    if build.status.success() || invalid_artifact.exists() {
        failures.push(format!(
            "invalid multi-field enum CLI build did not fail without an artifact: {}",
            output_text(&build)
        ));
    }

    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for path in [&tracked_root, &tracked_module] {
        if !path.is_file() {
            failures.push(format!(
                "tracked multi-field enum example is missing: {}",
                path.display()
            ));
        }
    }
    if tracked_root.is_file() && tracked_module.is_file() {
        let output = workspace.path("multi-field-enum.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
        );
        if !build.status.success() || !output.is_file() {
            failures.push(format!(
                "tracked multi-field enum build failed (artifact={}): {}",
                output.is_file(),
                output_text(&build)
            ));
        }
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW)).expect("read stable workflow");
    for marker in [
        "Test positional multi-field enum integration example",
        "examples/multi_field_enum/main.aero",
        "opt-22 -passes=verify -disable-output ../../multi_field_enum.ll",
        "llc-22 -verify-machineinstrs ../../multi_field_enum.ll",
        "clang-22 -no-pie ../../multi_field_enum.o -o ../../multi_field_enum",
        "multi-field enum example passed with exit code 193",
    ] {
        if !workflow.contains(marker) {
            failures.push(format!("stable workflow is missing {marker:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-069 positional multi-field enum failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
