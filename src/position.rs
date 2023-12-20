use crate::bitboard::Bitboard;

#[derive(Clone, Copy)]
pub enum Pieces {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    NoPiece
}

use Pieces::*;

pub struct Position {
    squares: [Pieces; 64],

    bb: [Bitboard; 6],
    
    white: Bitboard
}

impl Position {
    pub fn from_fen_string(fen: &str) -> Result<Position, ()> {
        let mut p = Position {
            squares: [NoPiece; 64],
            bb: [Bitboard::from_u64(0); 6],
            white: Bitboard::from_u64(0)
        };

        let mut column = 0;
        let mut row = 0;

        Ok(p)
    }
}