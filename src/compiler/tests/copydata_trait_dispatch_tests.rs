use compiler::{CompilerOptions, compile_program};

fn minimal_static_dispatch_source() -> &'static str {
    r#"
struct Reading { value: int }

trait Score {
    fn score(&self) -> int;
}

impl Score for Reading {
    fn score(&self) -> int {
        return 40;
    }
}

fn evaluate<T: Score>(reading: T) -> int {
    return reading.score();
}

fn main() -> int {
    let reading = Reading { value: 2 };
    return evaluate(reading) + reading.value;
}
"#
}

#[test]
fn required_copydata_trait_bound_dispatch_is_executable() {
    let llvm = compile_program(minimal_static_dispatch_source(), CompilerOptions::default())
        .expect("required-only CopyData trait dispatch must compile");

    assert!(
        llvm.contains("aero.trait.Score.for.Reading.score"),
        "LLVM omitted the concrete static-dispatch helper:\n{llvm}"
    );
    assert!(
        llvm.contains("call i32") || llvm.contains("call double"),
        "LLVM omitted the concrete trait-method call:\n{llvm}"
    );
}
