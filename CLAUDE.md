# Aero Programming Language

## Project
Aero is a high-performance, ergonomic programming language. The compiler is written in Rust with an LLVM backend.

## Quick Reference
- **Test**: `./tools/test.sh` (cargo fmt --check + cargo test)
- **Build**: `cargo build`
- **Lint**: `cargo clippy` (correctness lints are blocking)
- **Default branch**: `master`

## Structure
- `src/compiler/` - Compiler source (lexer, parser, semantic analysis, IR gen, LLVM codegen)
- `examples/` - Example Aero programs
- `benchmarks/` - Performance benchmarks
- `tools/` - Development scripts

## Development Rules
1. Run `./tools/test.sh` before every commit
2. Follow Rust conventions (rustfmt, clippy clean)
3. Correctness clippy lints are blocking — fix them, don't suppress
4. Keep build artifacts out of git (use .gitignore)

## Current Phase: Phase 5 (Advanced Features) — COMPLETE
- Ownership and move semantics (DONE)
- References and borrowing syntax (DONE)
- Borrow checker enforcement (DONE)
- Generics: type params, trait bounds, where clauses (DONE)
- Traits: registry, completeness checking, bound enforcement (DONE)
- 174 tests passing (63 unit + 52 optimizer + 59 frontend integration)
- 37/38 Phase 5 spec tests passing (1 needs assignment statements — future work)

## Completed Phases
- Phase 5: Advanced Features (ownership, borrowing, borrow checker, generics, traits)
- Phase 4: Data Structures (arrays, structs, enums, pattern matching, tuples, strings)
- Phase 3: Control flow, functions, semantic analysis
- Phase 2: Binary operations, type inference
- Phase 1: Lexer, parser, basic codegen

## Key Design Docs
- `aero_grammar.md` - Language grammar
- `aero_type_system.md` - Type system
- `aero_ownership_borrowing.md` - Ownership & borrowing model
