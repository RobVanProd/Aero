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

cd "$COMPILER_DIR"

cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings || true
cargo test
