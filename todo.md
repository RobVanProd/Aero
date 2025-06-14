## Phase 1: Formalize the Core Language Specification

- [x] Create a formal grammar document (EBNF)
- [x] Detail the type system rules
- [x] Define the ownership and borrowing model

## Phase 2: Establish Project Infrastructure

- [x] Create a public Git repository
- [x] Adopt a permissive open-source license
- [x] Establish a communication channel
- [x] Adopt a Code of Conduct

## Phase 3: Develop the Bootstrap Compiler Prototype

- [x] Implement the Frontend
  - [x] Lexer & Parser
  - [x] Semantic Analysis
- [x] Implement the Backend
  - [x] Intermediate Representation (IR)
  - [x] Code Generation

## Phase 4: Implement Foundational Benchmarks

- [x] Select initial benchmarks
- [x] Develop a rigorous measurement process
- [x] Create a simple benchmarking harness




## Phase 5: Create Comprehensive README

- [x] Create Comprehensive README




## Phase 6: Setup CI/CD and Roadmap

- [x] Setup GitHub Actions CI
- [x] Create Roadmap.md




## Phase 7: Implement Frontend - Variables & Expressions

- [x] Implement tokens & variable grammar (Lexer → Parser)
- [x] Land first passing test: let x = 3 + 4; should JIT-execute to exit-code 7.




## Phase 8: Implement Mid-end - Typed IR

- [x] IR design: simple SSA-ish struct
- [x] Lowering pass: AST → IR with fresh-register allocator.
- [x] Constant-folding (optional but tiny win).




## Phase 9: Implement Backend - LLVM IR (String Output)

- [x] Generate LLVM IR as a string for simple programs.
- [x] Update `main.rs` to call the modified `code_generator.rs` and print the generated LLVM IR string.
- [x] Create a simple test case (e.g., `let x = 3 + 4;`) and verify that the generated LLVM IR string is correct and can be compiled externally (e.g., using `llc` and `clang` in the sandbox).




## Phase 10: Stack-allocated variables & alloca / store / load

- [x] IR tweak: Give each let a virtual stack slot ID. Lower to: %ptr0 = alloca i64, store i64 %r0, i64* %ptr0, %r1 = load i64, i64* %ptr0
- [x] Code-gen string: Add helpers: fn fresh_reg() -> String and fn fresh_ptr() -> String so the emitter never re-uses names.
- [x] Semantic pass: Track VarInfo { ptr_name, ty } in a HashMap. Emit “use-before-init” diagnostics when a load occurs without a prior store.
- [x] CI test: New example: let a = 2; let b = a * 3; Assert IR compiles & the exit code is 6.


