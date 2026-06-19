# Rookforge

Rookforge is a from-scratch chess engine project written in Rust. The goal is to build a correct, well-tested classical engine that can grow into a public engineering case study.

For the current public execution ledger, see [EXECUTION_STATUS.md](EXECUTION_STATUS.md).

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
- `movegen`: move vocabulary, pseudo-legal generation, attack detection, and later legal move generation
- `search`: future search algorithms
- `eval`: future handcrafted evaluation
- `uci`: future Universal Chess Interface support

## Current Status

Rookforge is currently a production-grade scaffold with structural FEN parsing, board inspection helpers, UCI-style move parsing, combined pseudo-legal move generation, basic move application, attack detection, legal move filtering, basic perft, and local debug commands. It does not play chess yet.

Execution completed to date:

| Day | Completed |
| --- | --- |
| 001 | Rust workspace, core crate, CLI crate, placeholder modules, README, docs, and CI. |
| 002 | FEN parsing, position initialization, tests, local validation script, and Makefile workflow. |
| 003 | Square utilities, board helpers, pretty board display, FEN serialization, and board debug CLI. |
| 004 | UCI-style move parsing, promotion support, tests, and move debug CLI. |
| 005 | Pseudo-legal pawn move generation and pawn movegen CLI. |
| 006 | Pseudo-legal knight and king move generation and related CLI commands. |
| 007 | Pseudo-legal bishop, rook, and queen sliding move generation and related CLI commands. |
| 008 | Combined all-piece pseudo-legal move generation and `movegen all`. |
| 009 | Basic move application for quiet moves, captures, promotions, counters, castling-right updates, and `apply`. |
| 010 | Attack detection for pawns, knights, kings, sliders, queens, blockers, and CLI attack inspection. |
| 011 | King lookup, check detection, legal move filtering, and `movegen legal`. |
| 012 | Recursive legal-move perft, start-position node tests, and `perft --fen ... --depth ...`. |

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
- Pseudo-legal pawn move generation for pushes, captures, double pushes, and promotions
- Pseudo-legal knight and one-square king move generation
- Pseudo-legal bishop, rook, and queen sliding move generation
- Combined all-piece pseudo-legal move generation for pawns, knights, bishops, rooks, queens, and kings
- Basic move application for quiet moves, captures, promotions, side-to-move updates, move counters, and castling-right updates
- Attack detection for pawns, knights, kings, bishops, rooks, queens, and blockers
- King lookup and check detection
- Legal move filtering by applying pseudo-legal moves and rejecting king-unsafe results
- Basic recursive perft using legal moves
- Human-readable board display for debugging
- Unit tests for the placeholder types
- CLI tests for basic command behavior
- Local validation workflow through `make check`
- Rust GitHub Actions workflow
- Initial architecture and devlog docs

Intentionally not implemented yet:

- Castling execution
- En passant capture
- Unapply/reversible move history
- Perft divide mode
- Search
- Evaluation
- UCI protocol handling

## Roadmap

1. Add castling generation and application.
2. Add en passant generation and application.
3. Add a hardened perft validation suite and divide mode.
4. Add reversible move history and unapply scaffolding.
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
cargo run -- perft --fen startpos --depth 1
cargo run -- perft --fen startpos --depth 2
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
rookforge perft --fen startpos --depth 1
rookforge perft --fen startpos --depth 2
rookforge board --fen startpos
rookforge move --parse e2e4
rookforge movegen pawns --fen startpos
rookforge movegen knights --fen startpos
rookforge movegen kings --fen "8/8/8/8/4K3/8/8/8 w - - 0 1"
rookforge movegen bishops --fen "8/8/8/3B4/8/8/8/8 w - - 0 1"
rookforge movegen rooks --fen "8/8/8/3R4/8/8/8/8 w - - 0 1"
rookforge movegen queens --fen "8/8/8/3Q4/8/8/8/8 w - - 0 1"
rookforge movegen all --fen startpos
rookforge movegen legal --fen startpos
rookforge apply --fen startpos --move e2e4
rookforge attacks --fen "4r3/8/8/8/4K3/8/8/8 w - - 0 1" --square e4 --by black
rookforge attacks --fen startpos --square e4 --by black
rookforge attacks --fen startpos --square e4 --by white
```
