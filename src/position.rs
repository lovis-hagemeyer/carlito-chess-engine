use crate::bitboard::Bitboard;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pieces {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    NoPiece
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    White,
    Black
}

use Pieces::*;


const WHITE_CASTLE_KINGSIDE: u8 = 1 << 0;
const WHITE_CASTLE_QUEENSIDE: u8 = 1 << 1;
const BLACK_CASTLE_KINGSIDE: u8 = 1 << 2;
const BLACK_CASTLE_QUEENSIDE: u8 = 1 << 3;

const START_FEN: &str = "";

#[derive(Debug)]
pub struct Position {
    squares: [Pieces; 64],

    bb: [Bitboard; 6],
    
    white: Bitboard,

    full_move_clock: u32,
    half_move_clock: u32,

    castling_rights: u8,

    en_passant_file: Option<u8>,

    current_player: Color,
}

use Color::*;

impl Position {
    pub fn new() -> Position {
        Self::from_fen_string(START_FEN).expect("the fen of the starting position should parse without an error")
    }

    pub fn from_fen_string(fen: &str) -> Result<Position, ()> {
        let mut p = Position {
            squares: [NoPiece; 64],
            bb: [Bitboard::from_u64(0); 6],
            white: Bitboard::from_u64(0),
            full_move_clock: 0,
            half_move_clock: 0,
            castling_rights: 0,
            en_passant_file: None,
            current_player: White
        };

        let mut sections = fen.split(' ');


        p.parse_board(sections.next().ok_or(())?)?;
        p.parse_player(sections.next().ok_or(())?)?;
        p.parse_castling(sections.next().ok_or(())?)?;
        p.parse_en_passant(sections.next().ok_or(())?)?;

        p.full_move_clock = Self::parse_int(sections.next().ok_or(())?, u32::MAX/10)?;
        p.half_move_clock = Self::parse_int(sections.next().ok_or(())?, u32::MAX/10)?;

        if sections.next().is_some() {
            return Err(());
        }

        Ok(p)
    }

    fn parse_board(&mut self, s: &str) -> Result<(), ()> {
        let mut row = 0;
        let mut column = 0;

        for c in s.chars() {
            if column == 8 { //expecting '/'.
                if c != '/' {
                    return Err(());
                }
                if row >= 7{
                    return Err(());
                }

                row += 1;
                column = 0;

            } else if let Some(d) = c.to_digit(9) {
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

        if row != 7 || column != 8 {
            return Err(());
        }

        Ok(())
    } 

    fn parse_player(&mut self, s: &str) -> Result<(), ()> {
        if s.len() != 1 {
            return Err(());
        }

        self.current_player = match s.chars().next().unwrap() {
            'w' => White,
            'b' => Black,
            _ => return Err(())
        };

        Ok(())
    }

    fn parse_castling(&mut self, s: &str) -> Result<(), ()> {
        if s == "-" {
            self.castling_rights = 0;
            return Ok(());
        }

        if s.is_empty() {
            return Err(());
        }
        
        let mut iter = s.chars().peekable();

        for (i, c) in ['K', 'Q', 'k', 'q'].iter().enumerate() {
            if iter.peek().is_none() {
                break;
            }

            if iter.peek().unwrap() == c {
                self.castling_rights |= 1 << i;
                iter.next();
            }
        }

        if iter.next().is_some() {
            return Err(());
        }

        Ok(())
    }

    fn parse_en_passant(&mut self, s: &str) -> Result<(), ()> {

        if s == "-" {
            self.en_passant_file = None;
            return Ok(());
        }

        let square = Position::parse_square(s)?;

        let rank = square / 8;
        let file = square % 8;

        if (self.current_player == White && rank != 2) || (self.current_player == Black && rank != 5) {
            return Err(());
        }

        //only save en passant square, if there is a pseudo legal en passant move

        let from_rank = if self.current_player == White { rank + 1 } else { rank - 1 };

        //check if there is an enemy pawn above/below the en passant target square
        if !((self.squares[(from_rank*8+file) as usize] == Pawn) && (self.square_color(from_rank*8+file) != self.current_player)) {
            self.en_passant_file = None;
            return Ok(());
        }

        //check if there are own pawns next to the enemy pawn
        let mut from_squares = [-1,1].into_iter().map(|offset| offset + file as i32).filter(|x| *x <= 7 && *x >= 0).map(|x| x as u8 + 8*from_rank);
        let pawn_on_from_square = from_squares.any(|x| self.squares[x as usize] == Pawn && self.current_player == self.square_color(x));

        if pawn_on_from_square {
            self.en_passant_file = Some(file);
        } else {
            self.en_passant_file = None;
        }

        Ok(())
    }

    fn parse_int(s: &str, max_val: u32) -> Result<u32, ()> {
        let mut res: u32 = 0;
        
        for c in s.chars() {
            if let Some(d) = c.to_digit(10) {
                res = res.saturating_mul(10);
                res = res.saturating_add(d);

                res = res.min(max_val);
            } else {
                return Err(());
            }
        }

        Ok(res)
    }

    pub fn parse_square(s: &str) -> Result<u8, ()> {

        let mut str_iter = s.chars();

        let first_char = str_iter.next().ok_or(())?;
        let second_char = str_iter.next().ok_or(())?;
        
        if str_iter.next().is_some() {
            return Err(());
        }

        if !"abcdefgh".contains(first_char) {
            return Err(());
        }

        if !"12345678".contains(second_char) {
            return Err(());
        }     

        let file = first_char.to_digit(18).unwrap()-10;
        let rank = 8-second_char.to_digit(10).unwrap();

        println!("{}", 8*rank+file);

        Ok((8*rank+file) as u8)
    }

    pub fn square_color(&self, square: u8) -> Color {
        if self.white.contains(square) {
            White
        } else {
            Black
        }
    } 
}

