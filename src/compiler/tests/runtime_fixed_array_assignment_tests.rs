use compiler::{CompilerOptions, compile_program};

fn compile(source: &str) -> String {
    compile_program(source, CompilerOptions::default()).unwrap_or_else(|error| {
        panic!("runtime-indexed fixed-array assignment must compile: {error}")
    })
}

fn occurrences(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

#[test]
fn runtime_indexed_assignment_evaluates_target_once_before_rhs_and_reuses_bounds_guard() {
    let source = r#"
fn select_index() -> int {
    println!("selector");
    1
}

fn replacement() -> int {
    println!("rhs");
    41
}

fn main() -> int {
    let mut values = [10, 20];
    values[select_index()] = replacement();
    values[1]
}
"#;

    let llvm = compile(source);

    let selector = llvm
        .rfind("call i32 @select_index(")
        .expect("assignment target selector must execute exactly once");
    let nonnegative = selector
        + llvm[selector..]
            .find("fcmp oge double")
            .expect("assignment selector must reuse the CAP-001 nonnegative guard");
    let below_count = nonnegative
        + llvm[nonnegative..]
            .find("fcmp olt double")
            .expect("assignment selector must reuse the CAP-001 upper-bound guard");
    let branch = below_count
        + llvm[below_count..]
            .find("br i1")
            .expect("assignment selector bounds must control target address formation");
    let conversion = branch
        + llvm[branch..]
            .find("fptosi double")
            .expect("assignment selector must convert only on the safe path");
    let address = conversion
        + llvm[conversion..]
            .find("getelementptr inbounds [2 x double]")
            .expect("assignment target must use the checked typed array pointer");
    let rhs = address
        + llvm[address..]
            .find("call i32 @replacement(")
            .expect("assignment RHS must execute exactly once");
    let store = rhs
        + llvm[rhs..]
            .find("store double")
            .expect("assignment must store the exact scalar leaf");

    assert_eq!(llvm.match_indices("call i32 @select_index(").count(), 1);
    assert_eq!(llvm.match_indices("call i32 @replacement(").count(), 1);
    assert!(selector < nonnegative);
    assert!(nonnegative < below_count);
    assert!(below_count < branch);
    assert!(branch < conversion);
    assert!(conversion < address);
    assert!(address < rhs);
    assert!(rhs < store);
}

#[test]
fn runtime_assignment_composes_across_recursive_copydata_paths() {
    for (label, source, expected_guards, required_llvm) in [
        (
            "computed primitive selector",
            r#"
fn main() -> int {
    let mut values = [10, 20];
    let seed = 0;
    values[seed + 1] = 31;
    values[1]
}
"#,
            1,
            "store double",
        ),
        (
            "whole recursive struct element",
            r#"
struct Cell { value: int, trusted: bool }

fn main() -> int {
    let mut cells = [Cell { value: 10, trusted: 1 > 2 }, Cell { value: 20, trusted: 1 > 2 }];
    let index = 1;
    cells[index] = Cell { value: 41, trusted: 2 > 1 };
    cells[1].value
}
"#,
            1,
            "store %aero.struct.Cell",
        ),
        (
            "nested struct tuple and two runtime arrays",
            r#"
struct Cell { value: int, trusted: bool }
struct Grid { rows: [[(Cell, bool); 2]; 2] }

fn choose(index: int) -> int { index }
fn replacement() -> int { 43 }

fn main() -> int {
    let mut grid = Grid { rows: [[(Cell { value: 10, trusted: 1 > 2 }, 1 > 2), (Cell { value: 20, trusted: 1 > 2 }, 1 > 2)], [(Cell { value: 30, trusted: 1 > 2 }, 1 > 2), (Cell { value: 40, trusted: 1 > 2 }, 1 > 2)]] };
    let row = 1;
    let column = 0;
    grid.rows[choose(row)][choose(column)].0.value = replacement();
    grid.rows[1][0].0.value
}
"#,
            2,
            "call i32 @replacement()",
        ),
    ] {
        let llvm = compile(source);
        assert_eq!(
            occurrences(&llvm, "fcmp oge double"),
            expected_guards,
            "{label}: every runtime selector needs one nonnegative guard\n{llvm}"
        );
        assert_eq!(
            occurrences(&llvm, "fcmp olt double"),
            expected_guards,
            "{label}: every runtime selector needs one upper-bound guard\n{llvm}"
        );
        assert_eq!(
            occurrences(&llvm, "call void @llvm.trap()"),
            expected_guards,
            "{label}: every runtime selector needs one private trap edge\n{llvm}"
        );
        assert!(
            llvm.contains(required_llvm),
            "{label}: missing trusted store/lowering anchor {required_llvm:?}\n{llvm}"
        );
    }
}

#[test]
fn constant_assignment_keeps_the_static_fast_path() {
    let llvm = compile(
        r#"
fn main() -> int {
    let mut values = [10, 20];
    values[1] = 44;
    values[1]
}
"#,
    );

    assert!(!llvm.contains("@llvm.trap"));
    assert!(!llvm.contains("aero.bounds."));
    assert!(llvm.contains("getelementptr inbounds [2 x double]"));
    assert!(llvm.contains("store double"));
}

#[test]
fn non_int_runtime_assignment_selectors_fail_before_llvm() {
    for (label, declaration, selector, actual_type) in [
        ("bool", "let selector = 2 > 1;", "selector", "bool"),
        ("float", "let selector = 1.0;", "selector", "float"),
        ("string", "let selector = \"one\";", "selector", "String"),
        ("tuple", "let selector = (0, 1);", "selector", "(int, int)"),
        ("array", "let selector = [0, 1];", "selector", "[int; 2]"),
        (
            "reference",
            "let index = 1; let selector = &index;",
            "selector",
            "&int",
        ),
    ] {
        let source = format!(
            "fn main() -> int {{ let mut values = [10, 20]; {declaration} values[{selector}] = 41; values[0] }}"
        );
        let error = compile_program(&source, CompilerOptions::default())
            .expect_err("a non-int assignment selector must fail before LLVM");
        assert!(
            error.contains(&format!("expected int, actual {actual_type}")),
            "{label}: missing shared selector diagnostic: {error}"
        );
    }
}

#[test]
fn constant_and_zero_length_assignment_bounds_fail_at_compile_time() {
    for (label, source, expected) in [
        (
            "negative",
            "fn main() -> int { let mut values = [10, 20]; values[-1] = 41; 0 }",
            "nonnegative",
        ),
        (
            "equal to count",
            "fn main() -> int { let mut values = [10, 20]; values[2] = 41; 0 }",
            "outside 0..2",
        ),
        (
            "zero-length runtime selector",
            "fn main() -> int { let mut values: [int; 0] = []; let index = 0; values[index] = 41; 0 }",
            "outside 0..0",
        ),
    ] {
        let error = compile_program(source, CompilerOptions::default())
            .expect_err("out-of-bounds assignment target must fail before LLVM");
        assert!(
            error.contains(expected),
            "{label}: missing deterministic bound diagnostic {expected:?}: {error}"
        );
    }
}
