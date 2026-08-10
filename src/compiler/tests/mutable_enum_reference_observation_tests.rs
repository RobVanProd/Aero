use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 86;
const EXAMPLE_ROOT: &str = "examples/mutable_enum_reference_observation/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_enum_reference_observation/observations.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";

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
            "aero-mutable-enum-reference-observation-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CORE-086 test workspace");
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
            .is_some_and(|name| name.starts_with("aero-mutable-enum-reference-observation-"));
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

fn parsed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source)?)
        .map(|(_, ast)| ast)
}

fn expect_acceptance_in_both_paths(label: &str, source: &str, failures: &mut Vec<String>) {
    let analyzed = match analyzed(source) {
        Ok(ast) => ast,
        Err(error) => {
            failures.push(format!("{label}: semantic analysis failed: {error}"));
            return;
        }
    };
    for (route, ast) in [
        ("analyzed", analyzed),
        (
            "direct",
            match parsed(source) {
                Ok(ast) => ast,
                Err(error) => {
                    failures.push(format!("{label}: direct source did not parse: {error}"));
                    return;
                }
            },
        ),
    ] {
        let checked = match IrGenerator::new().try_generate_ir(ast) {
            Ok(checked) => checked,
            Err(error) => {
                failures.push(format!(
                    "{label}: {route} checked admission failed: {error}"
                ));
                continue;
            }
        };
        let debug = format!("{checked:#?}");
        if !debug.contains("CheckedMutableEnumMatchRead") {
            failures.push(format!(
                "{label}: {route} checked IR omitted the mutable read identity:\n{debug}"
            ));
        }
        if let Err(error) = CodeGenerator::new().try_generate_code(checked) {
            failures.push(format!("{label}: {route} verified LLVM failed: {error}"));
        }
    }
}

fn expect_rejection(source: &str, expected: &[&str]) -> Result<(), String> {
    let semantic = match analyzed(source) {
        Ok(_) => return Err("excluded CORE-086 source passed semantics".to_string()),
        Err(error) => error,
    };
    if !expected.iter().any(|fragment| semantic.contains(fragment)) {
        return Err(format!(
            "semantic diagnostic {semantic:?} contained none of {expected:?}"
        ));
    }
    let admission = match IrGenerator::new().try_generate_ir(parsed(source)?) {
        Ok(_) => return Err("excluded CORE-086 source passed checked admission".to_string()),
        Err(error) => error.to_string(),
    };
    if !expected.iter().any(|fragment| admission.contains(fragment)) {
        return Err(format!(
            "checked diagnostic {admission:?} contained none of {expected:?}"
        ));
    }
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Err(format!("excluded CORE-086 source emitted LLVM:\n{llvm}")),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => Ok(()),
        Err(error) => Err(format!(
            "compile diagnostic {error:?} contained none of {expected:?}"
        )),
    }
}

fn origin_and_order_source() -> &'static str {
    r#"
struct Leaf { value: int, ready: bool }

enum Unit { Idle, Ready }
enum Scalar { Empty, Count(int), Flag(bool) }
enum Packet { Empty, Pair(int, bool), Leaf(Leaf) }

fn inspect_unit(value: &mut Unit) -> int {
    let before = match *value { Unit::Idle => 1, Unit::Ready => 2 };
    *value = Unit::Ready;
    let after = match *value { Unit::Idle => 10, Unit::Ready => 20 };
    before + after
}

fn inspect_scalar(value: &mut Scalar) -> int {
    match *value {
        Scalar::Empty => 0,
        Scalar::Count(inner) => inner,
        Scalar::Flag(inner) => 3
    }
}

fn inspect_packet(value: &mut Packet) -> int {
    let before = match *value {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => number,
        Packet::Leaf(leaf) => leaf.value
    };
    *value = Packet::Pair(17, 1 < 2);
    let after = match *value {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => number,
        Packet::Leaf(leaf) => leaf.value
    };
    before + after
}

fn main() -> int {
    let mut unit = Unit::Idle;
    let mut scalar = Scalar::Count(5);
    let mut packet = Packet::Leaf(Leaf { value: 7, ready: 1 < 2 });
    let mut observed = inspect_unit(&mut unit);

    {
        let scalar_ref: &mut Scalar = &mut scalar;
        observed = observed + inspect_scalar(scalar_ref);
        observed = observed + match *scalar_ref {
            Scalar::Empty => 0,
            Scalar::Count(inner) => inner,
            Scalar::Flag(inner) => 3
        };
    }

    {
        let packet_ref = &mut packet;
        observed = observed + inspect_packet(packet_ref);
        observed = observed + match *packet_ref {
            Packet::Empty => 0,
            Packet::Pair(number, ready) => number,
            Packet::Leaf(leaf) => leaf.value
        };
    }

    match packet {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => observed + number,
        Packet::Leaf(leaf) => leaf.value
    }
}
"#
}

