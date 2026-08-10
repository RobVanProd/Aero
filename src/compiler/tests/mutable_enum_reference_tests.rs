use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 83;
const EXAMPLE_ROOT: &str = "examples/mutable_enum_references/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_enum_references/mutations.aero";
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
            "aero-mutable-enum-reference-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create mutable enum reference workspace");
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
            .is_some_and(|name| name.starts_with("aero-mutable-enum-reference-"));
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

fn tracked_root_source() -> &'static str {
    r#"mod mutations;

struct Leaf { value: int, ready: bool }

enum Mode { Off, On }
enum Packet { Empty, Pair(int, bool), Leaf(Leaf) }

fn score_mode(value: Mode) -> int {
    match value { Mode::Off => 0, Mode::On => 7 }
}

fn score_packet(value: Packet) -> int {
    match value {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => number + ready_score(ready),
        Packet::Leaf(leaf) => leaf.value
    }
}

fn ready_score(value: bool) -> int {
    if value { return 3; }
    0
}

fn main() -> int {
    const OFFSET: int = 1;

    let mut mode = Mode::Off;
    replace_mode(&mut mode);

    let mut packet = Packet::Leaf(Leaf { value: 1, ready: 1 < 2 });
    {
        let alias: &mut Packet = &mut packet;
        forward_packet(alias);
    }

    let values = [5, 6];
    let pair = (8, 9);
    score_mode(mode) + score_packet(packet) + values[0] + values[1]
        + pair.0 + pair.1 + "Aero".len() + OFFSET
}
"#
}

fn tracked_module_source() -> &'static str {
    r#"fn make_packet(value: int) -> Packet {
    Packet::Pair(value, value > 0)
}

fn replace_mode(value: &mut Mode) {
    *value = Mode::On;
}

fn replace_packet(value: &mut Packet) {
    *value = make_packet(40);
}

fn forward_packet(value: &mut Packet) {
    replace_packet(value);
}
"#
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

fn expect_rejection(source: &str, expected: &[&str]) -> Result<(), String> {
    let semantic = match analyzed(source) {
        Ok(_) => return Err("excluded CORE-083 source passed semantics".to_string()),
        Err(error) => error,
    };
    if !expected.iter().any(|fragment| semantic.contains(fragment)) {
        return Err(format!(
            "semantic diagnostic {semantic:?} contained none of {expected:?}"
        ));
    }
    let admission = match IrGenerator::new()
        .try_generate_ir(parsed(source).map_err(|error| format!("parse failed: {error}"))?)
    {
        Ok(_) => return Err("excluded CORE-083 source passed direct checked admission".to_string()),
        Err(error) => error.to_string(),
    };
    if !expected.iter().any(|fragment| admission.contains(fragment)) {
        return Err(format!(
            "checked-admission diagnostic {admission:?} contained none of {expected:?}"
        ));
    }
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Err(format!("excluded CORE-083 source emitted LLVM:\n{llvm}")),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => Ok(()),
        Err(error) => Err(format!(
            "compile diagnostic {error:?} contained none of {expected:?}"
        )),
    }
}

fn complete_source() -> &'static str {
    r#"
struct Leaf { value: int, ready: bool }

enum Unit { Zero, One }
enum Scalar { Empty, Count(int), Ready(bool) }
enum Packet { Empty, Pair(int, bool), Leaf(Leaf) }

fn make_packet(value: int) -> Packet { Packet::Pair(value, value > 0) }
fn choose_packet(flag: bool) -> Packet {
    let choice = Unit::One;
    match choice {
        Unit::Zero => Packet::Empty,
        Unit::One => Packet::Leaf(Leaf { value: 13, ready: flag })
    }
}

fn replace_unit(value: &mut Unit) { *value = Unit::One; }
fn replace_scalar(value: &mut Scalar) { *value = Scalar::Count(11); }
fn replace_packet_from_call(value: &mut Packet) { *value = make_packet(17); }
fn replace_packet_from_match(value: &mut Packet) { *value = choose_packet(1 < 2); }
fn replace_packet_from_owner(value: &mut Packet) {
    let source = Packet::Pair(19, 1 < 2);
    *value = source;
}
fn forward_packet(value: &mut Packet) { replace_packet_from_call(value); }

fn score_unit(value: Unit) -> int {
    match value { Unit::Zero => 0, Unit::One => 1 }
}
fn score_scalar(value: Scalar) -> int {
    match value { Scalar::Empty => 0, Scalar::Count(inner) => inner, Scalar::Ready(inner) => 2 }
}
fn score_packet(value: Packet) -> int {
    match value {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => number,
        Packet::Leaf(leaf) => leaf.value
    }
}

