# Aero Formal Language Specification (v1.0.0)

This document is the consolidated formal specification for Aero v1.0.0.
It defines the core syntax, static semantics, ownership model, module rules,
and execution model used by the reference compiler.

The normative split specifications remain:
- `docs/language/aero_grammar.md`
- `docs/language/aero_type_system.md`
- `docs/language/aero_ownership_borrowing.md`

This document aligns and unifies those rules into a single reference.

## 1. Conformance

An implementation conforms to Aero v1.0.0 if it:
- Accepts programs defined by the v1.0.0 grammar.
- Enforces the static semantics and ownership rules in this document.
- Rejects programs that violate type safety, ownership, or borrowing constraints.
- Produces behavior equivalent to the execution model in Section 8.

## 2. Lexical Structure

Aero source text is a sequence of Unicode scalar values interpreted as:
- Identifiers: `[_A-Za-z][_A-Za-z0-9]*`
- Keywords: `fn`, `let`, `mut`, `if`, `else`, `while`, `for`, `in`, `return`,
  `struct`, `enum`, `trait`, `impl`, `mod`, `use`, `pub`, `where`, `match`
- Literals: integer, float, string, and formatted string literals
- Punctuation and operators as defined in the grammar

Comments and whitespace separate tokens but do not affect semantics.

## 3. Concrete Syntax

The grammar is specified in EBNF in `aero_grammar.md`.

Core syntactic categories:
- Program
- Item declarations (`fn`, `struct`, `enum`, `trait`, `impl`, `mod`, `use`)
- Statements (`let`, assignment-like forms, control flow, expression statements)
- Expressions (literals, unary/binary, calls, method calls, indexing, field access)
- Type expressions (named, generic, arrays, tuples, references)

## 4. Static Semantics

Typing judgments use the form:

`Gamma; Delta |- e : T`

Where:
- `Gamma` is the type environment (variables, functions, traits, impls).
- `Delta` is the ownership/borrow environment.
- `e` is an expression and `T` is its type.

### 4.1 Variable and binding rules

- `let x = e;` introduces `x : T` where `Gamma; Delta |- e : T`.
- `let mut x = e;` marks `x` mutable.
- Rebinding in the same scope is rejected.
- Shadowing in nested scopes is permitted.

### 4.2 Function rules

- Function signatures are globally visible within a compilation unit.
- Argument arity and argument types must match parameter types.
- Return expressions must unify with declared return type.
- Generic parameters and `where` constraints are checked at call sites.

### 4.3 Operator rules

- Numeric operators require numeric operands.
- Boolean/logical operators require boolean operands.
- Comparison operators require compatible operand types.
- Implicit promotion follows v1.0.0 numeric rules (for example `int` to `float` in mixed arithmetic).

### 4.4 Trait rules

- Trait declarations define required method sets.
- `impl Trait for Type` must provide all required methods.
- Generic trait bounds (`T: Trait`) must be satisfied at instantiation/call sites.

## 5. Ownership and Borrowing

Ownership judgments use:

`Delta |- v : own(T)` for owned values,
`Delta |- r : &T` for shared borrows,
`Delta |- r : &mut T` for mutable borrows.

Rules:
- Each value has exactly one owner.
- Move transfers ownership; moved values are invalidated in prior owner context.
- Any number of shared borrows are allowed if no mutable borrow is active.
- Exactly one mutable borrow is allowed, and no shared borrows may overlap it.
- Borrow lifetimes are bounded by lexical scopes and use sites.

Violations are compile-time errors.

## 6. Modules and Name Resolution

Module declarations (`mod x;`) are resolved to source files according to the resolver rules:
- `x.aero`
- `x/mod.aero`

`use` imports bind names into the local scope.
`pub` controls visibility across module boundaries.

Circular module dependencies are rejected.

## 7. Intermediate Representation and Lowering

The reference implementation lowers typed AST into an SSA-like IR that includes:
- Arithmetic and comparison instructions
- Memory instructions (`alloca`, `load`, `store`)
- Control flow (`branch`, `jump`, `label`)
- Function call and return instructions
- Aggregate operations (arrays, structs, enums, vector-like operations)

The backend lowers IR to LLVM IR.

## 8. Execution Model

Aero execution model is deterministic under a single-threaded program order:
- Expression evaluation follows source order except where compile-time optimization preserves equivalence.
- Function calls establish a new stack frame/scope.
- `return` exits current function.
- Branch and loop control transfer follows lowered CFG edges.

Undefined behavior at runtime is minimized by compile-time ownership/type checking.

## 9. Diagnostics

A conforming implementation should report:
- Syntax errors with location information.
- Type mismatch and arity diagnostics.
- Ownership and borrow-check failures.
- Module resolution errors.

Diagnostics should prefer precise source ranges and actionable suggestions.

## 10. Phase 8 Interface Extensions (v1.0.0)

The following Phase 8 interfaces are part of the v1.0.0 tooling surface:

- Quantization interface with calibrated lowering for `INT8`, `FP8-E4M3`, and `FP8-E5M2`, including backend selection (`cpu`, `cuda`, `rocm`).
- Kernel-fusion and advanced graph-compilation pass with executable fused-kernel backend generation and safety fallbacks.
- `registry.aero` interface for search/publish/install workflows with offline index mode and live transport mode, including token auth and digest trust policy controls.
- Formal conformance and mechanized determinism checks exposed through the `aero conformance` command.

These interfaces define stable command and report shapes in v1.0.0 while allowing
backend execution strategies and proof depth to continue evolving.
