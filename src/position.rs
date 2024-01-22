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

#[derive(Debug, Clone)]
struct StackFrame {
    castling_rights: u8,

    en_passant_file: Option<u8>,

    half_move_clock: u32,

    captured_piece: Piece,

    pinned: Bitboard,

    hash: u64
}

#[derive(Debug, Clone)]
pub struct Position {
    squares: [Piece; 64],

    piece_bb: [Bitboard; 6],
    
    color_bb: [Bitboard; 2],

    full_move_clock: u32,

    current_player: Color,

    stack: Vec<StackFrame>,
}

impl Position {

    pub const LIGHT_SQUARES: Bitboard = Bitboard::from_u64(0xaa55aa55aa55aa55); 
    pub const DARK_SQUARES: Bitboard = Bitboard::from_u64(0x55aa55aa55aa55aa);

    pub fn new() -> Position {
        Self::from_fen_string(START_FEN).expect("the fen of the starting position should parse without an error")
    }

    /*
     * getters
     */

    pub fn pieces(&self, piece: Piece, player: Color) -> Bitboard {
        self.color_bb[player as usize] & self.piece_bb[piece as usize]
    }

    pub fn pieces_by_piece_type(&self, piece: Piece) -> Bitboard {
        self.piece_bb[piece as usize]
    }

    pub fn pieces_py_player(&self, player: Color) -> Bitboard {
        self.color_bb[player as usize]
    }

    pub fn occupied(&self) -> Bitboard {
        self.color_bb[Black as usize] | self.color_bb[White as usize]
    }

    pub fn king_square(&self, player: Color) -> u8 {
        self.pieces(King, player).into_iter().next().expect("each player should always have a king on the board.")
    }

    pub fn current_player(&self) -> Color {
        self.current_player
    }

    pub fn piece_on(&self, square: u8) -> Piece{
        self.squares[square as usize]
    }

    pub fn hash(&self) -> u64 {
        self.stack.last().unwrap().hash
    }

    pub fn half_move_clock(&self) -> u32 {
        self.stack.last().unwrap().half_move_clock
    }

    /*
     * fen parsing
     */

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
                captured_piece: NoPiece,
                pinned: Bitboard::new(),
                hash: 0
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

        p.stack.last_mut().unwrap().pinned = p.pinned_pieces();
        p.stack.last_mut().unwrap().hash = p.calculate_hash();

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

        //only save en passant square if there is a pseudo legal en passant move

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

