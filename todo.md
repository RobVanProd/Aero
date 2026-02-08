# Aero Development Tasks

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

**Status:** 174 tests passing (63 unit + 52 optimizer + 59 frontend)

---

## 📋 Phase 6: Standard Library (Next)

- [ ] `Vec<T>` dynamic arrays
- [ ] `HashMap<K, V>` hash maps
- [ ] `Result<T, E>` error handling
- [ ] `Option<T>` optional values
- [ ] File I/O operations
- [ ] Module system (`mod`, `use`)
- [ ] Visibility controls (`pub`)

## 📋 Phase 7: Tooling (Future)

- [ ] Language Server Protocol (LSP)
- [ ] Package manager
- [ ] Documentation generator
- [ ] Code formatter
