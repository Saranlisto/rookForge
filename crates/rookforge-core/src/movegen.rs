//! Move representation and parsing scaffold.
//!
//! Legal and pseudo-legal generation are intentionally not implemented yet.

use std::fmt;
use std::str::FromStr;

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
    pub const fn promotion(
        from: Square,
        to: Square,
        promotion: PieceKind,
    ) -> Result<Self, MoveParseError> {
        if !is_valid_promotion_piece(promotion) {
            return Err(MoveParseError::InvalidPromotionKind(promotion));
        }

        if !is_promotion_rank(to) {
            return Err(MoveParseError::InvalidPromotionRank { target: to });
        }

        Ok(Self {
            from,
            to,
            promotion: Some(promotion),
        })
    }

    /// Parses a UCI-style long algebraic move such as `e2e4` or `e7e8q`.
    pub fn from_uci(value: &str) -> Result<Self, MoveParseError> {
        value.parse()
    }

    /// Serializes the move into normalized UCI-style long algebraic notation.
    #[must_use]
    pub fn to_uci(self) -> String {
        let mut output = format!("{}{}", self.from.to_algebraic(), self.to.to_algebraic());

        if let Some(promotion) = self.promotion {
            output.push(promotion_to_uci_char(promotion));
        }

        output
    }
}

impl FromStr for Move {
    type Err = MoveParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let chars = value.chars().collect::<Vec<_>>();

        match chars.len() {
            4 | 5 => {}
            found => return Err(MoveParseError::InvalidLength { found }),
        }

        let from_value = chars[0..2].iter().collect::<String>();
        let to_value = chars[2..4].iter().collect::<String>();
        let from = Square::from_algebraic(&from_value)
            .ok_or_else(|| MoveParseError::InvalidSourceSquare(from_value.clone()))?;
        let to = Square::from_algebraic(&to_value)
            .ok_or_else(|| MoveParseError::InvalidTargetSquare(to_value.clone()))?;
        let promotion = chars
            .get(4)
            .copied()
            .map(parse_promotion_piece)
            .transpose()?;

        if promotion.is_some() && !is_promotion_rank(to) {
            return Err(MoveParseError::InvalidPromotionRank { target: to });
        }

        Ok(Self {
            from,
            to,
            promotion,
        })
    }
}

fn parse_promotion_piece(value: char) -> Result<PieceKind, MoveParseError> {
    match value.to_ascii_lowercase() {
        'q' => Ok(PieceKind::Queen),
        'r' => Ok(PieceKind::Rook),
        'b' => Ok(PieceKind::Bishop),
        'n' => Ok(PieceKind::Knight),
        _ => Err(MoveParseError::InvalidPromotionPiece(value)),
    }
}

const fn is_valid_promotion_piece(kind: PieceKind) -> bool {
    matches!(
        kind,
        PieceKind::Queen | PieceKind::Rook | PieceKind::Bishop | PieceKind::Knight
    )
}

const fn is_promotion_rank(square: Square) -> bool {
    matches!(square.rank(), 0 | 7)
}

const fn promotion_to_uci_char(kind: PieceKind) -> char {
    match kind {
        PieceKind::Queen => 'q',
        PieceKind::Rook => 'r',
        PieceKind::Bishop => 'b',
        PieceKind::Knight => 'n',
        PieceKind::King | PieceKind::Pawn => '?',
    }
}

/// Errors produced when parsing structurally invalid UCI-style moves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveParseError {
    InvalidLength { found: usize },
    InvalidSourceSquare(String),
    InvalidTargetSquare(String),
    InvalidPromotionPiece(char),
    InvalidPromotionKind(PieceKind),
    InvalidPromotionRank { target: Square },
}

impl fmt::Display for MoveParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength { found } => {
                write!(formatter, "expected 4 or 5 move characters, found {found}")
            }
            Self::InvalidSourceSquare(value) => {
                write!(formatter, "invalid source square `{value}`")
            }
            Self::InvalidTargetSquare(value) => {
                write!(formatter, "invalid target square `{value}`")
            }
            Self::InvalidPromotionPiece(value) => {
                write!(
                    formatter,
                    "invalid promotion piece `{value}`; expected q, r, b, or n"
                )
            }
            Self::InvalidPromotionKind(kind) => {
                write!(formatter, "invalid promotion kind `{kind:?}`")
            }
            Self::InvalidPromotionRank { target } => {
                write!(
                    formatter,
                    "promotion target `{}` must be on rank 1 or 8",
                    target.to_algebraic()
                )
            }
        }
    }
}

