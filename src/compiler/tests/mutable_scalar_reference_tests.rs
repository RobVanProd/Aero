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

const EXAMPLE_ROOT: &str = "examples/mutable_scalar_references/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_scalar_references/mutation.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 239;

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
            "aero-mutable-scalar-reference-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create mutable-reference test workspace");
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
            .is_some_and(|name| name.starts_with("aero-mutable-scalar-reference-"));
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
            "{label}: unsupported mutable-reference form compiled:\n{llvm}"
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

fn positive_source() -> &'static str {
    r#"
fn once(value: int) -> int { value + 1 }
fn mutate(start: int) -> int {
    let mut value: int = start;
    let alias: &mut int = &mut value;
    *alias = once(*alias);
    *alias = *alias + 2;
    let mut index = 0;
    while index < 3 {
        *alias = *alias + index;
        index = index + 1;
    }
    for item in [1, 2] {
        *alias = *alias + item;
    }
    if *alias > 0 { *alias = *alias + 4; } else { *alias = 0; }
    *alias
}
fn mutate_float(start: float) -> int {
    let mut ratio = start;
    let alias: &mut float = &mut ratio;
    *alias = *alias + 0.5;
    if *alias > 2.0 { return 9; }
    0
}
fn mutate_bool(input: bool) -> int {
    let mut ready: bool = 1 == 2;
    let alias: &mut bool = &mut ready;
    *alias = input;
    if *alias { return 8; }
    0
}
fn released() -> int {
    let mut value = 10;
    {
        let alias = &mut value;
        *alias = 12;
    }
    value = value + 1;
    value
}
fn main() -> int { mutate(1) + mutate_float(2.0) + mutate_bool(2 > 1) + released() }
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod mutation;

enum Signal { Idle, Count(int), Ready(bool) }
enum Mode { Off, On }
struct Packet { value: int, ready: bool }

fn read(value: &int) -> int { *value }
fn signal_score(value: Signal) -> int {
    match value {
        Signal::Idle => 1,
        Signal::Count(inner) => inner,
        Signal::Ready(inner) => 3
    }
}
fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 6 } }

