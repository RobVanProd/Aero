# Aero Initial Risk Register

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

Likelihood and impact are `LOW`, `MEDIUM`, `HIGH`, or `CRITICAL`. Status remains
open until a regression test and the applicable full gate prove closure.

| ID | Risk | Likelihood | Impact | Evidence | Required control | Status |
|---|---|---|---|---|---|---|
| R-001 | Invalid characters/numbers/strings are silently changed into valid tokens | HIGH | CRITICAL | Trusted paths use strict lexing at `b988318`; legacy recovery remains public for compatibility and LSP symbol indexing only | Keep recovery output ineligible for semantics/artifacts; add fuzz/property coverage and eventual diagnostic-accumulating migration | CONTROLLED — trusted paths closed |
| R-002 | Calls, annotations, and returns violate declared type contracts | HIGH | CRITICAL | Monomorphic numeric/void function calls and returns are controlled at `8d5d8e7`; initialized exact numeric binding annotations are controlled at `bc9a148`; boolean/generic/composite, uninitialized, and non-numeric contracts remain unenforced or uncertified | Retain separate boundaries for uninitialized and richer types; define conversion and assignment policy before broadening | PARTIALLY CONTROLLED — numeric functions and initialized numeric bindings closed |
| R-003 | Unsupported expressions are accepted with invented integer/zero semantics | HIGH | CRITICAL | Semantic and IR fallback arms for aggregates, closures, methods, and borrows remain; `%` is fail-closed at `302211e`; tuple values are controlled at reviewed `cbbe049`; named fields are controlled at reviewed `4e10d479`; exact Match candidate `08e7c2c` was rejected because default trait method bodies bypass preflight and still reach artifact generation | Add the frozen syntax-only preflight funnel and tests for default trait bodies; retain separate boundaries for each remaining fabricated-zero family and unsupported comparisons; eventually make IR fallible | OPEN — modulo, tuple values, and named fields controlled; Match correction pending; other families remain |
| R-004 | Ownership claims exceed enforcement and permit dangling/aliased/moved values | HIGH | CRITICAL | Shallow move tracking, no lifetime provenance, mutable references considered `Copy` | Freeze ownership model; CFG/provenance checking; permanent compile-fail suite | OPEN |
| R-005 | Invalid programs pass semantics then panic, miscompile, or produce invalid LLVM | HIGH | CRITICAL | Fatal parser propagation is controlled at `6ce85922`; numeric scalar/callable scope isolation is controlled at `bc9a148`; modulo is fail-closed at `302211e`; unreachable statements, original-AST phase crossing, infallible IR, backend fallbacks, and absent verification remain | Typed representation and verifier gates; fail-closed unsupported expressions; isolate unreachable-CFG cleanup | OPEN — parser/local epilogues, tested lexical scopes, and modulo controlled |
| R-006 | CLI and library compile through divergent module instances | HIGH | HIGH | Both `lib.rs` and `main.rs` declare overlapping compiler modules | Make library canonical; characterize behavior before thin-CLI refactor | OPEN |
| R-007 | Backend labels are mistaken for device execution | HIGH | HIGH | ROCm stops at object generation; CUDA run says unimplemented; README advertises broad run/graph/quantization surfaces | Independent backend status, explicit artifact/execution telemetry and hardware tests | OPEN |
| R-008 | Public 1.0/formal/safety messaging outruns evidence | HIGH | HIGH | `1.0.0` docs/CLI versus package `0.3.0`; three conformance cases and deterministic repetition | Unified version policy and evidence-based stability/claims language | OPEN |
| R-009 | Source ranges and recovery cannot support trustworthy diagnostics | HIGH | HIGH | Token start points only, no AST spans, one-character LSP ranges, recovery consumes valid code | End-to-end span model and recovery-retention tests | OPEN |
| R-010 | Grammar, tutorials, examples, and implementation define incompatible languages | HIGH | HIGH | Keyword/literal/field/rebinding/lifetime/top-level discrepancies | Freeze authoritative grammar subset; executable documentation examples | OPEN |
| R-011 | Aggregate and array lowering changes types or crashes | HIGH | HIGH | Only first array element checked; double storage and float-index truncation; composite fallback lowering | Homogeneity/index tests and typed aggregate IR before execution claims | OPEN |
| R-012 | Dormant, duplicated, or ignored tests create false coverage confidence | HIGH | HIGH | 78 duplicate active names, 38 ignored phase-5 tests, 299 dormant source tests, no key negative/fuzz suites | Classify backlog; restore valid tests deliberately; report distinct coverage and gates | OPEN |
| R-013 | User-facing commands report success without promised behavior | HIGH | HIGH | Compiler-oriented failures are nonzero at `6ce85922`, but `test` remains analysis-only, some usage/read branches return normally, and `run_aero_program` exits internally | Shared typed status contract and command maturity classification | OPEN — compiler failures partially controlled |
| R-014 | Quick Start and flagship examples fail new-user workflows | HIGH | MEDIUM | No root Cargo manifest; flagship uses unsupported syntax/absent packages | Run docs as CI programs or label conceptual; correct manifest paths | OPEN |
| R-015 | Tracked compilation benchmark reports successful non-compilations | HIGH | HIGH | Harness passes source as CLI command; unknown-command path exits zero and is timed | Quarantine affected claims, fix command/status/output correctness gates, rerun under protocol | OPEN |
| R-016 | Stable Rust/LLVM drift breaks reproducibility | MEDIUM | MEDIUM | CI tracks floating stable/nightly and no repository toolchain pin was found | Declare supported toolchains; capture lock/environment and platform gates | OPEN |
| R-017 | Registry install can escape its destination and publish omits package bytes | MEDIUM | CRITICAL | Untrusted resolved name/version are joined without containment; publish payload contains metadata only | Keep live operations disabled; validate paths/payload/response/auth transport with adversarial tests | OPEN |

## Priority order

1. Stop silent source corruption and invented semantics at phase boundaries.
2. Enforce the stable-core function/type contract before IR generation.
3. Make unsupported constructs fail explicitly until their full typed lowering is
   implemented.
4. Converge tooling on a canonical compiler pipeline and truthful status codes.
5. Reclassify public documentation/backends, then grow conformance and real
   execution evidence.

The first implementation task will be selected after all eight audit reports are
complete. Selection favors the smallest change that closes a critical acceptance
gap without choosing unresolved language semantics.
