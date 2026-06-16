# Rookforge Project Plan

## Completed

- Day 001: Initial Rust workspace, core and CLI crates, placeholder modules, README, docs, and Rust CI.
- Day 002: Structural FEN parsing, position initialization, board-content tests, local validation script, Makefile workflow, and CI clippy alignment.
- Day 003: Square indexing utilities, board inspection helpers, pretty board display, FEN round-trip serialization, and a CLI board debug command.
- Day 004: UCI-style move representation, parsing, promotion normalization, CLI move debug command, and local smoke checks.
- Day 005: Pseudo-legal pawn move generation for pushes, captures, double pushes, promotions, CLI pawn movegen debugging, and local smoke checks.
- Day 006: Pseudo-legal knight and one-square king move generation, shared leaper occupancy handling, CLI debug commands, and local smoke checks.

## Near-Term Plan

1. Add pseudo-legal sliding piece move generation for bishops, rooks, and queens.
2. Add make/unmake move scaffolding for board state transitions.
3. Add legal move filtering and check detection.
4. Add perft execution with known validation positions.
5. Add castling and en passant once board state transitions are reliable.
6. Add UCI protocol loop after core move generation is stable.

## Deferred

- Search
- Evaluation
- Opening books
- Lichess integration
- Web UI or replay viewer
