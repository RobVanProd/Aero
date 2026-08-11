use compiler::{CompilerOptions, compile_program};

fn compile(source: &str) -> String {
    compile_program(source, CompilerOptions::default())
        .unwrap_or_else(|error| panic!("dynamic fixed-array read must compile: {error}"))
}

fn occurrences(text: &str, needle: &str) -> usize {
    text.match_indices(needle).count()
}

#[test]
fn dynamic_fixed_array_index_is_guarded_before_inbounds_address_formation() {
    let llvm = compile(
        r#"
fn main() -> int {
    let values = [10, 20];
    let index = 1;
    values[index]
}
"#,
    );

    let nonnegative = llvm
        .find("fcmp oge double")
        .expect("dynamic index must be checked for a nonnegative ordered value");
    let below_count = llvm
        .find("fcmp olt double")
        .expect("dynamic index must be checked against the retained array count");
    let branch = llvm
        .find("br i1")
        .expect("dynamic index bounds predicate must control the access");
    let trap = llvm
        .find("call void @llvm.trap()")
        .expect("out-of-bounds dynamic index must trap before access");
    let safe = llvm
        .rfind("aero.bounds.safe")
        .expect("in-range dynamic indexing must continue in an explicit safe block");
    let conversion = llvm
        .find("fptosi double")
        .expect("the guarded numeric index must be converted for address formation");
    let address = llvm
        .rfind("getelementptr inbounds [2 x double]")
        .expect("the admitted fixed-array read must retain typed address formation");

    assert!(llvm.contains("declare void @llvm.trap()"));
    assert!(nonnegative < below_count);
    assert!(below_count < branch);
    assert!(branch < trap);
    assert!(trap < safe);
    assert!(safe < conversion);
    assert!(conversion < address);
}

#[test]
fn constant_fixed_array_index_does_not_add_a_runtime_trap_contract() {
    let llvm = compile(
        r#"
fn main() -> int {
    let values = [10, 20];
    values[1]
}
"#,
    );

    assert!(!llvm.contains("@llvm.trap"));
    assert!(!llvm.contains("aero.bounds."));
    assert!(llvm.contains("getelementptr inbounds [2 x double]"));
}

#[test]
fn every_dynamic_fixed_array_read_context_uses_the_shared_guard() {
    for (label, source, expected_guards) in [
        (
            "primitive local and computed index",
            r#"
fn main() -> int {
    let values = [10, 20];
    let seed = 0;
    let index = seed + 1;
    values[index]
}
"#,
            1,
        ),
        (
            "recursive CopyData through a struct field",
            r#"
struct Cell { value: int }
struct Table { cells: [Cell; 2] }

fn main() -> int {
    let table = Table { cells: [Cell { value: 31 }, Cell { value: 32 }] };
    let index = 1;
    table.cells[index].value
}
"#,
            1,
        ),
        (
            "nested fixed arrays",
            r#"
fn main() -> int {
    let values = [[41, 42], [43, 44]];
    let row = 1;
    let column = 0;
    values[row][column]
}
"#,
            2,
        ),
        (
            "fixed-array function transport",
            r#"
fn select(values: [int; 2], index: int) -> int {
    values[index]
}

fn main() -> int {
    let index = 1;
    select([51, 52], index)
}
"#,
            1,
        ),
    ] {
        let llvm = compile(source);
        assert_eq!(
            occurrences(&llvm, "declare void @llvm.trap()"),
            1,
            "{label}: trap declaration must be module-wide and singular"
        );
        for anchor in [
            "fcmp oge double",
            "fcmp olt double",
            "call void @llvm.trap()",
            "aero.bounds.safe.",
        ] {
            assert_eq!(
                occurrences(&llvm, anchor),
                if anchor == "aero.bounds.safe." {
                    expected_guards * 2
                } else {
                    expected_guards
                },
                "{label}: wrong number of shared guard anchors {anchor:?}\n{llvm}"
            );
        }
    }
}

#[test]
fn constant_out_of_bounds_indexes_keep_the_compile_time_diagnostic() {
    for (label, index) in [
        ("negative", "-1"),
        ("equal to count", "2"),
        ("above count", "3"),
    ] {
        let source = format!("fn main() -> int {{ let values = [10, 20]; values[{index}] }}");
        let error = compile_program(&source, CompilerOptions::default())
            .expect_err("constant out-of-bounds fixed-array index must reject");
        assert!(
            error.contains("outside 0..2"),
            "{label}: missing deterministic constant-bound diagnostic: {error}"
        );
    }
}
