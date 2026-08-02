# Aero Specification-to-Implementation Matrix

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

This matrix records stages independently. `Y` means direct evidence for the
listed slice, `P` means partial or known-defective support, `N` means absent,
`?` means not yet verified, and `—` means not applicable. The only feature-level
classifications are `ABSENT`, `DESIGNED`, `PARSED_ONLY`, `PARTIAL`,
`EXPERIMENTAL`, `END_TO_END`, and `STABLE`. No row is `STABLE` during the initial
audit.

Abbreviations: `Res` name resolution; `Ty` type checking; `Own` ownership
checking; `TIR` typed/structured IR; `BE` LLVM or other backend lowering; `Exec`
successful execution; `+/-/D` positive, negative, and diagnostic tests.

## Language features

| Feature | Spec | Lex | Parse | Res | Ty | Own | TIR | BE | Exec | + | - | D | Docs | Class |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| Integers/floats and arithmetic | Y | P | Y | — | P | — | P | P | P | Y | P | P | Y | PARTIAL |
| Booleans/chars | Y | N | N | — | P | — | N | N | N | N | N | N | Y | DESIGNED |
| Bindings and mutability | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Type annotations | Y | Y | Y | P | N | N | P | P | P | P | N | N | Y | PARSED_ONLY |
| Comparisons/logical/unary ops | Y | Y | P | — | P | — | P | P | ? | Y | P | P | Y | PARTIAL |
| Functions and returns | Y | Y | Y | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Function-call signatures | Y | Y | Y | N | N | N | P | P | P | P | N | N | Y | PARSED_ONLY |
| If/else | Y | Y | P | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| While/for/loop/break/continue | Y | Y | P | P | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Strings and formatting | Y | P | P | — | P | P | P | P | P | Y | P | P | Y | PARTIAL |
| Fixed arrays | Y | Y | Y | P | P | P | P | P | ? | Y | P | P | Y | PARTIAL |
| Tuples | Y | Y | Y | P | P | P | ? | ? | ? | Y | P | P | Y | PARTIAL |
| Structs and field access | Y | Y | P | N | P | P | P | P | ? | P | P | P | Y | PARTIAL |
| Enums and construction | Y | Y | P | N | P | P | P | P | ? | P | P | P | Y | PARTIAL |
| Pattern matching | Y | Y | P | N | N | N | P | P | ? | P | N | N | Y | PARSED_ONLY |
| Generics and substitutions | Y | Y | P | P | P | P | N | N | N | P | P | P | Y | PARSED_ONLY |
| Traits, bounds, and impls | Y | Y | P | P | P | P | N | N | N | P | P | P | Y | PARSED_ONLY |
| Moves | Y | — | Y | P | P | P | ? | ? | ? | P | P | P | Y | PARTIAL |
| Shared/mutable references | Y | Y | Y | P | P | P | ? | ? | ? | P | P | P | Y | PARTIAL |
| Closures | P | P | P | N | N | N | P | P | ? | P | N | N | P | PARSED_ONLY |
| Modules/imports/visibility | Y | Y | P | P | N | N | N | N | N | P | P | P | Y | PARSED_ONLY |
| Standard collections | P | Y | P | N | P | P | P | P | ? | P | P | P | P | EXPERIMENTAL |
| C/foreign-function interface | P | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? | ? | P | DESIGNED |

## Compiler, tooling, and ecosystem surfaces

| Surface | Interface | Shared compiler truth | Artifact/result | Failure tests | Integration evidence | Docs | Class |
|---|---:|---:|---:|---:|---:|---:|---|
| Library `compile_program` | Y | P | LLVM text or located parse error | Y | P | P | PARTIAL |
| Compiler options | Y | N | Ignored | N | N | P | PARSED_ONLY |
| CLI build/check | Y | N | P; surfaced compile failures nonzero | Y | P | Y | PARTIAL |
| CLI run | Y | N | CPU path; ROCm staged; CUDA absent | Y | P | Y | PARTIAL |
| CLI test | Y | N | Analysis-only result; failures nonzero | Y | P | Y | PARTIAL |
| Formatter | Y | N | Text trimming | N | N | P | EXPERIMENTAL |
| Diagnostics/source spans | Y | P | Point/one-char ranges | P | P | Y | PARTIAL |
| LSP | Y | N | P | P | P | Y | EXPERIMENTAL |
| Documentation generator | Y | P | Markdown | P | P | Y | EXPERIMENTAL |
| Profiler | Y | N | Timing/trace or located parse error | Y | P | Y | EXPERIMENTAL |
| Project initialization | Y | — | Project files | Y | P | Y | EXPERIMENTAL |
| Module resolver | Y | P | Resolved source | P | P | Y | EXPERIMENTAL |
| Registry | Y | — | Local search; incomplete live publish/install | P | N | Y | EXPERIMENTAL |
| Conformance command | Y | P | 3 cases + 4 deterministic checks | P | P | P | EXPERIMENTAL |
| Package lock/reproducible resolution | P | ? | ? | ? | ? | P | DESIGNED |

## Backend summary

Detailed stage evidence lives in `BACKEND_STATUS.md`.

| Backend/surface | Selectable | IR transform | Object | Link | Real execution | Numerical checks | Performance evidence | Class |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| CPU | Y | Y | P | P | P | P | P | PARTIAL |
| ROCm | Y | Y | P | N | N | N | Historical/under audit | EXPERIMENTAL |
| CUDA | Y | P | N | N | N | N | N | PARSED_ONLY |
| Graph compilation | Y | Y | — | — | Helper lowering only under audit | P | N | EXPERIMENTAL |
| Quantization | Y | Y | — | — | Helper lowering only under audit | P | N | EXPERIMENTAL |

## Evidence notes

- The lexer cannot return errors and currently converts some invalid input into
  valid tokens, so otherwise present lexical cells remain partial.
- Type annotations, function declarations, function-call signatures, and return
  types are not enforced by the active semantic path.
- Many declarations lose visibility, bounds, arguments, or source locations in
  the AST; parser presence alone therefore does not imply faithful parsing.
- The CLI and library declare overlapping compiler modules. Tooling rows cannot
  claim shared compiler truth until that duplication is removed.
- Current conformance determinism checks are useful regression evidence, not
  formal-semantics proof.
- At `6ce85922`, trusted library/build/check/run/test/profile parser paths reject
  malformed root and applicable direct-module sources with located errors. Lexer
  failures remain uncontrolled, and shared compiler truth remains partial.

This file must be tightened as audit items close. A row may become `END_TO_END`
only with source-to-execution evidence and all applicable positive, negative,
diagnostic, documentation, and backend gates. `STABLE` additionally requires a
declared compatibility policy and release-level coverage.
