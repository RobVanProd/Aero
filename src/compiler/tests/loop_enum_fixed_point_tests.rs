use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_file,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXPECTED_EXIT: i32 = 229;
const EXAMPLE_ROOT: &str = "examples/loop_enum_fixed_point/main.aero";
const EXAMPLE_MODULE: &str = "examples/loop_enum_fixed_point/fixed_points.aero";
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
            "aero-loop-enum-fixed-point-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("create loop fixed-point workspace");
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
            .is_some_and(|name| name.starts_with("aero-loop-enum-fixed-point-"));
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

fn source() -> &'static str {
    r#"
enum Token { A, B }

fn consume(value: Token) -> int {
    match value {
        Token::A => 1,
        Token::B => 2
    }
}

fn keep_going(value: Token, step: int) -> bool {
    let score = consume(value);
    if step < 2 { return score > 0; }
    1 > 2
}

fn while_moved_backedge() -> int {
    let mut owner = Token::A;
    let mut step = 0;
    let mut total = 0;
    while step < 3 {
        owner = Token::B;
        total = total + consume(owner);
        step = step + 1;
    }
    total
}

fn for_moved_backedge() -> int {
    let mut owner = Token::A;
    let mut total = 0;
    for item in [1, 2] {
        owner = Token::B;
        total = total + consume(owner) + item;
    }
    total
}

fn loop_moved_backedge_owned_break() -> int {
    let mut owner = Token::A;
    let mut step = 0;
    let mut total = 0;
    loop {
        owner = Token::B;
        total = total + consume(owner);
        step = step + 1;
        if step < 2 { continue; }
        owner = Token::A;
        break;
    }
    total + consume(owner)
}

fn moved_entry_reinitialized() -> int {
    let mut owner = Token::A;
    let mut total = consume(owner);
    loop {
        owner = Token::B;
        break;
    }
    total = total + consume(owner);
    total
}

fn mixed_backedges() -> int {
    let mut owner = Token::A;
    let mut step = 0;
    let mut total = 0;
    while step < 3 {
        owner = Token::B;
        if step < 1 {
            total = total + consume(owner);
            step = step + 1;
            continue;
        }
        step = step + 1;
    }
    total
}

fn condition_consumption() -> int {
    let mut owner = Token::A;
    let mut step = 0;
    let mut total = 0;
    while keep_going(owner, step) {
        total = total + step + 1;
        owner = Token::B;
        step = step + 1;
    }
    total
}

fn nested_nearest_loop_transfers() -> int {
    let mut outer = Token::A;
    let mut step = 0;
    let mut total = 0;
    while step < 2 {
        outer = Token::B;
        let mut inner = Token::A;
        loop {
            inner = Token::B;
            total = total + consume(inner);
            break;
        }
        total = total + consume(outer);
        step = step + 1;
    }
    total
}

fn main() -> int {
    let total = while_moved_backedge()
        + for_moved_backedge()
        + loop_moved_backedge_owned_break()
        + moved_entry_reinitialized()
        + mixed_backedges()
        + condition_consumption()
        + nested_nearest_loop_transfers();
    if total == 34 { return 229; }
    1
}
"#
}

