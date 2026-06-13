//! Board-level chess primitives and FEN parsing.

use std::fmt;
use std::str::FromStr;

/// Standard chess starting position in Forsyth-Edwards Notation.
pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Side to move or owner of a chess piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the opposite side.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    fn from_fen_field(value: &str) -> Result<Self, FenParseError> {
        match value {
            "w" => Ok(Self::White),
            "b" => Ok(Self::Black),
            _ => Err(FenParseError::InvalidSideToMove(value.to_string())),
        }
    }
}

/// Kind of chess piece, independent of color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceKind {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceKind {
    fn from_fen_char(value: char) -> Option<Self> {
        match value.to_ascii_lowercase() {
            'k' => Some(Self::King),
            'q' => Some(Self::Queen),
            'r' => Some(Self::Rook),
            'b' => Some(Self::Bishop),
            'n' => Some(Self::Knight),
            'p' => Some(Self::Pawn),
            _ => None,
        }
    }
}

/// A colored chess piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: Color,
    pub kind: PieceKind,
}

impl Piece {
    /// Creates a piece from a color and kind.
    #[must_use]
    pub const fn new(color: Color, kind: PieceKind) -> Self {
        Self { color, kind }
    }

    fn from_fen_char(value: char) -> Option<Self> {
        let color = if value.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        let kind = PieceKind::from_fen_char(value)?;

        Some(Self::new(color, kind))
    }
}

/// Zero-based board coordinate.
///
/// Files and ranks are stored as values from 0 through 7. Algebraic notation
/// parsing is intentionally deferred until FEN and move parsing are introduced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    file: u8,
    rank: u8,
}

impl Square {
    /// Creates a square when both coordinates are inside the board.
    #[must_use]
    pub const fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Self { file, rank })
        } else {
            None
        }
    }

    /// Zero-based file index, from 0 to 7.
    #[must_use]
    pub const fn file(self) -> u8 {
        self.file
    }

    /// Zero-based rank index, from 0 to 7.
    #[must_use]
    pub const fn rank(self) -> u8 {
        self.rank
    }

    /// Creates a square from algebraic notation such as `e4`.
    #[must_use]
    pub fn from_algebraic(value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        let [file, rank] = bytes else {
            return None;
        };

        if !(b'a'..=b'h').contains(file) || !(b'1'..=b'8').contains(rank) {
            return None;
        }

        Self::new(file - b'a', rank - b'1')
    }

    const fn index(self) -> usize {
        (self.rank as usize * 8) + self.file as usize
    }
}

/// Castling availability parsed from a FEN string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CastlingRights {
    pub white_kingside: bool,
    pub white_queenside: bool,
    pub black_kingside: bool,
    pub black_queenside: bool,
}

impl CastlingRights {
    /// Creates castling rights with no side allowed to castle.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            white_kingside: false,
            white_queenside: false,
            black_kingside: false,
            black_queenside: false,
        }
    }

    /// Returns true when no castling rights are available.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        !self.white_kingside
            && !self.white_queenside
            && !self.black_kingside
            && !self.black_queenside
    }

    fn from_fen_field(value: &str) -> Result<Self, FenParseError> {
        if value == "-" {
            return Ok(Self::none());
        }

        let mut rights = Self::none();

        for marker in value.chars() {
            match marker {
                'K' if !rights.white_kingside => rights.white_kingside = true,
                'Q' if !rights.white_queenside => rights.white_queenside = true,
                'k' if !rights.black_kingside => rights.black_kingside = true,
                'q' if !rights.black_queenside => rights.black_queenside = true,
                _ => return Err(FenParseError::InvalidCastlingRights(value.to_string())),
            }
        }

        Ok(rights)
    }
}

/// A structurally parsed chess position.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    squares: [Option<Piece>; 64],
    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant_target: Option<Square>,
    halfmove_clock: u32,
    fullmove_number: u32,
}

