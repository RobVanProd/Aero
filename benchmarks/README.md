# Aero Phase 3 Performance Benchmarks

This directory contains comprehensive performance benchmarks for Aero Phase 3 core language features, including function calls, control flow, I/O operations, and compilation speed.

## Benchmark Categories

### 1. Function Call Overhead Benchmarks
- **Simple function calls**: Measures basic function call and return overhead
- **Recursive functions**: Tests performance of recursive calls (fibonacci, factorial)
- **Nested function calls**: Evaluates performance of functions calling other functions
- **Function definition parsing**: Measures compilation time for many function definitions

**Test Files:**
- `aero/function_performance.aero` - Runtime function performance tests
- Rust benchmarks in `src/compiler/benches/function_call_overhead.rs`

### 2. Loop Performance Benchmarks
- **While loops**: Simple and nested while loop performance
- **For loops**: Range-based for loop performance
- **Infinite loops**: Loop with break/continue performance
- **Nested control flow**: Complex nested loop and conditional performance

**Test Files:**
- `aero/loop_performance.aero` - Runtime loop performance tests
- Rust benchmarks in `src/compiler/benches/loop_performance.rs`

### 3. I/O Operations Benchmarks
- **Simple print operations**: Basic print! and println! performance
- **Formatted printing**: Format string parsing and argument substitution
- **Complex formatting**: Multiple arguments and nested expressions
- **High-volume I/O**: Performance under many I/O operations

**Test Files:**
- `aero/io_performance.aero` - Runtime I/O performance tests
- Rust benchmarks in `src/compiler/benches/io_operations.rs`

### 4. Compilation Speed Benchmarks
- **Small programs**: Basic compilation speed baseline
- **Medium programs**: Moderate complexity compilation
- **Large programs**: Many functions and complex structures
- **Lexer performance**: Token-heavy code parsing
- **Parser performance**: Complex expression parsing

**Test Files:**
- Various example programs in `../examples/`
- Rust benchmarks in `src/compiler/benches/compilation_speed.rs`

### 5. Performance Regression Tests
- **Baseline comparisons**: Phase 1/2 vs Phase 3 performance
- **Memory usage**: Compilation memory overhead
- **Error handling**: Performance impact of error checking
- **Feature overhead**: Cost of new language features

**Test Files:**
- Generated test programs for regression testing
- Rust benchmarks in `src/compiler/benches/performance_regression.rs`

## Running Benchmarks

### Quick Start
```bash
# On Windows
scripts\run_performance_benchmarks.bat

# On Unix/Linux/macOS
scripts/run_performance_benchmarks.sh
```

### Manual Execution

#### Python Benchmarks (Recommended)
```bash
# Install Python 3.6+ if not already installed
python3 benchmarks/performance_benchmark.py
```

#### Rust Criterion Benchmarks
```bash
cd src/compiler

# Run all benchmarks
cargo bench

# Run specific benchmark categories
cargo bench --bench function_call_overhead
cargo bench --bench loop_performance
cargo bench --bench io_operations
cargo bench --bench compilation_speed
cargo bench --bench performance_regression

# Run lexer-only benchmarks (works even with compiler errors)
cargo bench --bench lexer_only_benchmarks
```

## Benchmark Results

Results are saved in the `results/` directory with timestamps:
- `performance_results_<timestamp>.json` - Python benchmark results
- `target/criterion/` - Rust criterion benchmark results and HTML reports

### Interpreting Results

#### Python Benchmark Results
```json
{
  "timestamp": 1234567890,
  "benchmarks": {
    "function_performance": {
      "function_performance.aero": {
        "compilation": {
          "mean": 0.1234,
          "median": 0.1200,
          "min": 0.1100,
          "max": 0.1400,
          "std_dev": 0.0050,
          "iterations": 10
        },
        "description": "Function call overhead and recursion performance"
      }
    }
  }
}
```

#### Rust Criterion Results
- **Mean time**: Average execution time
- **Confidence intervals**: Statistical confidence in measurements
- **Throughput**: Operations per second where applicable
- **Comparison**: Performance changes over time

## Performance Targets

### Phase 3 Performance Goals
- **Function call overhead**: < 10ns per call
- **Loop iteration**: < 1ns per iteration
- **I/O operations**: < 1μs per print statement
- **Compilation speed**: < 100ms for small programs
- **Memory usage**: < 50MB for typical programs

### Regression Thresholds
- **No more than 10% slowdown** compared to Phase 2 baseline
- **Memory usage increase < 20%** for equivalent functionality
- **Error handling overhead < 5%** for successful compilations

## Benchmark Infrastructure

### Python Benchmark Framework
- **Cross-platform**: Works on Windows, macOS, and Linux
- **Comprehensive**: Tests both compilation and execution
- **Flexible**: Easy to add new benchmark categories
- **Detailed reporting**: JSON output with statistical analysis

### Rust Criterion Framework
- **Statistical rigor**: Proper statistical analysis of performance
- **HTML reports**: Detailed visualizations and comparisons
- **Regression detection**: Automatic detection of performance changes
- **Micro-benchmarks**: Precise measurement of individual components

## Adding New Benchmarks

### Python Benchmarks
1. Add new test programs to `aero/` directory
2. Extend `performance_benchmark.py` with new benchmark methods
3. Update benchmark categories in `run_all_benchmarks()`

### Rust Benchmarks
1. Create new benchmark file in `src/compiler/benches/`
2. Add benchmark entry to `Cargo.toml`
3. Use criterion macros for statistical measurement

### Example New Benchmark
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_new_feature(c: &mut Criterion) {
    let test_code = r#"
    // Your test code here
    "#;

    c.bench_function("new_feature_test", |b| {
        b.iter(|| {
            // Benchmark code here
            black_box(test_code)
        })
    });
}

criterion_group!(new_benchmarks, benchmark_new_feature);
criterion_main!(new_benchmarks);
```

## Troubleshooting

### Common Issues
1. **Compiler build errors**: Use Python benchmarks or lexer-only Rust benchmarks
2. **Python not found**: Install Python 3.6+ and add to PATH
3. **Cargo not found**: Install Rust toolchain
4. **Permission errors**: Ensure scripts are executable

### Performance Analysis
- Use `target/criterion/report/index.html` for detailed Rust benchmark analysis
- Compare results over time to detect regressions
- Focus on mean times and confidence intervals
- Consider system load when interpreting results

## Continuous Integration

These benchmarks can be integrated into CI/CD pipelines:
- Run on every commit to detect regressions
- Compare against baseline performance
- Generate performance reports for releases
- Alert on significant performance changes

## Contributing

When adding new language features:
1. Add corresponding performance benchmarks
2. Establish performance baselines
3. Monitor for regressions
4. Update performance targets as needed