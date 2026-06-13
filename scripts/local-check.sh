#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo run -- --version
cargo run -- help
cargo run -- perft --help

