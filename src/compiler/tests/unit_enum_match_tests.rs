use compiler::ast::{AstNode, Expression, Pattern, Statement, VariantDeclKind};
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/unit_enum_matches/main.aero";
const EXAMPLE_MODULE: &str = "examples/unit_enum_matches/signals.aero";
const WORKFLOW: &str = ".github/workflows/rust.yml";
const EXPECTED_EXIT: i32 = 149;

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
            "aero-unit-enum-match-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create unit-enum match workspace");
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
            .is_some_and(|name| name.starts_with("aero-unit-enum-match-"));
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
            "{label}: unsupported unit-enum source compiled:\n{llvm}"
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
enum Phase { Cold, Warm, Hot }

fn mark(value: int) -> int { value }

fn choose() -> int {
    let phase: Phase = Phase::Warm;
    let moved = phase;
    return match moved {
        Phase::Hot => mark(91),
        Phase::Cold => mark(17),
        Phase::Warm => match Switch::On {
            Switch::On => mark(41),
            Switch::Off => mark(23)
        }
    };
}

fn main() -> int {
    let integer = match Phase::Hot {
        Phase::Warm => 20,
        Phase::Hot => choose(),
        Phase::Cold => 10
    };
    let ratio: float = match Switch::On {
        Switch::Off => 0.5,
        Switch::On => 1.5
    };
    let ready: bool = match Phase::Warm {
        Phase::Cold => 1 > 2,
        Phase::Hot => 2 < 1,
        Phase::Warm => 2 > 1
    };
    let values = [3, match Switch::Off { Switch::Off => 4, Switch::On => 5 }];
    println!("selected {}", match Switch::On { Switch::Off => 0, Switch::On => integer });
    if ready && ratio > 1.0
        && match Switch::On { Switch::Off => 1 < 0, Switch::On => values[1] == 4 } {
        return integer + 108;
    }
    return 1;
}
"#
}

