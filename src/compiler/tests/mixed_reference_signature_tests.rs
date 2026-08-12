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

const EXAMPLE_ROOT: &str = "examples/mixed_reference_signatures/main.aero";
const EXAMPLE_MODULE: &str = "examples/mixed_reference_signatures/references.aero";
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
        "aero-core088-{label}-{}-{nonce}-{serial}",
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
        Err(error) => diagnostics.push(error),
    }
    diagnostics
}

fn complete_mixed_reference_source() -> &'static str {
    r#"
struct Packet { value: int, ready: bool }
struct Row { value: int }
enum State { Idle, Count(int) }

fn mutable_first(target: &mut int, observed: &int) -> int {
    *target = *target + *observed;
    *target
}

fn mutable_middle(left: &int, target: &mut int, right: &int) -> int {
    *target = *target + *left + *right;
    *target
}

fn mutable_last(left: &int, right: &int, target: &mut int, bias: int) -> int {
    *target = *target + *left + *right + bias;
    *target
}

fn aggregate_mix(
    observed: &Packet,
    target: &mut Packet,
    values: &[int; 2],
    flags: (bool, int)
) -> int {
    let packet = *observed;
    let copied = *values;
    *target = Packet {
        value: packet.value + copied[0] + flags.1,
        ready: flags.0
    };
    let result = *target;
    result.value
}

fn enum_mix(observed: &State, target: &mut State) -> int {
    let amount = match *observed {
        State::Idle => 1,
        State::Count(inner) => inner
    };
    *target = State::Count(amount + 1);
    match *target {
        State::Idle => 0,
        State::Count(inner) => inner
    }
}

fn set_from(target: &mut bool, observed: &bool) {
    *target = *observed;
}

fn forwarded(left: &int, target: &mut int, right: &int, bias: int) -> int {
    mutable_last(left, right, target, bias)
}

fn main() -> int {
    let left = 2;
    let right = 3;
    let left_ref = &left;
    let left_again = &left;
    let right_ref = &right;
    let mut total = 1;

    let first = mutable_first(&mut total, left_ref);
    let middle = mutable_middle(left_ref, &mut total, right_ref);
    let last = mutable_last(left_again, right_ref, &mut total, 4);
    let mut forwarded_result = 0;
    {
        let alias = &mut total;
        forwarded_result = forwarded(left_ref, alias, right_ref, 5);
    }

    let observed_packet = Packet { value: 6, ready: 1 < 2 };
    let mut target_packet = Packet { value: 0, ready: 1 > 2 };
    let values = [7, 8];
    let aggregate = aggregate_mix(
        &observed_packet,
        &mut target_packet,
        &values,
        (1 < 2, 9)
    );

    let observed_state = State::Count(10);
    let mut target_state = State::Idle;
    let enum_result = enum_mix(&observed_state, &mut target_state);
    let truth = 2 > 1;
    let mut flag = 1 > 2;
    set_from(&mut flag, &truth);
    let mut projected_target = Row { value: 3 };
    let projected_observed = Row { value: 5 };
    let projected = mutable_first(
        &mut projected_target.value,
        &projected_observed.value
    );

    if flag && first == 3 && middle == 8 && last == 17
        && forwarded_result == 27 && aggregate == 22 && enum_result == 11
        && total == 27 && target_packet.ready && left == 2 && right == 3
        && projected == 8 && projected_target.value == 8 {
        return 88;
    }
    1
}
"#
}

