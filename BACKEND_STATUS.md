# Aero Backend Status

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

Backend stages are deliberately independent. A selectable label or transformed
LLVM module is not evidence for object generation, linking, launch, hardware
execution, numerical correctness, or performance.

## Stage matrix

| Backend | Interface | IR transformation | Object generation | Linking | Real execution | Numerical correctness | Performance evidence | Classification |
|---|---|---|---|---|---|---|---|---|
| CPU | `build`/`run` selectable | Host LLVM text generated | Implemented through `llc` when available | Implemented through `clang` when available | Process execution implemented; accepted pinned Linux and bounded Windows x86_64 exits exist | Small bounded exit-code checks only | Current README compilation series is invalid; no accepted performance claim | PARTIAL |
| ROCm | Explicit `rocm` target/backend selectable; ambiguous `gpu` is rejected | Module triple/data layout retargeted; backend-named graph/quant scalar helpers | `run` can ask `llc` for a temporary regular file; existence is not object validity | No HIP launcher/link path | Absent; `run` returns status 1 and states that no program ran | No Aero hardware result comparison | Tracked GGUF result is external llama.cpp only; not Aero execution | EXPERIMENTAL |
| CUDA | Explicit `cuda` target/backend selectable; ambiguous `gpu` is rejected | Target metadata/backend-named scalar helpers only | Absent from active `run` | Absent | Absent; `run` returns status 1 and recommends CPU | Absent | Absent | PARSED_ONLY |

## CPU path

`run_aero_program` writes LLVM, invokes `llc` for a host object, links with
`clang`, and executes the resulting process (`run_aero_program` in
`src/compiler/src/main.rs`, roughly lines 1869–1985). This is a real execution path
when those external tools exist.
The secondary Rust workflow verifies the accumulated bounded examples on Ubuntu by
expected exit status, and `CORE-014` adds the generated `Hello, Aero!` project with
explicit status and anchored-output proof. Accepted CORE-077 reaches native exit 227
on pinned stable/nightly Linux LLVM/Clang 22.1.8. Accepted public CORE-078 adds a
separate `windows-latest` x86_64 job that pins the official full LLVM/Clang 22.1.8
archive by release SHA-256 and requires MSVC-target LLVM, external/machine
verification, COFF object generation, Clang/MSVC linking, public `run`, manual
execution, invalid-build artifact hygiene, and exit 227. Exact implementation
`70f59fd72e96246b2ebefdf1ae53a9b7f3280cfe` passes all nine exact-head checks;
Linux stable/nightly preserve exits 149/223/227. The current local host still lacks
LLVM 22.

Accepted public CORE-079 changes no backend representation, layout, or ABI. Its tracked
direct-module fixed-point specimen is wired for external/machine verification,
object/link, public/manual execution, and exact exit 229 on pinned Linux and Windows
LLVM/Clang 22.1.8. Exact implementation
`5b1ec7340db72354542ab325a9f75cad398857c2` passes all nine exact-head checks;
stable/nightly Linux preserve exits 149/223/227 and execute exit 229, while the pinned
Windows lane preserves exit 227 and executes exit 229 through public and independent
native paths.

Local CORE-080 changes no checked-IR opcode, LLVM representation, layout, ABI,
workflow, or native specimen. Both retained import syntax families fail before checked
IR, and public check/build/run coverage proves artifact hygiene. Focused 13/13,
compatibility, complete all-features, static, documentation, diff-hygiene, and exact
root gates pass. The accepted native exits 149/223/227/229 must remain unchanged in all
exact-head public lanes before CORE-080 can be accepted.

The CPU path can fall back to direct Clang compilation if `llc` is missing. It
also prints a success message before interpreting a nonzero program exit status,
so status reporting needs an explicit contract test.

At the audit basis, the active compiler pipeline still had semantic and typed-IR
invariant failures. Successful execution of a few programs therefore does not make
the CPU language surface stable.

