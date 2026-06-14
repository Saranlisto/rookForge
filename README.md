# Rookforge

Rookforge is a from-scratch chess engine project written in Rust. The goal is to build a correct, well-tested classical engine that can grow into a public engineering case study.

## Project Objective

The project is focused on building a serious chess engine in measured stages:

- Correct board representation and rules
- Legal move generation verified by perft
- Search with negamax, alpha-beta pruning, and iterative deepening
- Handcrafted evaluation
- UCI compatibility
- Later tooling for benchmarking and Lichess integration

Correctness comes before strength. The first milestones are intentionally small so every layer can be tested before more engine behavior is added.

## Architecture Direction

The repository is a Rust workspace with two initial crates:

- `rookforge-core`: engine modules and shared chess types
- `rookforge-cli`: command-line entry point that builds the `rookforge` binary

Initial core modules:

- `board`: board-level primitives such as colors, piece kinds, pieces, and squares
- `movegen`: move vocabulary and, later, pseudo-legal and legal move generation
- `search`: future search algorithms
- `eval`: future handcrafted evaluation
- `uci`: future Universal Chess Interface support

## Current Status

Rookforge is currently a production-grade scaffold with structural FEN parsing, board inspection helpers, UCI-style move parsing, and local debug commands. It does not play chess yet.

Implemented:

- Cargo workspace
- Core library crate
- CLI crate with `rookforge` binary
- Core chess types: `Color`, `PieceKind`, `Piece`, `Square`, and `Move`
- Position storage for piece placement, side to move, castling rights, en passant target, and move counters
- Structural FEN parsing for standard Forsyth-Edwards Notation
- FEN serialization and round-trip tests
- Square indexing utilities using `a1 = 0` through `h8 = 63`
- UCI-style move parsing and serialization such as `e2e4` and `e7e8q`
- Human-readable board display for debugging
- Unit tests for the placeholder types
- CLI tests for basic command behavior
- Local validation workflow through `make check`
- Rust GitHub Actions workflow
- Initial architecture and devlog docs

Intentionally not implemented yet:

- Legal move generation
- Perft execution
- Search
- Evaluation
- UCI protocol handling

## Roadmap

1. Add make/unmake move structure.
2. Add pseudo-legal move generation.
3. Add legal move filtering and check detection.
4. Add perft with known test positions.
5. Add UCI command loop.
6. Add search and evaluation.
7. Add benchmarks and strength testing.
8. Add Lichess bot bridge after the core engine is stable.

## Local Development

Run the full local validation workflow before every commit:

```bash
make check
```

This runs formatting checks, clippy with warnings treated as errors, tests, build, and CLI smoke checks.

Useful local smoke commands:

```bash
cargo run -- board --fen startpos
cargo run -- board --fen "8/8/8/8/8/8/8/8 w - - 0 1"
cargo run -- move --parse e2e4
cargo run -- move --parse e7e8q
```

## Build And Test

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

CLI examples:

```bash
rookforge --version
rookforge help
rookforge perft --help
rookforge board --fen startpos
rookforge move --parse e2e4
```
