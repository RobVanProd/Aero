use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT_RELATIVE: &str = "examples/acyclic_copy_aggregates/main.aero";
const EXAMPLE_MODULE_RELATIVE: &str = "examples/acyclic_copy_aggregates/shapes.aero";
const WORKFLOW_RELATIVE: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 107;

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
            "aero-acyclic-copy-aggregates-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create acyclic Copy-aggregate workspace");
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
            .is_some_and(|name| name.starts_with("aero-acyclic-copy-aggregates-"));
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
            "{label}: unsupported Copy-aggregate source compiled:\n{llvm}"
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
struct Bundle {
    payload: Payload,
    samples: [Sample; 2],
    none: [Sample; 0],
    numbers: [int; 3],
    ratios: [float; 2],
    active: bool
}
struct Pair { left: Sample, right: Sample }
struct Payload { current: Sample, history: [Sample; 1] }
struct Sample { value: int, ratio: float, ready: bool }

fn sample(value: int) -> Sample {
    return Sample { ready: value > 0, ratio: 1.5, value: value };
}
fn mark_one() -> Sample { return sample(1); }
fn mark_two() -> Sample { return sample(2); }
fn ordered() -> Pair { return Pair { right: mark_two(), left: mark_one() }; }

fn make_bundle(seed: int) -> Bundle {
    let base = sample(seed);
    let payload = Payload { history: [base; 1], current: base };
    let rows = [base, sample(seed + 1)];
    return Bundle {
        active: seed > 0,
        ratios: [1.5, 2.5],
        numbers: [3, 4, 5],
        none: [],
        samples: rows,
        payload: payload
    };
}

fn identity(value: Bundle) -> Bundle { value }
fn rows(value: Bundle) -> [Sample; 2] { return value.samples; }
fn pair(value: Bundle) -> [Bundle; 2] { [value; 2] }
fn forward(value: Bundle) -> Bundle { return declared_later(value); }
fn declared_later(value: Bundle) -> Bundle { value }
fn recurse(value: Bundle, remaining: int) -> Bundle {
    if remaining > 0 { return recurse(value, remaining - 1); }
    return value;
}
fn scan(value: Bundle) -> int {
    let projected = value.samples;
    for item in projected.iter() {
        if item.value > 10 { return item.value; }
    }
    return projected.len();
}

fn main() -> int {
    let original = make_bundle(10);
    let copied: Bundle = original;
    let transported = identity(copied);
    let forwarded = forward(transported);
    let recursive = recurse(forwarded, 2);
    let projected = rows(recursive);
    let repeated = pair(original);
    let order = ordered();
    if original.payload.current.value == 10
        && copied.payload.history[0].value == 10
        && transported.samples[1].value == 11
        && recursive.none.len() == 0
        && projected.len() == 2
        && projected[0].ready
        && repeated[1].payload.current.value == 10
        && original.numbers[2] == 5
        && original.ratios[1] > 2.0
        && scan(original) == 11
        && order.left.value == 1
        && order.right.value == 2
        && "aero".starts_with("ae") {
        return 107;
    }
    return 1;
}
"#
}

fn tracked_root_source() -> &'static str {
    r#"mod shapes;

struct RootEnvelope { leaf: ModuleLeaf, leaves: [ModuleLeaf; 2], none: [ModuleLeaf; 0] }

fn root_roundtrip(value: RootEnvelope) -> RootEnvelope { value }
fn root_array_roundtrip(values: [RootEnvelope; 2]) -> [RootEnvelope; 2] { values }

