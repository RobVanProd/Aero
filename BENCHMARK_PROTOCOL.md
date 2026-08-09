# Aero Benchmark Protocol

No public performance claim may be added or updated unless this protocol is
satisfied and the publishing action is explicitly approved.

## Correctness gate

The benchmark program, baseline, and Aero output must first pass an exact or
documented tolerance-based correctness check. Failed cases remain in raw data
and invalidate aggregate claims that would omit them.

## Required manifest

Each run records:

- immutable Aero commit and dirty-tree status;
- benchmark and input source plus cryptographic hashes;
- expected output and the command that verifies it;
- operating system, kernel, CPU, memory, GPU, firmware where relevant;
- compiler, linker, LLVM, driver, runtime, and dependency versions;
- all build/runtime flags and environment variables affecting behavior;
- warm-up policy, iteration count, timeout, affinity, power/clock policy, and
  competing-load controls;
- raw samples, failures, median, p95, range or variance, and comparison method;
- artifact hashes and a complete reproduction command;
- exactly what setup, compilation, transfer, synchronization, and teardown time
  is included.

## Measurement boundaries

Report lexer, parser, semantic analysis, IR generation, complete compilation,
incremental compilation, link time, generated-program runtime, kernel runtime,
host-to-device transfer, device-to-host transfer, and end-to-end application
runtime separately whenever they are relevant. Do not compare different
boundaries under one speedup label.

## Baselines and accelerators

Baselines must run equivalent work, inputs, precision, output validation, and
measurement boundaries. Device claims must include captured evidence that Aero
generated, launched, and synchronized work on the named device. A backend flag,
LLVM target annotation, helper execution, or an external runtime benchmark does
not demonstrate Aero device execution.

## Evidence retention

Place the manifest, raw output, normalized summary, environment capture, failure
logs, and hashes beneath a uniquely named directory in `claim-verification/`.
Reference that immutable directory from `claim-verification/claims.json`. State
limitations next to the claim. Historical artifacts stay historical and are not
silently relabeled as current results.
