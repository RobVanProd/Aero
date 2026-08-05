use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_RELATIVE: &str = "examples/scalar_struct_copy_transport.aero";
const WORKFLOW_RELATIVE: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 63;

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
            "aero-struct-copy-transport-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create struct Copy transport workspace");
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
            .is_some_and(|name| name.starts_with("aero-struct-copy-transport-"));
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

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed) = analyzer.analyze(ast)?;
    let mut generator = IrGenerator::new();
    let checked = generator
        .try_generate_ir(analyzed)
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
            "{label}: unsupported Copy transport compiled:\n{llvm}"
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
struct Packet {
    total: int,
    alias: i32,
    ratio: float,
    precise: f64,
    ready: bool
}
struct Count { value: int }

fn passthrough(packet: Packet) -> Packet {
    let first: Packet = packet;
    let mut second = first;
    return second;
}

fn tail_copy(packet: Packet) -> Packet { packet }

fn literal_count(value: int) -> Count {
    return Count { value: value };
}

fn forward(packet: Packet) -> Packet {
    return declared_later(packet);
}

fn declared_later(packet: Packet) -> Packet {
    return packet;
}

fn recurse(packet: Packet, remaining: int) -> Packet {
    if remaining > 0 {
        return recurse(packet, remaining - 1);
    }
    return packet;
}

fn score(prefix: int, packet: Packet, suffix: int) -> int {
    return prefix + packet.total + packet.alias + suffix;
}

fn consume(packet: Packet) {
    let local = packet;
    local;
}

fn main() -> int {
    let original: Packet = Packet {
        ready: 1 < 2,
        precise: 4.5,
        ratio: 3.5,
        alias: 2,
        total: 20
    };
    let copy = original;
    let mut second: Packet = copy;
    consume(second);
    let returned = passthrough(second);
    let forwarded = forward(tail_copy(returned));
    let recursive = recurse(forwarded, 2);
    let chained = passthrough(passthrough(recursive));
    passthrough(chained);
    let values = [1, 2, 3];
    let text = "aero";
    if original.ready && copy.ready && second.ready && chained.ready
        && chained.ratio > 3.0 && chained.precise > 4.0
        && text.starts_with("ae") {
        return score(5, chained, 6) + original.total
            + literal_count(3).value + values.len() + text.len();
    }
    return 1;
}
"#
}

