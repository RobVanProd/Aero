# CAP-023 accepted-head evidence reproduction

This is the complete third-party target artifact and observable result procedure
for claim `aero_cap023_inference_correctness_918c9222_20260813`. It verifies only
the accepted CAP-023 source-embedded ReLU/argmax product at commit
`918c9222eb61e2435e18847e30b946cd08013238`, tree
`aba2876644b0183ab877b2e28d5e14001328c99a`, under schema
`aero-cap023-inference-evidence-v1`, tool contract
`cap024-inference-evidence-v1`, and oracle
`cap023-relu-argmax-inference-oracle-v1`.

## Initial capture

1. Use `.github/workflows/cap023-evidence.yml` or reproduce its commands on a
   compatible x86_64 host. Before checkout, run
   `git config --global core.autocrlf false`; after checkout, require
   `git config --get core.autocrlf` to print `false`. Resolve every path in the
   canonical input inventory with
   `git ls-tree 918c9222eb61e2435e18847e30b946cd08013238 -- <path>`, require its
   blob ID to equal the recorded blob, materialize its immutable bytes with
   `git cat-file blob <blob-id>`, and compare the resulting byte count and
   SHA-256 with the inventory before any build. Create a separate detached Git checkout
   of that exact commit at
   `$CAP024_TRANSPORT_ROOT/_workspace/subject`; require its ordered parents,
   HEAD tree, compiler subtree, every tracked worktree blob, and
   `git status --porcelain=v1 --untracked-files=all` to match before and after
   capture. The machine-readable shorthand for this boundary is
   `core.autocrlf false and canonical Git blob bytes`.
2. Acquire the two content-pinned LLVM archives listed in the canonical
   inventory below, and verify each archive's byte size and SHA-256 before
   extraction. Run
   `rustup toolchain install 1.97.1 --profile minimal --no-self-update` to
   install Rust release `1.97.1` explicitly rather than through a floating
   channel, set `RUSTUP_TOOLCHAIN=1.97.1`, then require `rustc -Vv` to report
   release `1.97.1` and
   commit `8bab26f4f68e0e26f0bb7960be334d5b520ea452`. Record and verify SHA-256
   and byte size for the final `cargo` and `rustc` executables exactly as the
   manifest's tool records require. This claim intentionally pins those final
   payload identities; it does not invent an unfrozen Rust archive digest. Do
   not replace any pinned tool or action with a channel, tag, package-manager
   substitute, or host linker.
3. Before either compiler build, the tool parses the canonical version-4
   `src/compiler/Cargo.lock`, requires its 112 nonlocal packages to be exact
   checksummed crates.io records, downloads each exact name/version `.crate`
   payload from `https://static.crates.io/crates/`, and verifies its SHA-256
   against the lockfile. It extracts those verified payloads into a fresh
   platform-local vendor directory, writes an explicit `CARGO_HOME` source
   replacement, and rejects any missing, extra, unchecksummed, or alternate
   package source. Hosted-runner Cargo caches are never inputs.
4. Invoke `tools/cap024_inference_evidence.py` in capture mode separately for
   `linux-x86_64` and `windows-x86_64`. The tool uses `CARGO_NET_OFFLINE true`,
   `--locked --offline`, the exact environment and command records listed
   below, and the embedded launch-support sources bound by the manifest.
5. Each lane builds the Aero compiler in two clean target directories, emits
   LLVM twice, verifies it, assembles bitcode, runs machine-instruction
   verification, links O0/O2 twice, and executes every retained native.
   Native O0 and O2 exit 91 with empty stdout and stderr. Two same-platform
   productions must have equal SHA-256 and byte size for `.ll`, `.bc`, `.s`,
   O0, and O2.
6. Each lane also records the public route. The semantic observation is parsed Exit code: 91
   and no application Output: or Error output:. Every command
   exit and every raw stream is retained losslessly as base64, SHA-256, and
   byte size. A capture exception becomes a canonical failure observation; do
   not replace a failed capture with a partial success record.
7. Aggregate exactly one canonical textual platform record from each lane.
   Validate `oracle.json`, the strict schema, every input identity, and every
   manifest file hash before treating the candidate manifest as publishable.

## Replay

