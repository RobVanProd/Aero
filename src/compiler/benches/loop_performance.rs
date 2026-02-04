use compiler::{CompilerOptions, compile_program};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn benchmark_while_loop_performance(c: &mut Criterion) {
    let simple_while_code = r#"
fn main() {
    let mut i = 0;
    while i < 1000 {
        i = i + 1;
    }
}
"#;

    let nested_while_code = r#"
fn main() {
    let mut i = 0;
    while i < 100 {
        let mut j = 0;
        while j < 100 {
            j = j + 1;
        }
        i = i + 1;
    }
}
"#;

    c.bench_function("simple_while_loop", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(simple_while_code), options)
        })
    });

    c.bench_function("nested_while_loops", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(nested_while_code), options)
        })
    });
}

fn benchmark_for_loop_performance(c: &mut Criterion) {
    let simple_for_code = r#"
fn main() {
    for i in 0..1000 {
        let x = i * 2;
    }
}
"#;

    let nested_for_code = r#"
fn main() {
    for i in 0..100 {
        for j in 0..100 {
            let x = i + j;
        }
    }
}
"#;

    c.bench_function("simple_for_loop", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(simple_for_code), options)
        })
    });

    c.bench_function("nested_for_loops", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(nested_for_code), options)
        })
    });
}

fn benchmark_infinite_loop_performance(c: &mut Criterion) {
    let loop_with_break_code = r#"
fn main() {
    let mut counter = 0;
    loop {
        counter = counter + 1;
        if counter >= 1000 {
            break;
        }
    }
}
"#;

    let loop_with_continue_code = r#"
fn main() {
    let mut counter = 0;
    loop {
        counter = counter + 1;
        if counter % 2 == 0 {
            continue;
        }
        if counter >= 1000 {
            break;
        }
    }
}
"#;

    c.bench_function("loop_with_break", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(loop_with_break_code), options)
        })
    });

    c.bench_function("loop_with_continue", |b| {
        b.iter(|| {
            let options = CompilerOptions::default();
            compile_program(black_box(loop_with_continue_code), options)
        })
    });
}

criterion_group!(
    loop_benchmarks,
    benchmark_while_loop_performance,
    benchmark_for_loop_performance,
    benchmark_infinite_loop_performance
);
criterion_main!(loop_benchmarks);
