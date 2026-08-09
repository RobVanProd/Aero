# Aero Development Tasks

> **Historical record — not current capability evidence.** Completion and version
> labels below preserve the original planning snapshot. Current evidence and open
> risks live in `PROJECT_STATE.md`, `CURRENT_CAPABILITY_AUDIT.md`, and
> `INITIAL_RISK_REGISTER.md`.

## ✅ Phase 1-2: Foundation (Complete)

- [x] Create formal grammar document (EBNF)
- [x] Detail type system rules
- [x] Define ownership and borrowing model
- [x] Implement Lexer & Parser
- [x] Implement Semantic Analysis
- [x] Implement IR Generation
- [x] Implement LLVM Code Generation
- [x] Create CLI tools (`aero build`, `aero run`)
- [x] Setup GitHub Actions CI/CD

## ✅ Phase 3: Control Flow & Functions (Complete)

- [x] Function definitions with parameters and return types
- [x] Function calls with type checking
- [x] If/else statements
- [x] While loops
- [x] For loops with ranges
- [x] Break and continue statements
- [x] Print macros (`print!`, `println!`)
- [x] Format string validation
- [x] Nested scopes and variable shadowing

## ✅ Phase 4: Data Structures (Complete)

- [x] Arrays with indexing
- [x] Structs with field access
- [x] Struct methods
- [x] Enums with variants
- [x] Pattern matching
- [x] Exhaustiveness checking
- [x] Tuples
- [x] Strings

## ✅ Phase 5: Advanced Features (Complete)

- [x] Ownership and move semantics
- [x] Shared references (`&T`)
- [x] Mutable references (`&mut T`)
- [x] Borrow checker enforcement
- [x] Generic type parameters
- [x] Trait bounds
- [x] Where clauses
- [x] Trait definitions
- [x] Trait implementations
- [x] Bound enforcement

## ✅ Phase 6: Standard Library (Complete)

- [x] `Vec<T>` dynamic arrays (IR-level Vec variants)
- [x] `HashMap<K, V>` hash maps
- [x] `Result<T, E>` error handling
- [x] `Option<T>` optional values
- [x] File I/O operations
- [x] Module system (`mod`, `use`)
- [x] Visibility controls (`pub`)

## ✅ Phase 7: Tooling (Complete)

- [x] Package manager (`aero-pkg`)
- [x] CLI expansion (`check`, `test`, `fmt`)
- [x] Compiler diagnostics with colored output

**Status:** 189+ tests passing | **Version:** 1.0.0

---

## 📋 Phase 8: Future (v1.1.0+)

- [ ] Language Server Protocol (LSP) server
- [ ] Documentation generator (`aero doc`)
- [ ] INT8/FP8 quantization interfaces
- [ ] Kernel fusion & graph compilation
- [ ] Native profiler & flame graphs
- [ ] Central package registry (registry.aero)
