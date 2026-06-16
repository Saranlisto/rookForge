//! Move representation and parsing scaffold.
//!
//! Legal and pseudo-legal generation are intentionally not implemented yet.

use std::fmt;
use std::str::FromStr;

use crate::board::{Color, PieceKind, Position, Square};

const PROMOTION_PIECES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];

/// Generates pseudo-legal pawn moves for the side to move.
///
/// This intentionally does not check king safety, en passant, or full move
/// legality. It only applies pawn movement structure against current occupancy.
#[must_use]
pub fn generate_pawn_moves(position: &Position) -> Vec<Move> {
    let side_to_move = position.side_to_move();
    let mut moves = Vec::new();

    for from in position.occupied_by_color(side_to_move) {
        if matches!(position.piece_at(from), Some(piece) if piece.kind == PieceKind::Pawn) {
            generate_pawn_moves_from(position, side_to_move, from, &mut moves);
        }
    }

    moves
}

fn generate_pawn_moves_from(
    position: &Position,
    color: Color,
    from: Square,
    moves: &mut Vec<Move>,
) {
    add_pawn_pushes(position, color, from, moves);
    add_pawn_captures(position, color, from, moves);
}

fn add_pawn_pushes(position: &Position, color: Color, from: Square, moves: &mut Vec<Move>) {
    let Some(one_step) = offset_square(from, 0, pawn_direction(color)) else {
        return;
    };

    if position.piece_at(one_step).is_some() {
        return;
    }

    add_pawn_move(from, one_step, moves);

    if from.rank() != pawn_starting_rank(color) {
        return;
    }

    let Some(two_steps) = offset_square(from, 0, pawn_direction(color) * 2) else {
        return;
    };

    if position.piece_at(two_steps).is_none() {
        moves.push(Move::quiet(from, two_steps));
    }
}

fn add_pawn_captures(position: &Position, color: Color, from: Square, moves: &mut Vec<Move>) {
    for file_delta in [-1, 1] {
        let Some(target) = offset_square(from, file_delta, pawn_direction(color)) else {
            continue;
        };

        if matches!(position.piece_at(target), Some(piece) if piece.color == color.opposite()) {
            add_pawn_move(from, target, moves);
        }
    }
}

fn add_pawn_move(from: Square, to: Square, moves: &mut Vec<Move>) {
    if is_promotion_rank(to) {
        for promotion in PROMOTION_PIECES {
            moves.push(
                Move::promotion(from, to, promotion)
                    .expect("promotion target is already on a promotion rank"),
            );
        }
    } else {
        moves.push(Move::quiet(from, to));
    }
}

const fn pawn_direction(color: Color) -> i8 {
    match color {
        Color::White => 1,
        Color::Black => -1,
    }
}

const fn pawn_starting_rank(color: Color) -> u8 {
    match color {
        Color::White => 1,
        Color::Black => 6,
    }
}

fn offset_square(square: Square, file_delta: i8, rank_delta: i8) -> Option<Square> {
    let file = square.file() as i8 + file_delta;
    let rank = square.rank() as i8 + rank_delta;

    if !(0..=7).contains(&file) || !(0..=7).contains(&rank) {
        return None;
    }

    Square::new(file as u8, rank as u8)
}

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

    fn pawn_moves_from_fen(fen: &str) -> Vec<String> {
        let position = Position::from_fen(fen).expect("valid test FEN");
        let mut moves = generate_pawn_moves(&position)
            .into_iter()
            .map(Move::to_uci)
            .collect::<Vec<_>>();
        moves.sort();
        moves
    }

    fn sorted_moves(values: &[&str]) -> Vec<String> {
        let mut moves = values
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        moves.sort();
        moves
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

    #[test]
    fn generates_white_pawn_single_push() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/4P3/8/8/8 w - - 0 1"),
            sorted_moves(&["e4e5"])
        );
    }

    #[test]
    fn generates_black_pawn_single_push() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/4p3/8/8/8/8 b - - 0 1"),
            sorted_moves(&["e5e4"])
        );
    }

    #[test]
    fn generates_white_pawn_double_push_from_starting_rank() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1"),
            sorted_moves(&["e2e3", "e2e4"])
        );
    }

    #[test]
    fn generates_black_pawn_double_push_from_starting_rank() {
        assert_eq!(
            pawn_moves_from_fen("8/4p3/8/8/8/8/8/8 b - - 0 1"),
            sorted_moves(&["e7e5", "e7e6"])
        );
    }

    #[test]
    fn double_push_is_blocked_by_piece_directly_ahead() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/8/4n3/4P3/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn double_push_is_blocked_by_piece_two_squares_ahead() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/4n3/8/4P3/8 w - - 0 1"),
            sorted_moves(&["e2e3"])
        );
    }

    #[test]
    fn generates_white_pawn_diagonal_captures() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/3npn2/4P3/8/8/8 w - - 0 1"),
            sorted_moves(&["e4d5", "e4f5"])
        );
    }

    #[test]
    fn generates_black_pawn_diagonal_captures() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/4p3/3PPP2/8/8/8 b - - 0 1"),
            sorted_moves(&["e5d4", "e5f4"])
        );
    }

    #[test]
    fn does_not_capture_empty_diagonal_squares() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/4p3/4P3/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn does_not_capture_own_pieces() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/3NpN2/4P3/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn generates_white_quiet_promotions() {
        assert_eq!(
            pawn_moves_from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1"),
            sorted_moves(&["e7e8q", "e7e8r", "e7e8b", "e7e8n"])
        );
    }

    #[test]
    fn generates_black_quiet_promotions() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/8/8/4p3/8 b - - 0 1"),
            sorted_moves(&["e2e1q", "e2e1r", "e2e1b", "e2e1n"])
        );
    }

    #[test]
    fn generates_white_promotion_captures() {
        assert_eq!(
            pawn_moves_from_fen("3nR3/4P3/8/8/8/8/8/8 w - - 0 1"),
            sorted_moves(&["e7d8q", "e7d8r", "e7d8b", "e7d8n"])
        );
    }

    #[test]
    fn generates_black_promotion_captures() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/8/8/4p3/3Nr3 b - - 0 1"),
            sorted_moves(&["e2d1q", "e2d1r", "e2d1b", "e2d1n"])
        );
    }

    #[test]
    fn edge_file_pawns_do_not_wrap_around_board() {
        assert_eq!(
            pawn_moves_from_fen("8/8/8/8/8/Nn4nN/P6P/8 w - - 0 1"),
            sorted_moves(&["a2b3", "h2g3"])
        );
    }

    #[test]
    fn starting_position_generates_sixteen_pawn_moves() {
        assert_eq!(
            pawn_moves_from_fen(crate::board::STARTING_POSITION_FEN),
            sorted_moves(&[
                "a2a3", "a2a4", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4", "e2e3", "e2e4",
                "f2f3", "f2f4", "g2g3", "g2g4", "h2h3", "h2h4",
            ])
        );
    }
}
