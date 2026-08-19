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

cd "$COMPILER_DIR"

cargo fmt --check
# The codebase has many experimental/unused paths (benches, perf scaffolding,
# compatibility shims) that are expected to trigger `dead_code` and other noisy lints.
#
# We treat *correctness* lints as high-signal and enforce them in CI.
cargo clippy --all-targets --all-features -- -D clippy::correctness
cargo test
