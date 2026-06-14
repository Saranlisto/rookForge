.PHONY: fmt clippy test build smoke check

fmt:
	cargo fmt

clippy:
	cargo clippy --all-targets -- -D warnings

test:
	cargo test

build:
	cargo build

smoke:
	cargo run -- --version
	cargo run -- help
	cargo run -- perft --help
	cargo run -- board --fen startpos
	cargo run -- board --fen "8/8/8/8/8/8/8/8 w - - 0 1"
	cargo run -- move --parse e2e4
	cargo run -- move --parse e7e8q

check:
	./scripts/local-check.sh
