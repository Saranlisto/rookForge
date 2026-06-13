//! Move generation module scaffold.
//!
//! Legal and pseudo-legal generation are intentionally not implemented yet.

use crate::board::{PieceKind, Square};

/// A chess move from one square to another, with an optional promotion kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceKind>,
}

impl Move {
    /// Creates a non-promotion move.
    #[must_use]
    pub const fn quiet(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    /// Creates a move with an explicit promotion kind.
    #[must_use]
    pub const fn promotion(from: Square, to: Square, promotion: PieceKind) -> Self {
        Self {
            from,
            to,
            promotion: Some(promotion),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quiet_move_has_no_promotion() {
        let from = Square::new(1, 0).expect("valid from square");
        let to = Square::new(2, 2).expect("valid to square");

        let mv = Move::quiet(from, to);

        assert_eq!(mv.from, from);
        assert_eq!(mv.to, to);
        assert_eq!(mv.promotion, None);
    }

    #[test]
    fn promotion_move_records_piece_kind() {
        let from = Square::new(0, 6).expect("valid from square");
        let to = Square::new(0, 7).expect("valid to square");

        let mv = Move::promotion(from, to, PieceKind::Queen);

        assert_eq!(mv.promotion, Some(PieceKind::Queen));
    }
}
