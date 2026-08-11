use compiler::{
    CompilerOptions, IrGenerator, compile_program, parse_with_locations,
    try_tokenize_with_locations,
};

fn minimal_static_dispatch_source() -> &'static str {
    r#"
struct Reading { value: int }

trait Score {
    fn score(&self) -> int;
}

impl Score for Reading {
    fn score(&self) -> int {
        return (*self).value;
    }
}

fn evaluate<T: Score>(reading: T) -> int {
    return reading.score();
}

fn main() -> int {
    let reading = Reading { value: 2 };
    return evaluate(reading) + 40;
}
"#
}

fn composed_static_dispatch_source() -> &'static str {
    r#"
struct Reading { value: int }
struct Offset { value: int }

trait Score {
    fn score(&self) -> int;
    fn combine(&self, left: int, right: int) -> int;
    fn observe(&self);
}

trait Bias {
    fn bias(&self) -> int;
}

impl Score for Reading {
    fn score(&self) -> int { return (*self).value; }
    fn combine(&self, left: int, right: int) -> int {
        return (*self).value + left + right;
    }
    fn observe(&self) { return; }
}

impl Bias for Reading {
    fn bias(&self) -> int { return 3; }
}

impl Score for Offset {
    fn score(&self) -> int { return (*self).value + 1; }
    fn combine(&self, left: int, right: int) -> int {
        return (*self).value + left - right;
    }
    fn observe(&self) { return; }
}

impl Bias for Offset {
    fn bias(&self) -> int { return 5; }
}

fn evaluate<T: Score + Bias>(value: T, left: int, right: int) -> int {
    value.observe();
    return value.score() + value.combine(left, right) + value.bias();
}

fn pair<T: Score, U: Bias>(left: T, right: U) -> int {
    return left.score() + right.bias();
}

fn main() -> int {
    let reading = Reading { value: 7 };
    let offset = Offset { value: 9 };
    let first = evaluate(reading, 4, 2);
    let second = evaluate(offset, 3, 1);
    let joined = pair(reading, offset);
    return first + second + joined + reading.value;
}
"#
}