impl Position {
    /// Creates an empty board with default game counters.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            squares: [None; 64],
            side_to_move: Color::White,
            castling_rights: CastlingRights::none(),
            en_passant_target: None,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Parses a structurally valid FEN string into a position.
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        fen.parse()
    }

    /// Returns the standard chess starting position.
    pub fn starting_position() -> Result<Self, FenParseError> {
        Self::from_fen(STARTING_POSITION_FEN)
    }

    /// Returns the piece on a square, if any.
    #[must_use]
    pub const fn piece_at(&self, square: Square) -> Option<Piece> {
        self.squares[square.index()]
    }

    /// Returns the side to move.
    #[must_use]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    /// Returns castling rights.
    #[must_use]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// Returns the en passant target square.
    #[must_use]
    pub const fn en_passant_target(&self) -> Option<Square> {
        self.en_passant_target
    }

    /// Returns the halfmove clock.
    #[must_use]
    pub const fn halfmove_clock(&self) -> u32 {
        self.halfmove_clock
    }

    /// Returns the fullmove number.
    #[must_use]
    pub const fn fullmove_number(&self) -> u32 {
        self.fullmove_number
    }

    const fn put_piece(&mut self, square: Square, piece: Piece) {
        self.squares[square.index()] = Some(piece);
    }
}

impl FromStr for Position {
    type Err = FenParseError;

    fn from_str(fen: &str) -> Result<Self, Self::Err> {
        let fields = fen.split_whitespace().collect::<Vec<_>>();
        if fields.len() != 6 {
            return Err(FenParseError::InvalidFieldCount {
                found: fields.len(),
            });
        }

        let mut position = Self::empty();
        parse_piece_placement(fields[0], &mut position)?;

        position.side_to_move = Color::from_fen_field(fields[1])?;
        position.castling_rights = CastlingRights::from_fen_field(fields[2])?;
        position.en_passant_target = parse_en_passant_target(fields[3])?;
        position.halfmove_clock = parse_halfmove_clock(fields[4])?;
        position.fullmove_number = parse_fullmove_number(fields[5])?;

        Ok(position)
    }
}

fn parse_piece_placement(value: &str, position: &mut Position) -> Result<(), FenParseError> {
    let ranks = value.split('/').collect::<Vec<_>>();
    if ranks.len() != 8 {
        return Err(FenParseError::InvalidRankCount { found: ranks.len() });
    }

    for (fen_rank_index, rank) in ranks.iter().enumerate() {
        let mut file = 0_u8;

        for marker in rank.chars() {
            if let Some(empty_squares) = marker.to_digit(10) {
                if !(1..=8).contains(&empty_squares) {
                    return Err(FenParseError::InvalidRankWidth {
                        rank: fen_rank_index + 1,
                        files: file,
                    });
                }

                file = file.saturating_add(empty_squares as u8);
            } else if marker.is_ascii_alphabetic() {
                let piece =
                    Piece::from_fen_char(marker).ok_or(FenParseError::InvalidPieceChar(marker))?;
                let board_rank = 7_u8 - fen_rank_index as u8;
                let square =
                    Square::new(file, board_rank).ok_or(FenParseError::InvalidRankWidth {
                        rank: fen_rank_index + 1,
                        files: file + 1,
                    })?;

                position.put_piece(square, piece);
                file += 1;
            } else {
                return Err(FenParseError::InvalidPieceChar(marker));
            }

            if file > 8 {
                return Err(FenParseError::InvalidRankWidth {
                    rank: fen_rank_index + 1,
                    files: file,
                });
            }
        }

        if file != 8 {
            return Err(FenParseError::InvalidRankWidth {
                rank: fen_rank_index + 1,
                files: file,
            });
        }
    }

    Ok(())
}

fn parse_en_passant_target(value: &str) -> Result<Option<Square>, FenParseError> {
    if value == "-" {
        return Ok(None);
    }

    Square::from_algebraic(value)
        .filter(|square| matches!(square.rank(), 2 | 5))
        .map(Some)
        .ok_or_else(|| FenParseError::InvalidEnPassantSquare(value.to_string()))
}

fn parse_halfmove_clock(value: &str) -> Result<u32, FenParseError> {
    value
        .parse::<u32>()
        .map_err(|_| FenParseError::InvalidHalfmoveClock(value.to_string()))
}

fn parse_fullmove_number(value: &str) -> Result<u32, FenParseError> {
    let fullmove_number = value
        .parse::<u32>()
        .map_err(|_| FenParseError::InvalidFullmoveNumber(value.to_string()))?;

    if fullmove_number == 0 {
        return Err(FenParseError::InvalidFullmoveNumber(value.to_string()));
    }

    Ok(fullmove_number)
}

