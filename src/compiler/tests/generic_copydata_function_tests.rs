use compiler::{
    CodeGenerator, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};

const EXECUTABLE_PRODUCT: &str = r#"
struct Reading<T> {
    value: T,
    valid: bool,
}

fn choose<T>(first: T, second: T, use_first: bool) -> T {
    if use_first {
        return first;
    }
    return second;
}

fn retain_with_reading<T>(value: T, baseline: Reading<int>) -> T {
    value
}

fn main() -> int {
    let baseline: Reading<int> = Reading { value: 9, valid: 1 < 2 };
    let scalar: int = retain_with_reading(choose(40, 41, 1 < 2), baseline);
    let marker: char = choose('a', 'b', 2 < 1);
    let first: Reading<int> = Reading { value: 1, valid: 1 < 2 };
    let second: Reading<int> = Reading { value: 2, valid: 2 < 3 };
    let selected: Reading<int> = choose(first, second, 3 < 4);
    if marker == 'b' && selected.valid {
        return scalar + selected.value;
    }
    return 0;
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
        semantic(source).expect_err("semantics must reject unsupported generic-function use");
    assert!(
        semantic_error.contains(diagnostic),
        "semantic diagnostic omitted {diagnostic:?}: {semantic_error}"
    );
    let checked_error = checked_without_semantics(source)
        .expect_err("raw checked admission must reject unsupported generic-function use");
    assert!(
        checked_error.contains(diagnostic),
        "checked diagnostic omitted {diagnostic:?}: {checked_error}"
    );
    let public_error = compile_program(source, CompilerOptions::default())
        .expect_err("public compilation must reject unsupported generic-function use");
    assert!(
        public_error.contains(diagnostic),
        "public diagnostic omitted {diagnostic:?}: {public_error}"
    );
}

fn assert_all_executable_routes_reject(source: &str) {
    semantic(source).expect_err("semantics must retain the generic-call quarantine");
    checked_without_semantics(source)
        .expect_err("raw checked admission must retain the generic-template quarantine");
    compile_program(source, CompilerOptions::default())
        .expect_err("public compilation must reject quarantined generic use");
}

const COMPLETE_TRANSPORT_PRODUCT: &str = r#"
struct Point { x: int, y: int }

fn select<T>(first: T, second: T, take_first: bool) -> T {
    let mut selected: T = second;
    if take_first {
        selected = first;
    }
    selected
}

fn first_of<T, U>(first: T, second: U, enabled: bool) -> T {
    if enabled { return first; }
    first
}

fn second_declared_first<T, U>(first: U, second: T) -> T {
    second
}

fn score_any<T>(value: T, score: int) -> int {
    score
}

fn ignore<T>(value: T) {
}

fn main() -> int {
    let point: Point = select(Point { x: 7, y: 8 }, Point { x: 9, y: 10 }, 1 < 2);
    let values: [int; 2] = select([2, 3], [4, 5], 1 > 2);
    let pair: (char, bool) = select(('a', 1 < 2), ('b', 2 < 1), 2 < 3);
    let projected: int = first_of(11, 'z', 3 < 4);
    let ordered: int = second_declared_first('q', 13);
    let bonus: int = score_any(pair, 1);
    ignore(point);
    if pair.0 == 'a' && pair.1 {
        return point.x + values[0] + projected + ordered + bonus;
    }
    0
}
"#;

#[test]
fn called_generic_copydata_function_reaches_semantics() {
    semantic(EXECUTABLE_PRODUCT)
        .expect("bound-free generic CopyData function calls must pass semantics");
}

#[test]
fn called_generic_copydata_function_reaches_checked_llvm_deterministically() {
    let checked = checked_without_semantics(EXECUTABLE_PRODUCT)
        .expect("raw checked admission must apply shared generic-function specialization");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(checked)
        .expect("independently verified generic-function IR must lower");
    let first = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("called generic CopyData function must compile");
    let second = compile_program(EXECUTABLE_PRODUCT, CompilerOptions::default())
        .expect("repeat generic-function compilation must succeed");

    assert_eq!(first, second, "generic-function LLVM must be deterministic");
    assert_eq!(first, raw_llvm, "semantic and raw checked routes drifted");
    assert!(
        first.contains("aero.generic.choose<int>")
            && first.contains("aero.generic.choose<char>")
            && first.contains("aero.generic.choose<Reading<int>>"),
        "all concrete specializations must remain distinguishable:\n{first}"
    );
    assert!(
        !first.contains("__aero$generic_function$"),
        "private generic-function identities leaked into LLVM:\n{first}"
    );
}

