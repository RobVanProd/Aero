use compiler::ast::{AstNode, Expression, Statement};
use compiler::errors::CompilerError;
use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, LogicalType, SemanticAnalyzer, Token,
    compile_file, compile_program, parse_with_locations, tokenize_with_locations,
    try_tokenize_with_locations,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
            "aero-char-copydata-{label}-{}-{nonce}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create character test workspace");
        Self { root }
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let expected = self
            .root
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("aero-char-copydata-"));
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

const RED_SOURCE: &str = r#"
fn choose(value: char, expected: char) -> int {
    if value == expected {
        return 197;
    } else {
        return 1;
    }
}

fn main() -> int {
    let ascii: char = 'A';
    let escaped: char = '\x41';
    let cjk = '界';
    let non_bmp = '😊';
    if ascii != escaped {
        return 2;
    } else if cjk == '界' {
        return choose(non_bmp, '😊');
    } else {
        return 3;
    }
}
"#;

const COPYDATA_SOURCE: &str = r#"
struct Glyph { value: char, history: [char; 2] }
struct Envelope { glyph: Glyph, pair: (char, Glyph), empty: [char; 0] }

enum Signal {
    Idle,
    One(char),
    Many(char, Glyph)
}

fn identity(value: char) -> char { value }
fn make(value: char) -> Glyph { Glyph { value: value, history: [value, '\x41'] } }
fn replace(value: &mut char) -> char {
    *value = '😊';
    *value
}
fn read(value: &char) -> char { *value }
fn char_score(value: char, expected: char, score: int) -> int {
    if value == expected { return score; }
    0
}
fn score(value: Signal) -> int {
    match value {
        Signal::Idle => 1,
        Signal::One(character) => char_score(character, '界', 2),
        Signal::Many(character, glyph) =>
            char_score(character, '😊', char_score(glyph.history[1], 'A', 4))
    }
}
fn extract(value: Signal) -> char {
    match value {
        Signal::Idle => '\0',
        Signal::One(character) => character,
        Signal::Many(character, glyph) => glyph.value
    }
}

fn main() -> int {
    let inferred = '界';
    let annotated: char = identity(inferred);
    let glyph = make(annotated);
    let envelope = Envelope {
        glyph: glyph,
        pair: ('😊', glyph),
        empty: []
    };
    let copied = envelope;
    let repeated = [copied; 2];
    let mut current: char = 'A';
    let before = read(&current);
    { let alias = &mut current; *alias = '界'; }
    let changed = replace(&mut current);
    let mut loop_value = 'A';
    let mut index = 0;
    while index < 1 {
        loop_value = '界';
        index = index + 1;
    }
    for item in ['😊'] {
        if item != '😊' { return 10; }
    }
    if before == 'A'
        && current == '😊'
        && changed == '😊'
        && loop_value == '界'
        && repeated[1].pair.0 == '😊'
        && repeated[0].pair.1.value == '界'
        && repeated[0].empty.len() == 0
        && score(Signal::Idle) == 1
        && score(Signal::One('界')) == 2
        && score(Signal::Many('😊', glyph)) == 4
        && extract(Signal::Idle) == '\0'
        && extract(Signal::One('界')) == '界'
        && extract(Signal::Many('😊', glyph)) == '界' {
        return 197;
    }
    1
}
"#;

#[test]
fn core_072_unicode_char_values_compile_through_the_public_pipeline() {
    let llvm = compile_program(RED_SOURCE, CompilerOptions::default())
        .unwrap_or_else(|error| panic!("CORE-072 character program must compile: {error}"));

    assert!(llvm.contains("define i32 @main()"));
    assert!(llvm.contains("ret i32 197"));
}

