use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT_RELATIVE: &str = "examples/fixed_copy_struct_arrays/main.aero";
const EXAMPLE_MODULE_RELATIVE: &str = "examples/fixed_copy_struct_arrays/shapes.aero";
const WORKFLOW_RELATIVE: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 77;

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
            "aero-fixed-copy-struct-arrays-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fixed Copy-struct array workspace");
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
            .is_some_and(|name| name.starts_with("aero-fixed-copy-struct-arrays-"));
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
            "{label}: unsupported struct array compiled:\n{llvm}"
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
struct Stamp { value: int }

fn make_packet(total: int, alias: int) -> Packet {
    return Packet {
        ready: 1 < 2,
        precise: 4.5,
        ratio: 3.5,
        alias: alias,
        total: total
    };
}

fn identity(packet: Packet) -> Packet { packet }

fn iteration_score() -> int {
    let values = [make_packet(4, 6), make_packet(5, 7)];
    for item in values.iter().iter() {
        if item.ready && item.total > 4 {
            return item.alias;
        }
    }
    return 0;
}

fn main() -> int {
    let original = make_packet(10, 1);
    let returned = identity(make_packet(20, 2));
    let literal: [Packet; 3] = [
        original,
        returned,
        Packet { total: 30, alias: 3, ratio: 6.5, precise: 7.5, ready: 2 > 1 }
    ];
    let copied = literal;
    let mut second: [Packet; 3] = copied;
    let repeated: [Packet; 2] = [original; 2];
    let empty: [Packet; 0] = [];
    let stamp = [Stamp { value: 9 }][0];

    if original.ready && returned.ready && second[2].ready
        && repeated[0].ready && stamp.value == 9
        && "aero".starts_with("ae") {
        return second[0].total + copied[1].total + literal[2].total
            + repeated[1].alias + literal.len() + repeated.len()
            + empty.len() + "aero".len() + iteration_score();
    }
    return 1;
}
"#
}

fn tracked_root_source() -> String {
    positive_source()
        .replacen("\nstruct Packet", "\nmod shapes;\n\nstruct Packet", 1)
        .replacen("make_packet(10, 1)", "make_packet(6, 1)", 1)
        .replacen(
            "+ empty.len() + \"aero\".len() + iteration_score();",
            "+ empty.len() + \"aero\".len() + iteration_score() + module_score();",
            1,
        )
}

fn tracked_module_source() -> &'static str {
    r#"
struct ModuleValue { value: int }

fn module_score() -> int {
    let values = [ModuleValue { value: 1 }, ModuleValue { value: 3 }];
    let copied = values;
    return values[0].value + copied[1].value;
}
"#
}

