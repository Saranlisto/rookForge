//! Core library scaffold for Rookforge.
//!
//! The crate defines the early module boundaries and the smallest shared chess
//! vocabulary needed by the CLI and future engine work.

pub mod board;
pub mod eval;
pub mod movegen;
pub mod search;
pub mod uci;

pub use board::{Color, Piece, PieceKind, Square};
pub use movegen::Move;

/// Human-readable engine name used by the CLI and future UCI identification.
pub const ENGINE_NAME: &str = "Rookforge";
