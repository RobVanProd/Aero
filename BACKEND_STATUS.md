# Aero Backend Status

Audit basis: `8f8c7337a4008082fd2a443fcc814b5847b8663f`.

Backend stages are deliberately independent. A selectable label or transformed
LLVM module is not evidence for object generation, linking, launch, hardware
execution, numerical correctness, or performance.

## Stage matrix

| Backend | Interface | IR transformation | Object generation | Linking | Real execution | Numerical correctness | Performance evidence | Classification |
|---|---|---|---|---|---|---|---|---|
| CPU | `build`/`run` selectable | Host LLVM text generated | Implemented through `llc` when available | Implemented through `clang` when available | Process execution implemented; small Linux CI exit-code programs exist | Small integer/float exit-code checks only | Current README compilation series is invalid; no accepted runtime claim in this audit | PARTIAL |
| ROCm | Target/backend/gpu flags selectable | Module triple/data layout retargeted; backend-named graph/quant helpers | AMDGPU object invocation through `llc` exists | No HIP launcher/link path in active `run` | Absent; CLI explicitly says launcher integration is staged | No Aero hardware result comparison | Tracked GGUF result is external llama.cpp only; not Aero execution | EXPERIMENTAL |
| CUDA | Target/backend/gpu flags selectable | Target metadata/backend-named helpers only | Absent from active `run` | Absent | Explicitly unimplemented | Absent | Absent | PARSED_ONLY |

## CPU path

`run_aero_program` writes LLVM, invokes `llc` for a host object, links with
`clang`, and executes the resulting process (`src/compiler/src/main.rs`, roughly
lines 1637–1710). This is a real execution path when those external tools exist.
The secondary Rust workflow verifies four small examples on Ubuntu by expected
exit status. The current Windows audit environment has not yet provided local
LLVM/linker execution evidence, so Windows support is not classified end-to-end.

The CPU path can fall back to direct Clang compilation if `llc` is missing. It
also prints a success message before interpreting a nonzero program exit status,
so status reporting needs an explicit contract test.

At the audit basis, the active compiler pipeline still had semantic and typed-IR
invariant failures. Successful execution of a few programs therefore does not make
the CPU language surface stable.

The local `CORE-010` production candidate closes the selected scalar IR-publication
boundary: checked source lowering is internally verified, final graph-transformed
and retargeted LLVM is externally verified with a qualified LLVM 22 tool before
cache/write/native publication, and invalid input/output produces no artifact.
Verifier subprocesses are contained as Unix process groups or Windows kill-on-close
jobs; Windows children are created suspended and assigned before their first
instruction so descendants cannot escape the deadline.
Focused contracts and the complete repository gate pass locally. This is pending
exact-diff review and public CI and does not expand the CPU feature or execution
classification.

## ROCm path

The CLI can retarget module metadata and invoke `llc` for an AMDGPU object, but it
does not link or launch that object. Its own message states that runtime execution
is staged for HIP launcher integration (`src/compiler/src/main.rs`, roughly lines
1712–1765). No current Aero program has been proven to transfer data, launch a
kernel, synchronize, or validate a result on ROCm in this audit.

The tracked GGUF ROCm evidence executes a local llama.cpp CLI. The README
qualifies it as an external reference; it is not evidence of Aero code generation
or execution.

GPU auto-detection honors an environment selection and otherwise probes ROCm
tool presence before falling back to CPU; it does not establish a usable device
and does not auto-probe CUDA. On the current host it selects CPU despite an AMD
GPU because the expected tools are absent.

## CUDA path

CUDA is accepted by backend parsing and target-selection interfaces, but active
`run` returns an explicit not-implemented message (`src/compiler/src/main.rs`,
roughly lines 1767–1771). No CUDA object, link, launch, correctness, or Aero
performance evidence was found.

## Graph compilation

Graph optimization performs textual LLVM analysis/transformation and can emit
ordinary internal scalar `double` helper functions whose names include a backend
label. It does not emit a verified device-kernel calling convention, device
intrinsics, memory transfers, launches, synchronization, or backend object/link
steps. Existing tests prove deterministic transformation and helper emission,
not CPU/GPU execution equivalence.

In the local `CORE-010` candidate, standalone graph optimization verifies both its
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

In the local `CORE-010` candidate, standalone quantization verifies both arbitrary
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