fn main() -> int {
    let mut unit = Unit::Zero;
    replace_unit(&mut unit);

    let mut scalar = Scalar::Ready(1 < 2);
    replace_scalar(&mut scalar);

    let mut first = Packet::Empty;
    forward_packet(&mut first);

    let mut second = Packet::Pair(1, 1 < 2);
    {
        let alias: &mut Packet = &mut second;
        replace_packet_from_match(alias);
    }

    let mut third = Packet::Empty;
    replace_packet_from_owner(&mut third);

    score_unit(unit) + score_scalar(scalar) + score_packet(first)
        + score_packet(second) + score_packet(third)
}
"#
}

#[test]
fn mutable_enum_reference_positive_class_is_checked_and_executable() {
    let llvm = compile_program(complete_source(), CompilerOptions::default())
        .unwrap_or_else(|error| panic!("CORE-083 positive class must compile: {error}"));

    for fragment in [
        "define void @replace_unit(i32*",
        "define void @replace_scalar({ i32, double, i1 }*",
        "define void @replace_packet_from_call(",
        "store i32",
        "store { i32, double, i1 }",
    ] {
        assert!(
            llvm.contains(fragment),
            "LLVM missing {fragment:?}:\n{llvm}"
        );
    }
}

#[test]
fn mutable_enum_reference_independent_admission_retains_exact_checked_identity() {
    let ast = analyzed(complete_source())
        .unwrap_or_else(|error| panic!("CORE-083 semantic analysis must pass: {error}"));
    let checked = IrGenerator::new()
        .try_generate_ir(ast)
        .unwrap_or_else(|error| panic!("CORE-083 checked admission must pass: {error}"));
    let debug = format!("{checked:#?}");
    for marker in [
        "CheckedMutableOwnedPlaceAlloca",
        "CheckedMutableReferenceParameter",
        "CheckedMutableBorrow",
        "CheckedMutableDereferenceAssignment",
        "CheckedMutableBorrowEnd",
        "Enum {",
    ] {
        assert!(
            debug.contains(marker),
            "checked IR missing {marker:?}:\n{debug}"
        );
    }
    CodeGenerator::new()
        .try_generate_code(checked)
        .unwrap_or_else(|error| panic!("CORE-083 verified LLVM generation must pass: {error}"));
}

#[test]
fn mutable_enum_reference_checked_admission_does_not_depend_on_semantic_mutation() {
    let checked = IrGenerator::new()
        .try_generate_ir(parsed(complete_source()).expect("CORE-083 source parses"))
        .unwrap_or_else(|error| {
            panic!("CORE-083 direct checked admission must classify the same class: {error}")
        });
    CodeGenerator::new()
        .try_generate_code(checked)
        .unwrap_or_else(|error| panic!("CORE-083 direct checked IR must verify: {error}"));
}