#[test]
fn scalar_struct_copy_transport_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    failures.extend(expect_success(
        "local copies, mixed calls, returns, forwarding, recursion, and system composition",
        positive_source(),
        &[
            "%aero.struct.Count = type { double }",
            "%aero.struct.Packet = type { double, double, double, double, i1 }",
            "define %aero.struct.Packet @passthrough(%aero.struct.Packet %aero.arg.packet)",
            "call %aero.struct.Packet @passthrough(",
            "load %aero.struct.Packet",
            "store %aero.struct.Packet",
        ],
    ));

    for (field_type, value, projection) in [
        ("int", "7", "return returned.field;"),
        ("i32", "8", "return returned.field;"),
        (
            "float",
            "9.5",
            "if returned.field > 9.0 { return 1; } return 0;",
        ),
        (
            "f64",
            "10.5",
            "if returned.field > 10.0 { return 1; } return 0;",
        ),
        ("bool", "1 < 2", "if returned.field { return 1; } return 0;"),
    ] {
        let source = format!(
            "struct Value {{ field: {field_type} }} fn identity(value: Value) -> Value {{ return value; }} fn main() -> int {{ let original = Value {{ field: {value} }}; let copy = original; let returned = identity(copy); {projection} }}"
        );
        failures.extend(expect_success(
            &format!("Copy transport scalar alias {field_type}"),
            &source,
            &[
                "%aero.struct.Value = type",
                "define %aero.struct.Value @identity",
                "call %aero.struct.Value @identity",
            ],
        ));
    }

    let additional_positive = [
        (
            "multiple struct parameters and exact multiplicity",
            "struct Value { field: int } fn combine(left: Value, bias: int, right: Value) -> int { return left.field + bias + right.field; } fn main() -> int { let value = Value { field: 4 }; return combine(value, 2, value); }",
        ),
        (
            "literal argument and literal return",
            "struct Value { field: int } fn make(value: int) -> Value { return Value { field: value }; } fn read(value: Value) -> int { return value.field; } fn main() -> int { return read(make(9)); }",
        ),
        (
            "tail call return and immediate call projection",
            "struct Value { field: int } fn identity(value: Value) -> Value { value } fn forward(value: Value) -> Value { identity(value) } fn main() -> int { return forward(Value { field: 11 }).field; }",
        ),
        (
            "discarded struct call preserves original",
            "struct Value { field: int } fn identity(value: Value) -> Value { return value; } fn main() -> int { let original = Value { field: 12 }; identity(original); return original.field; }",
        ),
    ];
    for (label, source) in additional_positive {
        failures.extend(expect_success(
            label,
            source,
            &["%aero.struct.Value = type"],
        ));
    }

    let negative_cases = [
        (
            "unsupported String-field parameter",
            "struct Value { field: String } fn consume(value: Value) -> int { return 1; } fn main() -> int { return 0; }",
            "function parameter `value` is not an admitted scalar type",
        ),
        (
            "unsupported nested-struct parameter",
            "struct Inner { field: int } struct Outer { inner: Inner } fn consume(value: Outer) -> int { return 1; } fn main() -> int { return 0; }",
            "function parameter `value` is not an admitted scalar type",
        ),
        (
            "unknown struct parameter",
            "fn consume(value: Missing) -> int { return 1; } fn main() -> int { return 0; }",
            "function parameter `value` is not an admitted scalar type",
        ),
        (
            "distinct struct argument mismatch",
            "struct Left { field: int } struct Right { field: int } fn consume(value: Left) -> int { return value.field; } fn main() -> int { return consume(Right { field: 1 }); }",
            "Function `consume` parameter `value` type mismatch: expected Left, actual Right",
        ),
        (
            "distinct struct return mismatch",
            "struct Left { field: int } struct Right { field: int } fn make() -> Left { return Right { field: 1 }; } fn main() -> int { return 0; }",
            "Function `make` return type mismatch: expected Left, actual Right",
        ),
        (
            "struct call arity mismatch",
            "struct Value { field: int } fn consume(value: Value) -> int { return value.field; } fn main() -> int { return consume(); }",
            "Function `consume` arity mismatch: expected 1, actual 0",
        ),
        (
            "process entry struct parameter",
            "struct Value { field: int } fn main(value: Value) -> int { return value.field; }",
            "process entry `main` cannot use struct parameters or returns",
        ),
        (
            "process entry struct return",
            "struct Value { field: int } fn main() -> Value { return Value { field: 1 }; }",
            "process entry `main` cannot use struct parameters or returns",
        ),
        (
            "generic struct transport function",
            "struct Value { field: int } fn identity<T>(value: Value) -> Value { return value; } fn main() -> int { return 0; }",
            "generic function IR is not admitted",
        ),
        (
            "non-Copy struct local remains excluded",
            "struct Value { field: String } fn main() -> int { let first = Value { field: \"x\" }; let second = first; return 0; }",
            "Struct construction expressions are not supported.",
        ),
    ];
    for (label, source, expected) in negative_cases {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked metadata specimen failed: {error}")),
        Ok((checked, llvm)) => {
            let metadata = format!("{:#?}", checked.metadata());
            for expected in [
                "name: \"Packet\"",
                "signature: FunctionSignature",
                "Struct {",
                "result: Struct",
                "Int",
                "Float",
                "Bool",
            ] {
                if !metadata.contains(expected) {
                    failures.push(format!(
                        "checked metadata lacks Copy transport anchor {expected:?}: {metadata}"
                    ));
                }
            }
            let count_position = llvm.find("%aero.struct.Count = type");
            let packet_position = llvm.find("%aero.struct.Packet = type");
            if !matches!((count_position, packet_position), (Some(left), Some(right)) if left < right)
            {
                failures.push(format!(
                    "LLVM struct schemas are absent or not exact-name sorted:\n{llvm}"
                ));
            }
            if llvm
                .matches("call %aero.struct.Packet @passthrough(")
                .count()
                != 4
            {
                failures.push(format!(
                    "nested/discarded passthrough calls were duplicated or lost:\n{llvm}"
                ));
            }
            if llvm.matches("alloca %aero.struct.Packet").count() < 12 {
                failures.push(format!(
                    "Copy transport did not allocate distinct aggregate storage:\n{llvm}"
                ));
            }
        }
    }

    #[allow(deprecated)]
    {
        let raw_source = "struct Packet { total: int, alias: i32, ratio: float, precise: f64, ready: bool } fn identity(value: Packet) -> Packet { return value; } fn main() -> int { return 0; }";
        let tokens = try_tokenize_with_locations(raw_source, None).expect("raw control lexes");
        let ast = parse_with_locations(tokens).expect("raw control parses");
        let mut generator = IrGenerator::new();
        let raw = generator.generate_ir(ast.clone());
        let unchecked = compiler::generate_code(raw);
        if unchecked.contains("%aero.struct") {
            failures.push(format!(
                "deprecated unchecked generation activated checked aggregate transport:\n{unchecked}"
            ));
        }

        let checked_tokens = try_tokenize_with_locations(positive_source(), None)
            .expect("checked generator reuse control lexes");
        let checked_ast =
            parse_with_locations(checked_tokens).expect("checked generator reuse control parses");
        let mut analyzer = SemanticAnalyzer::new();
        let (_, checked_ast) = analyzer
            .analyze(checked_ast)
            .expect("checked generator reuse control analyzes");
        generator
            .try_generate_ir(checked_ast)
            .expect("checked generator reuse control verifies");
        let reused_raw = generator.generate_ir(ast);
        let reused_unchecked = compiler::generate_code(reused_raw);
        if !reused_unchecked.contains("define double @identity(double %aero.arg.value)") {
            failures.push(format!(
                "deprecated unchecked generation lost its legacy identity definition after checked reuse:\n{reused_unchecked}"
            ));
        }
        if reused_unchecked.contains("%aero.struct") {
            failures.push(format!(
                "deprecated unchecked generation inherited checked aggregate state:\n{reused_unchecked}"
            ));
        }
    }

    let module_workspace = TestWorkspace::new("direct-module-product");
    let root_source = module_workspace.path("main.aero");
    let module_source = module_workspace.path("shapes.aero");
    let artifact = module_workspace.path("program.ll");
    fs::write(
        &root_source,
        "mod shapes; struct RootValue { count: int } fn root_roundtrip(value: RootValue) -> RootValue { return value; } fn main() -> int { let module_value = module_make(40); let copied = module_roundtrip(module_value); let root = RootValue { count: 12 }; return module_score(root_roundtrip(root), copied); }",
    )
    .expect("write root Copy transport source");
    fs::write(
        &module_source,
        "struct ModuleValue { count: int } fn module_make(value: int) -> ModuleValue { return ModuleValue { count: value }; } fn module_roundtrip(value: ModuleValue) -> ModuleValue { return value; } fn module_score(root: RootValue, module: ModuleValue) -> int { return root.count + module.count; }",
    )
    .expect("write module Copy transport source");
    let check = run_cli(&module_workspace, &[Path::new("check"), &root_source]);
    if !check.status.success() {
        failures.push(format!(
            "direct-module Copy transport check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &module_workspace,
        &[Path::new("build"), &root_source, Path::new("-o"), &artifact],
    );
    if !build.status.success() {
        failures.push(format!(
            "direct-module Copy transport build failed: {}",
            output_text(&build)
        ));
    }
    if !artifact.is_file() {
        failures.push("direct-module Copy transport build did not publish LLVM".to_string());
    }

    let invalid_workspace = TestWorkspace::new("invalid-no-artifact");
    let invalid_source = invalid_workspace.path("main.aero");
    let invalid_artifact = invalid_workspace.path("program.ll");
    fs::write(
        &invalid_source,
        "struct Left { field: int } struct Right { field: int } fn consume(value: Left) -> int { return value.field; } fn main() -> int { return consume(Right { field: 1 }); }",
    )
    .expect("write invalid Copy transport source");
    let invalid_build = run_cli(
        &invalid_workspace,
        &[
            Path::new("build"),
            &invalid_source,
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    if invalid_build.status.success() {
        failures.push(format!(
            "invalid Copy transport build exited zero: {}",
            output_text(&invalid_build)
        ));
    }
    if invalid_artifact.exists() {
        failures.push("invalid Copy transport build published an artifact".to_string());
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_RELATIVE);
    match fs::read_to_string(&example) {
        Err(error) => failures.push(format!(
            "tracked CORE-044 example is unavailable at {}: {error}",
            example.display()
        )),
        Ok(source) if source != positive_source() => failures.push(format!(
            "tracked CORE-044 example bytes differ from the tested source at {}",
            example.display()
        )),
        Ok(_) => {}
    }

    let workflow = root.join(WORKFLOW_RELATIVE);
    match fs::read_to_string(&workflow) {
        Err(error) => failures.push(format!("cannot read {}: {error}", workflow.display())),
        Ok(workflow) => {
            for anchor in [
                "Test scalar struct Copy transport integration example",
                "examples/scalar_struct_copy_transport.aero",
                "scalar_struct_copy_transport.ll",
                "opt-22 -passes=verify -disable-output",
                "llc-22 -verify-machineinstrs",
                "llc-22 -filetype=obj",
                "clang-22",
                "struct Copy transport example passed with exit code 63",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "Rust workflow is missing CORE-044 anchor {anchor:?}"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-044 struct Copy transport failures (expected native sentinel {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
