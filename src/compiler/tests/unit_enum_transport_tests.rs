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

const EXAMPLE_ROOT: &str = "examples/unit_enum_transport/main.aero";
const EXAMPLE_MODULE: &str = "examples/unit_enum_transport/phases.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 173;

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
            "aero-unit-enum-transport-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create unit-enum transport workspace");
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
            .is_some_and(|name| name.starts_with("aero-unit-enum-transport-"));
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

fn analyzed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    SemanticAnalyzer::new().analyze(ast).map(|(_, ast)| ast)
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
            "{label}: unsupported unit-enum transport compiled:\n{llvm}"
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
enum Solo { Only }
enum Phase { Cold, Warm, Hot }
enum Switch { Off, On }

struct Packet { score: int, ready: bool }

fn later(value: Phase) -> Phase { value }
fn forward(value: Phase) -> Phase { later(value) }
fn make_phase() -> Phase { Phase::Warm }
fn explicit_phase(flag: bool) -> Phase {
    if flag { return Phase::Hot; }
    return Phase::Cold;
}
fn make_solo() -> Solo { Solo::Only }

fn rank(value: Phase, bias: int, packet: Packet, values: [int; 2]) -> int {
    let base = match value {
        Phase::Cold => 3,
        Phase::Warm => 7,
        Phase::Hot => 11
    };
    base + bias + packet.score + values[1]
}

fn ready(value: Phase) -> bool {
    match value {
        Phase::Cold => 1 > 2,
        Phase::Warm => 2 > 1,
        Phase::Hot => 3 > 2
    }
}

fn ratio(value: Phase) -> float {
    match value {
        Phase::Cold => 0.5,
        Phase::Warm => 1.5,
        Phase::Hot => 2.5
    }
}

fn dual(phase: Phase, switch: Switch) -> int {
    let first = match phase {
        Phase::Cold => 1,
        Phase::Warm => 2,
        Phase::Hot => 3
    };
    let second = match switch {
        Switch::Off => 4,
        Switch::On => 8
    };
    first + second
}

fn sequence(value: Phase) -> [int; 2] {
    let base = match value {
        Phase::Cold => 20,
        Phase::Warm => 30,
        Phase::Hot => 40
    };
    [base, base + 1]
}

fn consume(value: Phase) {
    let ignored = match value {
        Phase::Cold => 1,
        Phase::Warm => 2,
        Phase::Hot => 3
    };
}

fn main() -> int {
    let made = make_phase();
    let moved: Phase = made;
    let forwarded = forward(moved);
    let packet = Packet { score: 5, ready: 2 > 1 };
    let score = rank(forwarded, 2, packet, [4, 6]);

    let produced = explicit_phase(2 > 1);
    let direct = match forward(produced) {
        Phase::Cold => 1,
        Phase::Warm => 2,
        Phase::Hot => 9
    };
    let nested = rank(forward(Phase::Cold), 1, packet, [2, 3]);
    let truth = ready(Phase::Warm);
    let amount = ratio(Phase::Hot);
    let both = dual(Phase::Warm, Switch::On);
    let values = sequence(Phase::Warm);
    let solo = match make_solo() { Solo::Only => 1 };
    consume(Phase::Cold);

    if packet.ready && truth && amount > 2.0 && direct == 9 && nested == 14
        && both == 10 && values[1] == 31 && solo == 1 {
        return score + 153;
    }
    return 1;
}
"#
}

