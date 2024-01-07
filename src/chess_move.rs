use crate::position::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CastlingType {
    WhiteCastleKingside,
    WhiteCastleQueenside,
    BlackCastleKingside,
    BlackCastleQueenside
}

enum SpecialMoveType {
    None,
    Promotion,
    EnPassant,
    Castling
}

#[derive(Copy, Clone, PartialEq)]
pub struct Move {
    /**
     * move format:
     * 6 bits: from square
     * 6 bits: to square
     * 2 bits: extra information
     * 2 bits: special move type
     *     0: no special move: 'extra information' is 0.
     *     1: promotion: 'extra information' contains the piece to promote to (0: knight, 1: bishop, 2: rook, 4: queen)
     *     2: en passant: 'extra information' is 0
     *     3: castling: 'extra information' stores the castling type. 'from' and 'to' store which squares the king moves from and to
     */
    m: u16    
}

impl Move {
    pub fn new(from: u8, to: u8) -> Move {
        Move {
            m: from as u16 | (to as u16) << 6 | (SpecialMoveType::None as u16) << 14
        }
    } 

    pub fn new_en_passant(from: u8, to: u8) -> Move {
        Move {
            m: from as u16 | (to as u16) << 6 | (SpecialMoveType::EnPassant as u16) << 14
        }
    }

    pub fn new_castling(castling_type: CastlingType) -> Move {
        CASTLE_MOVES_TABLE[castling_type as usize]
    }

    pub fn new_promotion(from: u8, to: u8, promote_to: Piece) -> Move {
        Move {
            m: from as u16 | (to as u16) << 6 | (promote_to as u16 - 1) << 12 | (SpecialMoveType::Promotion as u16) << 14
        }
    }

    pub fn from_string(string: &str, pos: &mut Position) -> Result<Move, ()> {
        //check for non ascii chararcters right away, so we can safely index into the string
        if !string.is_ascii() {
            return Err(());
        }

        let from = Position::parse_square(string.get(0..2).ok_or(())?)?;
        let to = Position::parse_square(string.get(2..4).ok_or(())?)?;
        let promote_to = string.chars().nth(5).map(|c| match c {
            'q' => Ok(Piece::Queen),
            'r' => Ok(Piece::Rook),
            'b' => Ok(Piece::Bishop),
            'n' => Ok(Piece::Knight),
            _ => Err(())
        }).transpose()?;

        for m in pos.legal_moves() {
            if m.from() == from && m.to() == to && m.promote_to() == promote_to {
                return Ok(m);
            }
        }

        Err(())
    }

    pub fn from(&self) -> u8 {
        (self.m & 0x3f) as u8
    }

    pub fn to(&self) -> u8 {
        ((self.m >> 6) & 0x3f) as u8
    }

    pub fn is_en_passant(&self) -> bool {
        (self.m >> 14) == SpecialMoveType::EnPassant as u16
    }

    pub fn promote_to(&self) -> Option<Piece> {
        if (self.m >> 14) != SpecialMoveType::Promotion as u16 {
            None
        } else {
            Some(
                match (self.m >> 12) & 0x3 {
                    0 => Piece::Knight,
                    1 => Piece::Bishop,
                    2 => Piece::Rook,
                    3 => Piece::Queen,
                    _ => panic!("this can't happen because we masked the value with 0x3")
                }
            ) 
        }
    }

    pub fn castling_type(&self) -> Option<CastlingType> {
        if self.m >> 14 != SpecialMoveType::Castling as u16 {
            None
        } else {
            Some(
                match (self.m >> 12) & 0x3 {
                    0 => CastlingType::WhiteCastleKingside,
                    1 => CastlingType::WhiteCastleQueenside,
                    2 => CastlingType::BlackCastleKingside,
                    3 => CastlingType::BlackCastleQueenside,
                    _ => panic!("this can't happen because we masked the value with 0x3")
                }
            )
        }
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        write!(f, "from: {:?}, to: {:?}, is en passant?: {:?}, castling type: {:?}, promote_to: {:?}", self.from(), self.to(), self.is_en_passant(), self.castling_type(), self.promote_to())?;
        Ok(())
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        write!(f, "{}", Position::square_to_string(self.from()))?;
        write!(f, "{}", Position::square_to_string(self.to()))?;
        if let Some(p) = self.promote_to() {
            write!(f, "{}", match p {
                Piece::Bishop => "b",
                Piece::Knight => "n",
                Piece::Queen => "q",
                Piece::Rook => "r",
                _ => panic!("wrong promotion piece (this shouldn't be possible)")
            })?;
        }
        Ok(())
    }
}

const CASTLE_MOVES_TABLE: [Move; 4] = castling_move_table();

const fn castling_move_table() -> [Move; 4] {
    let mut m = [Move { m: 0 }; 4];

    let mut i = 0;

    let king_from_squares = [60, 60, 4, 4];
    let king_to_squares = [62, 58, 6, 2];

    while i < 4 {
        m[i] = Move {
            m: king_from_squares[i] | king_to_squares[i] << 6 | (i as u16) << 12 | (SpecialMoveType::Castling as u16) << 14
        };
        i += 1;
    }

    m
}