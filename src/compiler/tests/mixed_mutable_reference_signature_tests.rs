use compiler::ast::AstNode;
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, LogicalType, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const EXAMPLE_ROOT: &str = "examples/mixed_mutable_reference_signatures/main.aero";
const EXAMPLE_MODULE: &str = "examples/mixed_mutable_reference_signatures/mixers.aero";
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("compiler crate must be nested below repository root")
        .to_path_buf()
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

fn complete_mixed_signature_source() -> &'static str {
    r#"
struct Packet { value: int, ready: bool }
enum State { Idle, Count(int) }

fn reference_first(value: &mut int, amount: int) -> int {
    *value = *value + amount;
    *value
}

fn reference_middle(prefix: int, value: &mut int, suffix: int) -> int {
    *value = *value + prefix + suffix;
    *value
}

fn reference_last(prefix: int, suffix: int, value: &mut int) -> int {
    *value = *value + prefix + suffix;
    *value
}

fn recursive_copydata_sides(
    value: &mut int,
    packet: Packet,
    values: [int; 2],
    flags: (bool, int)
) -> int {
    *value = *value + packet.value + values[0] + flags.1;
    *value
}

fn replace_packet(prefix: int, value: &mut Packet, suffix: int) -> int {
    *value = Packet { value: prefix + suffix, ready: prefix < suffix };
    let observed = *value;
    observed.value
}

fn replace_and_read_state(prefix: int, value: &mut State, suffix: int) -> int {
    *value = State::Count(prefix + suffix);
    match *value {
        State::Idle => 0,
        State::Count(inner) => inner
    }
}

fn set_flag(value: &mut bool, amount: int) {
    *value = amount > 0;
}

fn forwarded(value: &mut int, amount: int) -> int {
    reference_middle(1, value, amount)
}

fn main() -> int {
    let mut number = 1;
    let first = reference_first(&mut number, 2);
    let middle = reference_middle(3, &mut number, 4);
    let last = reference_last(5, 6, &mut number);

    let packet = Packet { value: 7, ready: 1 < 2 };
    let values = [8, 9];
    let aggregate = recursive_copydata_sides(&mut number, packet, values, (1 < 2, 10));

    let mut alias_result = 0;
    {
        let alias = &mut number;
        alias_result = forwarded(alias, 11);
    }

    let mut changed = Packet { value: 0, ready: 1 > 2 };
    let packet_result = replace_packet(12, &mut changed, 13);
    let mut state = State::Idle;
    let state_result = replace_and_read_state(14, &mut state, 15);
    let mut flag = 1 > 2;
    set_flag(&mut flag, 1);

    if flag && changed.ready && first == 3 && middle == 10 && last == 21
        && aggregate == 46 && alias_result == 58 && packet_result == 25
        && state_result == 29 && number == 58 {
        return 87;
    }
    1
}
"#
}