#[test]
fn core_072_char_is_recursive_copydata_across_the_admitted_transport_topology() {
    let llvm = compile_program(COPYDATA_SOURCE, CompilerOptions::default())
        .unwrap_or_else(|error| panic!("CORE-072 CopyData program must compile: {error}"));

    for fragment in [
        "%aero.struct.Glyph = type { i32, [2 x i32] }",
        "define i32 @identity(i32 %aero.arg.value)",
        "define i32 @replace(i32* %aero.arg.value)",
        "[0 x i32]",
        "icmp eq i32",
        "icmp ne i32",
    ] {
        assert!(
            llvm.contains(fragment),
            "LLVM missing {fragment:?}:\n{llvm}"
        );
    }
    assert!(
        !llvm.contains("sitofp i32 %aero.arg.value"),
        "character parameters crossed into the numeric double lane:\n{llvm}"
    );
}

#[test]
fn core_072_char_can_be_the_exact_result_of_identifier_bound_enum_match() {
    let source = r#"
enum Signal { Idle, One(char) }

fn extract(value: Signal) -> char {
    match value {
        Signal::Idle => '\0',
        Signal::One(character) => character
    }
}

fn main() -> int {
    if extract(Signal::One('界')) == '界' && extract(Signal::Idle) == '\0' {
        return 197;
    }
    1
}
"#;

    let llvm = compile_program(source, CompilerOptions::default())
        .unwrap_or_else(|error| panic!("character Match result must compile: {error}"));
    assert!(llvm.contains("define i32 @extract"), "{llvm}");
    assert!(llvm.contains("icmp eq i32"), "{llvm}");
}

#[test]
fn core_072_every_escape_executes_as_its_exact_character_value() {
    let source = r#"
fn main() -> int {
    let values = ['\n', '\r', '\t', '\\', '\'', '\"', '\0', '\x41'];
    if values[0] == '\n'
        && values[1] == '\r'
        && values[2] == '\t'
        && values[3] == '\\'
        && values[4] == '\''
        && values[5] == '\"'
        && values[6] == '\0'
        && values[7] == 'A' {
        return 197;
    }
    1
}
"#;
    let llvm = compile_program(source, CompilerOptions::default())
        .unwrap_or_else(|error| panic!("escaped character program must compile: {error}"));
    for code_point in [0, 9, 10, 13, 34, 39, 65, 92] {
        assert!(
            llvm.contains(&format!("i32 {code_point}")),
            "escaped scalar U+{code_point:04X} missing from LLVM:\n{llvm}"
        );
    }
}

#[test]
fn core_072_lexer_and_parser_retain_exact_unicode_scalar_identity_and_location() {
    let valid = [
        ("'A'", 'A'),
        ("'界'", '界'),
        ("'😊'", '😊'),
        ("'\\n'", '\n'),
        ("'\\r'", '\r'),
        ("'\\t'", '\t'),
        ("'\\\\'", '\\'),
        ("'\\\''", '\''),
        ("'\\\"'", '"'),
        ("'\\0'", '\0'),
        ("'\\x41'", 'A'),
        ("'\\xFF'", 'ÿ'),
    ];

    for (source, expected) in valid {
        let tokens = try_tokenize_with_locations(source, Some("chars.aero".to_string()))
            .unwrap_or_else(|error| panic!("valid {source:?} did not lex: {error}"));
        assert_eq!(
            tokens[0].token,
            Token::CharacterLiteral(expected),
            "{source:?}"
        );
        assert_eq!(tokens[0].location.line, 1, "{source:?}");
        assert_eq!(tokens[0].location.column, 1, "{source:?}");
        assert_eq!(tokens[0].location.filename.as_deref(), Some("chars.aero"));
    }

    let tokens = try_tokenize_with_locations("let value = '😊';", None).expect("lex parser probe");
    let ast = parse_with_locations(tokens).expect("parse character literal");
    let Some(AstNode::Statement(Statement::Function { .. })) = ast.first() else {
        // The source is intentionally a top-level binding, so inspect the exact node below.
        let Some(AstNode::Statement(Statement::Let {
            value: Some(Expression::CharacterLiteral(character)),
            ..
        })) = ast.first()
        else {
            panic!("parser lost character AST identity: {ast:#?}");
        };
        assert_eq!(*character, '😊');
        return;
    };
    unreachable!("parser probe unexpectedly became a function")
}

