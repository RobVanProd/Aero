use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_DIRECTORY: &str = "examples/representative_telemetry";
const WORKFLOW: &str = ".github/workflows/rust.yml";

const MAIN_SOURCE: &str = r#"mod model;
mod policy;

fn main() -> int {
    const BASE: int = 10 + 1;
    const EXPECTED: int = 91;

    let mut batch: Batch = make_batch();
    let calibration = [BASE, 20, 30];
    let mut sensor_index = 0;
    while sensor_index < 3 {
        batch.sensors[sensor_index].value = calibration[sensor_index];
        sensor_index = sensor_index + 1;
    }
    batch.meta.0 = 4;
    batch.meta.1 = 2 > 1;

    let first_sensor_index = 0;
    let observed = batch.sensors[first_sensor_index].value;
    let observed_ref = &observed;
    let bias = batch.meta.0;
    let mut total = batch.sensors[1].value + batch.sensors[2].value;
    total = add_bias(&mut total, observed_ref, bias);

    for weight in [1, 2, 3] {
        total = total + weight;
    }
    let mut step = 0;
    while step < 2 {
        total = total + step + 1;
        step = step + 1;
    }
    if batch.meta.1 {
        total = total + 16;
    } else {
        total = 1;
    }

    let trusted = batch.sensors[first_sensor_index].trusted;
    let decision = classify(total, trusted);
    let score = decision_score(decision);
    println!("telemetry score: {}", score);
    if score == EXPECTED { return EXPECTED; }
    1
}
"#;

const MODEL_SOURCE: &str = r#"struct Sensor { value: int, trusted: bool }
struct Batch { sensors: [Sensor; 3], meta: (int, bool) }

enum Decision {
    Normal(int),
    Alert(int, bool)
}

fn make_sensor(value: int, trusted: bool) -> Sensor {
    Sensor { value: value, trusted: trusted }
}

fn make_batch() -> Batch {
    let seed = make_sensor(0, 1 < 2);
    Batch {
        sensors: [seed, seed, seed],
        meta: (0, 1 > 2)
    }
}
"#;

const POLICY_SOURCE: &str = r#"fn add_bias(target: &mut int, observed: &int, bias: int) -> int {
    *target = *target + *observed + bias;
    *target
}

fn classify(total: int, trusted: bool) -> Decision {
    if trusted && total > 50 { return Decision::Alert(total, 1 < 2); }
    Decision::Normal(total)
}

fn add_urgent(urgent: bool, value: int) -> int {
    if urgent { return value + 1; }
    value
}

fn decision_score(decision: Decision) -> int {
    match decision {
        Decision::Normal(value) => value,
        Decision::Alert(value, urgent) => add_urgent(urgent, value)
    }
}
"#;

const NUMERIC_PRINT_ABI_SOURCE: &str = r#"fn main() -> int {
    let integer = 7;
    let floating = 2.5;
    let truth = integer > 3;
    print!("{} {} {}", integer, floating, truth);
    println!(" {}", 4);
    0
}
"#;

const IMMUTABLE_PROJECTED_UPDATE_SOURCE: &str = r#"struct Sensor { value: int, trusted: bool }

fn main() -> int {
    let sensor = Sensor { value: 1, trusted: 1 < 2 };
    sensor.value = 2;
    sensor.value
}
"#;

const UNKNOWN_PROJECTED_FIELD_SOURCE: &str = r#"struct Sensor { value: int, trusted: bool }

fn main() -> int {
    let mut sensor = Sensor { value: 1, trusted: 1 < 2 };
    sensor.missing = 2;
    sensor.value
}
"#;

const WRONG_POLICY_ARGUMENT_SOURCE: &str = r#"enum Decision { Normal(int), Alert(int, bool) }

fn classify(total: int, trusted: bool) -> Decision {
    if trusted { return Decision::Alert(total, trusted); }
    Decision::Normal(total)
}

fn main() -> int {
    let decision = classify(1, 2);
    match decision {
        Decision::Normal(value) => value,
        Decision::Alert(value, trusted) => value
    }
}
"#;

const NEGATIVE_INDEX_SOURCE: &str = r#"fn main() -> int {
    let values = [10, 20];
    let zero = 0;
    let index = zero - 1;
    let selected = values[index];
    println!("unreachable negative index: {}", selected);
    0
}
"#;

const UPPER_BOUND_INDEX_SOURCE: &str = r#"fn main() -> int {
    let values = [10, 20];
    let count = 2;
    let index = count;
    let selected = values[index];
    println!("unreachable upper-bound index: {}", selected);
    0
}
"#;

const NEGATIVE_WRITE_INDEX_SOURCE: &str = r#"fn replacement() -> int {
    println!("unreachable assignment rhs");
    41
}

fn main() -> int {
    let mut values = [10, 20];
    let zero = 0;
    let index = zero - 1;
    values[index] = replacement();
    println!("unreachable negative write");
    0
}
"#;

