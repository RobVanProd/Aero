#!/bin/bash

# Performance Benchmark Runner for Aero Phase 3 Features
# This script runs comprehensive performance benchmarks

set -e

echo "=== Aero Phase 3 Performance Benchmarks ==="
echo

# Change to the Aero root directory
cd "$(dirname "$0")/.."

# Check if Python is available
if ! command -v python3 &> /dev/null; then
    echo "Error: Python 3 is not installed or not in PATH"
    echo "Please install Python 3.6+ to run performance benchmarks"
    exit 1
fi

# Create results directory if it doesn't exist
mkdir -p benchmarks/results

# Run the Python benchmark script
echo "Running performance benchmarks..."
echo
python3 benchmarks/performance_benchmark.py

if [ $? -ne 0 ]; then
    echo
    echo "Error: Benchmark execution failed"
    exit 1
fi

echo
echo "Performance benchmarks completed successfully!"
echo "Results are saved in benchmarks/results/"
echo

# Try to run Rust benchmarks if the compiler builds successfully
echo "Attempting to run Rust criterion benchmarks..."
cd src/compiler

# Try to build first
if cargo build --release &> /dev/null; then
    echo "Compiler builds successfully, running Rust benchmarks..."
    
    # Run lexer benchmarks (these should work)
    if cargo bench --bench lexer_only_benchmarks 2>/dev/null; then
        echo "Lexer benchmarks completed successfully"
    else
        echo "Lexer benchmarks failed or not available"
    fi
else
    echo "Compiler has build errors, skipping Rust benchmarks"
    echo "Focus on Python benchmarks for now"
fi

cd ../..

echo
echo "All available benchmarks completed!"