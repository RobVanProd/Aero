# Aero Initial Risk Register

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

Likelihood and impact are `LOW`, `MEDIUM`, `HIGH`, or `CRITICAL`. Status remains
open until a regression test and the applicable full gate prove closure.

| ID | Risk | Likelihood | Impact | Evidence | Required control | Status |
|---|---|---|---|---|---|---|
| R-001 | Invalid characters/numbers/strings are silently changed into valid tokens | HIGH | CRITICAL | Trusted paths use strict lexing at `b988318`; legacy recovery remains public for compatibility and LSP symbol indexing only | Keep recovery output ineligible for semantics/artifacts; add fuzz/property coverage and eventual diagnostic-accumulating migration | CONTROLLED — trusted paths closed |
| R-002 | Calls, annotations, and returns violate declared type contracts | HIGH | CRITICAL | Monomorphic numeric/void function calls and returns are controlled at `8d5d8e7`; initialized exact numeric annotations are controlled at `bc9a148`; accepted `CORE-015` at `5d7aae0` closes selected binding/array false successes; accepted `CORE-023` at `67ccdf2` closes exact Boolean contracts for monomorphic non-entry helpers; accepted `CORE-025` at `1ec8beb` rejects initialized exact outer tuple binding annotations in semantics and checked admission before generation; accepted `CORE-028` at `e051452` rejects exact valueless outer tuple binding annotations at those same trusted boundaries; accepted `CORE-029` at `29bd2e0` rejects exact valueless immediate reference-to-tuple annotations there; accepted `CORE-030` at `97c0f04` rejects exact valueless immediate array-of-tuple annotations there; accepted `CORE-031` at `4bc7a345` rejects exact valueless immediate array-of-array-of-tuple annotations there; accepted `CORE-032` at `30d0d730` rejects exact initialized immediate array-of-tuple annotations after initializer validation; accepted `CORE-033` at `76a6e802` rejects exact initialized immediate array-of-array-of-tuple annotations after initializer validation | Preserve accepted exact controls and quarantine Boolean entry/ABI, lowercase string, custom/contextual/structural annotations, empty/nonnumeric arrays, other uninitialized annotations, other unsupported nested tuple shapes, tuple type/value support, and remaining generic-scope behavior until separately specified | PARTIALLY CONTROLLED — selected active false successes, including the exact initialized two-array-deep tuple fallback, closed; entry, excluded-type, tuple-support, and generic-scope gaps remain open |
| R-003 | Unsupported expressions are accepted with invented integer/zero semantics | HIGH | CRITICAL | `%`, tuple values, named fields, Match, and StructLiteral fail closed at their reviewed boundaries; accepted `CORE-010` at `db349ef` adds generic checked-IR rejection for every unadmitted trusted-path fallback, including ordinary MethodCall, enum construction, and Deref/Borrow | Retain checked admission on every trusted caller; keep non-deprecated raw `generate_ir` and deprecated `generate_code` ineligible as trusted public boundaries, while permitting only checked-wrapper reuse followed by verification; define aggregate/ownership/method semantics before implementation | CONTROLLED — trusted checked compiler paths no longer fabricate scalar values; direct public unchecked compatibility use remains uncertified |
| R-004 | Ownership claims exceed enforcement and permit dangling/aliased/moved values | HIGH | CRITICAL | Shallow move tracking, no lifetime provenance, mutable references considered `Copy` | Freeze ownership model; CFG/provenance checking; permanent compile-fail suite | OPEN |
| R-005 | Invalid programs pass semantics then panic, miscompile, or produce invalid LLVM | HIGH | CRITICAL | Accepted `CORE-010` at `db349ef` provides checked logical IR admission, mandatory in-process verification, exhaustive checked codegen errors, and qualified final LLVM 22 verification across trusted callers; focused contracts, full gate, and public CI pass | Preserve mandatory checked APIs/verifiers, deprecate/restrict then retire public unchecked compatibility APIs at a major boundary, and extend typed negative evidence as language forms become admitted | PARTIALLY CONTROLLED — trusted checked scalar IR and externally verified publication routes are controlled; CLI `InternalOnly` and library-returned LLVM without external verification, broader language semantics, and public unchecked APIs remain uncertified |
| R-006 | CLI and library compile through divergent module instances | HIGH | HIGH | Accepted `CORE-011` at `a711dd5` centralizes direct-module collection; public red `037f44d` proves ignored nondefaults; implementation `70cb0ad` and record-only closure `5a8cd06` pass all eight checks with pre-lexing rejection and byte-exact default preservation | Preserve the shared collector, default path, and fail-closed option boundary; separately design canonical library/thin-CLI orchestration and real option semantics | PARTIALLY CONTROLLED — direct-module and ignored-option boundaries are closed; duplicated orchestration, option meanings, and full convergence remain open |
| R-007 | Backend labels are mistaken for device execution | HIGH | HIGH | `AUDIT-024` at `9ddc571` proved the false success; tests-only `427fb4c` reproduced the public red boundary; exact three-review-approved implementation `8bde0ff` and record-only closure `2e0e17f` pass their full gates and all eight public checks with fail-closed ROCm/CUDA, explicit targets, non-device telemetry, and the Aero GGUF route disabled | Preserve the accepted false-success controls; require separate hardware execution/correctness gates before any device claim | PARTIALLY CONTROLLED — selected object-only/current-claim boundary accepted at `2e0e17f`; no Aero device evidence |
| R-008 | Public 1.0/formal/safety messaging outruns evidence | HIGH | HIGH | `AUDIT-022` reproduced the mismatch; reviewed public red `4b94dbd` bound exactly 2 preservation passes / 5 claim failures; exact three-review-approved implementation `cc984d0` derives CLI presentation from package metadata and bounds conformance/design/history claims; exact three-review-approved record-only closure `ea036f2` passes all eight public checks | Preserve manifest-derived CLI implementation version; distinguish the v1.0.0 language design target; keep conformance compatibility schema/counts unchanged; retain visible design/history qualifications and unsupported safety/type boundaries | CONTROLLED — selected public false-claim boundary accepted at `ea036f2`; no version/release or underlying language-safety capability is inferred |
| R-009 | Source ranges and recovery cannot support trustworthy diagnostics | HIGH | HIGH | Accepted `CORE-024` implementation `a3d110e` corrects parser start-column UTF-16 projection at the LSP boundary; token start points, no AST spans, synthetic one-character ranges, and recovery-retention gaps remain | Preserve the accepted adapter; require an end-to-end span model and recovery-retention tests before broader claims | OPEN — selected parser UTF-16 adapter controlled; trustworthy ranges remain open |
| R-010 | Grammar, tutorials, examples, and implementation define incompatible languages | HIGH | HIGH | Keyword/literal/field/rebinding/lifetime/top-level discrepancies | Freeze authoritative grammar subset; executable documentation examples | OPEN |
| R-011 | Aggregate and array lowering changes types or crashes | HIGH | HIGH | Accepted `CORE-015` at `5d7aae0` enforces selected numeric-array homogeneity/count/integer-index contracts before IR; mixed numeric and float-index cases now fail in semantics without artifacts | Preserve selected pre-IR controls; specify typed aggregate IR, bounds, mutation, layout, and execution separately | PARTIALLY CONTROLLED — selected phase-order subset closed; aggregate execution remains open |
| R-012 | Dormant, duplicated, or ignored tests create false coverage confidence | HIGH | HIGH | `AUDIT-023` classified 38 ignored tests; public-green preregistration `2c61535` selected a strict 22/16 split; exact three-review-approved implementation `8be8c21` has 4 strict lexer + 18 strict parser-retention passes, 16 explicit quarantines, 38 listed entries, exact full-gate success, and no production change; exact three-review-approved record-only closure `3dd3bb4` passes all eight public checks; Cargo overlap and 299 dormant tests remain | Preserve exact strict token/retained-AST assertions and the 16-test quarantine; report target entries, overlap, distinct evidence, and gates separately | PARTIALLY CONTROLLED — selected Phase 5 syntax-evidence classification accepted at `3dd3bb4`; 299 dormant tests, Cargo overlap, and every semantic capability remain open |
| R-013 | User-facing commands report success without promised behavior | HIGH | HIGH | Accepted `CORE-013` controls typed CLI statuses; accepted `CORE-019` controls analysis-only `aero test` wording; accepted `CORE-021` at closure `b99e445` controls delegated nonzero success wording; accepted `CORE-022` implementation `2a42324` controls final-entry init preflight before writes | Preserve accepted status/wording/preflight controls; keep general rollback/atomicity/race freedom, ancestor-symlink policy, executable-test design, command maturity, and helper architecture separate | PARTIALLY CONTROLLED — selected status, presentation, and dangling-entry init slices accepted; other command boundaries remain open |
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
new target while CodeQL remains green. The subsequent `CORE-016` implementation
passes the focused 7/7 claim contract, complete 7/7 CLI status contract, and exact
repository gate. R-008 remains open pending exact implementation review, public green
CI, and closure evidence; no stronger language, safety, conformance, or release claim
is inferred. Exact three-review-approved implementation `cc984d0` and record-only
closure `ea036f2` are public and all eight checks pass at each. R-008 is controlled
only for the selected public-claim boundary; R-004 ownership enforcement, R-007
backend execution evidence, and all other excluded capability risks remain open.

