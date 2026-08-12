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

const EXAMPLE_ROOT: &str = "examples/reference_parameter_transport/main.aero";
const EXAMPLE_MODULE: &str = "examples/reference_parameter_transport/borrows.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 211;

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
            "aero-reference-parameter-transport-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create reference-parameter transport workspace");
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
            .is_some_and(|name| name.starts_with("aero-reference-parameter-transport-"));
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
            "{label}: unsupported reference transport compiled:\n{llvm}"
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
fn read_int(value: &int, bias: int) -> int { *value + bias }
fn read_float(value: &float) -> bool { *value > 1.0 }
fn read_bool(value: &bool) -> bool { *value }
fn forward(value: &int, bias: int) -> int { read_int(value, bias) }
fn chain(value: &int) -> int {
    let alias = value;
    forward(alias, 2)
}
fn combine(left: &int, ratio: &float, ready: &bool, right: &int, bias: int) -> int {
    if *ready && *ratio > 1.0 { return *left + *right + bias; }
    0
}
fn recurse(value: &int, depth: int) -> int {
    if depth == 0 { return *value; }
    recurse(value, depth - 1)
}
fn consume(value: &int) { let seen = *value; }

fn main() -> int {
    let left = 40;
    let right = 2;
    let ratio = 1.5;
    let ready = left > right;
    let left_ref: &int = &left;
    let copied = left_ref;
    let right_ref = &right;
    let ratio_ref: &float = &ratio;
    let ready_ref: &bool = &ready;
    let first = &left;
    let second = &left;
    consume(copied);
    let total = combine(first, ratio_ref, ready_ref, right_ref, 1);
    if read_bool(ready_ref) && read_float(&ratio) && read_int(&left, 1) == 41
        && chain(copied) == 42 && recurse(second, 2) == 40
        && left == 40 && *left_ref == 40 {
        return total;
    }
    1
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod borrows;

enum Signal { Idle, Count(int), Ready(bool) }
enum Mode { Off, On }
struct Packet { value: int, ready: bool }
struct Envelope { packet: Packet, rows: [Packet; 2] }

fn roundtrip(value: Envelope) -> Envelope { value }
fn signal_score(value: Signal) -> int {
    match value {
        Signal::Idle => 1,
        Signal::Count(inner) => inner,
        Signal::Ready(inner) => 3
    }
}
fn make_signal(value: int) -> Signal { Signal::Count(value) }
fn mode_score(value: Mode) -> int { match value { Mode::Off => 1, Mode::On => 6 } }

fn main() -> int {
    let base = 100;
    let ratio = 2.5;
    let ready = base > 0;
    let base_ref: &int = &base;
    let alias = base_ref;
    let packet = Packet { value: 10, ready: ready };
    let envelope = roundtrip(Envelope { packet: packet, rows: [packet, packet] });
    let borrowed = module_chain(alias, &ratio, &ready, 3);
    let signal = signal_score(make_signal(40));
    let mode = mode_score(Mode::On);
    if envelope.packet.ready && envelope.rows[1].value == 10
        && "aero".len() == 4 && borrowed == 163 && *base_ref == 100 {
        return borrowed + signal + mode + envelope.rows.len();
    }
    1
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn module_read(value: &int, bias: int) -> int { *value + bias }
fn module_bool(value: &bool) -> bool { *value }
fn module_ratio(value: &float) -> int { if *value > 2.0 { return 20; } 0 }
fn module_forward(value: &int, bias: int) -> int { module_read(value, bias) }
fn module_chain(value: &int, ratio: &float, ready: &bool, bias: int) -> int {
    let alias = value;
    if module_bool(ready) {
        return module_forward(alias, bias) + module_ratio(ratio) + 40;
    }
    0
}
"#
}

#[test]
fn immutable_scalar_reference_parameter_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = "fn read(left: &int, ratio: &float, ready: &bool, bias: int) -> int { *left + bias } fn main() -> int { 0 }";
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
            })) = ast.first()
            else {
                failures.push(format!(
                    "parser omitted reference-parameter function: {ast:?}"
                ));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            let expected_pointees = ["int", "float", "bool"];
            if name != "read"
                || !type_params.is_empty()
                || parameters.len() != 4
                || !expected_pointees
                    .iter()
                    .enumerate()
                    .all(|(index, expected)| {
                        matches!(
                            &parameters[index].param_type,
                            Type::Reference(inner, false)
                                if matches!(inner.as_ref(), Type::Named(name) if name == expected)
                        )
                    })
                || !matches!(&parameters[3].param_type, Type::Named(name) if name == "int")
                || !matches!(return_type, Some(Type::Named(name)) if name == "int")
            {
                failures.push(format!(
                    "parser changed reference-parameter signature topology: {ast:?}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "complete reference transport composition",
            positive_source(),
            vec![
                "define i32 @read_int(double* %aero.arg.value, i32 %aero.arg.bias)",
                "define i1 @read_bool(i1* %aero.arg.value)",
                "getelementptr inbounds double, double* %aero.arg.value, i64 0",
                "call i32 @read_int(double* %ptr",
                "load i1, i1* %ptr",
            ],
        ),
        (
            "direct Int borrow argument",
            "fn read(value: &int) -> int { *value } fn main() -> int { let value = 7; read(&value) }",
            vec!["define i32 @read(double*", "call i32 @read(double* %ptr"],
        ),
        (
            "Float reference result consumer",
            "fn positive(value: &float) -> bool { *value > 0.0 } fn main() -> int { let value = 1.5; if positive(&value) { return 8; } 0 }",
            vec![
                "define i1 @positive(double*",
                "call i1 @positive(double* %ptr",
            ],
        ),
        (
            "Bool reference Void consumer",
            "fn consume(value: &bool) { let seen = *value; } fn main() -> int { let ready = 2 > 1; let alias = &ready; consume(alias); 9 }",
            vec!["define void @consume(i1*", "call void @consume(i1* %ptr"],
        ),
        (
            "multiple same-owner references",
            "fn sum(left: &int, right: &int) -> int { *left + *right } fn main() -> int { let value = 4; let first = &value; let second = &value; sum(first, second) }",
            vec!["@sum(double* %aero.arg.left, double* %aero.arg.right)"],
        ),
        (
            "forwarded parameter alias",
            "fn inner(value: &int) -> int { *value } fn outer(value: &int) -> int { let alias = value; inner(alias) } fn main() -> int { let value = 5; outer(&value) }",
            vec!["define i32 @outer(double*", "call i32 @inner(double* %ptr"],
        ),
        (
            "projected field borrow argument",
            "struct Row { value: int } fn read(value: &int) -> int { *value } fn main() -> int { let row = Row { value: 1 }; read(&row.value) }",
            vec![
                "getelementptr inbounds %aero.struct.Row",
                "call i32 @read(double* %ptr",
            ],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!(
            "checked reference-parameter IR/LLVM failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            if debug.matches("CheckedImmutableReferenceParameter").count() < 11 {
                failures.push(format!(
                    "checked IR lost immutable reference parameter binders:\n{debug}"
                ));
            }
            for logical in [
                "ImmutableReference { pointee: Int }",
                "ImmutableReference { pointee: Float }",
                "ImmutableReference { pointee: Bool }",
            ] {
                if !debug.contains(logical) {
                    failures.push(format!(
                        "checked metadata missing reference signature {logical:?}:\n{debug}"
                    ));
                }
            }
            if llvm.contains("inttoptr")
                || llvm.contains("ptrtoint")
                || llvm.contains("bitcast double*")
                || llvm.contains("bitcast i1*")
            {
                failures.push(format!(
                    "LLVM converted a verified reference through an unrelated type:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, required) in [
        (
            "CORE-059 array reference-parameter extension",
            "fn read(value: &[int; 2]) -> int { let copy = *value; copy[1] } fn main() -> int { let values = [1, 2]; read(&values) }",
            vec!["define i32 @read([2 x double]*"],
        ),
        (
            "CORE-059 struct reference-parameter extension",
            "struct Row { value: int } fn read(value: &Row) -> int { (*value).value } fn main() -> int { let row = Row { value: 2 }; read(&row) }",
            vec!["define i32 @read(%aero.struct.Row*"],
        ),
        (
            "CORE-059 admitted Copy-data by-value companion",
            "struct Row { value: int } fn read(value: &int, row: Row) -> int { *value + row.value } fn main() -> int { let value = 1; let row = Row { value: 2 }; read(&value, row) }",
            vec!["%aero.struct.Row %aero.arg.row"],
        ),
    ] {
        failures.extend(expect_success(label, source, &required));
    }

    for (label, source, expected) in [
        (
            "immutable reference result",
            "fn bad(value: &int) -> &int { value } fn main() -> int { 0 }",
            "reference results require lifetime semantics and are not supported by CORE-053",
        ),
        (
            "local reference result",
            "fn bad() -> &int { let value = 1; &value } fn main() -> int { 0 }",
            "reference results require lifetime semantics and are not supported by CORE-053",
        ),
        (
            "String reference parameter",
            "fn read(value: &string) -> int { 0 } fn main() -> int { 0 }",
            "admitted Copy-data",
        ),
        (
            "free enum reference dereference",
            "enum Value { One } fn read(value: &Value) -> Value { *value } fn main() -> int { 0 }",
            "not admitted Copy-data",
        ),
        (
            "nested reference parameter",
            "fn read(value: &&int) -> int { 0 } fn main() -> int { 0 }",
            "Expected type",
        ),
        (
            "enum by-value companion",
            "enum Value { One } fn read(value: &int, item: Value) -> int { *value } fn main() -> int { 0 }",
            "reference transport function `read` parameter `item` is not admitted Copy-data",
        ),
        (
            "generic reference function",
            "fn read<T>(value: &int) -> int { *value } fn main() -> int { 0 }",
            "generic reference transport functions are not supported by CORE-053",
        ),
        (
            "process entry reference parameter",
            "fn main(value: &int) -> int { *value }",
            "process entry cannot use reference parameters",
        ),
        (
            "wrong direct-borrow pointee",
            "fn read(value: &int) -> int { *value } fn main() -> int { let value = 1.5; read(&value) }",
            "parameter `value` type mismatch",
        ),
        (
            "scalar passed as reference",
            "fn read(value: &int) -> int { *value } fn main() -> int { let value = 1; read(value) }",
            "parameter `value` type mismatch",
        ),
        (
            "reference passed as scalar",
            "fn read(value: int) -> int { value } fn main() -> int { let value = 1; let alias = &value; read(alias) }",
            "parameter `value` type mismatch",
        ),
        (
            "borrowed literal argument",
            "fn read(value: &int) -> int { *value } fn main() -> int { read(&1) }",
            "requires an identifier place",
        ),
        (
            "borrowed computation argument",
            "fn read(value: &int) -> int { *value } fn main() -> int { let value = 1; read(&(value + 1)) }",
            "requires an identifier place",
        ),
        (
            "reference stored in aggregate",
            "struct Holder { value: &int } fn main() -> int { let value = 1; let holder = Holder { value: &value }; 0 }",
            "Struct construction expressions are not supported",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    if let Err(error) = compile_program(
        "fn read(value: &mut int) -> int { *value } fn main() -> int { 0 }",
        CompilerOptions::default(),
    ) {
        failures.push(format!(
            "CORE-056 mutable reference parameter definition regressed: {error}"
        ));
    }

    match analyzed_ast(
        "fn read(value: &int) -> int { *value } fn main() -> int { let value = 1; read(&value) }",
    ) {
        Err(error) => failures.push(format!("raw-containment setup failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedImmutableReferenceParameter") {
                failures.push(format!(
                    "deprecated raw generation activated checked reference transport:\n{debug}"
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
                    "tracked root reference-transport example drifted at {}",
                    example.display()
                ));
            }
            if module_actual != tracked_module_source() {
                failures.push(format!(
                    "tracked reference-transport module drifted at {}",
                    module.display()
                ));
            }
        }
        (root_result, module_result) => failures.push(format!(
            "tracked reference-transport example pair missing/unreadable: root={:?}, module={:?}",
            root_result.err(),
            module_result.err()
        )),
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for reference-parameter integration anchor");
    for anchor in [
        "Test reference-parameter transport integration example",
        "examples/reference_parameter_transport/main.aero",
        "cargo run -- build ../../examples/reference_parameter_transport/main.aero -o ../../reference_parameter_transport.ll",
        "if [ $exit_code -ne 211 ]; then",
        "reference-parameter transport example passed with exit code 211",
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
        let output_path = workspace.path("reference_parameter_transport.ll");
        let build = run_cli(
            &workspace,
            &[Path::new("build"), &example, Path::new("-o"), &output_path],
        );
        let diagnostics = output_text(&build);
        if !build.status.success() || !output_path.is_file() {
            failures.push(format!(
                "tracked reference-parameter example failed checked CLI build:\n{diagnostics}"
            ));
        }
    }

    let workspace = TestWorkspace::new("invalid-hygiene");
    let invalid = workspace.path("invalid.aero");
    let output_path = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn read(value: &int) -> int { *value } fn main() -> int { let value = 1.5; read(&value) }",
    )
    .expect("write invalid reference-parameter source");
    let output = run_cli(
        &workspace,
        &[Path::new("build"), &invalid, Path::new("-o"), &output_path],
    );
    let diagnostics = output_text(&output);
    if output.status.success()
        || output_path.exists()
        || !diagnostics.contains("parameter `value` type mismatch")
    {
        failures.push(format!(
            "invalid reference-parameter CLI hygiene failed (status={}, artifact={}):\n{}",
            output.status,
            output_path.exists(),
            diagnostics
        ));
    }

    assert!(
        failures.is_empty(),
        "CORE-053 immutable scalar-reference parameter failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n")
    );
}
