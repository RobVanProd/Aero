use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 84;
const EXAMPLE_ROOT: &str = "examples/immutable_enum_references/main.aero";
const EXAMPLE_MODULE: &str = "examples/immutable_enum_references/observations.aero";
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
            "aero-immutable-enum-reference-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create immutable enum reference workspace");
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
            .is_some_and(|name| name.starts_with("aero-immutable-enum-reference-"));
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

fn expect_rejection(source: &str, expected: &[&str]) -> Result<(), String> {
    let semantic = match SemanticAnalyzer::new().analyze(parsed(source)?) {
        Ok(_) => return Err("excluded CORE-084 source passed semantic analysis".to_string()),
        Err(error) => error,
    };
    if !expected.iter().any(|fragment| semantic.contains(fragment)) {
        return Err(format!(
            "semantic diagnostic {semantic:?} contained none of {expected:?}"
        ));
    }

    let admission = match IrGenerator::new().try_generate_ir(parsed(source)?) {
        Ok(_) => return Err("excluded CORE-084 source passed direct checked admission".to_string()),
        Err(error) => error.to_string(),
    };
    if !expected.iter().any(|fragment| admission.contains(fragment)) {
        return Err(format!(
            "checked-admission diagnostic {admission:?} contained none of {expected:?}"
        ));
    }

    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Err(format!("excluded CORE-084 source emitted LLVM:\n{llvm}")),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => Ok(()),
        Err(error) => Err(format!(
            "compile diagnostic {error:?} contained none of {expected:?}"
        )),
    }
}

fn positive_source() -> &'static str {
    r#"
struct Leaf { value: int, ready: bool }

enum Unit { Idle, Ready }
enum Scalar { Empty, Count(int), Flag(bool) }
enum Packet { Empty, Pair(int, bool), Leaf(Leaf) }

fn read_unit(value: &Unit) -> int {
    match *value { Unit::Idle => 1, Unit::Ready => 2 }
}

fn read_scalar(value: &Scalar) -> int {
    match *value {
        Scalar::Empty => 0,
        Scalar::Count(inner) => inner,
        Scalar::Flag(inner) => 3
    }
}

fn read_packet(value: &Packet) -> int {
    let alias = value;
    match *alias {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => number,
        Packet::Leaf(leaf) => leaf.value
    }
}

fn read_packet_twice(value: &Packet) -> int {
    read_packet(value) + read_packet(value)
}

fn main() -> int {
    let unit = Unit::Ready;
    let scalar = Scalar::Count(5);
    let packet = Packet::Pair(17, 1 < 2);

    let mut observed = 0;
    {
        let unit_ref: &Unit = &unit;
        let scalar_ref: &Scalar = &scalar;
        let first = &packet;
        let second = &packet;
        observed = read_unit(unit_ref) + read_scalar(scalar_ref)
            + read_packet(first) + read_packet_twice(second);
    }

    match packet {
        Packet::Empty => 1,
        Packet::Pair(number, ready) => observed + number,
        Packet::Leaf(leaf) => leaf.value
    }
}
"#
}

#[test]
fn immutable_enum_match_reference_positive_class_is_executable() {
    let llvm = compile_program(positive_source(), CompilerOptions::default())
        .unwrap_or_else(|error| panic!("CORE-084 positive class must compile: {error}"));
    for fragment in [
        "define i32 @read_unit(i32*",
        "define i32 @read_scalar({ i32, double, i1 }*",
        "define i32 @read_packet(",
        "load i32, i32*",
        "switch i32",
    ] {
        assert!(
            llvm.contains(fragment),
            "CORE-084 LLVM missing {fragment:?}:\n{llvm}"
        );
    }
}

#[test]
fn immutable_enum_match_reference_has_exact_checked_identity_in_both_admission_paths() {
    let ast = SemanticAnalyzer::new()
        .analyze(parsed(positive_source()).expect("CORE-084 positive source parses"))
        .map(|(_, ast)| ast)
        .unwrap_or_else(|error| panic!("CORE-084 semantic analysis must pass: {error}"));
    let checked = IrGenerator::new()
        .try_generate_ir(ast)
        .unwrap_or_else(|error| panic!("CORE-084 analyzed checked admission must pass: {error}"));
    let direct = IrGenerator::new()
        .try_generate_ir(parsed(positive_source()).expect("CORE-084 direct source parses"))
        .unwrap_or_else(|error| panic!("CORE-084 direct checked admission must pass: {error}"));

    for candidate in [&checked, &direct] {
        let debug = format!("{candidate:#?}");
        for marker in [
            "CheckedImmutableBorrow",
            "CheckedImmutableReferenceParameter",
            "CheckedImmutableEnumMatchRead",
            "CheckedEnumDispatch",
        ] {
            assert!(
                debug.contains(marker),
                "CORE-084 checked IR missing {marker:?}:\n{debug}"
            );
        }
        CodeGenerator::new()
            .try_generate_code(candidate.clone())
            .unwrap_or_else(|error| panic!("CORE-084 verified LLVM generation must pass: {error}"));
    }
}

