#!/bin/bash

# Test script for Aero compiler
# This script tests the basic functionality of the Aero compiler

set -e  # Exit on any error

echo "=== Aero Compiler Test Suite ==="
echo

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "Error: cargo is not installed. Please install Rust first."
    exit 1
fi

# Check if llc is available
if ! command -v llc &> /dev/null; then
    echo "Error: llc is not installed. Please install LLVM tools."
    exit 1
fi

# Check if clang is available
if ! command -v clang &> /dev/null; then
    echo "Error: clang is not installed. Please install clang."
    exit 1
fi

echo "✓ Prerequisites check passed"
echo

# Build the compiler
echo "Building Aero compiler..."
cd src/compiler
cargo build --release
cd ../..
echo "✓ Compiler built successfully"
echo

# Test 1: return15.aero
echo "Test 1: Testing return15.aero (should exit with code 15)"
./src/compiler/target/release/aero run examples/return15.aero
exit_code=$?
if [ $exit_code -eq 15 ]; then
    echo "✓ Test 1 passed: exit code $exit_code"
else
    echo "✗ Test 1 failed: expected exit code 15, got $exit_code"
    exit 1
fi
echo

# Test 2: variables.aero
echo "Test 2: Testing variables.aero (should exit with code 6)"
./src/compiler/target/release/aero run examples/variables.aero
exit_code=$?
if [ $exit_code -eq 6 ]; then
    echo "✓ Test 2 passed: exit code $exit_code"
else
    echo "✗ Test 2 failed: expected exit code 6, got $exit_code"
    exit 1
fi
echo

# Test 3: mixed.aero
echo "Test 3: Testing mixed.aero (should exit with code 7)"
./src/compiler/target/release/aero run examples/mixed.aero
exit_code=$?
if [ $exit_code -eq 7 ]; then
    echo "✓ Test 3 passed: exit code $exit_code"
else
    echo "✗ Test 3 failed: expected exit code 7, got $exit_code"
    exit 1
fi
echo

# Test 4: float_ops.aero
echo "Test 4: Testing float_ops.aero (should exit with code 7)"
./src/compiler/target/release/aero run examples/float_ops.aero
exit_code=$?
if [ $exit_code -eq 7 ]; then
    echo "✓ Test 4 passed: exit code $exit_code"
else
    echo "✗ Test 4 failed: expected exit code 7, got $exit_code"
    exit 1
fi
echo

echo "=== All tests passed! ==="
echo "The Aero compiler is working correctly."