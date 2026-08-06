use compiler::ast::{Parameter, Type};
use compiler::semantic_analyzer::{FunctionInfo, FunctionTable};
use compiler::types::Ty;
use compiler::{
    CodeGenerator, IrGenerator, SemanticAnalyzer, parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/function_call_contract/main.aero";
const EXAMPLE_MODULE: &str = "examples/function_call_contract/calls.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 181;
const SHARED_DIAGNOSTIC: &str = "Unsupported function call";

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
            "aero-function-call-contract-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create function-call contract workspace");
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
            .is_some_and(|name| name.starts_with("aero-function-call-contract-"));
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

fn parsed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source)?)
        .map(|(_, ast)| ast)
}

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed(source)?)
        .map_err(|error| error.to_string())?;
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .map_err(|error| error.to_string())?;
    Ok((checked, llvm))
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

fn complete_source() -> &'static str {
    r#"
struct Cell { value: int, flags: [bool; 2] }
enum Payload { Idle, Item(Cell) }

fn make_cell(value: int) -> Cell {
    Cell { value: value, flags: [value > 0, value < 0] }
}

fn roundtrip(value: Cell) -> Cell { value }

fn score(value: Payload) -> int {
    match value {
        Payload::Idle => 0,
        Payload::Item(cell) => cell.value + cell.flags.len()
    }
}

fn read(value: &int) -> int { *value }

fn bump(value: &mut int) -> int {
    *value = *value + 1;
    *value
}

fn observe(value: int) { let seen = value; }

fn countdown(value: int) -> int {
    if value == 0 { return 0; }
    countdown(value - 1) + 1
}

fn loop_score() -> int {
    let mut total = 0;
    for item in [1, 2] {
        let fresh = Payload::Item(roundtrip(make_cell(item)));
        total = total + score(fresh);
    }
    total
}

fn main() -> int {
    let base = 170;
    let seen = read(&base);
    let mut adjusted = seen;
    let changed = bump(&mut adjusted);
    observe(changed);
    let flags = [1 < 2, 2 < 1];
    if seen == 170
        && changed == 171
        && adjusted == 171
        && score(Payload::Item(roundtrip(make_cell(3)))) == 5
        && loop_score() == 7
        && countdown(2) == 2
        && flags.len() == 2
        && !flags.is_empty()
        && [1 < 2; 0].is_empty()
    {
        return 181;
    }
    1
}
"#
}

fn semantic_rejection(label: &str, source: &str, function: &str) -> Option<String> {
    match analyzed(source) {
        Ok(ast) => Some(format!(
            "{label}: semantic analysis fabricated a result for function {function:?}: {ast:#?}"
        )),
        Err(error) if error.contains(SHARED_DIAGNOSTIC) && error.contains(function) => None,
        Err(error) => Some(format!(
            "{label}: semantic diagnostic {error:?} omitted {SHARED_DIAGNOSTIC:?} and function {function:?}"
        )),
    }
}

