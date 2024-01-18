use std::fmt;

#[derive(Copy, Clone, PartialEq)]
pub struct Bitboard {
    b: u64
}

#[derive(Copy, Clone)]
pub struct BitboardIterator {
    b: u64
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft
}

use Direction::*;

impl Bitboard {
    pub const fn new() -> Bitboard {
        Bitboard {
            b: 0
        }
    }

    pub const fn from_coords(file: u8, rank: u8) -> Bitboard {
        Bitboard { 
            b: 1 << (8*rank+file) 
        }
    }

    pub const fn from_square(square: u8) -> Bitboard {
        Bitboard {
            b: 1 << square
        }
    }

    pub fn from_squares(squares: &[u8]) -> Bitboard {
        squares.iter().fold(Bitboard::new(), |bitboard, square| { bitboard | Bitboard::from_square(*square) })
    }

    pub const fn from_u64(bb: u64) -> Bitboard {
        Bitboard {
            b: bb
        }
    }

    pub const fn contains(&self, square: u8) -> bool {
        ((1 << square) & self.b) != 0
    }

    pub const fn shift(&self, d: Direction) -> Bitboard {
        Bitboard::from_u64( match d {
            Direction::Up => self.b >> 8,
            Direction::UpRight => (self.b >> 7) & !0x0101010101010101,
            Direction::Right => (self.b << 1) & !0x0101010101010101,
            Direction::DownRight => (self.b << 9) & !0x0101010101010101,
            Direction::Down => self.b << 8,
            Direction::DownLeft => (self.b << 7) & !0x8080808080808080,
            Direction::Left => (self.b >> 1) & !0x8080808080808080,
            Direction::UpLeft => (self.b >> 9) & !0x8080808080808080
        })
    }

    pub const fn is_empty(&self) -> bool {
        self.b == 0
    }

    pub const fn count_squares(&self) -> u32 {
        self.b.count_ones()
    }

    pub const fn file(f: u8) -> Bitboard {
        Bitboard {
            b: 0x0101010101010101 << f
        }
    }

    pub const fn rank(r: u8) -> Bitboard {
        Bitboard {
            b: 0xff << (r*8)
        }
    }

    pub fn line(s1: u8, s2: u8) -> Bitboard {
        SQUARE_SQUARE_LINE_TABLE[s1 as usize][s2 as usize]
    }

    pub fn in_between(s1: u8, s2: u8) -> Bitboard {
        BETWEEN_TABLE[s1 as usize][s2 as usize]
    }

    pub fn is_aligned(s1: u8, s2: u8, s3: u8) -> bool {
        !(Self::line(s1, s2) & Bitboard::from_square(s3)).is_empty()
    }

    pub fn bishop_attacks(square: u8, occupied: Bitboard) -> Bitboard {
       Bitboard::diagonal_attacks(square, occupied) | Bitboard::antidiagonal_attacks(square, occupied)
    }

    pub fn rook_attacks(square: u8, occupied: Bitboard) -> Bitboard {
        Bitboard::rank_attacks(square, occupied) | Bitboard::file_attacks(square, occupied)
    }

    pub fn knight_attacks(square: u8) -> Bitboard {
        KNIGHT_ATTACKS_TABLE[square as usize]
    }
    
    pub fn king_attacks(square: u8) -> Bitboard {
        KING_ATTACKS_TABLE[square as usize]
    }

    fn diagonal_attacks(square: u8, occupied: Bitboard) -> Bitboard {
        let index = u64::overflowing_mul((DIAGONALS_TABLE[square as usize][0] & occupied).b, Bitboard::file(1).b).0 >> 58;
        KINDERGARTEN_ATTACKS_TABLE[(square % 8) as usize][index as usize] & DIAGONALS_TABLE[square as usize][0]
    }

    fn antidiagonal_attacks(square: u8, occupied: Bitboard) -> Bitboard {
        let index = u64::overflowing_mul((DIAGONALS_TABLE[square as usize][1] & occupied).b, Bitboard::file(1).b).0 >> 58;
        KINDERGARTEN_ATTACKS_TABLE[(square % 8) as usize][index as usize] & DIAGONALS_TABLE[square as usize][1]
    }

    fn rank_attacks(square: u8, occupied: Bitboard) -> Bitboard {
        let index = ((Bitboard::rank(square / 8) & occupied).b >> ((square / 8)*8+1)) & 0x3f;
        KINDERGARTEN_ATTACKS_TABLE[(square % 8) as usize][index as usize] & Bitboard::rank(square / 8)
    }

