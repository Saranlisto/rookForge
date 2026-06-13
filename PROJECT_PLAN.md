# Rookforge Project Plan

## Completed

- Day 001: Initial Rust workspace, core and CLI crates, placeholder modules, README, docs, and Rust CI.
- Day 002: Structural FEN parsing, position initialization, board-content tests, local validation script, Makefile workflow, and CI clippy alignment.

## Near-Term Plan

1. Add make/unmake move scaffolding for board state transitions.
2. Add pseudo-legal move generation.
3. Add legal move filtering and check detection.
4. Add perft execution with known validation positions.
5. Add UCI protocol loop after core move generation is stable.

## Deferred

- Search
- Evaluation
- Opening books
- Lichess integration
- Web UI or replay viewer

