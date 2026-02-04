use compiler::{CompilerOptions, compile_program};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Instant;

fn benchmark_function_call_overhead(c: &mut Criterion) {
    let simple_function_code = r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
}
"#;

    let recursive_function_code = r#"
fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn main() {
    let result = factorial(10);
}
"#;

    let nested_calls_code = r#"
fn inner(x: i32) -> i32 {
    return x * 2;
}

fn middle(x: i32) -> i32 {
    return inner(x) + 1;
}

fn outer(x: i32) -> i32 {
    return middle(x) * 3;
}

fn main() {
    let result = outer(5);
}
"#;

    c.bench_function("simple_function_call", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(simple_function_code), options)
        })
    });

    c.bench_function("recursive_function_call", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(recursive_function_code), options)
        })
    });

    c.bench_function("nested_function_calls", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(nested_calls_code), options)
        })
    });
}

fn benchmark_function_definition_parsing(c: &mut Criterion) {
    let many_functions_code = (0..100)
        .map(|i| format!("fn func{}(x: i32) -> i32 {{ return x + {}; }}", i, i))
        .collect::<Vec<_>>()
        .join("\n")
        + "\nfn main() { let x = func0(1); }";

    c.bench_function("many_function_definitions", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(&many_functions_code), options)
        })
    });
}

criterion_group!(
    function_benchmarks,
    benchmark_function_call_overhead,
    benchmark_function_definition_parsing
);
criterion_main!(function_benchmarks);
