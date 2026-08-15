use compiler::{
    CheckedIr, CompilerOptions, IrGenerator, SemanticAnalyzer, check_program, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const SHALLOW_LOGICAL_SOURCE: &str = r#"
fn first() -> bool {
    return 1 < 2;
}

fn second() -> bool {
    return 2 < 3;
}

fn third() -> bool {
    return 4 < 3;
}

fn fourth() -> bool {
    return 5 < 6;
}

fn main() -> int {
    if first() && second() || third() && fourth() {
        return 92;
    }
    return 1;
}
"#;

const CHECKED_IR_SNAPSHOT_SOURCE: &str = r#"
fn main() -> int {
    let value: bool = (1 < 2) && (2 < 3) || (4 < 3) && (5 < 6);
    if value {
        return 92;
    }
    return 1;
}
"#;

const LEFT_FAILURE_SOURCE: &str = r#"
fn main() -> int {
    if (2147483648 == 0) && (1 / 0 == 0) {
        return 92;
    }
    return 1;
}
"#;

const RIGHT_FAILURE_SOURCE: &str = r#"
fn main() -> int {
    if (1 / 0 == 0) && (2147483648 == 0) {
        return 92;
    }
    return 1;
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
            .expect("system clock must follow the Unix epoch")
            .as_nanos();
        let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-core092-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create CORE-092 workspace");
        Self { root }
    }

    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.root.join(name);
        fs::write(&path, source).expect("write CORE-092 source");
        path
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-core092-"));
        if self.root.starts_with(std::env::temp_dir()) && expected {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below the repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn checked_ir(source: &str) -> CheckedIr {
    let tokens = try_tokenize_with_locations(source, None).expect("logical source must lex");
    let ast = parse_with_locations(tokens).expect("logical source must parse");
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed) = analyzer
        .analyze(ast)
        .expect("logical source must pass semantic analysis");
    let mut generator = IrGenerator::new();
    generator
        .try_generate_ir(analyzed)
        .expect("logical source must pass checked IR")
}

fn md5_hex(bytes: &[u8]) -> String {
    format!("{:x}", md5::compute(bytes))
}

fn deep_logical_source(terms: usize) -> String {
    assert!(terms >= 2);
    let condition = (0..terms)
        .map(|value| format!("{value} < {}", value + 1))
        .collect::<Vec<_>>()
        .join(" && ");
    format!(
        "fn main() -> int {{\n    let contract = {condition};\n    if contract {{\n        return 92;\n    }}\n    return 1;\n}}\n"
    )
}

fn run_aero(workspace: &TestWorkspace, arguments: &[&Path]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command
        .current_dir(&workspace.root)
        .stdin(Stdio::null())
        .arg(arguments[0])
        .arg(arguments[1]);
    command.output().expect("launch isolated Aero child")
}

fn visible_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn accepted_shallow_logical_behavior_and_failure_order_are_frozen() {
    let checked = checked_ir(SHALLOW_LOGICAL_SOURCE);
    let checked_metadata = format!("{:?}", checked.metadata());
    let checked_snapshot = checked_ir(CHECKED_IR_SNAPSHOT_SOURCE);
    let checked_snapshot_debug = format!("{checked_snapshot:?}");
    let llvm = compile_program(SHALLOW_LOGICAL_SOURCE, CompilerOptions::default())
        .expect("shallow logical source must compile");

    assert_eq!(
        md5_hex(checked_metadata.as_bytes()),
        "ed58fb917b03347410c9d1c7ac7f90d6",
        "accepted checked metadata changed before the stack-safety correction"
    );
    assert_eq!(
        md5_hex(checked_snapshot_debug.as_bytes()),
        "6644809fc3fce047c713ed814416c0ff",
        "accepted logical instruction/register sequence changed"
    );
    assert_eq!(
        checked_ir(SHALLOW_LOGICAL_SOURCE),
        checked,
        "shallow checked IR is not deterministic"
    );
    assert_eq!(
        checked_ir(CHECKED_IR_SNAPSHOT_SOURCE),
        checked_snapshot,
        "single-function checked IR snapshot is not deterministic"
    );
    assert_eq!(
        md5_hex(llvm.as_bytes()),
        "4d6ef00c66c5c5fa8f855abace1758fd",
        "accepted LLVM changed before the stack-safety correction"
    );
    assert_eq!(
        compile_program(SHALLOW_LOGICAL_SOURCE, CompilerOptions::default())
            .expect("repeated shallow logical compilation must succeed"),
        llvm,
        "shallow logical compilation is not deterministic"
    );

    assert_eq!(
        check_program(LEFT_FAILURE_SOURCE, CompilerOptions::default())
            .expect_err("left invalid logical leaf must fail checked admission"),
        "IR Generation Error: integer literal is outside the admitted i32 range"
    );
    assert_eq!(
        check_program(RIGHT_FAILURE_SOURCE, CompilerOptions::default())
            .expect_err("right invalid logical leaf must fail checked admission"),
        "IR Generation Error: constant integer division by zero"
    );
}

#[test]
fn deep_left_associated_logical_source_checks_and_runs_without_stack_overflow() {
    let workspace = TestWorkspace::new("deep-child");
    let source = workspace.write("main.aero", &deep_logical_source(24));
    let check = run_aero(&workspace, &[Path::new("check"), &source]);
    assert!(
        check.status.success(),
        "deep logical check did not complete normally (status={:?}):\n{}",
        check.status.code(),
        visible_output(&check)
    );

    let run = run_aero(&workspace, &[Path::new("run"), &source]);
    assert_eq!(
        run.status.code(),
        Some(92),
        "deep logical native run drifted or overflowed:\n{}",
        visible_output(&run)
    );
}

#[test]
fn logical_validation_and_lowering_have_explicit_worklist_authorities() {
    let generator = read(&repository_root().join("src/compiler/src/ir_generator.rs"));
    let production = generator
        .split("\n#[cfg(test)]")
        .next()
        .expect("IR generator must have a production prefix");

    for anchor in [
        "fn validate_logical_expression_iterative",
        "enum LogicalLoweringTask",
        "fn generate_logical_ir_iterative",
    ] {
        assert!(
            production.contains(anchor),
            "CORE-092 intentional structural red: `{anchor}` is absent"
        );
    }

    let validation_function = production
        .split("\n    fn validate_expression(")
        .nth(1)
        .and_then(|tail| tail.split("\n    fn static_string_value(").next())
        .expect("active expression validator must remain isolated");
    let validation_arm = validation_function
        .split("Expression::Logical { left, right, .. } =>")
        .nth(1)
        .and_then(|tail| tail.split("Expression::Unary").next())
        .expect("active logical validation arm must remain isolated");
    assert!(
        validation_arm.contains("Self::validate_logical_expression_iterative("),
        "active logical validation does not delegate to its worklist"
    );
    assert!(
        !validation_arm.contains("Self::validate_expression("),
        "active logical validation still directly recurses into a child"
    );

    let lowering = production
        .split("fn generate_logical_ir(")
        .nth(1)
        .and_then(|tail| tail.split("\n    fn generate_unary_ir(").next())
        .expect("active logical lowering helper must remain isolated");
    assert!(
        lowering.contains("self.generate_logical_ir_iterative("),
        "active logical lowering does not delegate to its worklist"
    );
    assert!(
        !lowering.contains("self.generate_expression_ir(left")
            && !lowering.contains("self.generate_expression_ir(right"),
        "active logical lowering still directly recurses into a child"
    );
}
