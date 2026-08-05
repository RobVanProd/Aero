use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_RELATIVE: &str = "examples/monomorphic_scalar_structs.aero";
const WORKFLOW_RELATIVE: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 53;

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
            "aero-struct-execution-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create struct execution workspace");
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
            .is_some_and(|name| name.starts_with("aero-struct-execution-"));
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

fn compile_failure(source: &str) -> Option<String> {
    compile_program(source, CompilerOptions::default()).err()
}

fn expect_public_success(label: &str, source: &str, required: &[&str]) -> Vec<String> {
    match compile_program(source, CompilerOptions::default()) {
        Err(error) => vec![format!("{label}: expected success, got {error}")],
        Ok(llvm) => required
            .iter()
            .filter(|fragment| !llvm.contains(**fragment))
            .map(|fragment| format!("{label}: LLVM missing {fragment:?}\n{llvm}"))
            .collect(),
    }
}

fn expect_public_rejection(label: &str, source: &str, expected: &str) -> Option<String> {
    match compile_failure(source) {
        None => Some(format!("{label}: unsupported struct source compiled")),
        Some(error) if error.contains(expected) => None,
        Some(error) => Some(format!(
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
struct ScalarAliases {
    first: int,
    second: i32,
    ratio: float,
    precise: f64,
    ready: bool
}
struct Single { value: int }

fn projected() -> int {
    let record: ScalarAliases = ScalarAliases {
        ready: 1 < 2,
        precise: 4.5,
        ratio: 3.5,
        second: 2,
        first: 40
    };
    let mut another = ScalarAliases {
        first: 1,
        second: 2,
        ratio: 3.0,
        precise: 4.0,
        ready: 2 > 1
    };
    ScalarAliases {
        first: 0,
        second: 0,
        ratio: 0.0,
        precise: 0.0,
        ready: 1 < 2
    };
    let direct: int = Single { value: 3 }.value;
    let numeric: float = record.ratio + record.precise;
    if record.ready && another.ready && numeric > 7.0 {
        return record.first + record.second + direct + another.first + another.second + 5;
    }
    return 1;
}

fn main() -> int {
    return projected();
}
"#
}

#[test]
fn monomorphic_scalar_struct_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    failures.extend(expect_public_success(
        "all aliases, both mutabilities, reordered construction, discard, and projection",
        positive_source(),
        &[
            "%aero.struct.ScalarAliases = type { double, double, double, double, i1 }",
            "%aero.struct.Single = type { double }",
            "alloca %aero.struct.ScalarAliases",
            "getelementptr inbounds %aero.struct.ScalarAliases",
            "getelementptr inbounds %aero.struct.Single",
            "load i1",
            "load double",
        ],
    ));

    let one_field_aliases = [
        ("int", "7"),
        ("i32", "8"),
        ("float", "9.5"),
        ("f64", "10.5"),
        ("bool", "1 < 2"),
    ];
    for (field_type, value) in one_field_aliases {
        let projection = if field_type == "bool" {
            "if value.field { return 1; } return 0;".to_string()
        } else if matches!(field_type, "float" | "f64") {
            "if value.field > 0.0 { return 1; } return 0;".to_string()
        } else {
            "return value.field;".to_string()
        };
        let source = format!(
            "struct Value {{ field: {field_type} }} fn main() -> int {{ let value = Value {{ field: {value} }}; {projection} }}"
        );
        failures.extend(expect_public_success(
            &format!("single-field alias {field_type}"),
            &source,
            &[
                "%aero.struct.Value = type",
                "getelementptr inbounds %aero.struct.Value",
            ],
        ));
    }

    failures.extend(expect_public_success(
        "recursive tuple-valued struct field",
        "struct Value { field: (int, int) } fn main() -> int { let value = Value { field: (1, 2) }; (value.field).1 }",
        &[
            "%aero.struct.Value = type { { double, double } }",
            "getelementptr inbounds %aero.struct.Value",
            "getelementptr inbounds { double, double }",
        ],
    ));

    let declaration_only = [
        "struct Empty {} fn main() {}",
        "struct Generic<T> { value: T } fn main() {}",
        "struct Text { value: String } fn main() {}",
        "struct Nested { value: [int; 2] } fn main() {}",
    ];
    for source in declaration_only {
        match compile_program(source, CompilerOptions::default()) {
            Err(error) => failures.push(format!(
                "unsupported declaration must remain syntax-only until constructed: {error}"
            )),
            Ok(llvm) if llvm.contains("aero.struct") => failures.push(format!(
                "unsupported declaration emitted runtime struct type:\n{llvm}"
            )),
            Ok(_) => {}
        }
    }

    let negative_cases = [
        (
            "unknown definition",
            "fn main() { let value = Ghost { field: 1 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "ambiguous definition",
            "struct Value { field: int } struct Value { field: int } fn main() { let value = Value { field: 1 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "generic definition",
            "struct Value<T> { field: T } fn main() { let value = Value { field: 1 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "duplicate declared field",
            "struct Value { field: int, field: int } fn main() { let value = Value { field: 1 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "String field",
            "struct Value { field: String } fn main() { let value = Value { field: \"x\" }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "custom field",
            "struct Value { field: Missing } fn main() { let value = Value { field: 1 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "reference field",
            "struct Value { field: &int } fn main() { let base: int = 1; let value = Value { field: &base }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "missing construction field",
            "struct Pair { first: int, second: int } fn main() { let value = Pair { first: 1 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "extra construction field",
            "struct Value { field: int } fn main() { let value = Value { field: 1, extra: 2 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "duplicate construction field",
            "struct Value { field: int } fn main() { let value = Value { field: 1, field: 2 }; }",
            "Struct construction expressions are not supported.",
        ),
        (
            "wrong Int field type",
            "struct Value { field: int } fn main() { let value = Value { field: 1.5 }; }",
            "struct `Value` field `field` type mismatch: expected int, actual float",
        ),
        (
            "wrong Float field type",
            "struct Value { field: float } fn main() { let value = Value { field: 1 }; }",
            "struct `Value` field `field` type mismatch: expected float, actual int",
        ),
        (
            "wrong Bool field type",
            "struct Value { field: bool } fn main() { let value = Value { field: 1 }; }",
            "struct `Value` field `field` type mismatch: expected bool, actual int",
        ),
        (
            "wrong custom annotation",
            "struct Value { field: int } struct Other { field: int } fn main() { let value: Other = Value { field: 1 }; }",
            "struct binding annotation mismatch: expected Value, actual Other",
        ),
        (
            "unknown projected field",
            "struct Value { field: int } fn main() { let value = Value { field: 1 }; let result: int = value.missing; }",
            "struct `Value` has no field `missing`",
        ),
        (
            "non-struct projected field",
            "fn main() { let value: int = 1; let result: int = value.field; }",
            "Field access expressions are not supported.",
        ),
        (
            "root construction",
            "struct Value { field: int } Value { field: 1 }; fn main() {}",
            "Struct construction expressions are not supported.",
        ),
        (
            "generic function construction",
            "struct Value { field: int } fn make<T>() { let value = Value { field: 1 }; } fn main() {}",
            "Struct construction expressions are not supported.",
        ),
        (
            "impl construction",
            "struct Value { field: int } impl Value { fn make() { let value = Value { field: 1 }; } } fn main() {}",
            "Struct construction expressions are not supported.",
        ),
        (
            "trait default construction",
            "struct Value { field: int } trait Make { fn make(&self) { let value = Value { field: 1 }; } } fn main() {}",
            "Struct construction expressions are not supported.",
        ),
        (
            "nested function construction",
            "struct Value { field: int } fn outer() { fn inner() { let value = Value { field: 1 }; } } fn main() {}",
            "Struct construction expressions are not supported.",
        ),
        (
            "closure parent fails closed before struct construction",
            "struct Value { field: int } fn main() { let make = |x: int| Value { field: x }; }",
            "closure expressions are parsed but unsupported in executable code",
        ),
        (
            "struct condition",
            "struct Value { field: int } fn main() { if Value { field: 1 } { return; } }",
            "Error: If condition must be boolean, found: Value",
        ),
        (
            "struct index receiver",
            "struct Value { field: int } fn main() { let result = Value { field: 1 }[0]; }",
            "Cannot index into non-array type",
        ),
        (
            "struct comparison",
            "struct Value { field: int } fn main() { let same = Value { field: 1 } == Value { field: 1 }; }",
            "comparison operand types are not admitted; expected numeric operands or Bool with Bool",
        ),
        (
            "struct enum payload",
            "struct Value { field: int } enum Choice { Item(Value) } fn main() { let choice = Choice::Item(Value { field: 1 }); }",
            "enum `Choice` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "struct Option payload",
            "struct Value { field: int } fn main() { let value = Some(Value { field: 1 }); }",
            "enum construction is not admitted in checked IR",
        ),
        (
            "struct print argument",
            "struct Value { field: int } fn main() { println!(\"{}\", Value { field: 1 }); }",
            "Error: Argument 1 of type `Value` is not printable.",
        ),
    ];

    for (label, source, expected) in negative_cases {
        if let Some(failure) = expect_public_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    let child_precedence = [
        (
            "admitted first tuple field advances to second source child",
            "struct Pair { first: int, second: int } fn main() { let value = Pair { first: (1, 2), second: missing }; }",
            "Error: Use of undeclared variable `missing`.",
        ),
        (
            "written second field is first evaluated child",
            "struct Pair { first: int, second: int } fn main() { let value = Pair { second: missing, first: (1, 2) }; }",
            "Error: Use of undeclared variable `missing`.",
        ),
    ];
    for (label, source, expected) in child_precedence {
        if let Some(failure) = expect_public_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked metadata specimen failed: {error}")),
        Ok((checked, llvm)) => {
            let metadata_debug = format!("{:#?}", checked.metadata());
            let struct_places = metadata_debug.matches("Struct {").count();
            if struct_places < 3 {
                failures.push(format!(
                    "checked metadata recorded {struct_places} struct places, expected at least 3: {:#?}",
                    checked.metadata()
                ));
            }
            for expected in ["name: \"ScalarAliases\"", "Int", "Float", "Bool"] {
                if !metadata_debug.contains(expected) {
                    failures.push(format!(
                        "checked metadata lacks struct schema anchor {expected:?}: {metadata_debug}"
                    ));
                }
            }
            let scalar_position = llvm.find("%aero.struct.ScalarAliases = type");
            let single_position = llvm.find("%aero.struct.Single = type");
            if !matches!((scalar_position, single_position), (Some(left), Some(right)) if left < right)
            {
                failures.push(format!(
                    "LLVM struct definitions are absent or not exact-name sorted:\n{llvm}"
                ));
            }
        }
    }

    #[allow(deprecated)]
    {
        let raw_source =
            "struct Value { field: int } fn main() { let value = Value { field: 1 }; }";
        let tokens = try_tokenize_with_locations(raw_source, None).expect("raw control lexes");
        let ast = parse_with_locations(tokens).expect("raw control parses");
        let raw = IrGenerator::new().generate_ir(ast);
        let unchecked = compiler::generate_code(raw);
        if unchecked.contains("aero.struct") || unchecked.contains("getelementptr inbounds %aero") {
            failures.push(format!(
                "deprecated unchecked generation changed its legacy struct behavior:\n{unchecked}"
            ));
        }
    }

    let module_workspace = TestWorkspace::new("direct-module-positive");
    let root = module_workspace.path("main.aero");
    let module = module_workspace.path("shapes.aero");
    let artifact = module_workspace.path("program.ll");
    fs::write(
        &root,
        "mod shapes; fn main() -> int { return module_projection(); }",
    )
    .expect("write root struct integration source");
    fs::write(
        &module,
        "struct ModuleValue { count: int, ready: bool } fn module_projection() -> int { let value = ModuleValue { ready: 1 < 2, count: 53 }; if value.ready { return value.count; } return 1; }",
    )
    .expect("write direct module struct integration source");
    let check = run_cli(&module_workspace, &[Path::new("check"), &root]);
    if !check.status.success() {
        failures.push(format!(
            "direct-module struct check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &module_workspace,
        &[Path::new("build"), &root, Path::new("-o"), &artifact],
    );
    if !build.status.success() {
        failures.push(format!(
            "direct-module struct build failed: {}",
            output_text(&build)
        ));
    }
    if !artifact.is_file() {
        failures.push("direct-module struct build did not publish LLVM artifact".to_string());
    }

    let invalid_workspace = TestWorkspace::new("invalid-no-artifact");
    let invalid_source = invalid_workspace.path("main.aero");
    let invalid_artifact = invalid_workspace.path("program.ll");
    fs::write(
        &invalid_source,
        "struct Value { field: int } fn main() { let value = Value { field: 1.5 }; }",
    )
    .expect("write invalid struct source");
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
            "invalid struct build exited zero: {}",
            output_text(&invalid_build)
        ));
    }
    if invalid_artifact.exists() {
        failures.push("invalid struct build published an artifact".to_string());
    }

    let root = repository_root();
    let example = root.join(EXAMPLE_RELATIVE);
    match fs::read_to_string(&example) {
        Err(error) => failures.push(format!(
            "tracked CORE-043 example is unavailable at {}: {error}",
            example.display()
        )),
        Ok(source) if source != positive_source() => failures.push(format!(
            "tracked CORE-043 example bytes differ from the tested source at {}",
            example.display()
        )),
        Ok(_) => {}
    }

    let workflow = root.join(WORKFLOW_RELATIVE);
    match fs::read_to_string(&workflow) {
        Err(error) => failures.push(format!("cannot read {}: {error}", workflow.display())),
        Ok(workflow) => {
            for anchor in [
                "Test monomorphic scalar struct integration example",
                "examples/monomorphic_scalar_structs.aero",
                "monomorphic_scalar_structs.ll",
                "opt-22 -passes=verify -disable-output",
                "llc-22 -verify-machineinstrs",
                "llc-22 -filetype=obj",
                "clang-22",
                "struct example passed with exit code 53",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "Rust workflow is missing CORE-043 anchor {anchor:?}"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-043 struct class failures (expected native sentinel {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