`AUDIT-023` classified the 38 ignored Phase 5 entries rather than counting their broad
recovery-path outcomes as coverage. Exact three-review-approved `CORE-017`
implementation `8be8c21` now runs 4 strict lexer and 18 strict parser-retention tests,
keeps 14 semantic and 2 generic-impl tests explicitly quarantined, passes the exact
repository gate, and passes all eight public checks without production changes. R-012
is partially controlled only for this selected classification after exact three-review-
approved record-only closure `3dd3bb4` also passed all eight public checks. The 299
dormant tests and library/binary Cargo overlap remain open residual evidence risks,
and no semantic capability is inferred.

`AUDIT-024` at clean accepted public head `9ddc571` re-ranks R-007 as the highest
bounded active false success. All three independent auditors traced ROCm `run` from
required LLVM verification through temporary AMDGPU `llc` invocation to status zero
without an object-existence check, link, HIP launcher, device transfer,
synchronization, or execution. They also confirmed that `gpu` is a heuristic rather
than device proof, graph/quantization are externally verified scalar-helper textual
transforms, and the immutable claim records already keep the sole real GGUF run
external to Aero. `CORE-018` selects fail-closed status/postcondition behavior and
current claim reclassification only. R-007 remains OPEN until independent Aero
hardware correctness and execution evidence satisfies `BACKEND_STATUS.md`.

The exact tests-only checkpoint was published as `427fb4c`: both compiler jobs and
stable/nightly failed as prescribed while all CodeQL checks passed. The local
implementation candidate turns CLI 10/10 and claims 7/7 green without hardware,
numerical, benchmark, schema, dependency, workflow, or immutable-evidence changes.
Exact three-review-approved implementation `8bde0ff` passes the complete gate and all
eight public checks. Exact three-review-approved record-only closure `2e0e17f` also
passes all eight checks. The selected false-success correction is accepted at that
public closure. R-007 remains open for real Aero accelerator execution and correctness
evidence; no hardware, numerical, or performance capability is inferred.

`AUDIT-025` at clean accepted public head `d0bd54e` re-ranks the next bounded work.
R-002/R-004 remain higher raw safety risks but need unfrozen type/ownership semantics;
R-005 needs a major-boundary unchecked-API policy; R-007 needs hardware; R-009/R-010
and broad R-006 convergence are architectural; R-011 needs bounds/layout/execution;
R-012 remains a bounded-per-slice backlog; and R-016 needs a toolchain policy. The
documented `aero test` command is the highest-reach bounded active claim defect: it
performs read/parse/direct-module/semantic analysis only while presenting files as
running and passed. `CORE-019` selects exact presentation and tests only. Ignored
nondefault `CompilerOptions` remain the next bounded runner-up under R-006.

