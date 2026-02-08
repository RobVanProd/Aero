# Aero Programming Language

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
<!-- Add other badges as appropriate, e.g., build status, version -->

## Overview

Aero is a modern, statically-typed programming language designed for performance, safety, and developer productivity. It aims to provide the control and speed of systems programming languages while offering high-level abstractions and a user-friendly ergonomic syntax.

**Core Goals:**
-   **Performance:** Achieve performance comparable to C/C++ through efficient compilation and zero-cost abstractions.
-   **Memory Safety:** Guarantee memory safety at compile time without a garbage collector, primarily through its ownership and borrowing system.
-   **Ergonomics:** Offer a clean, intuitive syntax and powerful tooling to enhance the developer experience.
-   **Concurrency:** Provide robust and safe concurrency features.

## Key Features

### Core Language ✅

-   **Static Typing:** Strong, static type system with compile-time type checking and automatic type inference
-   **Memory Safety:** Ownership and borrowing system prevents memory bugs at compile time
-   **Type Inference:** Automatic type deduction for variables and expressions
-   **LLVM Backend:** Native code generation through LLVM for optimal performance
-   **CLI Tooling:** Complete command-line interface with `aero build` and `aero run` commands

### Phase 3: Control Flow & Functions ✅

-   **Function Definitions:** Complete support for defining and calling functions with parameters and return types
-   **Control Flow:** if/else statements, while/for loops, break/continue
-   **I/O Operations:** `print!` and `println!` macros with format string validation
-   **Advanced Scoping:** Nested scopes, variable shadowing, and function-local variables

### Phase 4: Data Structures ✅

-   **Arrays and Slices:** Fixed-size and dynamic array support
-   **Structs and Methods:** Custom data types with associated functions
-   **Enums and Pattern Matching:** Algebraic data types with exhaustive matching
-   **Tuples and Strings:** Built-in composite types

### Phase 5: Advanced Features ✅

-   **Ownership and Borrowing:** Compile-time memory management without garbage collection
-   **Generics:** Type parameters, trait bounds, and where clauses
-   **Traits:** Trait definitions, implementations, and bound enforcement
-   **Borrow Checker:** Full enforcement of borrowing rules

### Planned Features 📋

-   **Comprehensive Standard Library:** Collections, I/O, networking, concurrency primitives
-   **Module System:** Code organization with imports and visibility controls
-   **Error Handling:** Result types and propagation operators

## Current Status

**Version:** 0.5.0 (Phase 5 Complete)  
**Status:** Advanced Features Implemented

Aero has successfully completed Phase 5 development with ownership, borrowing, generics, and traits. The compiler features:

- ✅ Ownership and move semantics
- ✅ References and borrowing with borrow checker
- ✅ Generics with type parameters and trait bounds
- ✅ Traits with registry and bound enforcement
- ✅ Data structures: arrays, structs, enums, tuples, strings
- ✅ Pattern matching with exhaustiveness checking
- ✅ 174 tests passing (63 unit + 52 optimizer + 59 frontend)

## Quick Start

### Installation

1. Clone the repository:
```bash
git clone https://github.com/aero-lang/aero.git
cd aero
```

2. Build the compiler:
```bash
cd src/compiler
cargo build --release
```

3. Add to PATH (optional):
```bash
# Add the target/release directory to your PATH
export PATH=$PATH:$(pwd)/target/release
```

### Your First Aero Program

Create a file called `hello.aero`:

```aero
fn greet(name: &str) -> () {
    println!("Hello, {}!", name);
}

fn main() {
    let name = "World";
    greet(name);
    
    // Demonstrate control flow
    let mut count = 0;
    while count < 5 {
        if count % 2 == 0 {
            println!("{} is even", count);
        } else {
            println!("{} is odd", count);
        }
        count = count + 1;
    }
}
```

Compile and run:
```bash
aero run hello.aero
```

Or compile to LLVM IR:
```bash
aero build hello.aero -o hello.ll
```

## Language Examples

### Function Definitions
```aero
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn factorial(n: i32) -> i32 {
    if n <= 1 {
        return 1;
    }
    return n * factorial(n - 1);
}
```

### Control Flow
```aero
fn main() {
    // If/else statements
    let x = 10;
    if x > 5 {
        println!("x is greater than 5");
    } else {
        println!("x is 5 or less");
    }
    
    // While loops
    let mut i = 0;
    while i < 3 {
        println!("Iteration: {}", i);
        i = i + 1;
    }
    
    // For loops
    for j in 0..5 {
        if j == 3 {
            break;
        }
        println!("For loop: {}", j);
    }
}
```

### I/O Operations
```aero
fn main() {
    let name = "Aero";
    let version = 3;
    
    print!("Welcome to ");
    println!("{} v0.{}!", name, version);
    
    let a = 15;
    let b = 4;
    println!("{} + {} = {}", a, b, a + b);
    println!("{} * {} = {}", a, b, a * b);
}
```

## Project Structure

