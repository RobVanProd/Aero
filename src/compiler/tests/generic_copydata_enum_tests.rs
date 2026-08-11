use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const EXECUTABLE_PRODUCT: &str = r#"
struct Reading<T> {
    value: T,
    valid: bool,
}

enum Sample<T> {
    Present(T),
    Missing,
}

fn make_reading() -> Reading<int> {
    Reading { value: 1, valid: 1 < 2 }
}

fn score_number(value: Sample<int>) -> int {
    match value {
        Sample::Present(number) => number,
        Sample::Missing => 0,
    }
}

fn score_marker(value: Sample<char>) -> int {
    match value {
        Sample::Present(marker) => 2,
        Sample::Missing => 0,
    }
}

fn score_reading(value: Sample<Reading<int>>) -> int {
    match value {
        Sample::Present(reading) => reading.value,
        Sample::Missing => 0,
    }
}

fn main() -> int {
    let number: Sample<int> = Sample::Present(40);
    let marker: Sample<char> = Sample::Present('a');
    let reading: Sample<Reading<int>> = Sample::Present(make_reading());
    return score_number(number) + score_marker(marker) + score_reading(reading);
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
    let semantic_error = semantic(source).expect_err("semantics must reject excluded generic enum");
    assert!(
        semantic_error.contains(diagnostic),
        "semantic diagnostic omitted {diagnostic:?}: {semantic_error}"
    );
    let checked_error = checked_without_semantics(source)
        .expect_err("raw checked admission must reject excluded generic enum");
    assert!(
        checked_error.contains(diagnostic),
        "checked diagnostic omitted {diagnostic:?}: {checked_error}"
    );
    let public_error = compile_program(source, CompilerOptions::default())
        .expect_err("public compilation must reject excluded generic enum");
    assert!(
        public_error.contains(diagnostic),
        "public diagnostic omitted {diagnostic:?}: {public_error}"
    );
}

const COMPLETE_PRODUCT: &str = r#"
enum Outcome<T, U> {
    Ready(T, [U; 2]),
    Cached((T, U)),
    Missing,
}

fn make_ready(value: int) -> Outcome<int, char> {
    Outcome::Ready(value, ['a', 'b'])
}

fn make_cached() -> Outcome<int, char> {
    Outcome::Cached((7, 'z'))
}

fn score(value: Outcome<int, char>) -> int {
    match value {
        Outcome::Ready(number, markers) => number,
        Outcome::Cached(pair) => pair.0,
        Outcome::Missing => 0,
    }
}

fn main() -> int {
    let mut state: Outcome<int, char> = make_ready(11);
    if 1 > 2 {
        state = Outcome::Ready(13, ['x', 'y']);
    } else {
        state = make_cached();
    }
    let mut step = 0;
    while step < 1 {
        state = Outcome::Cached((7, 'z'));
        step = step + 1;
    }
    return score(state) + score(Outcome::Missing);
}
"#;

#[test]
fn explicit_generic_copydata_enums_reach_both_trusted_routes() {
    semantic(EXECUTABLE_PRODUCT)
        .expect("explicit generic CopyData enum applications must pass semantics");
    let checked = checked_without_semantics(EXECUTABLE_PRODUCT)
        .expect("raw checked admission must apply shared generic-enum specialization");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-enum IR must lower");
    let public_llvm = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("explicit generic CopyData enum applications must compile");

    assert_eq!(raw_llvm, public_llvm, "trusted generic-enum routes drifted");
    for identity in ["Sample<int>", "Sample<char>", "Sample<Reading<int>>"] {
        assert!(
            public_llvm.contains(identity),
            "specialized enum identity {identity:?} is not source-readable:\n{public_llvm}"
        );
    }
    assert!(
        !public_llvm.contains("__aero$generic_enum$"),
        "private generic-enum identity leaked into LLVM:\n{public_llvm}"
    );
}

#[test]
fn generic_enum_bound_metadata_is_not_silently_discarded() {
    let ast = parsed("enum Sample<T: Comparable> { Present(T), Missing }");
    let [
        compiler::ast::AstNode::Statement(compiler::ast::Statement::EnumDef {
            trait_bounds, ..
        }),
    ] = ast.as_slice()
    else {
        panic!("bounded generic-enum fixture did not retain its declaration")
    };
    assert_eq!(
        trait_bounds,
        &vec![("T".to_string(), vec!["Comparable".to_string()])]
    );
    assert_shared_rejection(
        "enum Sample<T: Comparable> { Present(T), Missing } fn main() -> int { 0 }",
        "generic enum `Sample` trait bounds are not admitted in CAP-006",
    );
}

#[test]
fn complete_generic_enum_declaration_and_context_product_is_executable() {
    semantic(COMPLETE_PRODUCT).expect("multi-parameter generic enum product must pass semantics");
    let raw = CodeGenerator::new()
        .try_generate_code(
            checked_without_semantics(COMPLETE_PRODUCT)
                .expect("multi-parameter product must pass raw checked admission"),
        )
        .expect("multi-parameter checked product must lower");
    let public = compile_program(COMPLETE_PRODUCT, CompilerOptions::default())
        .expect("multi-parameter generic enum product must compile");
    assert_eq!(raw, public, "multi-parameter trusted routes drifted");
    assert!(public.contains("; Aero generic enum: Outcome<int,char>"));
    assert!(!public.contains("__aero$generic_enum$"));
}

