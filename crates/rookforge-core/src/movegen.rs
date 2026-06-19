//! Move representation, parsing, pseudo-legal generation, and legal filtering.

use std::fmt;
use std::str::FromStr;

use crate::board::{CastlingRights, Color, Piece, PieceKind, Position, Square};

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

/// Finds the king square for `color`, if the position contains one.
#[must_use]
pub fn find_king(position: &Position, color: Color) -> Option<Square> {
    position.occupied_by_color(color).into_iter().find(
        |&square| matches!(position.piece_at(square), Some(piece) if piece.kind == PieceKind::King),
    )
}

/// Returns true if `color`'s king is currently attacked by the opponent.
///
/// Positions without a king for `color` are treated as not in check so that
/// structurally parsed test positions can still be inspected.
#[must_use]
pub fn is_in_check(position: &Position, color: Color) -> bool {
    find_king(position, color)
        .is_some_and(|king_square| is_square_attacked(position, king_square, color.opposite()))
}

/// Generates currently supported legal moves for the side to move.
///
/// This filters pseudo-legal moves by applying each move and rejecting any move
/// that leaves the moving side's king in check. Castling and en passant are not
/// generated yet.
#[must_use]
pub fn generate_legal_moves(position: &Position) -> Vec<Move> {
    let moving_side = position.side_to_move();

    generate_pseudo_legal_moves(position)
        .into_iter()
        .filter(|&mv| !targets_opponent_king(position, mv))
        .filter_map(|mv| {
            apply_move(position, mv)
                .ok()
                .filter(|candidate| !is_in_check(candidate, moving_side))
                .map(|_| mv)
        })
        .collect()
}

/// Counts legal move-tree leaf nodes to `depth`.
///
/// Depth 0 counts the current node as one. Deeper depths recursively apply
/// legal moves and sum each child node count. Castling and en passant are not
/// available until those rules are implemented in legal move generation.
#[must_use]
pub fn perft(position: &Position, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }

    let moves = generate_legal_moves(position);

    if depth == 1 {
        return moves.len() as u64;
    }

    moves
        .into_iter()
        .filter_map(|mv| apply_move(position, mv).ok())
        .map(|candidate| perft(&candidate, depth - 1))
        .sum()
}

/// Returns true when `square` is attacked by any piece of `by_color`.
///
/// This ignores side to move, does not mutate the position, and does not check
/// whether either side's king is legally safe.
#[must_use]
pub fn is_square_attacked(position: &Position, square: Square, by_color: Color) -> bool {
    is_attacked_by_pawn(position, square, by_color)
        || is_attacked_by_leaper(
            position,
            square,
            by_color,
            PieceKind::Knight,
            &KNIGHT_DELTAS,
        )
        || is_attacked_by_leaper(position, square, by_color, PieceKind::King, &KING_DELTAS)
        || is_attacked_by_slider(
            position,
            square,
            by_color,
            &BISHOP_DIRECTIONS,
            PieceKind::Bishop,
        )
        || is_attacked_by_slider(
            position,
            square,
            by_color,
            &ROOK_DIRECTIONS,
            PieceKind::Rook,
        )
}

/// Applies a structurally valid move to a position and returns the resulting position.
///
/// This does not check whether the move is pseudo-legal or legal. It only moves
/// pieces, handles captures/promotions, updates turn metadata, and clears or
/// sets simple state that is affected by the move.
pub fn apply_move(position: &Position, mv: Move) -> Result<Position, MoveApplyError> {
    let moving_piece = position
        .piece_at(mv.from)
        .ok_or(MoveApplyError::EmptySourceSquare { source: mv.from })?;
    let captured_piece = position.piece_at(mv.to);
    let placed_piece = piece_after_promotion(moving_piece, mv.promotion)?;
    let moving_side = position.side_to_move();

    let mut next = position.clone();
    let mut castling_rights = next.castling_rights();

    update_castling_rights_for_move(&mut castling_rights, moving_piece, mv.from);
    if let Some(piece) = captured_piece {
        update_castling_rights_for_capture(&mut castling_rights, piece, mv.to);
    }

    next.clear_square(mv.from);
    next.set_piece(mv.to, placed_piece);
    next.set_castling_rights(castling_rights);
    next.set_en_passant_target(en_passant_target_after_move(moving_piece, mv));

    if moving_piece.kind == PieceKind::Pawn || captured_piece.is_some() {
        next.set_halfmove_clock(0);
    } else {
        next.set_halfmove_clock(position.halfmove_clock().saturating_add(1));
    }

    if moving_side == Color::Black {
        next.set_fullmove_number(position.fullmove_number().saturating_add(1));
    }

    next.set_side_to_move(moving_side.opposite());

    Ok(next)
}