/// Errors produced when parsing structurally invalid FEN strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FenParseError {
    InvalidFieldCount { found: usize },
    InvalidRankCount { found: usize },
    InvalidRankWidth { rank: usize, files: u8 },
    InvalidPieceChar(char),
    InvalidSideToMove(String),
    InvalidCastlingRights(String),
    InvalidEnPassantSquare(String),
    InvalidHalfmoveClock(String),
    InvalidFullmoveNumber(String),
}

impl fmt::Display for FenParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFieldCount { found } => {
                write!(formatter, "expected 6 FEN fields, found {found}")
            }
            Self::InvalidRankCount { found } => {
                write!(formatter, "expected 8 FEN ranks, found {found}")
            }
            Self::InvalidRankWidth { rank, files } => {
                write!(
                    formatter,
                    "expected 8 files in FEN rank {rank}, found {files}"
                )
            }
            Self::InvalidPieceChar(value) => write!(formatter, "invalid FEN piece `{value}`"),
            Self::InvalidSideToMove(value) => write!(formatter, "invalid side to move `{value}`"),
            Self::InvalidCastlingRights(value) => {
                write!(formatter, "invalid castling rights `{value}`")
            }
            Self::InvalidEnPassantSquare(value) => {
                write!(formatter, "invalid en passant square `{value}`")
            }
            Self::InvalidHalfmoveClock(value) => {
                write!(formatter, "invalid halfmove clock `{value}`")
            }
            Self::InvalidFullmoveNumber(value) => {
                write!(formatter, "invalid fullmove number `{value}`")
            }
        }
    }
}

impl std::error::Error for FenParseError {}

