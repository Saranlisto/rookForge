//! Fixed-depth search.
//!
//! Search scores are reported from the root side-to-move perspective:
//! positive is good for the side to move, negative is bad for the side to move.

use crate::board::{Color, Position};
use crate::eval::evaluate;
use crate::movegen::{apply_move, generate_legal_moves, Move};

const NEGATIVE_INFINITY: i32 = i32::MIN / 2;

/// Result returned by the current fixed-depth search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score_cp: i32,
    pub depth: u32,
    pub nodes: u64,
}

/// Converts the white-positive static evaluation to side-to-move perspective.
#[must_use]
pub fn evaluate_for_side_to_move(position: &Position) -> i32 {
    match position.side_to_move() {
        Color::White => evaluate(position),
        Color::Black => -evaluate(position),
    }
}

/// Searches a position using plain fixed-depth negamax.
///
/// This intentionally does not use alpha-beta pruning, move ordering,
/// quiescence search, transposition tables, or mate scores.
#[must_use]
pub fn search_best_move(position: &Position, depth: u32) -> SearchResult {
    if depth == 0 {
        return SearchResult {
            best_move: None,
            score_cp: evaluate_for_side_to_move(position),
            depth,
            nodes: 1,
        };
    }

    let mut nodes = 0;
    let mut best_move = None;
    let mut best_score = NEGATIVE_INFINITY;

    for mv in generate_legal_moves(position) {
        let Ok(candidate) = apply_move(position, mv) else {
            continue;
        };
        let score = -negamax(&candidate, depth - 1, &mut nodes);

        if best_move.is_none() || score > best_score {
            best_move = Some(mv);
            best_score = score;
        }
    }

    if best_move.is_none() {
        nodes += 1;
        best_score = evaluate_for_side_to_move(position);
    }

    SearchResult {
        best_move,
        score_cp: best_score,
        depth,
        nodes,
    }
}

fn negamax(position: &Position, depth: u32, nodes: &mut u64) -> i32 {
    *nodes += 1;

    if depth == 0 {
        return evaluate_for_side_to_move(position);
    }

    let mut best_score = NEGATIVE_INFINITY;
    let mut found_move = false;

    for mv in generate_legal_moves(position) {
        let Ok(candidate) = apply_move(position, mv) else {
            continue;
        };
        found_move = true;
        best_score = best_score.max(-negamax(&candidate, depth - 1, nodes));
    }

    if found_move {
        best_score
    } else {
        evaluate_for_side_to_move(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{PieceKind, STARTING_POSITION_FEN};

    fn position(fen: &str) -> Position {
        Position::from_fen(fen).expect("valid test FEN")
    }

    #[test]
    fn depth_zero_returns_static_evaluation() {
        let position = position("8/8/8/8/8/8/4P3/4K2k w - - 0 1");
        let result = search_best_move(&position, 0);

        assert_eq!(
            result,
            SearchResult {
                best_move: None,
                score_cp: 100,
                depth: 0,
                nodes: 1,
            }
        );
    }

    #[test]
    fn evaluation_flips_for_black_to_move() {
        let position = position("8/8/8/8/8/8/4P3/4K2k b - - 0 1");

        assert_eq!(evaluate(&position), 100);
        assert_eq!(evaluate_for_side_to_move(&position), -100);
    }

    #[test]
    fn depth_one_returns_legal_move_from_starting_position() {
        let position = position(STARTING_POSITION_FEN);
        let legal_moves = generate_legal_moves(&position);
        let result = search_best_move(&position, 1);

        assert!(result.best_move.is_some());
        assert!(legal_moves.contains(&result.best_move.expect("best move")));
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
        let result = search_best_move(&position, 1);
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
    }

    #[test]
    fn normal_position_searches_nodes() {
        let position = position(STARTING_POSITION_FEN);
        let result = search_best_move(&position, 1);

        assert!(result.nodes > 0);
    }
}