#[test]
fn complete_mixed_exclusive_shared_reference_class_is_checked_and_executable() {
    let source = complete_mixed_reference_source();
    let semantic_ast = analyzed(source).expect("complete mixed-reference source must analyze");
    let checked = IrGenerator::new()
        .try_generate_ir(semantic_ast)
        .expect("semantic-to-checked mixed-reference signatures must be admitted");
    let direct = IrGenerator::new()
        .try_generate_ir(parsed(source).expect("complete mixed-reference source must parse"))
        .expect("semantic-independent admission must use the same topology predicate");
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .expect("verified mixed-reference signatures must lower to LLVM");
    CodeGenerator::new()
        .try_generate_code(direct)
        .expect("directly admitted mixed-reference signatures must independently verify");
    compile_program(source, CompilerOptions::default())
        .expect("complete mixed exclusive/shared-reference class must compile publicly");

    let middle = &checked.metadata().functions["mutable_middle"].signature;
    assert_eq!(
        middle.parameters,
        vec![
            (
                "left".to_string(),
                LogicalType::ImmutableReference {
                    pointee: Box::new(LogicalType::Int),
                },
            ),
            (
                "target".to_string(),
                LogicalType::MutableReference {
                    pointee: Box::new(LogicalType::Int),
                },
            ),
            (
                "right".to_string(),
                LogicalType::ImmutableReference {
                    pointee: Box::new(LogicalType::Int),
                },
            ),
        ]
    );
    let last = &checked.metadata().functions["mutable_last"].signature;
    assert!(matches!(
        &last.parameters[..],
        [
            (_, LogicalType::ImmutableReference { .. }),
            (_, LogicalType::ImmutableReference { .. }),
            (_, LogicalType::MutableReference { .. }),
            (_, LogicalType::Int)
        ]
    ));

    let debug = format!("{checked:#?}");
    let borrows = debug.matches("CheckedMutableBorrow {").count();
    let ends = debug.matches("CheckedMutableBorrowEnd {").count();
    assert_eq!(
        borrows, ends,
        "every exclusive call temporary must end once"
    );
    assert!(
        borrows >= 8,
        "the complete source must exercise every call mode"
    );
    assert!(
        debug.matches("CheckedImmutableReferenceParameter").count() >= 11,
        "checked IR lost immutable-reference binders: {debug}"
    );
    assert!(
        debug.matches("CheckedMutableReferenceParameter").count() >= 7,
        "checked IR lost mutable-reference binders: {debug}"
    );
    assert!(
        debug.matches("CheckedProjectedBorrow").count() >= 2,
        "checked IR lost mixed projected-reference loans: {debug}"
    );
    for anchor in [
        "define i32 @mutable_first(",
        "define i32 @mutable_middle(",
        "define i32 @mutable_last(",
        "define i32 @aggregate_mix(",
        "define i32 @enum_mix(",
        "define void @set_from(",
        "call i32 @mutable_first(",
        "call i32 @mutable_middle(",
        "call i32 @mutable_last(",
    ] {
        assert!(llvm.contains(anchor), "LLVM missing {anchor:?}:\n{llvm}");
    }
    assert!(!llvm.contains("inttoptr") && !llvm.contains("ptrtoint"));
}