#[cfg(test)]
fn square(value: &str) -> Square {
    Square::from_algebraic(value).expect("valid test square")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_opposite_switches_sides() {
        assert_eq!(Color::White.opposite(), Color::Black);
        assert_eq!(Color::Black.opposite(), Color::White);
    }

    #[test]
    fn piece_keeps_color_and_kind() {
        let piece = Piece::new(Color::White, PieceKind::Knight);

        assert_eq!(piece.color, Color::White);
        assert_eq!(piece.kind, PieceKind::Knight);
    }

    #[test]
    fn square_accepts_coordinates_inside_board() {
        let square = Square::new(4, 3).expect("valid square");

        assert_eq!(square.file(), 4);
        assert_eq!(square.rank(), 3);
    }

    #[test]
    fn square_rejects_coordinates_outside_board() {
        assert_eq!(Square::new(8, 0), None);
        assert_eq!(Square::new(0, 8), None);
    }

    #[test]
    fn square_parses_algebraic_coordinates() {
        assert_eq!(
            Square::from_algebraic("a1"),
            Some(Square::new(0, 0).unwrap())
        );
        assert_eq!(
            Square::from_algebraic("h8"),
            Some(Square::new(7, 7).unwrap())
        );
        assert_eq!(Square::from_algebraic("i1"), None);
        assert_eq!(Square::from_algebraic("a9"), None);
        assert_eq!(Square::from_algebraic("a10"), None);
    }

    #[test]
    fn parses_standard_starting_position() {
        let position = Position::from_fen(STARTING_POSITION_FEN).expect("starting position");

        assert_eq!(position.side_to_move(), Color::White);
        assert_eq!(
            position.castling_rights(),
            CastlingRights {
                white_kingside: true,
                white_queenside: true,
                black_kingside: true,
                black_queenside: true,
            }
        );
        assert_eq!(position.en_passant_target(), None);
        assert_eq!(position.halfmove_clock(), 0);
        assert_eq!(position.fullmove_number(), 1);
        assert_eq!(
            position.piece_at(square("a1")),
            Some(Piece::new(Color::White, PieceKind::Rook))
        );
        assert_eq!(
            position.piece_at(square("e1")),
            Some(Piece::new(Color::White, PieceKind::King))
        );
        assert_eq!(
            position.piece_at(square("d8")),
            Some(Piece::new(Color::Black, PieceKind::Queen))
        );
        assert_eq!(
            position.piece_at(square("h7")),
            Some(Piece::new(Color::Black, PieceKind::Pawn))
        );
        assert_eq!(position.piece_at(square("e4")), None);
    }

    #[test]
    fn parses_empty_board() {
        let position = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").expect("empty board");

        assert_eq!(position.side_to_move(), Color::White);
        assert!(position.castling_rights().is_empty());
        assert_eq!(position.en_passant_target(), None);
        assert_eq!(position.piece_at(square("a1")), None);
        assert_eq!(position.piece_at(square("h8")), None);
    }

    #[test]
    fn parses_position_with_black_to_move() {
        let position =
            Position::from_fen("8/8/8/3p4/4P3/8/8/8 b - - 12 34").expect("black to move");

        assert_eq!(position.side_to_move(), Color::Black);
        assert_eq!(position.halfmove_clock(), 12);
        assert_eq!(position.fullmove_number(), 34);
        assert_eq!(
            position.piece_at(square("d5")),
            Some(Piece::new(Color::Black, PieceKind::Pawn))
        );
        assert_eq!(
            position.piece_at(square("e4")),
            Some(Piece::new(Color::White, PieceKind::Pawn))
        );
    }

    #[test]
    fn parses_all_castling_rights() {
        let position =
            Position::from_fen("8/8/8/8/8/8/8/8 w KQkq - 0 1").expect("all castling rights");

        assert!(position.castling_rights().white_kingside);
        assert!(position.castling_rights().white_queenside);
        assert!(position.castling_rights().black_kingside);
        assert!(position.castling_rights().black_queenside);
    }

    #[test]
    fn parses_no_castling_rights() {
        let position = Position::from_fen("8/8/8/8/8/8/8/8 w - - 0 1").expect("no castling rights");

        assert!(position.castling_rights().is_empty());
    }

    #[test]
    fn parses_partial_castling_rights() {
        let position =
            Position::from_fen("8/8/8/8/8/8/8/8 w Kq - 0 1").expect("partial castling rights");

        assert!(position.castling_rights().white_kingside);
        assert!(!position.castling_rights().white_queenside);
        assert!(!position.castling_rights().black_kingside);
        assert!(position.castling_rights().black_queenside);
    }

    #[test]
    fn parses_en_passant_square() {
        let position =
            Position::from_fen("8/8/8/3pP3/8/8/8/8 w - d6 0 1").expect("en passant target");

        assert_eq!(position.en_passant_target(), Some(square("d6")));
        assert_eq!(
            position.piece_at(square("d5")),
            Some(Piece::new(Color::Black, PieceKind::Pawn))
        );
        assert_eq!(
            position.piece_at(square("e5")),
            Some(Piece::new(Color::White, PieceKind::Pawn))
        );
    }

    #[test]
    fn rejects_malformed_fen_strings() {
        let cases = [
            (
                "8/8/8/8/8/8/8 w - - 0 1",
                FenParseError::InvalidRankCount { found: 7 },
            ),
            (
                "8/8/8/8/8/8/8/9 w - - 0 1",
                FenParseError::InvalidRankWidth { rank: 8, files: 0 },
            ),
            (
                "8/8/8/8/8/8/8/7 w - - 0 1",
                FenParseError::InvalidRankWidth { rank: 8, files: 7 },
            ),
            (
                "8/8/8/8/8/8/8/7x w - - 0 1",
                FenParseError::InvalidPieceChar('x'),
            ),
            (
                "8/8/8/8/8/8/8/8 x - - 0 1",
                FenParseError::InvalidSideToMove("x".to_string()),
            ),
            (
                "8/8/8/8/8/8/8/8 w ABC - 0 1",
                FenParseError::InvalidCastlingRights("ABC".to_string()),
            ),
            (
                "8/8/8/8/8/8/8/8 w - e4 0 1",
                FenParseError::InvalidEnPassantSquare("e4".to_string()),
            ),
            (
                "8/8/8/8/8/8/8/8 w - - nope 1",
                FenParseError::InvalidHalfmoveClock("nope".to_string()),
            ),
            (
                "8/8/8/8/8/8/8/8 w - - 0 0",
                FenParseError::InvalidFullmoveNumber("0".to_string()),
            ),
            (
                "8/8/8/8/8/8/8/8 w - - 0",
                FenParseError::InvalidFieldCount { found: 5 },
            ),
        ];

        for (fen, expected_error) in cases {
            assert_eq!(Position::from_fen(fen), Err(expected_error));
        }
    }
}