The accepted `CORE-010` production implementation closes the selected scalar IR-publication
boundary: checked source lowering is internally verified, final graph-transformed
and retargeted LLVM is externally verified with a qualified LLVM 22 tool before
cache/write/native publication, and invalid input/output produces no artifact.
Verifier subprocesses are contained as Unix process groups or Windows kill-on-close
jobs; Windows children are created suspended and assigned before their first
instruction so descendants cannot escape the deadline.
Focused contracts, the complete repository gate, three exact-diff reviews, and all
required public CI checks pass at head `db349ef`. This does not expand the CPU
feature or execution classification.

Accepted `CORE-064` adds no backend or ABI class. For already admitted
private enum schemas it emits exact typed alloca/load/store operations for mutable
whole-owner replacement, guarded by generalized checked identities and independent
schema verification. Stable job `92376666972` uses LLVM/Clang 22.1.8 to reject the
known-invalid verifier control, externally verify, machine-verify, object-lower,
explicitly link the private non-PIE executable, and observe exact native exit 131;
nightly job `92376666842` repeats exit 131. This does not establish a public backend,
layout, or ABI class.

Accepted `CORE-065` also adds no backend representation or ABI class. It reuses
the accepted private enum values, typed places, branches, and merge blocks while an
independent verifier proves enum-owner consumption across the checked CFG. The tracked
direct-module specimen passes LLVM/Clang 22.1.8 external verification, machine
verification, object lowering, explicit private non-PIE linking, and native exit 137
in stable job `92454648190`; nightly job `92454648318` repeats exit 137. No loop
ownership, public layout, backend stability, or ABI claim follows.

Accepted `CORE-066` also changes no enum representation or ABI. It adds an
exact `for` continue/increment CFG block and certifies fresh per-iteration enum
definitions under the existing private types. The implementation candidate
`e40804ea86888b38548fd5bf42926be2be7eb5ed` builds the two-file source to LLVM;
pinned stable job `92463336662` installs LLVM/Clang 22.1.8, rejects the known-invalid
fixture, externally verifies, machine-verifies, object-lowers, explicitly links, and
executes exact exit 149. Nightly job `92463336701` independently repeats exit 149.
This is exact public evidence for the bounded loop-local class, not a general loop,
ownership, representation, ABI, or backend-stability claim.

Accepted public `CORE-067` adds no runtime method-dispatch ABI, collection
representation, String layout, or new checked-IR opcode. Supported recursive CopyData
fixed-array length/emptiness and established immutable compile-time String predicates
lower to exact immediate scalar values; Array/Vec `.iter()` retains its existing
receiver compatibility. One shared classifier is consumed by checked admission and
trusted lowering, whose unsupported-method path has no fabricated scalar fallback.
The tracked direct-module program emits Windows LLVM, links with Visual Studio Clang
19.1.5, and executes exact exit 167. Stable job `92473492653` on LLVM/Clang 22.1.8
rejects the known-invalid fixture, externally verifies, machine-verifies, object-
lowers, privately non-PIE links, and executes the same exit; nightly job `92473492801`
independently repeats exit 167. This is public evidence only for the exact bounded
intrinsic-method class and establishes no general method, layout, ABI, backend-
stability, or performance class.

Accepted public `CORE-068` changes no call ABI, aggregate representation, checked-IR
opcode, or verifier rule. One shared call classifier admits only an exact existing
nongeneric signature and exact argument/result contract before trusted lowering;
missing, declared-but-unadmitted, generic, closure, unsupported-annotation, wrong-
arity/type, and `Void`-in-value calls reject before checked IR instead of manufacturing
`Int` or an LLVM call. The tracked direct-module specimen emits deterministic Windows
LLVM, links with Visual Studio Clang 19.1.5, and executes exact exit 181. This host has
no LLVM 22 and truthfully reports `InternalOnly`. Stable job `92484486912` on pinned
LLVM/Clang 22.1.8 rejects the known-invalid fixture, externally verifies, machine-
verifies, object-lowers, privately non-PIE links, and executes exact exit 181; nightly
job `92484486872` independently repeats external verification and exit 181. This is
public evidence only for the exact bounded function-call class and establishes no
general callable ABI, layout, backend-stability, safety, or performance class.

