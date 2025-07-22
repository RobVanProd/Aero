#!/usr/bin/env python3
"""
Performance Benchmark Suite for Aero Phase 3 Features
This script runs performance benchmarks for function calls, loops, I/O operations,
and compilation speed to validate Phase 3 implementation performance.
"""

import os
import sys
import time
import subprocess
import statistics
import json
from pathlib import Path
from typing import Dict, List, Tuple, Optional

class AeroBenchmark:
    def __init__(self, aero_root: Path):
        self.aero_root = aero_root
        self.compiler_path = aero_root / "src" / "compiler"
        self.benchmarks_dir = aero_root / "benchmarks" / "aero"
        self.results = {}
        
    def run_compilation_benchmark(self, source_file: Path, iterations: int = 10) -> Dict:
        """Run compilation benchmark for a given source file"""
        times = []
        
        for i in range(iterations):
            start_time = time.perf_counter()
            
            try:
                # Run the Aero compiler
                result = subprocess.run([
                    "cargo", "run", "--release", "--", str(source_file)
                ], 
                cwd=self.compiler_path,
                capture_output=True,
                text=True,
                timeout=30
                )
                
                end_time = time.perf_counter()
                compilation_time = end_time - start_time
                
                if result.returncode == 0:
                    times.append(compilation_time)
                else:
                    print(f"Compilation failed for {source_file}: {result.stderr}")
                    
            except subprocess.TimeoutExpired:
                print(f"Compilation timeout for {source_file}")
            except Exception as e:
                print(f"Error running compilation benchmark: {e}")
        
        if times:
            return {
                "mean": statistics.mean(times),
                "median": statistics.median(times),
                "min": min(times),
                "max": max(times),
                "std_dev": statistics.stdev(times) if len(times) > 1 else 0,
                "iterations": len(times)
            }
        else:
            return {"error": "No successful compilations"}
    
    def run_execution_benchmark(self, executable_path: Path, iterations: int = 5) -> Dict:
        """Run execution benchmark for a compiled program"""
        times = []
        
        for i in range(iterations):
            start_time = time.perf_counter()
            
            try:
                result = subprocess.run([str(executable_path)], 
                                      capture_output=True, 
                                      text=True, 
                                      timeout=10)
                
                end_time = time.perf_counter()
                execution_time = end_time - start_time
                
                if result.returncode == 0:
                    times.append(execution_time)
                else:
                    print(f"Execution failed for {executable_path}: {result.stderr}")
                    
            except subprocess.TimeoutExpired:
                print(f"Execution timeout for {executable_path}")
            except Exception as e:
                print(f"Error running execution benchmark: {e}")
        
        if times:
            return {
                "mean": statistics.mean(times),
                "median": statistics.median(times),
                "min": min(times),
                "max": max(times),
                "std_dev": statistics.stdev(times) if len(times) > 1 else 0,
                "iterations": len(times)
            }
        else:
            return {"error": "No successful executions"}
    
    def benchmark_function_performance(self) -> Dict:
        """Benchmark function call overhead and performance"""
        print("Running function performance benchmarks...")
        
        function_files = [
            "function_performance.aero",
        ]
        
        results = {}
        
        for file_name in function_files:
            file_path = self.benchmarks_dir / file_name
            if file_path.exists():
                print(f"  Benchmarking {file_name}...")
                results[file_name] = {
                    "compilation": self.run_compilation_benchmark(file_path),
                    "description": "Function call overhead and recursion performance"
                }
            else:
                print(f"  Warning: {file_name} not found")
                
        return results
    
    def benchmark_loop_performance(self) -> Dict:
        """Benchmark loop constructs performance"""
        print("Running loop performance benchmarks...")
        
        loop_files = [
            "loop_performance.aero",
        ]
        
        results = {}
        
        for file_name in loop_files:
            file_path = self.benchmarks_dir / file_name
            if file_path.exists():
                print(f"  Benchmarking {file_name}...")
                results[file_name] = {
                    "compilation": self.run_compilation_benchmark(file_path),
                    "description": "While, for, and infinite loop performance"
                }
            else:
                print(f"  Warning: {file_name} not found")
                
        return results
    
    def benchmark_io_performance(self) -> Dict:
        """Benchmark I/O operations performance"""
        print("Running I/O performance benchmarks...")
        
        io_files = [
            "io_performance.aero",
        ]
        
        results = {}
        
        for file_name in io_files:
            file_path = self.benchmarks_dir / file_name
            if file_path.exists():
                print(f"  Benchmarking {file_name}...")
                results[file_name] = {
                    "compilation": self.run_compilation_benchmark(file_path),
                    "description": "Print and format string performance"
                }
            else:
                print(f"  Warning: {file_name} not found")
                
        return results
    
    def benchmark_compilation_speed(self) -> Dict:
        """Benchmark compilation speed for various program sizes"""
        print("Running compilation speed benchmarks...")
        
        # Test with example programs of different sizes
        example_files = []
        examples_dir = self.aero_root / "examples"
        
        if examples_dir.exists():
            for file_path in examples_dir.glob("*.aero"):
                example_files.append(file_path)
        
        results = {}
        
        for file_path in example_files:
            print(f"  Benchmarking compilation of {file_path.name}...")
            
            # Get file size for context
            file_size = file_path.stat().st_size
            
            results[file_path.name] = {
                "compilation": self.run_compilation_benchmark(file_path),
                "file_size_bytes": file_size,
                "description": f"Compilation speed for {file_path.name}"
            }
                
        return results
    
    def benchmark_performance_regression(self) -> Dict:
        """Run regression tests to ensure Phase 3 doesn't degrade performance"""
        print("Running performance regression benchmarks...")
        
        # Create simple test programs to measure baseline performance
        test_programs = {
            "simple_arithmetic.aero": '''
fn main() {
    let a = 10;
    let b = 20;
    let result = a + b * 2 - 5;
}
''',
            "simple_functions.aero": '''
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn main() {
    let result = add(5, 3);
}
''',
            "simple_loops.aero": '''
fn main() {
    for i in 0..100 {
        let x = i * 2;
    }
}
''',
            "simple_io.aero": '''
fn main() {
    println!("Hello, World!");
}
'''
        }
        
        results = {}
        temp_dir = self.aero_root / "temp_benchmarks"
        temp_dir.mkdir(exist_ok=True)
        
        try:
            for program_name, program_code in test_programs.items():
                temp_file = temp_dir / program_name
                temp_file.write_text(program_code)
                
                print(f"  Benchmarking {program_name}...")
                results[program_name] = {
                    "compilation": self.run_compilation_benchmark(temp_file),
                    "description": f"Regression test for {program_name}"
                }
        finally:
            # Clean up temporary files
            for temp_file in temp_dir.glob("*.aero"):
                temp_file.unlink()
            temp_dir.rmdir()
                
        return results
    
    def run_all_benchmarks(self) -> Dict:
        """Run all performance benchmarks"""
        print("=== Aero Phase 3 Performance Benchmarks ===")
        print()
        
        all_results = {
            "timestamp": time.time(),
            "benchmarks": {
                "function_performance": self.benchmark_function_performance(),
                "loop_performance": self.benchmark_loop_performance(),
                "io_performance": self.benchmark_io_performance(),
                "compilation_speed": self.benchmark_compilation_speed(),
                "performance_regression": self.benchmark_performance_regression()
            }
        }
        
        return all_results
    
    def save_results(self, results: Dict, output_file: Path):
        """Save benchmark results to JSON file"""
        with open(output_file, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"Results saved to {output_file}")
    
    def print_summary(self, results: Dict):
        """Print a summary of benchmark results"""
        print("\n=== Benchmark Summary ===")
        
        for category, benchmarks in results["benchmarks"].items():
            print(f"\n{category.replace('_', ' ').title()}:")
            
            for test_name, test_results in benchmarks.items():
                if "compilation" in test_results:
                    comp_results = test_results["compilation"]
                    if "mean" in comp_results:
                        print(f"  {test_name}: {comp_results['mean']:.4f}s avg compilation")
                    elif "error" in comp_results:
                        print(f"  {test_name}: {comp_results['error']}")

def main():
    # Find the Aero root directory
    current_dir = Path(__file__).parent
    aero_root = current_dir.parent
    
    if not (aero_root / "src" / "compiler").exists():
        print("Error: Could not find Aero compiler directory")
        sys.exit(1)
    
    # Create benchmark runner
    benchmark = AeroBenchmark(aero_root)
    
    # Run all benchmarks
    results = benchmark.run_all_benchmarks()
    
    # Save results
    results_file = aero_root / "benchmarks" / "results" / f"performance_results_{int(time.time())}.json"
    results_file.parent.mkdir(exist_ok=True)
    benchmark.save_results(results, results_file)
    
    # Print summary
    benchmark.print_summary(results)
    
    print(f"\nBenchmark suite completed!")
    print(f"Detailed results saved to: {results_file}")

if __name__ == "__main__":
    main()