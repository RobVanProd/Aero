# Aero Initial Risk Register

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

Likelihood and impact are `LOW`, `MEDIUM`, `HIGH`, or `CRITICAL`. Status remains
open until a regression test and the applicable full gate prove closure.

| ID | Risk | Likelihood | Impact | Evidence | Required control | Status |
|---|---|---|---|---|---|---|
| R-001 | Invalid characters/numbers/strings are silently changed into valid tokens | HIGH | CRITICAL | Trusted paths use strict lexing at `b988318`; legacy recovery remains public for compatibility and LSP symbol indexing only | Keep recovery output ineligible for semantics/artifacts; add fuzz/property coverage and eventual diagnostic-accumulating migration | CONTROLLED — trusted paths closed |
| R-002 | Calls, annotations, and returns violate declared type contracts | HIGH | CRITICAL | Monomorphic numeric/void function calls and returns are controlled at `8d5d8e7`; initialized exact numeric annotations are controlled at `bc9a148`; accepted `CORE-015` at `5d7aae0` closes four reproduced bool/canonical-String/nonempty-flat-fixed-numeric-array false successes and moves selected numeric-array checks before IR | Preserve accepted exact controls and quarantine lowercase string, custom/contextual/structural annotations, empty/nonnumeric arrays, and generic-scope annotation/array behavior until separately specified | PARTIALLY CONTROLLED — selected active false successes closed; excluded and generic-scope gaps remain open |
| R-003 | Unsupported expressions are accepted with invented integer/zero semantics | HIGH | CRITICAL | `%`, tuple values, named fields, Match, and StructLiteral fail closed at their reviewed boundaries; accepted `CORE-010` at `db349ef` adds generic checked-IR rejection for every unadmitted trusted-path fallback, including ordinary MethodCall, enum construction, and Deref/Borrow | Retain checked admission on every trusted caller; keep non-deprecated raw `generate_ir` and deprecated `generate_code` ineligible as trusted public boundaries, while permitting only checked-wrapper reuse followed by verification; define aggregate/ownership/method semantics before implementation | CONTROLLED — trusted checked compiler paths no longer fabricate scalar values; direct public unchecked compatibility use remains uncertified |
| R-004 | Ownership claims exceed enforcement and permit dangling/aliased/moved values | HIGH | CRITICAL | Shallow move tracking, no lifetime provenance, mutable references considered `Copy` | Freeze ownership model; CFG/provenance checking; permanent compile-fail suite | OPEN |
| R-005 | Invalid programs pass semantics then panic, miscompile, or produce invalid LLVM | HIGH | CRITICAL | Accepted `CORE-010` at `db349ef` provides checked logical IR admission, mandatory in-process verification, exhaustive checked codegen errors, and qualified final LLVM 22 verification across trusted callers; focused contracts, full gate, and public CI pass | Preserve mandatory checked APIs/verifiers, deprecate/restrict then retire public unchecked compatibility APIs at a major boundary, and extend typed negative evidence as language forms become admitted | PARTIALLY CONTROLLED — trusted checked scalar IR and externally verified publication routes are controlled; CLI `InternalOnly` and library-returned LLVM without external verification, broader language semantics, and public unchecked APIs remain uncertified |
| R-006 | CLI and library compile through divergent module instances | HIGH | HIGH | Accepted `CORE-011` at `a711dd5` centralizes direct-module collection across both compiled trees, rejects source-only/nested declarations, and includes exact module sources in pre-lookup cache identity; focused/full local gates, exact review, and public CI pass | Preserve the shared collector, then separately design canonical library/thin-CLI orchestration and `CompilerOptions` | PARTIALLY CONTROLLED — direct-module source-set boundary is closed; duplicated orchestration and full pipeline convergence remain open |
| R-007 | Backend labels are mistaken for device execution | HIGH | HIGH | ROCm stops at object generation; CUDA run says unimplemented; README advertises broad run/graph/quantization surfaces | Independent backend status, explicit artifact/execution telemetry and hardware tests | OPEN |
| R-008 | Public 1.0/formal/safety messaging outruns evidence | HIGH | HIGH | `AUDIT-022` at `c612f3b` reproduces package `0.3.0` versus CLI/banner `1.0.0`; reviewed public red `4b94dbd` reproduces exactly 2 preservation passes / 5 claim failures; the local implementation candidate derives CLI presentation from package metadata, classifies three cases/four repetitions as regression evidence, and visibly bounds generic/ownership/safety documents; focused 7/7 claim and 7/7 CLI targets plus exact full gate pass | Preserve manifest-derived CLI implementation version; distinguish the v1.0.0 language design target; keep conformance compatibility schema/counts unchanged; retain visible design/history qualifications and unsupported safety/type boundaries | CANDIDATE CONTROLLED — local implementation is green but R-008 remains open until exact three-review approval, public green CI, and acceptance closure; no version/release change authorized |
| R-009 | Source ranges and recovery cannot support trustworthy diagnostics | HIGH | HIGH | Token start points only, no AST spans, one-character LSP ranges, recovery consumes valid code | End-to-end span model and recovery-retention tests | OPEN |
| R-010 | Grammar, tutorials, examples, and implementation define incompatible languages | HIGH | HIGH | Keyword/literal/field/rebinding/lifetime/top-level discrepancies | Freeze authoritative grammar subset; executable documentation examples | OPEN |
| R-011 | Aggregate and array lowering changes types or crashes | HIGH | HIGH | Accepted `CORE-015` at `5d7aae0` enforces selected numeric-array homogeneity/count/integer-index contracts before IR; mixed numeric and float-index cases now fail in semantics without artifacts | Preserve selected pre-IR controls; specify typed aggregate IR, bounds, mutation, layout, and execution separately | PARTIALLY CONTROLLED — selected phase-order subset closed; aggregate execution remains open |
| R-012 | Dormant, duplicated, or ignored tests create false coverage confidence | HIGH | HIGH | At `c612f3b`, 38 phase-5 tests remain ignored; an explicit ignored run passes 36 and fails 2, but recovery parsing and unsupported ownership/struct behavior make blind activation unsound; Cargo also compiles overlapping library/binary test names | Classify each ignored test against current syntax and frozen behavior before activation; report target entries, overlap, distinct evidence, and gates separately | OPEN |
| R-013 | User-facing commands report success without promised behavior | HIGH | HIGH | Corrected `AUDIT-019` probes at `b7bb429` reproduced status zero across no/unknown command, malformed usage, missing inputs, registry/conformance errors, and the bare benchmark source path; accepted public `CORE-013` at `a78dd00` maps these outcomes through one typed `0/1/2` boundary and passes focused/full local gates, three exact reviews, and all eight public checks; `test` remains analysis-only and `run_aero_program` passes through arbitrary program exits | Preserve the typed CLI-owned status contract; delegated exits, atomic rollback, command maturity, and helper architecture remain separate | CONTROLLED — CLI-owned status boundary closed at `a78dd00` |
| R-014 | Quick Start and flagship examples fail new-user workflows | HIGH | MEDIUM | `AUDIT-020` at `18526ff` reproduces root `cargo build --release` exit 101, the wrong root binary path, and the unsupported `aeronum`/`aeronn` flagship; public red checkpoint `fc77e99` reproduces the three frozen gaps; accepted public `c56b1d5` passes the focused/full local gates, three exact reviews, and all eight public checks, including the exact generated-project path and anchored output in stable Linux CI | Preserve the exact minimal generated-project tests/CI path, manifest/binary paths, verifier prerequisites, and capability qualifications | CONTROLLED — generated-project Quick Start accepted at `c56b1d5` |
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

