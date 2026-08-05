use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT_RELATIVE: &str = "examples/fixed_copy_array_transport/main.aero";
const EXAMPLE_MODULE_RELATIVE: &str = "examples/fixed_copy_array_transport/arrays.aero";
const WORKFLOW_RELATIVE: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 91;

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
            "aero-fixed-copy-array-transport-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fixed Copy-array transport workspace");
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
            .is_some_and(|name| name.starts_with("aero-fixed-copy-array-transport-"));
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
    let checked = IrGenerator::new()
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
            "{label}: unsupported fixed Copy-array transport compiled:\n{llvm}"
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
struct Packet { total: int, ratio: float, ready: bool }

fn int_identity(values: [int; 3]) -> [int; 3] { return values; }
fn float_tail(values: [float; 2]) -> [float; 2] { values }
fn packet_identity(values: [Packet; 2]) -> [Packet; 2] {
    let copied = values;
    return copied;
}
fn empty_identity(values: [i32; 0]) -> [int; 0] { values }
fn forward(values: [int; 3]) -> [i32; 3] { return declared_later(values); }
fn declared_later(values: [i32; 3]) -> [int; 3] { values }
fn recurse(values: [int; 3], remaining: int) -> [int; 3] {
    if remaining > 0 { return recurse(values, remaining - 1); }
    return values;
}
fn mixed(prefix: int, values: [int; 3], packet: Packet, packets: [Packet; 2], suffix: int) -> int {
    return prefix + values[0] + packet.total + packets[1].total + suffix;
}
fn scan(values: [Packet; 2]) -> int {
    for value in values {
        if !value.ready { return 0; }
    }
    return values.len();
}