    pub fn parse_int(s: &str, max_val: u32) -> Result<u32, ()> {
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

    pub fn square_to_string(square: u8) -> String {
        let mut res = String::from(char::from_digit((square%8+10) as u32, 18).unwrap());
        res.push(char::from_digit((7-square/8+1) as u32, 9).unwrap());
        res
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

        let moved_piece = self.squares[m.from() as usize];
        let captured_piece = self.squares[m.to() as usize];

        self.stack.push(StackFrame {
            castling_rights: self.stack.last().unwrap().castling_rights,
            en_passant_file: None,
            half_move_clock: self.stack.last().unwrap().half_move_clock + 1,
            captured_piece,
            pinned: Bitboard::new(),
            hash: 0
        });

        if captured_piece != NoPiece {
            self.color_bb[!self.current_player as usize] &= !Bitboard::from_square(m.to());
            self.piece_bb[captured_piece as usize] &= !Bitboard::from_square(m.to());
            self.stack.last_mut().unwrap().half_move_clock = 0;
        }

        self.color_bb[self.current_player as usize] &= !Bitboard::from_square(m.from());
        self.color_bb[self.current_player as usize] |= Bitboard::from_square(m.to());
        self.piece_bb[moved_piece as usize] &= !Bitboard::from_square(m.from());
        self.piece_bb[moved_piece as usize] |= Bitboard::from_square(m.to());
        self.squares[m.from() as usize] = NoPiece;
        self.squares[m.to() as usize] = moved_piece;

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

            //half move clock is set to zero below

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
        
                if !(en_passant_capture_squares & self.pieces(Pawn, !self.current_player)).is_empty() { //opposite color pawns next to target square
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
                self.stack.last_mut().unwrap().castling_rights &= !(1 << i);
            }
        }

        if self.current_player == Black {
            self.full_move_clock += 1;
        }

        self.current_player = !self.current_player;

        self.stack.last_mut().unwrap().pinned = self.pinned_pieces();
        self.stack.last_mut().unwrap().hash = self.calculate_hash();
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

    pub fn legal_moves(&mut self) -> Vec<Move> {
        self.pseudo_legal_moves().into_iter().filter(|m| self.is_legal(*m)).collect()
    }

    fn pseudo_legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        let target_squares = self.check_blocking_squares(self.current_player);

        let in_check = target_squares != !Bitboard::new();

        //knight moves
        for piece in self.pieces(Knight, self.current_player) {
            let knight_moves = Bitboard::knight_attacks(piece) & !self.pieces_py_player(self.current_player) & target_squares;
            for target in knight_moves {
                moves.push(Move::new(piece, target));
            }
        }

        //pawn moves
        let forward = if self.current_player == White { Direction::Up } else { Direction::Down };
        let sideways: [Direction; 2] = if self.current_player == White { [Direction::UpLeft, Direction::UpRight] } else { [Direction::DownLeft, Direction::DownRight] };
        let double_move_target_rank = if self.current_player == White { Bitboard::rank(4) } else { Bitboard::rank(3) };

        let forward_targets = self.pieces(Pawn, self.current_player).shift(forward) & !self.occupied();
        let double_move_targets = forward_targets.shift(forward) & !self.occupied() & double_move_target_rank;
        
        Self::generate_pawn_moves(&mut moves, forward_targets & target_squares, forward.offset());
        Self::generate_pawn_moves(&mut moves, double_move_targets & target_squares, 2 * forward.offset());

        for direction in sideways {
            let capture_targets = self.pieces(Pawn, self.current_player).shift(direction) & self.pieces_py_player(!self.current_player);
            Self::generate_pawn_moves(&mut moves, capture_targets & target_squares, direction.offset());
        }

        //bishop moves and diagonal queen moves
        for piece in self.pieces(Bishop, self.current_player) | self.pieces(Queen, self.current_player) {
            let bishop_moves = Bitboard::bishop_attacks(piece, self.occupied()) & !self.pieces_py_player(self.current_player) & target_squares;
            for target in bishop_moves {
                moves.push(Move::new(piece, target));
            }
        }

        //rook moves and vertical and horizontal queen moves
        for piece in self.pieces(Rook, self.current_player) | self.pieces(Queen, self.current_player) {
            let rook_moves = Bitboard::rook_attacks(piece, self.occupied()) & !self.pieces_py_player(self.current_player) & target_squares;
            for target in rook_moves {
                moves.push(Move::new(piece, target));
            }
        }

        //king moves
        let king_targets = Bitboard::king_attacks(self.king_square(self.current_player)) & !self.pieces_py_player(self.current_player);
        for to in king_targets {
            moves.push(Move::new(self.king_square(self.current_player), to));
        }

        //castling
        if !in_check {
            let mut castling_blockers = [Bitboard::new(); 4];
            castling_blockers[CastlingType::WhiteCastleKingside as usize] = Bitboard::from_squares(&[61, 62]);
            castling_blockers[CastlingType::WhiteCastleQueenside as usize] = Bitboard::from_squares(&[57,58,59]);
            castling_blockers[CastlingType::BlackCastleKingside as usize] = Bitboard::from_squares(&[5,6]);
            castling_blockers[CastlingType::BlackCastleQueenside as usize] = Bitboard::from_squares(&[1,2,3]);

            let castling_types = if self.current_player == White {
                [CastlingType::WhiteCastleKingside, CastlingType::WhiteCastleQueenside]   
            } else {
                [CastlingType::BlackCastleKingside, CastlingType::BlackCastleQueenside]
            };

            for castling_type in castling_types.into_iter() {
                if(self.stack.last().unwrap().castling_rights & (1 << castling_type as u8)) != 0
                        && (castling_blockers[castling_type as usize] & self.occupied()).is_empty() {
                    
                    moves.push(Move::new_castling(castling_type));
                }
            }
        }

        //en passant
        if let Some(file) = self.stack.last().unwrap().en_passant_file {
            let (from_rank, to_rank) = if self.current_player == White {
                (3, 2)
            } else {
                (4, 5)
            };

            let en_passant_square = Bitboard::from_square(from_rank*8+file);

            let from_squares = self.pieces(Pawn, self.current_player) & (en_passant_square.shift(Direction::Left) | en_passant_square.shift(Direction::Right));
        
            for from in from_squares {
                moves.push(Move::new_en_passant(from, to_rank*8+file));
            }
        }

        moves
    }