#[test]
fn core_072_invalid_character_literals_are_one_fail_closed_diagnostic_class() {
    for source in [
        "''", "'ab'", "'A", "'\\q'", "'\\x'", "'\\x4'", "'\\xGG'", "'\\x414'", "'\\x41z'", "'\n'",
        "'\r'",
    ] {
        match try_tokenize_with_locations(source, Some("bad-char.aero".to_string())) {
            Err(CompilerError::InvalidCharacterLiteral { location }) => {
                assert_eq!(location.line, 1, "{source:?}");
                assert_eq!(location.column, 1, "{source:?}");
                assert_eq!(location.filename.as_deref(), Some("bad-char.aero"));
            }
            other => panic!("{source:?} escaped exact character diagnostic: {other:?}"),
        }
    }

    let recovered = tokenize_with_locations("'ab'; 7", None);
    assert!(
        recovered
            .iter()
            .all(|token| !matches!(token.token, Token::CharacterLiteral(_))),
        "recovery fabricated a character: {recovered:?}"
    );
    assert!(
        recovered
            .iter()
            .any(|token| token.token == Token::IntegerLiteral(7)),
        "recovery failed to resynchronize: {recovered:?}"
    );
}

fn expect_rejection(label: &str, source: &str, expected: &str) {
    match compile_program(source, CompilerOptions::default()) {
        Ok(llvm) => panic!("{label}: unsupported character source compiled:\n{llvm}"),
        Err(error) => assert!(
            error.contains(expected),
            "{label}: expected {expected:?}, got {error:?}"
        ),
    }
}

#[test]
fn core_072_excluded_operations_and_type_substitutions_fail_before_ir() {
    for (label, source, expected) in [
        (
            "arithmetic",
            "fn main() -> int { let value = 'A' + 'B'; 0 }",
            "Type mismatch in arithmetic operation",
        ),
        (
            "ordering",
            "fn main() -> int { if 'A' < 'B' { return 1; } 0 }",
            "character comparisons require",
        ),
        (
            "logical",
            "fn main() -> int { if 'A' && 'B' { return 1; } 0 }",
            "must be boolean",
        ),
        (
            "logical not",
            "fn main() -> int { let value = !'A'; 0 }",
            "requires boolean operand",
        ),
        (
            "negation",
            "fn main() -> int { let value = -'A'; 0 }",
            "requires numeric operand",
        ),
        (
            "integer annotation substitution",
            "fn main() -> int { let value: int = 'A'; 0 }",
            "expected int, actual char",
        ),
        (
            "character annotation substitution",
            "fn main() -> int { let value: char = 65; 0 }",
            "expected char, actual int",
        ),
        (
            "function argument substitution",
            "fn take(value: char) -> int { 0 } fn main() -> int { take(65) }",
            "expected char, actual int",
        ),
        (
            "function return substitution",
            "fn bad() -> char { 65 } fn main() -> int { 0 }",
            "return type mismatch: expected char, actual int",
        ),
        (
            "array element substitution",
            "fn main() -> int { let values = ['A', 65]; 0 }",
            "array element type mismatch: expected char, actual int",
        ),
        (
            "struct field substitution",
            "struct Glyph { value: char } fn main() -> int { let value = Glyph { value: 65 }; 0 }",
            "field `value` type mismatch: expected char, actual int",
        ),
        (
            "formatting remains excluded",
            "fn main() -> int { println!(\"{}\", 'A'); 0 }",
            "type `char` is not printable",
        ),
        (
            "method dispatch remains excluded",
            "fn main() -> int { return 'A'.len(); }",
            "receiver type has no executable intrinsic method contract",
        ),
        (
            "literal pattern execution remains excluded",
            "fn main() -> int { let value = 'A'; return match value { 'A' => 1, _ => 0 }; }",
            "Match expressions are not supported",
        ),
    ] {
        expect_rejection(label, source, expected);
    }
}