fn main() -> int {
    let original: [int; 3] = [10, 11, 12];
    let returned = int_identity(original);
    let forwarded = forward(returned);
    let recursive = recurse(forwarded, 2);
    let floats = float_tail([1.5, 2.5]);
    let packets: [Packet; 2] = [
        Packet { total: 20, ratio: 2.0, ready: 1 < 2 },
        Packet { total: 21, ratio: 3.0, ready: 2 < 3 }
    ];
    let copied = packet_identity(packets);
    let empty: [int; 0] = [];
    let returned_empty = empty_identity(empty);
    let immediate = int_identity([4, 5, 6]);
    if original[0] == 10 && returned[1] == 11 && recursive[2] == 12
        && immediate[1] == 5 && int_identity(original).len() == 3
        && floats[1] > 2.0 && packets[0].total == 20
        && copied[1].ready && scan(copied) == 2 && returned_empty.len() == 0
        && mixed(1, original, copied[0], packets, 2) == 54
        && "aero".len() == 4 {
        return 91;
    }
    return 1;
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod arrays;

struct RootValue { field: int, ready: bool }

fn root_roundtrip(values: [int; 3]) -> [int; 3] { return values; }
fn root_struct_roundtrip(values: [RootValue; 2]) -> [RootValue; 2] { values }

fn main() -> int {
    let original = [10, 11, 12];
    let root_values = [
        RootValue { field: 20, ready: 1 < 2 },
        RootValue { field: 21, ready: 2 < 3 }
    ];
    let returned = root_roundtrip(original);
    let copied = root_struct_roundtrip(root_values);
    let module_values = module_roundtrip(module_make(30));
    let empty: [float; 0] = [];
    let returned_empty = module_empty(empty);
    if original[0] == 10 && returned[1] == 11 && copied[1].ready
        && module_values[0].field == 30 && module_values[1].field == 31
        && returned_empty.len() == 0 && module_scan(module_values) == 2
        && "aero".starts_with("ae") {
        return 91;
    }
    return 1;
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"struct ModuleValue { field: int, ready: bool }

fn module_make(base: int) -> [ModuleValue; 2] {
    return [
        ModuleValue { field: base, ready: 1 < 2 },
        ModuleValue { field: base + 1, ready: 2 < 3 }
    ];
}
fn module_roundtrip(values: [ModuleValue; 2]) -> [ModuleValue; 2] { values }
fn module_empty(values: [float; 0]) -> [f64; 0] { return values; }
fn module_scan(values: [ModuleValue; 2]) -> int {
    for value in values {
        if !value.ready { return 0; }
    }
    return values.len();
}
"#
}

#[test]
fn fixed_copy_array_transport_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    failures.extend(expect_success(
        "all existing flat Copy arrays cross internal calls by value",
        positive_source(),
        &[
            "define [3 x double] @int_identity([3 x double] %aero.arg.values)",
            "define [2 x double] @float_tail([2 x double] %aero.arg.values)",
            "define [2 x %aero.struct.Packet] @packet_identity([2 x %aero.struct.Packet] %aero.arg.values)",
            "call [3 x double] @int_identity([3 x double]",
            "call [2 x %aero.struct.Packet] @packet_identity([2 x %aero.struct.Packet]",
            "load [3 x double], [3 x double]*",
            "store [3 x double]",
            "ret [3 x double]",
        ],
    ));

    for (label, source, llvm_type) in [
        (
            "int alias spelling and zero count",
            "fn identity(values: [i32; 0]) -> [int; 0] { values } fn main() -> int { let values: [int; 0] = []; return identity(values).len(); }",
            "[0 x double]",
        ),
        (
            "float alias spelling and repeat argument",
            "fn identity(values: [f64; 2]) -> [float; 2] { return values; } fn main() -> int { let values = identity([1.5; 2]); if values[1] > 1.0 { return 1; } return 0; }",
            "[2 x double]",
        ),
        (
            "Copy-struct literal argument and immediate projection",
            "struct Value { field: int } fn identity(values: [Value; 1]) -> [Value; 1] { values } fn main() -> int { return identity([Value { field: 7 }])[0].field; }",
            "[1 x %aero.struct.Value]",
        ),
        (
            "recursive Bool array transport",
            "fn identity(values: [bool; 1]) -> [bool; 1] { values } fn main() -> int { let values = identity([1 < 2]); if values[0] { return 1; } 0 }",
            "[1 x i1]",
        ),
        (
            "recursive nested array transport",
            "fn identity(values: [[int; 1]; 1]) -> [[int; 1]; 1] { values } fn main() -> int { let values = identity([[7]]); values[0][0] }",
            "[1 x [1 x double]]",
        ),
    ] {
        failures.extend(expect_success(
            label,
            source,
            &["define ", llvm_type, "call "],
        ));
    }

    for (label, source, expected) in [
        (
            "argument count mismatch",
            "fn take(values: [int; 2]) -> int { return values.len(); } fn main() -> int { return take([1, 2, 3]); }",
            "parameter `values` type mismatch: expected [int; 2], actual [int; 3]",
        ),
        (
            "argument numeric kind mismatch",
            "fn take(values: [int; 1]) -> int { return values[0]; } fn main() -> int { return take([1.5]); }",
            "parameter `values` type mismatch: expected [int; 1], actual [float; 1]",
        ),
        (
            "argument exact struct mismatch",
            "struct Left { field: int } struct Right { field: int } fn take(values: [Left; 1]) -> int { return values[0].field; } fn main() -> int { return take([Right { field: 1 }]); }",
            "parameter `values` type mismatch: expected [Left; 1], actual [Right; 1]",
        ),
        (
            "return count mismatch",
            "fn make() -> [int; 2] { return [1, 2, 3]; } fn main() -> int { return 0; }",
            "return type mismatch: expected [int; 2], actual [int; 3]",
        ),
        (
            "return exact struct mismatch",
            "struct Left { field: int } struct Right { field: int } fn make() -> [Left; 1] { return [Right { field: 1 }]; } fn main() -> int { return 0; }",
            "return type mismatch: expected [Left; 1], actual [Right; 1]",
        ),
        (
            "String arrays remain excluded",
            "fn take(values: [String; 1]) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter `values` uses an unsupported fixed-array element type",
        ),
        (
            "process entry array parameter",
            "fn main(values: [int; 1]) -> int { return values[0]; }",
            "process entry `main` cannot use aggregate parameters or returns",
        ),
        (
            "process entry array return",
            "fn main() -> [int; 1] { return [1]; }",
            "process entry `main` cannot use aggregate parameters or returns",
        ),
        (
            "generic array transport remains excluded",
            "fn identity<T>(values: [int; 1]) -> [int; 1] { values } fn main() -> int { return 0; }",
            "generic function IR is not admitted",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked metadata specimen failed: {error}")),
        Ok((checked, llvm)) => {
            let metadata = format!("{:#?}", checked.metadata());
            for anchor in [
                "Array {",
                "element: Int",
                "element: Float",
                "name: \"Packet\"",
                "count: 0",
                "count: 2",
                "count: 3",
            ] {
                if !metadata.contains(anchor) {
                    failures.push(format!(
                        "checked metadata lacks array transport anchor {anchor:?}: {metadata}"
                    ));
                }
            }
            if llvm.matches("call [3 x double] @int_identity(").count() != 3 {
                failures.push(format!(
                    "array calls were duplicated or lost during lowering:\n{llvm}"
                ));
            }
        }
    }

    #[allow(deprecated)]
    {
        let raw_source =
            "fn identity(values: [int; 1]) -> [int; 1] { values } fn main() -> int { return 0; }";
        let tokens = try_tokenize_with_locations(raw_source, None).expect("raw control lexes");
        let ast = parse_with_locations(tokens).expect("raw control parses");
        let raw = IrGenerator::new().generate_ir(ast);
        let unchecked = compiler::generate_code(raw);
        if unchecked.contains("define [1 x double] @identity") {
            failures.push(format!(
                "deprecated unchecked generation activated checked array transport:\n{unchecked}"
            ));
        }
    }

    let module_workspace = TestWorkspace::new("direct-module-product");
    let root_source = module_workspace.path("main.aero");
    let module_source = module_workspace.path("arrays.aero");
    let artifact = module_workspace.path("program.ll");
    fs::write(&root_source, tracked_root_source()).expect("write root array transport source");
    fs::write(&module_source, tracked_module_source())
        .expect("write module array transport source");
    let check = run_cli(&module_workspace, &[Path::new("check"), &root_source]);
    if !check.status.success() {
        failures.push(format!(
            "direct-module fixed Copy-array transport check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &module_workspace,
        &[Path::new("build"), &root_source, Path::new("-o"), &artifact],
    );
    if !build.status.success() {
        failures.push(format!(
            "direct-module fixed Copy-array transport build failed: {}",
            output_text(&build)
        ));
    }
    if !artifact.is_file() {
        failures.push("direct-module fixed Copy-array transport build omitted LLVM".to_string());
    }

    let invalid_workspace = TestWorkspace::new("invalid-no-artifact");
    let invalid_source = invalid_workspace.path("main.aero");
    let invalid_artifact = invalid_workspace.path("program.ll");
    fs::write(
        &invalid_source,
        "fn take(values: [int; 2]) -> int { return values.len(); } fn main() -> int { return take([1]); }",
    )
    .expect("write invalid array transport source");
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
            "invalid array transport build exited zero: {}",
            output_text(&invalid_build)
        ));
    }
    if invalid_artifact.exists() {
        failures.push("invalid array transport build published an artifact".to_string());
    }

    let root = repository_root();
    for (relative, expected_source) in [
        (EXAMPLE_ROOT_RELATIVE, tracked_root_source()),
        (EXAMPLE_MODULE_RELATIVE, tracked_module_source()),
    ] {
        let path = root.join(relative);
        match fs::read_to_string(&path) {
            Err(error) => failures.push(format!(
                "tracked CORE-046 example is unavailable at {}: {error}",
                path.display()
            )),
            Ok(source) if source != expected_source => failures.push(format!(
                "tracked CORE-046 example bytes differ from tested source at {}",
                path.display()
            )),
            Ok(_) => {}
        }
    }

    let workflow = root.join(WORKFLOW_RELATIVE);
    match fs::read_to_string(&workflow) {
        Err(error) => failures.push(format!("cannot read {}: {error}", workflow.display())),
        Ok(workflow) => {
            for anchor in [
                "Test fixed Copy-array transport integration example",
                "examples/fixed_copy_array_transport/main.aero",
                "fixed_copy_array_transport.ll",
                "opt-22 -passes=verify -disable-output",
                "llc-22 -verify-machineinstrs",
                "llc-22 -filetype=obj",
                "clang-22",
                "fixed Copy-array transport example passed with exit code 91",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "Rust workflow is missing CORE-046 anchor {anchor:?}"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-046 fixed Copy-array transport failures (expected native sentinel {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