    fn pinned_pieces(&self) -> Bitboard {
        let king_square = self.king_square(self.current_player);
        let rook_attack_king_blockers = Bitboard::rook_attacks(king_square, self.occupied()) & self.pieces_py_player(self.current_player);
        let bishop_attack_king_blockers = Bitboard::bishop_attacks(king_square, self.occupied()) & self.pieces_py_player(self.current_player);

        let enemy_rooks_and_queens = self.pieces(Rook, !self.current_player) | self.pieces(Queen, !self.current_player);
        let enemy_bishops_and_queens = self.pieces(Bishop, !self.current_player) | self.pieces(Queen, !self.current_player);

        let rook_attack_pinners = Bitboard::rook_attacks(king_square, self.occupied() & !rook_attack_king_blockers) & enemy_rooks_and_queens;
        let bishop_attack_pinners = Bitboard::bishop_attacks(king_square, self.occupied() & !bishop_attack_king_blockers) & enemy_bishops_and_queens;
    
        let pinners = rook_attack_pinners | bishop_attack_pinners;
        let blockers = rook_attack_king_blockers | bishop_attack_king_blockers;

        let mut pinned = Bitboard::new();
        for p in pinners {
            pinned |= blockers & Bitboard::in_between(king_square, p);
        }

        pinned
    }

    fn is_legal(&mut self, m: Move) -> bool {
        if m.is_en_passant() {
            let tmp1 = self.piece_bb[Pawn as usize];
            let tmp2 = self.color_bb[0];
            let tmp3 = self.color_bb[1];

            let captured_pawn_square = if self.current_player == White {
                m.to() + 8
            } else {
                m.to() - 8
            };

            self.piece_bb[Pawn as usize] &= !Bitboard::from_square(m.from());
            self.piece_bb[Pawn as usize] &= !Bitboard::from_square(captured_pawn_square);
            self.piece_bb[Pawn as usize] |= Bitboard::from_square(m.to());

            self.color_bb[self.current_player as usize] &= !Bitboard::from_square(m.from());
            self.color_bb[self.current_player as usize] |= Bitboard::from_square(m.to());
            self.color_bb[!self.current_player as usize] &= !Bitboard::from_square(captured_pawn_square);

            let res = !self.is_attacked(self.king_square(self.current_player), self.current_player);

            self.piece_bb[Pawn as usize] = tmp1;
            self.color_bb[0] = tmp2;
            self.color_bb[1] = tmp3;

            return res;
        
        } else if let Some(c) = m.castling_type() {
            let castling_squares = [[61, 62], [59, 58], [6, 5], [2,3]];
            return !self.is_attacked(castling_squares[c as usize][0], self.current_player) 
                    && !self.is_attacked(castling_squares[c as usize][1], self.current_player);
        
        } else if self.stack.last().unwrap().pinned.contains(m.from()) {

            return Bitboard::is_aligned(self.king_square(self.current_player), m.from(), m.to());

        } else if self.pieces_by_piece_type(King).contains(m.from()) {

            return !self.is_attacked(m.to(), self.current_player);

        }

        true
    }

    pub fn is_capture(&self, m: Move) -> bool {
        m.is_en_passant() || self.piece_on(m.to()) != NoPiece
    }

    pub fn is_attacked(&self, square: u8, player: Color) -> bool {
        if !(Bitboard::knight_attacks(square) & self.pieces(Knight, !player)).is_empty(){
            return true;
        }

        let pawn_attacks_squares = if player == White {
            Bitboard::from_square(square).shift(Direction::UpLeft) | Bitboard::from_square(square).shift(Direction::UpRight)
        } else {
            Bitboard::from_square(square).shift(Direction::DownLeft) | Bitboard::from_square(square).shift(Direction::DownRight)
        };

        if !(pawn_attacks_squares & self.pieces(Pawn, !player)).is_empty() {
            return true;
        }

        let enemy_bishops_and_queens = self.pieces(Queen, !player) | self.pieces(Bishop, !player);
        if !(Bitboard::bishop_attacks(square, self.occupied() & !self.pieces(King, player)) & enemy_bishops_and_queens).is_empty() {
            return true;
        }

        let enemey_rooks_and_queens = self.pieces(Queen, !player) | self.pieces(Rook, !player);
        if !(Bitboard::rook_attacks(square, self.occupied() & !self.pieces(King, player)) & enemey_rooks_and_queens).is_empty() {
            return true;
        }

        if Bitboard::king_attacks(self.king_square(!player)).contains(square) {
            return true;
        }

        false
    }

