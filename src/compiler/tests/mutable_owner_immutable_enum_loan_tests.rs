use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 85;
const EXAMPLE_ROOT: &str = "examples/mutable_owner_immutable_enum_loans/main.aero";
const EXAMPLE_MODULE: &str = "examples/mutable_owner_immutable_enum_loans/observations.aero";
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
            "aero-mutable-owner-immutable-enum-loan-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CORE-085 test workspace");
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
            .is_some_and(|name| name.starts_with("aero-mutable-owner-immutable-enum-loan-"));
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

fn observe_function_scope() -> int {
    let mut owner = Packet::Pair(19, 1 < 2);
    let alias = &owner;
    match *alias {
        Packet::Empty => 0,
        Packet::Pair(number, ready) => number,
        Packet::Leaf(leaf) => leaf.value
    }
}

fn replace_scalar(value: &mut Scalar) {
    *value = Scalar::Count(11);
}

fn replace_packet(value: &mut Packet) {
    *value = Packet::Leaf(Leaf { value: 13, ready: 1 < 2 });
}

fn main() -> int {
    let mut unit = Unit::Ready;
    let mut scalar = Scalar::Count(5);
    let mut packet = Packet::Pair(7, 1 < 2);
    let mut observed = 0;

    {
        let unit_ref: &Unit = &unit;
        let scalar_ref = &scalar;
        let first: &Packet = &packet;
        let second = &packet;
        observed = read_unit(unit_ref) + read_scalar(scalar_ref)
            + read_packet(first) + read_packet(second);
        {
            let nested = &packet;
            observed = observed + read_packet(nested);
        }
    }

    unit = Unit::Idle;
    replace_scalar(&mut scalar);
    replace_packet(&mut packet);
    observed = observed + read_unit(&unit) + read_scalar(&scalar) + read_packet(&packet)
        + observe_function_scope();

    match packet {
        Packet::Empty => observed,
        Packet::Pair(number, ready) => observed + number,
        Packet::Leaf(leaf) => observed + leaf.value
    }
}

"#
}

fn expect_rejection(label: &str, source: &str, expected: &[&str]) -> Result<(), String> {
    let semantic = match SemanticAnalyzer::new().analyze(parsed(source)?) {
        Ok(_) => {
            return Err(format!(
                "{label}: excluded CORE-085 source passed semantic analysis"
            ));
        }
        Err(error) => error,
    };
    if !expected.iter().any(|fragment| semantic.contains(fragment)) {
        return Err(format!(
            "{label}: semantic diagnostic {semantic:?} contained none of {expected:?}"
        ));
    }

    let admission = match IrGenerator::new().try_generate_ir(parsed(source)?) {
        Ok(_) => {
            return Err(format!(
                "{label}: excluded CORE-085 source passed direct checked admission"
            ));
        }
        Err(error) => error.to_string(),
    };
    if !expected.iter().any(|fragment| admission.contains(fragment)) {
        return Err(format!(
            "{label}: checked-admission diagnostic {admission:?} contained none of {expected:?}"
        ));
    }

    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => Err(format!(
            "{label}: excluded CORE-085 source emitted LLVM:\n{llvm}"
        )),
        Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => Ok(()),
        Err(error) => Err(format!(
            "{label}: compile diagnostic {error:?} contained none of {expected:?}"
        )),
    }
}

#[test]
fn mutable_owner_immutable_enum_loan_complete_positive_class_is_executable() {
    let llvm = compile_program(positive_source(), CompilerOptions::default())
        .unwrap_or_else(|error| panic!("CORE-085 positive class must compile: {error}"));
    for fragment in [
        "define i32 @read_unit(i32*",
        "define i32 @read_scalar({ i32, double, i1 }*",
        "define i32 @read_packet(",
        "define void @replace_scalar(",
        "define void @replace_packet(",
        "switch i32",
    ] {
        assert!(
            llvm.contains(fragment),
            "CORE-085 LLVM missing {fragment:?}:\n{llvm}"
        );
    }
}

