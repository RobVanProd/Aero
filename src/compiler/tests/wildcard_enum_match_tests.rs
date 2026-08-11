use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, check_file, compile_file,
    compile_program, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const TERMINAL_WILDCARD: &str = r#"
enum Phase { Cold, Warm, Hot }

fn score(value: Phase) -> int {
    match value {
        Phase::Hot => 40,
        _ => 2,
    }
}

fn main() -> int {
    score(Phase::Warm)
}
"#;

const PAYLOAD_WILDCARD: &str = r#"
enum Outcome { Ready(int), Failed(char) }

fn score(value: Outcome) -> int {
    match value {
        Outcome::Ready(number) => number,
        Outcome::Failed(_) => 0,
    }
}

fn main() -> int {
    score(Outcome::Failed('e'))
}
"#;

const COMPLETE_PRODUCT: &str = r#"
enum State {
    Idle,
    Reading(int, bool),
    Failed(char),
    Closed,
}

enum Toggle { Off, On }

enum Sample<T> {
    Present(T),
    Missing,
}

fn score_state(value: State) -> int {
    match value {
        State::Reading(number, _) => number,
        State::Failed(_) => 5,
        _ => 2,
    }
}

fn ignore_state(value: State) -> int {
    match value { _ => 1 }
}

fn option_score(value: Option<int>) -> int {
    match value {
        Some(_) => 2,
        _ => 3,
    }
}

fn result_score(value: Result<int, char>) -> int {
    match value {
        Ok(number) => number,
        Err(_) => 1,
    }
}

fn sample_score(value: Sample<int>) -> int {
    match value {
        Sample::Present(_) => 4,
        _ => 1,
    }
}

fn nested_score(value: State) -> int {
    match value {
        State::Reading(number, _) => match Toggle::On {
            Toggle::On => number,
            _ => 0,
        },
        _ => 0,
    }
}

fn main() -> int {
    let present: Option<int> = Some(10);
    let absent: Option<int> = None;
    let success: Result<int, char> = Ok(7);
    let failure: Result<int, char> = Err('e');
    let sample: Sample<int> = Sample::Present(11);
    let missing: Sample<int> = Sample::Missing;
    score_state(State::Reading(40, 1 < 2))
        + score_state(State::Failed('x'))
        + score_state(State::Idle)
        + score_state(State::Closed)
        + ignore_state(State::Idle)
        + option_score(present)
        + option_score(absent)
        + result_score(success)
        + result_score(failure)
        + sample_score(sample)
        + sample_score(missing)
        + nested_score(State::Reading(6, 2 < 3))
}
"#;

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
            "aero-wildcard-enum-match-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create wildcard enum Match workspace");
        Self { root }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.path(relative);
        fs::write(&path, contents).expect("write wildcard enum Match fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-wildcard-enum-match-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn parsed(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn semantic(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source))
        .map(|(_, ast)| ast)
}

fn checked_without_semantics(source: &str) -> Result<compiler::CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(parsed(source))
        .map_err(|error| error.to_string())
}