#[test]
fn complete_mixed_mutable_reference_signature_class_is_executable() {
    let source = complete_mixed_signature_source();
    let analyzed = analyzed(source).expect("complete mixed signature source must analyze");
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed)
        .expect("semantic-to-checked mixed signatures must be admitted");
    let direct = IrGenerator::new()
        .try_generate_ir(parsed(source).expect("complete mixed signature source must parse"))
        .expect("semantic-independent checked admission must use the same contract");
    let llvm = CodeGenerator::new()
        .try_generate_code(checked.clone())
        .expect("verified mixed signatures must lower to LLVM");
    CodeGenerator::new()
        .try_generate_code(direct)
        .expect("directly admitted mixed signatures must independently verify and lower");
    compile_program(source, CompilerOptions::default())
        .expect("complete mixed mutable-reference signature class must compile publicly");

    let middle = &checked.metadata().functions["reference_middle"].signature;
    assert_eq!(
        middle.parameters,
        vec![
            ("prefix".to_string(), LogicalType::Int),
            (
                "value".to_string(),
                LogicalType::MutableReference {
                    pointee: Box::new(LogicalType::Int),
                },
            ),
            ("suffix".to_string(), LogicalType::Int),
        ]
    );
    let last = &checked.metadata().functions["reference_last"].signature;
    assert!(matches!(
        &last.parameters[..],
        [(_, LogicalType::Int), (_, LogicalType::Int), (_, LogicalType::MutableReference { pointee })]
            if pointee.as_ref() == &LogicalType::Int
    ));

    let debug = format!("{checked:#?}");
    let borrows = debug.matches("CheckedMutableBorrow {").count();
    let ends = debug.matches("CheckedMutableBorrowEnd {").count();
    assert_eq!(
        borrows, ends,
        "every mutable call temporary must end exactly once"
    );
    assert!(
        borrows >= 9,
        "the complete source must exercise every direct/reborrow call"
    );
    for identity in [
        "CheckedMutableReferenceParameter",
        "CheckedMutableDereferenceAssignment",
        "CheckedMutableEnumMatchRead",
    ] {
        assert!(
            debug.contains(identity),
            "checked IR missing {identity}:\n{debug}"
        );
    }
    for anchor in [
        "define i32 @reference_first(double*",
        "define i32 @reference_middle(i32",
        "double* %aero.arg.value",
        "define i32 @reference_last(i32",
        "call i32 @reference_middle(i32",
        "call i32 @reference_last(i32",
        "define void @set_flag(i1*",
    ] {
        assert!(llvm.contains(anchor), "LLVM missing {anchor:?}:\n{llvm}");
    }
    assert!(!llvm.contains("inttoptr") && !llvm.contains("ptrtoint"));
}