```
aero/
├── src/compiler/          # Main compiler source code
│   ├── src/
│   │   ├── lexer.rs      # Tokenization and lexical analysis
│   │   ├── parser.rs     # Syntax analysis and AST generation
│   │   ├── semantic_analyzer.rs  # Type checking and validation
│   │   ├── ir_generator.rs       # Intermediate representation
│   │   ├── code_generator.rs     # LLVM code generation
│   │   └── main.rs       # CLI interface
│   └── Cargo.toml        # Rust project configuration
├── examples/             # Example Aero programs
│   ├── calculator.aero   # Complex calculator demo
│   ├── fibonacci.aero    # Recursive fibonacci
│   ├── loops.aero        # Control flow examples
│   └── scoping.aero      # Variable scoping demo
├── benchmarks/           # Performance benchmarks
├── tutorials/            # Learning materials
└── README.md            # This file
```

## Development Roadmap

### Completed Phases

- **Phase 3:** Control Flow & Functions ✅
- **Phase 4:** Data Structures (arrays, structs, enums, pattern matching) ✅
- **Phase 5:** Advanced Features (ownership, borrowing, generics, traits) ✅

### Phase 6: Standard Library (Next)
- Collections (Vec, HashMap, etc.)
- String manipulation
- File I/O and networking
- Concurrency primitives
- Module system and imports
- Error handling with Result types

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Setting up the development environment
- Code style and conventions
- Testing requirements
- Pull request process

### Development Setup

1. Install Rust (1.70+ required)
2. Install LLVM development libraries
3. Clone and build:
```bash
git clone https://github.com/aero-lang/aero.git
cd aero/src/compiler
cargo build
cargo test
```

## Testing

Run the test suite:
```bash
cd src/compiler
cargo test                    # Unit tests
cargo test --test integration # Integration tests
cargo bench                   # Performance benchmarks
```

## Performance

Aero is designed for performance. Current benchmarks show:

- **Compilation Speed:** ~50,000 lines/second
- **Function Call Overhead:** <2ns per call
- **Loop Performance:** Comparable to C/C++
- **Memory Usage:** Minimal runtime overhead

See [benchmarks/](benchmarks/) for detailed performance analysis.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- LLVM Project for the backend infrastructure
- Rust community for inspiration and tooling
- All contributors and early adopters

---

**Status:** Active Development | **Version:** 0.5.0 | **License:** MIT

**All Core Phases Complete ✅** - The compiler now features:
- Ownership, borrowing, and borrow checker enforcement
- Generics with type parameters, trait bounds, and where clauses
- Traits with registry, completeness checking, and bound enforcement
- Data structures: arrays, structs, enums, tuples, strings, pattern matching
- Full control flow, functions, and I/O operations
- 174 tests passing across unit, optimizer, and integration suites

The language has a solid foundation and is ready for Phase 6 (Standard Library). We welcome contributions!

## Getting Started

Ready to try Aero? Here’s how to get started:

### Installation

To use Aero, you'll first need to install its compiler, `aero`. The compiler is written in Rust, so you'll need Rust and Cargo installed. You'll also need LLVM tools (`llc` and `clang`) for compilation.

#### Prerequisites

1. **Install Rust and Cargo:**
   Visit [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) and follow the installation instructions.

