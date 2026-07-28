//! Fixed-depth alpha-beta search with quiescence.
//!
//! Search scores are reported from the root side-to-move perspective:
//! positive is good for the side to move, negative is bad for the side to move.

use std::time::{Duration, Instant};

use crate::board::{Color, PieceKind, Position};
use crate::eval::{evaluate, material_value};
use crate::movegen::{apply_move, generate_legal_moves, is_in_check, Move};

const INFINITY: i32 = 1_000_000;

/// Mate score used before depth adjustment.
pub const MATE_SCORE: i32 = 100_000;
/// Maximum capture/promotion plies searched by quiescence.
pub const MAX_QUIESCENCE_PLY: u32 = 8;

/// Search algorithm used to produce a result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchKind {
    AlphaBeta,
    AlphaBetaQuiescence,
}

impl SearchKind {
    /// Returns the stable CLI/documentation label for this search kind.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::AlphaBeta => "alpha-beta",
            Self::AlphaBetaQuiescence => "alpha-beta+quiescence",
        }
    }
}

/// High-level outcome of a search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOutcome {
    BestMove,
    Checkmate,
    Stalemate,
}

impl SearchOutcome {
    /// Returns the stable CLI/documentation label for this outcome.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BestMove => "bestmove",
            Self::Checkmate => "checkmate",
            Self::Stalemate => "stalemate",
        }
    }
}

/// Options for fixed-depth search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchOptions {
    pub depth: u32,
    pub quiescence: bool,
}

impl SearchOptions {
    /// Creates default search options for a fixed depth.
    #[must_use]
    pub const fn new(depth: u32) -> Self {
        Self {
            depth,
            quiescence: true,
        }
    }

    /// Creates fixed-depth search options with quiescence disabled.
    #[must_use]
    pub const fn without_quiescence(depth: u32) -> Self {
        Self {
            depth,
            quiescence: false,
        }
    }

    const fn kind(self) -> SearchKind {
        if self.quiescence {
            SearchKind::AlphaBetaQuiescence
        } else {
            SearchKind::AlphaBeta
        }
    }
}

/// Result returned by fixed-depth search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score_cp: i32,
    pub depth: u32,
    pub nodes: u64,
    pub qnodes: u64,
    pub elapsed_ms: u128,
    pub nodes_per_second: u64,
    pub outcome: SearchOutcome,
    pub search: SearchKind,
}

#[derive(Debug, Default)]
struct SearchStats {
    nodes: u64,
    qnodes: u64,
}

/// Converts the white-positive static evaluation to side-to-move perspective.
#[must_use]
pub fn evaluate_for_side_to_move(position: &Position) -> i32 {
    match position.side_to_move() {
        Color::White => evaluate(position),
        Color::Black => -evaluate(position),
    }
}

/// Returns true if the side to move is checkmated.
#[must_use]
pub fn is_checkmate(position: &Position) -> bool {
    generate_legal_moves(position).is_empty() && is_in_check(position, position.side_to_move())
}

/// Returns true if the side to move is stalemated.
#[must_use]
pub fn is_stalemate(position: &Position) -> bool {
    generate_legal_moves(position).is_empty() && !is_in_check(position, position.side_to_move())
}

/// Searches a position using fixed-depth alpha-beta with quiescence enabled.
#[must_use]
pub fn search_best_move(position: &Position, depth: u32) -> SearchResult {
    search_best_move_with_options(position, SearchOptions::new(depth))
}

/// Searches a position using fixed-depth alpha-beta without quiescence.
#[must_use]
pub fn search_best_move_without_quiescence(position: &Position, depth: u32) -> SearchResult {
    search_best_move_with_options(position, SearchOptions::without_quiescence(depth))
}

