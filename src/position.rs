use crate::bitboard::*;
use crate::chess_move::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
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

use Color::*;

impl std::ops::Not for Color {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            White => Black,
            Black => White
        }
    }
}

use Piece::*;

const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug)]
struct StackFrame {
    castling_rights: u8,

    en_passant_file: Option<u8>,

    half_move_clock: u32,

    captured_piece: Piece,
}

#[derive(Debug)]
pub struct Position {
    squares: [Piece; 64],

    piece_bb: [Bitboard; 6],
    
    color_bb: [Bitboard; 2],

    full_move_clock: u32,

    current_player: Color,

    stack: Vec<StackFrame>,
}

impl Position {
    pub fn new() -> Position {
        Self::from_fen_string(START_FEN).expect("the fen of the starting position should parse without an error")
    }

    pub fn from_fen_string(fen: &str) -> Result<Position, ()> {
        let mut p = Position {
            squares: [NoPiece; 64],
            piece_bb: [Bitboard::from_u64(0); 6],
            color_bb: [Bitboard::from_u64(0); 2],
            full_move_clock: 0,
            current_player: White,
            stack: vec![StackFrame {
                castling_rights: 0,
                en_passant_file: None,
                half_move_clock: 0,
                captured_piece: NoPiece
            }]
        };

        let mut sections = fen.split(' ');


        p.parse_board(sections.next().ok_or(())?)?;
        p.parse_player(sections.next().ok_or(())?)?;
        p.parse_castling(sections.next().ok_or(())?)?;
        p.parse_en_passant(sections.next().ok_or(())?)?;

        p.stack.last_mut().unwrap().half_move_clock = Self::parse_int(sections.next().ok_or(())?, u32::MAX/10)?;
        p.full_move_clock = Self::parse_int(sections.next().ok_or(())?, u32::MAX/10)?;

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

                let piece_color = if c.is_ascii_uppercase() {
                    White
                } else {
                    Black
                };
                self.color_bb[piece_color as usize] |= Bitboard::from_coords(column, row);
                self.piece_bb[self.squares[(column+8*row) as usize] as usize] |= Bitboard::from_coords(column, row);
                
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
            self.stack.last_mut().unwrap().castling_rights = 0;
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
                self.stack.last_mut().unwrap().castling_rights |= 1 << i;
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
            self.stack.last_mut().unwrap().en_passant_file = None;
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
        
        if !((self.squares[(from_rank*8+file) as usize] == Pawn) && (self.square_color(from_rank*8+file) != Some(self.current_player))) {
            self.stack.last_mut().unwrap().en_passant_file = None;
            return Ok(());
        }

        //check if there are own pawns next to the enemy pawn
        let mut from_squares = [-1,1].into_iter().map(|offset| offset + file as i32).filter(|x| *x <= 7 && *x >= 0).map(|x| x as u8 + 8*from_rank);
        let pawn_on_from_square = from_squares.any(|x| self.squares[x as usize] == Pawn && Some(self.current_player) == self.square_color(x));

        if pawn_on_from_square {
            self.stack.last_mut().unwrap().en_passant_file = Some(file);
        } else {
            self.stack.last_mut().unwrap().en_passant_file = None;
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

        Ok((8*rank+file) as u8)
    }

    pub fn square_color(&self, square: u8) -> Option<Color> {
        if self.color_bb[Color::White as usize].contains(square) {
            Some(White)
        } else if self.color_bb[Color::Black as usize].contains(square) {
            Some(Black)
        } else {
            None
        }
    }

    /*
     * making and unmaking moves
     */

    const CASTLE_ROOK_FROM: [u8; 4] = [63, 56, 7, 0];
    const CASTLE_ROOK_TO: [u8; 4] = [61 , 59, 5, 3];

    pub fn make_move(&mut self, m: Move) {
        assert!(!self.stack.is_empty());

        self.stack.push(StackFrame {
            castling_rights: self.stack.last().unwrap().castling_rights,
            en_passant_file: None,
            half_move_clock: self.stack.last().unwrap().half_move_clock + 1,
            captured_piece: self.squares[m.to() as usize]
        });
        
        let moved_piece = self.squares[m.from() as usize];

        self.color_bb[self.current_player as usize] &= !Bitboard::from_square(m.from());
        self.color_bb[self.current_player as usize] |= Bitboard::from_square(m.to());
        self.piece_bb[moved_piece as usize] &= !Bitboard::from_square(m.from());
        self.piece_bb[moved_piece as usize] |= Bitboard::from_square(m.to());
        self.squares[m.from() as usize] = NoPiece;
        self.squares[m.to() as usize] = moved_piece;

        if self.stack.last().unwrap().captured_piece != NoPiece {
            self.color_bb[!self.current_player as usize] &= !Bitboard::from_square(m.to());
            self.piece_bb[!self.current_player as usize] &= !Bitboard::from_square(m.to());
            self.stack.last_mut().unwrap().half_move_clock = 0;
        }

        if m.is_en_passant() {
            let captured_pawn_square = if self.current_player == White {
                m.to() + 8
            } else {
                m.to() - 8
            };

            self.piece_bb[Pawn as usize] &= !Bitboard::from_square(captured_pawn_square);
            self.color_bb[!self.current_player as usize] &= !Bitboard::from_square(captured_pawn_square);
            self.squares[captured_pawn_square as usize] = NoPiece;

            self.stack.last_mut().unwrap().half_move_clock = 0;
            
        } else if let Some(p) = m.promote_to() {
            self.piece_bb[Pawn as usize] &= !Bitboard::from_square(m.to());
            self.piece_bb[p as usize] |= Bitboard::from_square(m.to());
            self.squares[m.to() as usize] = p;

            self.stack.last_mut().unwrap().half_move_clock = 0;

        } else if let Some(c) = m.castling_type() {
            let rook_from = Self::CASTLE_ROOK_FROM[c as usize];
            let rook_to = Self::CASTLE_ROOK_TO[c as usize];

            self.squares[rook_from as usize] = NoPiece;
            self.squares[rook_to as usize] = Rook;

            self.piece_bb[Rook as usize] &= !Bitboard::from_square(rook_from);
            self.piece_bb[Rook as usize] |= Bitboard::from_square(rook_to);

            self.color_bb[self.current_player as usize] &= !Bitboard::from_square(rook_from);
            self.color_bb[self.current_player as usize] |= Bitboard::from_square(rook_to);

            self.stack.last_mut().unwrap().half_move_clock = 0;

            //castling rights are removed below
        }

        if moved_piece == Pawn {
            if ((m.to() as i32) - (m.from() as i32)) % 16 == 0 { //two step pawn move
                let en_passant_capture_squares = Bitboard::from_square(m.to()).shift(Direction::Left) | Bitboard::from_square(m.to()).shift(Direction::Right);
        
                if !(en_passant_capture_squares & self.color_bb[!self.current_player as usize] & self.piece_bb[Pawn as usize]).is_empty() { //opposite color pawns next to target square
                    self.stack.last_mut().unwrap().en_passant_file = Some(m.to() % 8);
                } 
            }

            self.stack.last_mut().unwrap().half_move_clock = 0;

        } 
        
        if moved_piece == King {
            if self.current_player == White {
                self.stack.last_mut().unwrap().castling_rights &= !(1 << CastlingType::WhiteCastleKingside as u8) & !(1 << CastlingType::WhiteCastleQueenside as u8);
            } else {
                self.stack.last_mut().unwrap().castling_rights &= !(1 << CastlingType::BlackCastleKingside as u8) & !(1 << CastlingType::BlackCastleQueenside as u8);
            }
        }

        for i in 0..4 {
            if m.to() == Self::CASTLE_ROOK_FROM[i] || m.from() == Self::CASTLE_ROOK_FROM[i] {
                self.stack.last_mut().unwrap().castling_rights &= 1 << i;
            }
        }

        if self.current_player == Black {
            self.full_move_clock += 1;
        }

        self.current_player = !self.current_player;
    }
    
    pub fn unmake_move(&mut self, m: Move) {
        let captured_piece = self.stack.pop().unwrap().captured_piece;
        let moved_piece = self.squares[m.to() as usize];

        self.current_player = !self.current_player;

        self.piece_bb[moved_piece as usize] |= Bitboard::from_square(m.from());
        self.piece_bb[moved_piece as usize] &= !Bitboard::from_square(m.to());

        self.color_bb[self.current_player as usize] |= Bitboard::from_square(m.from());
        self.color_bb[self.current_player as usize] &= !Bitboard::from_square(m.to());

        self.squares[m.to() as usize] = captured_piece;
        self.squares[m.from() as usize] = moved_piece;

        if captured_piece != NoPiece {
            self.piece_bb[captured_piece as usize] |= Bitboard::from_square(m.to());
            self.color_bb[!self.current_player as usize] |= Bitboard::from_square(m.to());
        }

        if m.is_en_passant() {
            let captured_pawn_square = if self.current_player == White {
                m.to() + 8
            } else {
                m.to() - 8
            };

            self.piece_bb[Pawn as usize] |= Bitboard::from_square(captured_pawn_square);
            self.color_bb[!self.current_player as usize] |= Bitboard::from_square(captured_pawn_square);
            self.squares[captured_pawn_square as usize] = Pawn;
        } else if let Some(p) = m.promote_to() {
            self.squares[m.from() as usize] = Pawn;
            self.piece_bb[Pawn as usize] |= Bitboard::from_square(m.from());
            self.piece_bb[p as usize] &= !Bitboard::from_square(m.from());
            
        } else if let Some(c) = m.castling_type() {
            let rook_from = Self::CASTLE_ROOK_FROM[c as usize];
            let rook_to = Self::CASTLE_ROOK_TO[c as usize];

            self.squares[rook_from as usize] = Rook;
            self.squares[rook_to as usize] = NoPiece;

            self.piece_bb[Rook as usize] |= Bitboard::from_square(rook_from);
            self.piece_bb[Rook as usize] &= !Bitboard::from_square(rook_to);

            self.color_bb[self.current_player as usize] |= Bitboard::from_square(rook_from);
            self.color_bb[self.current_player as usize] &= !Bitboard::from_square(rook_to);
        } 
    }

    
    /*
     * move generation
     */

    pub fn get_legal_moves(&self) -> Vec<Move> {
        let moves = Vec::new();

        moves
    }

    fn is_legal(&self, m: Move) -> bool {
        false
    }

    fn king_in_check(&self, player: Color) -> bool {
        false
    }
}

