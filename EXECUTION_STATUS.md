# Rookforge Execution Status

This file is the public progress ledger for Rookforge. It summarizes what has been executed so far and what is intentionally still pending.

## Current Snapshot

Rookforge is in the early engine-core phase. The repository can parse positions, inspect boards, parse UCI-style moves, and generate pseudo-legal moves for all piece types. It does not yet validate king safety or play a legal chess game.

## Completed Work

| Day | Area | Execution Completed |
| --- | --- | --- |
| 001 | Repository scaffold | Created the Rust workspace, core crate, CLI crate, placeholder modules, README, architecture docs, devlog, and Rust CI. |
| 002 | FEN and local checks | Added structural FEN parsing, `Position`, castling rights, en passant square storage, move counters, FEN tests, `scripts/local-check.sh`, and `Makefile` checks. |
| 003 | Board inspection | Added square utilities, board helpers, FEN serialization, pretty board rendering, and `rookforge board --fen ...`. |
| 004 | Move notation | Added `Move`, UCI-style move parsing/serialization, promotion support, tests, and `rookforge move --parse ...`. |
| 005 | Pawn movegen | Added pseudo-legal pawn pushes, captures, double pushes, promotions, tests, and `rookforge movegen pawns`. |
| 006 | Leaper movegen | Added pseudo-legal knight and one-square king moves, shared leaper occupancy handling, tests, and `movegen knights` / `movegen kings`. |
| 007 | Sliding movegen | Added pseudo-legal bishop, rook, and queen moves, shared ray traversal, tests, and `movegen bishops` / `movegen rooks` / `movegen queens`. |
| 008 | Combined movegen | Added `generate_pseudo_legal_moves`, all-piece pseudo-legal CLI debug output through `movegen all`, and starting-position count coverage of 20 moves. |

## Public Commands Available

```bash
rookforge --version
rookforge help
rookforge perft --help
rookforge board --fen startpos
rookforge move --parse e2e4
rookforge movegen pawns --fen startpos
rookforge movegen knights --fen startpos
rookforge movegen kings --fen "8/8/8/8/4K3/8/8/8 w - - 0 1"
rookforge movegen bishops --fen "8/8/8/3B4/8/8/8/8 w - - 0 1"
rookforge movegen rooks --fen "8/8/8/3R4/8/8/8/8 w - - 0 1"
rookforge movegen queens --fen "8/8/8/3Q4/8/8/8/8 w - - 0 1"
rookforge movegen all --fen startpos
```

## Current Validation Workflow

```bash
make check
```

This runs formatting checks, clippy with warnings treated as errors, tests, build, and CLI smoke checks.

## Intentionally Not Implemented Yet

- Legal move filtering
- Check detection
- Castling move generation
- En passant move generation
- Make/unmake move transitions
- Real perft execution
- Search
- Evaluation
- UCI engine protocol loop
- Opening books
- Lichess integration
- Web UI or replay viewer

## Next Recommended Execution

Day 009 should add make/unmake move scaffolding and reversible board-state transitions. That gives the engine a safe base for legal move filtering, perft, and later search.