fn result_product_source() -> &'static str {
    r#"
struct View { count: int, ready: bool }

enum Gate { Left, Right }

fn choose_int(value: &mut Gate) -> int {
    match *value { Gate::Left => 7, Gate::Right => 11 }
}
fn choose_float(value: &mut Gate) -> float {
    match *value { Gate::Left => 1.5, Gate::Right => 2.5 }
}
fn choose_bool(value: &mut Gate) -> bool {
    match *value { Gate::Left => 1 < 2, Gate::Right => 1 > 2 }
}
fn choose_char(value: &mut Gate) -> char {
    match *value { Gate::Left => 'a', Gate::Right => 'z' }
}
fn choose_array(value: &mut Gate) -> [int; 2] {
    match *value { Gate::Left => [3, 4], Gate::Right => [5, 6] }
}
fn choose_empty(value: &mut Gate) -> [int; 0] {
    let empty: [int; 0] = [];
    match *value { Gate::Left => empty, Gate::Right => empty }
}
fn choose_tuple(value: &mut Gate) -> ((char, bool), [float; 2]) {
    match *value {
        Gate::Left => (('x', 1 < 2), [1.5, 2.5]),
        Gate::Right => (('y', 1 > 2), [3.5, 4.5])
    }
}
fn choose_struct(value: &mut Gate) -> View {
    match *value {
        Gate::Left => View { count: 13, ready: 1 < 2 },
        Gate::Right => View { count: 17, ready: 1 > 2 }
    }
}
fn observe_void(value: &mut Gate) {
    match *value {
        Gate::Left => println!("left"),
        Gate::Right => println!("right")
    };
}

fn main() -> int {
    let mut gate = Gate::Left;
    let number = choose_int(&mut gate);
    let decimal = choose_float(&mut gate);
    let condition = choose_bool(&mut gate);
    let character = choose_char(&mut gate);
    let values = choose_array(&mut gate);
    let empty = choose_empty(&mut gate);
    let pair = choose_tuple(&mut gate);
    let view = choose_struct(&mut gate);
    observe_void(&mut gate);
    if decimal > 1.0 && condition && character == 'a' && empty.len() == 0
        && (pair.0).1 && view.ready {
        return number + values[0] + view.count;
    }
    1
}
"#
}

fn void_origin_source() -> &'static str {
    r#"
enum Gate { Left, Right }

fn observe_owned(value: Gate) {
    match value {
        Gate::Left => println!("owned-left"),
        Gate::Right => println!("owned-right")
    };
}

fn observe_immutable(value: &Gate) {
    match *value {
        Gate::Left => println!("immutable-left"),
        Gate::Right => println!("immutable-right")
    };
}

fn observe_mutable(value: &mut Gate) {
    match *value {
        Gate::Left => println!("mutable-left"),
        Gate::Right => println!("mutable-right")
    };
}

fn main() -> int {
    let immutable = Gate::Left;
    observe_immutable(&immutable);
    let mut mutable = Gate::Right;
    observe_mutable(&mut mutable);
    observe_owned(Gate::Left);
    86
}
"#
}