    fn check_blocking_squares(&self, player: Color) -> Bitboard {
        let mut blocking_squares = !Bitboard::new();

        let king_square = self.king_square(self.current_player);

        // attacks from nonsliding pieces (pawns and knights)

        let knight_attacks = Bitboard::knight_attacks(king_square) & self.pieces(Knight, !self.current_player);
        
        let pawn_attack_squares = if player == White {
            Bitboard::from_square(king_square).shift(Direction::UpLeft) | Bitboard::from_square(king_square).shift(Direction::UpRight)
        } else {
            Bitboard::from_square(king_square).shift(Direction::DownLeft) | Bitboard::from_square(king_square).shift(Direction::DownRight)
        };
        let pawn_attacks = pawn_attack_squares & self.pieces(Pawn, !self.current_player);

        let non_sliding_attacks = pawn_attacks | knight_attacks;
        
        //there can only be at most one non sliding piece attacking the king
        if let Some(square) = non_sliding_attacks.into_iter().next() {
            blocking_squares &= Bitboard::from_square(square);
        }

        //diagonal sliding attacks
        let enemy_bishops_and_queens = self.pieces(Bishop, !self.current_player) | self.pieces(Queen, !self.current_player);
        let diag_attacks = Bitboard::bishop_attacks(king_square, self.occupied());
        let diag_attackers = diag_attacks & enemy_bishops_and_queens;
        
        for attacker in diag_attackers {
            blocking_squares &= diag_attacks & Bitboard::in_between(king_square, attacker);
        }

        //rank and file sliding attacks
        let enemy_rooks_and_queens = self.pieces(Rook, !self.current_player) | self.pieces(Queen, !self.current_player);
        let rook_attacks = Bitboard::rook_attacks(king_square, self.occupied());
        let rook_attackers = rook_attacks & enemy_rooks_and_queens;

        for attacker in rook_attackers {
            blocking_squares &= rook_attacks & Bitboard::in_between(king_square, attacker);
        }


        blocking_squares
    }

    fn generate_pawn_moves(moves: &mut Vec<Move>, targets: Bitboard, offset: i8) {
        for to in targets & !(Bitboard::rank(0) | Bitboard::rank(7)) {
            moves.push(Move::new((to as i8 - offset) as u8, to));
        }

        for to in targets & (Bitboard::rank(0) | Bitboard::rank(7)) {
            /* the order in which promotion moves are generated is important. If the engine thinks
               two promotions are equally good, it should choose the heavier piece.
            */
            for p in [Queen, Rook, Bishop, Knight] {
                moves.push(Move::new_promotion((to as i8 - offset) as u8, to, p));
            }
        }
    }
    
    /*
     * draw detection
     */

    pub fn has_repetition(&self, ply: u16) -> bool {
        
        let mut repetitions = 0;

        for (i, hash) in self.stack.iter().enumerate().map(|(i, f)| (i, f.hash)).step_by(2).skip(1) {
            if i > self.stack.last().unwrap().half_move_clock as usize{
                break;
            }

            if hash == self.stack.last().unwrap().hash {
                if i <= ply as usize{
                    return true;
                }

                repetitions += 1;
                if repetitions >= 2 {
                    return true;
                }
            }
        }

        false
    }

     
    pub fn insufficient_material(&self) -> bool {
        let major_pieces = (self.pieces_by_piece_type(Rook) | self.pieces_by_piece_type(Queen)).count_squares();
        let bishops = self.pieces_by_piece_type(Bishop).count_squares();
        let knights = self.pieces_by_piece_type(Knight).count_squares();
        let pawns = self.pieces_by_piece_type(Pawn).count_squares();

        if major_pieces == 0 && pawns == 0 {
            //king vs king + knight and king vs king + bishop
            if bishops + knights <= 1 {
                return true;
            }

            if bishops == 2 && self.pieces(Bishop, White).count_squares() == 1 //each player has one bishop
                    && (Self::LIGHT_SQUARES & self.pieces_by_piece_type(Bishop)).count_squares() != 1 { //the bishops are on same colored squares
                return true;
            }
        }

        false
    }


    /*
     * zobrist hash
     */

