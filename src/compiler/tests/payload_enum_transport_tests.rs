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

const EXAMPLE_ROOT: &str = "examples/payload_enum_transport/main.aero";
const EXAMPLE_MODULE: &str = "examples/payload_enum_transport/signals.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 197;

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
            "aero-payload-enum-transport-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create payload-enum transport workspace");
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
            .is_some_and(|name| name.starts_with("aero-payload-enum-transport-"));
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
            "{label}: unsupported payload-enum transport compiled:\n{llvm}"
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
enum Signal { Idle, Count(int), Ratio(float), Ready(bool) }
enum Token { Value(int) }
enum Switch { Off, On }

struct Packet { score: int, ready: bool }

fn identity(value: Signal) -> Signal { value }
fn later(value: Signal) -> Signal { identity(value) }
fn forward(value: Signal) -> Signal { later(value) }
fn make_count(value: int) -> Signal { Signal::Count(value) }
fn make_ratio(value: float) -> Signal { Signal::Ratio(value) }
fn make_ready(value: bool) -> Signal { Signal::Ready(value) }
fn choose(flag: bool) -> Signal {
    if flag { return Signal::Count(8); }
    return Signal::Idle;
}

fn score(value: Signal, bias: int, packet: Packet, values: [int; 2]) -> int {
    let base = match value {
        Signal::Idle => 1,
        Signal::Count(inner) => inner,
        Signal::Ratio(inner) => 20,
        Signal::Ready(inner) => 30
    };
    base + bias + packet.score + values[1]
}

fn truth(value: Signal) -> bool {
    match value {
        Signal::Idle => 1 > 2,
        Signal::Count(inner) => inner > 0,
        Signal::Ratio(inner) => inner > 0.0,
        Signal::Ready(inner) => inner
    }
}

fn amount(value: Signal) -> float {
    match value {
        Signal::Idle => 0.5,
        Signal::Count(inner) => 1.5,
        Signal::Ratio(inner) => inner,
        Signal::Ready(inner) => 2.5
    }
}

fn dual(signal: Signal, token: Token, switch: Switch) -> int {
    let first = match signal {
        Signal::Idle => 1,
        Signal::Count(inner) => inner,
        Signal::Ratio(inner) => 3,
        Signal::Ready(inner) => 4
    };
    let second = match token { Token::Value(inner) => inner };
    let third = match switch { Switch::Off => 5, Switch::On => 9 };
    first + second + third
}

fn consume(value: Signal) {
    let ignored = match value {
        Signal::Idle => 0,
        Signal::Count(inner) => inner,
        Signal::Ratio(inner) => 2,
        Signal::Ready(inner) => 3
    };
}

fn main() -> int {
    let made = make_count(11);
    let moved: Signal = made;
    let forwarded = forward(moved);
    let packet = Packet { score: 5, ready: 2 > 1 };
    let total = score(forwarded, 2, packet, [4, 6]);

    let direct = match forward(choose(2 > 1)) {
        Signal::Idle => 1,
        Signal::Count(inner) => inner,
        Signal::Ratio(inner) => 3,
        Signal::Ready(inner) => 4
    };
    let nested = score(forward(Signal::Count(3)), 1, packet, [2, 3]);
    let yes = truth(make_ready(2 > 1));
    let ratio = amount(make_ratio(2.75));
    let combined = dual(Signal::Count(7), Token::Value(6), Switch::On);
    consume(Signal::Idle);

    if packet.ready && yes && ratio > 2.7 && direct == 8 && nested == 12
        && combined == 22 {
        return total;
    }
    return 1;
}
"#
}