`AUDIT-026` at public-green preregistration `2c61ff9` completes that clean-head
comparison. All three independent auditors rank the ignored public options as the
best bounded active false-success correction: 62 in-repository calls were default-only,
all field combinations traversed the same checked compiler path at that audited head,
and CLI targets were separate. Lead-owned DEC-025 accepts the explicit compatibility change
from silent nondefault success to one pre-lexing error while preserving the public
facade and exact default behavior. At preregistration, R-006 remained only partially
controlled pending tests-first, implementation, full-gate, exact-review, and public
evidence. Broad pipeline convergence remained out of scope.

Exact three-review-approved `fae1374` publishes that audit closure and DEC-025 with
all eight checks green. Exact three-review-approved tests-only `037f44d` then proves
the public 1/1 red split in both compiler runs and nightly Rust, with stable cancelled
during tests by fail-fast and all four CodeQL checks green. The local one-guard
implementation is focused 2/2, preservation 40/40, and complete-gate green. R-006
remained partially controlled pending exact implementation review and public green;
no option meaning or broad pipeline convergence was inferred at that checkpoint.

Exact three-review-approved implementation `70cb0ad` passes focused 2/2,
preservation 40/40, the exact complete local gate, and all eight public checks in
compiler runs `30834445685`/`30834446600`, Rust `30834446605`, CodeQL
`30834443841`, and aggregate `91756251121`. The selected ignored-option boundary is
controlled. R-006 remains partially controlled because public options still have no
meaning and the CLI/library compiler orchestration remains duplicated.

Exact three-review-approved record-only closure `5a8cd06`, tree `df4a04a`, diff
`85ef52a4`, passes compiler runs `30835593703`/`30835597576`, stable/nightly Rust
run `30835597620`, CodeQL run `30835594365`, and aggregate `91759990615`. It closes
the evidence loop without changing risk severity or residual status. `AUDIT-027`
must now compare the remaining open risks from a clean public head before the lead
selects another implementation.

`AUDIT-027` at public-green basis `aa3e7a8` compares every OPEN or PARTIALLY
CONTROLLED risk. All three auditors rank R-013 first. The reconciled A/B/C comparison
selects nonzero delegated CPU success wording over the lower-reach dangling-entry
`init` gap and the architecture-only hidden helper exit. DEC-026 and `CORE-021`
freeze a one-branch presentation correction. Exact three-review-approved tests-only
`0873f65` publicly reproduces the boundary in compiler `30839264536` /
`30839272375` and nightly Rust `30839272429`; stable is fail-fast cancelled during
tests, while CodeQL `30839264268` and aggregate `91772180985` pass. Exact reviewed
implementation `a4327be` passes compiler `30839860335` / `30839862442`, Rust
`30839862423`, CodeQL `30839859840`, and aggregate `91774125621`; the selected
presentation boundary is accepted. R-013 remains partially controlled because the
unselected command boundaries and all other risks remain open.

Corrected exact record-only closure `b99e445`, tree `8a4c2d77`, diff `5abbf3a7`,
passes compiler `30840427466` / `30840426655`, stable/nightly Rust `30840428215`,
CodeQL `30840415565`, and aggregate `91775938704`. `AUDIT-028` is preregistered to
compare the complete remaining set R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/
R-012/R-013/R-016 without inherited ordering or implementation authority.

`AUDIT-028` at public-green basis `399e04f` ranks all eleven residuals. The three top
threes are R-011/R-013/R-002, R-013/R-012/R-002, and R-002/R-013/R-010. The lead
selects R-013's universal-top-two dangling-entry `aero init` preflight because its
existing no-overwrite policy freezes the answer, it changes zero compiler phases, and
the existing Unix fixture is deterministic. R-011 is stopped on compile-error versus
trap/unchecked bounds semantics; R-002 remains the wider runner-up. R-013 remains
partially controlled. Triple-reviewed tests-only `7cd8aba` reproduces exact Linux
compiler 10/1 in `30843119793` / `30843125522` and nightly Rust `30843124314` while
CodeQL remains green. Triple-reviewed implementation `2a42324` passes focused/local
gates and all eight public checks in compiler `30843592298` / `30843592784`, Rust
`30843595560`, CodeQL `30843589175`, and aggregate `91786468184`. The selected
final-entry preflight is accepted. Exact triple-reviewed record closure `aa29a00`
passes compiler `30844324249` / `30844328660`, Rust `30844328850`, CodeQL
`30844325051`, and aggregate `91788926688`. Other R-013 boundaries remain open, and
all unselected residuals retain their prior status.

Exact triple-reviewed status synchronization `21153f3`, tree `d667ce37`, diff
`c69c5a1e`, passes compiler `30844798322` / `30844802332`, Rust `30844802044`,
CodeQL `30844799426`, and aggregate `91790481511`. `AUDIT-029` is preregistered to
rank the complete remaining set R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/
R-012/R-013/R-016 from a clean head, separating every accepted sub-slice from its
open residual and prohibiting implementation until separate reconciliation and a
frozen task contract.

`AUDIT-029` is complete from exact public-green basis `0e5cba1`, tree `6ac88db4`.
The three full rankings select R-002 Boolean helper contracts, R-010 grammar claim
containment, and R-009 parser UTF-16 columns respectively; all rank R-012 second.
The lead selects R-002's distinct Boolean helper boundary: semantics currently omits
Boolean function contracts, so direct analysis accepts invalid Boolean calls/returns
and infers valid Boolean calls as `Int`, while checked IR already admits exact
Boolean/LLVM-`i1` signatures. `CORE-023` preregisters one semantic phase only for
monomorphic non-entry helpers. R-002 remains PARTIALLY CONTROLLED; custom,
contextual, structural, generic, aggregate, String, reference, method, closure,
coercion, entry-point, and ABI behavior remain excluded, as do every other residual's
recorded stops.

