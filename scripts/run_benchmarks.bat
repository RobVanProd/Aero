@echo off
REM Comprehensive benchmark runner for Aero Phase 3 features
REM This script runs all performance benchmarks and generates reports

echo === Aero Phase 3 Performance Benchmarks ===
echo Starting benchmark suite...
echo.

REM Change to the compiler directory
cd /d "%~dp0\..\src\compiler"

REM Ensure we're in release mode for accurate benchmarks
set CARGO_PROFILE_RELEASE_DEBUG=true

echo 1. Function Call Overhead Benchmarks
echo ----------------------------------------
cargo bench --bench function_call_overhead -- --output-format pretty
if %ERRORLEVEL% neq 0 (
    echo X Function Call Performance Tests failed
    exit /b 1
) else (
    echo √ Function Call Performance Tests completed successfully
)
echo.

echo 2. Loop Performance Benchmarks
echo ----------------------------------------
cargo bench --bench loop_performance -- --output-format pretty
if %ERRORLEVEL% neq 0 (
    echo X Loop Construct Performance Tests failed
    exit /b 1
) else (
    echo √ Loop Construct Performance Tests completed successfully
)
echo.

echo 3. I/O Operations Benchmarks
echo ----------------------------------------
cargo bench --bench io_operations -- --output-format pretty
if %ERRORLEVEL% neq 0 (
    echo X I/O Operation Performance Tests failed
    exit /b 1
) else (
    echo √ I/O Operation Performance Tests completed successfully
)
echo.

echo 4. Compilation Speed Benchmarks
echo ----------------------------------------
cargo bench --bench compilation_speed -- --output-format pretty
if %ERRORLEVEL% neq 0 (
    echo X Compilation Performance Tests failed
    exit /b 1
) else (
    echo √ Compilation Performance Tests completed successfully
)
echo.

echo 5. Performance Regression Benchmarks
echo ----------------------------------------
cargo bench --bench performance_regression -- --output-format pretty
if %ERRORLEVEL% neq 0 (
    echo X Regression Testing Against Baseline failed
    exit /b 1
) else (
    echo √ Regression Testing Against Baseline completed successfully
)
echo.

echo === Benchmark Summary ===
echo All benchmarks completed successfully!
echo.
echo Results have been saved to target\criterion\
echo You can view detailed HTML reports by opening:
echo   target\criterion\report\index.html
echo.
echo To compare results over time, run this script regularly
echo and use 'cargo bench' with specific benchmark names.
echo.
echo Benchmark suite completed!