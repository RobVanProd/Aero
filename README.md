# Aero Programming Language

[![License: MIT](httpsa://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
<!-- Add other badges as appropriate, e.g., build status, version -->

## Overview

Aero is a modern, statically-typed programming language designed for performance, safety, and developer productivity. It aims to provide the control and speed of systems programming languages while offering high-level abstractions and a user-friendly ergonomic syntax.

**Core Goals:**
-   **Performance:** Achieve performance comparable to C/C++ through efficient compilation and zero-cost abstractions.
-   **Memory Safety:** Guarantee memory safety at compile time without a garbage collector, primarily through its ownership and borrowing system.
-   **Ergonomics:** Offer a clean, intuitive syntax and powerful tooling to enhance the developer experience.
-   **Concurrency:** Provide robust and safe concurrency features.

## Key Features

### Currently Implemented ✅

-   **Static Typing:** A strong, static type system with compile-time type checking and automatic type inference
-   **Type Inference:** Automatic type deduction for variables and expressions while maintaining type safety
-   **Memory Safety:** Stack-allocated variables with proper lifetime management
-   **Arithmetic Operations:** Full support for integer and floating-point arithmetic with automatic type promotion
-   **Variable System:** Immutable variables by default with explicit mutability support (`let` vs `let mut`)
-   **Comprehensive Error Reporting:** Clear error messages with type information and validation
-   **LLVM Backend:** Native code generation through LLVM for optimal performance
-   **CLI Tooling:** Complete command-line interface with `aero build` and `aero run` commands

### Phase 3 Features (In Development) 🚧

-   **Function Definitions:** Define and call functions with parameters and return types (`fn name(params) -> type`)
-   **Control Flow:** If/else statements, while loops, for loops, and infinite loops with break/continue
-   **I/O Operations:** Print macros (`print!`, `println!`) with format string validation
-   **Enhanced Type System:** Comparison operators (`==`, `!=`, `<`, `>`, `<=`, `>=`) and logical operators (`&&`, `||`, `!`)
-   **Advanced Scoping:** Nested scopes, variable shadowing, and function-local variables
-   **Semantic Validation:** Comprehensive compile-time checking for all language constructs

### Planned Features 📋

-   **Ownership and Borrowing:** A compile-time memory management system that prevents dangling pointers, data races, and other common memory-related bugs without a garbage collector
-   **Generics and Traits:** Powerful tools for creating reusable abstractions and achieving polymorphism
-   **Pattern Matching:** Expressive control flow and destructuring of data
-   **Data Structures:** Structs, enums, arrays, and other composite types
-   **Modular Design:** Code organization into modules for better structure and reusability
-   **Comprehensive Standard Library:** Essential utilities for common programming tasks

## Current Status

Aero is an actively developed programming language. It is currently in **Phase 3: Core Language Features** (as outlined in our [Roadmap.md](Roadmap.md)). Phase 2 has been completed successfully, and we are now implementing advanced language features including functions, control flow, and I/O operations.

**Phase 2 Complete ✅** - The compiler now has:
- Full lexical analysis and parsing
- Complete semantic analysis with type checking
- LLVM IR generation and native compilation
- Working CLI tools (`aero build` and `aero run`)
- Comprehensive test suite and CI/CD

**Phase 3 In Progress 🚧** - Currently implementing:
- Function definitions and calls
- Control flow statements (if/else, loops)
- I/O operations (print!, println!)
- Enhanced type system with comparisons and logical operations
- Advanced scope management and variable mutability

The language is not yet production-ready but has a solid foundation and is rapidly evolving. We welcome feedback and contributions!

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
    // example.aero - Demonstrates current Aero features
    let x = 10;
    let y = 3.5;
    let result = x + y;  // Automatic type promotion: int + float = float
    return result;       // Returns 13.5 (truncated to 13 as exit code)
    ```

This program showcases:
- Variable declarations with type inference
- Mixed integer and floating-point arithmetic
- Automatic type promotion
- Return statements

### What's Currently Working

Aero can currently compile and run programs with these features:

**Variables and Types:**
```aero
let x = 42;           // Integer variable
let y = 3.14;         // Float variable
let z = x + y;        // Mixed arithmetic with type promotion
```

**Arithmetic Operations:**
```aero
let a = 10 + 5;       // Addition
let b = 20 - 8;       // Subtraction  
let c = 4 * 6;        // Multiplication
let d = 15 / 3;       // Division
```

**Return Values:**
```aero
let final_result = (a + b) * c / d;
return final_result;  // Program exit code
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
-   **Language Design Documents**:
    -   [Aero Grammar (EBNF)](aero_grammar.md)
    -   [Type System Rules](aero_type_system.md)
    -   [Ownership and Borrowing Model](aero_ownership_borrowing.md)

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
