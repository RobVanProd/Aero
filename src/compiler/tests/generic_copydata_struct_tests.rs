use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const EXECUTABLE_PRODUCT: &str = r#"
struct Reading<T> {
    value: T,
    valid: bool,
}

fn make_reading(value: int, valid: bool) -> Reading<int> {
    Reading { value: value, valid: valid }
}

fn score_reading(reading: Reading<int>) -> int {
    if reading.valid {
        return reading.value;
    }
    return 0;
}

fn score_marker(marker: Reading<char>) -> int {
    if marker.valid && marker.value == 'a' {
        return 2;
    }
    return 0;
}

fn main() -> int {
    let mut reading: Reading<int> = make_reading(40, 1 < 2);
    reading = Reading { value: 41, valid: 2 < 3 };
    let marker: Reading<char> = Reading { value: 'a', valid: 3 < 4 };
    return score_reading(reading) + score_marker(marker);
}
"#;

fn parsed(source: &str) -> Vec<compiler::ast::AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn semantic(source: &str) -> Result<Vec<compiler::ast::AstNode>, String> {
    SemanticAnalyzer::new()
        .analyze(parsed(source))
        .map(|(_, ast)| ast)
}

fn checked_without_semantics(source: &str) -> Result<compiler::CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(parsed(source))
        .map_err(|error| error.to_string())
}

fn assert_shared_rejection(source: &str, diagnostic: &str) {
    let semantic_error =
        semantic(source).expect_err("semantics must reject unsupported generic use");
    assert!(
        semantic_error.contains(diagnostic),
        "semantic diagnostic omitted {diagnostic:?}: {semantic_error}"
    );
    let checked_error = checked_without_semantics(source)
        .expect_err("raw checked admission must reject unsupported generic use");
    assert!(
        checked_error.contains(diagnostic),
        "checked diagnostic omitted {diagnostic:?}: {checked_error}"
    );
    let public_error = compile_program(source, CompilerOptions::default())
        .expect_err("public compilation must reject unsupported generic use");
    assert!(
        public_error.contains(diagnostic),
        "public diagnostic omitted {diagnostic:?}: {public_error}"
    );
}

const COMPOSED_COPYDATA_PRODUCT: &str = r#"
struct Leaf { code: char, count: int }
struct Envelope<T, U> {
    first: T,
    repeated: (T, [U; 2]),
    leaf: Leaf,
}
struct Holder { item: Envelope<int, char> }

fn make_envelope(value: int) -> Envelope<int, char> {
    Envelope {
        first: value,
        repeated: (value, ['a', 'b']),
        leaf: Leaf { code: 'z', count: 3 }
    }
}

fn inspect(value: Envelope<int, char>) -> int {
    return value.first + value.repeated.0 + value.leaf.count;
}

fn accept(value: Envelope<int, char>) -> int { inspect(value) }

fn main() -> int {
    let mut value: Envelope<int, char> = make_envelope(5);
    value.first = 6;
    let value_ref: &Envelope<int, char> = &value;
    let copied = *value_ref;
    let array: [Envelope<int, char>; 2] = [copied, copied];
    let tuple: (Envelope<int, char>, int) = (array[1], 2);
    let holder: Holder = Holder { item: tuple.0 };
    return accept(holder.item) + tuple.1;
}
"#;

const NESTED_CONCRETE_APPLICATION: &str = r#"
struct Box<T> { value: T }
struct Wrapper<T> { item: T }

fn main() -> int {
    let wrapped: Wrapper<Box<int>> = Wrapper { item: Box { value: 17 } };
    return wrapped.item.value;
}
"#;

#[test]
fn explicit_generic_copydata_structs_reach_semantics() {
    semantic(EXECUTABLE_PRODUCT)
        .expect("explicit generic CopyData struct applications must pass semantics");
}

#[test]
fn explicit_generic_copydata_structs_reach_checked_llvm_deterministically() {
    let checked = checked_without_semantics(EXECUTABLE_PRODUCT)
        .expect("raw AST admission must apply the shared generic-struct elaboration");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-struct IR must lower");
    let first = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("explicit generic CopyData struct program must compile");
    let second = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("repeat generic CopyData struct compilation must succeed");

    assert_eq!(first, second, "generic-struct LLVM must be deterministic");
    assert_eq!(first, raw_llvm, "semantic and raw checked routes drifted");
    assert!(
        first.contains("define i32 @score_reading(") && first.contains("define i32 @score_marker("),
        "generic-struct consumers were not emitted:\n{first}"
    );
    assert!(
        !first.contains("__aero$generic_struct$"),
        "private generic-struct identities leaked into LLVM:\n{first}"
    );
}

