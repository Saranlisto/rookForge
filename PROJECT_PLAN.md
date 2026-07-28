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
- Day 012: Basic recursive perft using legal moves, start-position depth 1 and 2 validation, CLI `perft --fen ... --depth ...`, and local smoke checks.
- Day 013: Castling generation, castling legality checks, castling move application, castling tests, and castling legal-move smoke checks.
- Day 014: En passant target handling, en passant generation, en passant capture application, discovered-check filtering, tests, and local smoke checks.
- Day 015: Hardened perft validation suite, Kiwipete reference counts, perft divide output, timing/throughput CLI fields, and `make perft`.
- Day 016: Material-only static evaluation, white-positive centipawn convention, evaluation tests, CLI `eval --fen`, and local smoke checks.
- Day 017: Fixed-depth negamax search, best-move selection, side-to-move score convention, CLI `search --fen ... --depth ...`, and local smoke checks.
- Day 018: Alpha-beta search, checkmate/stalemate scoring, tactical move ordering, quiescence search, richer search stats, and local smoke checks.
- Day 019: CLI and release polish with consistent help sections, cleaner invalid FEN/move/depth errors, search JSON output, release-build docs, and `make release`.

## Near-Term Plan

1. Add iterative deepening and a basic time budget.
2. Add UCI protocol loop after core move generation is stable.
3. Add reversible move history and unapply scaffolding for search and perft.
4. Add transposition tables.
5. Add piece-square tables and deeper handcrafted evaluation terms.

## Deferred

- Transposition tables
- Advanced evaluation terms
- Opening books
- Lichess integration
- Web UI or replay viewer
