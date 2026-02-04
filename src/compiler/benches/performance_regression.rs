use compiler::{CompilerOptions, compile_program};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use std::time::Instant;

/// Performance regression tests to ensure Phase 3 features don't degrade performance
fn benchmark_baseline_performance(c: &mut Criterion) {
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

    c.bench_function("baseline_arithmetic", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(arithmetic_code), options)
        })
    });

    c.bench_function("baseline_variables", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(variable_code), options)
        })
    });
}

fn benchmark_phase3_vs_baseline(c: &mut Criterion) {
    // Equivalent functionality: baseline vs Phase 3
    let baseline_repeated_code = r#"
fn main() {
    let result1 = 5 + 3;
    let result2 = 5 + 3;
    let result3 = 5 + 3;
    let result4 = 5 + 3;
    let result5 = 5 + 3;
}
"#;

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

    c.bench_function("baseline_repeated_operations", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(baseline_repeated_code), options)
        })
    });

    c.bench_function("phase3_function_calls", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(phase3_function_code), options)
        })
    });
}

fn benchmark_memory_usage_regression(c: &mut Criterion) {
    // Test memory usage doesn't regress with new features
    let memory_intensive_code = {
        let mut code = String::new();

        // Many function definitions
        for i in 0..200 {
            code.push_str(&format!("fn func{}() -> i32 {{ return {}; }}\n", i, i));
        }

        // Complex control flow
        code.push_str("fn main() {\n");
        code.push_str("    for i in 0..100 {\n");
        code.push_str("        if i % 2 == 0 {\n");
        code.push_str("            let mut j = 0;\n");
        code.push_str("            while j < 10 {\n");
        code.push_str("                j = j + 1;\n");
        code.push_str("            }\n");
        code.push_str("        }\n");
        code.push_str("    }\n");
        code.push_str("}\n");

        code
    };

    c.bench_function("memory_intensive_compilation", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(&memory_intensive_code), options)
        })
    });
}

fn benchmark_error_handling_performance(c: &mut Criterion) {
    // Test that error handling doesn't significantly impact performance
    let error_prone_code = r#"
fn main() {
    // This should compile successfully but test error handling paths
    let x = 5;
    if x > 0 {
        println!("Positive: {}", x);
    } else {
        println!("Non-positive: {}", x);
    }
}
"#;

    let syntax_error_code = r#"
fn main() {
    let x = 5
    // Missing semicolon - should trigger error handling
}
"#;

    c.bench_function("successful_compilation_with_error_checks", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(error_prone_code), options)
        })
    });

    c.bench_function("error_handling_performance", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            let _ = compile_program(black_box(syntax_error_code), options);
            // Ignore the error result, we're just measuring performance
        })
    });
}

criterion_group!(
    regression_benchmarks,
    benchmark_baseline_performance,
    benchmark_phase3_vs_baseline,
    benchmark_memory_usage_regression,
    benchmark_error_handling_performance
);
criterion_main!(regression_benchmarks);