    const fn random_number(n: u32) -> u64 {
        let mut state = 0;
        let mut i = 0;
        while i < n+2 {
            state = u64::overflowing_add(u64::overflowing_mul(2862933555777941757, state).0, 3037000493).0;
            i+= 1;
        }

        state
    }

    const fn random_array<const N: usize>(offset: u32) -> [u64; N] {
        let mut arr = [0; N];

        let mut i = 0;
        while i < N {
            arr[i] = Self::random_number(offset + i as u32);
            i += 1;
        }

        arr
    }

    const fn random_array_2d<const N: usize, const M: usize>(offset: u32) -> [[u64; M]; N] {
        let mut arr = [[0; M]; N];

        let mut i = 0; 
        while i < N {
            arr[i] = Self::random_array(offset + (i*M) as u32);
            i += 1;
        }

        arr
    }

    const fn random_array_3d<const N: usize, const M: usize, const L: usize>(offset: u32) -> [[[u64; L]; M]; N] {
        let mut arr = [[[0; L]; M]; N];

        let mut i = 0; 
        while i < N {
            arr[i] = Self::random_array_2d(offset + (i*M*L) as u32);
            i += 1;
        }

        arr
    }

    const ZOBRIST_PIECES: [[[u64; 64]; 6]; 2] = Self::random_array_3d(0);
    const ZOBRIST_CASTLING_RIGHT: [u64; 16] = Self::random_array(64*6*2);
    const ZOBRIST_EN_PASSANT: [u64; 8] = Self::random_array(64*6*2+16);
    const ZOBRIST_CURRENT_PLAYER: [u64; 2] = Self::random_array(64*6*2+16+8);

    fn calculate_hash(&self) -> u64 {

        let mut hash = 0;
        
        for p in [Black, White] {
            for s in self.pieces_py_player(p) {
                hash ^= Self::ZOBRIST_PIECES[p as usize][self.piece_on(s) as usize][s as usize];
            }
        }

        hash ^= Self::ZOBRIST_CASTLING_RIGHT[self.stack.last().unwrap().castling_rights as usize];
        
        if let Some(e) = self.stack.last().unwrap().en_passant_file {
            hash ^= Self::ZOBRIST_EN_PASSANT[e as usize];
        }

        hash ^= Self::ZOBRIST_CURRENT_PLAYER[self.current_player() as usize];

        hash
    }


    /*
     * perft function
     */

    pub fn perft(&mut self, depth: u32) -> u64{
        if depth == 0 {
            1
        } else if depth == 1 {
            self.legal_moves().len() as u64
        } else {
            let mut result = 0;
            for m in self.legal_moves().into_iter() {
                self.make_move(m);
                result += self.perft(depth-1);
                self.unmake_move(m);
            }
    
            result
        }
    }
}


/*
 * perft testing
 */

const PERFT_POSITIONS: [&str; 7] = ["rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                                    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                                    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
                                    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
                                    "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
                                    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
                                    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"];


const PERFT_RESULTS: &[&[u64]] = &[&[20, 400, 8_902, 197_281, 4_865_609, 119_060_324, 3_195_901_860],
                                   &[48, 2_039, 97_862, 4_085_603, 193_690_690, 8_031_647_685],
                                   &[14, 191, 2_812, 43_238, 674_624, 11_030_083, 178_633_661, 3_009_794_393],
                                   &[6, 264, 9_467, 422_333, 15_833_292, 706_045_033],
                                   &[6, 264, 9_467, 422_333, 15_833_292, 706_045_033],
                                   &[44, 1_486, 62_379, 2_103_487, 89_941_194],
                                   &[46, 2_079, 89_890, 3_894_594, 164_075_551, 6_923_051_137]];

#[test]
fn perft_small() {
    for (i, &fen) in PERFT_POSITIONS.iter().enumerate() {
        for depth in 1..5 {
            assert_eq!(PERFT_RESULTS[i][(depth-1) as usize], Position::from_fen_string(fen).unwrap().perft(depth));
        }
    }
}

#[test]
#[ignore]
fn perft_full() {
    for (i, &fen) in PERFT_POSITIONS.iter().enumerate() {
        for depth in 1..PERFT_RESULTS[i].len()+1 {
            assert_eq!(PERFT_RESULTS[i][depth-1], Position::from_fen_string(fen).unwrap().perft(depth as u32));
        }
    }
}