fn piece_after_promotion(
    moving_piece: Piece,
    promotion: Option<PieceKind>,
) -> Result<Piece, MoveApplyError> {
    let Some(promotion) = promotion else {
        return Ok(moving_piece);
    };

    if !is_valid_promotion_piece(promotion) {
        return Err(MoveApplyError::InvalidPromotionPiece(promotion));
    }

    if moving_piece.kind != PieceKind::Pawn {
        return Err(MoveApplyError::PromotionWithoutPawn {
            source_kind: moving_piece.kind,
        });
    }

    Ok(Piece::new(moving_piece.color, promotion))
}

fn en_passant_target_after_move(moving_piece: Piece, mv: Move) -> Option<Square> {
    if moving_piece.kind != PieceKind::Pawn || mv.from.file() != mv.to.file() {
        return None;
    }

    let from_rank = mv.from.rank();
    let to_rank = mv.to.rank();

    if from_rank.abs_diff(to_rank) == 2 {
        let passed_rank = (from_rank + to_rank) / 2;
        Square::new(mv.from.file(), passed_rank)
    } else {
        None
    }
}

fn update_castling_rights_for_move(rights: &mut CastlingRights, moving_piece: Piece, from: Square) {
    match (moving_piece.color, moving_piece.kind) {
        (Color::White, PieceKind::King) => {
            rights.white_kingside = false;
            rights.white_queenside = false;
        }
        (Color::Black, PieceKind::King) => {
            rights.black_kingside = false;
            rights.black_queenside = false;
        }
        (Color::White, PieceKind::Rook) if from == square("h1") => {
            rights.white_kingside = false;
        }
        (Color::White, PieceKind::Rook) if from == square("a1") => {
            rights.white_queenside = false;
        }
        (Color::Black, PieceKind::Rook) if from == square("h8") => {
            rights.black_kingside = false;
        }
        (Color::Black, PieceKind::Rook) if from == square("a8") => {
            rights.black_queenside = false;
        }
        _ => {}
    }
}

fn update_castling_rights_for_capture(
    rights: &mut CastlingRights,
    captured_piece: Piece,
    target: Square,
) {
    match (captured_piece.color, captured_piece.kind) {
        (Color::White, PieceKind::Rook) if target == square("h1") => {
            rights.white_kingside = false;
        }
        (Color::White, PieceKind::Rook) if target == square("a1") => {
            rights.white_queenside = false;
        }
        (Color::Black, PieceKind::Rook) if target == square("h8") => {
            rights.black_kingside = false;
        }
        (Color::Black, PieceKind::Rook) if target == square("a8") => {
            rights.black_queenside = false;
        }
        _ => {}
    }
}

fn square(value: &str) -> Square {
    Square::from_algebraic(value).expect("hard-coded square is valid")
}

fn is_attacked_by_pawn(position: &Position, square: Square, by_color: Color) -> bool {
    for file_delta in [-1, 1] {
        let Some(source) =
            offset_square(square, file_delta, pawn_attack_source_rank_delta(by_color))
        else {
            continue;
        };

        if position.piece_at(source) == Some(Piece::new(by_color, PieceKind::Pawn)) {
            return true;
        }
    }

    false
}

fn pawn_attack_source_rank_delta(color: Color) -> i8 {
    match color {
        Color::White => -1,
        Color::Black => 1,
    }
}