#[test]
fn excluded_generic_enum_class_fails_closed_in_all_trusted_routes() {
    for (label, source, diagnostic) in [
        (
            "duplicate definition",
            "enum E<T> { V(T) } enum E<T> { V(T) } fn main() -> int { 0 }",
            "duplicate generic enum definition `E`",
        ),
        (
            "reserved carrier name",
            "enum Option<T> { Value(T) } fn main() -> int { 0 }",
            "generic enum `Option` has an invalid definition",
        ),
        (
            "empty definition",
            "enum E<T> {} fn main() -> int { 0 }",
            "generic enum `E` has an invalid definition",
        ),
        (
            "duplicate parameter",
            "enum E<T, T> { V(T) } fn main() -> int { 0 }",
            "duplicate or invalid type parameter `T`",
        ),
        (
            "unused parameter",
            "enum E<T, U> { V(T) } fn main() -> int { 0 }",
            "generic enum `E` has unused type parameter(s): U",
        ),
        (
            "duplicate variant",
            "enum E<T> { V(T), V(T) } fn main() -> int { 0 }",
            "generic enum `E` has duplicate or invalid variant `V`",
        ),
        (
            "empty positional declaration",
            "enum E<T> { Empty(), V(T) } fn main() -> int { 0 }",
            "generic enum `E` variant `Empty` cannot use an empty positional field list",
        ),
        (
            "named-field variant",
            "enum E<T> { V { value: T } } fn main() -> int { 0 }",
            "generic enum `E` named-field variants are not admitted in CAP-006",
        ),
        (
            "recursive template",
            "enum E<T> { Next(E), V(T) } fn main() -> int { 0 }",
            "recursive generic enum `E` is not admitted in CAP-006",
        ),
        (
            "nested generic template",
            "enum E<T> { V(Vec<T>) } fn main() -> int { 0 }",
            "nested generic applications in generic enum `E` payloads are not admitted in CAP-006",
        ),
        (
            "reference template",
            "enum E<T> { V(&T) } fn main() -> int { 0 }",
            "generic enum `E` payloads must be recursive finite CopyData",
        ),
        (
            "unknown concrete template field",
            "enum E<T> { V(T, Missing) } fn main() -> int { 0 }",
            "generic enum `E` payloads must be recursive finite CopyData",
        ),
        (
            "missing arguments",
            "enum E<T> { V(T) } fn main() -> int { let value: E = E::V(1); 0 }",
            "generic enum `E` requires explicit type arguments in binding annotations",
        ),
        (
            "wrong application arity",
            "enum E<T> { V(T) } fn main() -> int { let value: E<int, char> = E::V(1); 0 }",
            "generic enum `E` requires 1 type argument(s), actual 2",
        ),
        (
            "String argument",
            "enum E<T> { V(T) } fn main() -> int { let value: E<String> = E::V(\"x\"); 0 }",
            "generic enum application `E<String>` is not recursive finite CopyData",
        ),
        (
            "reference argument",
            "enum E<T> { V(T) } fn main() -> int { let value: int = 1; let item: E<&int> = E::V(&value); 0 }",
            "generic enum application `E<&int>` is not recursive finite CopyData",
        ),
        (
            "owned enum argument",
            "enum Owned { A } enum E<T> { V(T) } fn main() -> int { let item: E<Owned> = E::V(Owned::A); 0 }",
            "generic enum application `E<Owned>` is not recursive finite CopyData",
        ),
        (
            "nested generic argument",
            "enum E<T> { V(T) } fn main() -> int { let item: E<Vec<int>> = E::V(1); 0 }",
            "generic enum application `E<Vec<int>>` is not recursive finite CopyData",
        ),
        (
            "generic-to-generic argument",
            "enum E<T> { V(T) } fn main() -> int { let item: E<E<int>> = E::V(E::V(1)); 0 }",
            "generic enum application `E<E<int>>` is not recursive finite CopyData",
        ),
        (
            "cyclic struct argument",
            "struct Node { next: Node } enum E<T> { V(T) } fn take(value: E<Node>) -> int { 0 } fn main() -> int { 0 }",
            "generic enum application `E<Node>` is not recursive finite CopyData",
        ),
        (
            "context-free constructor",
            "enum E<T> { V(T) } fn main() -> int { E::V(1); 0 }",
            "generic enum constructor `E::V` requires an exact expected E<...> type",
        ),
        (
            "inferred binding constructor",
            "enum E<T> { V(T) } fn main() -> int { let value = E::V(1); 0 }",
            "generic enum constructor `E::V` requires an exact expected E<...> type",
        ),
        (
            "cross-family constructor",
            "enum E<T> { V(T) } enum F<T> { V(T) } fn main() -> int { let item: E<int> = F::V(1); 0 }",
            "generic enum constructor `F::V` does not match expected type E<int>",
        ),
        (
            "unknown variant",
            "enum E<T> { V(T) } fn main() -> int { let item: E<int> = E::Missing(1); 0 }",
            "generic enum `E<int>` has no variant `Missing`",
        ),
        (
            "missing payload",
            "enum E<T> { V(T) } fn main() -> int { let item: E<int> = E::V; 0 }",
            "generic enum `E<int>` variant `V` requires 1 positional field(s)",
        ),
        (
            "unexpected unit payload",
            "enum E<T> { V(T), Empty } fn main() -> int { let item: E<int> = E::Empty(1); 0 }",
            "generic enum `E<int>` variant `Empty` does not accept payload data",
        ),
        (
            "constructor arity",
            "enum E<T> { V(T, int) } fn main() -> int { let item: E<int> = E::V(1); 0 }",
            "generic enum `E<int>` variant `V` requires 2 positional field(s), actual 1",
        ),
        (
            "constructor payload type",
            "enum E<T> { V(T) } fn main() -> int { let item: E<int> = E::V('a'); 0 }",
            "enum `E<int>` variant `V` payload type mismatch: expected int, actual char",
        ),
        (
            "cross-specialization assignment",
            "enum E<T> { V(T) } fn main() -> int { let mut target: E<int> = E::V(1); let source: E<char> = E::V('a'); target = source; 0 }",
            "type mismatch",
        ),
        (
            "cross-specialization argument",
            "enum E<T> { V(T) } fn take(value: E<int>) -> int { 0 } fn main() -> int { let value: E<char> = E::V('a'); take(value) }",
            "type mismatch: expected E<int>, actual E<char>",
        ),
        (
            "cross-specialization return",
            "enum E<T> { V(T) } fn make() -> E<int> { let value: E<char> = E::V('a'); value } fn main() -> int { 0 }",
            "type mismatch",
        ),
        (
            "cross-family pattern",
            "enum E<T> { V(T) } enum F<T> { V(T) } fn main() -> int { let value: E<int> = E::V(1); match value { F::V(number) => number } }",
            "generic enum pattern `F::V` does not match scrutinee type E<int>",
        ),
        (
            "struct storage",
            "enum E<T> { V(T) } struct Holder { value: E<int> } fn main() -> int { 0 }",
            "generic enum applications are not admitted inside struct fields in CAP-006",
        ),
        (
            "enum storage",
            "enum E<T> { V(T) } enum Holder { Value(E<int>) } fn main() -> int { 0 }",
            "generic enum applications are not admitted inside enum payloads in CAP-006",
        ),
        (
            "array storage",
            "enum E<T> { V(T) } fn main() -> int { let items: [E<int>; 1] = [E::V(1)]; 0 }",
            "generic enum applications are not admitted inside binding annotations in CAP-006",
        ),
        (
            "tuple storage",
            "enum E<T> { V(T) } fn main() -> int { let item: (E<int>, int) = (E::V(1), 2); 0 }",
            "generic enum applications are not admitted inside binding annotations in CAP-006",
        ),
        (
            "reference storage",
            "enum E<T> { V(T) } fn read(value: &E<int>) -> int { 0 } fn main() -> int { 0 }",
            "generic enum applications are not admitted inside function parameters in CAP-006",
        ),
        (
            "local immutable reference",
            "enum E<T> { V(T) } fn main() -> int { let owner: E<int> = E::V(1); let alias = &owner; 0 }",
            "generic enum references are not admitted in CAP-006",
        ),
        (
            "local mutable reference",
            "enum E<T> { V(T) } fn main() -> int { let mut owner: E<int> = E::V(1); let alias = &mut owner; 0 }",
            "generic enum references are not admitted in CAP-006",
        ),
        (
            "constant storage",
            "enum E<T> { V(T) } const ITEM: E<int> = E::V(1); fn main() -> int { 0 }",
            "unsupported composite annotation",
        ),
        (
            "generic function transport",
            "enum E<T> { V(T) } fn keep<U>(value: E<int>) -> E<int> { value } fn main() -> int { 0 }",
            "generic function `keep` cannot transport an explicit generic enum in CAP-006",
        ),
        (
            "trait signature",
            "enum E<T> { V(T) } trait Read { fn read(value: E<int>) -> int; } fn main() -> int { 0 }",
            "generic enums are not admitted in trait signatures in CAP-006",
        ),
        (
            "impl method",
            "struct Holder { value: int } enum E<T> { V(T) } impl Holder { fn read(value: E<int>) -> int { 0 } } fn main() -> int { 0 }",
            "generic enums are not admitted in impl methods in CAP-006",
        ),
        (
            "closure parameter",
            "enum E<T> { V(T) } fn main() { let callback = |value: E<int>| 1; }",
            "generic enums are not admitted in closure syntax in CAP-006",
        ),
    ] {
        assert!(!parsed(source).is_empty(), "{label} fixture must parse");
        assert_shared_rejection(source, diagnostic);
    }
}