impl std::error::Error for MoveParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(value: &str) -> Square {
        Square::from_algebraic(value).expect("valid test square")
    }

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

        let mv = Move::promotion(from, to, PieceKind::Queen).expect("valid promotion");

        assert_eq!(mv.promotion, Some(PieceKind::Queen));
    }

    #[test]
    fn promotion_move_rejects_king_and_pawn() {
        let from = square("e7");
        let to = square("e8");

        assert_eq!(
            Move::promotion(from, to, PieceKind::King),
            Err(MoveParseError::InvalidPromotionKind(PieceKind::King))
        );
        assert_eq!(
            Move::promotion(from, to, PieceKind::Pawn),
            Err(MoveParseError::InvalidPromotionKind(PieceKind::Pawn))
        );
    }

    #[test]
    fn parses_normal_move() {
        let mv = Move::from_uci("e2e4").expect("valid move");

        assert_eq!(mv.from, square("e2"));
        assert_eq!(mv.to, square("e4"));
        assert_eq!(mv.promotion, None);
        assert_eq!(mv.to_uci(), "e2e4");
    }

    #[test]
    fn parses_knight_move() {
        let mv = Move::from_uci("g1f3").expect("valid knight move");

        assert_eq!(mv.from, square("g1"));
        assert_eq!(mv.to, square("f3"));
        assert_eq!(mv.promotion, None);
        assert_eq!(mv.to_uci(), "g1f3");
    }

    #[test]
    fn parses_promotion_moves() {
        let cases = [
            ("e7e8q", PieceKind::Queen),
            ("e7e8r", PieceKind::Rook),
            ("e7e8b", PieceKind::Bishop),
            ("e7e8n", PieceKind::Knight),
            ("a2a1q", PieceKind::Queen),
        ];

        for (value, promotion) in cases {
            let mv = Move::from_uci(value).expect("valid promotion move");

            assert_eq!(mv.promotion, Some(promotion));
            assert_eq!(mv.to_uci(), value);
        }
    }

    #[test]
    fn normalizes_uppercase_promotion_moves() {
        let queen = Move::from_uci("e7e8Q").expect("valid uppercase promotion");
        let knight = Move::from_uci("a2a1N").expect("valid uppercase promotion");

        assert_eq!(queen.promotion, Some(PieceKind::Queen));
        assert_eq!(queen.to_uci(), "e7e8q");
        assert_eq!(knight.promotion, Some(PieceKind::Knight));
        assert_eq!(knight.to_uci(), "a2a1n");
    }

    #[test]
    fn rejects_invalid_source_square() {
        assert_eq!(
            Move::from_uci("i2e4"),
            Err(MoveParseError::InvalidSourceSquare("i2".to_string()))
        );
        assert_eq!(
            Move::from_uci("e9e4"),
            Err(MoveParseError::InvalidSourceSquare("e9".to_string()))
        );
    }

    #[test]
    fn rejects_invalid_target_square() {
        assert_eq!(
            Move::from_uci("e2i4"),
            Err(MoveParseError::InvalidTargetSquare("i4".to_string()))
        );
        assert_eq!(
            Move::from_uci("e2e9"),
            Err(MoveParseError::InvalidTargetSquare("e9".to_string()))
        );
    }

    #[test]
    fn rejects_invalid_promotion_piece() {
        assert_eq!(
            Move::from_uci("e2e4x"),
            Err(MoveParseError::InvalidPromotionPiece('x'))
        );
        assert_eq!(
            Move::from_uci("e7e8k"),
            Err(MoveParseError::InvalidPromotionPiece('k'))
        );
        assert_eq!(
            Move::from_uci("e7e8p"),
            Err(MoveParseError::InvalidPromotionPiece('p'))
        );
    }

    #[test]
    fn rejects_promotion_away_from_back_rank() {
        assert_eq!(
            Move::from_uci("e2e4q"),
            Err(MoveParseError::InvalidPromotionRank {
                target: square("e4")
            })
        );
    }

    #[test]
    fn rejects_too_short_strings() {
        assert_eq!(
            Move::from_uci(""),
            Err(MoveParseError::InvalidLength { found: 0 })
        );
        assert_eq!(
            Move::from_uci("e2"),
            Err(MoveParseError::InvalidLength { found: 2 })
        );
        assert_eq!(
            Move::from_uci("e2e"),
            Err(MoveParseError::InvalidLength { found: 3 })
        );
    }

    #[test]
    fn rejects_too_long_strings() {
        assert_eq!(
            Move::from_uci("e2e4qq"),
            Err(MoveParseError::InvalidLength { found: 6 })
        );
    }

    #[test]
    fn serializes_round_trip_moves() {
        let cases = ["e2e4", "g1f3", "b8c6", "e7e8q", "a2a1n"];

        for value in cases {
            assert_eq!(Move::from_uci(value).expect("valid move").to_uci(), value);
        }
    }
}
