use compiler::{CompilerOptions, compile_program};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn benchmark_print_operations(c: &mut Criterion) {
    let simple_print_code = r#"
fn main() {
    print!("Hello, World!");
}
"#;

    let println_code = r#"
fn main() {
    println!("Hello, World!");
}
"#;

    let formatted_print_code = r#"
fn main() {
    let x = 42;
    let y = 3.14;
    println!("Value: {}, Float: {}", x, y);
}
"#;

    let multiple_prints_code = r#"
fn main() {
    for i in 0..100 {
        println!("Iteration: {}", i);
    }
}
"#;

    c.bench_function("simple_print", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(simple_print_code), options)
        })
    });

    c.bench_function("println_operation", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(println_code), options)
        })
    });

    c.bench_function("formatted_print", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(formatted_print_code), options)
        })
    });

    c.bench_function("multiple_print_operations", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(multiple_prints_code), options)
        })
    });
}

fn benchmark_format_string_parsing(c: &mut Criterion) {
    let complex_format_code = r#"
fn main() {
    let a = 1;
    let b = 2;
    let c = 3;
    let d = 4;
    let e = 5;
    println!("Values: {} + {} + {} + {} + {} = {}", a, b, c, d, e, a + b + c + d + e);
}
"#;

    let nested_format_code = r#"
fn main() {
    for i in 0..10 {
        for j in 0..10 {
            println!("Matrix[{}][{}] = {}", i, j, i * j);
        }
    }
}
"#;

    c.bench_function("complex_format_string", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(complex_format_code), options)
        })
    });

    c.bench_function("nested_format_operations", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(nested_format_code), options)
        })
    });
}

criterion_group!(
    io_benchmarks,
    benchmark_print_operations,
    benchmark_format_string_parsing
);
criterion_main!(io_benchmarks);
