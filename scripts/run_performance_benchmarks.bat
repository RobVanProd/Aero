@echo off
REM Performance Benchmark Runner for Aero Phase 3 Features
REM This script runs comprehensive performance benchmarks

echo === Aero Phase 3 Performance Benchmarks ===
echo.

REM Change to the Aero root directory
cd /d "%~dp0\.."

REM Check if Python is available
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo Error: Python is not installed or not in PATH
    echo Please install Python 3.6+ to run performance benchmarks
    pause
    exit /b 1
)

REM Create results directory if it doesn't exist
if not exist "benchmarks\results" mkdir "benchmarks\results"

REM Run the Python benchmark script
echo Running performance benchmarks...
echo.
python benchmarks\performance_benchmark.py

if %errorlevel% neq 0 (
    echo.
    echo Error: Benchmark execution failed
    pause
    exit /b 1
)

echo.
echo Performance benchmarks completed successfully!
echo Results are saved in benchmarks\results\
echo.

REM Try to run Rust benchmarks if the compiler builds successfully
echo Attempting to run Rust criterion benchmarks...
cd src\compiler

REM Try to build first
cargo build --release >nul 2>&1
if %errorlevel% equ 0 (
    echo Compiler builds successfully, running Rust benchmarks...
    
    REM Run lexer benchmarks (these should work)
    cargo bench --bench lexer_only_benchmarks 2>nul
    if %errorlevel% equ 0 (
        echo Lexer benchmarks completed successfully
    ) else (
        echo Lexer benchmarks failed or not available
    )
) else (
    echo Compiler has build errors, skipping Rust benchmarks
    echo Focus on Python benchmarks for now
)

cd ..\..

echo.
echo All available benchmarks completed!
pause