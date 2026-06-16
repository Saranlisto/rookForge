#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
cargo run -- --version
cargo run -- help
cargo run -- perft --help
cargo run -- board --fen startpos
cargo run -- board --fen "8/8/8/8/8/8/8/8 w - - 0 1"
cargo run -- move --parse e2e4
cargo run -- move --parse e7e8q
cargo run -- movegen pawns --fen startpos
