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

check:
	./scripts/local-check.sh