fn is_attacked_by_leaper(
    position: &Position,
    square: Square,
    by_color: Color,
    kind: PieceKind,
    deltas: &[(i8, i8)],
) -> bool {
    deltas.iter().any(|&(file_delta, rank_delta)| {
        offset_square(square, file_delta, rank_delta).and_then(|source| position.piece_at(source))
            == Some(Piece::new(by_color, kind))
    })
}

fn is_attacked_by_slider(
    position: &Position,
    square: Square,
    by_color: Color,
    directions: &[(i8, i8)],
    primary_kind: PieceKind,
) -> bool {
    for &(file_delta, rank_delta) in directions {
        let mut current = square;

        while let Some(source) = offset_square(current, file_delta, rank_delta) {
            let Some(piece) = position.piece_at(source) else {
                current = source;
                continue;
            };

            if piece.color == by_color
                && (piece.kind == primary_kind || piece.kind == PieceKind::Queen)
            {
                return true;
            }

            break;
        }
    }

    false
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

fn targets_opponent_king(position: &Position, mv: Move) -> bool {
    matches!(position.piece_at(mv.to), Some(piece) if piece.kind == PieceKind::King)
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

/// Errors produced when applying a structurally invalid move to a position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveApplyError {
    EmptySourceSquare { source: Square },
    InvalidPromotionPiece(PieceKind),
    PromotionWithoutPawn { source_kind: PieceKind },
}

impl fmt::Display for MoveApplyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySourceSquare { source } => {
                write!(
                    formatter,
                    "source square `{}` is empty",
                    source.to_algebraic()
                )
            }
            Self::InvalidPromotionPiece(kind) => {
                write!(formatter, "invalid promotion piece `{kind:?}`")
            }
            Self::PromotionWithoutPawn { source_kind } => {
                write!(
                    formatter,
                    "promotion requested for non-pawn source piece `{source_kind:?}`"
                )
            }
        }
    }
}