#[test]
fn mutable_owner_immutable_enum_loans_have_exact_checked_identity_in_both_paths() {
    let ast = SemanticAnalyzer::new()
        .analyze(parsed(positive_source()).expect("CORE-085 positive source parses"))
        .map(|(_, ast)| ast)
        .unwrap_or_else(|error| panic!("CORE-085 semantic analysis must pass: {error}"));
    let checked = IrGenerator::new()
        .try_generate_ir(ast)
        .unwrap_or_else(|error| panic!("CORE-085 analyzed checked admission must pass: {error}"));
    let direct = IrGenerator::new()
        .try_generate_ir(parsed(positive_source()).expect("CORE-085 direct source parses"))
        .unwrap_or_else(|error| panic!("CORE-085 direct checked admission must pass: {error}"));

    for candidate in [&checked, &direct] {
        let debug = format!("{candidate:#?}");
        for marker in [
            "CheckedMutableOwnedPlaceAlloca",
            "CheckedImmutableBorrow",
            "CheckedImmutableEnumMatchRead",
            "CheckedMutableOwnerImmutableEnumBorrowEnd",
            "CheckedMutableBorrow",
            "CheckedMutableBorrowEnd",
        ] {
            assert!(
                debug.contains(marker),
                "CORE-085 checked IR missing {marker:?}:\n{debug}"
            );
        }
        CodeGenerator::new()
            .try_generate_code(candidate.clone())
            .unwrap_or_else(|error| panic!("CORE-085 verified LLVM generation must pass: {error}"));
    }

    let lexical_end_source = r#"
enum E { A, B }
fn read(value: &E) -> int { match *value { E::A => 1, E::B => 2 } }
fn main() -> int {
    let mut owner = E::A;
    let mut observed = 0;
    {
        let alias = &owner;
        observed = read(alias) + read(alias);
    }
    owner = E::B;
    observed + match owner { E::A => 1, E::B => 2 }
}
"#;
    let analyzed = SemanticAnalyzer::new()
        .analyze(parsed(lexical_end_source).expect("lexical-end source parses"))
        .map(|(_, ast)| ast)
        .unwrap_or_else(|error| panic!("lexical-end source analyzes: {error}"));
    for candidate in [
        IrGenerator::new()
            .try_generate_ir(analyzed)
            .expect("analyzed lexical-end source reaches checked IR"),
        IrGenerator::new()
            .try_generate_ir(parsed(lexical_end_source).expect("direct lexical-end source parses"))
            .expect("direct lexical-end source reaches checked IR"),
    ] {
        let debug = format!("{candidate:#?}");
        let calls = debug
            .match_indices("function: \"read\"")
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        let ends = debug
            .match_indices("CheckedMutableOwnerImmutableEnumBorrowEnd")
            .map(|(position, _)| position)
            .collect::<Vec<_>>();
        assert_eq!(calls.len(), 2, "bound alias must be called twice:\n{debug}");
        assert_eq!(
            ends.len(),
            1,
            "bound alias must have one exact end:\n{debug}"
        );
        assert!(
            calls[1] < ends[0],
            "bound alias ended before its final call instead of at lexical exit:\n{debug}"
        );
    }
}

