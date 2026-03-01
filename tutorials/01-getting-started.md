# Tutorial 1: Getting Started with Aero

Welcome to Aero! Aero is a modern, statically-typed programming language designed for performance, safety, and developer productivity. It aims to combine the power of systems programming with high-level abstractions and a user-friendly syntax.

This tutorial will guide you through installing the Aero compiler, writing your first Aero program, and compiling and running it.

## Prerequisites

Before you begin, you'll need to have Rust and Cargo installed on your system, as the Aero compiler is currently built using Cargo. If you don't have them, please visit [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install) for instructions.

## Installation

The Aero compiler is named `aero`. You can install it from the source code using Cargo.

1.  **Clone the Aero Repository (if you haven't already):**
    If you're working with the Aero source code, you've likely already done this. If not, you would typically clone the repository:
    ```bash
    # git clone <repository_url>
    # cd <repository_directory>
    ```
    (Replace `<repository_url>` and `<repository_directory>` with the actual URL and local path).

2.  **Install the Compiler:**
    Navigate to the root directory of the Aero project (the one containing the `Cargo.toml` for the entire project, which should include the compiler workspace). Then, run the following command:

    ```bash
    cargo install --path src/compiler
    ```
    This command tells Cargo to build the `aero` compiler binary from the `src/compiler` path and install it into Cargo's binary directory (usually `~/.cargo/bin/`).

3.  **Verify Installation:**
    After the installation is complete, ensure that Cargo's binary directory is in your system's `PATH` environment variable. You can then verify the installation by opening a new terminal session and typing:
    ```bash
    aero --version
    ```
    This should print the Aero compiler version if the installation was successful. If you get a "command not found" error, ensure `~/.cargo/bin` is in your `PATH`.

## Your First Aero Program: "Hello, world!"

Let's write a classic "Hello, world!" program using the v1.0.0 project scaffold.

1.  **Initialize a project:**
    ```bash
    aero init hello_aero
    cd hello_aero
    ```
    This creates:
    - `aero.toml`
    - `src/main.aero`

2.  **Write the code:**
    Open `src/main.aero` and use:

    ```aero
    // src/main.aero - Our first Aero program
    fn main() {
        // `println!` prints a line of text to the console.
        println!("Hello, world!");
    }
    ```

### Code Explanation:

*   `// src/main.aero - Our first Aero program`: This is a single-line comment. Comments are ignored by the compiler but are useful for humans reading the code.
*   `fn main() { ... }`: This defines a function named `main`. The `main` function is special: it's always the first code that runs in every executable Aero program.
    *   `fn` is the keyword used to declare a function.
    *   `main` is the name of the function.
    *   `()` indicates that this function takes no parameters.
    *   `{ ... }` The function body is enclosed in curly braces.
*   `println!("Hello, world!");`: This line does the work of printing text to the screen.
    *   `println!` is a built-in macro that prints text followed by a newline. The `!` indicates it's a macro.
    *   `"Hello, world!"` is a string literal that we pass as an argument to `println!`.
    *   Aero statements are typically terminated with a semicolon `;`.

## Compiling and Running

Now that you have your "Hello, world!" program, let's compile and run it.

### 1. Build to LLVM IR

Use `build` to generate LLVM IR:

```bash
aero build src/main.aero -o main.ll
```

### 2. Compile and Run Directly

Use `run` to compile and execute in one command:

```bash
aero run src/main.aero
```

You should see:

```
Hello, world!
```

### 3. Type-check only

Use `check` when you only want diagnostics:

```bash
aero check src/main.aero
```

### 4. Optional: editor tooling with LSP

Start the language server over stdio:

```bash
aero lsp
```

Current `aero lsp` support in v1.0.0 includes:

- Syntax diagnostics as you type
- Completion suggestions
- Hover information
- Go-to-definition
- Document symbols

## What's Next?

Congratulations on running your first Aero program!

In the next tutorials, we will explore:
*   Variables and Data Types
*   Functions in more detail
*   Control Flow (if/else, loops)
*   Ownership and Borrowing (Aero's key memory safety feature)
*   Structs and Enums
*   And much more!

Keep exploring and happy coding in Aero!
