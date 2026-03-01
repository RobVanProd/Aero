# Aero Development Roadmap

This document outlines the development roadmap for the Aero programming language.

## Completed Phases

### Phase 1-2: Foundation ✅

- **Lexer & Parser:** Complete tokenization and AST generation
- **Semantic Analysis:** Symbol table, type inference, validation
- **Type System:** Static typing with int/float distinction and promotion
- **IR & Code Generation:** SSA-style IR with LLVM backend
- **CLI Tools:** `aero build`, `aero run`, `aero check`, `aero test`, `aero fmt`, `aero init`, `aero lsp`
- **CI/CD:** GitHub Actions with comprehensive test matrix

### Phase 3: Control Flow & Functions ✅

- **Function Definitions:** Parameters, return types, recursion
- **Control Flow:** if/else, while/for loops, break/continue
- **I/O Operations:** `print!` and `println!` macros
- **Scoping:** Nested scopes, variable shadowing, mutability

### Phase 4: Data Structures ✅

- **Arrays:** Fixed-size and dynamic arrays with indexing
- **Structs:** Custom data types with field access and methods
- **Enums:** Algebraic data types with variants
- **Pattern Matching:** Match expressions with exhaustiveness checking
- **Tuples:** Anonymous composite types
- **Strings:** Built-in string type

### Phase 5: Advanced Features ✅

- **Ownership:** Move semantics for value transfer
- **Borrowing:** Shared (`&T`) and mutable (`&mut T`) references
- **Borrow Checker:** Compile-time enforcement of borrowing rules
- **Generics:** Type parameters with trait bounds and where clauses
- **Traits:** Trait definitions, implementations, and bound enforcement

### Phase 6: Standard Library ✅

- **Core Collections:** `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`, `String`
- **Error Handling:** `Result<T, E>`, `Option<T>`, `?` operator
- **I/O:** File reading/writing, standard streams, buffered I/O
- **Module System:** `mod` declarations, `use` imports, `pub` visibility
- **Concurrency:** `Arc`, `Mutex` synchronization primitives

### Phase 7: Tooling & Developer Experience ✅

- **Package Manager:** `aero-pkg` with dependency resolution and workspaces
- **CLI Expansion:** `aero check` (type-check only), `aero test`, `aero fmt`, `aero init`
- **Basic LSP Diagnostics:** `aero lsp` with syntax diagnostics from parser multi-error recovery
- **Compiler Diagnostics:** Colored errors, source snippets, suggestions
- **Closures & Lambdas:** `|x, y| { ... }` syntax with capture semantics

**Current Status:** 189+ tests passing | **Version:** 1.0.0

---

## Future Phases

### Phase 8: Optimization & Ecosystem (v1.1.0+)
- Advanced LSP features (completion, hover, go-to-definition, symbols)
- Documentation generator (`aero doc`)
- INT8/FP8 quantization interfaces
- Kernel fusion & advanced graph compilation
- Native profiler & flame graphs
- Central package registry (registry.aero)
- Formal language specification

---

**Version:** 1.0.0 | **License:** MIT