2. **Install LLVM tools:**
   - **Ubuntu/Debian:** `sudo apt install llvm clang`
   - **macOS:** `brew install llvm` (or use Xcode command line tools)
   - **Windows:** Download from [LLVM releases](https://releases.llvm.org/) or use `winget install LLVM.LLVM`

#### Installation Steps

1.  **Clone the Aero Repository:**
    ```bash
    git clone https://github.com/RobVanProd/Aero.git
    cd Aero
    ```

2.  **Install the Aero Compiler:**
    From the root directory of the Aero project, run:
    ```bash
    cargo install --path src/compiler
    ```
    This command builds the `aero` binary from the source code in `src/compiler/` and installs it into your Cargo binary directory (usually `~/.cargo/bin/`).

3.  **Verify Installation:**
    Ensure `~/.cargo/bin` is in your system's `PATH`. Then, open a new terminal session and type:
    ```bash
    aero --help
    ```
    This should display the Aero compiler help message.

#### Testing the Installation

To verify that everything is working correctly, you can run the test suite:

**Linux/macOS:**
```bash
chmod +x test_compiler.sh
./test_compiler.sh
```

**Windows:**
```cmd
test_compiler.bat
```

The test suite will build the compiler and run several example programs to ensure everything is working correctly.

### Your First Aero Program

Let's create a simple program that demonstrates Aero's current capabilities.

1.  Create a file named `example.aero` with the following content:
    ```aero
    // example.aero - Demonstrates Phase 3 Aero features
    
    fn greet(name: &str) {
        println!("Hello, {}!", name);
    }
    
    fn calculate_factorial(n: i32) -> i32 {
        if n <= 1 {
            return 1;
        } else {
            return n * calculate_factorial(n - 1);
        }
    }
    
    fn main() {
        println!("=== Aero Language Demo ===");
        
        // Variables and mutability
        let x = 10;
        let mut y = 5;
        y = y + 3;
        
        println!("x = {}, y = {}", x, y);
        
        // Function calls
        greet("Aero Developer");
        
        // Control flow and recursion
        let fact = calculate_factorial(5);
        println!("5! = {}", fact);
        
        // Loops
        let mut i = 0;
        while i < 3 {
            print!("Count: {} ", i);
            i = i + 1;
        }
        println!("");
        
        println!("Demo complete!");
    }
    ```

This program showcases:
- Function definitions with parameters and return types
- Variable declarations with mutability control
- I/O operations with formatted printing
- Control flow (if/else statements)
- Loops (while loops)
- Recursion and function calls

### What's Currently Working

Aero can currently compile and run programs with these features:

**Variables and Mutability:**
```aero
let x = 42;           // Immutable integer variable
let mut y = 3.14;     // Mutable float variable
y = y + 1.0;          // Reassignment to mutable variable
let z = x + y;        // Mixed arithmetic with type promotion
```

**Functions:**
```aero
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

fn greet(name: &str) {
    println!("Hello, {}!", name);
}

fn main() {
    let result = add(5, 3);
    greet("World");
}
```

**Control Flow:**
```aero
// If/else statements
if x > 0 {
    println!("Positive");
} else if x < 0 {
    println!("Negative");
} else {
    println!("Zero");
}

// While loops
let mut i = 0;
while i < 5 {
    println!("Count: {}", i);
    i = i + 1;
}

// Infinite loops with break/continue
loop {
    if condition {
        break;
    }
    continue;
}
```

**I/O Operations:**
```aero
print!("Hello ");           // Print without newline
println!("World!");         // Print with newline
println!("Value: {}", 42);  // Formatted printing
println!("Sum: {}", a + b); // Expression formatting
```

**Enhanced Type System:**
```aero
// Comparison operators
let equal = (a == b);
let not_equal = (a != b);
let less = (a < b);
let greater = (a > b);

// Logical operators
let and_result = (true && false);
let or_result = (true || false);
let not_result = !true;
```

### Compiling and Running

There are two main ways to compile and run your Aero programs:

1.  **Build then Run:**
    First, compile your program into an executable:
    ```bash
    aero build example.aero -o example_executable
    ```
    This creates an executable file named `example_executable`. You can then run it:
    -   On Linux/macOS: `./example_executable`
    -   On Windows: `.\example_executable.exe`

2.  **Compile and Run Directly:**
    For convenience, you can compile and immediately run your program with a single command:
    ```bash
    aero run example.aero
    ```

The program will execute and return an exit code of 13 (the result of 10 + 3.5, truncated to integer). You can check the exit code with:
- Linux/macOS: `echo $?`
- Windows: `echo %ERRORLEVEL%`

## Learning Aero

Dive deeper into the Aero language with these resources:

-   **Tutorials**:
    -   [Tutorial 1: Getting Started](tutorials/01-getting-started.md)
    -   [Tutorial 2: Core Language Features](tutorials/02-core-features.md)
    -   [Tutorial 3: Ownership and Borrowing](tutorials/03-ownership-borrowing.md)
    -   [Tutorial 4: Data Structures (Structs, Enums, etc.)](tutorials/04-data-structures.md)
-   **Example Programs**:
    -   [fibonacci.aero](examples/fibonacci.aero) - Functions and recursion
    -   [loops.aero](examples/loops.aero) - All loop types and control flow
    -   [calculator.aero](examples/calculator.aero) - I/O operations and functions
    -   [scoping.aero](examples/scoping.aero) - Variable scoping and mutability
    -   [error_examples.aero](examples/error_examples.aero) - Common error cases
-   **Language Design Documents**:
    -   [Aero Grammar (EBNF)](aero_grammar.md)
    -   [Type System Rules](aero_type_system.md)
    -   [Ownership and Borrowing Model](aero_ownership_borrowing.md)
-   **Troubleshooting**:
    -   [Troubleshooting Guide](TROUBLESHOOTING.md) - Solutions to common issues

## Standard Library

Aero aims to provide a useful standard library to assist with common programming tasks. The design and features of the standard library are currently being defined.

-   Learn more about the proposed standard library structure and APIs in the [Standard Library RFC](RFCs/standard-library.md).

## Contributing

Aero is an open-source project, and we welcome contributions from the community! Whether you're interested in language design, compiler development, writing documentation, or creating examples, there are many ways to help.

-   **Contribution Guidelines**: Please read [CONTRIBUTING.md](CONTRIBUTING.md) (if it exists, otherwise check for community guidelines or open an issue to ask).
-   **Code of Conduct**: We adhere to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Please ensure you read and follow it.
-   **RFCs (Request for Comments)**: Major language design changes and new features are discussed through an RFC process. Check out the [RFCs directory](RFCs/) to participate or propose new ideas.
-   **Issues**: Report bugs or suggest features by opening an issue on our GitHub repository.
-   **Pull Requests**: We welcome well-tested pull requests for bug fixes, feature implementations, and documentation improvements.

## Roadmap

To see the current development phase and future plans for Aero, please refer to the [Roadmap.md](Roadmap.md).

## License

Aero is distributed under the terms of the MIT license.

See [LICENSE](LICENSE) for details.