Run the tool in replay mode with the tracked bundle and a fresh two-platform
manifest. It compares the sorted-compact-JSON-plus-LF canonical projection
byte-for-byte. Fresh observations never rewrite accepted observations. The
fresh observation document must bind the accepted manifest SHA-256 and list
the exact 132 excluded pointer/value leaves in canonical order; every other
difference is claim-bearing and fails replay.

The raw pinned-tool version, Aero LLVM-build, and public-run diagnostic streams
are traceability-only replay exclusions. Parsed tool versions, tool paths,
payload SHA-256/size identities, and version-command exits remain claim-bearing.
The two compiler executable identities, runner image, and
kernel values are also traceability observations. Command exits, parsed public
semantics, pinned-tool identities, and every retained target artifact remain in
the compared projection.

## Failure observations and limitations

The runner image and kernel observation are not an immutable or reconstructible
evidence input. A repeat on another host is new corroboration, not proof that
the host matches the accepted runner. Cross-operating-system artifact hashes
are neither required nor implied. Byte sizes are footprint facts only. There is
no performance claim, general-inference claim, stable ABI claim, accelerator
claim, resource-use claim, or safety claim.

## Canonical identifier, command, and replay-pointer inventory

The lines below are literal machine-checkable identifiers consumed by the
standard-library verifier. They are part of the procedure rather than an
alternate source of truth.