#[test]
fn mutable_enum_reference_complete_origin_schema_and_order_product_is_executable() {
    let mut failures = Vec::new();
    expect_acceptance_in_both_paths(
        "unit/unary/multi-field local-and-parameter read/write/read product",
        origin_and_order_source(),
        &mut failures,
    );
    match compile_program(origin_and_order_source(), CompilerOptions::default()) {
        Ok(llvm) => {
            for marker in [
                "define i32 @inspect_unit(i32*",
                "define i32 @inspect_scalar({ i32, double, i1 }*",
                "define i32 @inspect_packet(",
                "load i32, i32*",
                "switch i32",
            ] {
                if !llvm.contains(marker) {
                    failures.push(format!("origin/order LLVM omitted {marker:?}:\n{llvm}"));
                }
            }
        }
        Err(error) => failures.push(format!("origin/order product did not compile: {error}")),
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

#[test]
fn mutable_enum_reference_match_uses_the_complete_shared_copydata_result_predicate() {
    let mut failures = Vec::new();
    expect_acceptance_in_both_paths(
        "primitive/array/tuple/struct/Void result product",
        result_product_source(),
        &mut failures,
    );
    match compile_program(result_product_source(), CompilerOptions::default()) {
        Ok(llvm) => {
            for marker in [
                "[2 x double]",
                "[0 x double]",
                "{ { i32, i1 }, [2 x double] }",
                "%aero.struct.View",
            ] {
                if !llvm.contains(marker) {
                    failures.push(format!("result-product LLVM omitted {marker:?}:\n{llvm}"));
                }
            }
        }
        Err(error) => failures.push(format!("result product did not compile: {error}")),
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

#[test]
fn enum_match_void_result_is_shared_across_all_admitted_scrutinee_origins() {
    let ast = analyzed(void_origin_source())
        .unwrap_or_else(|error| panic!("shared Void source must pass semantics: {error}"));
    let checked = IrGenerator::new()
        .try_generate_ir(ast)
        .unwrap_or_else(|error| panic!("shared Void source must pass checked admission: {error}"));
    let debug = format!("{checked:#?}");
    for marker in [
        "CheckedImmutableEnumMatchRead",
        "CheckedMutableEnumMatchRead",
        "CheckedEnumDispatch",
    ] {
        assert!(
            debug.contains(marker),
            "shared Void checked IR omitted {marker:?}:\n{debug}"
        );
    }
    assert!(
        !debug.contains("CheckedMatchResultPlaceAlloca"),
        "discarded Void Matches must not fabricate result storage:\n{debug}"
    );
    CodeGenerator::new()
        .try_generate_code(checked)
        .unwrap_or_else(|error| panic!("shared Void checked IR must verify and lower: {error}"));
    compile_program(void_origin_source(), CompilerOptions::default())
        .unwrap_or_else(|error| panic!("shared Void source must compile: {error}"));
}

#[test]
fn mutable_enum_reference_observation_exclusions_fail_closed_in_both_trust_phases() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "free enum return",
            "enum E { A } fn copy(value: &mut E) -> E { *value } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum binding",
            "enum E { A } fn read(value: &mut E) -> int { let copy = *value; 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum comparison",
            "enum E { A } fn read(value: &mut E) -> int { if *value == E::A { return 1; } 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum by-value argument",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn read(value: &mut E) -> int { take(*value) } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum array storage",
            "enum E { A } fn read(value: &mut E) -> int { let values = [*value]; 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data", "array"],
        ),
        (
            "free enum tuple storage",
            "enum E { A } fn read(value: &mut E) -> int { let pair = (*value, 1); 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data", "tuple"],
        ),
        (
            "non-identifier mutable enum Match",
            "enum E { A } fn main() -> int { let mut owner = E::A; match *(&mut owner) { E::A => 1 } }",
            &["requires an identifier reference"],
        ),
        (
            "owned enum Match result",
            "enum E { A } fn copy(value: &mut E) -> int { let copied = match *value { E::A => E::A }; 0 } fn main() -> int { 0 }",
            &["must produce admitted Copy-data or Void"],
        ),
        (
            "String Match result",
            "enum E { A } fn copy(value: &mut E) -> int { let copied = match *value { E::A => \"no\" }; 0 } fn main() -> int { 0 }",
            &[
                "must produce admitted Copy-data or Void",
                "admitted CopyData, owned enum, or Void",
                "the only additional homogeneous result is Void",
            ],
        ),
        (
            "print used as a return value",
            "fn value() -> int { return print!(\"x\"); } fn main() -> int { 0 }",
            &[
                "return type mismatch",
                "Void expressions cannot be used as values",
            ],
        ),
        (
            "println used as a binding value",
            "fn main() -> int { let fabricated = println!(\"x\"); 0 }",
            &[
                "Void expression cannot be used as a value",
                "Void expressions cannot be used as values",
            ],
        ),
        (
            "print used as a function argument",
            "fn take(value: int) -> int { value } fn main() -> int { take(print!(\"x\")) }",
            &[
                "expected int, actual ()",
                "Void expressions cannot be used as values",
            ],
        ),
        (
            "println used as a comparison operand",
            "fn main() -> int { if println!(\"x\") == 0 { return 1; } 0 }",
            &[
                "Cannot compare types `()` and `int`",
                "Void expressions cannot be used as values",
            ],
        ),
        (
            "reference result escape",
            "enum E { A } fn escape(value: &mut E) -> &mut E { value } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "entry mutable reference parameter",
            "enum E { A } fn main(value: &mut E) -> int { match *value { E::A => 1 } }",
            &["process entry cannot use reference parameters"],
        ),
        (
            "mixed signature",
            "enum E { A } fn read(value: &mut E, count: int) -> int { match *value { E::A => count } } fn main() -> int { 0 }",
            &["exactly one mutable-reference parameter"],
        ),
        (
            "generic enum",
            "enum E<T> { Value(T) } fn read(value: &mut E) -> int { 0 } fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "named-field enum",
            "enum E { Value { count: int } } fn read(value: &mut E) -> int { 0 } fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "String-payload enum",
            "enum E { Value(String) } fn read(value: &mut E) -> int { 0 } fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "immutable owner mutable borrow",
            "enum E { A } fn read(value: &mut E) -> int { match *value { E::A => 1 } } fn main() -> int { let owner = E::A; read(&mut owner) }",
            &["must be declared mutable"],
        ),
        (
            "moved owner mutable borrow",
            "enum E { A } fn read(value: &mut E) -> int { match *value { E::A => 1 } } fn main() -> int { let mut owner = E::A; let moved = owner; read(&mut owner) }",
            &["because it was moved", "Use of moved value"],
        ),
        (
            "second mutable loan",
            "enum E { A } fn main() -> int { let mut owner = E::A; let first = &mut owner; let second = &mut owner; match *first { E::A => 1 } }",
            &["already borrowed as mutable"],
        ),
        (
            "overlapping immutable loan",
            "enum E { A } fn main() -> int { let mut owner = E::A; let first = &mut owner; let second = &owner; match *first { E::A => 1 } }",
            &["already borrowed as mutable", "borrowed as mutable"],
        ),
        (
            "relocated alias",
            "enum E { A } fn main() -> int { let mut owner = E::A; let first = &mut owner; let second = first; match *second { E::A => 1 } }",
            &["mutable reference aliases cannot be copied or relocated"],
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, expected) in cases {
        if let Err(error) = expect_rejection(source, expected) {
            failures.push(format!("{label}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

#[test]
fn mutable_enum_reference_observation_cli_and_system_gate_are_anchored() {
    let mut failures = Vec::new();
    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    for (label, path, anchors) in [
        (
            "root",
            &tracked_root,
            &[
                "mod observations;",
                "observe_mode(&mut mode)",
                "observe_then_replace(alias)",
                "score_packet(packet)",
            ][..],
        ),
        (
            "module",
            &tracked_module,
            &[
                "fn observe_mode(value: &mut Mode)",
                "fn observe_then_replace(value: &mut Packet)",
                "fn observe_packet_void(value: &mut Packet)",
                "*value = Packet::Pair",
            ][..],
        ),
    ] {
        match fs::read_to_string(path) {
            Ok(source) => {
                for anchor in anchors {
                    if source.matches(anchor).count() != 1 {
                        failures.push(format!(
                            "tracked CORE-086 {label} must contain one {anchor:?} at {}",
                            path.display()
                        ));
                    }
                }
            }
            Err(error) => failures.push(format!(
                "tracked CORE-086 {label} is unreadable at {}: {error}",
                path.display()
            )),
        }
    }

    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("define i32 @observe_then_replace(")
                && llvm.contains("switch i32") => {}
        Ok(llvm) => failures.push(format!("tracked CORE-086 LLVM omitted evidence:\n{llvm}")),
        Err(error) => failures.push(format!("tracked CORE-086 example failed: {error}")),
    }

    let workspace = TestWorkspace::new("tracked-example");
    let artifact = workspace.path("mutable-enum-reference-observation.ll");
    let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "CORE-086 CLI check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &tracked_root,
            Path::new("-o"),
            &artifact,
        ],
    );
    if !build.status.success() || !artifact.is_file() {
        failures.push(format!(
            "CORE-086 CLI build failed (artifact={}): {}",
            artifact.is_file(),
            output_text(&build)
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW)).expect("read Rust workflow");
    for anchor in [
        "Test mutable enum reference observation integration example",
        "cargo run -- check ../../examples/mutable_enum_reference_observation/main.aero",
        "cargo run -- run ../../examples/mutable_enum_reference_observation/main.aero",
        "opt-22 -passes=verify -disable-output ../../mutable_enum_reference_observation.ll",
        "llc-22 -verify-machineinstrs ../../mutable_enum_reference_observation.ll",
        "mutable enum reference observation example passed with exit code 86",
        "Test Windows mutable enum reference observation system specimen",
        "Windows mutable enum reference observation public run passed with exit code 86",
        "Windows mutable enum reference observation manual native execution passed with exit code 86",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "workflow must contain one CORE-086 anchor {anchor:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-086 system integration failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
