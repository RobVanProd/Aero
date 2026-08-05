use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/intrinsic_method_contract/main.aero";
const EXAMPLE_MODULE: &str = "examples/intrinsic_method_contract/queries.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 167;
const SHARED_DIAGNOSTIC: &str = "Unsupported intrinsic method call";

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
            "aero-intrinsic-method-contract-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create intrinsic method contract workspace");
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
            .is_some_and(|name| name.starts_with("aero-intrinsic-method-contract-"));
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
struct Leaf { value: int, active: bool }
struct Holder {
    flags: [bool; 2],
    pairs: [(int, bool); 2],
    matrix: [[int; 2]; 2],
    leaves: [Leaf; 2],
    empty: [[bool; 2]; 0]
}

fn leaf(value: int) -> Leaf {
    Leaf { value: value, active: value > 0 }
}

fn make_flags() -> [bool; 2] {
    [1 < 2, 2 < 1]
}

fn inspect_arrays(
    empty: [bool; 0],
    pairs: [(int, bool); 2],
    matrix: [[int; 2]; 2],
    leaves: [Leaf; 2]
) -> int {
    if empty.is_empty()
        && !pairs.is_empty()
        && !matrix.is_empty()
        && !leaves.is_empty()
    {
        return empty.len() + pairs.len() + matrix.len() + leaves.len();
    }
    0
}

fn inspect_holder(value: Holder) -> int {
    if value.empty.is_empty()
        && !value.flags.is_empty()
        && !value.pairs.is_empty()
        && !value.matrix.is_empty()
        && !value.leaves.is_empty()
    {
        return value.empty.len()
            + value.flags.len()
            + value.pairs.len()
            + value.matrix.len()
            + value.leaves.len();
    }
    0
}

fn main() -> int {
    let first = leaf(5);
    let second = leaf(6);
    let pairs = [(3, 1 < 2), (4, 2 < 1)];
    let matrix = [[1, 2], [3, 4]];
    let leaves = [first, second];
    let holder = Holder {
        flags: make_flags(),
        pairs: pairs,
        matrix: matrix,
        leaves: leaves,
        empty: []
    };
    let empty: [bool; 0] = [];
    let total = inspect_arrays(empty, pairs, matrix, leaves)
        + inspect_holder(holder)
        + make_flags().len()
        + [[1, 2]].len();
    let text: String = "Aero 🚀";
    if total == 17
        && !make_flags().is_empty()
        && [1 < 2; 0].is_empty()
        && !text.is_empty()
        && text.contains("ero")
        && text.starts_with("Aero")
        && text.ends_with("🚀")
        && text.len() == 6
    {
        return 167;
    }
    1
}
"#
}

fn semantic_rejection(label: &str, source: &str, method: &str) -> Option<String> {
    match analyzed(source) {
        Ok(ast) => Some(format!(
            "{label}: semantic analysis fabricated a method result for {method:?}: {ast:#?}"
        )),
        Err(error) if error.contains(SHARED_DIAGNOSTIC) && error.contains(method) => None,
        Err(error) => Some(format!(
            "{label}: semantic diagnostic {error:?} omitted {SHARED_DIAGNOSTIC:?} and method {method:?}"
        )),
    }
}

