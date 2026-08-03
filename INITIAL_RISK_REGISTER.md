# Aero Initial Risk Register

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

Likelihood and impact are `LOW`, `MEDIUM`, `HIGH`, or `CRITICAL`. Status remains
open until a regression test and the applicable full gate prove closure.

| ID | Risk | Likelihood | Impact | Evidence | Required control | Status |
|---|---|---|---|---|---|---|
| R-001 | Invalid characters/numbers/strings are silently changed into valid tokens | HIGH | CRITICAL | Trusted paths use strict lexing at `b988318`; legacy recovery remains public for compatibility and LSP symbol indexing only | Keep recovery output ineligible for semantics/artifacts; add fuzz/property coverage and eventual diagnostic-accumulating migration | CONTROLLED — trusted paths closed |
| R-002 | Calls, annotations, and returns violate declared type contracts | HIGH | CRITICAL | Monomorphic numeric/void function calls and returns are controlled at `8d5d8e7`; initialized exact numeric binding annotations are controlled at `bc9a148`; boolean/generic/composite, uninitialized, and non-numeric contracts remain unenforced or uncertified | Retain separate boundaries for uninitialized and richer types; define conversion and assignment policy before broadening | PARTIALLY CONTROLLED — numeric functions and initialized numeric bindings closed |
| R-003 | Unsupported expressions are accepted with invented integer/zero semantics | HIGH | CRITICAL | `%`, tuple values, named fields, Match, and StructLiteral fail closed at their reviewed boundaries; accepted `CORE-010` at `db349ef` adds generic checked-IR rejection for every unadmitted trusted-path fallback, including ordinary MethodCall, enum construction, and Deref/Borrow | Retain checked admission on every trusted caller; keep non-deprecated raw `generate_ir` and deprecated `generate_code` ineligible as trusted public boundaries, while permitting only checked-wrapper reuse followed by verification; define aggregate/ownership/method semantics before implementation | CONTROLLED — trusted checked compiler paths no longer fabricate scalar values; direct public unchecked compatibility use remains uncertified |
| R-004 | Ownership claims exceed enforcement and permit dangling/aliased/moved values | HIGH | CRITICAL | Shallow move tracking, no lifetime provenance, mutable references considered `Copy` | Freeze ownership model; CFG/provenance checking; permanent compile-fail suite | OPEN |
| R-005 | Invalid programs pass semantics then panic, miscompile, or produce invalid LLVM | HIGH | CRITICAL | Accepted `CORE-010` at `db349ef` provides checked logical IR admission, mandatory in-process verification, exhaustive checked codegen errors, and qualified final LLVM 22 verification across trusted callers; focused contracts, full gate, and public CI pass | Preserve mandatory checked APIs/verifiers, deprecate/restrict then retire public unchecked compatibility APIs at a major boundary, and extend typed negative evidence as language forms become admitted | PARTIALLY CONTROLLED — trusted checked scalar IR and externally verified publication routes are controlled; CLI `InternalOnly` and library-returned LLVM without external verification, broader language semantics, and public unchecked APIs remain uncertified |
| R-006 | CLI and library compile through divergent module instances | HIGH | HIGH | Accepted `CORE-011` at `a711dd5` centralizes direct-module collection across both compiled trees, rejects source-only/nested declarations, and includes exact module sources in pre-lookup cache identity; focused/full local gates, exact review, and public CI pass | Preserve the shared collector, then separately design canonical library/thin-CLI orchestration and `CompilerOptions` | PARTIALLY CONTROLLED — direct-module source-set boundary is closed; duplicated orchestration and full pipeline convergence remain open |
| R-007 | Backend labels are mistaken for device execution | HIGH | HIGH | ROCm stops at object generation; CUDA run says unimplemented; README advertises broad run/graph/quantization surfaces | Independent backend status, explicit artifact/execution telemetry and hardware tests | OPEN |
| R-008 | Public 1.0/formal/safety messaging outruns evidence | HIGH | HIGH | `1.0.0` docs/CLI versus package `0.3.0`; three conformance cases and deterministic repetition | Unified version policy and evidence-based stability/claims language | OPEN |
| R-009 | Source ranges and recovery cannot support trustworthy diagnostics | HIGH | HIGH | Token start points only, no AST spans, one-character LSP ranges, recovery consumes valid code | End-to-end span model and recovery-retention tests | OPEN |
| R-010 | Grammar, tutorials, examples, and implementation define incompatible languages | HIGH | HIGH | Keyword/literal/field/rebinding/lifetime/top-level discrepancies | Freeze authoritative grammar subset; executable documentation examples | OPEN |
| R-011 | Aggregate and array lowering changes types or crashes | HIGH | HIGH | Only first array element checked; double storage and float-index truncation; composite fallback lowering | Homogeneity/index tests and typed aggregate IR before execution claims | OPEN |
| R-012 | Dormant, duplicated, or ignored tests create false coverage confidence | HIGH | HIGH | 78 duplicate active names, 38 ignored phase-5 tests, 299 dormant source tests, no key negative/fuzz suites | Classify backlog; restore valid tests deliberately; report distinct coverage and gates | OPEN |
| R-013 | User-facing commands report success without promised behavior | HIGH | HIGH | Corrected `AUDIT-019` probes at `b7bb429` reproduced status zero across no/unknown command, malformed usage, missing inputs, registry/conformance errors, and the bare benchmark source path; accepted public `CORE-013` at `a78dd00` maps these outcomes through one typed `0/1/2` boundary and passes focused/full local gates, three exact reviews, and all eight public checks; `test` remains analysis-only and `run_aero_program` passes through arbitrary program exits | Preserve the typed CLI-owned status contract; delegated exits, atomic rollback, command maturity, and helper architecture remain separate | CONTROLLED — CLI-owned status boundary closed at `a78dd00` |
| R-014 | Quick Start and flagship examples fail new-user workflows | HIGH | MEDIUM | No root Cargo manifest; flagship uses unsupported syntax/absent packages | Run docs as CI programs or label conceptual; correct manifest paths | OPEN |
| R-015 | Tracked compilation benchmark reports successful non-compilations | HIGH | HIGH | `AUDIT-019` confirmed `performance_benchmark.py` timed the CLI's bare-source unknown-command route as success; accepted public `CORE-013` at `a78dd00` returns `2`, classifies exactly two compilation series invalid, splits historical lexer evidence, and preserves all artifacts; the shell harness remains simulated and no benchmark was run | Preserve fail-closed bare-source handling and invalid classifications; a separate protocol-complete benchmark repair/rerun remains required | CONTROLLED — false-success claim path closed; benchmark remains invalid |
| R-016 | Stable Rust/LLVM drift breaks reproducibility | MEDIUM | MEDIUM | CI tracks floating stable/nightly and no repository toolchain pin was found | Declare supported toolchains; capture lock/environment and platform gates | OPEN |
| R-017 | Registry install can escape its destination and publish omits package bytes | MEDIUM | CRITICAL | Accepted CORE-012 at `6780a23` guards every live function and CLI live branch before auth/I/O/transport while keeping local search and dry-run plans credential/network-free; focused/full gates, exact review, and all public CI checks pass | Preserve quarantine; later specify and adversarially test paths, payload, response, auth, overwrite, dependencies, and transport before separate re-enablement | CONTROLLED — live transport fail closed; protocol remains unimplemented |