#[test]
fn immutable_enum_match_reference_exclusions_fail_closed_in_both_trust_phases() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "free enum return",
            "enum E { A } fn copy(value: &E) -> E { *value } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum binding",
            "enum E { A } fn read(value: &E) -> int { let copy = *value; 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum comparison",
            "enum E { A } fn read(value: &E) -> int { if *value == E::A { return 1; } 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum by-value argument",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn read(value: &E) -> int { take(*value) } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free enum array storage",
            "enum E { A } fn read(value: &E) -> int { let values = [*value]; 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data", "not admitted"],
        ),
        (
            "free enum tuple storage",
            "enum E { A } fn read(value: &E) -> int { let pair = (*value, 1); 0 } fn main() -> int { 0 }",
            &["not admitted Copy-data", "not admitted"],
        ),
        (
            "mutable owner immutable loan",
            "enum E { A } fn main() -> int { let mut owner = E::A; let alias = &owner; 0 }",
            &["mutable-owner loan lifetimes"],
        ),
        (
            "mutable enum reference read",
            "enum E { A } fn read(value: &mut E) -> int { match *value { E::A => 1 } } fn main() -> int { 0 }",
            &["Match through mutable enum reference"],
        ),
        (
            "non-identifier enum reference Match",
            "enum E { A } fn main() -> int { let owner = E::A; match *(&owner) { E::A => 1 } }",
            &["requires an identifier reference"],
        ),
        (
            "owned enum Match result from reference",
            "enum E { A } fn copy(value: &E) -> int { let copied = match *value { E::A => E::A }; 0 } fn main() -> int { 0 }",
            &["must produce admitted Copy-data or Void"],
        ),
        (
            "reference result escape",
            "enum E { A } fn escape(value: &E) -> &E { value } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "entry reference parameter",
            "enum E { A } fn main(value: &E) -> int { match *value { E::A => 1 } }",
            &["process entry cannot use reference parameters"],
        ),
        (
            "generic enum reference",
            "enum E<T> { Value(T) } fn read(value: &E) -> int { 0 } fn main() -> int { 0 }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
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
fn immutable_enum_match_reference_cli_and_system_integration_are_anchored() {
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
                "let first = &packet;",
                "owned_packet_score(packet)",
            ][..],
        ),
        (
            "module",
            &tracked_module,
            &[
                "fn read_mode(value: &Mode)",
                "match *value { Mode::Off",
                "fn read_packet_twice(first: &Packet, second: &Packet)",
            ][..],
        ),
    ] {
        match fs::read_to_string(path) {
            Ok(source) => {
                for anchor in anchors {
                    if source.matches(anchor).count() != 1 {
                        failures.push(format!(
                            "tracked immutable enum reference {label} must contain one {anchor:?} at {}",
                            path.display()
                        ));
                    }
                }
            }
            Err(error) => failures.push(format!(
                "tracked immutable enum reference {label} is unreadable at {}: {error}",
                path.display()
            )),
        }
    }

    match compile_file(&tracked_root, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("define i32 @read_mode(i32*")
                && llvm.contains("define i32 @read_signal(")
                && llvm.contains("define i32 @read_packet_twice(")
                && llvm.contains("load { i32, double, i1 }, { i32, double, i1 }*") => {}
        Ok(llvm) => failures.push(format!(
            "tracked immutable enum reference LLVM omitted exact read evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!(
            "tracked immutable enum reference compilation failed: {error}"
        )),
    }

    let workspace = TestWorkspace::new("tracked-example");
    let artifact = workspace.path("immutable-enum-reference.ll");
    let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
    if !check.status.success() {
        failures.push(format!(
            "tracked immutable enum reference CLI check failed: {}",
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
            "tracked immutable enum reference CLI build failed (artifact={}): {}",
            artifact.is_file(),
            output_text(&build)
        ));
    }

    let invalid_workspace = TestWorkspace::new("cli-negative");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_artifact = invalid_workspace.path("must-not-exist.ll");
    fs::write(
        &invalid,
        "enum E { A } fn copy(value: &E) -> E { *value } fn main() -> int { 0 }",
    )
    .expect("write invalid immutable enum reference source");
    for command in ["check", "run"] {
        let output = run_cli(&invalid_workspace, &[Path::new(command), &invalid]);
        let diagnostic = output_text(&output);
        if output.status.success() || !diagnostic.contains("not admitted Copy-data") {
            failures.push(format!(
                "invalid immutable enum reference CLI {command} did not fail with the shared cause: {diagnostic}"
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
            "invalid immutable enum reference CLI build did not fail closed without an artifact: {invalid_diagnostic}"
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW))
        .expect("read Rust workflow for immutable enum reference integration anchors");
    for anchor in [
        "Test immutable enum reference integration example",
        "cargo run -- check ../../examples/immutable_enum_references/main.aero",
        "cargo run -- run ../../examples/immutable_enum_references/main.aero",
        "opt-22 -passes=verify -disable-output ../../immutable_enum_references.ll",
        "llc-22 -verify-machineinstrs ../../immutable_enum_references.ll",
        "clang-22 -no-pie ../../immutable_enum_references.o -o ../../immutable_enum_references",
        "immutable enum reference example passed with exit code 84",
        "Test Windows immutable enum reference system specimen",
        "Windows immutable enum reference public run passed with exit code 84",
        "Windows immutable enum reference manual native execution passed with exit code 84",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "public workflow must contain exactly one CORE-084 anchor {anchor:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-084 system integration failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
