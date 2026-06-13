# 0001: Project Direction

## Status

Accepted.

## Context

Rookforge is intended to become a from-scratch Rust chess engine and public portfolio project. The engine should grow in clear layers so correctness, testing, and maintainability remain visible throughout the work.

The first repository milestone should not implement chess behavior. Its purpose is to establish the workspace, module boundaries, basic vocabulary, documentation, and CI checks.

## Decision

Use a Rust workspace with a core library crate and a CLI crate:

- `rookforge-core` owns engine concepts and future engine logic.
- `rookforge-cli` owns the `rookforge` command-line binary.

The initial core module layout is:

- `board`
- `movegen`
- `search`
- `eval`
- `uci`

Only minimal placeholder chess types are introduced at this stage. Real move generation, search, evaluation, and protocol handling are deferred.

## Consequences

This keeps the initial repository small, buildable, and easy to review. It also creates stable places for future work without committing to premature engine internals.

The next architectural decisions should cover board representation, square indexing conventions, FEN parsing, and make/unmake move state.