#[test]
fn owned_unit_enum_match_class_is_complete_checked_and_executable() {
    let mut failures = Vec::new();

    let parser_source = r#"
enum Phase { Cold, Warm, Hot }
fn main() -> int {
    return match Phase::Warm {
        Phase::Hot => 3,
        Phase::Cold => 1,
        Phase::Warm => 2
    };
}
"#;
    match try_tokenize_with_locations(parser_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("parser retention failed: {error}")),
        Ok(ast) => {
            let Some(AstNode::Statement(Statement::EnumDef {
                name,
                variants,
                type_params,
            })) = ast.first()
            else {
                failures.push(format!("parser omitted enum definition: {ast:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            if name != "Phase"
                || !type_params.is_empty()
                || variants.len() != 3
                || variants
                    .iter()
                    .any(|variant| !matches!(variant.kind, VariantDeclKind::Unit))
            {
                failures.push(format!("parser changed unit-enum shape: {ast:?}"));
            }
            let Some(AstNode::Statement(Statement::Function { body, .. })) = ast.get(1) else {
                failures.push(format!("parser omitted function body: {ast:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            let Some(Statement::Return(Some(Expression::Match { expr, arms }))) =
                body.statements.first()
            else {
                failures.push(format!("parser omitted returned match: {body:?}"));
                return assert!(failures.is_empty(), "{}", failures.join("\n\n"));
            };
            if !matches!(
                expr.as_ref(),
                Expression::EnumVariant { enum_name, variant, data: None }
                    if enum_name == "Phase" && variant == "Warm"
            ) || arms.len() != 3
                || !arms
                    .iter()
                    .all(|arm| matches!(arm.pattern, Pattern::Enum { data: None, .. }))
            {
                failures.push(format!("parser changed match topology: {body:?}"));
            }
        }
    }

    failures.extend(expect_success(
        "all unit-enum result types, parent contexts, moves, arm orders, and nesting",
        positive_source(),
        &["switch i32", "match_arm_", "match_end_", "call i32 @mark"],
    ));

    match checked_ir_and_llvm(positive_source()) {
        Err(error) => failures.push(format!("checked enum IR/LLVM failed: {error}")),
        Ok((checked, llvm)) => {
            let debug = format!("{checked:?}");
            if debug.matches("CheckedEnumVariant").count() < 7 {
                failures.push(format!("checked IR lost unit variants:\n{debug}"));
            }
            if debug.matches("CheckedEnumDispatch").count() < 7 {
                failures.push(format!("checked IR lost exhaustive dispatches:\n{debug}"));
            }
            for schema in [
                "EnumSchema { name: \"Phase\", variants: [EnumVariantSchema { name: \"Cold\", payload: None }, EnumVariantSchema { name: \"Warm\", payload: None }, EnumVariantSchema { name: \"Hot\", payload: None }] }",
                "EnumSchema { name: \"Switch\", variants: [EnumVariantSchema { name: \"Off\", payload: None }, EnumVariantSchema { name: \"On\", payload: None }] }",
            ] {
                if !debug.contains(schema) {
                    failures.push(format!("checked metadata missing {schema:?}:\n{debug}"));
                }
            }
            if llvm.matches("switch i32").count() < 7
                || llvm.contains("EnumConstruct")
                || llvm.contains("EnumDiscriminant")
            {
                failures.push(format!(
                    "LLVM did not retain checked enum dispatch:\n{llvm}"
                ));
            }
        }
    }

    for (label, source, expected) in [
        (
            "unknown enum",
            "fn main() { let value = Missing::Only; }",
            "enum `Missing` has no unique admitted definition",
        ),
        (
            "unknown variant",
            "enum Phase { Cold, Warm } fn main() { let value = Phase::Hot; }",
            "enum `Phase` has no variant `Hot`",
        ),
        (
            "payload on unit variant",
            "enum Phase { Cold, Warm } fn main() { let value = Phase::Warm(1); }",
            "enum `Phase` variant `Warm` does not accept payload data",
        ),
        (
            "struct variant definition",
            "enum Phase { Cold, Warm { value: int } } fn main() { let value = Phase::Cold; }",
            "enum `Phase` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "generic definition",
            "enum Phase<T> { Cold, Warm } fn main() { let value = Phase::Cold; }",
            "enum `Phase` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "empty definition",
            "enum Phase {} fn main() { let value = Phase::Cold; }",
            "enum `Phase` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "duplicate variant",
            "enum Phase { Cold, Cold } fn main() { let value = Phase::Cold; }",
            "enum `Phase` is not an admitted non-generic unit-or-unary-scalar enum",
        ),
        (
            "duplicate enum definition",
            "enum Phase { Cold } enum Phase { Cold } fn main() { let value = Phase::Cold; }",
            "enum `Phase` has no unique admitted definition",
        ),
        (
            "struct enum collision",
            "struct Phase { value: int } enum Phase { Cold } fn main() { let value = Phase::Cold; }",
            "enum `Phase` has no unique admitted definition",
        ),
        (
            "annotation mismatch",
            "enum Phase { Cold } enum Other { Cold } fn main() { let value: Other = Phase::Cold; }",
            "enum binding annotation mismatch: expected Phase, actual Other",
        ),
        (
            "mutable binding",
            "enum Phase { Cold } fn main() { let mut value = Phase::Cold; }",
            "mutable enum bindings are not admitted",
        ),
        (
            "missing arm",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1 }; }",
            "enum match must cover every declared variant exactly once",
        ),
        (
            "duplicate arm",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, Phase::Cold => 2 }; }",
            "enum match contains duplicate variant `Cold`",
        ),
        (
            "unknown arm",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, Phase::Hot => 2 }; }",
            "enum `Phase` has no variant `Hot`",
        ),
        (
            "foreign arm",
            "enum Phase { Cold, Warm } enum Other { Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, Other::Warm => 2 }; }",
            "enum match arm names `Other`, expected `Phase`",
        ),
        (
            "payload arm",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, Phase::Warm(x) => 2 }; }",
            "enum match variant `Warm` does not accept a payload binding",
        ),
        (
            "wildcard arm",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, _ => 2 }; }",
            "enum match requires one explicit variant arm per declared variant",
        ),
        (
            "binding arm",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, other => 2 }; }",
            "enum match requires one explicit variant arm per declared variant",
        ),
        (
            "result mismatch",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1, Phase::Warm => 1.5 }; }",
            "enum match arm result mismatch: expected int, actual float",
        ),
        (
            "unsupported result",
            "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => \"cold\", Phase::Warm => \"warm\" }; }",
            "enum match arms must return Int, Float, or Bool",
        ),
        (
            "use after match",
            "enum Phase { Cold, Warm } fn main() { let phase = Phase::Cold; let value = match phase { Phase::Cold => 1, Phase::Warm => 2 }; let reused = phase; }",
            "Use of moved value `phase`",
        ),
        (
            "arm reuses consumed scrutinee",
            "enum Phase { Cold, Warm } fn main() { let phase = Phase::Cold; let value = match phase { Phase::Cold => match phase { Phase::Cold => 1, Phase::Warm => 2 }, Phase::Warm => 3 }; }",
            "enum match arm reuses consumed scrutinee `phase`",
        ),
        (
            "duplicate consumption",
            "enum Phase { Cold, Warm } fn pair(a: int, b: int) -> int { a + b } fn main() { let phase = Phase::Cold; let value = pair(match phase { Phase::Cold => 1, Phase::Warm => 2 }, match phase { Phase::Cold => 3, Phase::Warm => 4 }); }",
            "enum `phase` is consumed more than once in one expression",
        ),
        (
            "closure context fails closed before match classification",
            "enum Phase { Cold, Warm } fn main() { let choose = |seed: int| match Phase::Cold { Phase::Cold => seed, Phase::Warm => 2 }; let value = choose(1); }",
            "closure expressions are parsed but unsupported in executable code",
        ),
        (
            "numeric match remains excluded",
            "fn main() { let value = match 1 { 1 => 2, _ => 3 }; }",
            "Match expressions are not supported",
        ),
        (
            "option match remains excluded",
            "fn main() { let value = match Option::Some(1) { _ => 0 }; }",
            "Match expressions are not supported",
        ),
    ] {
        if let Some(failure) = expect_rejection(label, source, expected) {
            failures.push(failure);
        }
    }

    let raw_source = "enum Phase { Cold, Warm } fn main() { let value = match Phase::Warm { Phase::Cold => 11, Phase::Warm => 22 }; }";
    match try_tokenize_with_locations(raw_source, None)
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()))
    {
        Err(error) => failures.push(format!("raw containment parse failed: {error}")),
        Ok(ast) => {
            let raw = IrGenerator::new().generate_ir(ast);
            let debug = format!("{raw:?}");
            if debug.contains("CheckedEnumVariant")
                || debug.contains("CheckedEnumDispatch")
                || !debug.contains("ImmInt(0)")
            {
                failures.push(format!(
                    "deprecated raw path activated checked enum semantics: {debug}"
                ));
            }
        }
    }

    let workspace = TestWorkspace::new("cli");
    let valid = workspace.path("valid.aero");
    let invalid = workspace.path("invalid.aero");
    let valid_artifact = workspace.path("valid.ll");
    let invalid_artifact = workspace.path("invalid.ll");
    fs::write(&valid, parser_source).expect("write valid unit-enum source");
    fs::write(
        &invalid,
        "enum Phase { Cold, Warm } fn main() { let value = match Phase::Cold { Phase::Cold => 1 }; }",
    )
    .expect("write invalid unit-enum source");

    let valid_check = run_cli(&workspace, &[Path::new("check"), &valid]);
    if !valid_check.status.success() {
        failures.push(format!(
            "CLI check rejected valid unit-enum match: {}",
            output_text(&valid_check)
        ));
    }
    let valid_build = run_cli(
        &workspace,
        &[Path::new("build"), &valid, Path::new("-o"), &valid_artifact],
    );
    if !valid_build.status.success() || !valid_artifact.exists() {
        failures.push(format!(
            "CLI build failed to publish valid LLVM: {}",
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
        || !output_text(&invalid_build)
            .contains("enum match must cover every declared variant exactly once")
    {
        failures.push(format!(
            "CLI invalid match did not fail closed without an artifact: {}",
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
                "Test owned unit-enum match integration example",
                "examples/unit_enum_matches/main.aero",
                "opt-22 -passes=verify -disable-output ../../unit_enum_matches.ll",
                "llc-22 -verify-machineinstrs ../../unit_enum_matches.ll",
                "clang-22 ../../unit_enum_matches.o -o ../../unit_enum_matches",
                "Expected exit code 149",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!("stable workflow missing {anchor:?}"));
                }
            }
        }
    }
    if EXPECTED_EXIT != 149 {
        failures.push("unit-enum native exit contract drifted".to_string());
    }

    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}
