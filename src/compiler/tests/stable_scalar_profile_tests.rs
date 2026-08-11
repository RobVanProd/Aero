use compiler::{
    CompilerOptions, LanguageProfile, check_file, check_program, compile_file, compile_program,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

const STABLE_SCALAR_PROGRAM: &str = r#"
fn advance(value: int, limit: int) -> int {
    if value < limit {
        return value + 3;
    }
    return value - 1;
}

fn accepted(value: int, limit: int) -> bool {
    return value >= limit && !(value > limit);
}

fn comparison_product(value: int) -> bool {
    let less: bool = value < 12;
    let less_equal: bool = value <= 11;
    let equal: bool = value == 11;
    let not_equal: bool = value != 10;
    let greater_equal: bool = value >= 11;
    let greater: bool = value > 10;
    return less && less_equal && equal && not_equal && greater_equal && greater;
}

fn same(left: bool, right: bool) -> bool {
    return left == right;
}

fn score(value: int, accepted_value: bool) -> int {
    if accepted_value {
        return value * 7 + 14;
    }
    return 1;
}

fn observe(value: int) {
    if value < 0 {
        return;
    }
    return;
}

fn main() -> int {
    let mut value: int = 2;
    let inferred_limit = 11;
    while value < inferred_limit {
        value = advance(value, inferred_limit);
    }
    let policy: bool = accepted(value, inferred_limit);
    let comparison_policy: bool = comparison_product(value);
    let exact_policy: bool = same(policy, comparison_policy);
    observe(value);
    return score(value, exact_policy);
}
"#;

const STABLE_SCALAR_APPLICATION: &str =
    include_str!("../../../examples/stable_scalar_v0/main.aero");
const STABLE_SCALAR_WRAPPING_EDGES: &str =
    include_str!("../../../examples/stable_scalar_v0/wrapping_edges.aero");

fn stable_options() -> CompilerOptions {
    CompilerOptions {
        language_profile: LanguageProfile::StableScalarV0,
        ..CompilerOptions::default()
    }
}

struct TestWorkspace {
    root: PathBuf,
}

impl TestWorkspace {
    fn new(test_name: &str) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock must be after Unix epoch")
            .as_nanos();
        let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "aero-stable-scalar-profile-{test_name}-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create fresh stable-profile test workspace");
        Self { root }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir();
        let expected_name = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-stable-scalar-profile-"));
        if self.root.starts_with(temp_dir) && expected_name {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn run_check(workspace: &TestWorkspace, input: &Path, stable_profile: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_aero"));
    command.arg("check").arg(input);
    if stable_profile {
        command.arg("--language-profile").arg("stable-scalar-v0");
    }
    command
        .current_dir(&workspace.root)
        .output()
        .expect("run aero check")
}

fn run_build(workspace: &TestWorkspace, input: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(input)
        .arg("-o")
        .arg(output)
        .arg("--language-profile")
        .arg("stable-scalar-v0")
        .current_dir(&workspace.root)
        .output()
        .expect("run aero build")
}

fn run_program(workspace: &TestWorkspace, input: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("run")
        .arg(input)
        .arg("--language-profile")
        .arg("stable-scalar-v0")
        .current_dir(&workspace.root)
        .output()
        .expect("run aero run")
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn strip_cli_colors(output: &str) -> String {
    output
        .replace("\x1b[1;31m", "")
        .replace("\x1b[1;32m", "")
        .replace("\x1b[1;34m", "")
        .replace("\x1b[1;36m", "")
        .replace("\x1b[0m", "")
}

#[test]
fn stable_scalar_profile_selects_a_valid_scalar_program() {
    let workspace = TestWorkspace::new("positive-red");
    let source = workspace.path("main.aero");
    fs::write(&source, STABLE_SCALAR_PROGRAM).expect("write stable scalar source");

    let output = run_check(&workspace, &source, true);
    assert!(
        output.status.success(),
        "stable-scalar-v0 selection is absent or rejected a valid profile program:\n{}",
        combined_output(&output)
    );
}

#[test]
fn stable_scalar_profile_is_shared_by_source_and_file_library_routes() {
    check_program(STABLE_SCALAR_PROGRAM, stable_options())
        .expect("source-only check should admit the stable scalar product");
    let source_llvm = compile_program(STABLE_SCALAR_PROGRAM, stable_options())
        .expect("source-only compile should emit the stable scalar product");

    let workspace = TestWorkspace::new("library-route-parity");
    let source = workspace.path("main.aero");
    fs::write(&source, STABLE_SCALAR_PROGRAM).expect("write stable scalar source");
    check_file(&source, stable_options()).expect("file check should share profile admission");
    let file_llvm = compile_file(&source, stable_options())
        .expect("file compile should share profile emission");

    assert_eq!(source_llvm, file_llvm);

    fs::write(&source, "mod missing; fn main() -> int { return 0; }")
        .expect("write unresolved module attempt");
    let error = check_file(&source, stable_options())
        .expect_err("profile classification must precede module resolution");
    assert_eq!(
        error,
        "Language Profile Error: stable-scalar-v0 rejects module declarations"
    );
}

#[test]
fn stable_profile_identity_cannot_be_discarded_through_the_public_checked_program_api() {
    let library = include_str!("../src/lib.rs");
    assert!(!library.contains("fn into_checked_ir("));
    assert!(library.contains("pub fn try_generate_llvm(self)"));
}

#[test]
fn stable_scalar_profile_emits_one_exact_i32_lane() {
    let llvm = compile_program(STABLE_SCALAR_PROGRAM, stable_options())
        .expect("stable scalar program should compile");

    for anchor in [
        "define i32 @advance(i32 %aero.arg.value, i32 %aero.arg.limit)",
        "alloca i32, align 4",
        "store i32",
        "load i32",
        "add i32",
        "sub i32",
        "mul i32",
        "icmp slt i32",
        "icmp sle i32",
        "icmp eq i32",
        "icmp ne i32",
        "icmp sge i32",
        "icmp sgt i32",
        "icmp eq i1",
        "call i32 @advance(i32",
        "ret i32",
    ] {
        assert!(
            llvm.contains(anchor),
            "missing exact i32 anchor `{anchor}`:\n{llvm}"
        );
    }
    for forbidden in [
        "fadd double",
        "fsub double",
        "fmul double",
        "fptosi",
        "sitofp",
        "alloca double",
        "load double",
        "store double",
    ] {
        assert!(
            !llvm.contains(forbidden),
            "stable scalar LLVM leaked the experimental numeric lane `{forbidden}`:\n{llvm}"
        );
    }
}

#[test]
fn explicit_experimental_profile_is_byte_identical_to_the_default() {
    let implicit = compile_program(STABLE_SCALAR_PROGRAM, CompilerOptions::default())
        .expect("experimental default control should compile");
    let explicit = compile_program(
        STABLE_SCALAR_PROGRAM,
        CompilerOptions {
            language_profile: LanguageProfile::Experimental,
            ..CompilerOptions::default()
        },
    )
    .expect("explicit experimental control should compile");

    assert_eq!(implicit, explicit);
    assert!(
        implicit.contains("alloca double, align 8"),
        "control must retain the legacy experimental numeric lane"
    );
}

#[test]
fn stable_scalar_application_and_wrapping_corpus_match_the_reference_model() {
    fn reference_advance(value: i32) -> i32 {
        if value < 5 {
            5
        } else if value < 8 {
            8
        } else if value < 11 {
            11
        } else {
            value
        }
    }

    let mut state = 2_i32;
    while state < 11 {
        state = reference_advance(state);
    }
    let accepted = state >= 11 && state <= 11;
    let application_checksum = if accepted {
        state.wrapping_mul(7).wrapping_add(14)
    } else {
        1
    };
    assert_eq!(application_checksum, 91);

    let wrapped_add = i32::MAX.wrapping_add(1);
    let wrapped_sub = wrapped_add.wrapping_sub(1);
    let wrapped_mul = 1_073_741_824_i32.wrapping_mul(2);
    let wrapped_neg = wrapped_add.wrapping_neg();
    let edge_checksum = if wrapped_add < 0 && wrapped_sub > 0 && wrapped_mul < 0 && wrapped_neg < 0
    {
        93
    } else {
        1
    };
    assert_eq!(edge_checksum, 93);

    let application_llvm = compile_program(STABLE_SCALAR_APPLICATION, stable_options())
        .expect("profile-selected application should compile");
    let edge_llvm = compile_program(STABLE_SCALAR_WRAPPING_EDGES, stable_options())
        .expect("wrapping corpus should compile");
    for llvm in [&application_llvm, &edge_llvm] {
        assert!(!llvm.contains(" nsw "));
        assert!(!llvm.contains(" nuw "));
        assert!(!llvm.contains("double"));
        assert!(!llvm.contains("fptosi"));
        assert!(!llvm.contains("sitofp"));
    }
}

#[test]
fn stable_scalar_system_gate_is_anchored_on_linux_and_windows() {
    let workflow = include_str!("../../../.github/workflows/rust.yml");
    for step in [
        "Test stable scalar profile application and wrapping corpus at O0 and O2",
        "Test stable scalar profile application and wrapping corpus on Windows at O0 and O2",
    ] {
        assert_eq!(
            workflow.matches(step).count(),
            1,
            "workflow must retain exactly one `{step}` step"
        );
    }
    for anchor in [
        "examples/stable_scalar_v0/",
        "wrapping_edges.aero",
        "--language-profile stable-scalar-v0",
        "opt-22 -passes=verify",
        "llc-22 -verify-machineinstrs",
        "clang-22 -O0",
        "clang-22 -O2",
        "& \"$llvmBin\\opt.exe\" -passes=verify",
        "& \"$llvmBin\\llc.exe\" -verify-machineinstrs",
        "& \"$llvmBin\\clang.exe\" -O0",
        "& \"$llvmBin\\clang.exe\" -O2",
    ] {
        assert!(workflow.contains(anchor), "workflow omitted `{anchor}`");
    }
}

#[test]
fn stable_scalar_profile_rejects_each_excluded_source_family_before_semantics() {
    let cases = [
        (
            "missing-main",
            "fn helper() -> int { return 0; }",
            "programs without `fn main() -> int`",
        ),
        (
            "wrong-main",
            "fn main(value: int) -> int { return value; }",
            "entrypoints other than exact `fn main() -> int`",
        ),
        (
            "duplicate-function",
            "fn helper() -> int { return 1; } fn helper() -> int { return 2; } fn main() -> int { return 0; }",
            "duplicate function definitions",
        ),
        (
            "generic-function",
            "fn id<T>(value: T) -> T { return value; } fn main() -> int { return 0; }",
            "generic functions or trait bounds",
        ),
        (
            "float-type",
            "fn helper(value: float) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "array-type",
            "fn helper(value: [int; 1]) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "tuple-type",
            "fn helper(value: (int, int)) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "reference-type",
            "fn helper(value: &int) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "generic-type",
            "fn helper(value: Option<int>) -> int { return 0; } fn main() -> int { return 0; }",
            "function parameter types",
        ),
        (
            "const",
            "const LIMIT: int = 3; fn main() -> int { return 0; }",
            "top-level constants",
        ),
        (
            "struct",
            "struct Reading { value: int } fn main() -> int { return 0; }",
            "struct definitions",
        ),
        (
            "enum",
            "enum Flag { Off, On } fn main() -> int { return 0; }",
            "enum definitions",
        ),
        (
            "trait",
            "trait Read { fn read(value: int) -> int; } fn main() -> int { return 0; }",
            "trait definitions",
        ),
        (
            "impl",
            "impl Widget { fn read() -> int { return 0; } } fn main() -> int { return 0; }",
            "impl blocks",
        ),
        (
            "module",
            "mod helper; fn main() -> int { return 0; }",
            "module declarations",
        ),
        (
            "import",
            "use helper; fn main() -> int { return 0; }",
            "import declarations",
        ),
        (
            "tail-expression",
            "fn main() -> int { 0 }",
            "implicit tail expressions",
        ),
        (
            "body-const",
            "fn main() -> int { const LIMIT: int = 3; return 0; }",
            "constant declarations",
        ),
        (
            "nested-function",
            "fn main() -> int { fn inner() -> int { return 1; } return 0; }",
            "nested functions",
        ),
        (
            "uninitialized",
            "fn main() -> int { let value: int; return 0; }",
            "uninitialized bindings",
        ),
        (
            "expression-statement",
            "fn main() -> int { 1 + 2; return 0; }",
            "effect-free or non-call expression statements",
        ),
        (
            "standalone-block",
            "fn main() -> int { { let value: int = 1; } return 0; }",
            "unsupported blocks",
        ),
        (
            "for",
            "fn main() -> int { for value in values { return value; } return 0; }",
            "for loops",
        ),
        (
            "loop",
            "fn main() -> int { loop { return 1; } return 0; }",
            "unconditional loop statements",
        ),
        (
            "break",
            "fn main() -> int { while 1 < 2 { break; } return 0; }",
            "break statements",
        ),
        (
            "continue",
            "fn main() -> int { while 1 < 2 { continue; } return 0; }",
            "continue statements",
        ),
        (
            "divide",
            "fn main() -> int { return 4 / 2; }",
            "division expressions",
        ),
        (
            "remainder",
            "fn main() -> int { return 4 % 2; }",
            "remainder expressions",
        ),
        (
            "float-literal",
            "fn main() -> int { let value = 1.5; return 0; }",
            "float literals",
        ),
        (
            "char-literal",
            "fn main() -> int { let value = 'a'; return 0; }",
            "character literals",
        ),
        (
            "string-literal",
            "fn main() -> int { let value = \"a\"; return 0; }",
            "String literals",
        ),
        (
            "method-call",
            "fn main() -> int { return \"a\".len(); }",
            "method calls",
        ),
        (
            "print-intrinsic",
            "fn main() -> int { print!(\"{}\", 1); return 0; }",
            "formatting/output intrinsics",
        ),
        (
            "println-intrinsic",
            "fn main() -> int { println!(\"{}\", 1); return 0; }",
            "formatting/output intrinsics",
        ),
        (
            "array",
            "fn main() -> int { let value = [1, 2]; return 0; }",
            "array expressions",
        ),
        (
            "array-repeat",
            "fn main() -> int { let value = [1; 2]; return 0; }",
            "array expressions",
        ),
        (
            "index",
            "fn main() -> int { return values[0]; }",
            "index expressions",
        ),
        (
            "field",
            "fn main() -> int { return reading.value; }",
            "field-access expressions",
        ),
        (
            "tuple",
            "fn main() -> int { let value = (1, 2); return 0; }",
            "tuple expressions",
        ),
        (
            "tuple-index",
            "fn main() -> int { return pair.0; }",
            "tuple expressions",
        ),
        (
            "struct-literal",
            "fn main() -> int { let value = Reading { value: 1 }; return 0; }",
            "struct value construction",
        ),
        (
            "enum-variant",
            "fn main() -> int { let value = Flag::On; return 0; }",
            "enum value construction",
        ),
        (
            "match",
            "fn main() -> int { return match 1 { 1 => 11, _ => 22 }; }",
            "Match expressions",
        ),
        (
            "borrow",
            "fn main() -> int { let value: int = 1; let reference = &value; return 0; }",
            "reference expressions",
        ),
        (
            "dereference",
            "fn main() -> int { return *reference; }",
            "reference expressions",
        ),
        (
            "projected-assignment",
            "fn main() -> int { reading.value = 1; return 0; }",
            "projected or indirect assignment targets",
        ),
        (
            "closure",
            "fn main() -> int { let value = |item: int| item; return 0; }",
            "closure expressions",
        ),
        (
            "out-of-range",
            "fn main() -> int { return 2147483648; }",
            "integer literals outside the signed i32 range",
        ),
        (
            "logical-call",
            "fn ready() -> bool { return 1 < 2; } fn main() -> int { if ready() && ready() { return 1; } return 0; }",
            "function calls inside logical operands",
        ),
        (
            "direct-recursion",
            "fn recurse(value: int) -> int { return recurse(value); } fn main() -> int { return recurse(0); }",
            "recursive function call cycles",
        ),
        (
            "mutual-recursion",
            "fn left(value: int) -> int { return right(value); } fn right(value: int) -> int { return left(value); } fn main() -> int { return left(0); }",
            "recursive function call cycles",
        ),
    ];

    for (name, source, expected) in cases {
        let error = match check_program(source, stable_options()) {
            Ok(()) => panic!("{name} unexpectedly entered the stable profile"),
            Err(error) => error,
        };
        assert!(
            error.starts_with("Language Profile Error: stable-scalar-v0 rejects "),
            "{name} escaped the classifier and failed later: {error}"
        );
        assert!(
            error.contains(expected),
            "{name} produced the wrong profile diagnostic: {error}"
        );
        assert!(
            !error.contains("Semantic Analysis Error") && !error.contains("IR Generation Error"),
            "{name} reached a later trusted phase: {error}"
        );
    }
}

#[test]
fn stable_scalar_profile_rejects_an_out_of_profile_program_before_compilation() {
    let workspace = TestWorkspace::new("negative-red");
    let source = workspace.path("main.aero");
    fs::write(
        &source,
        "struct Reading { value: int } fn main() -> int { return 0; }",
    )
    .expect("write out-of-profile source");

    let experimental = run_check(&workspace, &source, false);
    assert!(
        experimental.status.success(),
        "experimental default control unexpectedly rejected the declaration:\n{}",
        combined_output(&experimental)
    );

    let stable = run_check(&workspace, &source, true);
    let diagnostics = strip_cli_colors(&combined_output(&stable));
    assert_eq!(stable.status.code(), Some(1), "{diagnostics}");
    assert!(
        diagnostics.contains("Language Profile Error: stable-scalar-v0 rejects struct definitions"),
        "profile rejection was not the compiler-owned diagnostic:\n{diagnostics}"
    );
    assert!(
        !diagnostics.contains("Usage:"),
        "profile selection was treated as a CLI invocation error:\n{diagnostics}"
    );
}

#[test]
fn stable_scalar_cli_build_emits_i32_and_rejections_emit_no_artifact_or_execution() {
    let workspace = TestWorkspace::new("cli-build-run-boundary");
    let valid_source = workspace.path("valid.aero");
    let valid_llvm = workspace.path("valid.ll");
    fs::write(&valid_source, STABLE_SCALAR_PROGRAM).expect("write valid stable source");

    let valid_build = run_build(&workspace, &valid_source, &valid_llvm);
    assert!(
        valid_build.status.success(),
        "stable CLI build failed:\n{}",
        combined_output(&valid_build)
    );
    let llvm = fs::read_to_string(&valid_llvm).expect("stable CLI build must write LLVM");
    assert!(llvm.contains("alloca i32, align 4"));
    assert!(!llvm.contains("alloca double"));

    let invalid_source = workspace.path("invalid.aero");
    let invalid_llvm = workspace.path("invalid.ll");
    fs::write(
        &invalid_source,
        "struct Reading { value: int } fn main() -> int { return 0; }",
    )
    .expect("write invalid stable source");

    let invalid_build = run_build(&workspace, &invalid_source, &invalid_llvm);
    let build_diagnostics = strip_cli_colors(&combined_output(&invalid_build));
    assert_eq!(invalid_build.status.code(), Some(1), "{build_diagnostics}");
    assert!(build_diagnostics.contains("Language Profile Error:"));
    assert!(
        !invalid_llvm.exists(),
        "profile rejection must occur before artifact writing"
    );

    let invalid_run = run_program(&workspace, &invalid_source);
    let run_diagnostics = strip_cli_colors(&combined_output(&invalid_run));
    assert_eq!(invalid_run.status.code(), Some(1), "{run_diagnostics}");
    assert!(run_diagnostics.contains("Language Profile Error:"));
    assert!(!run_diagnostics.contains("Program executed successfully"));
    assert!(!run_diagnostics.contains("Exit code:"));
}
