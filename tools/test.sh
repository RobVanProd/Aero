#!/usr/bin/env bash
set -euo pipefail

# Run compiler crate tests from repo root.
# Usage: ./tools/test.sh

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPILER_DIR="$ROOT_DIR/src/compiler"

if [[ ! -f "$COMPILER_DIR/Cargo.toml" ]]; then
  echo "ERROR: Cargo.toml not found at $COMPILER_DIR" >&2
  exit 1
fi

# Load rustup env if present
if [[ -f "$HOME/.cargo/env" ]]; then
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
fi

# Keep every byte this gate produces on the repository's own drive.
#
# AGENTS.md requires all task output off the system drive, but nothing enforced
# it: this script set no output location at all, so the requirement depended on
# each operator exporting the right variables by hand. Two of them are easy to
# get wrong. Cargo honours CARGO_TARGET_DIR and otherwise writes under the
# repository, which is already off C:. clang does NOT honour TMPDIR on Windows -
# it reads TMP and TEMP - so a session that set only TMPDIR sent every
# intermediate object to the system drive, and with TMP and TEMP unset entirely
# clang falls back to %USERPROFILE% itself. That is how a full C: surfaced as
# "unable to open output file ...: 'no space on device'" inside an unrelated
# test target and read as a product regression.
#
# Defaults are repo-relative so they land on whatever drive the repository is
# on, and any value already exported is respected.
: "${CARGO_TARGET_DIR:=$ROOT_DIR/target}"
GATE_TMP="${GATE_TMP:-$ROOT_DIR/target/gate-tmp}"
mkdir -p "$GATE_TMP"
if command -v cygpath >/dev/null 2>&1; then
  GATE_TMP_NATIVE="$(cygpath -w "$GATE_TMP")"
else
  GATE_TMP_NATIVE="$GATE_TMP"
fi
: "${TMPDIR:=$GATE_TMP}"
TMP="$GATE_TMP_NATIVE"
TEMP="$GATE_TMP_NATIVE"
export CARGO_TARGET_DIR TMPDIR TMP TEMP

# Fail loudly rather than quietly filling the system drive.
for var in CARGO_TARGET_DIR TMPDIR TMP TEMP; do
  value="${!var}"
  case "$value" in
    [Cc]:*|/c/*|/cygdrive/c/*)
      echo "ERROR: $var points at the system drive ($value)." >&2
      echo "       AGENTS.md requires gate output off C:. Set GATE_TMP and" >&2
      echo "       CARGO_TARGET_DIR to a location on another drive." >&2
      exit 1
      ;;
  esac
done

# Cap build and test parallelism so a gate cannot drive the machine into commit
# exhaustion.
#
# On 2026-08-19 a full-parallelism `cargo build` on this repository died with
# `memory allocation of 3670016 bytes failed`, taking rustc down with internal
# compiler errors in unrelated crates (`ryu`, `anstyle`) on the way. Neither is
# a product failure, and neither is a disk-space failure in the ordinary sense.
# The machine has 32 GB of RAM and had free physical memory at the time; what
# was exhausted was the *system commit limit* - 81.8 GB, with 1.3 GB free -
# because the OS-managed pagefile could not grow. It could not grow because the
# system drive was down to 291 MB.
#
# The block above keeps our own output off the system drive, and it is not
# enough on its own: parallel `rustc` plus dozens of linked `clang` test
# executables is precisely the workload that drives commit charge up, and the
# pagefile lives on the system drive no matter where CARGO_TARGET_DIR, TMP and
# TEMP point. Capping the work in flight is the part that addresses the cause.
#
# Two is measured to build and gate this repository cleanly on that machine. The
# cost is wall clock: the full gate goes from roughly 25-40 minutes to roughly
# 40. That is the correct trade. A gate that dies after half an hour costs more
# than a slower one that finishes, and it costs it twice, because an OOM in a
# dependency reads like a product regression until someone measures the commit
# limit.
#
# Any value already exported wins, so a machine with headroom can raise or
# remove the cap without editing this script:
#
#     CARGO_BUILD_JOBS=8 RUST_TEST_THREADS=8 ./tools/test.sh
#
: "${CARGO_BUILD_JOBS:=2}"
: "${RUST_TEST_THREADS:=2}"
export CARGO_BUILD_JOBS RUST_TEST_THREADS

cd "$COMPILER_DIR"

cargo fmt --check
# The codebase has many experimental/unused paths (benches, perf scaffolding,
# compatibility shims) that are expected to trigger `dead_code` and other noisy lints.
#
# We treat *correctness* lints as high-signal and enforce them in CI.
cargo clippy --all-targets --all-features -- -D clippy::correctness
cargo test