    fn file_attacks(square: u8, occupied: Bitboard) -> Bitboard {
        let c2h7_diagonal: u64 = 0x0080402010080400;

        let a_file_occ = Bitboard::file(0).b & (occupied.b >> (square%8));
        let index = u64::overflowing_mul(c2h7_diagonal, a_file_occ).0 >> 58;
        
        Bitboard {
            b: KINDERGARTEN_FILE_ATTACKS_TABLE[(square/8) as usize][index as usize].b << (square%8) 
        }
    }
}

impl IntoIterator for Bitboard {
    type Item = u8;
    type IntoIter = BitboardIterator;

    fn into_iter(self) -> Self::IntoIter {
        BitboardIterator {
            b: self.b
        }
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        for i in 0..8 {
            writeln!(f)?;
            for j in 0..8 {
                if self.contains(8*i+j) {
                    write!(f, "1 ")?;
                } else {
                    write!(f, "0 ")?;
                }
            }
        }
        Ok(())
    }
}


impl Direction {
    pub const fn opposite(&self) -> Direction {
        match *self {
            Up => Down,
            UpRight => DownLeft,
            Right => Left,
            DownRight => UpLeft,
            Down => Up,
            DownLeft => UpRight,
            Left => Right,
            UpLeft => DownRight
        }
    }

    pub const fn offset(&self) -> i8 {
        match *self {
            Up => -8,
            UpRight => -7,
            Right => 1,
            DownRight => 9,
            Down => 8,
            DownLeft => 7,
            Left => -1,
            UpLeft => -9
        }
    }
}

impl Iterator for BitboardIterator {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.b == 0 {
            return None
        }        
        let square = self.b.trailing_zeros() as u8;
        self.b &= !(1 << square);
        Some(square)
    }
}

/*
 * bitwise logic operations for Bitboards
 */

impl std::ops::BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard {b: self.b | rhs.b }
    }
}

impl std::ops::BitOrAssign for Bitboard {
        
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard {b: self.b & rhs.b }
    }
}

impl std::ops::BitAndAssign for Bitboard {
        
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Bitboard {b: !self.b }
    }
}


/*
 * look up tables 
 */

static KNIGHT_ATTACKS_TABLE: [Bitboard; 64] = get_knight_attacks_table(); //512B
static KING_ATTACKS_TABLE: [Bitboard; 64] = get_king_attacks_table(); //512B 
static SQUARE_SQUARE_LINE_TABLE: [[Bitboard; 64]; 64] = get_square_square_line_table(); //32KByte
static BETWEEN_TABLE: [[Bitboard; 64]; 64] = get_between_table(); //32KByte
static DIAGONALS_TABLE: [[Bitboard; 2]; 64] = get_diagonals_table(); //1KByte
static KINDERGARTEN_ATTACKS_TABLE: [[Bitboard; 64]; 8] = get_kindergarten_attacks_table(); //4KByte
static KINDERGARTEN_FILE_ATTACKS_TABLE: [[Bitboard; 64]; 8] = get_kindergarten_file_attacks_table(); //4KByte

const fn get_knight_attacks_table() -> [Bitboard; 64] {
    let mut table = [Bitboard{ b: 0 }; 64];

    let mut i = 0;
    while i < 64 {

        let offsets = [(-1,-2), (-2,-1), (1, -2), (-2, 1), (-1, 2), (2, -1), (1,2), (2,1)];

        let knight_rank = i/8;
        let knight_file = i%8;

        let mut result: u64 = 0;

        let mut j = 0;
        while j < offsets.len() {
            
            let rank = knight_rank + offsets[j].0;
            let file = knight_file + offsets[j].1;

            if 0 <= rank && rank <= 7 && 0 <= file && file <= 7 {
                result |= 1 << (file + rank*8);
            }

            j += 1;
        }

        table[i as usize] = Bitboard {
            b: result
        };

        i += 1;
    }

    table
}

const fn get_king_attacks_table() -> [Bitboard; 64] {
    let mut table = [Bitboard::new(); 64];

    let mut square = 0;
    while square < 64 {
        let bb = Bitboard::from_square(square);

        let mut result: u64 = 0;
        
        let dirs = [Up, UpRight, Right, DownRight, Down, DownLeft, Left, UpLeft];
        let mut i = 0;
        while i < dirs.len() {
            result |= (bb.shift(dirs[i])).b;
            i += 1;
        }
        
        table[square as usize] = Bitboard::from_u64(result);
        
        square += 1;
    }

    table
}

