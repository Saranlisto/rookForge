//! Move representation, parsing, and early pseudo-legal generation.
//!
//! Full legal move generation is intentionally not implemented yet.

use std::fmt;
use std::str::FromStr;

use crate::board::{Color, PieceKind, Position, Square};

const PROMOTION_PIECES: [PieceKind; 4] = [
    PieceKind::Queen,
    PieceKind::Rook,
    PieceKind::Bishop,
    PieceKind::Knight,
];
const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const KING_DELTAS: [(i8, i8); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];
const BISHOP_DIRECTIONS: [(i8, i8); 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
const ROOK_DIRECTIONS: [(i8, i8); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];
const QUEEN_DIRECTIONS: [(i8, i8); 8] = [
    (0, 1),
    (0, -1),
    (1, 0),
    (-1, 0),
    (1, 1),
    (-1, 1),
    (1, -1),
    (-1, -1),
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

/// Generates pseudo-legal knight moves for the side to move.
///
/// This skips off-board moves and friendly-occupied targets, but it does not
/// check whether the king is in check.
#[must_use]
pub fn generate_knight_moves(position: &Position) -> Vec<Move> {
    generate_leaper_moves(position, PieceKind::Knight, &KNIGHT_DELTAS)
}

/// Generates pseudo-legal one-square king moves for the side to move.
///
/// Castling and attacked-square checks are intentionally not implemented here.
#[must_use]
pub fn generate_king_moves(position: &Position) -> Vec<Move> {
    generate_leaper_moves(position, PieceKind::King, &KING_DELTAS)
}

/// Generates the currently implemented non-sliding pseudo-legal moves.
#[must_use]
pub fn generate_non_sliding_moves(position: &Position) -> Vec<Move> {
    let mut moves = generate_pawn_moves(position);
    moves.extend(generate_knight_moves(position));
    moves.extend(generate_king_moves(position));
    moves
}

/// Generates pseudo-legal bishop moves for the side to move.
#[must_use]
pub fn generate_bishop_moves(position: &Position) -> Vec<Move> {
    generate_sliding_moves_for_kind(position, PieceKind::Bishop, &BISHOP_DIRECTIONS)
}

/// Generates pseudo-legal rook moves for the side to move.
#[must_use]
pub fn generate_rook_moves(position: &Position) -> Vec<Move> {
    generate_sliding_moves_for_kind(position, PieceKind::Rook, &ROOK_DIRECTIONS)
}

/// Generates pseudo-legal queen moves for the side to move.
#[must_use]
pub fn generate_queen_moves(position: &Position) -> Vec<Move> {
    generate_sliding_moves_for_kind(position, PieceKind::Queen, &QUEEN_DIRECTIONS)
}

/// Generates the currently implemented sliding pseudo-legal moves.
#[must_use]
pub fn generate_sliding_piece_moves(position: &Position) -> Vec<Move> {
    let mut moves = generate_bishop_moves(position);
    moves.extend(generate_rook_moves(position));
    moves.extend(generate_queen_moves(position));
    moves
}

/// Generates all currently implemented pseudo-legal moves for the side to move.
///
/// This intentionally excludes castling, en passant, check detection, and legal
/// move filtering. The output order is deterministic by piece family.
#[must_use]
pub fn generate_pseudo_legal_moves(position: &Position) -> Vec<Move> {
    let mut moves = generate_pawn_moves(position);
    moves.extend(generate_knight_moves(position));
    moves.extend(generate_bishop_moves(position));
    moves.extend(generate_rook_moves(position));
    moves.extend(generate_queen_moves(position));
    moves.extend(generate_king_moves(position));
    moves
}

fn generate_leaper_moves(position: &Position, kind: PieceKind, deltas: &[(i8, i8)]) -> Vec<Move> {
    let side_to_move = position.side_to_move();
    let mut moves = Vec::new();

    for from in position.occupied_by_color(side_to_move) {
        if matches!(position.piece_at(from), Some(piece) if piece.kind == kind) {
            add_leaper_moves_from(position, side_to_move, from, deltas, &mut moves);
        }
    }

    moves
}

fn add_leaper_moves_from(
    position: &Position,
    color: Color,
    from: Square,
    deltas: &[(i8, i8)],
    moves: &mut Vec<Move>,
) {
    for &(file_delta, rank_delta) in deltas {
        let Some(to) = offset_square(from, file_delta, rank_delta) else {
            continue;
        };

        if can_land_on(position, color, to) {
            moves.push(Move::quiet(from, to));
        }
    }
}

fn can_land_on(position: &Position, color: Color, target: Square) -> bool {
    !matches!(position.piece_at(target), Some(piece) if piece.color == color)
}

fn generate_sliding_moves_for_kind(
    position: &Position,
    kind: PieceKind,
    directions: &[(i8, i8)],
) -> Vec<Move> {
    let side_to_move = position.side_to_move();
    let mut moves = Vec::new();

    for from in position.occupied_by_color(side_to_move) {
        if matches!(position.piece_at(from), Some(piece) if piece.kind == kind) {
            add_sliding_moves_from(position, side_to_move, from, directions, &mut moves);
        }
    }

    moves
}

fn add_sliding_moves_from(
    position: &Position,
    color: Color,
    from: Square,
    directions: &[(i8, i8)],
    moves: &mut Vec<Move>,
) {
    for &(file_delta, rank_delta) in directions {
        let mut current = from;

        while let Some(to) = offset_square(current, file_delta, rank_delta) {
            match position.piece_at(to) {
                Some(piece) if piece.color == color => break,
                Some(_) => {
                    moves.push(Move::quiet(from, to));
                    break;
                }
                None => {
                    moves.push(Move::quiet(from, to));
                    current = to;
                }
            }
        }
    }
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

    fn moves_from_fen(fen: &str, generator: fn(&Position) -> Vec<Move>) -> Vec<String> {
        let position = Position::from_fen(fen).expect("valid test FEN");
        let mut moves = generator(&position)
            .into_iter()
            .map(Move::to_uci)
            .collect::<Vec<_>>();
        moves.sort();
        moves
    }

    fn pawn_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_pawn_moves)
    }

    fn knight_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_knight_moves)
    }

    fn king_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_king_moves)
    }

    fn bishop_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_bishop_moves)
    }

    fn rook_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_rook_moves)
    }

    fn queen_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_queen_moves)
    }

    fn pseudo_legal_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_pseudo_legal_moves)
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

    #[test]
    fn white_knight_in_center_has_eight_moves() {
        assert_eq!(
            knight_moves_from_fen("8/8/8/8/4N3/8/8/8 w - - 0 1"),
            sorted_moves(&["e4c3", "e4c5", "e4d2", "e4d6", "e4f2", "e4f6", "e4g3", "e4g5",])
        );
    }

    #[test]
    fn black_knight_in_center_has_eight_moves() {
        assert_eq!(
            knight_moves_from_fen("8/8/8/8/4n3/8/8/8 b - - 0 1"),
            sorted_moves(&["e4c3", "e4c5", "e4d2", "e4d6", "e4f2", "e4f6", "e4g3", "e4g5",])
        );
    }

    #[test]
    fn knight_on_a1_has_only_valid_board_moves() {
        assert_eq!(
            knight_moves_from_fen("8/8/8/8/8/8/8/N7 w - - 0 1"),
            sorted_moves(&["a1b3", "a1c2"])
        );
    }

    #[test]
    fn knight_on_h8_has_only_valid_board_moves() {
        assert_eq!(
            knight_moves_from_fen("7N/8/8/8/8/8/8/8 w - - 0 1"),
            sorted_moves(&["h8f7", "h8g6"])
        );
    }

    #[test]
    fn knight_cannot_capture_own_piece() {
        assert_eq!(
            knight_moves_from_fen("8/8/3P1P2/8/4N3/8/8/8 w - - 0 1"),
            sorted_moves(&["e4c3", "e4c5", "e4d2", "e4f2", "e4g3", "e4g5"])
        );
    }

    #[test]
    fn knight_can_capture_opponent_piece() {
        assert_eq!(
            knight_moves_from_fen("8/8/3p1p2/8/4N3/8/8/8 w - - 0 1"),
            sorted_moves(&["e4c3", "e4c5", "e4d2", "e4d6", "e4f2", "e4f6", "e4g3", "e4g5",])
        );
    }

    #[test]
    fn starting_position_generates_four_white_knight_moves() {
        assert_eq!(
            knight_moves_from_fen(crate::board::STARTING_POSITION_FEN),
            sorted_moves(&["b1a3", "b1c3", "g1f3", "g1h3"])
        );
    }

    #[test]
    fn starting_position_generates_four_black_knight_moves() {
        assert_eq!(
            knight_moves_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1"),
            sorted_moves(&["b8a6", "b8c6", "g8f6", "g8h6"])
        );
    }

    #[test]
    fn king_in_center_has_eight_moves() {
        assert_eq!(
            king_moves_from_fen("8/8/8/8/4K3/8/8/8 w - - 0 1"),
            sorted_moves(&["e4d3", "e4d4", "e4d5", "e4e3", "e4e5", "e4f3", "e4f4", "e4f5",])
        );
    }

    #[test]
    fn king_on_a1_has_only_valid_board_moves() {
        assert_eq!(
            king_moves_from_fen("8/8/8/8/8/8/8/K7 w - - 0 1"),
            sorted_moves(&["a1a2", "a1b1", "a1b2"])
        );
    }

    #[test]
    fn king_on_h8_has_only_valid_board_moves() {
        assert_eq!(
            king_moves_from_fen("7K/8/8/8/8/8/8/8 w - - 0 1"),
            sorted_moves(&["h8g7", "h8g8", "h8h7"])
        );
    }

    #[test]
    fn king_cannot_move_onto_own_piece() {
        assert_eq!(
            king_moves_from_fen("8/8/8/3PPP2/3PKP2/3PPP2/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn king_can_capture_opponent_piece() {
        assert_eq!(
            king_moves_from_fen("8/8/8/3pPP2/3PKP2/3PPP2/8/8 w - - 0 1"),
            sorted_moves(&["e4d5"])
        );
    }

    #[test]
    fn king_moves_are_generated_only_for_side_to_move() {
        assert_eq!(
            king_moves_from_fen("k7/8/8/8/4K3/8/8/8 b - - 0 1"),
            sorted_moves(&["a8a7", "a8b7", "a8b8"])
        );
    }

    #[test]
    fn king_move_generation_does_not_generate_castling() {
        assert_eq!(
            king_moves_from_fen("8/8/8/8/8/8/8/R3K2R w KQ - 0 1"),
            sorted_moves(&["e1d1", "e1d2", "e1e2", "e1f1", "e1f2"])
        );
    }

    #[test]
    fn bishop_in_center_has_thirteen_moves() {
        assert_eq!(
            bishop_moves_from_fen("8/8/8/3B4/8/8/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a2", "d5a8", "d5b3", "d5b7", "d5c4", "d5c6", "d5e4", "d5e6", "d5f3", "d5f7",
                "d5g2", "d5g8", "d5h1",
            ])
        );
    }

    #[test]
    fn bishop_on_a1_has_seven_moves() {
        assert_eq!(
            bishop_moves_from_fen("8/8/8/8/8/8/8/B7 w - - 0 1"),
            sorted_moves(&["a1b2", "a1c3", "a1d4", "a1e5", "a1f6", "a1g7", "a1h8"])
        );
    }

    #[test]
    fn bishop_on_h8_has_seven_moves() {
        assert_eq!(
            bishop_moves_from_fen("7B/8/8/8/8/8/8/8 w - - 0 1"),
            sorted_moves(&["h8a1", "h8b2", "h8c3", "h8d4", "h8e5", "h8f6", "h8g7"])
        );
    }

    #[test]
    fn bishop_cannot_move_through_own_piece() {
        assert_eq!(
            bishop_moves_from_fen("8/5P2/8/3B4/8/1P6/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a8", "d5b7", "d5c4", "d5c6", "d5e4", "d5e6", "d5f3", "d5g2", "d5h1",
            ])
        );
    }

    #[test]
    fn bishop_cannot_capture_own_piece() {
        assert_eq!(
            bishop_moves_from_fen("8/8/2P1P3/3B4/2P1P3/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn bishop_can_capture_opponent_piece_and_stops() {
        assert_eq!(
            bishop_moves_from_fen("8/5p2/8/3B4/8/1p6/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a8", "d5b3", "d5b7", "d5c4", "d5c6", "d5e4", "d5e6", "d5f3", "d5f7", "d5g2",
                "d5h1",
            ])
        );
    }

    #[test]
    fn bishop_does_not_wrap_around_edges() {
        assert_eq!(
            bishop_moves_from_fen("8/8/8/8/8/8/8/7B w - - 0 1"),
            sorted_moves(&["h1a8", "h1b7", "h1c6", "h1d5", "h1e4", "h1f3", "h1g2"])
        );
    }

    #[test]
    fn starting_position_bishops_are_blocked() {
        assert_eq!(
            bishop_moves_from_fen(crate::board::STARTING_POSITION_FEN),
            Vec::<String>::new()
        );
    }

    #[test]
    fn rook_in_center_has_fourteen_moves() {
        assert_eq!(
            rook_moves_from_fen("8/8/8/3R4/8/8/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a5", "d5b5", "d5c5", "d5d1", "d5d2", "d5d3", "d5d4", "d5d6", "d5d7", "d5d8",
                "d5e5", "d5f5", "d5g5", "d5h5",
            ])
        );
    }

    #[test]
    fn rook_on_a1_has_fourteen_moves() {
        assert_eq!(
            rook_moves_from_fen("8/8/8/8/8/8/8/R7 w - - 0 1"),
            sorted_moves(&[
                "a1a2", "a1a3", "a1a4", "a1a5", "a1a6", "a1a7", "a1a8", "a1b1", "a1c1", "a1d1",
                "a1e1", "a1f1", "a1g1", "a1h1",
            ])
        );
    }

    #[test]
    fn rook_cannot_move_through_own_piece() {
        assert_eq!(
            rook_moves_from_fen("8/3P4/8/3R1P2/8/8/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a5", "d5b5", "d5c5", "d5d1", "d5d2", "d5d3", "d5d4", "d5d6", "d5e5",
            ])
        );
    }

    #[test]
    fn rook_cannot_capture_own_piece() {
        assert_eq!(
            rook_moves_from_fen("8/8/3P4/2PRP3/3P4/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn rook_can_capture_opponent_piece_and_stops() {
        assert_eq!(
            rook_moves_from_fen("8/3p4/8/3R1p2/8/8/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a5", "d5b5", "d5c5", "d5d1", "d5d2", "d5d3", "d5d4", "d5d6", "d5d7", "d5e5",
                "d5f5",
            ])
        );
    }

    #[test]
    fn rook_does_not_wrap_around_edges() {
        assert_eq!(
            rook_moves_from_fen("8/8/8/8/7R/8/8/8 w - - 0 1"),
            sorted_moves(&[
                "h4a4", "h4b4", "h4c4", "h4d4", "h4e4", "h4f4", "h4g4", "h4h1", "h4h2", "h4h3",
                "h4h5", "h4h6", "h4h7", "h4h8",
            ])
        );
    }

    #[test]
    fn starting_position_rooks_are_blocked() {
        assert_eq!(
            rook_moves_from_fen(crate::board::STARTING_POSITION_FEN),
            Vec::<String>::new()
        );
    }

    #[test]
    fn queen_in_center_has_twenty_seven_moves() {
        assert_eq!(
            queen_moves_from_fen("8/8/8/3Q4/8/8/8/8 w - - 0 1"),
            sorted_moves(&[
                "d5a2", "d5a5", "d5a8", "d5b3", "d5b5", "d5b7", "d5c4", "d5c5", "d5c6", "d5d1",
                "d5d2", "d5d3", "d5d4", "d5d6", "d5d7", "d5d8", "d5e4", "d5e5", "d5e6", "d5f3",
                "d5f5", "d5f7", "d5g2", "d5g5", "d5g8", "d5h1", "d5h5",
            ])
        );
    }

    #[test]
    fn queen_on_a1_has_twenty_one_moves() {
        assert_eq!(
            queen_moves_from_fen("8/8/8/8/8/8/8/Q7 w - - 0 1"),
            sorted_moves(&[
                "a1a2", "a1a3", "a1a4", "a1a5", "a1a6", "a1a7", "a1a8", "a1b1", "a1b2", "a1c1",
                "a1c3", "a1d1", "a1d4", "a1e1", "a1e5", "a1f1", "a1f6", "a1g1", "a1g7", "a1h1",
                "a1h8",
            ])
        );
    }

    #[test]
    fn queen_cannot_move_through_own_piece() {
        assert_eq!(
            queen_moves_from_fen("8/8/2PPP3/2PQP3/2PPP3/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn queen_cannot_capture_own_piece() {
        assert_eq!(
            queen_moves_from_fen("8/8/2PPP3/2PQP3/2PPP3/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn queen_can_capture_opponent_pieces_and_stops() {
        assert_eq!(
            queen_moves_from_fen("8/8/2ppp3/2pQp3/2ppp3/8/8/8 w - - 0 1"),
            sorted_moves(&["d5c4", "d5c5", "d5c6", "d5d4", "d5d6", "d5e4", "d5e5", "d5e6"])
        );
    }

    #[test]
    fn queen_combines_rook_like_and_bishop_like_movement() {
        let moves = queen_moves_from_fen("8/8/8/3Q4/8/8/8/8 w - - 0 1");

        assert!(moves.contains(&"d5d8".to_string()));
        assert!(moves.contains(&"d5a5".to_string()));
        assert!(moves.contains(&"d5a8".to_string()));
        assert!(moves.contains(&"d5h1".to_string()));
    }

    #[test]
    fn starting_position_queen_is_blocked() {
        assert_eq!(
            queen_moves_from_fen(crate::board::STARTING_POSITION_FEN),
            Vec::<String>::new()
        );
    }

    #[test]
    fn starting_position_generates_twenty_pseudo_legal_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen(crate::board::STARTING_POSITION_FEN),
            sorted_moves(&[
                "a2a3", "a2a4", "b1a3", "b1c3", "b2b3", "b2b4", "c2c3", "c2c4", "d2d3", "d2d4",
                "e2e3", "e2e4", "f2f3", "f2f4", "g1f3", "g1h3", "g2g3", "g2g4", "h2h3", "h2h4",
            ])
        );
    }

    #[test]
    fn black_starting_position_generates_twenty_pseudo_legal_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1",),
            sorted_moves(&[
                "a7a5", "a7a6", "b7b5", "b7b6", "b8a6", "b8c6", "c7c5", "c7c6", "d7d5", "d7d6",
                "e7e5", "e7e6", "f7f5", "f7f6", "g7g5", "g7g6", "g8f6", "g8h6", "h7h5", "h7h6",
            ])
        );
    }

    #[test]
    fn empty_board_generates_no_pseudo_legal_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/8/8/8/8/8 w - - 0 1"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_queen_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/3Q4/8/8/8/8 w - - 0 1"),
            queen_moves_from_fen("8/8/8/3Q4/8/8/8/8 w - - 0 1")
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_rook_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/3R4/8/8/8/8 w - - 0 1"),
            rook_moves_from_fen("8/8/8/3R4/8/8/8/8 w - - 0 1")
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_bishop_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/3B4/8/8/8/8 w - - 0 1"),
            bishop_moves_from_fen("8/8/8/3B4/8/8/8/8 w - - 0 1")
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_king_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/8/4K3/8/8/8 w - - 0 1"),
            king_moves_from_fen("8/8/8/8/4K3/8/8/8 w - - 0 1")
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_knight_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/8/4N3/8/8/8 w - - 0 1"),
            knight_moves_from_fen("8/8/8/8/4N3/8/8/8 w - - 0 1")
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_pawn_moves() {
        assert_eq!(
            pseudo_legal_moves_from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1"),
            pawn_moves_from_fen("8/8/8/8/8/8/4P3/8 w - - 0 1")
        );
    }

    #[test]
    fn pseudo_legal_generator_includes_multiple_piece_types() {
        let fen = "8/8/8/3Q4/4N3/8/4P3/4K3 w - - 0 1";
        let mut expected = pawn_moves_from_fen(fen);
        expected.extend(knight_moves_from_fen(fen));
        expected.extend(bishop_moves_from_fen(fen));
        expected.extend(rook_moves_from_fen(fen));
        expected.extend(queen_moves_from_fen(fen));
        expected.extend(king_moves_from_fen(fen));
        expected.sort();

        let moves = pseudo_legal_moves_from_fen(fen);

        assert!(moves.contains(&"e2e3".to_string()));
        assert!(moves.contains(&"e4c3".to_string()));
        assert!(moves.contains(&"d5d8".to_string()));
        assert!(moves.contains(&"e1d1".to_string()));
        assert_eq!(moves, expected);
    }
}
