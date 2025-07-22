#!/bin/bash

# Comprehensive benchmark runner for Aero Phase 3 features
# This script runs all performance benchmarks and generates reports

set -e

echo "=== Aero Phase 3 Performance Benchmarks ==="
echo "Starting benchmark suite..."
echo

# Change to the compiler directory
cd "$(dirname "$0")/../src/compiler"

# Ensure we're in release mode for accurate benchmarks
export CARGO_PROFILE_RELEASE_DEBUG=true

# Function to run a specific benchmark
run_benchmark() {
    local bench_name=$1
    local description=$2
    
    echo "Running $description..."
    echo "----------------------------------------"
    
    # Run the benchmark and capture output
    if cargo bench --bench "$bench_name" -- --output-format pretty; then
        echo "✓ $description completed successfully"
    else
        echo "✗ $description failed"
        return 1
    fi
    
    echo
}

# Create benchmarks directory if it doesn't exist
mkdir -p ../../benchmarks/results

# Run all benchmarks
echo "1. Function Call Overhead Benchmarks"
run_benchmark "function_call_overhead" "Function Call Performance Tests"

echo "2. Loop Performance Benchmarks"
run_benchmark "loop_performance" "Loop Construct Performance Tests"

echo "3. I/O Operations Benchmarks"
run_benchmark "io_operations" "I/O Operation Performance Tests"

echo "4. Compilation Speed Benchmarks"
run_benchmark "compilation_speed" "Compilation Performance Tests"

echo "5. Performance Regression Benchmarks"
run_benchmark "performance_regression" "Regression Testing Against Baseline"

# Generate summary report
echo "=== Benchmark Summary ==="
echo "All benchmarks completed successfully!"
echo
echo "Results have been saved to target/criterion/"
echo "You can view detailed HTML reports by opening:"
echo "  target/criterion/report/index.html"
echo
echo "To compare results over time, run this script regularly"
echo "and use 'cargo bench' with specific benchmark names."
echo

# Optional: Generate a simple performance report
if command -v python3 &> /dev/null; then
    echo "Generating performance summary..."
    python3 << 'EOF'
import os
import json
from pathlib import Path

criterion_dir = Path("target/criterion")
if criterion_dir.exists():
    print("\n=== Performance Summary ===")
    
    # Look for benchmark results
    for bench_dir in criterion_dir.iterdir():
        if bench_dir.is_dir() and bench_dir.name != "report":
            estimates_file = bench_dir / "base" / "estimates.json"
            if estimates_file.exists():
                try:
                    with open(estimates_file) as f:
                        data = json.load(f)
                        mean_time = data.get("mean", {}).get("point_estimate", 0)
                        # Convert from nanoseconds to milliseconds
                        mean_ms = mean_time / 1_000_000
                        print(f"{bench_dir.name}: {mean_ms:.3f} ms")
                except:
                    pass
else:
    print("No criterion results found. Run benchmarks first.")
EOF
fi

echo
echo "Benchmark suite completed!"