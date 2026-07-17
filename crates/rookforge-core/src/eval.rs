//! Static evaluation.
//!
//! Score convention:
//! - Positive scores are better for White.
//! - Negative scores are better for Black.
//! - Zero means equal material.

use crate::board::{Color, PieceKind, Position, Square};

/// Pawn material value in centipawns.
pub const PAWN_VALUE: i32 = 100;
/// Knight material value in centipawns.
pub const KNIGHT_VALUE: i32 = 320;
/// Bishop material value in centipawns.
pub const BISHOP_VALUE: i32 = 330;
/// Rook material value in centipawns.
pub const ROOK_VALUE: i32 = 500;
/// Queen material value in centipawns.
pub const QUEEN_VALUE: i32 = 900;
/// King material value in centipawns.
pub const KING_VALUE: i32 = 0;

/// Evaluates a position using material only.
///
/// This evaluation is intentionally independent of side to move.
#[must_use]
pub fn evaluate(position: &Position) -> i32 {
    (0..64)
        .filter_map(Square::from_index)
        .filter_map(|square| position.piece_at(square))
        .map(|piece| match piece.color {
            Color::White => material_value(piece.kind),
            Color::Black => -material_value(piece.kind),
        })
        .sum()
}

/// Returns the material value for a piece kind.
#[must_use]
pub const fn material_value(kind: PieceKind) -> i32 {
    match kind {
        PieceKind::Pawn => PAWN_VALUE,
        PieceKind::Knight => KNIGHT_VALUE,
        PieceKind::Bishop => BISHOP_VALUE,
        PieceKind::Rook => ROOK_VALUE,
        PieceKind::Queen => QUEEN_VALUE,
        PieceKind::King => KING_VALUE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::STARTING_POSITION_FEN;
    use crate::movegen::{apply_move, Move};

    fn evaluate_fen(fen: &str) -> i32 {
        let position = Position::from_fen(fen).expect("valid test FEN");

        evaluate(&position)
    }

    #[test]
    fn starting_position_evaluates_to_zero() {
        assert_eq!(evaluate_fen(STARTING_POSITION_FEN), 0);
    }

    #[test]
    fn empty_board_evaluates_to_zero() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/8/8 w - - 0 1"), 0);
    }

    #[test]
    fn white_extra_pawn_evaluates_to_positive_one_hundred() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/4P3/4K2k w - - 0 1"), 100);
    }

    #[test]
    fn black_extra_pawn_evaluates_to_negative_one_hundred() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/4p3/4K2k w - - 0 1"), -100);
    }

    #[test]
    fn white_extra_knight_evaluates_to_positive_three_hundred_twenty() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/8/4KN1k w - - 0 1"), 320);
    }

    #[test]
    fn black_extra_bishop_evaluates_to_negative_three_hundred_thirty() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/8/4Kb1k w - - 0 1"), -330);
    }

    #[test]
    fn white_extra_rook_evaluates_to_positive_five_hundred() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/8/4KR1k w - - 0 1"), 500);
    }

    #[test]
    fn black_extra_queen_evaluates_to_negative_nine_hundred() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/8/4Kq1k w - - 0 1"), -900);
    }

    #[test]
    fn mixed_material_position_evaluates_correctly() {
        assert_eq!(evaluate_fen("8/8/8/8/8/8/4PP2/2n1KB1k w - - 0 1"), 210);
    }

    #[test]
    fn promotion_impact_is_reflected_after_applying_promotion_move() {
        let position =
            Position::from_fen("8/4P3/8/8/8/8/8/4K2k w - - 0 1").expect("valid promotion test FEN");
        let promotion = Move::from_uci("e7e8q").expect("valid promotion move");
        let promoted = apply_move(&position, promotion).expect("promotion applies");

        assert_eq!(evaluate(&position), 100);
        assert_eq!(evaluate(&promoted), 900);
    }
}