#[test]
fn mixed_signature_exclusions_fail_closed_in_every_trust_phase() {
    let cases: &[(&str, &str, &[&str])] = &[
        (
            "repeated source across two mutable references",
            "fn bad(left: &mut int, amount: int, right: &mut int) -> int { *left + amount + *right } fn main() -> int { let mut value = 1; bad(&mut value, 2, &mut value) }",
            &["pairwise-distinct source identities"],
        ),
        (
            "mutable source reused by immutable reference argument",
            "fn bad(value: &mut int, other: &int, second: &mut int) -> int { *value + *other + *second } fn main() -> int { let mut value = 1; let mut second = 2; bad(&mut value, &value, &mut second) }",
            &["non-mutable arguments must be independent of reference source `value`"],
        ),
        (
            "reference result",
            "fn bad(value: &mut int, amount: int) -> &mut int { value } fn main() -> int { 0 }",
            &["reference results require lifetime semantics"],
        ),
        (
            "entry reference parameter",
            "fn main(value: &mut int, amount: int) -> int { *value + amount }",
            &["process entry cannot use reference parameters"],
        ),
        (
            "generic reference function",
            "fn bad<T>(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { 0 }",
            &["generic reference transport functions are not supported"],
        ),
        (
            "String side parameter",
            "fn bad(value: &mut int, text: String) -> int { *value } fn main() -> int { 0 }",
            &["parameter `text` is not admitted Copy-data"],
        ),
        (
            "String result",
            "fn bad(value: &mut int, amount: int) -> String { \"no\" } fn main() -> int { 0 }",
            &["return type is not admitted Copy-data or Void"],
        ),
        (
            "unsupported mutable pointee",
            "fn bad(value: &mut String, amount: int) -> int { amount } fn main() -> int { 0 }",
            &["mutable reference parameter pointee is not admitted Copy-data"],
        ),
        (
            "wrong arity",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; add(&mut value) }",
            &[
                "requires exactly 2 arguments",
                "mutable references at positions 1",
            ],
        ),
        (
            "wrong reference position",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; add(2, &mut value) }",
            &["mutable-reference identifier or direct `&mut`"],
        ),
        (
            "immutable reference argument",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; add(&value, 2) }",
            &["mutable-reference identifier or direct `&mut`"],
        ),
        (
            "immutable owner",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let value = 1; add(&mut value, 2) }",
            &["must be declared mutable"],
        ),
        (
            "uninitialized reference owner",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value: int; add(&mut value, 2) }",
            &[
                "uninitialized variable `value`",
                "source `value` is not an initialized local binding",
            ],
        ),
        (
            "uninitialized CopyData side argument",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; let amount: int; add(&mut value, amount) }",
            &[
                "uninitialized variable `amount`",
                "checked IR has no binding for `amount`",
            ],
        ),
        (
            "wrong CopyData side type",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; add(&mut value, 1 < 2) }",
            &["expected int, actual bool"],
        ),
        (
            "direct owner reused as side argument",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; add(&mut value, value) }",
            &["non-mutable arguments must be independent of reference source `value`"],
        ),
        (
            "mutable alias reused as side argument",
            "fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut value = 1; let alias = &mut value; add(alias, *alias) }",
            &["non-mutable arguments must be independent of reference source `alias`"],
        ),
        (
            "projected mutable source",
            "struct Row { value: int } fn add(value: &mut int, amount: int) -> int { *value + amount } fn main() -> int { let mut row = Row { value: 1 }; add(&mut row.value, 2) }",
            &["requires an identifier place"],
        ),
    ];

    let mut failures = Vec::new();
    for (label, source, expected) in cases {
        let diagnostics = rejection_in_all_trust_phases(source);
        if diagnostics.len() != 3 {
            failures.push(format!(
                "{label}: one or more trust phases accepted unexpectedly: {diagnostics:?}"
            ));
            continue;
        }
        for diagnostic in &diagnostics {
            if diagnostic.contains("accepted the source") {
                failures.push(format!(
                    "{label}: a trust phase accepted unexpectedly: {diagnostics:?}"
                ));
                continue;
            }
            if !expected
                .iter()
                .any(|expected| diagnostic.contains(expected))
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
    let main_source = fs::read_to_string(&main).expect("read tracked CORE-087 root example");
    let module_source = fs::read_to_string(&module).expect("read tracked CORE-087 module");
    for anchor in [
        "mod mixers;",
        "adjust_first(&mut number, 2)",
        "adjust_middle(3, &mut number, 4)",
        "adjust_last(5, 6, &mut number)",
        "adjust_aggregate(&mut number, packet, values, (1 < 2, 10))",
        "return 87;",
    ] {
        assert!(
            main_source.contains(anchor),
            "root example missing {anchor:?}"
        );
    }
    for anchor in [
        "fn adjust_first(value: &mut int, amount: int)",
        "fn adjust_middle(prefix: int, value: &mut int, suffix: int)",
        "fn adjust_last(prefix: int, suffix: int, value: &mut int)",
        "fn adjust_aggregate(",
        "fn replace_and_observe(prefix: int, value: &mut State, suffix: int)",
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
        .expect("run CORE-087 public CLI check");
    assert!(
        check.status.success(),
        "tracked CORE-087 check failed:\n{}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    let serial = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let output = std::env::temp_dir().join(format!(
        "aero-core087-{}-{nonce}-{serial}.ll",
        std::process::id()
    ));
    let build = Command::new(env!("CARGO_BIN_EXE_aero"))
        .arg("build")
        .arg(&main)
        .arg("-o")
        .arg(&output)
        .current_dir(&root)
        .output()
        .expect("run CORE-087 public CLI build");
    let artifact_exists = output.is_file();
    if artifact_exists {
        fs::remove_file(&output).expect("remove exact CORE-087 temporary LLVM artifact");
    }
    assert!(
        build.status.success() && artifact_exists,
        "tracked CORE-087 build failed or omitted LLVM:\n{}{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let workflow =
        fs::read_to_string(root.join(".github/workflows/rust.yml")).expect("read Rust workflow");
    for anchor in [
        "Test mixed mutable-reference and CopyData signature integration example",
        "cargo run -- run ../../examples/mixed_mutable_reference_signatures/main.aero",
        "mixed mutable-reference signature example passed with exit code 87",
        "Test Windows mixed mutable-reference and CopyData signature system specimen",
        "Windows mixed mutable-reference signature public run passed with exit code 87",
        "Windows mixed mutable-reference signature manual native execution passed with exit code 87",
    ] {
        assert_eq!(
            workflow.matches(anchor).count(),
            1,
            "workflow anchor {anchor:?} must occur exactly once"
        );
    }
}