#[test]
fn generic_copydata_substitution_composes_with_aggregates_mutation_and_references() {
    let llvm = compile_program(COMPOSED_COPYDATA_PRODUCT, CompilerOptions::default())
        .expect("generic CopyData structs must reuse admitted aggregate/reference behavior");
    assert!(
        llvm.contains("%\"aero.struct.Envelope<int,char>\" = type")
            && llvm.contains("define i32 @inspect(")
            && llvm.contains("getelementptr inbounds"),
        "composed generic CopyData product omitted expected LLVM evidence:\n{llvm}"
    );
    assert!(!llvm.contains("__aero$generic_struct$"));
}

#[test]
fn concrete_generic_applications_can_compose_recursively() {
    let llvm = compile_program(NESTED_CONCRETE_APPLICATION, CompilerOptions::default())
        .expect("a concrete generic CopyData application must itself be a reusable type argument");
    assert!(
        llvm.contains("%\"aero.struct.Box<int>\" = type")
            && llvm.contains("%\"aero.struct.Wrapper<Box<int>>\" = type"),
        "recursive concrete monomorphization was not source-readable:\n{llvm}"
    );
    assert!(!llvm.contains("__aero$generic_struct$"));
}

#[test]
fn unsupported_generic_struct_class_fails_closed_in_both_trusted_routes() {
    for (label, source, diagnostic) in [
        (
            "context-free literal",
            "struct Box<T> { value: T } fn main() -> int { Box { value: 1 }; 0 }",
            "generic struct literal `Box` requires an exact expected Box<...> type",
        ),
        (
            "wrong arity",
            "struct Box<T> { value: T } fn main() -> int { let value: Box<int, bool> = Box { value: 1 }; 0 }",
            "generic struct `Box` requires 1 type argument(s), actual 2",
        ),
        (
            "unused parameter",
            "struct Box<T> { value: int } fn main() -> int { 0 }",
            "generic struct `Box` has unused type parameter(s): T",
        ),
        (
            "duplicate parameter",
            "struct Pair<T, T> { left: T, right: T } fn main() -> int { 0 }",
            "duplicate or invalid type parameter `T`",
        ),
        (
            "duplicate definition",
            "struct Box<T> { value: T } struct Box<T> { value: T } fn main() -> int { 0 }",
            "duplicate generic struct definition `Box`",
        ),
        (
            "duplicate field",
            "struct Box<T> { value: T, value: T } fn main() -> int { 0 }",
            "generic struct `Box` has duplicate or invalid field `value`",
        ),
        (
            "nested template application",
            "struct Inner<T> { value: T } struct Outer<T> { inner: Inner<T> } fn main() -> int { 0 }",
            "nested generic applications in generic struct `Outer` fields are not admitted",
        ),
        (
            "recursive definition",
            "struct Link<T> { value: T, next: Link } fn main() -> int { 0 }",
            "recursive generic struct `Link` is not admitted",
        ),
        (
            "non-CopyData argument",
            "struct Box<T> { value: T } fn main() -> int { let value: Box<String> = Box { value: \"x\" }; 0 }",
            "generic struct application `Box<String>` is not recursive finite CopyData",
        ),
        (
            "reference argument",
            "struct Box<T> { value: T } fn main() -> int { let value: int = 1; let boxed: Box<&int> = Box { value: &value }; 0 }",
            "generic struct `Box` requires recursive finite CopyData type arguments",
        ),
        (
            "unknown dependency",
            "struct Box<T> { value: T, missing: Missing } fn main() -> int { let value: Box<int> = Box { value: 1, missing: Missing { value: 2 } }; 0 }",
            "generic struct application `Box<int>` is not recursive finite CopyData",
        ),
        (
            "conflicting literal context",
            "struct Box<T> { value: T } struct Other<T> { value: T } fn main() -> int { let value: Box<int> = Other { value: 1 }; 0 }",
            "generic struct literal `Other` does not match expected type Box<int>",
        ),
        (
            "generic function transport",
            "struct Box<T> { value: T } fn identity<T>(value: Box<int>) -> Box<int> { value } fn main() -> int { 0 }",
            "generic function `identity` cannot transport an explicit generic CopyData struct",
        ),
    ] {
        let parsed = parsed(source);
        assert!(!parsed.is_empty(), "{label} fixture must parse");
        assert_shared_rejection(source, diagnostic);
    }
}