fn main() -> int {
    let module = module_roundtrip(module_make(20));
    let root = RootEnvelope { none: [], leaves: module.leaves, leaf: module.leaf };
    let copied = root_roundtrip(root);
    let values = root_array_roundtrip([copied; 2]);
    if root.leaf.value == 20 && copied.leaves[1].value == 21
        && values[1].leaf.ready && values[0].none.len() == 0
        && module_scan(module) == 2 && "aero".len() == 4 {
        return 107;
    }
    return 1;
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"struct ModuleEnvelope { leaf: ModuleLeaf, leaves: [ModuleLeaf; 2] }
struct ModuleLeaf { value: int, ready: bool }

fn module_make(value: int) -> ModuleEnvelope {
    let leaf = ModuleLeaf { ready: value > 0, value: value };
    return ModuleEnvelope {
        leaves: [leaf, ModuleLeaf { value: value + 1, ready: 1 < 2 }],
        leaf: leaf
    };
}
fn module_roundtrip(value: ModuleEnvelope) -> ModuleEnvelope { return value; }
fn module_scan(value: ModuleEnvelope) -> int {
    let leaves = value.leaves;
    for leaf in leaves.iter() {
        if !leaf.ready { return 0; }
    }
    return leaves.len();
}
"#
}

#[test]
fn acyclic_copy_aggregate_class_is_complete_and_executable() {
    let mut failures = Vec::new();

    failures.extend(expect_success(
        "forward/deep definitions, construction, Copy projection, transport, arrays, and operations",
        positive_source(),
        &[
            "%aero.struct.Sample = type { double, double, i1 }",
            "%aero.struct.Payload = type { %aero.struct.Sample, [1 x %aero.struct.Sample] }",
            "%aero.struct.Bundle = type { %aero.struct.Payload, [2 x %aero.struct.Sample], [0 x %aero.struct.Sample], [3 x double], [2 x double], i1 }",
            "define %aero.struct.Bundle @identity(%aero.struct.Bundle %aero.arg.value)",
            "define [2 x %aero.struct.Bundle] @pair(%aero.struct.Bundle %aero.arg.value)",
            "getelementptr inbounds %aero.struct.Bundle",
            "load %aero.struct.Payload",
            "store [2 x %aero.struct.Sample]",
            "ret %aero.struct.Bundle",
        ],
    ));

    for (label, source, llvm_fragments) in [
        (
            "direct nested Copy field at arbitrary finite depth",
            "struct A { value: int } struct B { a: A } struct C { b: B } struct D { c: C } fn main() -> int { let value = D { c: C { b: B { a: A { value: 7 } } } }; let copied = value; return copied.c.b.a.value + value.c.b.a.value; }",
            &[
                "%aero.struct.D = type { %aero.struct.C }",
                "load %aero.struct.C",
            ][..],
        ),
        (
            "forward reference through zero and nonzero struct arrays",
            "struct Outer { empty: [Inner; 0], values: [Inner; 2] } struct Inner { value: int } fn main() -> int { let value = Outer { values: [Inner { value: 3 }; 2], empty: [] }; return value.values[1].value + value.empty.len(); }",
            &["[0 x %aero.struct.Inner]", "[2 x %aero.struct.Inner]"][..],
        ),
        (
            "numeric array fields and projection into existing operations",
            "struct Value { ints: [int; 3], floats: [f64; 2] } fn main() -> int { let value = Value { floats: [1.5, 2.5], ints: [4, 5, 6] }; let ints = value.ints; if value.floats[1] > 2.0 { return ints[2] + value.ints.len(); } return 0; }",
            &["[3 x double]", "[2 x double]"][..],
        ),
        (
            "aggregate field and array values from calls and projections",
            "struct Inner { value: int } struct Outer { inner: Inner, values: [Inner; 1] } fn make(value: int) -> Inner { Inner { value: value } } fn take(value: Outer) -> Outer { value } fn main() -> int { let first = Outer { values: [make(8)], inner: make(7) }; let second = Outer { inner: first.inner, values: first.values }; return take(second).values[0].value; }",
            &[
                "call %aero.struct.Inner @make",
                "call %aero.struct.Outer @take",
            ][..],
        ),
    ] {
        failures.extend(expect_success(label, source, llvm_fragments));
    }

    for (label, source, expected) in [
        (
            "nested field exact name mismatch",
            "struct Inner { value: int } struct Other { value: int } struct Outer { inner: Inner } fn main() -> int { let value = Outer { inner: Other { value: 1 } }; return 0; }",
            "struct `Outer` field `inner` type mismatch: expected Inner, actual Other",
        ),
        (
            "numeric array field count mismatch",
            "struct Value { field: [int; 2] } fn main() -> int { let value = Value { field: [1] }; return 0; }",
            "struct `Value` field `field` type mismatch: expected [int; 2], actual [int; 1]",
        ),
        (
            "struct array field element mismatch",
            "struct Left { value: int } struct Right { value: int } struct Holder { values: [Left; 1] } fn main() -> int { let value = Holder { values: [Right { value: 1 }] }; return 0; }",
            "struct `Holder` field `values` type mismatch: expected [Left; 1], actual [Right; 1]",
        ),
        (
            "deep function argument mismatch",
            "struct Leaf { value: int } struct Left { leaf: Leaf } struct Right { leaf: Leaf } fn take(value: Left) -> int { return value.leaf.value; } fn main() -> int { return take(Right { leaf: Leaf { value: 1 } }); }",
            "parameter `value` type mismatch: expected Left, actual Right",
        ),
        (
            "deep function return mismatch",
            "struct Leaf { value: int } struct Left { leaf: Leaf } struct Right { leaf: Leaf } fn make() -> Left { return Right { leaf: Leaf { value: 1 } }; } fn main() -> int { return 0; }",
            "return type mismatch: expected Left, actual Right",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    for (label, source) in [
        (
            "self cycle",
            "struct Node { next: Node } fn take(value: Node) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "mutual forward cycle",
            "struct Left { right: Right } struct Right { left: Left } fn take(value: Left) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "zero-array-mediated cycle",
            "struct Loop { values: [Loop; 0] } fn take(value: Loop) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "dependent on cycle",
            "struct Cycle { next: Cycle } struct Parent { child: Cycle } fn take(value: Parent) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "direct nested array field",
            "struct Value { field: [[int; 1]; 1] } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "Bool array field",
            "struct Value { field: [bool; 1] } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "String field",
            "struct Value { field: String } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "unknown field dependency",
            "struct Value { field: Missing } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "generic dependency",
            "struct Generic<T> { field: T } struct Value { field: Generic } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "ambiguous dependency",
            "struct Inner { field: int } struct Inner { field: int } struct Value { field: Inner } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
        (
            "empty dependency",
            "struct Empty {} struct Value { field: Empty } fn take(value: Value) -> int { return 0; } fn main() -> int { return 0; }",
        ),
    ] {
        if let Some(failure) = expect_rejection(
            label,
            source,
            "function parameter `value` is not an admitted scalar type",
        ) {
            failures.push(failure);
        }
    }

    for source in [
        "struct SelfCycle { value: SelfCycle } fn main() -> int { return 0; }",
        "struct Left { right: Right } struct Right { left: Left } fn main() -> int { return 0; }",
        "struct NestedArray { value: [[int; 1]; 1] } fn main() -> int { return 0; }",
        "struct BoolArray { value: [bool; 1] } fn main() -> int { return 0; }",
        "struct Resource { value: String } fn main() -> int { return 0; }",
    ] {
        match compile_program(source, CompilerOptions::default()) {
            Err(error) => failures.push(format!(
                "excluded declaration must remain syntax-only until used: {error}"
            )),
            Ok(llvm) if llvm.contains("aero.struct") => failures.push(format!(
                "excluded declaration emitted a runtime struct schema:\n{llvm}"
            )),
            Ok(_) => {}
        }
    }

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!(
            "recursive checked metadata specimen failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let metadata = format!("{:#?}", checked.metadata());
            for anchor in [
                "name: \"Bundle\"",
                "name: \"Payload\"",
                "name: \"Sample\"",
                "Array {",
                "count: 0",
                "count: 2",
                "element: Int",
                "element: Float",
            ] {
                if !metadata.contains(anchor) {
                    failures.push(format!(
                        "checked metadata lacks recursive aggregate anchor {anchor:?}: {metadata}"
                    ));
                }
            }

            let Some(start) = llvm.find("define %aero.struct.Pair @ordered(") else {
                failures.push(format!("LLVM omitted ordered aggregate function:\n{llvm}"));
                assert!(failures.is_empty(), "{}", failures.join("\n\n"));
                return;
            };
            let ordered_body = &llvm[start..];
            let mark_two = ordered_body.find("call %aero.struct.Sample @mark_two(");
            let mark_one = ordered_body.find("call %aero.struct.Sample @mark_one(");
            if !matches!((mark_two, mark_one), (Some(two), Some(one)) if two < one) {
                failures.push(format!(
                    "written struct-field evaluation order was not preserved:\n{ordered_body}"
                ));
            }
        }
    }

    #[allow(deprecated)]
    {
        let raw_source = "struct Inner { value: int } struct Outer { inner: Inner } fn main() -> int { let value = Outer { inner: Inner { value: 1 } }; return value.inner.value; }";
        let tokens = try_tokenize_with_locations(raw_source, None).expect("raw control lexes");
        let ast = parse_with_locations(tokens).expect("raw control parses");
        let raw = IrGenerator::new().generate_ir(ast);
        let unchecked = compiler::generate_code(raw);
        if unchecked.contains("%aero.struct.Outer = type") {
            failures.push(format!(
                "deprecated unchecked generation activated checked aggregate closure:\n{unchecked}"
            ));
        }
    }

    let module_workspace = TestWorkspace::new("direct-module-product");
    let root_source = module_workspace.path("main.aero");
    let module_source = module_workspace.path("shapes.aero");
    let artifact = module_workspace.path("program.ll");
    fs::write(&root_source, tracked_root_source()).expect("write root aggregate source");
    fs::write(&module_source, tracked_module_source()).expect("write module aggregate source");
    let check = run_cli(&module_workspace, &[Path::new("check"), &root_source]);
    if !check.status.success() {
        failures.push(format!(
            "direct-module acyclic aggregate check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &module_workspace,
        &[Path::new("build"), &root_source, Path::new("-o"), &artifact],
    );
    if !build.status.success() {
        failures.push(format!(
            "direct-module acyclic aggregate build failed: {}",
            output_text(&build)
        ));
    }
    if !artifact.is_file() {
        failures.push("direct-module acyclic aggregate build omitted LLVM".to_string());
    }

    let invalid_workspace = TestWorkspace::new("invalid-no-artifact");
    let invalid_source = invalid_workspace.path("main.aero");
    let invalid_artifact = invalid_workspace.path("program.ll");
    fs::write(
        &invalid_source,
        "struct Loop { values: [Loop; 0] } fn take(value: Loop) -> int { return 0; } fn main() -> int { return 0; }",
    )
    .expect("write cyclic aggregate source");
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
            "cyclic aggregate build exited zero: {}",
            output_text(&invalid_build)
        ));
    }
    if invalid_artifact.exists() {
        failures.push("cyclic aggregate build published an artifact".to_string());
    }

    let root = repository_root();
    for (relative, expected_source) in [
        (EXAMPLE_ROOT_RELATIVE, tracked_root_source()),
        (EXAMPLE_MODULE_RELATIVE, tracked_module_source()),
    ] {
        let path = root.join(relative);
        match fs::read_to_string(&path) {
            Err(error) => failures.push(format!(
                "tracked CORE-047 example is unavailable at {}: {error}",
                path.display()
            )),
            Ok(source) if source != expected_source => failures.push(format!(
                "tracked CORE-047 example bytes differ from tested source at {}",
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
                "Test acyclic Copy-aggregate integration example",
                "examples/acyclic_copy_aggregates/main.aero",
                "acyclic_copy_aggregates.ll",
                "opt-22 -passes=verify -disable-output",
                "llc-22 -verify-machineinstrs",
                "llc-22 -filetype=obj",
                "clang-22",
                "acyclic Copy-aggregate example passed with exit code 107",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "Rust workflow is missing CORE-047 anchor {anchor:?}"
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-047 acyclic Copy-aggregate failures (expected native sentinel {EXPECTED_EXIT}):\n{}",
        failures.join("\n\n")
    );
}