Accepted `CORE-023` now controls the selected R-002 Boolean-helper boundary.
Preregistration `1c28a7b` is all-eight green. Triple-reviewed tests-only `c3f6e90`
produces exact compiler 13/1 in `30848723940` / `30848725388` and nightly Rust
`30848725757`, with stable fail-fast cancelled and CodeQL/aggregate green. Triple-
reviewed implementation `67ccdf2`, tree `c0b538c9`, passes the focused regression,
function/binding/typed-IR preservation, the exact full gate, compiler
`30850000615` / `30850005598`, Rust `30850005670`, CodeQL `30850001251`, and
aggregate `91807553635`. R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED:
Boolean entry/ABI and every excluded String/custom/contextual/structural/generic/
composite/reference/closure/method/coercion/defaulting boundary remain open or
quarantined.

Exact triple-reviewed `CORE-023` record closure `0b88530`, tree `71ac4da7`, diff
`adba01a1`, passes compiler `30850519757` / `30850524194`, stable/nightly Rust
`30850524148`, CodeQL `30850520457`, and aggregate `91809289681`. R-002 remains
HIGH/CRITICAL and PARTIALLY CONTROLLED. `AUDIT-030` is preregistered to rank the
complete remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/
R-016 set from this clean head, excluding every accepted sub-slice and carrying no
implementation authority.

`AUDIT-030` is complete from exact public-green authorization `d4e3c75`, tree
`9a07c10c`, with compiler `30851275589` / `30851278460`, Rust `30851278586`,
CodeQL `30851276053`, and aggregate `91811764009`. All three independent rankings
place R-009 in their top three and two rank it first. `CORE-024` therefore selects
only parser-diagnostic UTF-16 start-column projection in the LSP layer. R-009 remains
OPEN: token/AST end spans, recovery retention, and trustworthy end-to-end ranges are
excluded. R-010 is the bounded runner-up; every other residual retains its recorded
semantic, architectural, policy, evidence, or hardware stop.

Triple-reviewed `CORE-024` tests-only `ab8508e` reproduces the selected R-009
defect as exact 148/149 in compiler `30853599874` / `30853602996` and stable/nightly
Rust `30853603035`, while CodeQL `30853601414` and aggregate `91819440238` pass.
Exact triple-reviewed one-file implementation `a3d110e`, tree `79ccfca1`, diff
`74bfbcea`, passes focused 1/1, LSP 10/10, the full local gate, compiler
`30854094706` / `30854099595`, Rust `30854099899`, CodeQL `30854094981`, and
aggregate `91821038577`. The selected parser-start UTF-16 adapter is controlled,
but R-009 remains HIGH/HIGH and OPEN for token/AST end spans, recovery retention,
and trustworthy end-to-end ranges.

Corrected exact `CORE-024` closure `226b7fb`, tree `1337945c`, diff `861b5ec3`,
passes compiler `30854853182` / `30854856449`, Rust `30854856190`, CodeQL
`30854853829`, and aggregate `91823492290` after three fresh approvals. R-009
remains HIGH/HIGH and OPEN. `AUDIT-031` is preregistered to rank the complete
remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set
from this clean head, excluding every accepted sub-slice and carrying no
implementation authority.

`AUDIT-031` is authorized and all-eight public green at `ba258c6`, tree `651762a8`,
with compiler `30855407928` / `30855410819`, Rust `30855410731`, CodeQL
`30855409113`, and aggregate `91825280915`. Three complete rankings and targeted
reconciliation select one distinct R-002 containment: initialized exact outer tuple
annotations currently disappear in both semantics and checked admission, allowing a
scalar RHS to reach generation. `CORE-025` preregisters rejection only, after child
validation and before binding insertion, across traversed statement contexts. R-002
remains HIGH/CRITICAL and PARTIALLY CONTROLLED; every tuple value/layout/ABI,
uninitialized/nested annotation, and other excluded type boundary stays open or
quarantined. R-010 is the bounded runner-up; all other stops remain.

Accepted `CORE-025` now controls only that selected R-002 false success.
Triple-reviewed preregistration `722d4d1` is all-eight green. Corrected
triple-reviewed tests-only `39ccd9c`, tree `5b05499f`, produces exactly 16 passed/1 failed in
compiler `30857467570` / `30857469931` and the nightly job in Rust `30857470046`;
stable is fail-fast cancelled, while CodeQL `30857468030` and aggregate
`91831822409` pass. Triple-reviewed implementation `1ec8beb`, tree `ac2c8fdd`,
passes focused 1/1, binding 17/17, the exact full gate, compiler `30857775577` /
`30857777431`, stable/nightly Rust `30857777314`, CodeQL `30857775231`, and
aggregate `91832840108`. Initialized exact outer tuple binding annotations now stop
after child validation in both semantic analysis and checked admission, before
binding insertion or generation. R-002 remains HIGH/CRITICAL and PARTIALLY
CONTROLLED: tuple values, uninitialized/nested annotations, every other excluded
type/shape, entry/ABI, and generic behavior remain open or quarantined.

Corrected exact `CORE-025` record closure `b0fe242`, tree `2a5d233f`, diff
`98916b4d`, passes compiler `30858384541` / `30858387195`, stable/nightly Rust
`30858387193`, CodeQL `30858385234`, and aggregate `91834740790` after three exact
approvals. R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED. `AUDIT-032` is
preregistered to rank the complete remaining R-002/R-004/R-005/R-006/R-007/R-009/
R-010/R-011/R-012/R-013/R-016 set from a clean public head, excluding every
accepted sub-slice and carrying no implementation authority.

