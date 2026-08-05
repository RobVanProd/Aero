use compiler::ast::{AstNode, Expression};
use compiler::{
    CheckedIr, CompilerOptions, IrGenerator, SemanticAnalyzer, compile_program,
    parse_with_locations, try_tokenize_with_locations,
};
use std::fs;
use std::path::PathBuf;

const EXISTING_METHOD_REJECTION: &str =
    "method calls other than exact zero-argument array/Vec .iter() are not admitted";
const PUBLIC_EXISTING_METHOD_REJECTION: &str = "IR Generation Error: method calls other than exact zero-argument array/Vec .iter() are not admitted";

fn parsed(source: &str) -> Vec<AstNode> {
    let tokens = try_tokenize_with_locations(source, None).expect("fixture must lex");
    parse_with_locations(tokens).expect("fixture must parse")
}

fn analyzed(source: &str) -> Result<Vec<AstNode>, String> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(parsed(source)).map(|(_, ast)| ast)
}

fn checked_ast(ast: Vec<AstNode>) -> Result<CheckedIr, String> {
    IrGenerator::new()
        .try_generate_ir(ast)
        .map_err(|error| error.to_string())
}

fn checked_source(source: &str) -> Result<CheckedIr, String> {
    analyzed(source).and_then(checked_ast)
}

fn checked_parsed_source(source: &str) -> Result<CheckedIr, String> {
    checked_ast(parsed(source))
}

fn raw_debug(source: &str) -> String {
    let mut functions = IrGenerator::new()
        .generate_ir(parsed(source))
        .into_iter()
        .collect::<Vec<_>>();
    functions.sort_by(|left, right| left.0.cmp(&right.0));
    format!("{functions:#?}")
}

fn expect_acceptance<T>(failures: &mut Vec<String>, label: &str, result: Result<T, String>) {
    if let Err(error) = result {
        failures.push(format!("{label}: unexpectedly rejected: {error}"));
    }
}

fn expect_exact_rejection<T>(
    failures: &mut Vec<String>,
    label: &str,
    result: Result<T, String>,
    expected: &str,
) {
    match result {
        Ok(_) => failures.push(format!("{label}: unexpectedly accepted")),
        Err(error) if error != expected => failures.push(format!(
            "{label}: diagnostic {error:?} did not equal {expected:?}"
        )),
        Err(_) => {}
    }
}

fn expect_same_raw(failures: &mut Vec<String>, label: &str, candidate: &str, control: &str) {
    let candidate = raw_debug(candidate);
    let control = raw_debug(control);
    if candidate != control {
        failures.push(format!(
            "{label}: raw fabricated-zero compatibility changed\ncandidate:\n{candidate}\ncontrol:\n{control}"
        ));
    }
}