#[test]
fn complete_type_parameter_and_transport_product_reaches_checked_llvm() {
    semantic(COMPLETE_TRANSPORT_PRODUCT)
        .expect("the complete bound-free CopyData transport product must pass semantics");
    let raw_checked = checked_without_semantics(COMPLETE_TRANSPORT_PRODUCT)
        .expect("raw checked admission must specialize the complete product");
    let raw_llvm = CodeGenerator::new()
        .try_generate_code(raw_checked)
        .expect("the independently verified complete product must lower");
    let public_llvm = compile_program(COMPLETE_TRANSPORT_PRODUCT, CompilerOptions::default())
        .expect("the complete generic-function product must compile");

    assert_eq!(raw_llvm, public_llvm, "trusted routes drifted");
    for specialization in [
        "aero.generic.select<Point>",
        "aero.generic.select<[int;2]>",
        "aero.generic.select<(char,bool)>",
        "aero.generic.first_of<int,char>",
        "aero.generic.second_declared_first<int,char>",
        "aero.generic.score_any<(char,bool)>",
        "aero.generic.ignore<Point>",
    ] {
        assert!(
            public_llvm.contains(specialization),
            "LLVM omitted {specialization:?}:\n{public_llvm}"
        );
    }
}

#[test]
fn unsupported_generic_function_declarations_and_calls_fail_closed_in_all_routes() {
    for (label, source) in [
        (
            "unused type parameter",
            "fn pick<T, U>(value: T) -> T { value } fn main() -> int { pick(1) }",
        ),
        (
            "result-only inference",
            "fn make<T>(value: int) -> T { value } fn main() -> int { make(1) }",
        ),
        (
            "nested array parameter",
            "fn pick<T>(value: [T; 1]) -> T { value[0] } fn main() -> int { pick([1]) }",
        ),
        (
            "nested tuple result",
            "fn pair<T>(value: T) -> (T, int) { (value, 1) } fn main() -> int { pair(1).1 }",
        ),
        (
            "bound",
            "fn pick<T: Copy>(value: T) -> T { value } fn main() -> int { pick(1) }",
        ),
        (
            "where bound",
            "fn pick<T>(value: T) -> T where T: Copy { value } fn main() -> int { pick(1) }",
        ),
    ] {
        assert!(!parsed(source).is_empty(), "{label} fixture must parse");
        assert_all_executable_routes_reject(source);
    }

    for (label, source, diagnostic) in [
        (
            "wrong arity",
            "fn pick<T>(value: T) -> T { value } fn main() -> int { pick(1, 2) }",
            "expects 1 argument(s), actual 2",
        ),
        (
            "duplicate type parameter",
            "fn pick<T, T>(value: T) -> T { value } fn main() -> int { pick(1) }",
            "duplicate or invalid type parameter `T`",
        ),
        (
            "generic entry",
            "fn main<T>(value: T) -> T { value }",
            "generic function `main` has an invalid or reserved name",
        ),
        (
            "duplicate definition",
            "fn pick<T>(value: T) -> T { value } fn pick<U>(value: U) -> U { value } fn main() -> int { pick(1) }",
            "duplicate generic function definition `pick`",
        ),
        (
            "conflicting repeated substitution",
            "fn pick<T>(first: T, second: T) -> T { first } fn main() -> int { pick(1, 'a') }",
            "inferred conflicting types for `T`: int and char",
        ),
        (
            "wrong concrete side argument",
            "fn pick<T>(value: T, enabled: bool) -> T { value } fn main() -> int { pick(1, 2) }",
            "argument for `enabled` requires bool, actual int",
        ),
        (
            "string argument",
            "fn pick<T>(value: T) -> T { value } fn main() -> int { let text = pick(\"a\"); 0 }",
            "requires recursive finite CopyData arguments",
        ),
        (
            "reference argument",
            "fn pick<T>(value: T) -> T { value } fn main() -> int { let value = 1; let alias = pick(&value); 0 }",
            "requires recursive finite CopyData arguments",
        ),
        (
            "owned enum argument",
            "enum State { Ready } fn pick<T>(value: T) -> T { value } fn main() -> int { let selected = pick(State::Ready); 0 }",
            "requires exact CopyData argument types",
        ),
        (
            "generic recursion",
            "fn recur<T>(value: T) -> int { recur(1); 0 } fn main() -> int { recur(1) }",
            "calls generic function `recur`",
        ),
        (
            "generic-to-generic call",
            "fn inner<T>(value: T) -> T { value } fn outer<U>(value: U) -> int { inner(1); 0 } fn main() -> int { outer(1) }",
            "calls generic function `inner`",
        ),
    ] {
        assert!(!parsed(source).is_empty(), "{label} fixture must parse");
        assert_shared_rejection(source, diagnostic);
    }
}

#[test]
fn every_forbidden_parametric_value_use_shares_one_rejection_boundary() {
    for (label, body) in [
        ("arithmetic", "value + value"),
        ("comparison", "if value == value { return value; } value"),
        ("logical", "if value && value { return value; } value"),
        ("method receiver", "value.len(); value"),
        ("projection", "value.0"),
        ("index", "value[0]"),
        ("borrow", "let alias = &value; value"),
        ("dereference", "*value"),
        ("aggregate storage", "let stored = [value]; value"),
        ("print", "println!(\"{}\", value); value"),
        ("call argument", "consume(value); value"),
    ] {
        let source = format!(
            "fn consume(value: int) -> int {{ value }} fn bad<T>(value: T) -> T {{ {body} }} fn main() -> int {{ bad(1) }}"
        );
        assert!(!parsed(&source).is_empty(), "{label} fixture must parse");
        assert_shared_rejection(&source, "outside CAP-005 whole-value transport");
    }
}