918c9222eb61e2435e18847e30b946cd08013238
aba2876644b0183ab877b2e28d5e14001328c99a
aero_cap023_inference_correctness_918c9222_20260813
aero-cap023-inference-evidence-v1
cap024-inference-evidence-v1
cap023-relu-argmax-inference-oracle-v1
tools/cap024_inference_evidence.py
linux-x86_64
windows-x86_64
918c9222eb61e2435e18847e30b946cd08013238
aba2876644b0183ab877b2e28d5e14001328c99a
e9b281504446465cfc8fcbe17c65cce92df0e83a
d21c91fc312c70c47c6bb865ba1465e762255f0c
0ba0d06899b7e95d6b5b6f90a14804d18651806c
examples/fixed_int_array_v0/relu_argmax_inference.aero
5d5fe74e4acc351cb4326e85c4d69f320a37f3c6
8244ca26fc90ce708801e12ec6a7192bdedfd01e1a1429c1479d36e233b1bb6c
8224
.github/workflows/rust.yml
888a1d6b699725ebdd8b8fd6c762c1b58cd823a3
32c820df765c6f42025d46a9f95049610fb8c301233f51920c7182fda74a92f5
264585
src/compiler/tests/fixed_int_array_profile_tests.rs
959033d0fd255b947d16aa83efe914b517ced412
6300d3e2a9ef51c270c9ea876a54e70be3fae0e55ccaab5bb81a060a36af5103
257332
src/compiler/Cargo.toml
156dee0fc73aad0bf832c216edbfc9d13fb70012
ee0ab0da24d5706101b37fdf94940fe863e097bcc02b0752b0bccaddf48ab96f
1072
src/compiler/Cargo.lock
24c4729076801853f7bebb4a3269c050f31b3a5a
076d1d4f06ed35627c45a93428aab3705fceafcada5f09ae1597ada6922ff280
26063
1.97.1
8bab26f4f68e0e26f0bb7960be334d5b520ea452
22.1.8
LLVM-22.1.8-Linux-X64.tar.xz
df0e1ecf16caf3489a272a5eea4eec9b0d82878f6477fa309504f918a0006384
1938859476
clang+llvm-22.1.8-x86_64-pc-windows-msvc.tar.xz
d96c2cc1736f4eb7fa43cb9bbdf56d93551a9ae0a9aadb9c99c3c3b2b712a234
862053924
linux-start.S
b95dbd79fd7b976862149e5635e148b9a9d2bbf20b2c3912a1f8d76c227379bb
205
windows-chkstk.S
b971f9c51534aff82d774c26b6a6f2312a3beeac5e1710a69f3d88bd5671f376
378
llvm
bitcode
assembly
executable_o0
executable_o2
clean_before
compiler_build_first
compiler_build_second
aero_build_llvm_first
aero_build_llvm_second
llvm_verify_first
llvm_verify_second
llvm_assemble_first
llvm_assemble_second
machine_verify_first
machine_verify_second
link_o0_first
link_o0_second
link_o2_first
link_o2_second
native_o0_first
native_o0_second
native_o2_first
native_o2_second
public_run
clean_after
/platforms/0/observations/runner_image
/platforms/0/observations/kernel
/platforms/0/compiler_executables/first/sha256
/platforms/0/compiler_executables/first/size
/platforms/0/compiler_executables/second/sha256
/platforms/0/compiler_executables/second/size
/platforms/0/commands/aero_build_llvm_first/stdout/base64
/platforms/0/commands/aero_build_llvm_first/stdout/sha256
/platforms/0/commands/aero_build_llvm_first/stdout/size
/platforms/0/commands/aero_build_llvm_first/stderr/base64
/platforms/0/commands/aero_build_llvm_first/stderr/sha256
/platforms/0/commands/aero_build_llvm_first/stderr/size
/platforms/0/commands/aero_build_llvm_second/stdout/base64
/platforms/0/commands/aero_build_llvm_second/stdout/sha256
/platforms/0/commands/aero_build_llvm_second/stdout/size
/platforms/0/commands/aero_build_llvm_second/stderr/base64
/platforms/0/commands/aero_build_llvm_second/stderr/sha256
/platforms/0/commands/aero_build_llvm_second/stderr/size
/platforms/0/commands/public_run/stdout/base64
/platforms/0/commands/public_run/stdout/sha256
/platforms/0/commands/public_run/stdout/size
/platforms/0/commands/public_run/stderr/base64
/platforms/0/commands/public_run/stderr/sha256
/platforms/0/commands/public_run/stderr/size
/platforms/1/observations/runner_image
/platforms/1/observations/kernel
/platforms/1/compiler_executables/first/sha256
/platforms/1/compiler_executables/first/size
/platforms/1/compiler_executables/second/sha256
/platforms/1/compiler_executables/second/size
/platforms/1/commands/aero_build_llvm_first/stdout/base64
/platforms/1/commands/aero_build_llvm_first/stdout/sha256
/platforms/1/commands/aero_build_llvm_first/stdout/size
/platforms/1/commands/aero_build_llvm_first/stderr/base64
/platforms/1/commands/aero_build_llvm_first/stderr/sha256
/platforms/1/commands/aero_build_llvm_first/stderr/size
/platforms/1/commands/aero_build_llvm_second/stdout/base64
/platforms/1/commands/aero_build_llvm_second/stdout/sha256
/platforms/1/commands/aero_build_llvm_second/stdout/size
/platforms/1/commands/aero_build_llvm_second/stderr/base64
/platforms/1/commands/aero_build_llvm_second/stderr/sha256
/platforms/1/commands/aero_build_llvm_second/stderr/size
/platforms/1/commands/public_run/stdout/base64
/platforms/1/commands/public_run/stdout/sha256
/platforms/1/commands/public_run/stdout/size
/platforms/1/commands/public_run/stderr/base64
/platforms/1/commands/public_run/stderr/sha256
/platforms/1/commands/public_run/stderr/size
/platforms/0/toolchain/tools/cargo/version/stdout/base64
/platforms/0/toolchain/tools/cargo/version/stdout/sha256
/platforms/0/toolchain/tools/cargo/version/stdout/size
/platforms/0/toolchain/tools/cargo/version/stderr/base64
/platforms/0/toolchain/tools/cargo/version/stderr/sha256
/platforms/0/toolchain/tools/cargo/version/stderr/size
/platforms/0/toolchain/tools/rustc/version/stdout/base64
/platforms/0/toolchain/tools/rustc/version/stdout/sha256
/platforms/0/toolchain/tools/rustc/version/stdout/size
/platforms/0/toolchain/tools/rustc/version/stderr/base64
/platforms/0/toolchain/tools/rustc/version/stderr/sha256
/platforms/0/toolchain/tools/rustc/version/stderr/size
/platforms/0/toolchain/tools/clang/version/stdout/base64
/platforms/0/toolchain/tools/clang/version/stdout/sha256
/platforms/0/toolchain/tools/clang/version/stdout/size
/platforms/0/toolchain/tools/clang/version/stderr/base64
/platforms/0/toolchain/tools/clang/version/stderr/sha256
/platforms/0/toolchain/tools/clang/version/stderr/size
/platforms/0/toolchain/tools/lld/version/stdout/base64
/platforms/0/toolchain/tools/lld/version/stdout/sha256
/platforms/0/toolchain/tools/lld/version/stdout/size
/platforms/0/toolchain/tools/lld/version/stderr/base64
/platforms/0/toolchain/tools/lld/version/stderr/sha256
/platforms/0/toolchain/tools/lld/version/stderr/size
/platforms/0/toolchain/tools/opt/version/stdout/base64
/platforms/0/toolchain/tools/opt/version/stdout/sha256
/platforms/0/toolchain/tools/opt/version/stdout/size
/platforms/0/toolchain/tools/opt/version/stderr/base64
/platforms/0/toolchain/tools/opt/version/stderr/sha256
/platforms/0/toolchain/tools/opt/version/stderr/size
/platforms/0/toolchain/tools/llvm-as/version/stdout/base64
/platforms/0/toolchain/tools/llvm-as/version/stdout/sha256
/platforms/0/toolchain/tools/llvm-as/version/stdout/size
/platforms/0/toolchain/tools/llvm-as/version/stderr/base64
/platforms/0/toolchain/tools/llvm-as/version/stderr/sha256
/platforms/0/toolchain/tools/llvm-as/version/stderr/size
/platforms/0/toolchain/tools/llc/version/stdout/base64
/platforms/0/toolchain/tools/llc/version/stdout/sha256
/platforms/0/toolchain/tools/llc/version/stdout/size
/platforms/0/toolchain/tools/llc/version/stderr/base64
/platforms/0/toolchain/tools/llc/version/stderr/sha256
/platforms/0/toolchain/tools/llc/version/stderr/size
/platforms/1/toolchain/tools/cargo/version/stdout/base64
/platforms/1/toolchain/tools/cargo/version/stdout/sha256
/platforms/1/toolchain/tools/cargo/version/stdout/size
/platforms/1/toolchain/tools/cargo/version/stderr/base64
/platforms/1/toolchain/tools/cargo/version/stderr/sha256
/platforms/1/toolchain/tools/cargo/version/stderr/size
/platforms/1/toolchain/tools/rustc/version/stdout/base64
/platforms/1/toolchain/tools/rustc/version/stdout/sha256
/platforms/1/toolchain/tools/rustc/version/stdout/size
/platforms/1/toolchain/tools/rustc/version/stderr/base64
/platforms/1/toolchain/tools/rustc/version/stderr/sha256
/platforms/1/toolchain/tools/rustc/version/stderr/size
/platforms/1/toolchain/tools/clang/version/stdout/base64
/platforms/1/toolchain/tools/clang/version/stdout/sha256
/platforms/1/toolchain/tools/clang/version/stdout/size
/platforms/1/toolchain/tools/clang/version/stderr/base64
/platforms/1/toolchain/tools/clang/version/stderr/sha256
/platforms/1/toolchain/tools/clang/version/stderr/size
/platforms/1/toolchain/tools/lld/version/stdout/base64
/platforms/1/toolchain/tools/lld/version/stdout/sha256
/platforms/1/toolchain/tools/lld/version/stdout/size
/platforms/1/toolchain/tools/lld/version/stderr/base64
/platforms/1/toolchain/tools/lld/version/stderr/sha256
/platforms/1/toolchain/tools/lld/version/stderr/size
/platforms/1/toolchain/tools/opt/version/stdout/base64
/platforms/1/toolchain/tools/opt/version/stdout/sha256
/platforms/1/toolchain/tools/opt/version/stdout/size
/platforms/1/toolchain/tools/opt/version/stderr/base64
/platforms/1/toolchain/tools/opt/version/stderr/sha256
/platforms/1/toolchain/tools/opt/version/stderr/size
/platforms/1/toolchain/tools/llvm-as/version/stdout/base64
/platforms/1/toolchain/tools/llvm-as/version/stdout/sha256
/platforms/1/toolchain/tools/llvm-as/version/stdout/size
/platforms/1/toolchain/tools/llvm-as/version/stderr/base64
/platforms/1/toolchain/tools/llvm-as/version/stderr/sha256
/platforms/1/toolchain/tools/llvm-as/version/stderr/size
/platforms/1/toolchain/tools/llc/version/stdout/base64
/platforms/1/toolchain/tools/llc/version/stdout/sha256
/platforms/1/toolchain/tools/llc/version/stdout/size
/platforms/1/toolchain/tools/llc/version/stderr/base64
/platforms/1/toolchain/tools/llc/version/stderr/sha256
/platforms/1/toolchain/tools/llc/version/stderr/size