const UPPER_BOUND_WRITE_INDEX_SOURCE: &str = r#"fn replacement() -> int {
    println!("unreachable assignment rhs");
    41
}

fn main() -> int {
    let mut values = [10, 20];
    let count = 2;
    let index = count;
    values[index] = replacement();
    println!("unreachable upper-bound write");
    0
}
"#;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-representative-telemetry-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create representative application workspace");
        Self { root }
    }

    fn write(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create representative fixture directory");
        }
        fs::write(&path, contents).expect("write representative fixture");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-representative-telemetry-"));
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

fn flattened_source() -> String {
    let root_without_modules = MAIN_SOURCE
        .strip_prefix("mod model;\nmod policy;\n\n")
        .expect("representative root keeps the frozen direct-module prefix");
    format!("{MODEL_SOURCE}\n{POLICY_SOURCE}\n{root_without_modules}")
}

fn run_aero(workspace: &TestWorkspace, arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&workspace.root)
        .args(arguments)
        .output()
        .expect("run Aero public CLI")
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn contains_regular_file(path: &Path) -> bool {
    if path.is_file() {
        return true;
    }
    path.is_dir()
        && fs::read_dir(path)
            .expect("read artifact directory")
            .any(|entry| contains_regular_file(&entry.expect("read artifact entry").path()))
}

#[test]
fn representative_scalar_application_is_composed_and_portable() {
    let mut failures = Vec::new();
    let workspace = TestWorkspace::new();
    let root = workspace.write("main.aero", MAIN_SOURCE);
    workspace.write("model.aero", MODEL_SOURCE);
    workspace.write("policy.aero", POLICY_SOURCE);

    let first_llvm = match compile_file(&root, CompilerOptions::default()) {
        Ok(llvm) => Some(llvm),
        Err(error) => {
            failures.push(format!(
                "representative direct-module application did not compile: {error}"
            ));
            None
        }
    };
    let second_llvm = compile_file(&root, CompilerOptions::default());
    if let (Some(first), Ok(second)) = (&first_llvm, second_llvm) {
        if first != &second {
            failures.push("representative file compilation was not deterministic".to_string());
        }
        for anchor in [
            "define i32 @main()",
            "define i32 @add_bias(",
            "define i32 @decision_score(",
            "telemetry score: %g",
            "declare void @llvm.trap()",
            "fcmp oge double",
            "fcmp olt double",
        ] {
            if !first.contains(anchor) {
                failures.push(format!("representative LLVM omitted anchor {anchor:?}"));
            }
        }
    }

    let flattened = flattened_source();
    let ast = try_tokenize_with_locations(&flattened, Some("representative.aero".to_string()))
        .map_err(|error| error.to_string())
        .and_then(|tokens| parse_with_locations(tokens).map_err(|error| error.to_string()));
    match ast {
        Err(error) => failures.push(format!("representative flattened source rejected: {error}")),
        Ok(ast) => {
            if let Err(error) = SemanticAnalyzer::new().analyze(ast.clone()) {
                failures.push(format!(
                    "representative semantic analysis rejected: {error}"
                ));
            }
            match IrGenerator::new().try_generate_ir(ast) {
                Err(error) => failures.push(format!(
                    "representative semantic-independent checked admission rejected: {error}"
                )),
                Ok(checked) => {
                    if let Err(error) = CodeGenerator::new().try_generate_code(checked) {
                        failures.push(format!(
                            "representative independent verified codegen rejected: {error}"
                        ));
                    }
                }
            }
        }
    }

    let check = run_aero(&workspace, &["check", "main.aero"]);
    if !check.status.success() {
        failures.push(format!(
            "representative public check failed: {}",
            output_text(&check)
        ));
    }
    let build = run_aero(
        &workspace,
        &["build", "main.aero", "-o", "representative.ll"],
    );
    if !build.status.success() {
        failures.push(format!(
            "representative public build failed: {}",
            output_text(&build)
        ));
    } else if !workspace.root.join("representative.ll").is_file() {
        failures
            .push("representative public build omitted its requested LLVM artifact".to_string());
    }

    let repository = repository_root();
    for (relative, expected) in [
        ("main.aero", MAIN_SOURCE),
        ("model.aero", MODEL_SOURCE),
        ("policy.aero", POLICY_SOURCE),
        (
            "compile_fail/immutable_projected_update.aero",
            IMMUTABLE_PROJECTED_UPDATE_SOURCE,
        ),
        (
            "compile_fail/unknown_projected_field.aero",
            UNKNOWN_PROJECTED_FIELD_SOURCE,
        ),
        (
            "compile_fail/wrong_policy_argument.aero",
            WRONG_POLICY_ARGUMENT_SOURCE,
        ),
        ("runtime_fail/negative_index.aero", NEGATIVE_INDEX_SOURCE),
        (
            "runtime_fail/upper_bound_index.aero",
            UPPER_BOUND_INDEX_SOURCE,
        ),
        (
            "runtime_fail/negative_write_index.aero",
            NEGATIVE_WRITE_INDEX_SOURCE,
        ),
        (
            "runtime_fail/upper_bound_write_index.aero",
            UPPER_BOUND_WRITE_INDEX_SOURCE,
        ),
    ] {
        let path = repository.join(EXAMPLE_DIRECTORY).join(relative);
        match fs::read_to_string(&path) {
            Ok(actual) if actual == expected => {}
            Ok(_) => failures.push(format!(
                "tracked representative source bytes differ at {}",
                path.display()
            )),
            Err(error) => failures.push(format!(
                "tracked representative source is unavailable at {}: {error}",
                path.display()
            )),
        }
    }

    let workflow_path = repository.join(WORKFLOW);
    match fs::read_to_string(&workflow_path) {
        Ok(workflow) => {
            for anchor in [
                "Test representative telemetry application at O0 and O2",
                "examples/representative_telemetry/main.aero",
                "representative_telemetry.o0",
                "representative_telemetry.o2",
                "telemetry score: 91",
                "representative telemetry test passed with exit code 91",
                "negative_index.aero",
                "upper_bound_index.aero",
                "negative_write_index.aero",
                "upper_bound_write_index.aero",
                "runtime bounds failure corpus passed at O0 and O2",
                "Test representative telemetry application on Windows at O0 and O2",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "native workflow {} omitted anchor {anchor:?}",
                        workflow_path.display()
                    ));
                }
            }
        }
        Err(error) => failures.push(format!(
            "native workflow {} is unavailable: {error}",
            workflow_path.display()
        )),
    }

    assert!(
        failures.is_empty(),
        "M1-001 representative scalar application failures:\n{}",
        failures.join("\n\n")
    );
}