#[test]
fn static_string_boolean_predicate_class_is_complete_and_ci_executable() {
    const CONTENT_AND_PROVENANCE: &str = r#"
        fn main() -> int {
            let empty = "";
            let ascii = "Aero language";
            let alias = ascii;
            let alias_two = alias;
            let annotated: String = "prefix-middle-suffix";
            let annotated_root = "rocket 🚀";
            let annotated_alias: String = annotated_root;
            let escapes = "\n\t\r\\\"\0\q";
            let normalized_fstring = f"raw {text}";
            let needle = "Aero";
            let needle_alias = needle;
            let needle_alias_two = needle_alias;
            let annotated_needle: String = "prefix";
            let suffix_root = "suffix";
            let annotated_suffix: String = suffix_root;
            let fstring_needle = f"{text}";
            let shadow = "outer";
            let shadow_needle = "outer";
            {
                let nested = alias_two;
                let nested_needle = needle_alias_two;
                let shadow = "inner";
                let shadow_needle = "in";
                nested.contains(nested_needle);
                shadow.starts_with(shadow_needle);
            }
            if empty.is_empty()
                && !ascii.is_empty()
                && ascii.contains(needle)
                && alias.contains("language")
                && alias_two.starts_with(needle_alias_two)
                && annotated.starts_with(annotated_needle)
                && annotated.ends_with(annotated_suffix)
                && annotated_alias.contains("🚀")
                && escapes.contains("\0")
                && normalized_fstring.ends_with(fstring_needle)
                && shadow.ends_with(shadow_needle)
                && ("direct").contains("rec")
            {
                return 43;
            }
            return 1;
        }
    "#;
    const CONTENT_RELATIONS: &str = r#"
        fn main() -> int {
            if "".is_empty()
                && "".contains("")
                && "".starts_with("")
                && "".ends_with("")
                && !"".contains("a")
                && !"".starts_with("a")
                && !"".ends_with("a")
                && "abc".contains("")
                && "abc".starts_with("")
                && "abc".ends_with("")
                && "abc".contains("abc")
                && "abc".starts_with("abc")
                && "abc".ends_with("abc")
                && "abcabc".contains("bca")
                && "abcabc".starts_with("abc")
                && "abcabc".ends_with("abc")
                && !"abc".contains("abcd")
                && !"abc".starts_with("bc")
                && !"abc".ends_with("ab")
                && !"abc".contains("z")
                && "é🚀中".contains("🚀")
                && "é🚀中".starts_with("é")
                && "é🚀中".ends_with("中")
                && !"é".contains("é")
                && "👩‍🚀".contains("‍")
                && "Aé🚀中z".contains("é🚀")
                && "\n\t\r\\\"\0\q".contains("\q")
            {
                return 43;
            }
            return 2;
        }
    "#;
    const BOOL_POSITIONS: &str = r#"
        fn identity(value: bool) -> bool { return value; }
        fn implicit() -> bool { "implicit".starts_with("imp") }
        fn main() -> int {
            let nonempty: bool = !"a".is_empty();
            let contains = "abc".contains("b");
            let inverted = !"abc".contains("z");
            let conjunction = "abc".starts_with("a") && "abc".ends_with("c");
            let disjunction = "abc".contains("z") || "abc".contains("b");
            let bool_equality = "abc".starts_with("a") == "abc".ends_with("c");
            { let nested = "nested".contains("est"); nested; }
            while "loop".contains("missing") {
                "body".is_empty();
                break;
            }
            println!("{}", "print".ends_with("int"));
            "discard".contains("card");
            if identity(nonempty) && implicit() && contains && inverted
                && conjunction && disjunction && bool_equality
            {
                return 43;
            }
            return 3;
        }
    "#;
    const LEGACY_TOP_LEVEL: &str =
        "let text = \"Aero\"; let found = text.contains(\"er\"); if found { return 43; } return 1;";

    let mut failures = Vec::new();

    let example_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/static_string_predicates.aero");
    let example = match fs::read_to_string(&example_path) {
        Ok(example) => Some(example),
        Err(error) => {
            failures.push(format!(
                "complete executable example {} is unavailable: {error}",
                example_path.display()
            ));
            None
        }
    };

    for (label, source) in [
        ("content/provenance product", CONTENT_AND_PROVENANCE),
        ("content-relation product", CONTENT_RELATIONS),
        ("Bool placement/traversal product", BOOL_POSITIONS),
        ("legacy top-level statement route", LEGACY_TOP_LEVEL),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} semantics"),
            analyzed(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} checked IR"),
            checked_source(source),
        );
        expect_acceptance(
            &mut failures,
            &format!("{label} public compile"),
            compile_program(source, CompilerOptions::default()),
        );
    }

    if let Some(example) = example {
        expect_acceptance(
            &mut failures,
            "complete executable example semantics",
            analyzed(&example),
        );
        expect_acceptance(
            &mut failures,
            "complete executable example checked IR",
            checked_source(&example),
        );
        match compile_program(&example, CompilerOptions::default()) {
            Err(error) => failures.push(format!("example compilation failed: {error}")),
            Ok(first) => {
                match compile_program(&example, CompilerOptions::default()) {
                    Ok(second) if second == first => {}
                    Ok(_) => failures.push(
                        "single-function static predicate LLVM was not byte-identical".to_string(),
                    ),
                    Err(error) => {
                        failures.push(format!("second example compilation failed: {error}"))
                    }
                }
                if !first.contains("ret i32 43") {
                    failures.push(format!("example omitted exact static exit 43:\n{first}"));
                }
                if !first.contains("icmp eq i32 0, 0") {
                    failures.push(format!(
                        "example omitted Bool-producing constant comparison:\n{first}"
                    ));
                }
                for forbidden in [
                    "strstr",
                    "string_contains",
                    "string_starts_with",
                    "string_ends_with",
                    "StringPredicate",
                ] {
                    if first.contains(forbidden) {
                        failures.push(format!(
                            "example activated forbidden runtime marker {forbidden}:\n{first}"
                        ));
                    }
                }
            }
        }
    }

    match checked_source(
        "fn main() { let found = \"é🚀中\".contains(\"🚀\"); let empty = \"\".is_empty(); }",
    ) {
        Err(error) => failures.push(format!("checked predicate IR specimen failed: {error}")),
        Ok(ir) => {
            let debug = format!("{ir:?}");
            if !debug.contains("ICmp") || !debug.contains("op: \"eq\"") {
                failures.push(format!("checked predicates omitted Bool ICmp:\n{debug}"));
            }
            for forbidden in ["ImmString", "StringPredicate", "string_contains", "strstr"] {
                if debug.contains(forbidden) {
                    failures.push(format!(
                        "checked IR retained forbidden String marker {forbidden}:\n{debug}"
                    ));
                }
            }
        }
    }

    for (label, source, expected) in [
        (
            "mutable literal receiver",
            "fn main() { let mut text = \"a\"; let found = text.contains(\"a\"); }",
            "mutable string bindings are not admitted; checked strings are immutable compile-time aliases",
        ),
        (
            "String parameter receiver",
            "fn probe(text: String) -> bool { return text.contains(\"a\"); }",
            "function parameter `text` is not an admitted scalar type",
        ),
        (
            "String parameter argument",
            "fn probe(needle: String) -> bool { return \"a\".contains(needle); }",
            "function parameter `needle` is not an admitted scalar type",
        ),
        (
            "String return",
            "fn probe() -> String { return \"a\"; }",
            "function return type is not an admitted scalar or Void type",
        ),
        (
            "generic function outer boundary",
            "fn probe<T>() -> bool { return \"a\".contains(\"a\"); }",
            "generic function IR is not admitted in CORE-010",
        ),
        (
            "non-generic impl boundary",
            "impl Widget { fn probe() { let found = \"a\".contains(\"a\"); } }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "generic impl boundary",
            "impl<T> Widget { fn probe() { let found = \"a\".starts_with(\"a\"); } }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "closure boundary fails closed before predicate classification",
            "fn main() { let probe = |ignored: int| \"a\".ends_with(\"a\"); let found = probe(0); }",
            "Error: closure expressions are parsed but unsupported in executable code at 1:25.",
        ),
        (
            "receiver child precedes classification",
            "fn main() { let found = missing.contains(\"a\"); }",
            "checked IR has no binding for `missing`",
        ),
        (
            "admitted tuple argument reaches arity classification",
            "fn main() { let found = \"a\".contains((1, 2), \"a\"); }",
            "compile-time string .contains() expects exactly 1 argument, got 2",
        ),
        (
            "earlier missing argument precedes later nested predicate",
            "fn main() { let found = \"a\".contains(missing, \"x\".is_empty()); }",
            "checked IR has no binding for `missing`",
        ),
        (
            "second argument child precedes arity classification",
            "fn main() { let found = \"a\".contains(\"a\", missing); }",
            "checked IR has no binding for `missing`",
        ),
        (
            "non-String predicate argument",
            "fn main() { let found = \"a\".contains(1); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "String-producing receiver chain",
            "fn main() { let found = \"a\".trim().contains(\"a\"); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "predicate result used as receiver",
            "fn main() { let found = \"a\".contains(\"a\").is_empty(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "field receiver remains excluded",
            "fn main() { let found = missing.field.contains(\"a\"); }",
            "aggregate expression is not admitted in checked IR",
        ),
        (
            "indexed String receiver remains excluded",
            "fn main() { let found = (\"abc\"[0]).contains(\"a\"); }",
            "indexing requires an admitted fixed array",
        ),
        (
            "borrowed receiver remains excluded",
            "fn main() { let found = (&\"a\").contains(\"a\"); }",
            "a local immutable Copy-data borrow requires an identifier place",
        ),
        (
            "dereferenced receiver remains excluded",
            "fn main() { let text = \"a\"; let found = (*text).contains(\"a\"); }",
            "cannot dereference a non-reference value",
        ),
        (
            "array is_empty remains excluded",
            "fn main() { let values = [1, 2]; let empty = values.is_empty(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "array contains remains excluded",
            "fn main() { let values = [1, 2]; let found = values.contains(1); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "Vec is_empty remains outside checked parameters",
            "fn probe(values: Vec<int>) -> bool { return values.is_empty(); }",
            "function parameter `values` is not an admitted scalar type",
        ),
        (
            "HashMap contains_key remains outside checked parameters",
            "fn probe(values: HashMap<int, int>) -> bool { return values.contains_key(1); }",
            "function parameter `values` is not an admitted scalar type",
        ),
        (
            "case-variant method remains excluded",
            "fn main() { let found = \"a\".Contains(\"a\"); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "other String method remains excluded",
            "fn main() { let changed = \"a\".to_uppercase(); }",
            EXISTING_METHOD_REJECTION,
        ),
        (
            "is_empty one argument",
            "fn main() { let empty = \"a\".is_empty(1); }",
            "compile-time string .is_empty() expects exactly 0 arguments, got 1",
        ),
        (
            "is_empty two arguments",
            "fn main() { let empty = \"a\".is_empty(1, 2); }",
            "compile-time string .is_empty() expects exactly 0 arguments, got 2",
        ),
        (
            "contains missing argument",
            "fn main() { let found = \"a\".contains(); }",
            "compile-time string .contains() expects exactly 1 argument, got 0",
        ),
        (
            "contains two arguments",
            "fn main() { let found = \"a\".contains(\"a\", \"b\"); }",
            "compile-time string .contains() expects exactly 1 argument, got 2",
        ),
        (
            "starts_with missing argument",
            "fn main() { let found = \"a\".starts_with(); }",
            "compile-time string .starts_with() expects exactly 1 argument, got 0",
        ),
        (
            "starts_with two arguments",
            "fn main() { let found = \"a\".starts_with(\"a\", \"b\"); }",
            "compile-time string .starts_with() expects exactly 1 argument, got 2",
        ),
        (
            "ends_with missing argument",
            "fn main() { let found = \"a\".ends_with(); }",
            "compile-time string .ends_with() expects exactly 1 argument, got 0",
        ),
        (
            "ends_with two arguments",
            "fn main() { let found = \"a\".ends_with(\"a\", \"b\"); }",
            "compile-time string .ends_with() expects exactly 1 argument, got 2",
        ),
    ] {
        expect_exact_rejection(
            &mut failures,
            label,
            checked_parsed_source(source),
            expected,
        );
        if expected.starts_with("compile-time string .") {
            let public_expected = format!("IR Generation Error: {expected}");
            expect_exact_rejection(
                &mut failures,
                &format!("{label} public boundary"),
                compile_program(source, CompilerOptions::default()),
                &public_expected,
            );
        }
    }

    for (label, binding) in [
        (
            "lowercase string argument annotation",
            "let text: string = \"a\";",
        ),
        (
            "custom named argument annotation",
            "let text: Widget = \"a\";",
        ),
        (
            "custom annotated argument alias",
            "let base = \"a\"; let text: Widget = base;",
        ),
        (
            "immutable reference argument annotation",
            "let text: &String = \"a\";",
        ),
        (
            "mutable reference argument annotation",
            "let text: &mut String = \"a\";",
        ),
        (
            "one-argument generic argument annotation",
            "let text: Vec<int> = \"a\";",
        ),
        (
            "two-argument generic argument annotation",
            "let text: HashMap<String, int> = \"a\";",
        ),
        (
            "zero-count array argument annotation",
            "let text: [int; 0] = \"a\";",
        ),
        (
            "nested array argument annotation",
            "let text: [[int; 1]; 1] = \"a\";",
        ),
        (
            "array-around-reference argument annotation",
            "let text: [&String; 0] = \"a\";",
        ),
    ] {
        let source = format!("fn main() {{ {binding} let found = \"a\".contains(text); }}");
        expect_acceptance(
            &mut failures,
            &format!("{label} keeps prior semantic preservation"),
            analyzed(&source),
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked provenance quarantine"),
            checked_source(&source),
            EXISTING_METHOD_REJECTION,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} public provenance quarantine"),
            compile_program(&source, CompilerOptions::default()),
            PUBLIC_EXISTING_METHOD_REJECTION,
        );
    }

    for (label, source) in [
        (
            "lowercase string annotation",
            "fn main() { let text: string = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "custom named annotation",
            "fn main() { let text: Widget = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "custom annotated alias",
            "fn main() { let base = \"a\"; let text: Widget = base; let found = text.contains(\"a\"); }",
        ),
        (
            "immutable reference annotation",
            "fn main() { let text: &String = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "mutable reference annotation",
            "fn main() { let text: &mut String = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "one-argument generic annotation",
            "fn main() { let text: Vec<int> = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "two-argument generic annotation",
            "fn main() { let text: HashMap<String, int> = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "zero-count array annotation",
            "fn main() { let text: [int; 0] = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "nested array annotation",
            "fn main() { let text: [[int; 1]; 1] = \"a\"; let found = text.contains(\"a\"); }",
        ),
        (
            "array-around-reference annotation",
            "fn main() { let text: [&String; 0] = \"a\"; let found = text.contains(\"a\"); }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} keeps prior semantic preservation"),
            analyzed(source),
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} checked provenance quarantine"),
            checked_source(source),
            EXISTING_METHOD_REJECTION,
        );
        expect_exact_rejection(
            &mut failures,
            &format!("{label} public provenance quarantine"),
            compile_program(source, CompilerOptions::default()),
            PUBLIC_EXISTING_METHOD_REJECTION,
        );
    }

    for (label, source) in [
        (
            "existing static String len",
            "fn main() -> int { return \"é🚀中\".len(); }",
        ),
        (
            "existing exact String equality",
            "fn main() -> int { if \"a\" == \"a\" { return 41; } return 0; }",
        ),
        (
            "existing fixed-array iter",
            "fn main() { for item in [1, 2].iter() { item; } }",
        ),
    ] {
        expect_acceptance(
            &mut failures,
            &format!("{label} checked behavior"),
            checked_source(source),
        );
    }

    let top_level_expression = vec![AstNode::Expression(Expression::MethodCall {
        object: Box::new(Expression::StringLiteral("a".to_string())),
        method: "contains".to_string(),
        arguments: vec![Expression::StringLiteral("a".to_string())],
    })];
    expect_exact_rejection(
        &mut failures,
        "top-level AstNode::Expression boundary",
        checked_ast(top_level_expression),
        "top-level expressions are not admitted in checked IR",
    );

    const TRAIT_DEFAULT: &str =
        "trait Contract<T> { fn probe() { let found = \"a\".contains(\"a\"); } }";
    expect_acceptance(
        &mut failures,
        "trait default remains semantic syntax-only",
        analyzed(TRAIT_DEFAULT),
    );
    expect_acceptance(
        &mut failures,
        "trait default remains checked syntax-only",
        checked_source(TRAIT_DEFAULT),
    );

    expect_same_raw(
        &mut failures,
        "ordinary raw route",
        "return \"abc\".contains(\"b\");",
        "return 0;",
    );
    expect_same_raw(
        &mut failures,
        "named-function raw route",
        "fn probe() -> bool { return \"abc\".starts_with(\"a\"); }",
        "fn probe() -> bool { return 0; }",
    );
    expect_same_raw(
        &mut failures,
        "closure raw route",
        "let probe = |ignored: int| \"abc\".ends_with(\"c\"); return 0;",
        "let probe = |ignored: int| 0; return 0;",
    );

    let workflow_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/rust.yml");
    match fs::read_to_string(&workflow_path) {
        Err(error) => failures.push(format!(
            "could not inspect Rust CI workflow {}: {error}",
            workflow_path.display()
        )),
        Ok(workflow) => {
            for anchor in [
                "Test static string Boolean predicates example",
                "cargo run -- build ../../examples/static_string_predicates.aero",
                "opt-22 -passes=verify -disable-output ../../static_string_predicates.ll",
                "llc-22 -verify-machineinstrs ../../static_string_predicates.ll -o /dev/null",
                "llc-22 -filetype=obj ../../static_string_predicates.ll -o ../../static_string_predicates.o",
                "clang-22 ../../static_string_predicates.o -o ../../static_string_predicates",
                "../../static_string_predicates",
                "if [ $exit_code -ne 43 ]; then",
                "static_string_predicates test passed with exit code $exit_code",
            ] {
                if !workflow.contains(anchor) {
                    failures.push(format!(
                        "Rust CI omitted static-predicate anchor {anchor:?}"
                    ));
                }
            }
        }
    }

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