fn direct_checked_admission(source: &str) -> Result<(), String> {
    let tokens = try_tokenize_with_locations(source, None).map_err(|error| error.to_string())?;
    let ast = parse_with_locations(tokens).map_err(|error| error.to_string())?;
    IrGenerator::new()
        .try_generate_ir(ast)
        .map(|_| ())
        .map_err(|error| error.to_string())
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

#[test]
fn trait_dispatch_composes_traits_methods_arguments_void_and_type_parameters() {
    let llvm = compile_program(
        composed_static_dispatch_source(),
        CompilerOptions::default(),
    )
    .expect("composed required-only trait dispatch must compile");
    direct_checked_admission(composed_static_dispatch_source())
        .expect("semantic-independent checked admission must accept the same class");

    for symbol in [
        "aero.trait.Score.for.Reading.score",
        "aero.trait.Score.for.Reading.combine",
        "aero.trait.Score.for.Reading.observe",
        "aero.trait.Bias.for.Reading.bias",
        "aero.trait.Score.for.Offset.score",
        "aero.trait.Score.for.Offset.combine",
        "aero.trait.Score.for.Offset.observe",
        "aero.trait.Bias.for.Offset.bias",
    ] {
        assert!(
            llvm.contains(symbol),
            "LLVM omitted concrete static-dispatch symbol {symbol}:\n{llvm}"
        );
    }
    assert!(
        !llvm.contains("__aero$trait_call$"),
        "compiler-private dispatch marker escaped normalization:\n{llvm}"
    );
}

#[test]
fn excluded_trait_dispatch_shapes_fail_before_trusted_llvm() {
    let cases = [
        (
            "unknown trait",
            "struct S { x: int } fn use_it<T: Missing>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "unknown trait `Missing`",
        ),
        (
            "generic trait",
            "struct S { x: int } trait Read<T> { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "nongeneric trait `Read`",
        ),
        (
            "default method",
            "struct S { x: int } trait Read { fn read(&self) -> int { return 1; } } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "does not admit default method",
        ),
        (
            "mutable receiver",
            "struct S { x: int } trait Read { fn read(&mut self) -> int; } impl Read for S { fn read(&mut self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "leading immutable &self receiver",
        ),
        (
            "missing method",
            "struct S { x: int } trait Read { fn read(&self) -> int; fn other(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read() + value.other(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "missing required method `other`",
        ),
        (
            "extra method",
            "struct S { x: int } trait Read { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } fn extra(&self) -> int { return 2; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "defines extra method `extra`",
        ),
        (
            "wrong signature",
            "struct S { x: int } trait Read { fn read(&self, add: int) -> int; } impl Read for S { fn read(&self, add: bool) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(2); } fn main() -> int { return use_it(S { x: 1 }); }",
            "does not match its exact trait signature",
        ),
        (
            "wrong method order",
            "struct S { x: int } trait Read { fn first(&self) -> int; fn second(&self) -> int; } impl Read for S { fn second(&self) -> int { return 2; } fn first(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.first() + value.second(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "method order does not match the trait declaration",
        ),
        (
            "non-struct impl target",
            "trait Read { fn read(&self) -> int; } impl Read for int { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(1); }",
            "must be a unique nongeneric recursive CopyData struct",
        ),
        (
            "inherent impl",
            "struct S { x: int } trait Read { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } impl S { fn other(&self) -> int { return 2; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "does not admit inherent impl blocks",
        ),
        (
            "unused trait declaration",
            "struct S { x: int } trait Read { fn read(&self) -> int; } trait Other { fn other(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "does not admit unused trait declaration `Other`",
        ),
        (
            "impl for unbound trait",
            "struct S { x: int } trait Read { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } impl Other for S { fn other(&self) -> int { return 2; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "does not admit impl for unbound trait `Other`",
        ),
        (
            "unused bound",
            "struct S { x: int } trait Read { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> T { return value; } fn main() -> int { return use_it(S { x: 1 }).x; }",
            "declares unused bound `T: Read`",
        ),
        (
            "duplicate bound",
            "struct S { x: int } trait Read { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read + Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "duplicate or unknown bound `T: Read`",
        ),
        (
            "ambiguous method",
            "struct S { x: int } trait Left { fn read(&self) -> int; } trait Right { fn read(&self) -> int; } impl Left for S { fn read(&self) -> int { return 1; } } impl Right for S { fn read(&self) -> int { return 2; } } fn use_it<T: Left + Right>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(S { x: 1 }); }",
            "not uniquely supplied by bounds",
        ),
        (
            "projected receiver",
            "struct Inner { x: int } struct Outer { inner: Inner } trait Read { fn read(&self) -> int; } impl Read for Inner { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.inner.read(); } fn main() -> int { return use_it(Inner { x: 1 }); }",
            "requires a direct bounded-parameter receiver",
        ),
        (
            "unsatisfied concrete bound",
            "struct S { x: int } struct Other { x: int } trait Read { fn read(&self) -> int; } impl Read for S { fn read(&self) -> int { return 1; } } fn use_it<T: Read>(value: T) -> int { return value.read(); } fn main() -> int { return use_it(Other { x: 1 }); }",
            "does not implement trait `Read`",
        ),
    ];

    for (label, source, expected) in cases {
        let error = compile_program(source, CompilerOptions::default())
            .expect_err("excluded trait-dispatch source must fail");
        assert!(
            error.contains(expected),
            "{label}: unexpected public diagnostic:\n{error}"
        );
        let direct = direct_checked_admission(source)
            .expect_err("excluded trait-dispatch source reached checked IR");
        assert!(
            direct.contains(expected),
            "{label}: semantic-independent diagnostic diverged:\n{direct}"
        );
    }
}