#[test]
fn fixed_copy_struct_array_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    failures.extend(expect_success(
        "literal/repeat/empty arrays, Copy aliases, length, index, iteration, and composition",
        positive_source(),
        &[
            "%aero.struct.Packet = type { double, double, double, double, i1 }",
            "%aero.struct.Stamp = type { double }",
            "alloca [3 x %aero.struct.Packet]",
            "alloca [2 x %aero.struct.Packet]",
            "alloca [0 x %aero.struct.Packet]",
            "getelementptr inbounds [3 x %aero.struct.Packet]",
            "load %aero.struct.Packet",
            "store %aero.struct.Packet",
        ],
    ));

    for (field_type, value, use_value) in [
        ("int", "7", "return copied[0].field;"),
        ("i32", "8", "return copied[0].field;"),
        (
            "float",
            "9.5",
            "if copied[0].field > 9.0 { return 1; } return 0;",
        ),
        (
            "f64",
            "10.5",
            "if copied[0].field > 10.0 { return 1; } return 0;",
        ),
        (
            "bool",
            "1 < 2",
            "if copied[0].field { return 1; } return 0;",
        ),
    ] {
        let source = format!(
            "struct Value {{ field: {field_type} }} fn main() -> int {{ let values: [Value; 1] = [Value {{ field: {value} }}]; let copied = values; {use_value} }}"
        );
        failures.extend(expect_success(
            &format!("Copy struct array scalar alias {field_type}"),
            &source,
            &["alloca [1 x %aero.struct.Value]"],
        ));
    }

    for (label, source) in [
        (
            "repeat count zero evaluates an admitted Copy value",
            "struct Value { field: int } fn make() -> Value { return Value { field: 1 }; } fn main() -> int { let values = [make(); 0]; return values.len(); }",
        ),
        (
            "mutable exact binding and repeated aliases preserve the original",
            "struct Value { field: int } fn main() -> int { let original: [Value; 2] = [Value { field: 4 }; 2]; let first = original; let mut second: [Value; 2] = first; return original[0].field + first[1].field + second.len(); }",
        ),
        (
            "immediate literal index and projection",
            "struct Value { field: int } fn main() -> int { return [Value { field: 11 }, Value { field: 12 }][1].field; }",
        ),
        (
            "struct-bearing function contains local arrays without array ABI",
            "struct Value { field: int } fn inspect(value: Value) -> Value { let values = [value; 2]; return values[1]; } fn main() -> int { return inspect(Value { field: 13 }).field; }",
        ),
    ] {
        failures.extend(expect_success(
            label,
            source,
            &["%aero.struct.Value = type", " x %aero.struct.Value]"],
        ));
    }

    for (label, source, expected) in [
        (
            "heterogeneous exact struct names",
            "struct Left { field: int } struct Right { field: int } fn main() { let values = [Left { field: 1 }, Right { field: 2 }]; }",
            "fixed Copy-struct arrays require one exact element type",
        ),
        (
            "annotation struct-name mismatch",
            "struct Left { field: int } struct Right { field: int } fn main() { let values: [Left; 1] = [Right { field: 1 }]; }",
            "fixed Copy-struct array annotation mismatch",
        ),
        (
            "annotation count mismatch",
            "struct Value { field: int } fn main() { let values: [Value; 1] = [Value { field: 1 }, Value { field: 2 }]; }",
            "fixed Copy-struct array annotation mismatch",
        ),
        (
            "dynamic source index",
            "struct Value { field: int } fn main() { let values = [Value { field: 1 }]; let index = 0; let item = values[index]; }",
            "fixed Copy-struct array index must be a compile-time integer constant",
        ),
        (
            "negative source index",
            "struct Value { field: int } fn main() { let values = [Value { field: 1 }]; let item = values[-1]; }",
            "fixed Copy-struct array index -1 is outside 0..1",
        ),
        (
            "out-of-range source index",
            "struct Value { field: int } fn main() { let values = [Value { field: 1 }]; let item = values[1]; }",
            "fixed Copy-struct array index 1 is outside 0..1",
        ),
        (
            "typed empty source index",
            "struct Value { field: int } fn main() { let values: [Value; 0] = []; let item = values[0]; }",
            "cannot index a zero-length fixed array",
        ),
        (
            "Float source index",
            "struct Value { field: int } fn main() { let values = [Value { field: 1 }]; let item = values[0.0]; }",
            "array index type mismatch",
        ),
        (
            "String-bearing struct remains excluded",
            "struct Value { field: String } fn main() { let values = [Value { field: \"x\" }]; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "nested fixed array remains excluded",
            "struct Value { field: int } fn main() { let values = [[Value { field: 1 }]]; }",
            "only fixed numeric arrays are admitted",
        ),
        (
            "generic function context remains excluded",
            "struct Value { field: int } fn inspect<T>() { let values = [Value { field: 1 }]; } fn main() {}",
            "Struct construction expressions are not supported.",
        ),
        (
            "closure context remains excluded",
            "struct Value { field: int } fn main() { let make = |ignored: int| [Value { field: ignored }]; }",
            "Struct construction expressions are not supported.",
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
            for expected in [
                "Array {",
                "Struct {",
                "name: \"Packet\"",
                "count: 3",
                "count: 2",
                "count: 0",
                "Bool",
            ] {
                if !metadata.contains(expected) {
                    failures.push(format!(
                        "checked metadata lacks fixed Copy-struct array anchor {expected:?}: {metadata}"
                    ));
                }
            }
            if llvm.matches("alloca [3 x %aero.struct.Packet]").count() < 3 {
                failures.push(format!(
                    "array Copy aliases did not receive distinct storage:\n{llvm}"
                ));
            }
        }
    }

    let first = compile_program(positive_source(), CompilerOptions::default());
    let second = compile_program(positive_source(), CompilerOptions::default());
    if matches!((&first, &second), (Ok(left), Ok(right)) if left != right) {
        failures.push("fixed Copy-struct array LLVM was not byte-identical".to_string());
    }

    #[allow(deprecated)]
    {
        let raw_source =
            "struct Value { field: int } fn main() { let values = [Value { field: 1 }]; }";
        let tokens = try_tokenize_with_locations(raw_source, None).expect("raw control lexes");
        let ast = parse_with_locations(tokens).expect("raw control parses");
        let raw = IrGenerator::new().generate_ir(ast);
        let unchecked = compiler::generate_code(raw);
        if unchecked.contains("CheckedCopyStructArray") {
            failures.push(format!(
                "deprecated unchecked generation activated checked struct arrays:\n{unchecked}"
            ));
        }
    }

    let module_workspace = TestWorkspace::new("direct-module-product");
    let root_source = module_workspace.path("main.aero");
    let module_source = module_workspace.path("shapes.aero");
    let artifact = module_workspace.path("program.ll");
    fs::write(
        &root_source,
        "mod shapes; struct RootValue { field: int } fn main() -> int { let root = [RootValue { field: 5 }; 2]; return module_score(root[1].field); }",
    )
    .expect("write root fixed Copy-struct array source");
    fs::write(
        &module_source,
        "struct ModuleValue { field: int } fn module_score(base: int) -> int { let values: [ModuleValue; 2] = [ModuleValue { field: 7 }, ModuleValue { field: 8 }]; let copied = values; return base + values[0].field + copied[1].field; }",
    )
    .expect("write module fixed Copy-struct array source");
    let check = run_cli(&module_workspace, &[Path::new("check"), &root_source]);
    if !check.status.success() {
        failures.push(format!(
            "direct-module fixed Copy-struct array check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &module_workspace,
        &[Path::new("build"), &root_source, Path::new("-o"), &artifact],
    );
    if !build.status.success() {
        failures.push(format!(
            "direct-module fixed Copy-struct array build failed: {}",
            output_text(&build)
        ));
    }
    if !artifact.is_file() {
        failures.push("direct-module fixed Copy-struct array build omitted LLVM".to_string());
    }

    let invalid_workspace = TestWorkspace::new("invalid-no-artifact");
    let invalid_source = invalid_workspace.path("main.aero");
    let invalid_artifact = invalid_workspace.path("program.ll");
    fs::write(
        &invalid_source,
        "struct Value { field: int } fn main() { let values = [Value { field: 1 }]; let index = 0; let item = values[index]; }",
    )
    .expect("write invalid fixed Copy-struct array source");
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
            "invalid fixed Copy-struct array build exited zero: {}",
            output_text(&invalid_build)
        ));
    }
    if invalid_artifact.exists() {
        failures.push("invalid fixed Copy-struct array build published an artifact".to_string());
    }

    let root = repository_root();
    let example_root = root.join(EXAMPLE_ROOT_RELATIVE);
    match fs::read_to_string(&example_root) {
        Err(error) => failures.push(format!(
            "tracked CORE-045 root example is unavailable at {}: {error}",
            example_root.display()
        )),
        Ok(source) if source != tracked_root_source() => failures.push(format!(
            "tracked CORE-045 root example bytes differ from tested source at {}",
            example_root.display()
        )),
        Ok(_) => {}
    }
    let example_module = root.join(EXAMPLE_MODULE_RELATIVE);
    match fs::read_to_string(&example_module) {
        Err(error) => failures.push(format!(
            "tracked CORE-045 module example is unavailable at {}: {error}",
            example_module.display()
        )),
        Ok(source) if source != tracked_module_source() => failures.push(format!(
            "tracked CORE-045 module example bytes differ from tested source at {}",
            example_module.display()
        )),
        Ok(_) => {}
    }

    let workflow = root.join(WORKFLOW_RELATIVE);
    match fs::read_to_string(&workflow) {
        Err(error) => failures.push(format!("cannot read {}: {error}", workflow.display())),
        Ok(workflow) => {
            for anchor in [
                "Test fixed Copy-struct array integration example",
                "examples/fixed_copy_struct_arrays/main.aero",
                "fixed_copy_struct_arrays.ll",
                "opt-22 -passes=verify -disable-output",
                "llc-22 -verify-machineinstrs",
                "llc-22 -filetype=obj",
                "clang-22",
                "fixed Copy-struct array example passed with exit code 77",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "Rust workflow is missing CORE-045 anchor {anchor:?}"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-045 fixed Copy-struct array failures (expected native sentinel {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