#[test]
fn representative_compile_fail_corpus_rejects_before_artifacts() {
    for (name, source, diagnostic) in [
        (
            "immutable_projected_update",
            IMMUTABLE_PROJECTED_UPDATE_SOURCE,
            "projected assignment root `sensor` must be a mutable local owned binding",
        ),
        (
            "unknown_projected_field",
            UNKNOWN_PROJECTED_FIELD_SOURCE,
            "struct `Sensor` has no field `missing`",
        ),
        (
            "wrong_policy_argument",
            WRONG_POLICY_ARGUMENT_SOURCE,
            "parameter `trusted` type mismatch: expected bool, actual int",
        ),
    ] {
        let workspace = TestWorkspace::new();
        workspace.write("main.aero", source);

        for command in ["check", "build", "run"] {
            let output_name = format!("{name}-{command}.ll");
            let arguments = if command == "build" {
                vec![command, "main.aero", "-o", output_name.as_str()]
            } else {
                vec![command, "main.aero"]
            };
            let output = run_aero(&workspace, &arguments);
            let rendered = output_text(&output);
            assert_eq!(
                output.status.code(),
                Some(1),
                "{name} public {command} must reject with status 1: {rendered}"
            );
            assert!(
                rendered.contains(diagnostic),
                "{name} public {command} omitted deterministic diagnostic {diagnostic:?}: {rendered}"
            );
            assert!(
                !workspace.root.join(&output_name).exists(),
                "{name} public {command} emitted forbidden requested artifact {output_name}"
            );
            assert!(
                !contains_regular_file(&workspace.root.join("target")),
                "{name} public {command} emitted a native artifact after rejection"
            );
        }
    }
}

#[test]
fn admitted_numeric_print_arguments_keep_their_llvm_vararg_type() {
    let workspace = TestWorkspace::new();
    let root = workspace.write("numeric_print_abi.aero", NUMERIC_PRINT_ABI_SOURCE);
    let llvm = compile_file(&root, CompilerOptions::default())
        .expect("admitted numeric print ABI specimen must compile");
    let calls = llvm
        .lines()
        .filter(|line| line.contains("call i32") && line.contains("@printf("))
        .collect::<Vec<_>>();

    assert_eq!(calls.len(), 2, "expected print! and println! call sites");
    assert!(
        calls
            .iter()
            .all(|call| call.contains("call i32 (i8*, ...) @printf(")),
        "printf calls must retain their explicit variadic callee type: {calls:?}"
    );
    assert_eq!(
        calls[0].matches(", double ").count(),
        3,
        "all computed int/float/bool arguments must remain typed LLVM doubles: {}",
        calls[0]
    );
    assert_eq!(
        calls[1].matches(", double ").count(),
        1,
        "the immediate integer argument must remain a typed LLVM double: {}",
        calls[1]
    );
    assert!(
        calls.iter().all(|call| !call.contains(", i64 ")),
        "numeric printf arguments must not be rewritten as raw integer bits: {calls:?}"
    );
    assert!(
        !llvm.contains("bitcast double") || !llvm.contains("to i64"),
        "numeric print lowering must not manufacture i64 varargs from double values"
    );
}