#[test]
fn mutable_enum_reference_exclusions_fail_closed_in_both_trust_phases() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "free immutable enum parameter read",
            "enum E { A } fn inspect(value: &E) -> E { *value } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free immutable enum local-alias read",
            "enum E { A } fn main() -> int { let owner = E::A; let alias = &owner; let copy = *alias; 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "read through mutable enum reference",
            "enum E { A } fn read(value: &mut E) -> E { *value } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "Match through mutable enum reference",
            "enum E { A } fn read(value: &mut E) -> int { match *value { E::A => 1 } } fn main() -> int { 0 }",
            &[
                "Match through mutable enum reference",
                "not admitted Copy-data",
                "Match expressions are not supported",
            ],
        ),
        (
            "reference result escape",
            "enum E { A } fn escape(value: &mut E) -> &mut E { value } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "mixed mutable signature",
            "enum E { A } fn bad(value: &mut E, count: int) {} fn main() -> int { 0 }",
            &["exactly one mutable-reference parameter"],
        ),
        (
            "multiple mutable signature",
            "enum E { A } fn bad(left: &mut E, right: &mut E) {} fn main() -> int { 0 }",
            &["exactly one mutable-reference parameter"],
        ),
        (
            "entry reference parameter",
            "enum E { A } fn main(value: &mut E) -> int { 0 }",
            &["process entry cannot use reference parameters"],
        ),
        (
            "immutable owner",
            "enum E { A } fn set(value: &mut E) { *value = E::A; } fn main() -> int { let owner = E::A; set(&mut owner); 0 }",
            &["must be declared mutable"],
        ),
        (
            "moved owner",
            "enum E { A } fn set(value: &mut E) { *value = E::A; } fn main() -> int { let mut owner = E::A; let moved = owner; set(&mut owner); 0 }",
            &["because it was moved", "Use of moved value"],
        ),
        (
            "maybe-moved owner",
            "enum E { A } fn set(value: &mut E) { *value = E::A; } fn main() -> int { let mut owner = E::A; if 1 < 2 { let moved = owner; } set(&mut owner); 0 }",
            &["may have been moved"],
        ),
        (
            "wrong schema call",
            "enum Left { A } enum Right { B } fn set(value: &mut Left) { *value = Left::A; } fn main() -> int { let mut value = Right::B; set(&mut value); 0 }",
            &["pointee mismatch: expected Left, actual Right"],
        ),
        (
            "wrong schema assignment",
            "enum Left { A } enum Right { B } fn set(value: &mut Left) { *value = Right::B; } fn main() -> int { 0 }",
            &["type mismatch: expected Left, actual Right"],
        ),
        (
            "relocated alias",
            "enum E { A } fn main() -> int { let mut owner = E::A; let first = &mut owner; let second = first; 0 }",
            &["mutable reference aliases cannot be copied or relocated"],
        ),
        (
            "second mutable loan",
            "enum E { A } fn main() -> int { let mut owner = E::A; let first = &mut owner; let second = &mut owner; 0 }",
            &["already borrowed as mutable"],
        ),
        (
            "immutable loan conflict",
            "enum E { A } fn main() -> int { let mut owner = E::A; let first = &owner; let second = &mut owner; 0 }",
            &["mutable-owner loan lifetimes", "also borrowed as immutable"],
        ),
        (
            "generic enum",
            "enum E<T> { Value(T) } fn set(value: &mut E) {} fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "named-field enum",
            "enum E { Value { count: int } } fn set(value: &mut E) {} fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "String payload enum",
            "enum E { Value(String) } fn set(value: &mut E) {} fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "enum struct storage",
            "enum E { A } struct Holder { value: E } fn main() -> int { let value = Holder { value: E::A }; 0 }",
            &[
                "not an admitted",
                "Struct construction",
                "aggregate expression",
            ],
        ),
        (
            "enum array storage",
            "enum E { A } fn main() -> int { let values = [E::A]; 0 }",
            &["not admitted Copy-data", "array"],
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
fn mutable_enum_reference_cli_and_system_integration_are_anchored() {
    let mut failures = Vec::new();
    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);

    for (label, path, expected) in [
        ("root", &tracked_root, tracked_root_source()),
        ("module", &tracked_module, tracked_module_source()),
    ] {
        match fs::read_to_string(path) {
            Ok(actual) if actual == expected => {}
            Ok(actual) => failures.push(format!(
                "tracked mutable enum reference {label} drifted at {}:\n{actual}",
                path.display()
            )),
            Err(error) => failures.push(format!(
                "tracked mutable enum reference {label} is unreadable at {}: {error}",
                path.display()
            )),
        }
    }

    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("define void @replace_packet(")
                && llvm.contains("define void @forward_packet(")
                && llvm.contains("store { i32, { double, i1 }, %aero.struct.Leaf }") => {}
        Ok(llvm) => failures.push(format!(
            "tracked mutable enum reference LLVM omitted exact replacement evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!(
            "tracked mutable enum reference compilation failed: {error}"
        )),
    }

    let workspace = TestWorkspace::new("tracked-example");
    let artifact = workspace.path("mutable-enum-reference.ll");
    let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "tracked mutable enum reference CLI check failed: {}",
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
            "tracked mutable enum reference CLI build failed (artifact={}): {}",
            artifact.is_file(),
            output_text(&build)
        ));
    }

    let invalid_workspace = TestWorkspace::new("cli-negative");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_artifact = invalid_workspace.path("must-not-exist.ll");
    fs::write(
        &invalid,
        "enum E { A } fn inspect(value: &E) -> int { let copy = *value; 0 } fn main() -> int { let owner = E::A; inspect(&owner) }",
    )
    .expect("write invalid immutable enum reference source");
    for command in ["check", "run"] {
        let output = run_cli(&invalid_workspace, &[Path::new(command), &invalid]);
        let diagnostic = output_text(&output);
        if output.status.success() || !diagnostic.contains("not admitted Copy-data") {
            failures.push(format!(
                "invalid mutable enum reference CLI {command} did not fail with the shared cause: {diagnostic}"
            ));
        }
    }
    let invalid_build = run_cli(
        &invalid_workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    let invalid_diagnostic = output_text(&invalid_build);
    if invalid_build.status.success()
        || invalid_artifact.exists()
        || !invalid_diagnostic.contains("not admitted Copy-data")
    {
        failures.push(format!(
            "invalid mutable enum reference CLI build did not fail closed without an artifact: {invalid_diagnostic}"
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for mutable enum reference integration anchors");
    for anchor in [
        "Test mutable enum reference integration example",
        "cargo run -- check ../../examples/mutable_enum_references/main.aero",
        "cargo run -- run ../../examples/mutable_enum_references/main.aero",
        "opt-22 -passes=verify -disable-output ../../mutable_enum_references.ll",
        "llc-22 -verify-machineinstrs ../../mutable_enum_references.ll",
        "clang-22 -no-pie ../../mutable_enum_references.o -o ../../mutable_enum_references",
        "mutable enum reference example passed with exit code 83",
        "Test Windows mutable enum reference system specimen",
        "Windows mutable enum reference public run passed with exit code 83",
        "Windows mutable enum reference manual native execution passed with exit code 83",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "public workflow must contain exactly one CORE-083 anchor {anchor:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-083 system integration failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
