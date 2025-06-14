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

-   **Ownership and Borrowing:** A compile-time memory management system that prevents dangling pointers, data races, and other common memory-related bugs without a garbage collector.
-   **Static Typing:** A strong, static type system helps catch errors at compile time and provides a solid foundation for robust applications.
-   **Type Inference:** Reduces the need for explicit type annotations, making code cleaner while maintaining type safety.
-   **Generics and Traits:** Powerful tools for creating reusable abstractions and achieving polymorphism.
-   **Pattern Matching:** (Planned) For expressive control flow and destructuring of data.
-   **Modular Design:** Code can be organized into modules for better structure and reusability.
-   **Comprehensive Standard Library:** (In progress) Aims to provide essential utilities for common tasks.

## Current Status

Aero is an actively developed programming language. It is currently in **Phase 2: Core Feature Implementation and Tooling** (as outlined in our [Roadmap.md](Roadmap.md)). This means core language features are being implemented in the compiler, and foundational tooling is under development.

The language is not yet production-ready but is rapidly evolving. We welcome feedback and contributions!

## Getting Started

Ready to try Aero? Here’s how to get started:

### Installation

To use Aero, you'll first need to install its compiler, `aero`. The compiler is written in Rust, so you'll need Rust and Cargo installed. If you don't have them, please visit [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install).

1.  **Clone the Aero Repository (if you have it locally, navigate to its root):**
    ```bash
    # git clone <repository_url> # If you don't have it yet
    # cd aero # Or your repository's directory name
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
    aero --version
    ```
    This should display the installed Aero compiler version.

### Your First Aero Program

Let's create a simple "Hello, world!" program.

1.  Create a file named `hello.aero` with the following content:
    ```aero
    // hello.aero
    fn main() {
        io::println("Hello, world! Welcome to Aero!");
    }
    ```

### Compiling and Running

There are two main ways to compile and run your Aero programs:

1.  **Build then Run:**
    First, compile your program into an executable:
    ```bash
    aero build hello.aero -o hello_aero_executable
    ```
    This creates an executable file named `hello_aero_executable`. You can then run it:
    -   On Linux/macOS: `./hello_aero_executable`
    -   On Windows: `.\hello_aero_executable.exe`

2.  **Compile and Run Directly:**
    For convenience, you can compile and immediately run your program with a single command:
    ```bash
    aero run hello.aero
    ```

You should see the output: `Hello, world! Welcome to Aero!`

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
