use compiler::ast::{AstNode, Expression, Pattern, Statement, Type, VariantDeclKind};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/scalar_payload_enum/main.aero";
const EXAMPLE_MODULE: &str = "examples/scalar_payload_enum/signals.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 181;

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
            "aero-scalar-payload-enum-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create scalar-payload enum workspace");
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
            .is_some_and(|name| name.starts_with("aero-scalar-payload-enum-"));
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

fn analyzed_ast(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    SemanticAnalyzer::new().analyze(ast).map(|(_, ast)| ast)
}

fn checked_ir_and_llvm(source: &str) -> Result<(compiler::CheckedIr, String), String> {
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed_ast(source)?)
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
            "{label}: unsupported scalar-payload enum source compiled:\n{llvm}"
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
enum Switch { Off, On }
enum Signal { Idle, Count(int), Ratio(float), Ready(bool) }
enum Scalar { Value(int) }

fn mark(value: int) -> int { value }

fn main() -> int {
    let seed = 30;
    let original: Signal = Signal::Count(seed + 7);
    let moved = original;
    let count = match moved {
        Signal::Ratio(ratio) => mark(1),
        Signal::Idle => mark(2),
        Signal::Ready(ready) => mark(3),
        Signal::Count(value) => value + mark(2)
    };

    let ratio: float = match Signal::Ratio(1.5 + 0.5) {
        Signal::Ready(flag) => 0.0,
        Signal::Count(value) => 0.0,
        Signal::Idle => 0.0,
        Signal::Ratio(value) => value + 0.5
    };
    let ready: bool = match Signal::Ready(4 > 3) {
        Signal::Count(value) => value > 0,
        Signal::Ratio(value) => value > 0.0,
        Signal::Ready(flag) => flag && 5 > 4,
        Signal::Idle => 1 > 2
    };
    let unit = match Signal::Idle {
        Signal::Idle => 4,
        Signal::Count(value) => value,
        Signal::Ratio(value) => 5,
        Signal::Ready(flag) => 6
    };
    let nested = match Scalar::Value(match Switch::On {
        Switch::Off => 8,
        Switch::On => 9
    }) {
        Scalar::Value(inner) => inner + 1
    };
    let direct = match Signal::Count(match Scalar::Value(11) {
        Scalar::Value(inner) => inner + 1
    }) {
        Signal::Idle => 0,
        Signal::Count(bound) => bound,
        Signal::Ratio(value) => 0,
        Signal::Ready(flag) => 0
    };

    if ready && ratio > 2.0 && count == 39 && unit == 4 && nested == 10
        && direct == 12 && "aero".len() == 4 {
        return count + 142;
    }
    return 1;
}
"#
}