#[test]
fn core_072_checked_metadata_preserves_char_distinct_from_int_and_bool() {
    let tokens = try_tokenize_with_locations(
        "fn echo(value: char) -> char { value } fn main() -> int { if echo('界') == '界' { return 197; } 1 }",
        None,
    )
    .expect("lex checked metadata probe");
    let ast = parse_with_locations(tokens).expect("parse checked metadata probe");
    let mut analyzer = SemanticAnalyzer::new();
    let (_, analyzed) = analyzer
        .analyze(ast)
        .expect("analyze checked metadata probe");
    let checked = IrGenerator::new()
        .try_generate_ir(analyzed)
        .expect("generate checked character IR");
    let echo = checked
        .metadata()
        .functions
        .get("echo")
        .expect("checked echo metadata");
    assert_eq!(echo.signature.parameters[0].1, LogicalType::Char);
    assert_eq!(echo.signature.result, LogicalType::Char);
    assert_ne!(echo.signature.result, LogicalType::Int);
    assert_ne!(echo.signature.result, LogicalType::Bool);

    let llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("emit checked character LLVM");
    assert!(
        llvm.contains("define i32 @echo(i32 %aero.arg.value)"),
        "{llvm}"
    );
}

#[test]
fn core_072_tracked_direct_module_and_public_check_build_surfaces_are_exact() {
    let root = repository_root();
    let example_root = root.join("examples/char_copydata/main.aero");
    let example_module = root.join("examples/char_copydata/glyphs.aero");
    let root_source = fs::read_to_string(&example_root).expect("read tracked character root");
    let module_source = fs::read_to_string(&example_module).expect("read tracked character module");
    for (label, source, anchors) in [
        (
            "root",
            root_source.as_str(),
            &["mod glyphs;", "return 197;", "['😊']"][..],
        ),
        (
            "module",
            module_source.as_str(),
            &[
                "struct Glyph",
                "Many(char, Glyph)",
                "fn replace(value: &mut char)",
            ][..],
        ),
    ] {
        for anchor in anchors {
            assert!(
                source.contains(anchor),
                "tracked {label} missing {anchor:?}"
            );
        }
    }

    let llvm = compile_file(&example_root, CompilerOptions::default())
        .expect("compile tracked direct-module character example");
    assert!(llvm.contains("define i32 @main()"), "{llvm}");
    assert!(llvm.contains("ret i32 197"), "{llvm}");

    let workspace = TestWorkspace::new("cli");
    let artifact = workspace.root.join("char-copydata.ll");
    let check = Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&root)
        .arg("check")
        .arg(&example_root)
        .output()
        .expect("run public check");
    assert!(
        check.status.success(),
        "public check failed: {}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let build = Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&root)
        .arg("build")
        .arg(&example_root)
        .arg("-o")
        .arg(&artifact)
        .output()
        .expect("run public build");
    assert!(
        build.status.success(),
        "public build failed: {}{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(artifact.is_file(), "public build omitted LLVM artifact");

    let invalid = workspace.root.join("invalid.aero");
    let invalid_artifact = workspace.root.join("invalid.ll");
    fs::write(&invalid, "fn main() -> int { let value = 'ab'; 0 }")
        .expect("write invalid character source");
    let rejected = Command::new(env!("CARGO_BIN_EXE_aero"))
        .current_dir(&root)
        .arg("build")
        .arg(&invalid)
        .arg("-o")
        .arg(&invalid_artifact)
        .output()
        .expect("run rejected public build");
    assert!(
        !rejected.status.success(),
        "invalid public build exited zero"
    );
    assert!(
        !invalid_artifact.exists(),
        "invalid public build published an artifact"
    );

    let workflow =
        fs::read_to_string(root.join(".github/workflows/rust.yml")).expect("read Rust workflow");
    for anchor in [
        "Test Unicode char CopyData integration example",
        "examples/char_copydata/main.aero",
        "cargo run -- check",
        "cargo run -- run",
        "opt-22 -passes=verify -disable-output",
        "llc-22 -verify-machineinstrs",
        "llc-22 -filetype=obj",
        "clang-22 -no-pie",
        "Unicode char CopyData example passed with exit code 197",
    ] {
        assert!(workflow.contains(anchor), "workflow missing {anchor:?}");
    }
}