`AUDIT-032` authorization `b6b1c63`, tree `c8803965`, is triple-approved and
all-eight public green. Independent rankings and unanimous targeted reconciliation
select a distinct R-005 phase-order defect above R-010: direct checked-AST too-few
and too-many calls to eligible known scalar top-level helpers generate raw IR before
verifier `CallArity` rejection. `CORE-026` preregisters only an exact-arity checked-
admission guard for nongeneric, non-entry scalar/Void signatures after existing
child, local-callable, and Void-use validation. R-005 remains HIGH/CRITICAL and
PARTIALLY CONTROLLED; unchecked APIs, argument typing, other signatures/callables,
IR/verifier/codegen/backend behavior, and all public capability claims remain open
or unchanged. R-010 is the runner-up and every other recorded stop remains.

The first `CORE-026` authorization snapshot was rejected at P2 before publication:
duplicate top-level identities and verifier-invalid/reserved function or parameter
signatures were not explicitly ineligible, so the new arity phase could mask their
existing verifier diagnostics. The corrected contract requires one declaration and
verifier-valid, unique, non-reserved signature symbols, with green preservation
controls. R-005 remains HIGH/CRITICAL and PARTIALLY CONTROLLED; no risk status or
capability changes from authorization text alone.

Corrected `CORE-026` authorization `7dc3eac` is triple-approved and all-eight public
green. A first tests-first snapshot was rejected before publication for caller order
and missing composite/reference result controls. Corrected triple-reviewed tests-only
`1538a3e`, tree `8f3cd8fb`, publicly reproduces exactly 6 passed/1 failed with only
the selected phase-order target red; CodeQL and aggregate remain green.

Triple-reviewed implementation `8c2b2ec`, tree `eabd8939`, passes focused 1/1,
checked-IR 7/7, the exact full local gate, compiler `30862232159` / `30862233829`,
stable/nightly Rust `30862233777`, CodeQL `30862232615`, and aggregate `91846586968`.
Known eligible scalar/Void direct checked-AST wrong-arity calls now stop at Admission
before raw IR, while child/local/Void precedence, malformed or duplicate signature
failures, valid programs, verifier defense, source semantics, codegen, ABI, and
backends remain unchanged. R-005 remains HIGH/CRITICAL and PARTIALLY CONTROLLED;
unchecked APIs, broader callable/type contracts, and every public capability claim
remain open or unchanged.

Corrected exact `CORE-026` record closure `0a940ea`, tree `6ec4c609`, diff
`4e1db178`, passes compiler `30862783787` / `30862786131`, stable/nightly Rust
`30862786150`, CodeQL `30862784231`, and aggregate `91848258218` after three exact
approvals. Superseded closure snapshot `615c00b9` was rejected before publication for
stale gate chronology. R-005 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.
`AUDIT-033` is preregistered to rank the complete remaining R-002/R-004/R-005/R-006/
R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from the clean public closure,
excluding every accepted sub-slice and carrying no implementation authority.

`AUDIT-033` authorization `544b1ba`, tree `cdc3a085`, is triple-approved and
all-eight public green. Independent rankings and final targeted reconciliation select
R-010 documentation-authority containment above a stopped R-005 argument-type
admission candidate whose child-type verifier completeness is not yet frozen.
`CORE-027` preregisters only leading v1-design/current-implementation notices in the
split grammar and core-features tutorial plus replacement of one unqualified grammar
authority sentence. Every production, example, compiler behavior, and capability
classification remains unchanged. R-010 remains HIGH/HIGH and OPEN.

Accepted `CORE-027` controls only that selected documentation-authority boundary.
Triple-reviewed authorization `3574704` is all-eight green. Triple-reviewed
tests-first `f57cf2e`, tree `8a99d994`, publicly reproduces exactly 7 passed/1
failed with only the new authority contract red; stable is fail-fast cancelled,
while CodeQL and aggregate pass. The first implementation snapshot `01615da` was
rejected at P2 before publication for normalizing the grammar's final newline.
Corrected triple-reviewed implementation `b3e7910`, tree `2728bbc6`, diff
`90e1c4b6`, preserves the original EOF representation and passes focused 1/1,
version-claim 8/8, the exact full gate, compiler `30865344667` / `30865346597`,
stable/nightly Rust `30865346602`, CodeQL `30865345043`, and aggregate
`91855955012`. Every production, example, compiler behavior, and capability
classification remains unchanged. R-010 remains HIGH/HIGH and OPEN: actual grammar
compatibility, executable examples, migration, and implementation convergence are
not established by this containment.

Exact `CORE-027` record closure `d649c2d`, tree `b5ad7ee2`, diff `d4281863`, passes
compiler `30865772404` / `30865775196`, stable/nightly Rust `30865775214`, CodeQL
`30865772793`, and aggregate `91857289172` after three exact approvals. R-010
remains HIGH/HIGH and OPEN. `AUDIT-034` is preregistered to rank the complete
remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set
from this clean closure, excluding every accepted sub-slice including CORE-027
authority containment and carrying no implementation authority.

Public-green read-only `AUDIT-034` authorization `45783af`, tree `f1baa457`, passes
compiler `30866227485` / `30866229553`, stable/nightly Rust `30866229554`, CodeQL
`30866227939`, and aggregate `91858665436` after three exact approvals. Complete
independent rankings and unanimous targeted reconciliation select a distinct R-002
public false success above the R-005 verifier-contained runner-up: exact outer tuple
annotations on valueless bindings silently become `Int`, pass checked admission, and
can become integer zero in raw generation. At selection time, DEC-033 and
preregistered `CORE-028` authorized records only until their own gates passed; the
later bounded contract selected rejection of that exact AST in semantics and checked
admission. Initialized tuple controls,
nested tuple shapes, other valueless annotations, tuple support, valid output, and
every backend/capability claim remain unchanged. R-002 remains HIGH/CRITICAL and
PARTIALLY CONTROLLED.

