# Aero Development Roadmap

This document outlines the development roadmap for the Aero programming language.

## Completed Phases

### Phase 1-2: Foundation ✅

- **Lexer & Parser:** Complete tokenization and AST generation
- **Semantic Analysis:** Symbol table, type inference, validation
- **Type System:** Static typing with int/float distinction and promotion
- **IR & Code Generation:** SSA-style IR with LLVM backend
- **CLI Tools:** `aero build` and `aero run` commands
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

**Current Status:** 174 tests passing (63 unit + 52 optimizer + 59 frontend)

---

## Phase 6: Standard Library (Next)

The next phase focuses on building a comprehensive standard library.

### 6.1 Core Collections
- `Vec<T>` - Dynamic arrays
- `HashMap<K, V>` - Hash maps
- `HashSet<T>` - Hash sets
- `String` - UTF-8 string handling

### 6.2 I/O Operations
- File reading and writing
- Standard input/output streams
- Buffered I/O

### 6.3 Error Handling
- `Result<T, E>` type
- `Option<T>` type
- `?` operator for error propagation

### 6.4 Module System
- `mod` declarations
- `use` imports
- Visibility controls (`pub`, `pub(crate)`)

### 6.5 Concurrency (Stretch Goal)
- Threads
- Channels
- Synchronization primitives

---

## Future Phases

### Phase 7: Tooling & Ecosystem
- Language Server Protocol (LSP) implementation
- Package manager
- Documentation generator
- Formatter and linter

### Phase 8: Optimization & Performance
- Advanced LLVM optimizations
- Profile-guided optimization
- Link-time optimization

---

**Version:** 0.5.0 | **License:** MIT