#[test]
fn owned_unit_enum_transport_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
enum Phase { Cold, Warm, Hot }
fn forward(value: Phase, count: int) -> Phase { value }
fn main() -> int { return 0; }
"#;
    match try_tokenize_with_locations(parser_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("parser retention failed: {error}")),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::Function {
                name,
                parameters,
                return_type,
                type_params,
                ..
            })) = ast.get(1)
            else {
                failures.push(format!("parser omitted enum-transport function: {ast:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            if name != "forward"
                || !type_params.is_empty()
                || parameters.len() != 2
                || !matches!(&parameters[0].param_type, Type::Named(name) if name == "Phase")
                || !matches!(return_type, Some(Type::Named(name)) if name == "Phase")
                || !matches!(
                    parameters.get(1).map(|parameter| &parameter.param_type),
                    Some(Type::Named(name)) if name == "int"
                )
            {
                failures.push(format!("parser changed enum signature topology: {ast:?}"));
            }
        }
    }

    failures.extend(expect_success(
        "all owned enum transport origins and mixed checked signatures",
        positive_source(),
        &[
            "define i32 @forward(i32 %aero.arg.value)",
            "call i32 @forward(i32",
            "ret i32",
            "switch i32",
        ],
    ));

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked enum transport IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            if debug.matches("CheckedEnumParameter").count() < 9 {
                failures.push(format!("checked IR lost enum parameter binders:\n{debug}"));
            }
            if debug.matches("CheckedFunctionDef").count() < 11 {
                failures.push(format!("checked IR lost enum-bearing signatures:\n{debug}"));
            }
            for schema in [
                "EnumSchema { name: \"Solo\", variants: [EnumVariantSchema { name: \"Only\", payload: None }] }",
                "EnumSchema { name: \"Phase\", variants: [EnumVariantSchema { name: \"Cold\", payload: None }, EnumVariantSchema { name: \"Warm\", payload: None }, EnumVariantSchema { name: \"Hot\", payload: None }] }",
                "EnumSchema { name: \"Switch\", variants: [EnumVariantSchema { name: \"Off\", payload: None }, EnumVariantSchema { name: \"On\", payload: None }] }",
            ] {
                if !debug.contains(schema) {
                    failures.push(format!("checked metadata missing {schema:?}:\n{debug}"));
                }
            }
            let llvm_lines = llvm.lines().collect::<Vec<_>>();
            if llvm_lines.windows(3).any(|window| {
                window[1].contains("= call i32 @forward(i32")
                    && (window[0].contains("fptosi")
                        || window[0].contains("sitofp")
                        || window[2].contains("fptosi")
                        || window[2].contains("sitofp"))
            }) {
                failures.push(format!(
                    "LLVM numerically converted an enum transport value:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "unknown enum parameter",
            "fn take(value: Missing) -> int { 1 } fn main() -> int { 0 }",
            "function parameter `value` is not an admitted scalar type",
        ),
        (
            "process entry parameter",
            "enum Phase { Cold } fn main(value: Phase) -> int { 0 }",
            "process entry cannot transport enums",
        ),
        (
            "process entry result",
            "enum Phase { Cold } fn main() -> Phase { Phase::Cold }",
            "process entry cannot transport enums",
        ),
        (
            "generic enum transport function",
            "enum Phase { Cold } fn take<T>(value: Phase) -> Phase { value } fn main() -> int { 0 }",
            "generic enum transport functions are not admitted",
        ),
        (
            "enum array parameter",
            "enum Phase { Cold } fn take(values: [Phase; 2]) -> int { 1 } fn main() -> int { 0 }",
            "enum transport function `take` parameter `values` is not an admitted by-value type",
        ),
        (
            "enum reference parameter",
            "enum Phase { Cold } fn take(value: &Phase) -> int { 1 } fn main() -> int { 0 }",
            "immutable reference parameter pointee is not admitted Copy-data",
        ),
        (
            "String mixed parameter",
            "enum Phase { Cold } fn take(value: Phase, text: String) -> int { 1 } fn main() -> int { 0 }",
            "enum transport function `take` parameter `text` is not an admitted by-value type",
        ),
        (
            "wrong enum argument",
            "enum Phase { Cold } enum Other { Cold } fn take(value: Phase) -> int { 1 } fn main() -> int { take(Other::Cold) }",
            "Function `take` parameter `value` type mismatch: expected Phase, actual Other",
        ),
        (
            "wrong enum return",
            "enum Phase { Cold } enum Other { Cold } fn make() -> Phase { Other::Cold } fn main() -> int { 0 }",
            "Function `make` return type mismatch: expected Phase, actual Other",
        ),
        (
            "enum call arity",
            "enum Phase { Cold } fn take(value: Phase) -> int { 1 } fn main() -> int { take() }",
            "Function `take` arity mismatch: expected 1, actual 0",
        ),
        (
            "use after argument move",
            "enum Phase { Cold } fn take(value: Phase) -> int { 1 } fn main() -> int { let phase = Phase::Cold; let score = take(phase); let reused = phase; score }",
            "Use of moved value `phase`",
        ),
        (
            "discarded call moves argument",
            "enum Phase { Cold } fn consume(value: Phase) {} fn main() -> int { let phase = Phase::Cold; consume(phase); let reused = phase; 0 }",
            "Use of moved value `phase`",
        ),
        (
            "duplicate nested consumption",
            "enum Phase { Cold } fn take(value: Phase) -> int { 1 } fn pair(a: int, b: int) -> int { a + b } fn main() -> int { let phase = Phase::Cold; pair(take(phase), take(phase)) }",
            "enum `phase` is consumed more than once in one expression",
        ),
        (
            "call scrutinee arm reuse",
            "enum Phase { Cold } fn forward(value: Phase) -> Phase { value } fn main() -> int { let phase = Phase::Cold; match forward(phase) { Phase::Cold => match phase { Phase::Cold => 1 } } }",
            "enum match arm reuses consumed scrutinee `phase`",
        ),
        (
            "closure transport context fails closed before enum transport",
            "enum Phase { Cold } fn main() -> int { let f = |value: Phase| value; 0 }",
            "closure expressions are parsed but unsupported in executable code",
        ),
        (
            "nested function transport context",
            "enum Phase { Cold } fn outer() -> int { fn inner(value: Phase) -> int { match value { Phase::Cold => 1 } } 0 } fn main() -> int { outer() }",
            "Match expressions are not supported",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    let raw_source = "enum Phase { Cold } fn forward(value: Phase) -> Phase { value } fn main() -> int { let phase = forward(Phase::Cold); match phase { Phase::Cold => 1 } }";
    match try_tokenize_with_locations(raw_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("raw containment parse failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedEnumParameter")
                || debug.contains("CheckedFunctionDef")
                || debug.contains("CheckedEnumVariant")
                || debug.contains("CheckedEnumDispatch")
            {
                failures.push(format!(
                    "deprecated raw path activated checked enum transport: {debug}"
                ));
            }
        }
    }

    let workspace = TestWorkspace::new("cli");
    let valid = workspace.path("valid.aero");
    let invalid = workspace.path("invalid.aero");
    let valid_artifact = workspace.path("valid.ll");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &valid,
        "enum Phase { Cold, Warm } fn forward(value: Phase) -> Phase { value } fn main() -> int { match forward(Phase::Warm) { Phase::Cold => 1, Phase::Warm => 2 } }",
    )
    .expect("write valid enum transport source");
    fs::write(
        &invalid,
        "enum Phase { Cold } fn take(value: Phase) -> int { 1 } fn main() -> int { let phase = Phase::Cold; let score = take(phase); let reused = phase; score }",
    )
    .expect("write invalid enum transport source");

    let valid_check = run_cli(&workspace, &[Path::new("check"), &valid]);
    if !valid_check.status.success() {
        failures.push(format!(
            "CLI check rejected valid enum transport: {}",
            output_text(&valid_check)
        ));
    }
    let valid_build = run_cli(
        &workspace,
        &[Path::new("build"), &valid, Path::new("-o"), &valid_artifact],
    );
    if !valid_build.status.success() || !valid_artifact.exists() {
        failures.push(format!(
            "CLI build failed to publish valid enum transport LLVM: {}",
            output_text(&valid_build)
        ));
    }
    let invalid_build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    if invalid_build.status.success()
        || invalid_artifact.exists()
        || !output_text(&invalid_build).contains("Use of moved value `phase`")
    {
        failures.push(format!(
            "CLI invalid ownership did not fail closed without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let root = repository_root();
    for relative in [EXAMPLE_ROOT, EXAMPLE_MODULE] {
        let path = root.join(relative);
        if !path.is_file() {
            failures.push(format!(
                "tracked integration file missing: {}",
                path.display()
            ));
        }
    }
    let workflow_path = root.join(WORKFLOW);
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not read stable workflow {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test owned unit-enum transport integration example",
                "examples/unit_enum_transport/main.aero",
                "opt-22 -passes=verify -disable-output ../../unit_enum_transport.ll",
                "llc-22 -verify-machineinstrs ../../unit_enum_transport.ll",
                "clang-22 ../../unit_enum_transport.o -o ../../unit_enum_transport",
                "Expected exit code 173",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }
    if EXPECTED_EXIT != 173 {
        failures.push("unit-enum transport native exit contract drifted".to_string());
    }

    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