Accepted public `CORE-028` implementation `e051452`, tree `63985b2d`, diff
`79830403`, closes only the selected valueless exact outer-tuple fallback after
triple-reviewed public 16/1 red evidence. Semantics now rejects after same-scope
duplicate detection and before fake `Int`/insertion; checked admission independently
rejects before generation. Focused 1/1, binding 17/17, the exact full local gate,
compiler `30871337443` / `30871335738`, stable/nightly Rust `30871337440`, CodeQL
`30871336117`, and aggregate `91873866339` pass. Initialized and nested tuples,
other unsupported/valueless annotations, unchecked APIs, tuple support, valid-output
claims, and every backend/capability surface remain unchanged. R-002 stays
HIGH/CRITICAL and PARTIALLY CONTROLLED; a later separate `AUDIT-035` authorization
is still required before any new residual ranking.

The first `CORE-028` closure snapshot `a20548ec`, tree `8250ce11`, diff
`f0f181f9`, was rejected at P2 before publication for contradictory present-tense
authorization wording. Corrected snapshot `5cc3ccb8`, tree `2f935a66`, diff
`f11da400`, fixed that language but was also rejected at P2 before publication:
the canonical R-002 summary row still ended at CORE-025 and treated the now-closed
exact valueless outer-tuple case as residual. The current closure records CORE-028 in
that row and narrows the unresolved surface to other uninitialized annotations and
tuple annotations nested beneath non-tuple outer shapes. Likelihood, impact, and
PARTIALLY CONTROLLED status remain unchanged. A third snapshot `782bc8fb`, tree
`1914aaf7`, diff `e1962dbb`, contained the corrected row but was rejected at P2
before publication because the ledger described its fresh final-tree gate as both
passed and pending. The current closure records one unambiguous fresh exact-gate
result; this chronology correction changes no risk or capability classification.

Exact `CORE-028` record closure `032d0d0`, tree `443aacdc`, diff `93fce8ae`, passes
compiler `30872236535` / `30872238993`, stable/nightly Rust `30872239003`, CodeQL
`30872237025`, and aggregate `91876507154` after three exact approvals. R-002 remains
HIGH/CRITICAL and PARTIALLY CONTROLLED; no tuple capability or other risk status
changes. `AUDIT-035` is preregistered to rank the complete remaining R-002/R-004/
R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set only after its separate
exact local/review/public gates, excluding every accepted sub-slice including
CORE-028 and carrying no implementation or capability authority.

Corrected read-only `AUDIT-035` authorization `f1cd972`, tree `b9c6270b`, diff
`7f221d2a`, passes compiler `30872922468` / `30872923806`, stable/nightly Rust
`30872923874`, CodeQL `30872922858`, and aggregate `91878491979` after three exact
approvals. Complete independent rankings initially split between exact R-002 and
R-005 candidates. Unanimous targeted reconciliation selects the distinct R-002
valueless immediate reference-to-tuple false success: the unsupported annotation
becomes `Ty::Int`, passes checked admission, and can become verifier-valid integer
zero in raw generation, while the R-005 runner-up is already verifier-contained
before LLVM. CORE-029 preregisters only two exact rejection guards. All other
uninitialized/nested annotations, tuple/reference/ownership semantics, valid output,
and every backend/capability claim remain unchanged. R-002 remains HIGH/CRITICAL and
PARTIALLY CONTROLLED.

Accepted public `CORE-029` implementation `29bd2e0`, tree `53282149`, diff
`acc1c247`, closes only the selected valueless immediate reference-to-tuple fallback
after corrected triple-reviewed public 17/18 red evidence. Semantics rejects after
same-scope duplicate detection and before fake `Int`/insertion; checked admission
independently rejects before raw generation. Focused 1/1, binding 18/18, formatting,
the exact full local gate, compiler `30875100237` / `30875102914`, stable/nightly
Rust `30875102909`, CodeQL `30875100762`, and aggregate `91884963697` pass.
Initialized and deeper reference forms, scalar references, arrays/generics, other
unsupported annotations, unchecked APIs, tuple/reference/ownership support, valid-
output claims, and every backend/capability surface remain unchanged. R-002 stays
HIGH/CRITICAL and PARTIALLY CONTROLLED; at that implementation head, record closure
was still required before any new residual ranking or implementation authorization.

Exact `CORE-029` record closure `7222b9a`, tree `66084b36`, diff `90bf540c`, passes
compiler `30876033717` / `30876035730`, stable/nightly Rust `30876035761`, CodeQL
`30876034500`, and aggregate `91887644623` after three exact approvals. R-002
remains HIGH/CRITICAL and PARTIALLY CONTROLLED; no tuple/reference/ownership
capability or other risk status changes. `AUDIT-036` is preregistered to rank the
complete remaining R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/
R-016 set only after its separate exact local/review/public gates, excluding every
accepted sub-slice including CORE-029 and carrying no implementation or capability
authority.

Corrected read-only `AUDIT-036` authorization `f4ac505`, tree `3cdf89e6`, diff
`40896f51`, passes compiler `30876975678` / `30876977928`, stable/nightly Rust
`30876977905`, CodeQL `30876976155`, and aggregate `91890402326` after three exact
approvals. Three independent complete rankings unanimously select one distinct
R-002 residual: a valueless immediate array-of-tuple annotation is explicitly
accepted, silently becomes `Int`, and can reach verifier-valid scalar raw IR. The
R-005 runner-up is already contained before LLVM. CORE-030 is preregistered only to
reject the exact R-002 form at semantics and checked admission after its separate
contract gates and public tests-first red evidence. No risk status or capability
changes; R-002 remains HIGH/CRITICAL and PARTIALLY CONTROLLED.

Accepted public `CORE-030` implementation `97c0f04`, tree `aa3a9e3f`, diff
`06a104df`, closes only the selected valueless immediate array-of-tuple fallback
after triple-reviewed authorization and public 18/19 expected-red evidence.
Semantics rejects after same-scope duplicate detection and before fake
`Int`/insertion; checked admission independently rejects before raw generation.
Focused 1/1, binding 19/19, formatting, the exact full local gate, compiler
`30878810762` / `30878812430`, stable/nightly Rust `30878812406`, CodeQL
`30878811198`, and aggregate `91895661773` pass. Initialized and deeper forms,
scalar arrays, generic/Vec and reference wrappers, other unsupported annotations,
unchecked APIs, tuple/array support, valid-output claims, and every backend remain
unchanged. R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED; record closure is
required before another residual audit or implementation authorization.