#[test]
fn function_call_class_is_shared_fail_closed_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!("function call syntax was not retained: {error}"));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!(
            "multi-capability function-call program failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            for marker in ["Call {", "CheckedStruct", "CheckedEnum"] {
                if !debug.contains(marker) {
                    failures.push(format!(
                        "checked function-call IR omitted {marker:?}:\n{debug}"
                    ));
                }
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "function-call LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "function-call LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic function-call compilation failed: {error}"
                )),
            }
        }
    }

    for (label, source, function) in [
        (
            "inferred binding",
            "fn main() { let value = missing(); }",
            "missing",
        ),
        (
            "explicit annotation",
            "fn main() { let value: int = missing(); }",
            "missing",
        ),
        (
            "comparison operand",
            "fn main() { if missing() == 0 { return; } }",
            "missing",
        ),
        (
            "nested argument",
            "fn take(value: int) {} fn main() { take(missing()); }",
            "missing",
        ),
        (
            "return expression",
            "fn probe() -> int { return missing(); } fn main() {}",
            "missing",
        ),
        (
            "array element",
            "fn main() { let values = [missing()]; }",
            "missing",
        ),
        (
            "struct field",
            "struct S { value: int } fn main() { let item = S { value: missing() }; }",
            "missing",
        ),
        (
            "condition",
            "fn main() { if missing() { return; } }",
            "missing",
        ),
        (
            "wrong arity",
            "fn probe(value: int) -> int { value } fn main() { let bad = probe(); }",
            "probe",
        ),
        (
            "wrong argument type",
            "fn probe(value: int) -> int { value } fn main() { let bad = probe(1 < 2); }",
            "probe",
        ),
        (
            "void result in value position",
            "fn observe(value: int) {} fn main() { let bad = observe(1); }",
            "observe",
        ),
    ] {
        if let Some(failure) = semantic_rejection(label, source, function) {
            failures.push(failure);
        }
    }

    let mut legacy = FunctionTable::new();
    for (name, param_type) in [
        ("unknown_named", Type::Named("NotAdmitted".to_string())),
        (
            "fixed_array",
            Type::Array(Box::new(Type::Named("int".to_string())), 2),
        ),
        ("tuple", Type::Tuple(vec![Type::Named("int".to_string())])),
        (
            "reference",
            Type::Reference(Box::new(Type::Named("int".to_string())), false),
        ),
        (
            "generic",
            Type::Generic("Vec".to_string(), vec![Type::Named("int".to_string())]),
        ),
    ] {
        legacy
            .define_function(FunctionInfo {
                name: name.to_string(),
                parameters: vec![Parameter {
                    name: "value".to_string(),
                    param_type,
                }],
                return_type: Ty::Bool,
                defined_at: Some("function_call_contract_tests.rs".to_string()),
            })
            .expect("define legacy function contract");
        match legacy.validate_call(name, &[Ty::Int]) {
            Ok(result) => failures.push(format!(
                "legacy signature validator fabricated an int-compatible parameter for {name:?} and returned {result}"
            )),
            Err(error) if error.contains(SHARED_DIAGNOSTIC) && error.contains(name) => {}
            Err(error) => failures.push(format!(
                "legacy signature validator diagnostic {error:?} omitted {SHARED_DIAGNOSTIC:?} and {name:?}"
            )),
        }
    }

    let unresolved = "fn main() -> int { return missing(); }";
    match parsed(unresolved)
        .and_then(|ast| IrGenerator::new().try_generate_ir(ast).map_err(|error| error.to_string()))
    {
        Ok(checked) => failures.push(format!(
            "raw checked-IR admission accepted unresolved function call: {checked:#?}"
        )),
        Err(error) if error.contains(SHARED_DIAGNOSTIC) && error.contains("missing") => {}
        Err(error) => failures.push(format!(
            "raw checked-IR diagnostic {error:?} omitted {SHARED_DIAGNOSTIC:?} and function \"missing\""
        )),
    }

    let root = repository_root();
    let classifier = root.join("src/compiler/src/function_call_contract.rs");
    match fs::read_to_string(&classifier) {
        Err(error) => failures.push(format!(
            "shared function-call classifier is absent at {}: {error}",
            classifier.display()
        )),
        Ok(contents) => {
            for anchor in [
                "FunctionCallDisposition",
                "classify_function_call",
                "Supported",
                "ExplicitlyRejected",
                "PreservedContext",
            ] {
                if !contents.contains(anchor) {
                    failures.push(format!("shared classifier missing anchor {anchor:?}"));
                }
            }
        }
    }
    for path in [
        root.join("src/compiler/src/semantic_analyzer.rs"),
        root.join("src/compiler/src/ir_generator.rs"),
    ] {
        match fs::read_to_string(&path) {
            Ok(contents) if contents.contains("classify_function_call") => {}
            Ok(_) => failures.push(format!(
                "{} does not consume the shared function-call classifier",
                path.display()
            )),
            Err(error) => failures.push(format!("could not read {}: {error}", path.display())),
        }
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(&invalid, "fn main() -> int { return missing(); }")
        .expect("write invalid function-call source");
    for route in ["check", "run"] {
        let output = run_cli(&workspace, &[Path::new(route), &invalid]);
        if output.status.success() || !output_text(&output).contains(SHARED_DIAGNOSTIC) {
            failures.push(format!(
                "invalid function call CLI {route} did not fail through the shared diagnostic: {}",
                output_text(&output)
            ));
        }
    }
    let invalid_build = run_cli(
        &workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_artifact,
        ],
    );
    if invalid_build.status.success() || invalid_artifact.exists() {
        failures.push(format!(
            "invalid function call CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    for path in [root.join(EXAMPLE_ROOT), root.join(EXAMPLE_MODULE)] {
        if !path.is_file() {
            failures.push(format!(
                "tracked function-call system example missing: {}",
                path.display()
            ));
        }
    }

    let workflow_path = root.join(WORKFLOW);
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not read {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test function call contract integration example",
                "examples/function_call_contract/main.aero",
                "opt-22 -passes=verify -disable-output ../../function_call_contract.ll",
                "llc-22 -verify-machineinstrs ../../function_call_contract.ll",
                "clang-22 -no-pie ../../function_call_contract.o -o ../../function_call_contract",
                "Expected exit code 181",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-068 function-call contract failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