/// Searches a position with explicit fixed-depth search options.
#[must_use]
pub fn search_best_move_with_options(position: &Position, options: SearchOptions) -> SearchResult {
    let started_at = Instant::now();
    let mut stats = SearchStats::default();

    if options.depth == 0 {
        stats.nodes = 1;
        return finish_result(
            None,
            evaluate_for_side_to_move(position),
            options.depth,
            SearchOutcome::BestMove,
            options.kind(),
            stats,
            started_at.elapsed(),
        );
    }

    let legal_moves = generate_legal_moves(position);

    if legal_moves.is_empty() {
        stats.nodes = 1;
        return finish_result(
            None,
            terminal_score(position, 0),
            options.depth,
            terminal_outcome(position),
            options.kind(),
            stats,
            started_at.elapsed(),
        );
    }

    let mut alpha = -INFINITY;
    let beta = INFINITY;
    let mut best_move = None;
    let mut best_score = -INFINITY;

    for mv in ordered_moves(position, legal_moves) {
        let Ok(candidate) = apply_move(position, mv) else {
            continue;
        };
        let score = -alpha_beta(
            &candidate,
            options.depth - 1,
            -beta,
            -alpha,
            &mut stats,
            1,
            options.quiescence,
        );

        if best_move.is_none() || score > best_score {
            best_move = Some(mv);
            best_score = score;
        }

        alpha = alpha.max(best_score);
    }

    finish_result(
        best_move,
        best_score,
        options.depth,
        SearchOutcome::BestMove,
        options.kind(),
        stats,
        started_at.elapsed(),
    )
}

fn alpha_beta(
    position: &Position,
    depth: u32,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    ply: u32,
    quiescence_enabled: bool,
) -> i32 {
    stats.nodes = stats.nodes.saturating_add(1);

    let legal_moves = generate_legal_moves(position);

    if legal_moves.is_empty() {
        return terminal_score(position, ply);
    }

    if depth == 0 {
        return if quiescence_enabled {
            quiescence(position, alpha, beta, stats, 0, ply)
        } else {
            evaluate_for_side_to_move(position)
        };
    }

    for mv in ordered_moves(position, legal_moves) {
        let Ok(candidate) = apply_move(position, mv) else {
            continue;
        };
        let score = -alpha_beta(
            &candidate,
            depth - 1,
            -beta,
            -alpha,
            stats,
            ply.saturating_add(1),
            quiescence_enabled,
        );

        if score >= beta {
            return beta;
        }

        alpha = alpha.max(score);
    }

    alpha
}

fn quiescence(
    position: &Position,
    mut alpha: i32,
    beta: i32,
    stats: &mut SearchStats,
    qply: u32,
    ply: u32,
) -> i32 {
    stats.qnodes = stats.qnodes.saturating_add(1);

    let legal_moves = generate_legal_moves(position);

    if legal_moves.is_empty() {
        return terminal_score(position, ply);
    }

    let stand_pat = evaluate_for_side_to_move(position);

    if qply >= MAX_QUIESCENCE_PLY {
        return stand_pat;
    }

    if stand_pat >= beta {
        return beta;
    }

    alpha = alpha.max(stand_pat);

    for mv in tactical_moves(position, legal_moves) {
        let Ok(candidate) = apply_move(position, mv) else {
            continue;
        };
        let score = -quiescence(
            &candidate,
            -beta,
            -alpha,
            stats,
            qply.saturating_add(1),
            ply.saturating_add(1),
        );

        if score >= beta {
            return beta;
        }

        alpha = alpha.max(score);
    }

    alpha
}

/// Orders moves with a small deterministic tactical preference.
#[must_use]
pub fn ordered_moves(position: &Position, moves: Vec<Move>) -> Vec<Move> {
    let mut moves = moves;
    moves.sort_by(|left, right| {
        move_order_score(position, *right)
            .cmp(&move_order_score(position, *left))
            .then_with(|| left.to_uci().cmp(&right.to_uci()))
    });
    moves
}

fn tactical_moves(position: &Position, moves: Vec<Move>) -> Vec<Move> {
    ordered_moves(position, moves)
        .into_iter()
        .filter(|&mv| is_tactical_move(position, mv))
        .collect()
}

fn is_tactical_move(position: &Position, mv: Move) -> bool {
    mv.promotion.is_some() || captured_piece_kind(position, mv).is_some()
}

fn move_order_score(position: &Position, mv: Move) -> i32 {
    let promotion_score = mv.promotion.map_or(0, material_value);
    let capture_score = captured_piece_kind(position, mv).map_or(0, material_value);

    promotion_score.saturating_add(capture_score)
}

fn captured_piece_kind(position: &Position, mv: Move) -> Option<PieceKind> {
    position
        .piece_at(mv.to)
        .map(|piece| piece.kind)
        .or_else(|| en_passant_capture_kind(position, mv))
}

