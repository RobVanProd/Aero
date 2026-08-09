# Tutorial 1: Getting Started with Aero

Welcome to Aero! Aero is a modern, statically-typed programming language designed for performance, safety, and developer productivity. It aims to combine the power of systems programming with high-level abstractions and a user-friendly syntax.

This tutorial will guide you through installing the Aero compiler, writing your first Aero program, and compiling and running it.

## Prerequisites

Before you begin, install Rust and Cargo from
[rust-lang.org](https://www.rust-lang.org/tools/install). Native `aero run`
execution also requires the LLVM 22 and Clang tools listed in the repository's
[build prerequisites](../BUILD.md#prerequisites).

## Installation

The Aero compiler is named `aero`. The commands below build it from a repository
checkout without assuming a root Cargo workspace.

1.  **Clone the Aero repository (if you haven't already):**

    ```bash
    git clone https://github.com/RobVanProd/Aero.git
    cd Aero
    ```

2.  **Build the compiler:**

    From the repository root, select the compiler manifest explicitly:

    ```bash
    cargo build --release --manifest-path src/compiler/Cargo.toml
    ```

3.  **Add the release binary to PATH and verify it:**

    ```bash
    export PATH="$PWD/src/compiler/target/release:$PATH"
    aero --version
    ```

    Windows users should follow the exact
    [PowerShell build and PATH instructions](../BUILD.md#windows-powershell).
    Installing with `cargo install --path src/compiler` remains optional.

## Your First Aero Program: "Hello, Aero!"

Use the compiler's existing generated project as the executable first program.

1.  **Initialize a project:**
    ```bash
    aero init my_app
    cd my_app
    ```
    This creates:
    - `aero.toml`
    - `src/main.aero`

2.  **Write the code:**
    Open `src/main.aero` and use:

    ```aero
    fn main() {
        println!("Hello, Aero!");
    }
    ```

### Code Explanation:

*   `fn main() { ... }`: This defines a function named `main`. The `main` function is special: it's always the first code that runs in every executable Aero program.
    *   `fn` is the keyword used to declare a function.
    *   `main` is the name of the function.
    *   `()` indicates that this function takes no parameters.
    *   `{ ... }` The function body is enclosed in curly braces.
*   `println!("Hello, Aero!");`: This line does the work of printing text to the screen.
    *   `println!` is a built-in macro that prints text followed by a newline. The `!` indicates it's a macro.
    *   `"Hello, Aero!"` is a string literal that we pass as an argument to `println!`.
    *   Aero statements are typically terminated with a semicolon `;`.

## Compiling and Running

Now that you have your "Hello, Aero!" program, let's compile and run it.

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

CPU is the only current process-execution target. A ROCm request can reach only
temporary object emission; it has no HIP link, device launch, or program
execution. CUDA has no active object, link, or launch path. To inspect ROCm
target metadata without requesting execution, build LLVM IR explicitly:

```bash
aero build src/main.aero -o main.rocm.ll --target rocm --gpu gfx1101
```

The CPU command should complete successfully and include exactly this program-output
line:

```
Output: Hello, Aero!
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

Current experimental `aero lsp` support includes:

- Syntax diagnostics as you type
- Completion suggestions
- Hover information
- Go-to-definition
- Document symbols

### 5. Generate Markdown API docs

Use `doc` to generate API documentation from declarations in a source file:

```bash
aero doc src/main.aero -o main.md
```

### 6. Profile compilation stages

Use `profile` to print per-stage compilation timing and optionally export a trace file:

```bash
aero profile src/main.aero -o trace.json
```

### 7. Inspect graph helper rewriting

Use `graph-opt` on LLVM IR to rewrite selected chains to internal scalar helpers.
Backend labels are metadata and do not establish device execution:

```bash
aero graph-opt main.ll -o main.opt.ll --backend rocm --gpu gfx1101
```

### 8. Inspect quantization-label helper rewriting

Use `quantize` to rewrite floating-point operations through scalar-`double` helpers.
There is no real FP8 representation, per-channel execution, or
numerical-correctness proof:

```bash
aero quantize main.opt.ll -o main.int8.ll --mode int8 --backend rocm --gpu gfx1101 --calibration calib.json
```

Supported modes:
- `int8`
- `fp8-e4m3`
- `fp8-e5m2`

### 9. Registry commands (local + dry-run)

The current registry surface supports local-index search and network-free
publish/install planning:

```bash
aero registry search vision --index registry/index.json
aero registry publish . --dry-run
aero registry install vision-core --version 0.2.0 --target pkgs --dry-run
```

These routes do not resolve registry credentials and do not contact a network.
Live search and publish/install without `--dry-run` fail closed with
`live registry transport is disabled pending a reviewed protocol and trust boundary`.
Live transport remains a future design target; it will not be enabled until package,
response, authentication, destination, overwrite, and dependency contracts are
reviewed and tested adversarially.

### 10. Run conformance regression checks

Use `conformance` to run 3 example cases and 4 deterministic regression checks;
this is not a formal semantics proof:

```bash
aero conformance -o conformance_report.json
```

## What's Next?

Congratulations on running your first Aero program!

In the next tutorials, we will explore:
*   Variables and Data Types
*   Functions in more detail
*   Control Flow (if/else, loops)
*   The conceptual ownership and borrowing design (not current memory-safety enforcement)
*   Structs and Enums
*   And much more!

Keep exploring and happy coding in Aero!
