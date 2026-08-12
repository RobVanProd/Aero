use compiler::ast::AstNode;
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, LogicalType, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/multiple_mutable_reference_signatures/main.aero";
const EXAMPLE_MODULE: &str = "examples/multiple_mutable_reference_signatures/references.aero";
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
}

fn unique_temp_path(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "aero-core089-{label}-{}-{nonce}-{serial}",
        std::process::id()
    ))
}

fn command_output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parsed(source: &str) -> Result<Vec<AstNode>, String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    parse_with_locations(tokens).map_err(|error| error.to_string())
}

fn analyzed(source: &str) -> Result<Vec<AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source)?)
        .map(|(_, ast)| ast)
}

fn rejection_in_all_trust_phases(source: &str) -> Vec<String> {
    let mut diagnostics = Vec::new();
    match analyzed(source) {
        Ok(_) => diagnostics.push("semantic analysis accepted the source".to_string()),
        Err(error) => diagnostics.push(error),
    }
    match parsed(source).and_then(|ast| {
        IrGenerator::new()
            .try_generate_ir(ast)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }) {
        Ok(()) => diagnostics.push("direct checked admission accepted the source".to_string()),
        Err(error) => diagnostics.push(error),
    }
    match compile_program(source, CompilerOptions::default()) {
        Ok(_) => diagnostics.push("public compilation accepted the source".to_string()),
        Err(error) => diagnostics.push(error.to_string()),
    }
    diagnostics
}

fn multiple_exclusive_reference_source() -> &'static str {
    r#"
struct Row { value: int }
enum State { Idle, Count(int) }

fn update_pair(left: &mut int, right: &mut int) -> int {
    *left = *left + 1;
    *right = *right + 2;
    *left + *right
}

fn update_three(
    first: &mut int,
    observed: &int,
    bias: int,
    second: &mut int,
    third: &mut int
) -> int {
    *first = *first + *observed;
    *second = *second + bias;
    *third = *third + *first + *second;
    *first + *second + *third
}

fn forward_pair(left: &mut int, right: &mut int) -> int {
    update_pair(left, right)
}

fn replace_rows(
    left: &mut Row,
    observed: &Row,
    bias: (int, bool),
    right: &mut Row
) -> int {
    let source = *observed;
    *left = Row { value: source.value + bias.0 };
    let left_copy = *left;
    *right = Row { value: left_copy.value + 1 };
    let right_copy = *right;
    left_copy.value + right_copy.value
}

fn replace_states(left: &mut State, observed: &State, right: &mut State) -> int {
    let amount = match *observed {
        State::Idle => 1,
        State::Count(inner) => inner
    };
    *left = State::Count(amount + 1);
    *right = State::Count(amount + 2);
    let left_score = match *left {
        State::Idle => 0,
        State::Count(inner) => inner
    };
    let right_score = match *right {
        State::Idle => 0,
        State::Count(inner) => inner
    };
    left_score + right_score
}

fn set_flags(left: &mut bool, observed: &bool, right: &mut bool) {
    *left = *observed;
    *right = *observed;
}

fn main() -> int {
    let mut left = 1;
    let mut right = 2;
    let pair = update_pair(&mut left, &mut right);
    let observed = 3;
    let observed_ref = &observed;
    let mut third = 4;
    let triple = update_three(
        &mut left,
        observed_ref,
        5,
        &mut right,
        &mut third
    );
    let mut forwarded = 0;
    {
        let left_alias = &mut left;
        let right_alias = &mut right;
        forwarded = forward_pair(left_alias, right_alias);
    }
    let mut mixed_left = 1;
    let mut mixed_right = 2;
    let mut mixed_third = 3;
    let mut mixed = 0;
    {
        let mixed_alias = &mut mixed_left;
        mixed = update_three(
            mixed_alias,
            observed_ref,
            1,
            &mut mixed_right,
            &mut mixed_third
        );
    }
    let observed_row = Row { value: 7 };
    let mut left_row = Row { value: 0 };
    let mut right_row = Row { value: 0 };
    let rows = replace_rows(
        &mut left_row,
        &observed_row,
        (2, 1 < 2),
        &mut right_row
    );
    let observed_state = State::Count(4);
    let mut left_state = State::Idle;
    let mut right_state = State::Idle;
    let states = replace_states(
        &mut left_state,
        &observed_state,
        &mut right_state
    );
    let truth = 2 > 1;
    let mut left_flag = 1 > 2;
    let mut right_flag = 1 > 2;
    set_flags(&mut left_flag, &truth, &mut right_flag);
    let mut projected_left = Row { value: 10 };
    let mut projected_right = Row { value: 20 };
    let projected = update_pair(
        &mut projected_left.value,
        &mut projected_right.value
    );
    if pair == 6 && triple == 32 && forwarded == 17
        && left == 6 && right == 11 && third == 18 && mixed == 17
        && rows == 19 && states == 11 && left_flag && right_flag
        && projected == 33 && projected_left.value == 11
        && projected_right.value == 22 {
        return 89;
    }
    1
}
"#
}

