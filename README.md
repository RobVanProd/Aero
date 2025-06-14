# Aero Programming Language

## Project Overview

Aero is a high-performance, ergonomic programming language designed for systems programming, aiming to combine the speed and control of low-level languages with the safety and productivity of modern language features. Inspired by Rust, Aero emphasizes memory safety without garbage collection through a unique ownership and borrowing model, and aims for excellent performance through efficient compilation and a focus on zero-cost abstractions.

This repository contains the foundational work for the Aero project, including its formal language specifications, initial project infrastructure, a bootstrap compiler prototype, and foundational benchmarks.

## Phase 1: Initial Foundation and Minimum Viable Product

Phase 1 focused on establishing a solid foundation for Aero, encompassing formal language specification, essential project infrastructure, a preliminary compiler prototype, and initial benchmarking capabilities.

### 1. Formalized Core Language Specification

Before any compiler implementation, the core aspects of the Aero language were formally defined to ensure deliberate foundational decisions and provide definitive guides for implementation.

#### EBNF Grammar

The Extended Backus-Naur Form (EBNF) grammar defines the complete syntax for a core subset of Aero. This document serves as the definitive guide for the lexer and parser components of the compiler.

[aero_grammar.md](aero_grammar.md)

#### Type System Rules

This specification details the behavior of Aero's static type system, including rules for type inference, built-in types (`int`, `float`, `bool`, `string`, `char`, `unit`), and the precise syntax for generic type parameters and trait bounds. The type system is designed to ensure type safety and support robust generic programming.

[aero_type_system.md](aero_type_system.md)

#### Ownership and Borrowing Model

This is a critical component of Aero's design, ensuring memory safety and concurrency without a garbage collector. The document specifies the exact compile-time rules for ownership, mutable borrows (`&mut T`), and immutable borrows (`&T`), along with the error conditions the compiler must detect (e.g., simultaneous mutable references).

[aero_ownership_borrowing.md](aero_ownership_borrowing.md)

### 2. Established Project Infrastructure

To support open and collaborative development, essential project infrastructure has been set up.

#### Git Repository

This project is hosted on GitHub:

[https://github.com/RobVanProd/Aero](https://github.com/RobVanProd/Aero)

#### License

Aero is released under the permissive MIT License, encouraging broad adoption and contribution.

[LICENSE](LICENSE)

#### Community and Communication

Guidelines for community interaction and communication channels are outlined to foster an engaging and collaborative environment. This document will be updated with details about the chosen communication platform (e.g., Discourse, Zulip, or Discord) for general discussions and the Request for Comments (RFC) process.

[COMMUNITY.md](COMMUNITY.md)

#### Code of Conduct

To ensure an inclusive and welcoming environment for all contributors, the project adopts the Contributor Covenant Code of Conduct.

[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

### 3. Developed Bootstrap Compiler Prototype

A preliminary version of the Aero compiler, written in Rust, has been developed. This 


bootstrap compiler demonstrates the core compilation pipeline.

#### Compiler Structure

The compiler prototype is located in the `src/compiler/` directory and is implemented in Rust. It includes simulated components for:

-   **Lexer**: Reads Aero source files and transforms them into tokens.
    (`src/compiler/src/lexer.rs`)
-   **Parser**: Transforms tokens into an Abstract Syntax Tree (AST) based on the EBNF grammar.
    (`src/compiler/src/parser.rs`)
-   **Semantic Analysis**: Implements type checking and the borrow checker to enforce static type rules and ownership/borrowing rules, generating helpful error messages for violations.
    (`src/compiler/src/semantic_analyzer.rs`)
-   **Intermediate Representation (IR) Generation**: Lowers the AST into a typed Intermediate Representation, designed for future optimizations.
    (`src/compiler/src/ir_generator.rs`)
-   **Code Generation**: For the initial prototype, this component simulates targeting a well-known backend like LLVM, translating the IR into LLVM IR.
    (`src/compiler/src/code_generator.rs`)

#### Running the Compiler Prototype

To build and run the simulated compiler prototype:

```bash
cd src/compiler
source "$HOME/.cargo/env" # Ensure Rust environment is sourced
cargo build
cargo run
```

### 4. Implemented Foundational Benchmarks

To validate performance claims and guide optimization from the outset, initial benchmarking capabilities have been integrated.

#### Selected Benchmarks

A small, targeted set of initial benchmarks has been selected from sources like the Computer Language Benchmarks Game. Placeholder implementations in Aero are provided for:

-   **N-body Simulation**: A classic computational physics problem.
    (`benchmarks/aero/nbody.aero`)
-   **Mandelbrot Set Generation**: A computationally intensive graphics problem.
    (`benchmarks/aero/mandelbrot.aero`)

#### Benchmarking Harness

A simple benchmarking harness has been created to compile and run Aero test programs, measure simulated execution time and memory usage, and provide a framework for rigorous performance validation.

[run_benchmarks.sh](benchmarks/harness/run_benchmarks.sh)

#### Running Benchmarks

To run the simulated benchmarks:

```bash
cd benchmarks/harness
./run_benchmarks.sh
```

## Getting Started

To get a local copy of the project up and running, follow these simple steps.

### Prerequisites

-   Git
-   Rust (for building the compiler prototype)

### Installation

1.  Clone the repository:

    ```bash
    git clone https://github.com/RobVanProd/Aero.git
    cd Aero
    ```

2.  Install Rust (if you haven't already):

    ```bash
    curl --proto \'=https\' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    source "$HOME/.cargo/env"
    ```

3.  Build the compiler prototype:

    ```bash
    cd src/compiler
    cargo build
    ```

## Contributing

We welcome contributions to the Aero project! Please refer to the [COMMUNITY.md](COMMUNITY.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for guidelines on how to contribute and our community standards.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

-   Inspired by the Rust programming language for its innovative ownership and borrowing model.
-   The Computer Language Benchmarks Game for providing a valuable source of algorithmic tests.
-   LLVM Project for its robust compiler infrastructure.