#[test]
fn owned_scalar_payload_enum_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
enum Signal { Idle, Count(int), Ratio(float), Ready(bool) }
fn main() -> int {
    return match Signal::Count(41) {
        Signal::Idle => 0,
        Signal::Ratio(value) => 1,
        Signal::Ready(flag) => 2,
        Signal::Count(value) => value + 1
    };
}
"#;
    match try_tokenize_with_locations(parser_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("parser retention failed: {error}")),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::EnumDef { variants, .. })) = ast.first() else {
                failures.push(format!("parser omitted enum definition: {ast:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            let expected = [None, Some("int"), Some("float"), Some("bool")];
            for (variant, expected) in variants.iter().zip(expected) {
                match (&variant.kind, expected) {
                    (VariantDeclKind::Unit, None) => {}
                    (VariantDeclKind::Tuple(types), Some(name)) if matches!(types.as_slice(), [Type::Named(actual)] if actual == name) =>
                        {}
                    _ => failures.push(format!(
                        "parser changed payload declaration for {}: {:?}",
                        variant.name, variant.kind
                    )),
                }
            }
            let Some(AstNode::Statement(Statement::Function { body, .. })) = ast.get(1) else {
                failures.push(format!("parser omitted function body: {ast:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            let Some(Statement::Return(Some(Expression::Match { expr, arms }))) =
                body.statements.first()
            else {
                failures.push(format!("parser omitted returned Match: {body:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            if !matches!(
                expr.as_ref(),
                Expression::EnumVariant { enum_name, variant, data: Some(_) }
                    if enum_name == "Signal" && variant == "Count"
            ) || arms.len() != 4
                || !matches!(
                    arms[3].pattern,
                    Pattern::Enum { data: Some(ref data), .. }
                        if matches!(data.as_ref(), Pattern::Identifier(name) if name == "value")
                )
            {
                failures.push(format!(
                    "parser changed unary payload Match topology: {body:?}"
                ));
            }
        }
    }

    failures.extend(expect_success(
        "mixed scalar payload construction, moves, bound extraction, results, and nesting",
        positive_source(),
        &[
            "insertvalue { i32, double, i1 }",
            "extractvalue { i32, double, i1 }",
            "switch i32",
        ],
    ));

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked scalar-payload IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            for required in [
                "CheckedEnumVariant",
                "CheckedEnumPayload",
                "CheckedEnumDispatch",
                "EnumVariantSchema { name: \"Count\", payload: Some(Int) }",
                "EnumVariantSchema { name: \"Ratio\", payload: Some(Float) }",
                "EnumVariantSchema { name: \"Ready\", payload: Some(Bool) }",
            ] {
                if !debug.contains(required) {
                    failures.push(format!("checked enum IR missing {required:?}:\n{debug}"));
                }
            }
            if llvm.matches("insertvalue { i32, double, i1 }").count() < 6
                || llvm.matches("extractvalue { i32, double, i1 }").count() < 8
                || llvm.contains("EnumConstruct")
                || llvm.contains("EnumVariantData")
                || llvm.contains("EnumDiscriminant")
            {
                failures.push(format!("LLVM did not retain checked payload flow:\n{llvm}"));
            }
        }
    }

    for (label, source, expected) in [
        (
            "missing payload",
            "enum Signal { Idle, Count(int) } fn main() { let value = Signal::Count; }",
            "enum `Signal` variant `Count` requires one int payload",
        ),
        (
            "payload on unit",
            "enum Signal { Idle, Count(int) } fn main() { let value = Signal::Idle(1); }",
            "enum `Signal` variant `Idle` does not accept payload data",
        ),
        (
            "wrong Int payload",
            "enum Signal { Count(int) } fn main() { let value = Signal::Count(1.5); }",
            "enum `Signal` variant `Count` payload type mismatch: expected int, actual float",
        ),
        (
            "wrong Float payload",
            "enum Signal { Ratio(float) } fn main() { let value = Signal::Ratio(1); }",
            "enum `Signal` variant `Ratio` payload type mismatch: expected float, actual int",
        ),
        (
            "wrong Bool payload",
            "enum Signal { Ready(bool) } fn main() { let value = Signal::Ready(1); }",
            "enum `Signal` variant `Ready` payload type mismatch: expected bool, actual int",
        ),
        (
            "empty tuple declaration",
            "enum Signal { Empty() } fn main() { let value = Signal::Empty; }",
            "enum `Signal` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "multi payload declaration",
            "enum Signal { Pair(int, bool) } fn main() { let value = Signal::Pair(1); }",
            "enum `Signal` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "String payload declaration",
            "enum Signal { Text(String) } fn main() { let value = Signal::Text(\"x\"); }",
            "enum `Signal` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "struct variant declaration",
            "enum Signal { Named { value: int } } fn main() { let value = Signal::Named; }",
            "enum `Signal` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "mutable payload enum binding",
            "enum Signal { Count(int) } fn main() { let mut signal = Signal::Count(1); }",
            "mutable enum bindings are not admitted",
        ),
        (
            "missing payload pattern",
            "enum Signal { Idle, Count(int) } fn main() { let value = match Signal::Count(1) { Signal::Idle => 0, Signal::Count => 1 }; }",
            "enum match variant `Count` requires one identifier payload binding",
        ),
        (
            "payload pattern on unit",
            "enum Signal { Idle, Count(int) } fn main() { let value = match Signal::Idle { Signal::Idle(x) => 0, Signal::Count(value) => value }; }",
            "enum match variant `Idle` does not accept a payload binding",
        ),
        (
            "wildcard payload pattern",
            "enum Signal { Count(int) } fn main() { let value = match Signal::Count(1) { Signal::Count(_) => 0 }; }",
            "enum match variant `Count` requires one identifier payload binding",
        ),
        (
            "literal payload pattern",
            "enum Signal { Count(int) } fn main() { let value = match Signal::Count(1) { Signal::Count(1) => 0 }; }",
            "enum match variant `Count` requires one identifier payload binding",
        ),
        (
            "binding does not leak",
            "enum Signal { Count(int) } fn main() { let value = match Signal::Count(1) { Signal::Count(inner) => inner }; let leaked = inner; }",
            "Use of undeclared variable `inner`",
        ),
        (
            "binding cannot shadow consumed scrutinee",
            "enum Signal { Count(int) } fn main() { let signal = Signal::Count(1); let value = match signal { Signal::Count(signal) => signal }; }",
            "enum payload binding `signal` shadows consumed scrutinee",
        ),
        (
            "use after payload match",
            "enum Signal { Count(int) } fn main() { let signal = Signal::Count(1); let value = match signal { Signal::Count(inner) => inner }; let reused = signal; }",
            "Use of moved value `signal`",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    let raw_source = "enum Signal { Idle, Count(int) } fn main() { let value = match Signal::Count(4) { Signal::Idle => 0, Signal::Count(inner) => inner }; }";
    match try_tokenize_with_locations(raw_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("raw containment parse failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedEnumVariant")
                || debug.contains("CheckedEnumPayload")
                || debug.contains("CheckedEnumDispatch")
                || !debug.contains("ImmInt(0)")
            {
                failures.push(format!(
                    "raw path activated checked payload semantics: {debug}"
                ));
            }
        }
    }

    let workspace = TestWorkspace::new("cli");
    let valid = workspace.path("valid.aero");
    let invalid = workspace.path("invalid.aero");
    let valid_artifact = workspace.path("valid.ll");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(&valid, parser_source).expect("write valid payload-enum source");
    fs::write(
        &invalid,
        "enum Signal { Count(int) } fn main() { let value = Signal::Count(1.5); }",
    )
    .expect("write invalid payload-enum source");

    let valid_check = run_cli(&workspace, &[Path::new("check"), &valid]);
    if !valid_check.status.success() {
        failures.push(format!(
            "CLI check rejected valid payload enum: {}",
            output_text(&valid_check)
        ));
    }
    let valid_build = run_cli(
        &workspace,
        &[Path::new("build"), &valid, Path::new("-o"), &valid_artifact],
    );
    if !valid_build.status.success() || !valid_artifact.exists() {
        failures.push(format!(
            "CLI build failed to publish valid payload LLVM: {}",
            output_text(&valid_build)
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
    if invalid_build.status.success()
        || invalid_artifact.exists()
        || !output_text(&invalid_build).contains("payload type mismatch")
    {
        failures.push(format!(
            "CLI invalid payload did not fail closed without an artifact: {}",
            output_text(&invalid_build)
        ));
    }

    let root = repository_root();
    for relative in [EXAMPLE_ROOT, EXAMPLE_MODULE] {
        let path = root.join(relative);
        if !path.is_file() {
            failures.push(format!(
                "tracked integration file missing: {}",
                path.display()
            ));
        }
    }
    let workflow_path = root.join(WORKFLOW);
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not read stable workflow {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test owned scalar-payload enum integration example",
                "examples/scalar_payload_enum/main.aero",
                "opt-22 -passes=verify -disable-output ../../scalar_payload_enum.ll",
                "llc-22 -verify-machineinstrs ../../scalar_payload_enum.ll",
                "clang-22 ../../scalar_payload_enum.o -o ../../scalar_payload_enum",
                "Expected exit code 181",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }
    if EXPECTED_EXIT != 181 {
        failures.push("scalar-payload enum native exit contract drifted".to_string());
    }

    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
