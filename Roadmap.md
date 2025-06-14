
# Roadmap

This document outlines the roadmap for the Aero programming language project, focusing on the objectives and deliverables for Phase 2.

## Phase 2: Real MVP Compiler

**Objective**: Deliver a real MVP compiler that can parse, type-check, and run simple programs with variables & arithmetic, emitting LLVM IR.

### 0. Planning & CI

-   **GitHub Actions CI**: Implement `cargo test`, `cargo clippy -- -D warnings`, and a unit-test matrix (stable + nightly).
    -   *Acceptance Criteria*: CI passes on all pushes and pull requests to the `master` branch.

-   **Roadmap.md**: This document, outlining Phase 2 epics with acceptance criteria.
    -   *Acceptance Criteria*: Roadmap is clear, comprehensive, and kept up-to-date.

### 1. Front-end: Variables & Expressions

-   **Lexer**: Add tokens for `let`, identifiers, integer & float literals, `+-*/%`, `;`, `=`. 
    -   *Acceptance Criteria*: Lexer correctly tokenizes all specified elements.

-   **Parser**: Implement Pratt or recursive-descent parsing for expression precedence. Define AST nodes for `Let {name, expr}`, `Binary {op, lhs, rhs}`, `Number`, `Ident`.
    -   *Acceptance Criteria*: Parser correctly constructs AST for simple variable declarations and arithmetic expressions.

-   **Semantic Analysis**: Implement a symbol-table (scoped HashMap) to detect re-declaration & use-before-initialize. Implement basic type inference for numeric literals.
    -   *Acceptance Criteria*: Semantic analyzer correctly identifies and reports errors for re-declarations and use-before-initialize. Basic numeric type inference works as expected.

-   **Tests**: Add `tests/parser.rs` and `tests/semantic.rs` using `insta` or `pretty_assertions` snapshots.
    -   *Acceptance Criteria*: Comprehensive unit tests for lexer, parser, and semantic analysis are in place and passing.

### 2. Mid-end: Typed IR

-   **IR Design**: Design a simple SSA-ish struct for the Intermediate Representation (IR), including `enum Value { Reg(u32), Imm(i64) }`, `enum Inst { Add(Value, Value), Sub, Mul, Div, Store(String, Value), Load(String) }`, and `struct Function { body: Vec<Inst> }`.
    -   *Acceptance Criteria*: IR structure is well-defined and capable of representing simple programs.

-   **Lowering Pass**: Implement a lowering pass from AST to IR with a fresh-register allocator.
    -   *Acceptance Criteria*: AST is correctly translated into the defined IR.

-   **Constant-folding (optional but tiny win)**: Implement basic constant folding optimization.
    -   *Acceptance Criteria*: Simple constant expressions are folded during IR generation.

### 3. Back-end: LLVM IR via Inkwell

-   **Inkwell Integration**: Add `inkwell = "0.4"` to `Cargo.toml`.
    -   *Acceptance Criteria*: Inkwell dependency is correctly integrated.

-   **IR to LLVM Builder**: Map IR instructions to LLVM builder functions (e.g., `Add` to `build_int_add`).
    -   *Acceptance Criteria*: IR is correctly translated into LLVM IR.

-   **Emit and JIT-run**: Implement functionality to emit `.ll` files and JIT-run (using `ExecutionEngine::run_function`).
    -   *Acceptance Criteria*: Compiler can produce LLVM IR files and execute them via JIT.

-   **CLI**: Implement `aero build prog.aero -o prog` to produce native binary via `llc + clang`, or `aero run prog.aero` to JIT-execute.
    -   *Acceptance Criteria*: Command-line interface for building and running Aero programs is functional.

### 4. User-Facing Improvements

-   **`examples/` folder**: Create `hello.aero`, `fib.aero`, `calc.aero` examples that compile in CI.
    -   *Acceptance Criteria*: Examples are clear, functional, and pass CI checks.

-   **`cargo install --path src/compiler` instructions**: Provide easy installation instructions for testers.
    -   *Acceptance Criteria*: Users can easily install the Aero compiler.

-   **RFC process kickoff**: Open `RFCs/` directory & template (borrow Rust’s process).
    -   *Acceptance Criteria*: RFC process is initiated with a clear template.

### 5. Stretch Targets (Phase 2½)

-   Simple unit type `()` and `print!` intrinsic.
-   Error reporting with `codespan-reporting` (underline faulty code).
-   `Docs.rs` automated build for the compiler crate.


