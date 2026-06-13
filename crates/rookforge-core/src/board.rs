//! Board-level chess primitives.

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
}
