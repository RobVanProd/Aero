use compiler::{CompilerOptions, compile_program};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn benchmark_compilation_speed(c: &mut Criterion) {
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

    c.bench_function("small_program_compilation", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(small_program), options)
        })
    });

    c.bench_function("medium_program_compilation", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(medium_program), options)
        })
    });

    c.bench_function("large_program_compilation", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(&large_program), options)
        })
    });
}

fn benchmark_lexer_performance(c: &mut Criterion) {
    let token_heavy_code = {
        let mut code = String::new();
        code.push_str("fn main() {\n");

        // Generate code with many tokens
        for i in 0..1000 {
            code.push_str(&format!(
                "    let var{} = {} + {} * {} - {} / {};\n",
                i,
                i,
                i + 1,
                i + 2,
                i + 3,
                i + 4
            ));
        }

        code.push_str("}\n");
        code
    };

    c.bench_function("token_heavy_lexing", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(&token_heavy_code), options)
        })
    });
}

fn benchmark_parser_performance(c: &mut Criterion) {
    let complex_expressions_code = {
        let mut code = String::new();
        code.push_str("fn main() {\n");

        // Generate complex nested expressions
        for i in 0..100 {
            code.push_str(&format!(
                "    let result{} = ((({} + {}) * {}) - {}) / ({} + {});\n",
                i,
                i,
                i + 1,
                i + 2,
                i + 3,
                i + 4,
                i + 5
            ));
        }

        code.push_str("}\n");
        code
    };

    c.bench_function("complex_expressions_parsing", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(&complex_expressions_code), options)
        })
    });
}

criterion_group!(
    compilation_benchmarks,
    benchmark_compilation_speed,
    benchmark_lexer_performance,
    benchmark_parser_performance
);
criterion_main!(compilation_benchmarks);
