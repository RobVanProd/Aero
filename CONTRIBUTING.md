# Contributing to Aero

Thanks for your interest in contributing!

## Quick start

1. Fork + clone the repo
2. Follow the build instructions in [BUILD.md](BUILD.md)
3. Make a branch, commit your changes, open a PR

## Development workflow

### Build + test

```bash
cd src/compiler
cargo fmt
# Clippy is advisory for now; prefer fixing correctness lints first.
cargo clippy --all-targets --all-features -- -D clippy::correctness
cargo test
```

### Try the compiler

From the repo root:

```bash
./src/compiler/target/release/aero --help
./src/compiler/target/release/aero build examples/hello.aero -o hello.ll
```

## What to work on

- Issues and roadmap items in the repo
- Compiler correctness, diagnostics, and tests
- Documentation and examples

## Pull request guidelines

- Keep PRs focused and reasonably sized
- Include a short description of the change and how you tested it
- Add/update docs when behavior changes

## Code of conduct

Please follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