`AUDIT-020` at clean public acceptance head `18526ff` re-ranks the remaining work.
R-004, R-006, R-009, R-010, R-011, and R-012 require unfrozen semantics or more than
two phases; R-007 requires unavailable device evidence; R-008 requires an explicit
version/stability policy; and R-016 is lower likelihood/impact. R-014 is the smallest
externally visible bounded correction: `AUDIT-020` reproduced status 101 from the
former documented root build, while the corrected manifest-qualified build and
existing generated-project init/check path pass. `CORE-014` is documentation/test/
workflow-only and does not authorize language, compiler, backend, version, packaging,
or release changes. Accepted public `CORE-014` at `c56b1d5` implements that bounded
control: three exact reviews and all eight public checks pass, and stable Linux CI
executes the documented generated-project path through external LLVM 22 verification
with status zero and exactly one anchored `Output: Hello, Aero!` line. This closes
R-014 only for that generated CPU project; it does not establish Windows end-to-end,
accelerator, language-wide tutorial, version, packaging, release, or benchmark
evidence.

`AUDIT-021` at clean accepted public head `1535ce2` supersedes the next-slice
ranking after R-014 closure. R-004 remains the highest conceptual safety risk but
cannot be credibly changed without a frozen ownership/provenance model spanning more
than two phases. R-005 trusted publication paths remain controlled; public unchecked
API retirement needs a major-boundary policy. R-002 is the highest-severity active
false success: five reproduced initialized annotation mismatches pass check/build
and create LLVM artifacts. `CORE-015` selects four by preserving existing numeric
scalar enforcement and, outside active semantic generic scopes, adding exact bool/
canonical-String/nonempty-flat-fixed-numeric-array equality. The custom-name case, lowercase
`string`, contextual/structural forms, nonnumeric arrays, and new generic-scope
annotation/array behavior remain open under quarantine controls. The slice adds
non-generic numeric-array element/count/index checks in semantics and checked IR only. It does not select
conversions, assignment, generic/reference/tuple semantics, ownership, aggregate
layout, backend, version, or release behavior.

`AUDIT-022` at clean accepted public head `c612f3b` follows the accepted
`CORE-015` final-state sync. R-004 remains the highest conceptual safety risk but
still needs an unfrozen ownership/provenance model across more than two compiler
phases. Residual R-002 custom/contextual annotation enforcement is stopped because
an arbitrary name can denote an unresolved nominal type or an in-scope generic and
requires a separately frozen name-resolution/substitution contract. Remaining R-011
aggregate execution needs typed aggregate IR, bounds, layout, and backend work;
R-012 requires test-by-test recovery/stub classification rather than bulk activation.
R-006, R-009, and R-010 are broader architectural or cross-phase work; R-007 needs
unavailable device evidence; R-016 is lower likelihood/impact. R-008 is the highest
bounded active public false claim: the
package is `0.3.0` while both CLI version routes and the no-command banner say
`1.0.0`; three example conformance cases and four deterministic repetitions are
presented as formal/mechanized proof; and current-facing documents present
unverified type/ownership safety as enforced. `CORE-016` selects CLI presentation
plus explicit documentation classification only. It does not change the package
version, language semantics, JSON field names, conformance algorithms, backend,
release, registry, benchmark, or `master`.

Reviewed public red commit `4b94dbd` now proves exactly two preservation passes and
five bounded claim failures; both compiler-test jobs and nightly Rust reproduce the
new target while CodeQL remains green. The local `CORE-016` implementation candidate
passes the focused 7/7 claim contract, complete 7/7 CLI status contract, and exact
repository gate. R-008 remains open pending exact implementation review, public green
CI, and closure evidence; no stronger language, safety, conformance, or release claim
is inferred.