## ROCm path

The CLI can retarget module metadata and invoke `llc` with AMDGPU flags, but it
checks only that `llc` produced a temporary regular file. That postcondition does
not establish a valid or usable object. `run` then returns operational status 1
with an explicit statement that HIP linking and device launch are absent and no
program was executed. No current Aero program has been proven to transfer data,
launch a kernel, synchronize, or validate a result on ROCm in this audit.

The tracked GGUF ROCm evidence executes a local llama.cpp CLI. The README
qualifies it as an external reference; it is not evidence of Aero code generation
or execution.

An internal GPU auto-detection helper remains experimental, honors an environment
selection, and otherwise probes ROCm tool presence before falling back to CPU; it
does not establish a usable device and does not auto-probe CUDA. Public `build`
and `run` reject the ambiguous `gpu` alias before reading source and require an
explicit `cpu`, `rocm`, or `cuda` selection.

## CUDA path

CUDA is accepted by explicit backend and target-selection interfaces, but active
`run` returns operational status 1 and states that object generation, linking,
device launch, and program execution are unavailable. It recommends `--target
cpu` for execution. No CUDA object, link, launch, correctness, or Aero performance
evidence was found.

## Graph compilation

Graph optimization performs textual LLVM analysis/transformation and can emit
ordinary internal scalar `double` helper functions whose names include a backend
label. It does not emit a verified device-kernel calling convention, device
intrinsics, memory transfers, launches, synchronization, or backend object/link
steps. Existing tests prove deterministic transformation and helper emission,
not CPU/GPU execution equivalence.
Emitted text carries `execution_scope=internal-scalar-helper` and
`device_execution=false` telemetry; report field names remain compatibility data,
not evidence of executable device kernels.

In the accepted `CORE-010` production implementation, standalone graph optimization verifies both its
arbitrary LLVM input and final transformed output before publication. Verification
does not establish semantic equivalence of the textual transformation.

## Quantization

Quantization rewrites operations to scalar `double` helper calls. CPU, ROCm, and
CUDA selections currently vary names/comments rather than executable backend
mechanisms. FP8 mode has no demonstrated FP8 conversion/rounding behavior;
`per_channel` is metadata rather than an executed layout/calibration mechanism.
The audit also identified incorrect algebra in INT8 multiplication/division
dequantization, and conversion occurs before later clamps can protect exceptional
or out-of-range inputs. Current tests cover transformation shape and determinism,
not a trusted reference comparison across modes/backends.
Emitted text carries `execution_scope=scalar-double-helper` and
`device_execution=false`. Report notes explicitly deny real FP8 representation,
per-channel execution, numerical proof, and device execution.

In the accepted `CORE-010` production implementation, standalone quantization verifies both arbitrary
LLVM input and final transformed output before publication. This guards LLVM module
validity only; it does not resolve the numerical-correctness findings above.

## Required gates for a backend claim

1. Define backend capability and memory/execution semantics.
2. Emit verifiable target IR and retain the artifact.
3. Generate the intended target object and link/loader artifact.
4. Capture proof of launch on the named device, transfers, and synchronization.
5. Compare results with a trusted CPU reference under a stated tolerance.
6. Test unsupported hardware, compilation failure, launch failure, and explicit
   fallback observability.
7. Measure compilation, transfers, kernel time, and end-to-end time separately
   under `BENCHMARK_PROTOCOL.md`.

Until those gates pass, backend flags and transformations remain experimental
interfaces rather than heterogeneous execution support.