#[test]
fn intrinsic_method_class_is_shared_fail_closed_and_executable() {
    let mut failures = Vec::new();
    let source = complete_source();

    if let Err(error) = parsed(source) {
        failures.push(format!("intrinsic method syntax was not retained: {error}"));
    }

    match checked_ir_and_llvm(source) {
        Err(error) => failures.push(format!(
            "recursive CopyData array len/is_empty program failed: {error}"
        )),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:#?}");
            if !debug.contains("ICmp") || !debug.contains("CheckedStruct") {
                failures.push(format!(
                    "checked intrinsic query IR lost Bool/struct evidence:\n{debug}"
                ));
            }
            for forbidden in ["bitcast", "inttoptr", "ptrtoint"] {
                if llvm.contains(forbidden) {
                    failures.push(format!(
                        "intrinsic method LLVM contains forbidden fallback {forbidden:?}:\n{llvm}"
                    ));
                }
            }
            match checked_ir_and_llvm(source) {
                Ok((_, second)) if second == llvm => {}
                Ok((_, second)) => failures.push(format!(
                    "intrinsic method LLVM was nondeterministic:\nFIRST\n{llvm}\nSECOND\n{second}"
                )),
                Err(error) => failures.push(format!(
                    "second deterministic intrinsic-method compilation failed: {error}"
                )),
            }
        }
    }

    for (label, source, method) in [
        (
            "unknown scalar method",
            "fn main() { let value = (1).missing(); }",
            "missing",
        ),
        (
            "unknown bool method",
            "fn main() { let value = (1 < 2).missing(); }",
            "missing",
        ),
        (
            "unknown array method",
            "fn main() { let values = [1, 2]; let value = values.capacity(); }",
            "capacity",
        ),
        (
            "unsupported String transform",
            "fn main() { let text = \" Aero \"; let value = text.trim(); }",
            "trim",
        ),
        (
            "unknown struct method",
            "struct S { value: int } fn main() { let item = S { value: 1 }; let value = item.missing(); }",
            "missing",
        ),
        (
            "unknown tuple method",
            "fn main() { let item = (1, 1 < 2); let value = item.missing(); }",
            "missing",
        ),
        (
            "method nested in comparison",
            "fn main() { if [1, 2].missing() == 0 { return; } }",
            "missing",
        ),
        (
            "method nested in call argument",
            "fn take(value: int) {} fn main() { take(\"a\".trim()); }",
            "trim",
        ),
        (
            "method nested in return",
            "fn probe() -> int { return (1).missing(); } fn main() {}",
            "missing",
        ),
        (
            "method nested in array",
            "fn main() { let values = [(1).missing()]; }",
            "missing",
        ),
        (
            "chained scalar method",
            "fn main() { let value = [1, 2].len().missing(); }",
            "missing",
        ),
    ] {
        if let Some(failure) = semantic_rejection(label, source, method) {
            failures.push(failure);
        }
    }

    for (label, source, expected) in [
        (
            "array is_empty wrong arity",
            "fn main() { let value = [1, 2].is_empty(1); }",
            "expects exactly 0 arguments, got 1",
        ),
        (
            "runtime String parameter is rejected before its method body",
            "fn probe(text: String) -> bool { return text.is_empty(); } fn main() -> int { 0 }",
            "function parameter `text` is not an admitted scalar type",
        ),
        (
            "non-static String argument is rejected at its parameter contract",
            "fn probe(text: String) -> bool { return \"a\".contains(text); } fn main() -> int { 0 }",
            "function parameter `text` is not an admitted scalar type",
        ),
    ] {
        match compile_program(source, CompilerOptions::default()) {
            Ok(llvm) => failures.push(format!(
                "{label}: excluded method topology compiled:\n{llvm}"
            )),
            Err(error) if error.contains(expected) => {}
            Err(error) => failures.push(format!(
                "{label}: diagnostic {error:?} omitted {expected:?}"
            )),
        }
    }

    let root = repository_root();
    let classifier = root.join("src/compiler/src/method_call_contract.rs");
    match fs::read_to_string(&classifier) {
        Err(error) => failures.push(format!(
            "shared intrinsic-method classifier is absent at {}: {error}",
            classifier.display()
        )),
        Ok(contents) => {
            for anchor in [
                "IntrinsicMethodDisposition",
                "classify_intrinsic_method",
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
    for (path, consumer) in [
        (
            root.join("src/compiler/src/semantic_analyzer.rs"),
            "classify_intrinsic_method",
        ),
        (
            root.join("src/compiler/src/ir_generator.rs"),
            "classify_intrinsic_method",
        ),
    ] {
        match fs::read_to_string(&path) {
            Ok(contents) if contents.contains(consumer) => {}
            Ok(_) => failures.push(format!(
                "{} does not consume the shared intrinsic classifier",
                path.display()
            )),
            Err(error) => failures.push(format!("could not read {}: {error}", path.display())),
        }
    }

    let workspace = TestWorkspace::new("cli");
    let invalid = workspace.path("invalid.aero");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "fn main() { let values = [1, 2]; let bad = values.capacity(); }",
    )
    .expect("write invalid intrinsic method source");
    let invalid_check = run_cli(&workspace, &[Path::new("check"), &invalid]);
    if invalid_check.status.success() || !output_text(&invalid_check).contains(SHARED_DIAGNOSTIC) {
        failures.push(format!(
            "invalid intrinsic method CLI check did not fail through the shared diagnostic: {}",
            output_text(&invalid_check)
        ));
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
            "invalid intrinsic method CLI build did not fail without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    for path in [root.join(EXAMPLE_ROOT), root.join(EXAMPLE_MODULE)] {
        if !path.is_file() {
            failures.push(format!(
                "tracked intrinsic method example missing: {}",
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
                "Test intrinsic method contract integration example",
                "examples/intrinsic_method_contract/main.aero",
                "opt-22 -passes=verify -disable-output ../../intrinsic_method_contract.ll",
                "llc-22 -verify-machineinstrs ../../intrinsic_method_contract.ll",
                "clang-22 -no-pie ../../intrinsic_method_contract.o -o ../../intrinsic_method_contract",
                "Expected exit code 167",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "CORE-067 intrinsic method contract failures (expected exit {EXPECTED_EXIT}):\n{}",
        failures.join("\n---\n")
    );
}