#[test]
fn owned_payload_enum_transport_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
enum Signal { Idle, Count(int), Ratio(float), Ready(bool) }
fn forward(value: Signal, count: int) -> Signal { value }
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
                failures.push(format!(
                    "parser omitted payload-enum transport function: {ast:?}"
                ));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            if name != "forward"
                || !type_params.is_empty()
                || parameters.len() != 2
                || !matches!(&parameters[0].param_type, Type::Named(name) if name == "Signal")
                || !matches!(return_type, Some(Type::Named(name)) if name == "Signal")
                || !matches!(
                    parameters.get(1).map(|parameter| &parameter.param_type),
                    Some(Type::Named(name)) if name == "int"
                )
            {
                failures.push(format!(
                    "parser changed payload-enum signature topology: {ast:?}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "complete mixed-schema transport composition",
            positive_source(),
            vec![
                "define { i32, double, i1 } @forward({ i32, double, i1 } %aero.arg.value)",
                "call { i32, double, i1 } @forward({ i32, double, i1 }",
                "ret { i32, double, i1 }",
                "switch i32",
            ],
        ),
        (
            "Int payload parameter and scalar consumer",
            "enum Value { Int(int) } fn read(value: Value) -> int { match value { Value::Int(inner) => inner } } fn main() -> int { read(Value::Int(7)) }",
            vec![
                "define i32 @read({ i32, double, i1 }",
                "extractvalue { i32, double, i1 }",
            ],
        ),
        (
            "Float payload producer and result binding",
            "enum Value { Float(float) } fn make() -> Value { Value::Float(2.5) } fn main() -> int { let value = make(); match value { Value::Float(inner) => 4 } }",
            vec![
                "define { i32, double, i1 } @make()",
                "call { i32, double, i1 } @make()",
            ],
        ),
        (
            "Bool payload identity and direct-result Match",
            "enum Value { Bool(bool) } fn same(value: Value) -> Value { value } fn main() -> int { match same(Value::Bool(2 > 1)) { Value::Bool(inner) => 5 } }",
            vec![
                "define { i32, double, i1 } @same({ i32, double, i1 }",
                "ret { i32, double, i1 }",
            ],
        ),
        (
            "Void consumer",
            "enum Value { Int(int) } fn consume(value: Value) { let seen = match value { Value::Int(inner) => inner }; } fn main() -> int { consume(Value::Int(3)); 6 }",
            vec![
                "define void @consume({ i32, double, i1 }",
                "call void @consume({ i32, double, i1 }",
            ],
        ),
        (
            "multiple enum names and arguments",
            "enum Left { Value(int) } enum Right { Flag(bool) } fn join(left: Left, right: Right, bias: int) -> int { let a = match left { Left::Value(inner) => inner }; let b = match right { Right::Flag(inner) => 2 }; a + b + bias } fn main() -> int { join(Left::Value(3), Right::Flag(2 > 1), 4) }",
            vec![
                "@join({ i32, double, i1 }",
                "{ i32, double, i1 } %aero.arg.right",
            ],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!(
            "checked payload-enum transport IR/LLVM failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            if debug.matches("CheckedEnumParameter").count() < 10 {
                failures.push(format!(
                    "checked IR lost generalized enum parameter binders:\n{debug}"
                ));
            }
            if debug.matches("CheckedFunctionDef").count() < 12 {
                failures.push(format!(
                    "checked IR lost payload-enum-bearing signatures:\n{debug}"
                ));
            }
            for schema in [
                "EnumSchema { name: \"Signal\", variants: [EnumVariantSchema { name: \"Idle\", payload: None }, EnumVariantSchema { name: \"Count\", payload: Some(Int) }, EnumVariantSchema { name: \"Ratio\", payload: Some(Float) }, EnumVariantSchema { name: \"Ready\", payload: Some(Bool) }] }",
                "EnumSchema { name: \"Token\", variants: [EnumVariantSchema { name: \"Value\", payload: Some(Int) }] }",
                "EnumSchema { name: \"Switch\", variants: [EnumVariantSchema { name: \"Off\", payload: None }, EnumVariantSchema { name: \"On\", payload: None }] }",
            ] {
                if !debug.contains(schema) {
                    failures.push(format!("checked metadata missing {schema:?}:\n{debug}"));
                }
            }
            if llvm.contains("fptosi { i32, double, i1 }")
                || llvm.contains("sitofp { i32, double, i1 }")
                || llvm.contains("bitcast { i32, double, i1 }")
            {
                failures.push(format!(
                    "LLVM numerically converted a payload-enum value:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "empty tuple definition in signature",
            "enum Value { Empty() } fn take(value: Value) -> int { 1 } fn main() -> int { 0 }",
            "enum `Value` is not an admitted non-generic unit-or-positional-CopyData enum",
        ),
        (
            "multi-field constructor arity before transport",
            "enum Value { Pair(int, bool) } fn take(value: Value) -> int { 1 } fn main() -> int { take(Value::Pair(1)) }",
            "enum `Value` variant `Pair` requires 2 positional field(s), actual 1",
        ),
        (
            "String payload definition in signature",
            "enum Value { Text(String) } fn take(value: Value) -> int { 1 } fn main() -> int { 0 }",
            "enum `Value` is not an admitted non-generic unit-or-positional-CopyData enum",
        ),
        (
            "struct variant definition in signature",
            "enum Value { Named { value: int } } fn take(value: Value) -> int { 1 } fn main() -> int { 0 }",
            "enum `Value` is not an admitted non-generic unit-or-positional-CopyData enum",
        ),
        (
            "process entry parameter",
            "enum Value { Int(int) } fn main(value: Value) -> int { 0 }",
            "process entry cannot transport enums",
        ),
        (
            "process entry result",
            "enum Value { Int(int) } fn main() -> Value { Value::Int(0) }",
            "process entry cannot transport enums",
        ),
        (
            "generic transport function",
            "enum Value { Int(int) } fn take<T>(value: Value) -> Value { value } fn main() -> int { 0 }",
            "generic enum transport functions are not admitted",
        ),
        (
            "enum array parameter",
            "enum Value { Int(int) } fn take(values: [Value; 2]) -> int { 1 } fn main() -> int { 0 }",
            "enum transport function `take` parameter `values` is not an admitted by-value type",
        ),
        (
            "free enum reference dereference",
            "enum Value { Int(int) } fn take(value: &Value) -> Value { *value } fn main() -> int { 0 }",
            "not admitted Copy-data",
        ),
        (
            "String mixed parameter",
            "enum Value { Int(int) } fn take(value: Value, text: String) -> int { 1 } fn main() -> int { 0 }",
            "enum transport function `take` parameter `text` is not an admitted by-value type",
        ),
        (
            "wrong enum argument",
            "enum Value { Int(int) } enum Other { Int(int) } fn take(value: Value) -> int { 1 } fn main() -> int { take(Other::Int(2)) }",
            "Function `take` parameter `value` type mismatch: expected Value, actual Other",
        ),
        (
            "wrong enum return",
            "enum Value { Int(int) } enum Other { Int(int) } fn make() -> Value { Other::Int(2) } fn main() -> int { 0 }",
            "Function `make` return type mismatch: expected Value, actual Other",
        ),
        (
            "wrong constructor payload before transport",
            "enum Value { Int(int) } fn take(value: Value) -> int { 1 } fn main() -> int { take(Value::Int(2 > 1)) }",
            "enum `Value` variant `Int` payload type mismatch: expected int, actual bool",
        ),
        (
            "transport call arity",
            "enum Value { Int(int) } fn take(value: Value) -> int { 1 } fn main() -> int { take() }",
            "Function `take` arity mismatch: expected 1, actual 0",
        ),
        (
            "use after argument move",
            "enum Value { Int(int) } fn take(value: Value) -> int { 1 } fn main() -> int { let value = Value::Int(2); let score = take(value); let reused = value; score }",
            "Use of moved value `value`",
        ),
        (
            "discarded call moves argument",
            "enum Value { Int(int) } fn consume(value: Value) {} fn main() -> int { let value = Value::Int(2); consume(value); let reused = value; 0 }",
            "Use of moved value `value`",
        ),
        (
            "duplicate nested consumption",
            "enum Value { Int(int) } fn take(value: Value) -> int { 1 } fn pair(a: int, b: int) -> int { a + b } fn main() -> int { let value = Value::Int(2); pair(take(value), take(value)) }",
            "enum `value` is consumed more than once in one expression",
        ),
        (
            "call scrutinee arm reuse",
            "enum Value { Int(int) } fn forward(value: Value) -> Value { value } fn main() -> int { let value = Value::Int(2); match forward(value) { Value::Int(inner) => match value { Value::Int(other) => other } } }",
            "enum match arm reuses consumed scrutinee `value`",
        ),
        (
            "moved call result binding",
            "enum Value { Int(int) } fn make() -> Value { Value::Int(2) } fn main() -> int { let first = make(); let second = first; let third = first; 0 }",
            "Use of moved value `first`",
        ),
        (
            "closure transport context fails closed before enum transport",
            "enum Value { Int(int) } fn main() -> int { let f = |value: Value| value; 0 }",
            "closure expressions are parsed but unsupported in executable code",
        ),
        (
            "nested function transport context",
            "enum Value { Int(int) } fn outer() -> int { fn inner(value: Value) -> int { match value { Value::Int(inner) => inner } } 0 } fn main() -> int { outer() }",
            "Match expressions are not supported",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    let raw_source = "enum Value { Int(int) } fn forward(value: Value) -> Value { value } fn main() -> int { let value = forward(Value::Int(3)); match value { Value::Int(inner) => inner } }";
    match try_tokenize_with_locations(raw_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("raw containment parse failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedEnumParameter")
                || debug.contains("CheckedUnitEnumParameter")
                || debug.contains("CheckedFunctionDef")
                || debug.contains("CheckedEnumVariant")
                || debug.contains("CheckedEnumPayload")
                || debug.contains("CheckedEnumDispatch")
            {
                failures.push(format!(
                    "deprecated raw path activated checked payload transport: {debug}"
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
        "enum Value { Int(int), Flag(bool) } fn forward(value: Value) -> Value { value } fn main() -> int { match forward(Value::Int(7)) { Value::Int(inner) => inner, Value::Flag(inner) => 0 } }",
    )
    .expect("write valid payload-enum transport source");
    fs::write(
        &invalid,
        "enum Value { Int(int) } fn take(value: Value) -> int { 1 } fn main() -> int { let value = Value::Int(2); let score = take(value); let reused = value; score }",
    )
    .expect("write invalid payload-enum transport source");

    let valid_check = run_cli(&workspace, &[Path::new("check"), &valid]);
    if !valid_check.status.success() {
        failures.push(format!(
            "CLI check rejected valid payload-enum transport: {}",
            output_text(&valid_check)
        ));
    }
    let valid_build = run_cli(
        &workspace,
        &[Path::new("build"), &valid, Path::new("-o"), &valid_artifact],
    );
    if !valid_build.status.success() || !valid_artifact.exists() {
        failures.push(format!(
            "CLI build failed to publish valid payload-enum transport LLVM: {}",
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
        || !output_text(&invalid_build).contains("Use of moved value `value`")
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
                "Test owned scalar-payload enum transport integration example",
                "examples/payload_enum_transport/main.aero",
                "opt-22 -passes=verify -disable-output ../../payload_enum_transport.ll",
                "llc-22 -verify-machineinstrs ../../payload_enum_transport.ll",
                "clang-22 ../../payload_enum_transport.o -o ../../payload_enum_transport",
                "Expected exit code 197",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }
    if EXPECTED_EXIT != 197 {
        failures.push("payload-enum transport native exit contract drifted".to_string());
    }

    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