impl std::error::Error for MoveApplyError {}

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

    fn legal_moves_from_fen(fen: &str) -> Vec<String> {
        moves_from_fen(fen, generate_legal_moves)
    }

    fn perft_from_fen(fen: &str, depth: u32) -> u64 {
        let position = Position::from_fen(fen).expect("valid test FEN");

        perft(&position, depth)
    }

    fn apply_move_from_uci(fen: &str, value: &str) -> Result<Position, MoveApplyError> {
        let position = Position::from_fen(fen).expect("valid test FEN");
        let mv = Move::from_uci(value).expect("valid test move");

        apply_move(&position, mv)
    }

    fn square_attacked(fen: &str, target: &str, by_color: Color) -> bool {
        let position = Position::from_fen(fen).expect("valid test FEN");
        let target = square(target);

        is_square_attacked(&position, target, by_color)
    }

    fn in_check(fen: &str, color: Color) -> bool {
        let position = Position::from_fen(fen).expect("valid test FEN");

        is_in_check(&position, color)
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

    #[test]
    fn applying_e2e4_from_starting_position_moves_pawn() {
        let result =
            apply_move_from_uci(crate::board::STARTING_POSITION_FEN, "e2e4").expect("move applies");

        assert_eq!(result.piece_at(square("e2")), None);
        assert_eq!(
            result.piece_at(square("e4")),
            Some(Piece::new(Color::White, PieceKind::Pawn))
        );
        assert_eq!(
            result.to_fen(),
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"
        );
    }

    #[test]
    fn applying_e2e4_flips_side_to_move_to_black() {
        let result =
            apply_move_from_uci(crate::board::STARTING_POSITION_FEN, "e2e4").expect("move applies");

        assert_eq!(result.side_to_move(), Color::Black);
    }

    #[test]
    fn white_move_does_not_increment_fullmove_number() {
        let result =
            apply_move_from_uci(crate::board::STARTING_POSITION_FEN, "e2e4").expect("move applies");

        assert_eq!(result.fullmove_number(), 1);
    }

    #[test]
    fn black_move_increments_fullmove_number() {
        let result = apply_move_from_uci(
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
            "e7e5",
        )
        .expect("move applies");

        assert_eq!(result.side_to_move(), Color::White);
        assert_eq!(result.fullmove_number(), 2);
        assert_eq!(result.en_passant_target(), Some(square("e6")));
    }

    #[test]
    fn quiet_non_pawn_move_increments_halfmove_clock() {
        let result =
            apply_move_from_uci("8/8/8/8/8/8/8/N7 w - - 7 3", "a1b3").expect("move applies");

        assert_eq!(result.halfmove_clock(), 8);
    }

    #[test]
    fn pawn_move_resets_halfmove_clock() {
        let result =
            apply_move_from_uci("8/8/8/8/8/8/4P3/8 w - - 12 4", "e2e4").expect("move applies");

        assert_eq!(result.halfmove_clock(), 0);
    }

    #[test]
    fn capture_resets_halfmove_clock() {
        let result =
            apply_move_from_uci("8/5p2/8/8/2B5/8/8/8 w - - 9 4", "c4f7").expect("move applies");

        assert_eq!(result.halfmove_clock(), 0);
    }

    #[test]
    fn capture_removes_opponent_piece() {
        let result =
            apply_move_from_uci("8/5p2/8/8/2B5/8/8/8 w - - 9 4", "c4f7").expect("move applies");

        assert_eq!(result.piece_at(square("c4")), None);
        assert_eq!(
            result.piece_at(square("f7")),
            Some(Piece::new(Color::White, PieceKind::Bishop))
        );
        assert_eq!(result.count_pieces(), 1);
    }

    #[test]
    fn promotion_to_queen_works() {
        let result =
            apply_move_from_uci("8/4P3/8/8/8/8/8/8 w - - 0 1", "e7e8q").expect("move applies");

        assert_eq!(
            result.piece_at(square("e8")),
            Some(Piece::new(Color::White, PieceKind::Queen))
        );
        assert_eq!(result.to_fen(), "4Q3/8/8/8/8/8/8/8 b - - 0 1");
    }

    #[test]
    fn promotion_to_rook_works() {
        let result =
            apply_move_from_uci("8/4P3/8/8/8/8/8/8 w - - 0 1", "e7e8r").expect("move applies");

        assert_eq!(
            result.piece_at(square("e8")),
            Some(Piece::new(Color::White, PieceKind::Rook))
        );
    }

    #[test]
    fn promotion_to_bishop_works() {
        let result =
            apply_move_from_uci("8/4P3/8/8/8/8/8/8 w - - 0 1", "e7e8b").expect("move applies");

        assert_eq!(
            result.piece_at(square("e8")),
            Some(Piece::new(Color::White, PieceKind::Bishop))
        );
    }

    #[test]
    fn promotion_to_knight_works() {
        let result =
            apply_move_from_uci("8/4P3/8/8/8/8/8/8 w - - 0 1", "e7e8n").expect("move applies");

        assert_eq!(
            result.piece_at(square("e8")),
            Some(Piece::new(Color::White, PieceKind::Knight))
        );
    }

    #[test]
    fn promotion_with_non_pawn_source_returns_error() {
        assert_eq!(
            apply_move_from_uci("8/4N3/8/8/8/8/8/8 w - - 0 1", "e7e8q"),
            Err(MoveApplyError::PromotionWithoutPawn {
                source_kind: PieceKind::Knight,
            })
        );
    }

    #[test]
    fn invalid_promotion_piece_returns_error() {
        let position = Position::from_fen("8/4P3/8/8/8/8/8/8 w - - 0 1").expect("valid FEN");
        let mv = Move {
            from: square("e7"),
            to: square("e8"),
            promotion: Some(PieceKind::Pawn),
        };

        assert_eq!(
            apply_move(&position, mv),
            Err(MoveApplyError::InvalidPromotionPiece(PieceKind::Pawn))
        );
    }

    #[test]
    fn empty_source_square_returns_error() {
        assert_eq!(
            apply_move_from_uci(crate::board::STARTING_POSITION_FEN, "e3e4"),
            Err(MoveApplyError::EmptySourceSquare {
                source: square("e3"),
            })
        );
    }

    #[test]
    fn white_king_move_removes_both_white_castling_rights() {
        let result = apply_move_from_uci("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "e1e2")
            .expect("move applies");

        assert_eq!(
            result.castling_rights(),
            CastlingRights {
                white_kingside: false,
                white_queenside: false,
                black_kingside: true,
                black_queenside: true,
            }
        );
    }

    #[test]
    fn black_king_move_removes_both_black_castling_rights() {
        let result = apply_move_from_uci("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", "e8e7")
            .expect("move applies");

        assert_eq!(
            result.castling_rights(),
            CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: false,
                black_queenside: false,
            }
        );
    }

    #[test]
    fn white_rook_move_from_h1_removes_white_kingside_castling_right() {
        let result = apply_move_from_uci("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "h1h3")
            .expect("move applies");

        assert!(!result.castling_rights().white_kingside);
        assert!(result.castling_rights().white_queenside);
    }

    #[test]
    fn white_rook_move_from_a1_removes_white_queenside_castling_right() {
        let result = apply_move_from_uci("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "a1a3")
            .expect("move applies");

        assert!(result.castling_rights().white_kingside);
        assert!(!result.castling_rights().white_queenside);
    }

    #[test]
    fn black_rook_move_from_h8_removes_black_kingside_castling_right() {
        let result = apply_move_from_uci("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", "h8h6")
            .expect("move applies");

        assert!(!result.castling_rights().black_kingside);
        assert!(result.castling_rights().black_queenside);
    }

    #[test]
    fn black_rook_move_from_a8_removes_black_queenside_castling_right() {
        let result = apply_move_from_uci("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", "a8a6")
            .expect("move applies");

        assert!(result.castling_rights().black_kingside);
        assert!(!result.castling_rights().black_queenside);
    }

    #[test]
    fn capturing_rook_on_original_square_removes_related_castling_right() {
        let cases = [
            (
                "r3k2r/8/8/8/8/8/7q/R3K2R b KQkq - 0 1",
                "h2h1",
                CastlingRights {
                    white_kingside: false,
                    white_queenside: true,
                    black_kingside: true,
                    black_queenside: true,
                },
            ),
            (
                "r3k2r/8/8/8/8/8/q7/R3K2R b KQkq - 0 1",
                "a2a1",
                CastlingRights {
                    white_kingside: true,
                    white_queenside: false,
                    black_kingside: true,
                    black_queenside: true,
                },
            ),
            (
                "r3k2r/7Q/8/8/8/8/8/R3K2R w KQkq - 0 1",
                "h7h8",
                CastlingRights {
                    white_kingside: true,
                    white_queenside: true,
                    black_kingside: false,
                    black_queenside: true,
                },
            ),
            (
                "r3k2r/Q7/8/8/8/8/8/R3K2R w KQkq - 0 1",
                "a7a8",
                CastlingRights {
                    white_kingside: true,
                    white_queenside: true,
                    black_kingside: true,
                    black_queenside: false,
                },
            ),
        ];

        for (fen, mv, expected_rights) in cases {
            let result = apply_move_from_uci(fen, mv).expect("move applies");

            assert_eq!(result.castling_rights(), expected_rights);
        }
    }

    #[test]
    fn white_pawn_attacks_diagonally_upward() {
        assert!(square_attacked(
            "8/8/4P3/8/8/8/8/8 b - - 0 1",
            "d7",
            Color::White
        ));
        assert!(square_attacked(
            "8/8/4P3/8/8/8/8/8 b - - 0 1",
            "f7",
            Color::White
        ));
        assert!(!square_attacked(
            "8/8/4P3/8/8/8/8/8 b - - 0 1",
            "e7",
            Color::White
        ));
    }

    #[test]
    fn black_pawn_attacks_diagonally_downward() {
        assert!(square_attacked(
            "8/8/8/4p3/8/8/8/8 w - - 0 1",
            "d4",
            Color::Black
        ));
        assert!(square_attacked(
            "8/8/8/4p3/8/8/8/8 w - - 0 1",
            "f4",
            Color::Black
        ));
        assert!(!square_attacked(
            "8/8/8/4p3/8/8/8/8 w - - 0 1",
            "e4",
            Color::Black
        ));
    }

    #[test]
    fn pawn_edge_file_attacks_stay_on_board() {
        assert!(square_attacked(
            "8/8/8/8/8/8/P6p/8 w - - 0 1",
            "b3",
            Color::White
        ));
        assert!(square_attacked(
            "8/8/8/8/8/8/P6p/8 w - - 0 1",
            "g1",
            Color::Black
        ));
        assert!(!square_attacked(
            "8/8/8/8/8/8/P6p/8 w - - 0 1",
            "a3",
            Color::White
        ));
    }

    #[test]
    fn knight_attacks_from_center() {
        let fen = "8/8/8/8/4N3/8/8/8 b - - 0 1";

        for target in ["c3", "c5", "d2", "d6", "f2", "f6", "g3", "g5"] {
            assert!(square_attacked(fen, target, Color::White));
        }

        assert!(!square_attacked(fen, "e6", Color::White));
    }

    #[test]
    fn knight_attack_detection_handles_edges() {
        let fen = "8/8/8/8/8/8/8/N7 b - - 0 1";

        assert!(square_attacked(fen, "b3", Color::White));
        assert!(square_attacked(fen, "c2", Color::White));
        assert!(!square_attacked(fen, "a2", Color::White));
    }

    #[test]
    fn king_attacks_adjacent_squares() {
        let fen = "8/8/8/8/4K3/8/8/8 b - - 0 1";

        assert!(square_attacked(fen, "d5", Color::White));
        assert!(square_attacked(fen, "e5", Color::White));
        assert!(square_attacked(fen, "f3", Color::White));
        assert!(!square_attacked(fen, "e6", Color::White));
    }

    #[test]
    fn bishop_attacks_diagonally() {
        assert!(square_attacked(
            "8/8/7B/8/8/8/8/8 b - - 0 1",
            "e3",
            Color::White
        ));
    }

    #[test]
    fn bishop_attack_is_blocked_by_intervening_piece() {
        assert!(!square_attacked(
            "8/8/8/8/8/8/3P4/2B5 b - - 0 1",
            "h6",
            Color::White
        ));
    }

    #[test]
    fn rook_attacks_file_and_rank() {
        let fen = "8/8/8/8/3R4/8/8/8 b - - 0 1";

        assert!(square_attacked(fen, "d8", Color::White));
        assert!(square_attacked(fen, "a4", Color::White));
        assert!(!square_attacked(fen, "a8", Color::White));
    }

    #[test]
    fn rook_attack_is_blocked_by_intervening_piece() {
        assert!(!square_attacked(
            "8/8/3P4/8/3R4/8/8/8 b - - 0 1",
            "d8",
            Color::White
        ));
    }

    #[test]
    fn queen_attacks_diagonally() {
        assert!(square_attacked(
            "8/8/8/8/3Q4/8/8/8 b - - 0 1",
            "g7",
            Color::White
        ));
    }

    #[test]
    fn queen_attacks_orthogonally() {
        assert!(square_attacked(
            "8/8/8/8/3Q4/8/8/8 b - - 0 1",
            "d8",
            Color::White
        ));
    }

    #[test]
    fn queen_attack_is_blocked_by_intervening_piece() {
        assert!(!square_attacked(
            "8/8/8/4P3/3Q4/8/8/8 b - - 0 1",
            "g7",
            Color::White
        ));
    }

    #[test]
    fn empty_board_has_no_attacks() {
        assert!(!square_attacked(
            "8/8/8/8/8/8/8/8 w - - 0 1",
            "e4",
            Color::White
        ));
        assert!(!square_attacked(
            "8/8/8/8/8/8/8/8 w - - 0 1",
            "e4",
            Color::Black
        ));
    }

    #[test]
    fn attack_detection_ignores_side_to_move() {
        assert!(square_attacked(
            "4k3/8/8/8/4R3/8/8/4K3 b - - 0 1",
            "e8",
            Color::White
        ));
    }

    #[test]
    fn find_king_returns_square_for_color() {
        let position = Position::from_fen("4k3/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid test FEN");

        assert_eq!(find_king(&position, Color::White), Some(square("e1")));
        assert_eq!(find_king(&position, Color::Black), Some(square("e8")));
    }

    #[test]
    fn find_king_returns_none_when_missing() {
        let position = Position::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").expect("valid test FEN");

        assert_eq!(find_king(&position, Color::Black), None);
    }

    #[test]
    fn king_not_in_check_returns_false() {
        assert!(!in_check("4k3/8/8/8/8/8/8/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn missing_king_is_not_in_check() {
        assert!(!in_check("8/8/8/8/8/8/8/8 w - - 0 1", Color::White));
    }

    #[test]
    fn rook_check_is_detected() {
        assert!(in_check("4r3/8/8/8/8/8/8/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn bishop_check_is_detected() {
        assert!(in_check("8/8/8/8/7b/8/8/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn queen_check_is_detected() {
        assert!(in_check("4q3/8/8/8/8/8/8/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn knight_check_is_detected() {
        assert!(in_check("8/8/8/8/8/5n2/8/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn pawn_check_is_detected() {
        assert!(in_check("8/8/8/8/8/8/3p4/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn adjacent_king_check_is_detected() {
        assert!(in_check("8/8/8/8/8/8/4k3/4K3 w - - 0 1", Color::White));
    }

    #[test]
    fn legal_move_generation_filters_moves_that_expose_own_king() {
        let moves = legal_moves_from_fen("4r3/8/8/8/8/8/4R3/4K3 w - - 0 1");

        assert!(!moves.contains(&"e2d2".to_string()));
        assert!(moves.contains(&"e2e8".to_string()));
    }

    #[test]
    fn legal_move_generation_allows_moves_that_block_check() {
        let moves = legal_moves_from_fen("4r3/8/8/8/8/8/8/2B1K3 w - - 0 1");

        assert!(moves.contains(&"c1e3".to_string()));
    }

    #[test]
    fn legal_move_generation_allows_king_moves_out_of_check() {
        let moves = legal_moves_from_fen("4r3/8/8/8/8/8/8/4K3 w - - 0 1");

        assert!(moves.contains(&"e1d1".to_string()));
        assert!(!moves.contains(&"e1e2".to_string()));
    }

    #[test]
    fn legal_move_generation_allows_captures_of_checking_piece() {
        let moves = legal_moves_from_fen("8/8/8/8/8/8/4r3/4K3 w - - 0 1");

        assert!(moves.contains(&"e1e2".to_string()));
    }

    #[test]
    fn legal_move_generation_does_not_capture_opponent_king() {
        let moves = legal_moves_from_fen("8/8/8/8/8/8/4k3/4K3 w - - 0 1");

        assert!(!moves.contains(&"e1e2".to_string()));
    }

    #[test]
    fn starting_position_has_twenty_legal_moves() {
        assert_eq!(
            legal_moves_from_fen(crate::board::STARTING_POSITION_FEN).len(),
            20
        );
    }

    #[test]
    fn empty_board_has_no_legal_moves() {
        assert_eq!(legal_moves_from_fen("8/8/8/8/8/8/8/8 w - - 0 1").len(), 0);
    }

    #[test]
    fn perft_depth_zero_counts_current_node() {
        assert_eq!(perft_from_fen(crate::board::STARTING_POSITION_FEN, 0), 1);
    }

    #[test]
    fn perft_depth_one_from_starting_position_is_twenty() {
        assert_eq!(perft_from_fen(crate::board::STARTING_POSITION_FEN, 1), 20);
    }

    #[test]
    fn perft_depth_two_from_starting_position_is_four_hundred() {
        assert_eq!(perft_from_fen(crate::board::STARTING_POSITION_FEN, 2), 400);
    }

    #[test]
    fn perft_empty_board_depth_one_is_zero() {
        assert_eq!(perft_from_fen("8/8/8/8/8/8/8/8 w - - 0 1", 1), 0);
    }

    #[test]
    fn perft_king_only_position_counts_legal_king_moves() {
        assert_eq!(perft_from_fen("4k3/8/8/8/4K3/8/8/8 w - - 0 1", 1), 8);
    }

    #[test]
    fn perft_pinned_piece_position_excludes_illegal_exposures() {
        assert_eq!(perft_from_fen("k3r3/8/8/8/8/8/4R3/4K3 w - - 0 1", 1), 10);
    }
}
