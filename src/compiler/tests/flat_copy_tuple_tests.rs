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

const EXAMPLE_ROOT: &str = "examples/flat_copy_tuples/main.aero";
const EXAMPLE_MODULE: &str = "examples/flat_copy_tuples/tuples.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 23;

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
            "aero-flat-copy-tuples-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create flat tuple workspace");
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
            .is_some_and(|name| name.starts_with("aero-flat-copy-tuples-"));
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
            "{label}: unsupported tuple source compiled:\n{llvm}"
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
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn tracked_root_source() -> &'static str {
    r#"mod tuples;

enum Mode { Off, On }
struct Packet { value: int, ready: bool }

fn read(value: &int) -> int { *value }
fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 7 } }

fn main() -> int {
    let base = 0;
    let original: (int, float, bool) = make_tuple(5);
    let copied = original;
    let transported = tuple_roundtrip(copied, 1);
    let packet = Packet { value: 3, ready: transported.2 };
    let rows = [packet, packet];
    let mut total = transported.0;
    bump(&mut total);
    if original.0 == 5 && copied.1 == 2.5 && rows[1].ready
        && "aero".len() == 4 && read(&base) == 0 {
        return total + packet.value + rows.len() + mode_score(Mode::On) + "aero".len();
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn make_tuple(seed: int) -> (int, float, bool) {
    (seed, 2.5, 1 < 2)
}

fn tuple_roundtrip(value: (int, float, bool), offset: int) -> (int, float, bool) {
    (value.0 + offset, value.1, value.2)
}

fn bump(value: &mut int) { *value = *value + 1; }
"#
}

#[test]
fn flat_copy_scalar_tuple_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
fn pair(value: (int, float, bool)) -> (int, float, bool) { value }
fn main() -> int { let value: (int, float, bool) = (1, 2.5, 1 < 2); pair(value).0 }
"#;
    match parsed_ast(parser_source) {
        Err(error) => failures.push(format!("parser tuple retention failed: {error}")),
        Ok(ast) => {
            let debug = format!("{ast:#?}");
            if debug.matches("Tuple(").count() < 3
                || !debug.contains("TupleLiteral")
                || !debug.contains("TupleIndex")
                || !debug.contains("index: 0")
            {
                failures.push(format!("parser lost tuple topology:\n{debug}"));
            }
        }
    }

    for (label, source, required) in [
        (
            "all scalar kinds, exact annotation, copy, repeated read, and every projection",
            "fn main() -> int { let original: (int, float, bool) = (4, 2.5, 1 < 2); let copied = original; if original.0 == 4 && copied.1 == 2.5 && original.2 { return copied.0 + original.0; } 1 }",
            vec![
                "alloca { double, double, i1 }",
                "getelementptr inbounds { double, double, i1 }",
            ],
        ),
        (
            "arbitrary heterogeneous order and arity",
            "fn main() -> int { let value = (1 < 2, 1, 2.0, 3, 4 < 5); if value.0 && value.4 && value.2 == 2.0 { return value.1 + value.3; } 1 }",
            vec!["{ i1, double, double, double, i1 }", "i32 4"],
        ),
        (
            "immediate literal projection",
            "fn main() -> int { (3, 4.0, 1 < 2).0 + (5, 6).1 }",
            vec![
                "getelementptr inbounds { double, double, i1 }",
                "getelementptr inbounds { double, double }",
            ],
        ),
        (
            "mixed scalar and tuple parameter product with tuple result and forwarding",
            "fn forward(value: (int, bool), amount: int, ratio: float) -> (float, bool, int) { later(ratio, value.1, value.0 + amount) } fn later(ratio: float, ready: bool, value: int) -> (float, bool, int) { (ratio, ready, value) } fn main() -> int { let value = forward((3, 1 < 2), 4, 2.5); if value.0 == 2.5 && value.1 { return value.2; } 1 }",
            vec![
                "define { double, i1, double } @forward({ double, i1 }",
                "call { double, i1, double } @later",
                "call { double, i1, double } @forward",
            ],
        ),
        (
            "explicit and tail tuple returns plus immediate call-result projection",
            "fn explicit(value: int) -> (int, bool) { return (value, value > 0); } fn tail(value: int) -> (bool, int) { (value > 0, value + 1) } fn main() -> int { if explicit(5).1 && tail(5).0 { return explicit(5).0 + tail(5).1; } 1 }",
            vec![
                "define { double, i1 } @explicit",
                "define { i1, double } @tail",
                "call { double, i1 } @explicit",
            ],
        ),
        (
            "branch loop and terminating recursion",
            "fn climb(value: (int, bool), remaining: int) -> (int, bool) { if remaining > 0 { return climb((value.0 + 1, value.1), remaining - 1); } value } fn main() -> int { let value = climb((1, 1 < 2), 2); let mut total = 0; while total < value.0 { total = total + 1; } if value.1 { return total; } 1 }",
            vec!["call { double, i1 } @climb", "while_start", "if_then"],
        ),
        (
            "literal elements evaluate once in source order",
            "fn next(value: &mut int) -> int { *value = *value + 1; *value } fn main() -> int { let mut value = 0; let ordered = (next(&mut value), next(&mut value), value); if ordered.0 == 1 && ordered.1 == 2 && ordered.2 == 2 { return 9; } 1 }",
            vec!["call i32 @next", "{ double, double, double }"],
        ),
        (
            "recursive nested tuple storage and projection",
            "fn main() -> int { let value = (1, (2, 3)); (value.1).0 }",
            vec!["{ double, { double, double } }"],
        ),
        (
            "recursive array element storage and tuple-array projection",
            "fn main() -> int { let value = (1, [2, 3]); let pairs = [(4, 5), (6, 7)]; value.1[1] + (pairs[1]).0 }",
            vec!["{ double, [2 x double] }", "[2 x { double, double }]"],
        ),
        (
            "recursive tuple-bearing mixed aggregate signature",
            "struct Packet { value: int } fn combine(value: (int, bool), packet: Packet, values: [int; 2]) -> Packet { Packet { value: value.0 + packet.value + values[1] } } fn main() -> int { let result = combine((1, 1 < 2), Packet { value: 3 }, [4, 5]); result.value }",
            vec![
                "define %aero.struct.Packet @combine({ double, i1 }",
                "%aero.struct.Packet %aero.arg.packet",
                "[2 x double] %aero.arg.values",
            ],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    let checked_source = "fn pair(value: (int, bool), ratio: float) -> (float, bool, int) { (ratio, value.1, value.0) } fn main() -> int { let first = (4, 1 < 2); let copied = first; let result = pair(copied, 2.5); if result.0 == 2.5 && result.1 { return result.2; } 1 }";
    match checked_ir_and_llvm(checked_source) {
        Err(error) => failures.push(format!("checked tuple case failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for identity in ["Tuple", "CheckedTupleAlloca", "CheckedTupleFieldPtr"] {
                if !debug.contains(identity) {
                    failures.push(format!("checked IR missing {identity}:\n{debug}"));
                }
            }
            if debug.matches("CheckedTupleAlloca").count() < 4
                || debug.matches("CheckedTupleFieldPtr").count() < 9
            {
                failures.push(format!("checked tuple identities are incomplete:\n{debug}"));
            }
            for anchor in [
                "define { double, i1, double } @pair({ double, i1 }",
                "store { double, i1 }",
                "load { double, i1 }",
                "call { double, i1, double } @pair",
            ] {
                if !llvm.contains(anchor) {
                    failures.push(format!("tuple LLVM missing {anchor:?}:\n{llvm}"));
                }
            }
            if llvm.contains("inttoptr") || llvm.contains("ptrtoint") {
                failures.push(format!(
                    "tuple lowering used pointer/integer conversion:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "unit tuple remains excluded",
            "fn main() -> int { let value = (); 0 }",
            "Copy tuples require at least two recursively admitted CopyData elements",
        ),
        (
            "String element remains excluded",
            "fn main() -> int { let value = (1, \"aero\"); value.0 }",
            "Copy tuples require at least two recursively admitted CopyData elements",
        ),
        (
            "tuple array legacy len method remains excluded",
            "fn main() -> int { let values = [(1, 2), (3, 4)]; values.len() }",
            "method calls other than exact zero-argument array/Vec .iter() are not admitted",
        ),
        (
            "out of range projection",
            "fn main() -> int { let value = (1, 2); value.2 }",
            "tuple projection index 2 is outside 0..2",
        ),
        (
            "projection from scalar",
            "fn main() -> int { let value = 1; value.0 }",
            "tuple projection requires a recursively admitted Copy tuple",
        ),
        (
            "tuple binding annotation mismatch",
            "fn main() -> int { let value: (int, bool) = (1, 2.0); value.0 }",
            "tuple binding annotation mismatch: expected (int, bool), actual (int, float)",
        ),
        (
            "tuple print remains excluded",
            "fn main() -> int { println!(\"{}\", (1, 2)); 0 }",
            "Argument 1 of type `(int, int)` is not printable",
        ),
        (
            "tuple process-entry result remains excluded",
            "fn main() -> (int, bool) { (1, 1 < 2) }",
            "process entry `main` cannot use tuple parameters or returns",
        ),
        (
            "tuple process-entry parameter remains excluded",
            "fn main(value: (int, bool)) -> int { value.0 }",
            "process entry `main` cannot use tuple parameters or returns",
        ),
        (
            "generic tuple transport remains excluded",
            "fn bad<T>(value: (int, bool)) -> (int, bool) { value } fn main() -> int { 0 }",
            "generic function IR is not admitted",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    for source in [
        "fn main() -> int { let grouped: int = (7); grouped }",
        "fn main() -> int { let values = [1, 2]; values[1] }",
        "struct Pair { left: int, right: bool } fn main() -> int { let value = Pair { left: 3, right: 1 < 2 }; value.left }",
    ] {
        if let Err(error) = compile_program(source, CompilerOptions::default()) {
            failures.push(format!("adjacent compatibility regressed: {error}"));
        }
    }

    match analyzed_ast(checked_source) {
        Err(error) => failures.push(format!("raw containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:#?}");
            if debug.contains("CheckedTupleAlloca") || debug.contains("CheckedTupleFieldPtr") {
                failures.push(format!(
                    "deprecated raw generation activated checked tuple identities:\n{debug}"
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
                    "tracked tuple root drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!(
                    "tracked tuple module drifted at {}",
                    module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked flat tuple example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for flat tuple integration anchor");
    for anchor in [
        "Test flat Copy-scalar tuple integration example",
        "examples/flat_copy_tuples/main.aero",
        "cargo run -- build ../../examples/flat_copy_tuples/main.aero -o ../../flat_copy_tuples.ll",
        "if [ $exit_code -ne 23 ]; then",
        "flat Copy-scalar tuple example passed with exit code 23",
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
        let output_path = workspace.path("flat_copy_tuples.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked flat tuple example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("module-and-invalid-hygiene");
    let root_source = workspace.path("main.aero");
    let module_source = workspace.path("values.aero");
    let valid_output = workspace.path("valid.ll");
    fs::write(
        &root_source,
        "mod values; fn main() -> int { let value = module_pair(8); value.0 }",
    )
    .expect("write tuple module root");
    fs::write(
        &module_source,
        "fn module_pair(value: int) -> (int, bool) { (value, value > 0) }",
    )
    .expect("write tuple direct module");
    let valid = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &root_source,
            Path::new("-o"),
            &valid_output,
        ],
    );
    if !valid.status.success() || !valid_output.is_file() {
        failures.push(format!(
            "valid direct-module tuple build failed:\n{}",
            output_text(&valid)
        ));
    }

    let invalid = workspace.path("invalid.aero");
    let invalid_output = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn main() -> int { let value = (1, \"aero\"); value.0 }",
    )
    .expect("write invalid flat tuple source");
    let invalid_build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_output,
        ],
    );
    let diagnostics = output_text(&invalid_build);
    if invalid_build.status.success()
        || invalid_output.exists()
        || !diagnostics
            .contains("Copy tuples require at least two recursively admitted CopyData elements")
    {
        failures.push(format!(
            "invalid tuple CLI hygiene failed (status={}, artifact={}):\n{}",
            invalid_build.status,
            invalid_output.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-058 flat Copy-scalar tuple failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
