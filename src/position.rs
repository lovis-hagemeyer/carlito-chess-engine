use crate::bitboard::Bitboard;

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug)]
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

        let mut sections = fen.split(' ');


        p.parse_board(sections.next().ok_or(())?)?;
        p.parse_player(sections.next().ok_or(())?)?;
        p.parse_castling(sections.next().ok_or(())?)?;
        p.parse_en_passant(sections.next().ok_or(())?)?;

        Ok(p)
    }

    fn parse_board(&mut self, s: &str) -> Result<(), ()> {
        let mut row = 0;
        let mut column = 0;

        for c in s.chars() {
            println!("parsing {c}");

            if column == 8 { //expecting '/'.
                if c != '/' {
                    return Err(());
                }
                if row >= 7{
                    return Err(());
                }

                row += 1;
                column = 0;

            } else {
                if let Some(d) = c.to_digit(9) {
                    if c == '0' {
                        return Err(());
                    }
                    
                    column += d as u8;

                    if column > 8 {
                        return Err(());
                    }
                } else {
                    self.squares[(column+8*row) as usize] = match c.to_ascii_uppercase() {
                        'P' => Pawn,
                        'N' => Knight,
                        'B' => Bishop,
                        'R' => Rook,
                        'Q' => Queen,
                        'K' => King,
                        _ => return Err(())
                    };

                    if c.is_ascii_uppercase() {
                        self.white |= Bitboard::from_coords(column, row)
                    }

                    self.bb[self.squares[(column+8*row) as usize] as usize] |= Bitboard::from_coords(column, row);
                    
                    column += 1;
                }
            }
        }

        if row != 7 || column != 8 {
            return Err(());
        }

        Ok(())
    } 

    fn parse_player(&mut self, s: &str) -> Result<(), ()> {
        Ok(())
    }

    fn parse_castling(&mut self, s: &str) -> Result<(), ()> {
        Ok(())
    }

    fn parse_en_passant(&mut self, s: &str) -> Result<(), ()> {
        Ok(())
    }
}