Exact CORE-030 record closure `cd8add28`, tree `8ab06d62`, passes compiler
`30879329940` / `30879332975`, stable/nightly Rust `30879332995` attempt 2,
CodeQL `30879330627`, and aggregate `91897195358` after three exact approvals.
The initial Rust attempt's unchanged fake-verifier `ETXTBSY` fixture failure passed
on the focused failed-job rerun without a file or ref change. No risk status,
matrix cell, or capability class changes. Preregistered AUDIT-037 may re-rank the
complete remaining eleven-risk set only after its own exact authorization gates,
must exclude every accepted slice through CORE-030, and carries no test,
implementation, semantics, or capability authority.

Exact read-only AUDIT-037 authorization `987188fc`, tree `0b685659`, diff
`d3a9974b`, is triple-approved and public all-eight green in compiler
`30880025888` / `30880028697`, Rust `30880028653`, CodeQL `30880025866`, and
aggregate `91899286217`. Three complete independent rankings all place R-002 first.
After an initial exact-candidate split, targeted static reconciliation unanimously
selects the explicitly accepted valueless array-array-tuple fallback over the
reference-array-tuple alternative because it avoids reference mutability/ownership
associations at equal reach and phase count. CORE-031 preregisters only two exact
nonrecursive rejection guards after separate contract and public tests-first gates.
No risk status, matrix cell, or capability changes; R-002 remains HIGH/CRITICAL and
PARTIALLY CONTROLLED.

Accepted public CORE-031 implementation `4bc7a345`, tree `61361621`, canonical
diff `349e34ee`, closes only the selected exact valueless array-array-tuple fallback
after triple-reviewed authorization and public 19/20 expected-red evidence with
exactly nine false acceptances. Semantics rejects after same-scope duplicate
detection and before fake `Int`/insertion; checked admission independently rejects
before raw generation. Focused 1/1, binding 20/20, formatting, the exact full local
gate, compiler `30882153355` / `30882155935`, stable/nightly Rust `30882155921`,
CodeQL `30882154595`, and aggregate `91905705897` pass. Candidate B, initialized
and third-plus-depth forms, scalar/nested-scalar arrays, generic/reference wrappers,
other unsupported annotations, raw APIs, tuple/nested-array support, valid-output
claims, and every backend remain unchanged. R-002 stays HIGH/CRITICAL and PARTIALLY
CONTROLLED; record closure is required before another residual audit or
implementation authorization.

Exact CORE-031 record closure `45696091`, tree `480c3504`, canonical diff
`d682b0f6`, passes compiler `30882630407` / `30882632698`, stable/nightly Rust
`30882632696`, CodeQL `30882630822`, and aggregate `91907149874` after three exact
approvals. No risk status, matrix cell, or capability class changes. Preregistered
AUDIT-038 may re-rank the complete remaining eleven-risk set only after its own exact
authorization gates, must exclude every accepted slice through CORE-031, must not
inherit Candidate B or another prior order, and carries no test, implementation,
semantics, or capability authority.

Corrected read-only AUDIT-038 authorization `e4d58e59`, tree `f265d8af`, canonical
diff `31d09f92`, passes compiler `30883186212` / `30883188223`, stable/nightly Rust
`30883188248`, CodeQL `30883186829`, and aggregate `91908783685` after three exact
approvals. All complete rankings put R-002 first. Initial candidates split between
initialized immediate array-of-tuple and valueless triple-array tuple containment;
final compatibility reconciliation unanimously approves the initialized form because
CORE-025 already freezes initializer-child ordering and the exact surface is smaller
at equal trusted reach/two-phase cost. CORE-032 preregisters only two exact rejection
guards after separate contract and public tests-first gates, wherever those statement
paths are already traversed. Its first five-acceptance authorization snapshot was
rejected before publication because it omitted generic impl/function traversal; the
corrected contract freezes eight acceptances and preserves earlier outer-generic
diagnostics. No risk status, matrix cell, or capability changes; R-002 remains
HIGH/CRITICAL and PARTIALLY CONTROLLED.

Corrected CORE-032 authorization `449f3536`, tree `24edc1fe`, canonical diff
`d65f6b75`, is triple-approved and public all-eight green. Rejected tests-only
`1afe11d3` was never published because it omitted explicit array-literal target
coverage. Corrected `35eac8c4`, tree `b54a848b`, canonical diff `e600c2bc`, is
triple-approved and publicly reproduces exactly eight false acceptances as the sole
20/21 failure in compiler `30886282169` / `30886283814` and nightly Rust
`30886284165`; CodeQL `30886281888` and aggregate `91918210639` pass.

Accepted public CORE-032 implementation `30d0d730`, tree `653346ce`, canonical diff
`01e87768`, adds only exact semantic and checked-admission guards after initializer
validation and existing outer-tuple handling. Focused 1/1, binding 21/21, formatting,
two consecutive exact full gates, compiler `30886856260` / `30886858878`, Rust
`30886858960`, CodeQL `30886856518`, and aggregate `91919998289` pass after three
exact approvals. An earlier full-gate attempt returned exit 1 with output truncated
before attribution and remains recorded as unexplained. Candidate T/B, valueless and
deeper/wrapped forms, tuple/array value and compatibility, raw APIs, verifier/codegen,
ABI/ownership, valid output, and every backend remain unchanged. R-002 stays
HIGH/CRITICAL and PARTIALLY CONTROLLED; no matrix cell or capability class moves.

