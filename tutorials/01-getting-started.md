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

Let's write a classic "Hello, world!" program.

1.  **Create a new file:**
    Create a file named `hello.aero` and open it in your favorite text editor.

2.  **Write the code:**
    Enter the following Aero code into `hello.aero`:

    ```aero
    // hello.aero - Our first Aero program

    // The `main` function is the entry point of every Aero executable.
    fn main() {
        // `io::println` prints a line of text to the console.
        // It's part of the `io` module in the standard library.
        io::println("Hello, world!");
    }
    ```

### Code Explanation:

*   `// hello.aero - Our first Aero program`: This is a single-line comment. Comments are ignored by the compiler but are useful for humans reading the code.
*   `fn main() { ... }`: This defines a function named `main`. The `main` function is special: it's always the first code that runs in every executable Aero program.
    *   `fn` is the keyword used to declare a function.
    *   `main` is the name of the function.
    *   `()` indicates that this function takes no parameters.
    *   `{ ... }` The function body is enclosed in curly braces.
*   `io::println("Hello, world!");`: This line does the work of printing text to the screen.
    *   `io::println` calls the `println` function, which is part of the `io` module from Aero's standard library. The `::` syntax is used to access items within a module.
    *   `"Hello, world!"` is a string literal that we pass as an argument to `println`.
    *   Aero statements are typically terminated with a semicolon `;`.

## Compiling and Running

Now that you have your "Hello, world!" program, let's compile and run it.

### 1. Compiling Separately

You can compile your Aero program into an executable file first.

1.  **Compile the code:**
    Open your terminal, navigate to the directory where you saved `hello.aero`, and run:
    ```bash
    aero build hello.aero -o hello_aero_executable
    ```
    *   `aero build`: This is the command to the Aero compiler to build your code.
    *   `hello.aero`: This is the source file you want to compile.
    *   `-o hello_aero_executable`: The `-o` flag specifies the name for the output executable file. If you omit this, the executable might be named `hello` (or `hello.exe` on Windows) by default, based on the input file name.

    If the compilation is successful, you will see a new file named `hello_aero_executable` (or whatever you named it) in your directory.

2.  **Run the executable:**
    Now, you can run your compiled program:
    *   On Linux or macOS:
        ```bash
        ./hello_aero_executable
        ```
    *   On Windows:
        ```bash
        .\hello_aero_executable.exe
        ```
        (Or just `hello_aero_executable.exe`)

    You should see the output:
    ```
    Hello, world!
    ```

### 2. Compile and Run Directly

Aero also provides a convenient `run` command that compiles and immediately runs your program without explicitly creating an executable file in your current directory (it might create one in a temporary location).

1.  **Compile and run:**
    In your terminal, in the directory of `hello.aero`, type:
    ```bash
    aero run hello.aero
    ```
    This command will compile `hello.aero` and then execute it.

    You should see the same output:
    ```
    Hello, world!
    ```

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
