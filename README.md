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

Rookforge is currently a production-grade scaffold. It does not play chess yet.

Implemented:

- Cargo workspace
- Core library crate
- CLI crate with `rookforge` binary
- Placeholder chess types: `Color`, `PieceKind`, `Piece`, `Square`, and `Move`
- Unit tests for the placeholder types
- CLI tests for basic command behavior
- Rust GitHub Actions workflow
- Initial architecture and devlog docs

Intentionally not implemented yet:

- Board state
- FEN parsing
- Legal move generation
- Perft execution
- Search
- Evaluation
- UCI protocol handling

## Roadmap

1. Add board state and FEN parsing.
2. Add make/unmake move structure.
3. Add pseudo-legal move generation.
4. Add legal move filtering and check detection.
5. Add perft with known test positions.
6. Add UCI command loop.
7. Add search and evaluation.
8. Add benchmarks and strength testing.
9. Add Lichess bot bridge after the core engine is stable.

## Build And Test

```bash
cargo fmt
cargo test
cargo build
```

CLI examples:

```bash
rookforge --version
rookforge help
rookforge perft --help
```

