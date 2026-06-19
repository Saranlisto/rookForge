# Rookforge Project Plan

For the public execution ledger, see [EXECUTION_STATUS.md](EXECUTION_STATUS.md).

## Completed

- Day 001: Initial Rust workspace, core and CLI crates, placeholder modules, README, docs, and Rust CI.
- Day 002: Structural FEN parsing, position initialization, board-content tests, local validation script, Makefile workflow, and CI clippy alignment.
- Day 003: Square indexing utilities, board inspection helpers, pretty board display, FEN round-trip serialization, and a CLI board debug command.
- Day 004: UCI-style move representation, parsing, promotion normalization, CLI move debug command, and local smoke checks.
- Day 005: Pseudo-legal pawn move generation for pushes, captures, double pushes, promotions, CLI pawn movegen debugging, and local smoke checks.
- Day 006: Pseudo-legal knight and one-square king move generation, shared leaper occupancy handling, CLI debug commands, and local smoke checks.
- Day 007: Pseudo-legal bishop, rook, and queen sliding move generation, shared ray traversal, CLI debug commands, and local smoke checks.
- Day 008: Combined all-piece pseudo-legal move generation, start-position count of 20, CLI `movegen all`, and local smoke checks.
- Day 009: Basic move application for quiet moves, captures, promotions, counters, castling-right updates, CLI `apply`, and local smoke checks.
- Day 010: Attack detection for pawns, knights, kings, sliders, queens, blockers, side-to-move independence, CLI `attacks`, and local smoke checks.
- Day 011: King lookup, check detection, legal move filtering from pseudo-legal moves, CLI `movegen legal`, and local smoke checks.

## Near-Term Plan

1. Add perft execution with known validation positions.
2. Add castling generation and application.
3. Add en passant generation and application.
4. Add reversible move history and unapply scaffolding for search and perft.
5. Add UCI protocol loop after core move generation is stable.

## Deferred

- Search
- Evaluation
- Opening books
- Lichess integration
- Web UI or replay viewer
