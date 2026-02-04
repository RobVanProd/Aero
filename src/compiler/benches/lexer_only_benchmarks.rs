use compiler::tokenize;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn benchmark_function_call_tokenization(c: &mut Criterion) {
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

    c.bench_function("tokenize_simple_function", |b| {
        b.iter(|| tokenize(black_box(simple_function_code)))
    });

    c.bench_function("tokenize_recursive_function", |b| {
        b.iter(|| tokenize(black_box(recursive_function_code)))
    });

    c.bench_function("tokenize_nested_functions", |b| {
        b.iter(|| tokenize(black_box(nested_calls_code)))
    });
}

fn benchmark_loop_tokenization(c: &mut Criterion) {
    let while_loop_code = r#"
fn main() {
    let mut i = 0;
    while i < 1000 {
        i = i + 1;
    }
}
"#;

    let for_loop_code = r#"
fn main() {
    for i in 0..1000 {
        let x = i * 2;
    }
}
"#;

    let nested_loops_code = r#"
fn main() {
    for i in 0..100 {
        let mut j = 0;
        while j < 100 {
            if j % 2 == 0 {
                continue;
            }
            j = j + 1;
        }
    }
}
"#;

    c.bench_function("tokenize_while_loop", |b| {
        b.iter(|| tokenize(black_box(while_loop_code)))
    });

    c.bench_function("tokenize_for_loop", |b| {
        b.iter(|| tokenize(black_box(for_loop_code)))
    });

    c.bench_function("tokenize_nested_loops", |b| {
        b.iter(|| tokenize(black_box(nested_loops_code)))
    });
}

fn benchmark_io_tokenization(c: &mut Criterion) {
    let simple_print_code = r#"
fn main() {
    print!("Hello, World!");
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

    c.bench_function("tokenize_simple_io", |b| {
        b.iter(|| tokenize(black_box(simple_print_code)))
    });

    c.bench_function("tokenize_formatted_io", |b| {
        b.iter(|| tokenize(black_box(formatted_print_code)))
    });

    c.bench_function("tokenize_multiple_io", |b| {
        b.iter(|| tokenize(black_box(multiple_prints_code)))
    });
}

fn benchmark_compilation_speed_tokenization(c: &mut Criterion) {
    let small_program = r#"
fn main() {
    let x = 5;
    let y = 10;
    let result = x + y;
    println!("Result: {}", result);
}
"#;

    let medium_program = r#"
fn fibonacci(n: i32) -> i32 {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

fn main() {
    for i in 0..20 {
        let fib = fibonacci(i);
        let fact = factorial(i);
        println!("fib({}) = {}, fact({}) = {}", i, fib, i, fact);
    }
}
"#;

    // Generate a large program with many functions
    let large_program = {
        let mut code = String::new();

        // Generate 50 simple functions
        for i in 0..50 {
            code.push_str(&format!(
                "fn func{}(x: i32) -> i32 {{\n    return x + {};\n}}\n\n",
                i, i
            ));
        }

        // Generate main function that calls all of them
        code.push_str("fn main() {\n");
        for i in 0..50 {
            code.push_str(&format!("    let result{} = func{}({});\n", i, i, i));
        }
        code.push_str("    println!(\"All functions executed\");\n");
        code.push_str("}\n");

        code
    };

    c.bench_function("tokenize_small_program", |b| {
        b.iter(|| tokenize(black_box(small_program)))
    });

    c.bench_function("tokenize_medium_program", |b| {
        b.iter(|| tokenize(black_box(medium_program)))
    });

    c.bench_function("tokenize_large_program", |b| {
        b.iter(|| tokenize(black_box(&large_program)))
    });
}

fn benchmark_performance_regression_tokenization(c: &mut Criterion) {
    // Simple arithmetic (Phase 1 baseline)
    let arithmetic_code = r#"
fn main() {
    let a = 10;
    let b = 20;
    let result = a + b * 2 - 5;
}
"#;

    // Variable assignments (Phase 2 baseline)
    let variable_code = r#"
fn main() {
    let x = 5;
    let y = x + 10;
    let z = y * 2;
}
"#;

    // Phase 3 function code
    let phase3_function_code = r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result1 = add(5, 3);
    let result2 = add(5, 3);
    let result3 = add(5, 3);
    let result4 = add(5, 3);
    let result5 = add(5, 3);
}
"#;

    c.bench_function("tokenize_baseline_arithmetic", |b| {
        b.iter(|| tokenize(black_box(arithmetic_code)))
    });

    c.bench_function("tokenize_baseline_variables", |b| {
        b.iter(|| tokenize(black_box(variable_code)))
    });

    c.bench_function("tokenize_phase3_functions", |b| {
        b.iter(|| tokenize(black_box(phase3_function_code)))
    });
}

criterion_group!(
    lexer_benchmarks,
    benchmark_function_call_tokenization,
    benchmark_loop_tokenization,
    benchmark_io_tokenization,
    benchmark_compilation_speed_tokenization,
    benchmark_performance_regression_tokenization
);
criterion_main!(lexer_benchmarks);