First closure snapshot `7d7fe3d6`, tree `18c904fd`, canonical diff `407c3c86`,
passed its exact gate but was rejected unpublished by all three reviewers because
its state record left that completed gate as future work; one review also required
the known exit 1 above instead of generic nonzero. Second snapshot `48f2fd60`, tree
`86175cc1`, canonical diff `9f0ab102`, resolved those findings and received two
approvals but was rejected unpublished at P3 by the type reviewer because the
successful closure gate omitted literal `exit 0`. The twice-corrected records preserve
both rounds; their fresh exact gate exits 0 with 139/139 library, 149/149 binary, 7/7
doc, and 21/21 binding tests. Exact closure `9c82cbfc`, tree `b2a106ee`, canonical
diff `fc672744`, then received three approvals, was published unchanged, and passes
compiler `30888222316` / `30888225734`, Rust `30888226011`, CodeQL `30888222480`,
and aggregate `91924197947`. R-002 and every other risk status remain unchanged.

Preregistered read-only AUDIT-039 must independently re-rank the complete remaining
R-002/R-004/R-005/R-006/R-007/R-009/R-010/R-011/R-012/R-013/R-016 set from exact
clean public closure `9c82cbfc`, excluding every accepted slice through CORE-032 and
inheriting no earlier candidate or order. Every ranking must be complete and
evidence-cited, identify one bounded candidate or stop, and state reachability,
containment, semantic decisions, phase count, an exact deterministic failing specimen,
and preservation controls. Rejection, simulation, annotations, LLVM text, object
emission, and hardware execution remain distinct.

Exact AUDIT-039 authorization `fa522b2c`, tree `365a536d`, canonical diff `cefb797e`,
passes compiler `30888751268` / `30888754238`, stable/nightly Rust `30888754262`,
CodeQL `30888752230`, and aggregate `91925849313` after three exact approvals. Every
complete ranking puts R-002 first. Type/safety initially selects valueless exact
three-array Candidate T; IR/codegen and backend select initialized exact two-array
Candidate A. Targeted preference favors A two to one. The lead provisionally selects
A for its smaller predicate, two count dimensions, exact 12-acceptance surface, and
already-frozen initializer-child ordering; all three then explicitly approve exact A
with no semantic, compatibility, or phase blocker. AUDIT-039 remains read-only and
changes no risk status.

Preregistered CORE-033 permitted only initialized exact nonrecursive
`Array(Array(Tuple))` after initializer and existing initialized diagnostics in
semantic analysis and checked admission. Tests-first was required to reclassify both
then-current A acceptance rows and reproduce exactly 12 false acceptances: eight
count/phase, one public, two generic-impl, and one semantic generic-function; checked
generic-function outer rejection remained green. Candidate T, reference-array
Candidate B, other initialized/valueless three-plus depth, wrappers, raw APIs,
verifier/codegen, valid output, tuple/array meaning, bounds/layout, ownership/ABI,
and all backends remain unchanged. R-002 stays
HIGH/CRITICAL and PARTIALLY CONTROLLED; no matrix or capability class moves.

The prepared CORE-033 authorization's fresh exact full gate exited 0 with 139/139
library, 149/149 binary, 7/7 doc, and 21/21 binding tests. At that stage, exact
reviews, unchanged publication, and all eight checks were required before tests-
first; implementation required separate reviewed public-red evidence and remained
limited to two phases.

First CORE-033 authorization snapshot `d0500865`, tree `d2378320`, canonical diff
`97a15c9f`, passed its local gate but received one approval and two blocking reviews
because one ledger sentence mislabeled Candidate T's valueless population as
Candidate B. It remained unpublished. Corrected records keep Candidate T and the
reference-array Candidate B distinct; no risk status changes.

Corrected CORE-033 authorization `66207215`, tree `357c2731`, canonical diff
`96b5f403`, is triple-approved and public all-eight green. Unpublished tests snapshot
`7608b42c` was rejected for omitting an initialized three-array-deep preservation
control. Corrected tests-only `ac4cb2a5`, tree `852bff0b`, canonical diff `4ca50572`,
is triple-approved and publicly reproduces exactly 12 false acceptances as the sole
21/22 binding failure in compiler `30891243037` / `30891246443` and nightly Rust
`30891247469`; CodeQL `30891241566` and aggregate `91933672071` pass.

Accepted implementation `76a6e802`, tree `d8391348`, established PowerShell
full-index canonical diff `a75b59b2`, adds only exact semantic and checked-admission
guards. Formatting, focused 1/1, binding 22/22, the exact full local gate exit 0,
compiler `30891890629` / `30891898590`, stable/nightly Rust `30891897083`, CodeQL
`30891892219`, and aggregate `91935804190` pass after corrected-identity triple
approval. The initial review request's erroneous plain-diff `c17b1b6a` changed no
source, commit, tree, risk, or capability state. Candidate T, reference-array
Candidate B, other deeper/wrapped or valueless forms, tuple/nested-array meaning,
raw APIs, verifier/codegen, ABI/ownership, valid output, and all backends remain
unchanged. R-002 stays HIGH/CRITICAL and PARTIALLY CONTROLLED.

First CORE-033 six-record closure snapshot `fe90f583`, tree `90ac8ae6`, canonical
diff `89fe6824`, changed only the control records and passed its exact gate with
139/139 library, 149/149 binary, 7/7 claim, and 22/22 binding tests. It received two
approvals but was rejected at P1 before independent push or branch-head publication
because stale PROJECT_STATE wording could reopen tests-first and implementation.
First correction `19f688a`, tree `9d9c642f`, canonical diff `f885588c`, made that
chronology historical, passed the exact gate, received three approvals, and is
public all-eight green in compiler `30893002336` / `30893005706`, Rust
`30893006634`, CodeQL `30893002479`, and aggregate `91939375982`. Its linear push
also made `fe90f583` publicly reachable as an ancestor, so never-published wording
was inaccurate and closure was withheld. Final additive record correction and its
fresh exact gate exit 0 with 139/139 library, 149/149 binary, 7/7 claim, and 22/22
binding tests are recorded; review/public checks remain and no risk status changes.
