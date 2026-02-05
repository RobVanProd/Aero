#!/bin/bash

# Test script for Aero compiler
# This script tests the basic functionality of the Aero compiler

set -e  # Exit on any error

echo "=== Aero Compiler Test Suite ==="
echo

# Load rustup env if present (so `cargo` is on PATH in non-interactive shells)
if [[ -f "$HOME/.cargo/env" ]]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
fi

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

# Run tests in a temporary directory so `aero run` doesn't leave build artifacts in
# `examples/` (it currently emits .ll/.o and a binary next to the source file).
ROOT_DIR="$(pwd)"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT
cp "$ROOT_DIR"/examples/return15.aero "$ROOT_DIR"/examples/variables.aero "$ROOT_DIR"/examples/mixed.aero "$ROOT_DIR"/examples/float_ops.aero "$TMPDIR/"

# IMPORTANT: `aero run` currently constructs the executable path as `./<exe_file>`.
# If the input path is absolute, it will generate an invalid `./<absolute/path>`.
# So we `cd` into the temp dir and run using relative paths.
pushd "$TMPDIR" >/dev/null

# Test 1: return15.aero
echo "Test 1: Testing return15.aero (should exit with code 15)"
# This test expects a non-zero exit code, so we must temporarily disable `set -e`.
set +e
"$ROOT_DIR"/src/compiler/target/release/aero run return15.aero
exit_code=$?
set -e
if [ $exit_code -eq 15 ]; then
    echo "✓ Test 1 passed: exit code $exit_code"
else
    echo "✗ Test 1 failed: expected exit code 15, got $exit_code"
    exit 1
fi
echo

# Helper: run a program expected to exit non-zero without tripping `set -e`.
run_expect_exit() {
  local expected="$1"
  shift
  set +e
  "$@"
  local got=$?
  set -e
  if [ "$got" -eq "$expected" ]; then
    echo "✓ passed: exit code $got"
  else
    echo "✗ failed: expected exit code $expected, got $got"
    exit 1
  fi
}

# Test 2: variables.aero
echo "Test 2: Testing variables.aero (should exit with code 6)"
run_expect_exit 6 "$ROOT_DIR"/src/compiler/target/release/aero run variables.aero
echo

# Test 3: mixed.aero
echo "Test 3: Testing mixed.aero (should exit with code 7)"
run_expect_exit 7 "$ROOT_DIR"/src/compiler/target/release/aero run mixed.aero
echo

# Test 4: float_ops.aero
echo "Test 4: Testing float_ops.aero (should exit with code 7)"
run_expect_exit 7 "$ROOT_DIR"/src/compiler/target/release/aero run float_ops.aero
echo

popd >/dev/null

echo "=== All tests passed! ==="
echo "The Aero compiler is working correctly."