#[test]
fn complete_multiple_exclusive_reference_class_is_checked_and_executable() {
    let source = multiple_exclusive_reference_source();
    let ast = parsed(source).expect("multiple-exclusive source must parse before admission");
    let semantic_ast = SemanticAnalyzer::new()
        .analyze(ast.clone())
        .expect("complete multiple-exclusive class must analyze")
        .1;
    let checked = IrGenerator::new()
        .try_generate_ir(semantic_ast)
        .expect("semantic-to-checked multiple-exclusive class must be admitted");
    let direct = IrGenerator::new()
        .try_generate_ir(ast)
        .expect("semantic-independent admission must use the shared call contract");
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .expect("verified multiple-exclusive class must lower to LLVM");
    CodeGenerator::new()
        .try_generate_code(direct)
        .expect("direct multiple-exclusive checked IR must independently verify");
    compile_program(source, CompilerOptions::default())
        .expect("complete multiple-exclusive class must compile publicly");

    let signature = &checked.metadata().functions["update_three"].signature;
    assert!(matches!(
        &signature.parameters[..],
        [
            (_, LogicalType::MutableReference { .. }),
            (_, LogicalType::ImmutableReference { .. }),
            (_, LogicalType::Int),
            (_, LogicalType::MutableReference { .. }),
            (_, LogicalType::MutableReference { .. })
        ]
    ));
    let debug = format!("{checked:#?}");
    let borrows = debug.matches("CheckedMutableBorrow {").count();
    let ends = debug.matches("CheckedMutableBorrowEnd {").count();
    assert_eq!(borrows, ends, "every mutable call temporary must end once");
    assert!(
        borrows >= 18,
        "the source-mode product was not exercised: {debug}"
    );
    assert!(
        debug.matches("CheckedMutableReferenceParameter").count() >= 13,
        "multiple mutable binders were lost: {debug}"
    );
    assert!(
        debug.matches("CheckedProjectedBorrow").count() >= 2,
        "multiple projected mutable loans were lost: {debug}"
    );
    for anchor in [
        "define i32 @update_pair(",
        "define i32 @update_three(",
        "define i32 @replace_rows(",
        "define i32 @replace_states(",
        "define void @set_flags(",
        "call i32 @update_pair(",
        "call i32 @update_three(",
    ] {
        assert!(llvm.contains(anchor), "LLVM missing {anchor:?}:\n{llvm}");
    }
    assert!(!llvm.contains("inttoptr") && !llvm.contains("ptrtoint"));
}

