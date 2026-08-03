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

## Current Evidence Status
- **Minimal prototype / correctness recovery.** No language feature is classified stable.
- Scalar parsing, selected semantic contracts, checked IR, and the qualified CPU LLVM path have active evidence; see `CURRENT_CAPABILITY_AUDIT.md` and `SPEC_IMPLEMENTATION_MATRIX.md`.
- Ownership, borrowing, generic, trait, struct, enum, tuple, and pattern syntax does not establish complete type checking, a borrow checker, lifetime safety, layout, or execution support.
- The 38 Phase 5 integration tests remain ignored pending test-by-test recovery/stub classification; they are not current coverage evidence.
- Use `PROJECT_STATE.md` for current test counts and accepted public checkpoints.

## Key Design Docs
- `aero_grammar.md` - Language grammar
- `aero_type_system.md` - Type system
- `aero_ownership_borrowing.md` - Ownership & borrowing model