#[test]
fn mutable_owner_immutable_enum_loan_exclusions_fail_closed_in_both_trust_phases() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "assignment while live",
            "enum E { A, B } fn main() { let mut owner = E::A; let alias = &owner; owner = E::B; }",
            &["borrowed"],
        ),
        (
            "mutable borrow while live",
            "enum E { A } fn main() { let mut owner = E::A; let alias = &owner; let mutation = &mut owner; }",
            &["also borrowed as immutable"],
        ),
        (
            "move while live",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() -> int { let mut owner = E::A; let alias = &owner; take(owner) }",
            &["borrowed", "move"],
        ),
        (
            "owned Match while live",
            "enum E { A } fn main() -> int { let mut owner = E::A; let alias = &owner; match owner { E::A => 1 } }",
            &["borrowed", "move"],
        ),
        (
            "free dereference binding",
            "enum E { A } fn main() { let mut owner = E::A; let alias = &owner; let copy = *alias; }",
            &["not admitted Copy-data"],
        ),
        (
            "free dereference comparison",
            "enum E { A } fn main() -> int { let mut owner = E::A; let alias = &owner; if *alias == E::A { return 1; } 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free dereference by-value argument",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() -> int { let mut owner = E::A; let alias = &owner; take(*alias) }",
            &["not admitted Copy-data"],
        ),
        (
            "free dereference return",
            "enum E { A } fn copy() -> E { let mut owner = E::A; let alias = &owner; *alias } fn main() -> int { 0 }",
            &["not admitted Copy-data"],
        ),
        (
            "free dereference array storage",
            "enum E { A } fn main() { let mut owner = E::A; let alias = &owner; let values = [*alias]; }",
            &["not admitted Copy-data", "array"],
        ),
        (
            "free dereference tuple storage",
            "enum E { A } fn main() { let mut owner = E::A; let alias = &owner; let values = (*alias, 1); }",
            &["not admitted Copy-data", "tuple"],
        ),
        (
            "reference result escape",
            "enum E { A } fn escape() -> &E { let mut owner = E::A; &owner } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "non-identifier dereference Match",
            "enum E { A } fn main() -> int { let mut owner = E::A; match *(&owner) { E::A => 1 } }",
            &["requires an identifier reference"],
        ),
        (
            "schema mismatch annotation",
            "enum E { A } enum F { B } fn main() { let mut owner = E::A; let alias: &F = &owner; }",
            &["type mismatch", "expected &F"],
        ),
        (
            "uninitialized owner",
            "enum E { A } fn main() { let mut owner: E; let alias = &owner; }",
            &["uninitialized", "not an initialized local binding"],
        ),
        (
            "moved owner",
            "enum E { A } fn main() { let mut owner = E::A; let moved = owner; let alias = &owner; }",
            &["moved"],
        ),
        (
            "maybe-moved owner",
            "enum E { A } fn main() { let mut owner = E::A; if 1 < 2 { let moved = owner; } let alias = &owner; }",
            &["may have been moved"],
        ),
        (
            "generic enum",
            "enum E<T> { Value(T) } fn main() { let mut owner: E<int> = E::Value(1); let alias = &owner; }",
            &["generic enum references are not admitted in CAP-006"],
        ),
        (
            "String payload enum",
            "enum E { Value(String) } fn main() { let mut owner = E::Value(\"x\"); let alias = &owner; }",
            &["not an admitted non-generic unit-or-positional-CopyData enum"],
        ),
        (
            "live loan break",
            "enum E { A } fn main() { while 1 < 2 { let mut owner = E::A; let alias = &owner; break; } }",
            &["immutable enum loan", "borrow"],
        ),
        (
            "live loan continue",
            "enum E { A } fn main() { while 1 < 2 { let mut owner = E::A; let alias = &owner; continue; } }",
            &["immutable enum loan", "borrow"],
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, expected) in cases {
        if let Err(error) = expect_rejection(label, source, expected) {
            failures.push(error);
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

#[test]
fn mutable_owner_immutable_enum_loan_cli_module_and_system_gates_are_anchored() {
    let root = repository_root();
    let example = root.join(EXAMPLE_ROOT);
    let module = root.join(EXAMPLE_MODULE);
    let mut failures = Vec::new();

    for (label, path, anchors) in [
        (
            "root",
            &example,
            &[
                "mod observations;",
                "let mut mode = Mode::On;",
                "let mode_ref: &Mode = &mode;",
                "let second = &packet;",
                "replace_mode(&mut mode);",
                "read_packet(&packet)",
            ][..],
        ),
        (
            "module",
            &module,
            &[
                "fn read_mode(value: &Mode)",
                "match *value { Mode::Off",
                "fn replace_packet(value: &mut Packet)",
            ][..],
        ),
    ] {
        match fs::read_to_string(path) {
            Ok(source) => {
                for anchor in anchors {
                    if source.matches(anchor).count() != 1 {
                        failures.push(format!(
                            "tracked CORE-085 {label} must contain one {anchor:?} at {}",
                            path.display()
                        ));
                    }
                }
            }
            Err(error) => failures.push(format!(
                "tracked CORE-085 {label} is unreadable at {}: {error}",
                path.display()
            )),
        }
    }

    match compile_file(&example, CompilerOptions::default()) {
        Ok(llvm)
            if llvm.contains("define i32 @read_packet(")
                && llvm.contains("define void @replace_packet(")
                && llvm.contains("switch i32") => {}
        Ok(llvm) => failures.push(format!(
            "tracked CORE-085 LLVM omitted exact execution evidence:\n{llvm}"
        )),
        Err(error) => failures.push(format!("tracked CORE-085 compilation failed: {error}")),
    }

    let workspace = TestWorkspace::new("tracked-example");
    let artifact = workspace.path("mutable-owner-immutable-enum-loans.ll");
    let check = run_cli(&workspace, &[Path::new("check"), &example]);
    if !check.status.success() {
        failures.push(format!(
            "tracked CORE-085 CLI check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_cli(
        &workspace,
        &[Path::new("build"), &example, Path::new("-o"), &artifact],
    );
    if !build.status.success() || !artifact.is_file() {
        failures.push(format!(
            "tracked CORE-085 CLI build failed (artifact={}): {}",
            artifact.is_file(),
            output_text(&build)
        ));
    }

    let invalid_workspace = TestWorkspace::new("cli-negative");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_artifact = invalid_workspace.path("must-not-exist.ll");
    fs::write(
        &invalid,
        "enum E { A, B } fn main() { let mut owner = E::A; let alias = &owner; owner = E::B; }",
    )
    .expect("write invalid CORE-085 source");
    for command in ["check", "run"] {
        let output = run_cli(&invalid_workspace, &[Path::new(command), &invalid]);
        let diagnostic = output_text(&output);
        if output.status.success() || !diagnostic.contains("borrowed") {
            failures.push(format!(
                "invalid CORE-085 CLI {command} did not fail with the shared cause: {diagnostic}"
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
        || !invalid_diagnostic.contains("borrowed")
    {
        failures.push(format!(
            "invalid CORE-085 CLI build did not fail closed without an artifact: {invalid_diagnostic}"
        ));
    }

    let workflow = fs::read_to_string(root.join(WORKFLOW)).expect("read Rust workflow");
    for anchor in [
        "Test mutable-owner immutable enum loan integration example",
        "cargo run -- check ../../examples/mutable_owner_immutable_enum_loans/main.aero",
        "cargo run -- run ../../examples/mutable_owner_immutable_enum_loans/main.aero",
        "opt-22 -passes=verify -disable-output ../../mutable_owner_immutable_enum_loans.ll",
        "llc-22 -verify-machineinstrs ../../mutable_owner_immutable_enum_loans.ll",
        "clang-22 -no-pie ../../mutable_owner_immutable_enum_loans.o -o ../../mutable_owner_immutable_enum_loans",
        "mutable-owner immutable enum loan example passed with exit code 85",
        "Test Windows mutable-owner immutable enum loan system specimen",
        "Windows mutable-owner immutable enum loan public run passed with exit code 85",
        "Windows mutable-owner immutable enum loan manual native execution passed with exit code 85",
    ] {
        if workflow.matches(anchor).count() != 1 {
            failures.push(format!(
                "public workflow must contain exactly one CORE-085 anchor {anchor:?}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-085 integration failures (expected native exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