#[test]
fn multiple_exclusive_reference_exclusions_fail_closed_in_every_trust_phase() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "repeated direct owner",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut value = 1; bad(&mut value, &mut value) }",
            &["pairwise-distinct source identities"],
        ),
        (
            "repeated mutable alias",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut value = 1; let alias = &mut value; bad(alias, alias) }",
            &["pairwise-distinct source identities"],
        ),
        (
            "mutable and immutable overlap",
            "fn bad(left: &mut int, observed: &int, right: &mut int) -> int { *left + *observed + *right } fn main() -> int { let mut left = 1; let mut right = 2; bad(&mut left, &left, &mut right) }",
            &["non-mutable arguments must be independent of reference source `left`"],
        ),
        (
            "mutable and CopyData overlap",
            "fn bad(left: &mut int, amount: int, right: &mut int) -> int { *left + amount + *right } fn main() -> int { let mut left = 1; let mut right = 2; bad(&mut left, left + 1, &mut right) }",
            &["non-mutable arguments must be independent of reference source `left`"],
        ),
        (
            "wrong arity",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut left = 1; bad(&mut left) }",
            &["requires exactly 2 arguments", "positions 1, 2"],
        ),
        (
            "wrong mutable position and pointee",
            "fn bad(left: &mut int, right: &mut bool) -> int { *left } fn main() -> int { let mut left = 1; let mut right = 1 < 2; bad(&mut right, &mut left) }",
            &["pointee mismatch", "expected int, actual bool"],
        ),
        (
            "immutable reference in mutable position",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let left = 1; let mut right = 2; bad(&left, &mut right) }",
            &["position 1 requires a mutable-reference identifier or direct `&mut`"],
        ),
        (
            "immutable owner",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let left = 1; let mut right = 2; bad(&mut left, &mut right) }",
            &["must be declared mutable"],
        ),
        (
            "uninitialized owner",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut left: int; let mut right = 2; bad(&mut left, &mut right) }",
            &["uninitialized variable", "not an initialized local binding"],
        ),
        (
            "explicit dereference reborrow",
            "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut left = 1; let mut right = 2; let alias = &mut left; bad(&mut *alias, &mut right) }",
            &["requires an identifier place"],
        ),
        (
            "reference result",
            "fn bad(left: &mut int, right: &mut int) -> &mut int { left } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "entry references",
            "fn main(left: &mut int, right: &mut int) -> int { *left + *right }",
            &["process entry cannot use reference parameters"],
        ),
        (
            "generic references",
            "fn bad<T>(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { 0 }",
            &["generic reference transport functions are not supported"],
        ),
        (
            "unsupported mutable pointee",
            "fn bad(left: &mut String, right: &mut int) -> int { *right } fn main() -> int { 0 }",
            &["mutable reference parameter pointee is not admitted Copy-data"],
        ),
        (
            "unsupported side value",
            "fn bad(left: &mut int, text: String, right: &mut int) -> int { *left + *right } fn main() -> int { 0 }",
            &["parameter `text` is not admitted Copy-data"],
        ),
        (
            "unsupported result",
            "fn bad(left: &mut int, right: &mut int) -> String { \"no\" } fn main() -> int { 0 }",
            &["return type is not admitted Copy-data or Void"],
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, expected) in cases {
        let diagnostics = rejection_in_all_trust_phases(source);
        if diagnostics.len() != 3 {
            failures.push(format!(
                "{label}: incomplete trust results: {diagnostics:?}"
            ));
            continue;
        }
        for diagnostic in &diagnostics {
            if diagnostic.contains("accepted the source")
                || !expected
                    .iter()
                    .any(|fragment| diagnostic.contains(fragment))
            {
                failures.push(format!(
                    "{label}: diagnostic {diagnostic:?} contained none of {expected:?}"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n---\n"));
}

#[test]
fn tracked_direct_module_and_public_system_gate_are_anchored() {
    let root = repository_root();
    let main = root.join(EXAMPLE_ROOT);
    let module = root.join(EXAMPLE_MODULE);
    let main_source = fs::read_to_string(&main).expect("read tracked CORE-089 root example");
    let module_source = fs::read_to_string(&module).expect("read tracked CORE-089 module");

    for anchor in [
        "mod references;",
        "update_pair(&mut left, &mut right, observed_ref, 4)",
        "let triple = update_three(",
        "forward_pair(left_alias, right_alias)",
        "set_flags(&mut first_flag, &truth, &mut second_flag)",
        "return 89;",
    ] {
        assert!(
            main_source.contains(anchor),
            "root example missing {anchor:?}"
        );
    }
    for anchor in [
        "fn update_pair(",
        "left: &mut int,",
        "right: &mut int,",
        "fn update_three(",
        "third: &mut int,",
        "fn forward_pair(left: &mut int, right: &mut int)",
        "fn set_flags(first: &mut bool, observed: &bool, second: &mut bool)",
    ] {
        assert!(
            module_source.contains(anchor),
            "module example missing {anchor:?}"
        );
    }

    let check = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("check")
        .arg(&main)
        .current_dir(&root)
        .output()
        .expect("run CORE-089 public CLI check");
    assert!(
        check.status.success(),
        "tracked CORE-089 check failed:\n{}",
        command_output_text(&check)
    );

    let output = unique_temp_path("valid").with_extension("ll");
    let build = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(&main)
        .arg("-o")
        .arg(&output)
        .current_dir(&root)
        .output()
        .expect("run CORE-089 public CLI build");
    let artifact_exists = output.is_file();
    if artifact_exists {
        fs::remove_file(&output).expect("remove exact CORE-089 LLVM artifact");
    }
    assert!(
        build.status.success() && artifact_exists,
        "tracked CORE-089 build failed or omitted LLVM:\n{}",
        command_output_text(&build)
    );

    let invalid_root = unique_temp_path("invalid");
    fs::create_dir_all(&invalid_root).expect("create invalid CORE-089 workspace");
    let invalid_source = invalid_root.join("invalid.aero");
    let invalid_output = invalid_root.join("invalid.ll");
    fs::write(
        &invalid_source,
        "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut value = 1; bad(&mut value, &mut value) }",
    )
    .expect("write invalid CORE-089 source");
    let invalid = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(&invalid_source)
        .arg("-o")
        .arg(&invalid_output)
        .current_dir(&root)
        .output()
        .expect("run invalid CORE-089 public CLI build");
    let invalid_diagnostics = command_output_text(&invalid);
    assert!(
        !invalid.status.success()
            && !invalid_output.exists()
            && invalid_diagnostics.contains("pairwise-distinct source identities"),
        "invalid CORE-089 build did not fail closed without an artifact:\n{invalid_diagnostics}"
    );
    fs::remove_dir_all(&invalid_root).expect("remove invalid CORE-089 workspace");

    let workflow =
        fs::read_to_string(root.join(".github/workflows/rust.yml")).expect("read Rust workflow");
    for anchor in [
        "Test multiple exclusive-reference signature integration example",
        "cargo run -- run ../../examples/multiple_mutable_reference_signatures/main.aero",
        "multiple exclusive-reference signature example passed with exit code 89",
        "Test Windows multiple exclusive-reference signature system specimen",
        "Windows multiple-exclusive signature public run passed with exit code 89",
        "Windows multiple-exclusive signature manual native execution passed with exit code 89",
    ] {
        assert_eq!(
            workflow.matches(anchor).count(),
            1,
            "workflow anchor {anchor:?} must occur exactly once"
        );
    }
}