fn main() -> int {
    let base = 20;
    let packet = Packet { value: 4, ready: 1 == 1 };
    let rows = [packet, packet];
    let mutated = mutation_score(100, 4, 2.0, 1 == 1);
    let signal = signal_score(Signal::Count(30));
    let mode = mode_score(Mode::On);
    if packet.ready && mutated == 173 && "aero".len() == 4 {
        return mutated + read(&base) + signal + mode + rows.len() + packet.value + "aero".len();
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn mutation_score(start: int, limit: int, ratio_start: float, input: bool) -> int {
    let mut total = start;
    {
        let alias: &mut int = &mut total;
        let mut index = 0;
        while index < limit {
            *alias = *alias + index;
            index = index + 1;
        }
        if input { *alias = *alias + 20; } else { *alias = 0; }
    }
    total = total;

    let mut ratio = ratio_start;
    let ratio_alias: &mut float = &mut ratio;
    *ratio_alias = *ratio_alias + 0.5;
    let mut ratio_score = 0;
    if *ratio_alias > 2.0 { ratio_score = 30; }

    let mut ready = 1 == 2;
    let ready_alias: &mut bool = &mut ready;
    *ready_alias = input;
    let mut ready_score = 0;
    if *ready_alias { ready_score = 5; }

    let mut released = 10;
    {
        let released_alias = &mut released;
        *released_alias = 12;
    }
    total + ratio_score + ready_score + released
}
"#
}

#[test]
fn local_mutable_scalar_reference_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
fn main() -> int {
    let mut value = 1;
    let alias: &mut int = &mut value;
    *alias = *alias + 1;
    *alias
}
"#;
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!(
            "parser mutable-reference retention failed: {error}"
        )),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            if debug.matches("mutable: true").count() < 2
                || debug.matches("Deref").count() < 2
                || debug.matches("Assignment").count() != 1
            {
                failures.push(format!(
                    "parser changed mutable borrow/annotation/dereference assignment topology:\n{debug}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "complete local mutable-reference composition",
            positive_source(),
            vec![
                "define i32 @mutate(i32 %aero.arg.start)",
                "call i32 @once(i32",
                "getelementptr inbounds double, double* %ptr",
                "getelementptr inbounds i1, i1* %ptr",
                "store double",
                "store i1",
                "while_start",
            ],
        ),
        (
            "inferred Int mutable alias",
            "fn main() -> int { let mut value = 1; let alias = &mut value; *alias = 7; *alias }",
            vec![
                "getelementptr inbounds double",
                "store double 0x401C000000000000",
            ],
        ),
        (
            "i32 annotation mutable alias",
            "fn main() -> int { let mut value: i32 = 1; let alias: &mut i32 = &mut value; *alias = 8; *alias }",
            vec!["getelementptr inbounds double", "store double"],
        ),
        (
            "f64 annotation mutable alias",
            "fn main() -> int { let mut value: f64 = 1.5; let alias: &mut f64 = &mut value; *alias = 2.5; if *alias == 2.5 { return 9; } 0 }",
            vec!["getelementptr inbounds double", "store double", "fcmp"],
        ),
        (
            "Bool mutable alias",
            "fn main() -> int { let mut ready: bool = 1 == 2; let alias: &mut bool = &mut ready; *alias = 2 > 1; if *alias { return 8; } 0 }",
            vec!["getelementptr inbounds i1", "store i1", "load i1"],
        ),
        (
            "lexical mutable-borrow release",
            "fn main() -> int { let mut value = 1; { let alias = &mut value; *alias = 5; } value = value + 2; value }",
            vec!["store double", "load double"],
        ),
        (
            "nearest shadowed mutable owner",
            "fn main() -> int { let mut value = 1; { let mut value = 4; let alias = &mut value; *alias = 7; if *alias != 7 { return 1; } } let outer = &mut value; *outer = 9; *outer }",
            vec![
                "store double 0x401C000000000000",
                "store double 0x4022000000000000",
            ],
        ),
        (
            "branch-selected mutable dereference write",
            "fn main() -> int { let mut value = 1; let alias = &mut value; if *alias == 1 { *alias = 12; } else { *alias = 2; } *alias }",
            vec!["if_then", "if_else", "store double 0x4028000000000000"],
        ),
        (
            "while-carried mutable dereference write",
            "fn main() -> int { let mut value = 0; let alias = &mut value; while *alias < 4 { *alias = *alias + 1; } *alias }",
            vec!["while_start", "while_body", "store double"],
        ),
        (
            "compiler-bounded for mutable dereference write",
            "fn main() -> int { let mut total = 0; let alias = &mut total; for item in [1, 2, 3] { *alias = *alias + item; } *alias }",
            vec!["store double", "load double"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked mutable-reference IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            if debug.matches("CheckedMutableBorrow").count() < 5 {
                failures.push(format!(
                    "checked IR lost mutable borrow identities:\n{debug}"
                ));
            }
            if debug.matches("CheckedMutableDereferenceAssignment").count() < 9
                || !debug.contains("pointee: Int")
                || !debug.contains("pointee: Float")
                || !debug.contains("pointee: Bool")
            {
                failures.push(format!(
                    "checked IR lost mutable dereference-write metadata:\n{debug}"
                ));
            }
            if llvm.matches("call i32 @once(").count() != 1 {
                failures.push(format!(
                    "mutable-reference RHS call was not emitted exactly once:\n{llvm}"
                ));
            }
            if llvm.contains("inttoptr") || llvm.contains("ptrtoint") {
                failures.push(format!(
                    "mutable-reference lowering escaped through pointer/integer conversion:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "immutable source",
            "fn main() -> int { let value = 1; let alias = &mut value; *alias }",
            "mutable scalar borrow source `value` must be declared mutable",
        ),
        (
            "unknown source",
            "fn main() -> int { let alias = &mut missing; 0 }",
            "undeclared variable `missing`",
        ),
        (
            "uninitialized source",
            "fn main() -> int { let mut value: int; let alias = &mut value; *alias }",
            "uninitialized variable `value`",
        ),
        (
            "second mutable borrow",
            "fn main() -> int { let mut value = 1; let first = &mut value; let second = &mut value; *first + *second }",
            "already borrowed as mutable",
        ),
        (
            "mutable after immutable borrow",
            "fn main() -> int { let mut value = 1; let first = &value; let second = &mut value; *first + *second }",
            "also borrowed as immutable",
        ),
        (
            "immutable after mutable borrow",
            "fn main() -> int { let mut value = 1; let first = &mut value; let second = &value; *first + *second }",
            "also borrowed as mutable",
        ),
        (
            "owner read while mutably borrowed",
            "fn main() -> int { let mut value = 1; let alias = &mut value; let seen = value; *alias = 2; seen }",
            "cannot read `value` while it is mutably borrowed",
        ),
        (
            "owner write while mutably borrowed",
            "fn main() -> int { let mut value = 1; let alias = &mut value; value = 2; *alias }",
            "cannot assign to `value` while it is borrowed",
        ),
        (
            "mutable alias relocation",
            "fn main() -> int { let mut value = 1; let first = &mut value; let second = first; *second = 2; *second }",
            "mutable reference aliases cannot be copied or relocated by CORE-055",
        ),
        (
            "mutable alias reassignment",
            "fn main() -> int { let mut left = 1; let mut right = 2; let alias = &mut left; alias = &mut right; *alias }",
            "supports only Int, Float, or Bool",
        ),
        (
            "wrong mutable annotation",
            "fn main() -> int { let mut value = 1; let alias: &mut float = &mut value; 0 }",
            "type annotation mismatch",
        ),
        (
            "immutable annotation for mutable borrow",
            "fn main() -> int { let mut value = 1; let alias: &int = &mut value; 0 }",
            "type annotation mismatch",
        ),
        (
            "mutable annotation for immutable borrow",
            "fn main() -> int { let mut value = 1; let alias: &mut int = &value; 0 }",
            "type annotation mismatch",
        ),
        (
            "borrowed literal",
            "fn main() -> int { let alias = &mut 1; 0 }",
            "local mutable scalar borrow requires an identifier place",
        ),
        (
            "borrowed computation",
            "fn main() -> int { let mut value = 1; let alias = &mut (value + 1); 0 }",
            "local mutable scalar borrow requires an identifier place",
        ),
        (
            "borrowed dereference",
            "fn main() -> int { let mut value = 1; let first = &mut value; let second = &mut *first; 0 }",
            "local mutable scalar borrow requires an identifier place",
        ),
        (
            "borrowed field",
            "struct Row { value: int } fn main() -> int { let row = Row { value: 1 }; let alias = &mut row.value; 0 }",
            "local mutable scalar borrow requires an identifier place",
        ),
        (
            "borrowed index",
            "fn main() -> int { let values = [1, 2]; let alias = &mut values[0]; 0 }",
            "local mutable scalar borrow requires an identifier place",
        ),
        (
            "String pointee",
            "fn main() -> int { let mut value = \"a\"; let alias = &mut value; 0 }",
            "local mutable references support only Int, Float, or Bool pointees",
        ),
        (
            "array pointee",
            "fn main() -> int { let mut value = [1, 2]; let alias = &mut value; 0 }",
            "local mutable references support only Int, Float, or Bool pointees",
        ),
        (
            "struct pointee",
            "struct Row { value: int } fn main() -> int { let mut row = Row { value: 1 }; let alias = &mut row; 0 }",
            "local mutable references support only Int, Float, or Bool pointees",
        ),
        (
            "nested reference pointee",
            "fn main() -> int { let mut value = 1; let first = &mut value; let mut second = first; let third = &mut second; 0 }",
            "mutable reference aliases cannot be copied or relocated by CORE-055",
        ),
        (
            "write through immutable reference",
            "fn main() -> int { let value = 1; let alias = &value; *alias = 2; *alias }",
            "assignment through an immutable reference is not supported",
        ),
        (
            "wrong Int write RHS",
            "fn main() -> int { let mut value = 1; let alias = &mut value; *alias = 2.0; *alias }",
            "mutable reference assignment type mismatch: expected int, actual float",
        ),
        (
            "wrong Bool write RHS",
            "fn main() -> int { let mut ready = 1 == 1; let alias = &mut ready; *alias = 1; 0 }",
            "mutable reference assignment type mismatch: expected bool, actual int",
        ),
        (
            "non-identifier mutable reference operand",
            "fn main() -> int { let mut value = 1; *(&mut value) = 2; value }",
            "mutable reference assignment requires a local reference identifier",
        ),
        (
            "mutable reference result remains contained",
            "fn bad() -> &mut int { let mut value = 1; &mut value } fn main() -> int { 0 }",
            "reference results require lifetime semantics and are not supported by CORE-053",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    if let Err(error) = compile_program(
        "fn write(value: &mut int) -> int { *value = 2; *value } fn main() -> int { 0 }",
        CompilerOptions::default(),
    ) {
        failures.push(format!(
            "CORE-056 mutable reference parameter definition regressed: {error}"
        ));
    }

    for (label, source) in [
        (
            "compound mutable dereference assignment remains contained",
            "fn main() -> int { let mut value = 1; let alias = &mut value; *alias += 2; *alias }",
        ),
        (
            "chained mutable dereference assignment remains contained",
            "fn main() -> int { let mut left = 1; let mut right = 2; let a = &mut left; let b = &mut right; *a = *b = 3; *a }",
        ),
        (
            "mutable dereference assignment value remains contained",
            "fn main() -> int { let mut value = 1; let alias = &mut value; let result = (*alias = 2); result }",
        ),
    ] {
        if compile_program(source, CompilerOptions::default()).is_ok() {
            failures.push(format!("{label}: unsupported assignment syntax compiled"));
        }
    }

    match analyzed_ast(
        "fn main() -> int { let mut value = 1; let alias = &mut value; *alias = 2; *alias }",
    ) {
        Err(error) if error.contains("mutable references are not supported") => {}
        Err(error) => failures.push(format!(
            "raw-containment setup failed unexpectedly: {error}"
        )),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:#?}");
            if debug.contains("CheckedMutableBorrow")
                || debug.contains("CheckedMutableDereferenceAssignment")
            {
                failures.push(format!(
                    "deprecated raw generation activated checked mutable-reference lowering:\n{debug}"
                ));
            }
        }
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_ROOT);
    let module = root.join(EXAMPLE_MODULE);
    match (fs::read_to_string(&example), fs::read_to_string(&module)) {
        (Ok(actual), Ok(module_actual)) => {
            if actual != tracked_root_source() {
                failures.push(format!(
                    "tracked root mutable-reference example drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!(
                    "tracked mutable-reference module drifted at {}",
                    module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked mutable-reference example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for mutable-reference integration anchor");
    for anchor in [
        "Test local mutable scalar-reference integration example",
        "examples/mutable_scalar_references/main.aero",
        "cargo run -- build ../../examples/mutable_scalar_references/main.aero -o ../../mutable_scalar_references.ll",
        "if [ $exit_code -ne 239 ]; then",
        "local mutable scalar-reference example passed with exit code 239",
    ] {
        let count = workflow.matches(anchor).count();
        if count != 1 {
            failures.push(format!(
                "Rust workflow anchor {anchor:?} occurs {count} times instead of once"
            ));
        }
    }

    if example.is_file() && module.is_file() {
        let workspace = TestWorkspace::new("tracked-example");
        let output_path = workspace.path("mutable_scalar_references.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked mutable-reference example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = workspace.path("invalid.aero");
    let output_path = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn main() -> int { let value = 1; let alias = &mut value; *alias }",
    )
    .expect("write invalid mutable-reference source");
    let output = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&output);
    if output.status.success()
        || output_path.exists()
        || !diagnostics.contains("mutable scalar borrow source `value` must be declared mutable")
    {
        failures.push(format!(
            "invalid mutable-reference CLI hygiene failed (status={}, artifact={}):\n{}",
            output.status,
            output_path.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-055 local mutable scalar-reference failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