fn parsed_source(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn parsed() -> Result<Vec<compiler::ast::AstNode>, String> {
    parsed_source(source())
}

#[test]
fn convergent_direct_enum_loop_fixed_point_is_checked_and_executable() {
    let ast = parsed().expect("CORE-079 syntax must remain parsed");

    let analyzed = SemanticAnalyzer::new()
        .analyze(ast.clone())
        .map(|(_, ast)| ast)
        .unwrap_or_else(|error| {
            panic!("CORE-079 semantic fixed point rejected the complete class: {error}")
        });

    IrGenerator::new()
        .try_generate_ir(ast)
        .unwrap_or_else(|error| panic!("CORE-079 independent admission rejected: {error}"));

    let checked = IrGenerator::new()
        .try_generate_ir(analyzed)
        .unwrap_or_else(|error| panic!("CORE-079 checked IR rejected: {error}"));
    let llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .unwrap_or_else(|error| panic!("CORE-079 LLVM lowering rejected: {error}"));

    for marker in [
        "@while_moved_backedge(",
        "@for_moved_backedge(",
        "@loop_moved_backedge_owned_break(",
        "@moved_entry_reinitialized(",
        "@mixed_backedges(",
        "@condition_consumption(",
        "@nested_nearest_loop_transfers(",
        &format!("ret i32 {EXPECTED_EXIT}"),
    ] {
        assert!(
            llvm.contains(marker),
            "CORE-079 LLVM omitted {marker:?}:\n{llvm}"
        );
    }

    let second_ast = SemanticAnalyzer::new()
        .analyze(parsed().expect("second CORE-079 parse"))
        .map(|(_, ast)| ast)
        .expect("second CORE-079 semantic analysis");
    let second_checked = IrGenerator::new()
        .try_generate_ir(second_ast)
        .expect("second CORE-079 checked IR");
    let second_llvm = CodeGenerator::new()
        .try_generate_code(second_checked)
        .expect("second CORE-079 LLVM lowering");
    assert_eq!(llvm, second_llvm, "CORE-079 LLVM must be deterministic");

    let shadowing_source = "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() -> int { let owner = E::A; for owner in [1, 2] { let item = owner; } take(owner) }";
    let shadowing_ast = parsed_source(shadowing_source).expect("for-shadowing source parses");
    SemanticAnalyzer::new()
        .analyze(shadowing_ast.clone())
        .expect("semantic fixed point must preserve the shadowed outer enum owner");
    IrGenerator::new()
        .try_generate_ir(shadowing_ast)
        .expect("checked admission fixed point must restore the shadowed outer enum owner");
}

#[test]
fn unsafe_or_excluded_loop_fixed_points_fail_before_llvm() {
    let cases = [
        (
            "missing repeated-cycle reinitialization",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; let mut step = 0; while step < 2 { let score = take(owner); step = step + 1; } }",
            ["may have been moved", "moved value"],
        ),
        (
            "one path bypasses reinitialization",
            "enum E { A, B } fn take(value: E) -> int { match value { E::A => 1, E::B => 2 } } fn main() { let mut owner = E::A; let mut step = 0; while step < 2 { if step < 1 { owner = E::B; } let score = take(owner); step = step + 1; } }",
            ["may have been moved", "moved value"],
        ),
        (
            "maybe-moved while exit is used",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; let mut step = 0; while step < 1 { owner = E::A; let score = take(owner); step = step + 1; } let invalid = take(owner); }",
            ["may have been moved", "moved value"],
        ),
        (
            "moved loop break exit is used",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; loop { let score = take(owner); break; } let invalid = take(owner); }",
            ["moved value", "Use of moved value"],
        ),
        (
            "same-path double consumption",
            "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; loop { let first = take(owner); let second = take(owner); break; } }",
            ["moved value", "Use of moved value"],
        ),
        (
            "enum aggregate storage remains excluded",
            "enum E { A } fn main() { let mut owner = E::A; loop { let stored = [owner]; break; } }",
            ["not admitted", "array"],
        ),
    ];

    for (label, source, expected) in cases {
        let ast =
            parsed_source(source).unwrap_or_else(|error| panic!("{label} did not parse: {error}"));
        let semantic = SemanticAnalyzer::new().analyze(ast.clone());
        assert!(
            matches!(semantic, Err(ref error) if expected.iter().any(|fragment| error.contains(fragment))),
            "{label} semantic result did not fail closed with {expected:?}: {semantic:?}"
        );

        let admission = IrGenerator::new().try_generate_ir(ast);
        assert!(
            matches!(admission, Err(ref error) if expected.iter().any(|fragment| error.to_string().contains(fragment))),
            "{label} direct checked admission did not fail closed with {expected:?}: {admission:?}"
        );
    }
}

