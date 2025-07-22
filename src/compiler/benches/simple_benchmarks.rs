use criterion::{black_box, criterion_group, criterion_main, Criterion};
use compiler::{tokenize, parse, SemanticAnalyzer, IrGenerator, generate_code};

fn benchmark_lexer_performance(c: &mut Criterion) {
    let simple_code = r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
    println!("Result: {}", result);
}
"#;

    let complex_code = {
        let mut code = String::new();
        
        // Generate many functions
        for i in 0..100 {
            code.push_str(&format!(
                "fn func{}(x: i32) -> i32 {{ return x + {}; }}\n",
                i, i
            ));
        }
        
        // Generate main function
        code.push_str("fn main() {\n");
        for i in 0..100 {
            code.push_str(&format!("    let result{} = func{}({});\n", i, i, i));
        }
        code.push_str("}\n");
        
        code
    };

    c.bench_function("lexer_simple_code", |b| {
        b.iter(|| {
            tokenize(black_box(simple_code))
        })
    });

    c.bench_function("lexer_complex_code", |b| {
        b.iter(|| {
            tokenize(black_box(&complex_code))
        })
    });
}

fn benchmark_parser_performance(c: &mut Criterion) {
    let simple_code = r#"
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
}
"#;

    let control_flow_code = r#"
fn main() {
    for i in 0..10 {
        if i % 2 == 0 {
            println!("Even: {}", i);
        } else {
            println!("Odd: {}", i);
        }
    }
    
    let mut j = 0;
    while j < 5 {
        j = j + 1;
    }
    
    loop {
        if j > 10 {
            break;
        }
        j = j + 1;
    }
}
"#;

    c.bench_function("parser_simple_functions", |b| {
        b.iter(|| {
            let tokens = tokenize(black_box(simple_code));
            parse(black_box(tokens))
        })
    });

    c.bench_function("parser_control_flow", |b| {
        b.iter(|| {
            let tokens = tokenize(black_box(control_flow_code));
            parse(black_box(tokens))
        })
    });
}

fn benchmark_tokenization_patterns(c: &mut Criterion) {
    let function_heavy_code = {
        let mut code = String::new();
        for i in 0..50 {
            code.push_str(&format!(
                "fn fibonacci{}(n: i32) -> i32 {{\n",
                i
            ));
            code.push_str("    if n <= 1 {\n");
            code.push_str("        return n;\n");
            code.push_str("    } else {\n");
            code.push_str(&format!("        return fibonacci{}(n - 1) + fibonacci{}(n - 2);\n", i, i));
            code.push_str("    }\n");
            code.push_str("}\n\n");
        }
        code
    };

    let io_heavy_code = {
        let mut code = String::new();
        code.push_str("fn main() {\n");
        for i in 0..100 {
            code.push_str(&format!(
                "    println!(\"Iteration {}: value = {{}}\", {});\n",
                i, i * 2
            ));
        }
        code.push_str("}\n");
        code
    };

    let loop_heavy_code = {
        let mut code = String::new();
        code.push_str("fn main() {\n");
        for i in 0..20 {
            code.push_str(&format!("    for i{} in 0..{} {{\n", i, i + 10));
            code.push_str(&format!("        while i{} < {} {{\n", i, i + 5));
            code.push_str(&format!("            if i{} % 2 == 0 {{\n", i));
            code.push_str("                continue;\n");
            code.push_str("            }\n");
            code.push_str(&format!("            i{} = i{} + 1;\n", i, i));
            code.push_str("        }\n");
            code.push_str("    }\n");
        }
        code.push_str("}\n");
        code
    };

    c.bench_function("tokenize_function_heavy", |b| {
        b.iter(|| {
            tokenize(black_box(&function_heavy_code))
        })
    });

    c.bench_function("tokenize_io_heavy", |b| {
        b.iter(|| {
            tokenize(black_box(&io_heavy_code))
        })
    });

    c.bench_function("tokenize_loop_heavy", |b| {
        b.iter(|| {
            tokenize(black_box(&loop_heavy_code))
        })
    });
}

criterion_group!(
    simple_benchmarks,
    benchmark_lexer_performance,
    benchmark_parser_performance,
    benchmark_tokenization_patterns
);
criterion_main!(simple_benchmarks);