fn en_passant_capture_kind(position: &Position, mv: Move) -> Option<PieceKind> {
    let moving_piece = position.piece_at(mv.from)?;

    if moving_piece.kind == PieceKind::Pawn
        && position.en_passant_target() == Some(mv.to)
        && position.piece_at(mv.to).is_none()
        && mv.from.file().abs_diff(mv.to.file()) == 1
    {
        Some(PieceKind::Pawn)
    } else {
        None
    }
}

fn terminal_outcome(position: &Position) -> SearchOutcome {
    if is_in_check(position, position.side_to_move()) {
        SearchOutcome::Checkmate
    } else {
        SearchOutcome::Stalemate
    }
}

fn terminal_score(position: &Position, ply: u32) -> i32 {
    match terminal_outcome(position) {
        SearchOutcome::Checkmate => -mate_score(ply),
        SearchOutcome::Stalemate => 0,
        SearchOutcome::BestMove => evaluate_for_side_to_move(position),
    }
}

fn mate_score(ply: u32) -> i32 {
    let ply = i32::try_from(ply).unwrap_or(MATE_SCORE - 1);

    MATE_SCORE.saturating_sub(ply).max(1)
}

fn finish_result(
    best_move: Option<Move>,
    score_cp: i32,
    depth: u32,
    outcome: SearchOutcome,
    search: SearchKind,
    stats: SearchStats,
    elapsed: Duration,
) -> SearchResult {
    SearchResult {
        best_move,
        score_cp,
        depth,
        nodes: stats.nodes,
        qnodes: stats.qnodes,
        elapsed_ms: elapsed.as_millis(),
        nodes_per_second: nodes_per_second(stats.nodes.saturating_add(stats.qnodes), elapsed),
        outcome,
        search,
    }
}

