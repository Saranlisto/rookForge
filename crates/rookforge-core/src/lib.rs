//! Core library scaffold for Rookforge.
//!
//! The crate defines the early module boundaries and the smallest shared chess
//! vocabulary needed by the CLI and future engine work.

pub mod board;
pub mod eval;
pub mod movegen;
pub mod search;
pub mod uci;

pub use board::{
    CastlingRights, Color, FenParseError, Piece, PieceKind, Position, Square, STARTING_POSITION_FEN,
};
pub use movegen::{
    apply_move, find_king, generate_bishop_moves, generate_castling_moves, generate_king_moves,
    generate_knight_moves, generate_legal_moves, generate_non_sliding_moves, generate_pawn_moves,
    generate_pseudo_legal_moves, generate_queen_moves, generate_rook_moves,
    generate_sliding_piece_moves, is_in_check, is_square_attacked, perft, Move, MoveApplyError,
    MoveParseError,
};

/// Human-readable engine name used by the CLI and future UCI identification.
pub const ENGINE_NAME: &str = "Rookforge";