fn shared_rejection(label: &str, source: &str, expected: &[&str]) -> Vec<String> {
    let mut failures = Vec::new();
    for (route, result) in [
        ("semantic", semantic(source).map(|_| ())),
        (
            "checked admission",
            checked_without_semantics(source).map(|_| ()),
        ),
        (
            "public compilation",
            compile_program(source, CompilerOptions::default()).map(|_| ()),
        ),
    ] {
        match result {
            Ok(()) => failures.push(format!("{label}: {route} unexpectedly accepted source")),
            Err(error) if expected.iter().any(|fragment| error.contains(fragment)) => {}
            Err(error) => failures.push(format!(
                "{label}: {route} diagnostic {error:?} omitted every expected fragment {expected:?}"
            )),
        }
    }
    failures
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

#[test]
fn terminal_and_payload_wildcards_reach_both_trusted_routes() {
    let mut failures = Vec::new();

    for (label, source) in [
        ("terminal wildcard", TERMINAL_WILDCARD),
        ("payload wildcard", PAYLOAD_WILDCARD),
    ] {
        if let Err(error) = semantic(source) {
            failures.push(format!(
                "{label}: semantic analysis rejected parsed wildcard: {error}"
            ));
        }

        match checked_without_semantics(source) {
            Err(error) => failures.push(format!(
                "{label}: independent checked admission rejected parsed wildcard: {error}"
            )),
            Ok(checked) => {
                if let Err(error) = CodeGenerator::new().try_generate_code(checked) {
                    failures.push(format!(
                        "{label}: trusted lowering rejected wildcard IR: {error}"
                    ));
                }
            }
        }

        if let Err(error) = compile_program(source, CompilerOptions::default()) {
            failures.push(format!(
                "{label}: public compilation rejected wildcard: {error}"
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "CAP-008 wildcard enum Match red:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn complete_wildcard_class_composes_without_ignored_payload_extraction() {
    semantic(COMPLETE_PRODUCT).expect("complete wildcard product must pass semantics");
    let checked = checked_without_semantics(COMPLETE_PRODUCT)
        .expect("complete wildcard product must pass independent checked admission");
    let checked_debug = format!("{checked:#?}");
    let independent_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("verified wildcard product must lower");
    let public_llvm = compile_program(COMPLETE_PRODUCT, CompilerOptions::default())
        .expect("complete wildcard product must compile publicly");

    assert_eq!(
        independent_llvm, public_llvm,
        "trusted wildcard routes produced different LLVM"
    );
    assert!(
        checked_debug.matches("CheckedEnumDispatch").count() >= 7,
        "nested/user/carrier/generic wildcard dispatches were lost:\n{checked_debug}"
    );
    assert!(
        checked_debug.contains("CheckedEnumField") && checked_debug.contains("field_index: 0"),
        "mixed multi-field pattern lost its retained first-field extraction:\n{checked_debug}"
    );
    assert!(
        !checked_debug.contains("field_index: 1"),
        "ignored multi-field payload was extracted:\n{checked_debug}"
    );
    assert!(
        public_llvm.matches("switch i32").count() >= 7,
        "wildcard products did not retain checked enum dispatch:\n{public_llvm}"
    );
    assert!(
        !public_llvm.contains("__aero$carrier$") && !public_llvm.contains("__aero$generic_enum$"),
        "private normalized identities leaked into LLVM:\n{public_llvm}"
    );
}

#[test]
fn wildcard_mapping_and_pattern_failures_are_shared_and_closed() {
    let cases: [(&str, &str, &[&str]); 15] = [
        (
            "nonfinal wildcard",
            "enum E { A, B } fn main() -> int { match E::A { _ => 1, E::B => 2 } }",
            &["wildcard arm must be the final arm"],
        ),
        (
            "duplicate wildcard",
            "enum E { A, B } fn main() -> int { match E::A { _ => 1, _ => 2 } }",
            &["wildcard arm must be the final arm"],
        ),
        (
            "redundant wildcard",
            "enum E { A, B } fn main() -> int { match E::A { E::A => 1, E::B => 2, _ => 3 } }",
            &["wildcard arm is unreachable", "complete explicit coverage"],
        ),
        (
            "duplicate explicit variant",
            "enum E { A, B } fn main() -> int { match E::A { E::A => 1, E::A => 2, _ => 3 } }",
            &["duplicate variant `A`"],
        ),
        (
            "unknown explicit variant",
            "enum E { A, B } fn main() -> int { match E::A { E::Missing => 1, _ => 2 } }",
            &["has no variant `Missing`"],
        ),
        (
            "foreign explicit variant",
            "enum E { A, B } enum F { A } fn main() -> int { match E::A { F::A => 1, _ => 2 } }",
            &["names `F`, expected `E`"],
        ),
        (
            "incomplete explicit coverage",
            "enum E { A, B } fn main() -> int { match E::A { E::A => 1 } }",
            &["cover every declared variant exactly once"],
        ),
        (
            "wrong payload arity",
            "enum E { Pair(int, bool) } fn main() -> int { match E::Pair(1, 1 < 2) { E::Pair(value) => value } }",
            &["requires 2 payload field pattern(s), actual 1"],
        ),
        (
            "duplicate payload identifier",
            "enum E { Pair(int, int) } fn main() -> int { match E::Pair(1, 2) { E::Pair(value, value) => value } }",
            &["duplicate payload binding `value`"],
        ),
        (
            "nested payload pattern",
            "enum E { Pair((int, bool), int) } fn main() -> int { match E::Pair((1, 1 < 2), 2) { E::Pair((number, flag), _) => 0 } }",
            &["identifier bindings or `_` wildcards"],
        ),
        (
            "literal payload pattern",
            "enum E { Value(int) } fn main() -> int { match E::Value(1) { E::Value(1) => 0 } }",
            &["identifier bindings or `_` wildcards"],
        ),
        (
            "top-level binding pattern",
            "enum E { A, B } fn main() -> int { match E::A { value => 1 } }",
            &["explicit variant arms", "terminal `_` wildcard"],
        ),
        (
            "top-level literal pattern",
            "enum E { A, B } fn main() -> int { match E::A { 1 => 1 } }",
            &["explicit variant arms", "terminal `_` wildcard"],
        ),
        (
            "wildcard result mismatch",
            "enum E { A, B } fn main() { let result = match E::A { E::A => 1, _ => 1.5 }; }",
            &["result mismatch", "expected int, actual float"],
        ),
        (
            "wildcard arm reuses moved owner",
            "enum E { A, B } fn main() -> int { let value = E::A; match value { E::A => 1, _ => match value { E::A => 2, E::B => 3 } } }",
            &["arm reuses consumed scrutinee `value`"],
        ),
    ];

    let failures = cases
        .into_iter()
        .flat_map(|(label, source, expected)| shared_rejection(label, source, expected))
        .collect::<Vec<_>>();
    assert!(
        failures.is_empty(),
        "CAP-008 negative wildcard matrix:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn direct_module_and_cli_products_preserve_wildcard_artifact_contracts() {
    let workspace = TestWorkspace::new("direct-module-cli");
    let module = r#"
enum State { Idle, Ready(int), Failed(char) }

fn score(value: State) -> int {
    match value {
        State::Ready(number) => number,
        State::Failed(_) => 1,
        _ => 0,
    }
}
"#;
    let valid = workspace.write(
        "main.aero",
        "mod model; fn main() -> int { score(State::Ready(42)) }",
    );
    workspace.write("model.aero", module);

    check_file(&valid, CompilerOptions::default())
        .expect("file-aware public check must admit direct-module wildcards");
    let file_llvm = compile_file(&valid, CompilerOptions::default())
        .expect("file-aware public compilation must admit direct-module wildcards");
    assert!(
        file_llvm.contains("switch i32"),
        "direct-module wildcard Match lost checked dispatch:\n{file_llvm}"
    );

    let check = run_cli(&workspace, &[Path::new("check"), Path::new("main.aero")]);
    assert!(
        check.status.success(),
        "CLI check rejected direct-module wildcard product: {}",
        output_text(&check)
    );
    let artifact = workspace.path("valid.ll");
    let build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            Path::new("main.aero"),
            Path::new("-o"),
            &artifact,
        ],
    );
    assert!(
        build.status.success() && artifact.is_file(),
        "CLI build failed to publish valid wildcard LLVM: {}",
        output_text(&build)
    );

    workspace.write(
        "invalid.aero",
        "enum E { A, B } fn main() -> int { match E::A { _ => 1, E::B => 2 } }",
    );
    let invalid_artifact = workspace.path("invalid.ll");
    for command in ["check", "run"] {
        let output = run_cli(&workspace, &[Path::new(command), Path::new("invalid.aero")]);
        assert!(
            !output.status.success(),
            "invalid wildcard CLI {command} succeeded: {}",
            output_text(&output)
        );
    }
    let invalid_build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            Path::new("invalid.aero"),
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    assert!(
        !invalid_build.status.success() && !invalid_artifact.exists(),
        "invalid wildcard build published an artifact: {}",
        output_text(&invalid_build)
    );
}