## Priority order

1. Stop silent source corruption and invented semantics at phase boundaries.
2. Enforce the stable-core function/type contract before IR generation.
3. Make unsupported constructs fail explicitly until their full typed lowering is
   implemented.
4. Converge tooling on a canonical compiler pipeline and truthful status codes.
5. Reclassify public documentation/backends, then grow conformance and real
   execution evidence.

`AUDIT-018` at clean public head `8598a4c` ranks R-017 first because an active,
incomplete remote-input boundary can reach credentials, HTTP, and filesystem writes.
R-004 remains critical but stops on unfrozen ownership/provenance semantics spanning
more than two compiler phases. R-013/R-015 follow as the next tooling boundary;
R-011 currently rejects the reproduced mixed-array/index cases before artifact
publication, although too late in checked IR for the mixed numeric case. `CORE-012`
was preregistered as quarantine only, not as a registry protocol implementation.

Accepted `CORE-012` closes the active live-transport boundary without designing or
enabling a protocol. `AUDIT-019` at clean public head `b7bb429` discarded one invalid
argument-dropping probe, then reproduced broad zero-status usage/operational failures
with corrected explicit arguments. R-004 remains stopped on unfrozen multi-phase
ownership semantics; reproduced R-011 cases fail closed before output. Accepted
public `CORE-013` at `a78dd00` implements the bounded slice: one typed CLI-owned status
contract and evidence-preserving invalidation of compilation claims that depended on
the bare-source false success. Its focused/full local gates, three exact reviews, and
all eight public checks pass. Delegated program exits and non-atomic write rollback stay outside
the taxonomy. It does not repair or run a benchmark.