fn nodes_per_second(nodes: u64, elapsed: Duration) -> u64 {
    let rate = (u128::from(nodes) * 1_000_000_000)
        .checked_div(elapsed.as_nanos())
        .unwrap_or(u128::from(nodes));
    rate.min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{PieceKind, STARTING_POSITION_FEN};

    fn position(fen: &str) -> Position {
        Position::from_fen(fen).expect("valid test FEN")
    }

    #[test]
    fn depth_zero_returns_static_evaluation_without_quiescence() {
        let position = position("8/8/8/8/8/8/4P3/4K2k w - - 0 1");
        let result = search_best_move_without_quiescence(&position, 0);

        assert_eq!(result.best_move, None);
        assert_eq!(result.score_cp, 100);
        assert_eq!(result.depth, 0);
        assert_eq!(result.nodes, 1);
        assert_eq!(result.qnodes, 0);
    }

    #[test]
    fn evaluation_flips_for_black_to_move() {
        let position = position("8/8/8/8/8/8/4P3/4K2k b - - 0 1");

        assert_eq!(evaluate(&position), 100);
        assert_eq!(evaluate_for_side_to_move(&position), -100);
    }

    #[test]
    fn alpha_beta_returns_legal_move_from_starting_position() {
        let position = position(STARTING_POSITION_FEN);
        let legal_moves = generate_legal_moves(&position);
        let result = search_best_move(&position, 1);

        assert!(result.best_move.is_some());
        assert!(legal_moves.contains(&result.best_move.expect("best move")));
        assert_eq!(result.search, SearchKind::AlphaBetaQuiescence);
    }

    #[test]
    fn search_result_move_is_legal() {
        let position = position("k7/5r2/8/8/8/8/8/4KQ2 w - - 0 1");
        let result = search_best_move(&position, 1);
        let best_move = result.best_move.expect("best move");

        assert!(generate_legal_moves(&position).contains(&best_move));
    }

    #[test]
    fn search_captures_hanging_material() {
        let position = position("k7/5r2/8/8/8/8/8/4KQ2 w - - 0 1");
        let result = search_best_move_without_quiescence(&position, 1);
        let best_move = result.best_move.expect("best move");

        assert_eq!(best_move.to_uci(), "f1f7");
        assert_eq!(
            position
                .piece_at(best_move.to)
                .expect("captured piece")
                .kind,
            PieceKind::Rook
        );
        assert_eq!(result.score_cp, 900);
    }

    #[test]
    fn search_prefers_queen_promotion() {
        let position = position("k7/4P3/8/8/8/8/8/4K3 w - - 0 1");
        let result = search_best_move(&position, 1);

        assert_eq!(result.best_move.expect("best promotion").to_uci(), "e7e8q");
        assert_eq!(result.score_cp, 900);
    }

    #[test]
    fn empty_position_returns_no_best_move() {
        let position = position("8/8/8/8/8/8/8/8 w - - 0 1");
        let result = search_best_move(&position, 2);

        assert_eq!(result.best_move, None);
        assert_eq!(result.score_cp, 0);
        assert_eq!(result.nodes, 1);
        assert_eq!(result.outcome, SearchOutcome::Stalemate);
    }

    #[test]
    fn normal_position_searches_nodes() {
        let position = position(STARTING_POSITION_FEN);
        let result = search_best_move(&position, 1);

        assert!(result.nodes > 0);
        assert!(result.qnodes > 0);
        assert!(result.nodes_per_second > 0);
    }

    #[test]
    fn checkmate_detection_uses_legal_moves_and_check() {
        let position = position("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");

        assert!(is_checkmate(&position));
        assert!(!is_stalemate(&position));
    }

    #[test]
    fn stalemate_detection_uses_legal_moves_and_check() {
        let position = position("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");

        assert!(is_stalemate(&position));
        assert!(!is_checkmate(&position));
    }

    #[test]
    fn search_returns_checkmate_outcome_gracefully() {
        let position = position("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");
        let result = search_best_move(&position, 3);

        assert_eq!(result.best_move, None);
        assert_eq!(result.outcome, SearchOutcome::Checkmate);
        assert_eq!(result.score_cp, -MATE_SCORE);
    }

    #[test]
    fn search_returns_stalemate_outcome_gracefully() {
        let position = position("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1");
        let result = search_best_move(&position, 3);

        assert_eq!(result.best_move, None);
        assert_eq!(result.outcome, SearchOutcome::Stalemate);
        assert_eq!(result.score_cp, 0);
    }

    #[test]
    fn mate_score_outranks_material() {
        let position = position("7k/8/5K2/8/8/8/6Q1/8 w - - 0 1");
        let result = search_best_move_without_quiescence(&position, 1);

        assert!(result.best_move.is_some());
        assert!(result.score_cp > material_value(PieceKind::Queen));
    }

    #[test]
    fn quiescence_returns_static_evaluation_when_no_captures() {
        let position = position("8/8/8/8/8/8/4P3/4K2k w - - 0 1");
        let mut stats = SearchStats::default();

        assert_eq!(
            quiescence(&position, -INFINITY, INFINITY, &mut stats, 0, 0),
            100
        );
        assert_eq!(stats.qnodes, 1);
    }

    #[test]
    fn quiescence_considers_obvious_winning_capture() {
        let position = position("k7/5r2/8/8/8/8/8/4KQ2 w - - 0 1");
        let mut stats = SearchStats::default();

        assert_eq!(
            quiescence(&position, -INFINITY, INFINITY, &mut stats, 0, 0),
            900
        );
        assert!(stats.qnodes > 1);
    }

    #[test]
    fn quiescence_tactical_moves_exclude_quiet_moves() {
        let position = position("k7/5r2/8/8/8/8/8/4KQ2 w - - 0 1");
        let moves = tactical_moves(&position, generate_legal_moves(&position))
            .into_iter()
            .map(Move::to_uci)
            .collect::<Vec<_>>();

        assert!(moves.contains(&"f1f7".to_string()));
        assert!(!moves.contains(&"e1d1".to_string()));
    }

    #[test]
    fn quiescence_includes_promotions() {
        let position = position("k7/4P3/8/8/8/8/8/4K3 w - - 0 1");
        let moves = tactical_moves(&position, generate_legal_moves(&position))
            .into_iter()
            .map(Move::to_uci)
            .collect::<Vec<_>>();

        assert!(moves.contains(&"e7e8q".to_string()));
    }

    #[test]
    fn quiescence_respects_depth_guard() {
        let position = position("k7/5r2/8/8/8/8/8/4KQ2 w - - 0 1");
        let mut stats = SearchStats::default();

        assert_eq!(
            quiescence(
                &position,
                -INFINITY,
                INFINITY,
                &mut stats,
                MAX_QUIESCENCE_PLY,
                0,
            ),
            400
        );
        assert_eq!(stats.qnodes, 1);
    }

    #[test]
    fn no_quiescence_path_reports_alpha_beta_kind() {
        let position = position(STARTING_POSITION_FEN);
        let result = search_best_move_without_quiescence(&position, 1);

        assert_eq!(result.search, SearchKind::AlphaBeta);
        assert_eq!(result.qnodes, 0);
    }
}