#[test]
fn mixed_reference_exclusions_fail_closed_in_every_trust_phase() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "repeated mutable source",
            "fn bad(left: &mut int, observed: &int, right: &mut int) -> int { *left + *observed + *right } fn main() -> int { let mut value = 1; let observed = 2; bad(&mut value, &observed, &mut value) }",
            &["pairwise-distinct source identities"],
        ),
        (
            "direct mutable and immutable overlap",
            "fn bad(target: &mut int, observed: &int) -> int { *target + *observed } fn main() -> int { let mut value = 1; bad(&mut value, &value) }",
            &["arguments must be independent of reference source"],
        ),
        (
            "overlap with mutable source in expression",
            "fn bad(observed: &int, target: &mut int, amount: int) -> int { *target + *observed + amount } fn main() -> int { let mut value = 1; bad(&value, &mut value, value) }",
            &["arguments must be independent of reference source"],
        ),
        (
            "reference result",
            "fn bad(target: &mut int, observed: &int) -> &int { observed } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "entry references",
            "fn main(target: &mut int, observed: &int) -> int { *target + *observed }",
            &["process entry cannot use reference parameters"],
        ),
        (
            "generic references",
            "fn bad<T>(target: &mut int, observed: &int) -> int { *target + *observed } fn main() -> int { 0 }",
            &["generic reference transport functions are not supported"],
        ),
        (
            "unsupported immutable pointee",
            "fn bad(target: &mut int, observed: &String) -> int { *target } fn main() -> int { 0 }",
            &["immutable reference parameter pointee is not admitted Copy-data"],
        ),
        (
            "unsupported mutable pointee",
            "fn bad(target: &mut String, observed: &int) -> int { *observed } fn main() -> int { 0 }",
            &["mutable reference parameter pointee is not admitted Copy-data"],
        ),
        (
            "unsupported side value",
            "fn bad(target: &mut int, observed: &int, text: String) -> int { *target + *observed } fn main() -> int { 0 }",
            &["parameter", "not admitted Copy-data"],
        ),
        (
            "wrong arity",
            "fn bad(target: &mut int, observed: &int) -> int { *target + *observed } fn main() -> int { let mut target = 1; bad(&mut target) }",
            &[
                "requires exactly 2 arguments",
                "mutable references at positions 1",
            ],
        ),
        (
            "wrong mutable position",
            "fn bad(observed: &int, target: &mut int) -> int { *target + *observed } fn main() -> int { let observed = 1; let mut target = 2; bad(&mut target, &observed) }",
            &["mutable-reference identifier or direct"],
        ),
        (
            "wrong immutable type",
            "fn bad(target: &mut int, observed: &bool) -> int { *target } fn main() -> int { let mut target = 1; let observed = 2; bad(&mut target, &observed) }",
            &["parameter", "type mismatch"],
        ),
        (
            "immutable mutable-source owner",
            "fn bad(target: &mut int, observed: &int) -> int { *target + *observed } fn main() -> int { let target = 1; let observed = 2; bad(&mut target, &observed) }",
            &["must be declared mutable"],
        ),
        (
            "uninitialized mutable source",
            "fn bad(target: &mut int, observed: &int) -> int { *target + *observed } fn main() -> int { let mut target: int; let observed = 2; bad(&mut target, &observed) }",
            &["uninitialized variable", "not an initialized local binding"],
        ),
        (
            "uninitialized immutable source",
            "fn bad(target: &mut int, observed: &int) -> int { *target + *observed } fn main() -> int { let mut target = 1; let observed: int; bad(&mut target, &observed) }",
            &[
                "uninitialized variable",
                "no binding for",
                "not an initialized local binding",
            ],
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, expected) in cases {
        let diagnostics = rejection_in_all_trust_phases(source);
        if diagnostics.len() != 3 {
            failures.push(format!(
                "{label}: a trust phase accepted unexpectedly: {diagnostics:?}"
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
    let main_source = fs::read_to_string(&main).expect("read tracked CORE-088 root example");
    let module_source = fs::read_to_string(&module).expect("read tracked CORE-088 module");

    for anchor in [
        "mod references;",
        "mutable_first(&mut total, left_ref)",
        "mutable_middle(left_ref, &mut total, right_ref)",
        "mutable_last(left_again, right_ref, &mut total, 4)",
        "aggregate_mix(",
        "enum_mix(&observed_state, &mut target_state)",
        "set_from(&mut flag, &truth)",
        "return 88;",
    ] {
        assert!(
            main_source.contains(anchor),
            "root example missing {anchor:?}"
        );
    }
    for anchor in [
        "fn mutable_first(target: &mut int, observed: &int)",
        "fn mutable_middle(left: &int, target: &mut int, right: &int)",
        "fn mutable_last(left: &int, right: &int, target: &mut int, bias: int)",
        "fn aggregate_mix(",
        "fn enum_mix(observed: &State, target: &mut State)",
        "fn set_from(target: &mut bool, observed: &bool)",
        "fn forwarded(left: &int, target: &mut int, right: &int, bias: int)",
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
        .expect("run CORE-088 public CLI check");
    assert!(
        check.status.success(),
        "tracked CORE-088 check failed:\n{}",
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
        .expect("run CORE-088 public CLI build");
    let artifact_exists = output.is_file();
    if artifact_exists {
        fs::remove_file(&output).expect("remove exact CORE-088 LLVM artifact");
    }
    assert!(
        build.status.success() && artifact_exists,
        "tracked CORE-088 build failed or omitted LLVM:\n{}",
        command_output_text(&build)
    );

    let invalid_root = unique_temp_path("invalid");
    fs::create_dir_all(&invalid_root).expect("create invalid CORE-088 workspace");
    let invalid_source = invalid_root.join("invalid.aero");
    let invalid_output = invalid_root.join("invalid.ll");
    fs::write(
        &invalid_source,
        "fn bad(left: &mut int, right: &mut int) -> int { *left + *right } fn main() -> int { let mut value = 1; bad(&mut value, &mut value) }",
    )
    .expect("write invalid CORE-088 source");
    let invalid = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(&invalid_source)
        .arg("-o")
        .arg(&invalid_output)
        .current_dir(&root)
        .output()
        .expect("run invalid CORE-088 public CLI build");
    let invalid_diagnostics = command_output_text(&invalid);
    assert!(
        !invalid.status.success()
            && !invalid_output.exists()
            && invalid_diagnostics.contains("pairwise-distinct source identities"),
        "invalid CORE-088 build did not fail closed without an artifact:\n{invalid_diagnostics}"
    );
    fs::remove_dir_all(&invalid_root).expect("remove invalid CORE-088 workspace");

    let workflow =
        fs::read_to_string(root.join(".github/workflows/rust.yml")).expect("read Rust workflow");
    for anchor in [
        "Test mixed exclusive/shared-reference signature integration example",
        "cargo run -- run ../../examples/mixed_reference_signatures/main.aero",
        "mixed exclusive/shared-reference signature example passed with exit code 88",
        "Test Windows mixed exclusive/shared-reference signature system specimen",
        "Windows mixed-reference signature public run passed with exit code 88",
        "Windows mixed-reference signature manual native execution passed with exit code 88",
    ] {
        assert_eq!(
            workflow.matches(anchor).count(),
            1,
            "workflow anchor {anchor:?} must occur exactly once"
        );
    }
}
