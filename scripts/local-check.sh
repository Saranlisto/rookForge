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
cargo run -- movegen knights --fen startpos
cargo run -- movegen kings --fen "8/8/8/8/4K3/8/8/8 w - - 0 1"
cargo run -- movegen bishops --fen "8/8/8/3B4/8/8/8/8 w - - 0 1"
cargo run -- movegen rooks --fen "8/8/8/3R4/8/8/8/8 w - - 0 1"
cargo run -- movegen queens --fen "8/8/8/3Q4/8/8/8/8 w - - 0 1"
cargo run -- movegen all --fen startpos
cargo run -- movegen legal --fen startpos
cargo run -- apply --fen startpos --move e2e4
cargo run -- attacks --fen "4r3/8/8/8/4K3/8/8/8 w - - 0 1" --square e4 --by black
cargo run -- attacks --fen startpos --square e4 --by black
cargo run -- attacks --fen startpos --square e4 --by white