#[test]
fn tracked_loop_fixed_point_example_and_public_gates_are_exact() {
    let root = repository_root();
    let tracked_root = root.join(EXAMPLE_ROOT);
    let tracked_module = root.join(EXAMPLE_MODULE);
    let root_source = fs::read_to_string(&tracked_root).expect("read CORE-079 root example");
    let module_source = fs::read_to_string(&tracked_module).expect("read CORE-079 module example");
    for marker in ["mod fixed_points;", "if total == 34 { return 229; }"] {
        assert!(
            root_source.contains(marker),
            "tracked CORE-079 root omitted {marker:?}:\n{root_source}"
        );
    }
    for marker in [
        "fn while_moved_backedge()",
        "fn for_moved_backedge()",
        "fn loop_moved_backedge_owned_break()",
        "fn moved_entry_reinitialized()",
        "fn mixed_backedges()",
        "fn condition_consumption()",
        "fn nested_nearest_loop_transfers()",
    ] {
        assert!(
            module_source.contains(marker),
            "tracked CORE-079 module omitted {marker:?}:\n{module_source}"
        );
    }

    let llvm = compile_file(&tracked_root, CompilerOptions::default())
        .expect("tracked CORE-079 example must compile");
    for marker in [
        "@while_moved_backedge(",
        "@for_moved_backedge(",
        "@condition_consumption(",
        "@nested_nearest_loop_transfers(",
        "ret i32 229",
    ] {
        assert!(
            llvm.contains(marker),
            "tracked CORE-079 LLVM omitted {marker:?}:\n{llvm}"
        );
    }
    assert_eq!(
        llvm,
        compile_file(&tracked_root, CompilerOptions::default())
            .expect("second tracked CORE-079 compilation"),
        "tracked CORE-079 file compilation must be deterministic"
    );

    let workspace = TestWorkspace::new("tracked-example");
    let output = workspace.path("loop_enum_fixed_point.ll");
    let check = run_cli(&workspace, &[Path::new("check"), &tracked_root]);
    assert!(
        check.status.success(),
        "tracked CORE-079 CLI check failed: {}",
        output_text(&check)
    );
    let build = run_cli(
        &workspace,
        &[Path::new("build"), &tracked_root, Path::new("-o"), &output],
    );
    assert!(
        build.status.success() && output.is_file(),
        "tracked CORE-079 CLI build failed or omitted artifact: {}",
        output_text(&build)
    );

    let invalid_workspace = TestWorkspace::new("invalid-artifact-hygiene");
    let invalid = invalid_workspace.path("invalid.aero");
    let invalid_output = invalid_workspace.path("invalid.ll");
    fs::write(
        &invalid,
        "enum E { A } fn take(value: E) -> int { match value { E::A => 1 } } fn main() { let mut owner = E::A; let mut step = 0; while step < 2 { let score = take(owner); step = step + 1; } }",
    )
    .expect("write invalid CORE-079 source");
    let invalid_check = run_cli(&invalid_workspace, &[Path::new("check"), &invalid]);
    assert!(
        !invalid_check.status.success()
            && output_text(&invalid_check).contains("may have been moved"),
        "invalid CORE-079 CLI check did not fail closed: {}",
        output_text(&invalid_check)
    );
    let invalid_build = run_cli(
        &invalid_workspace,
        &[
            Path::new("build"),
            &invalid,
            Path::new("-o"),
            &invalid_output,
        ],
    );
    assert!(
        !invalid_build.status.success() && !invalid_output.exists(),
        "invalid CORE-079 CLI build succeeded or left an artifact: {}",
        output_text(&invalid_build)
    );

    let workflow = fs::read_to_string(root.join(WORKFLOW)).expect("read Rust workflow");
    for anchor in [
        "Test convergent loop enum fixed-point integration example",
        "cargo run -- check ../../examples/loop_enum_fixed_point/main.aero",
        "cargo run -- run ../../examples/loop_enum_fixed_point/main.aero",
        "opt-22 -passes=verify -disable-output ../../loop_enum_fixed_point.ll",
        "llc-22 -verify-machineinstrs ../../loop_enum_fixed_point.ll",
        "clang-22 -no-pie ../../loop_enum_fixed_point.o -o ../../loop_enum_fixed_point",
        "convergent loop enum fixed-point example passed with exit code 229",
        "Test Windows loop enum fixed-point system specimen",
        "Windows loop fixed-point public run passed with exit code 229",
        "Windows loop fixed-point manual native execution passed with exit code 229",
    ] {
        assert_eq!(
            workflow.matches(anchor).count(),
            1,
            "public workflow must contain exactly one CORE-079 anchor {anchor:?}"
        );
    }
    for preserved in [
        "loop-local enum ownership example passed with exit code 149",
        "unified CopyData Match result example passed with exit code 223",
        "balanced loop enum ownership example passed with exit code 227",
        "Windows public run passed with exit code 227",
        "Windows manual native execution passed with exit code 227",
    ] {
        assert_eq!(
            workflow.matches(preserved).count(),
            1,
            "CORE-079 must preserve exactly one prior system anchor {preserved:?}"
        );
    }
}