const fn get_square_square_line_table() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard { b: 0 }; 64]; 64];

    let mut s1 = 0;
    while s1 < 64 {

        let mut i = 0;
        let directions = [Up, UpRight, Right, DownRight];
        while i < 4 {

            let mut line = Bitboard::from_square(s1);
            let mut b = Bitboard::from_square(s1).shift(directions[i as usize]);

            while !b.is_empty() {
                line = Bitboard::from_u64(line.b | b.b);
                b = b.shift(directions[i as usize]);
            }

            b = Bitboard::from_square(s1).shift(directions[i as usize].opposite());

            while !b.is_empty() {
                line = Bitboard::from_u64(line.b | b.b);
                b = b.shift(directions[i as usize].opposite());
            }

            let mut line_iter = line;

            while !line_iter.is_empty() {
                let s2 = line_iter.b.trailing_zeros();
                line_iter = Bitboard::from_u64(line_iter.b & !Bitboard::from_square(s2 as u8).b);

                table[s1 as usize][s2 as usize] = Bitboard::from_u64(table[s1 as usize][s2 as usize].b | line.b);
            }
            
            i += 1;
        }

        s1 += 1;
    }

    table
}

pub const fn get_diagonals_table() -> [[Bitboard; 2]; 64] {
    let mut table = [[Bitboard::new(); 2]; 64];

    let mut square = 0;
    while square < 64 {

        let mut diagonal = Bitboard::from_square(square);
        let mut anti_diagonal = Bitboard::from_square(square);
        let mut i = 0;
        while i < 7 {
            diagonal = Bitboard { b: diagonal.b | diagonal.shift(UpRight).b | diagonal.shift(DownLeft).b };
            anti_diagonal = Bitboard { b: anti_diagonal.b | anti_diagonal.shift(UpLeft).b | anti_diagonal.shift(DownRight).b };
            i += 1;
        }

        table[square as usize][0] = diagonal;
        table[square as usize][1] = anti_diagonal; 

        square += 1;
    }

    table
}

const fn get_kindergarten_attacks_table() -> [[Bitboard; 64]; 8] {
    let mut table = [[Bitboard::new(); 64]; 8];

    let mut index: u8 = 0;
    while index < 64 {

        let mut square = 0;
        while square < 8 {
            let occ:u8 = index << 1;
            let mut res: u64 = 0;

            //attack ray to the right
            let mut attacked_square: i32 = square-1;
            while attacked_square >= 0 {
                res |= 1 << attacked_square;
                if (1 << attacked_square) & occ != 0 {
                    break;
                }
                attacked_square -= 1;
            }

            //attack ray to the left
            attacked_square = square + 1;
            while attacked_square <= 7 {
                res |= 1 << attacked_square;
                if (1 << attacked_square) & occ != 0 {
                    break;
                }
                attacked_square += 1;
            }

            let mut j = 0;
            while j < 8 {
                res = res | (res << 8);
                j += 1;
            }

            table[square as usize][index as usize] = Bitboard::from_u64(res);

            square += 1;
        }

        index += 1;
    }

    table
}

const fn get_kindergarten_file_attacks_table() -> [[Bitboard; 64]; 8] {
    let rank_attack_table = get_kindergarten_attacks_table();
    let mut table = [[Bitboard::new(); 64]; 8];

    let mut index = 0;
    while index < 64 {

        let mut square = 0;
        while square < 8 {

            let rank_attack = rank_attack_table[7-square][index].b & Bitboard::rank(0).b;
            let a1h8_diagonal: u64 = 0x8040201008040201;

            let file_attack = (u64::overflowing_mul(rank_attack, a1h8_diagonal).0 >> 7) & Bitboard::file(0).b;

            table[square][index] = Bitboard::from_u64(file_attack);

            square += 1;
        }

        index += 1;
    }

    table
}

const fn get_between_table() -> [[Bitboard; 64]; 64] {
    let mut table = [[Bitboard::new(); 64]; 64];

    let mut square1 = 0;
    while square1 < 64 {

        let directions = [Up, UpRight, Right, DownRight, Down, DownLeft, Left, UpLeft];
        let mut i = 0;
        while i < 8 {

            let mut in_between = Bitboard::new();

            let mut square_bb = Bitboard::from_square(square1);

            while !square_bb.is_empty() {

                in_between = Bitboard::from_u64(square_bb.b | in_between.b);

                table[square1 as usize][square_bb.b.trailing_zeros() as usize] = in_between;

                square_bb = square_bb.shift(directions[i]);
            }

            i += 1;
        }

        square1 += 1
    }

    table
}