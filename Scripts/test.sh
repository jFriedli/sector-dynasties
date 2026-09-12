#!/usr/bin/env bash
# Runs the Rust half of the test suite the same way CI does. Frontend checks
# live under web/ (see CONTRIBUTING.md); they're separate because they need
# npm and the generated wasm bridge, neither of which every Rust-only change
# needs to touch.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
