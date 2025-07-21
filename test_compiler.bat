@echo off
REM Test script for Aero compiler (Windows version)
REM This script tests the basic functionality of the Aero compiler

echo === Aero Compiler Test Suite ===
echo.

REM Check if cargo is available
cargo --version >nul 2>&1
if errorlevel 1 (
    echo Error: cargo is not installed. Please install Rust first.
    exit /b 1
)

REM Check if llc is available
llc --version >nul 2>&1
if errorlevel 1 (
    echo Error: llc is not installed. Please install LLVM tools.
    exit /b 1
)

REM Check if clang is available
clang --version >nul 2>&1
if errorlevel 1 (
    echo Error: clang is not installed. Please install clang.
    exit /b 1
)

echo ✓ Prerequisites check passed
echo.

REM Build the compiler
echo Building Aero compiler...
cd src\compiler
cargo build --release
if errorlevel 1 (
    echo Error: Failed to build compiler
    exit /b 1
)
cd ..\..
echo ✓ Compiler built successfully
echo.

REM Test 1: return15.aero
echo Test 1: Testing return15.aero (should exit with code 15)
src\compiler\target\release\aero.exe run examples\return15.aero
if %errorlevel% equ 15 (
    echo ✓ Test 1 passed: exit code %errorlevel%
) else (
    echo ✗ Test 1 failed: expected exit code 15, got %errorlevel%
    exit /b 1
)
echo.

REM Test 2: variables.aero
echo Test 2: Testing variables.aero (should exit with code 6)
src\compiler\target\release\aero.exe run examples\variables.aero
if %errorlevel% equ 6 (
    echo ✓ Test 2 passed: exit code %errorlevel%
) else (
    echo ✗ Test 2 failed: expected exit code 6, got %errorlevel%
    exit /b 1
)
echo.

REM Test 3: mixed.aero
echo Test 3: Testing mixed.aero (should exit with code 7)
src\compiler\target\release\aero.exe run examples\mixed.aero
if %errorlevel% equ 7 (
    echo ✓ Test 3 passed: exit code %errorlevel%
) else (
    echo ✗ Test 3 failed: expected exit code 7, got %errorlevel%
    exit /b 1
)
echo.

REM Test 4: float_ops.aero
echo Test 4: Testing float_ops.aero (should exit with code 7)
src\compiler\target\release\aero.exe run examples\float_ops.aero
if %errorlevel% equ 7 (
    echo ✓ Test 4 passed: exit code %errorlevel%
) else (
    echo ✗ Test 4 failed: expected exit code 7, got %errorlevel%
    exit /b 1
)
echo.

echo === All tests passed! ===
echo The Aero compiler is working correctly.