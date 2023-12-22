use std::fmt;

#[derive(Copy, Clone)]
pub struct Bitboard {
    b: u64
}

#[derive(Copy, Clone)]
pub struct BitboardIterator {
    b: u64
}

impl Bitboard {
    pub fn from_coords(x: u8, y: u8) -> Bitboard {
        Bitboard { 
            b: 1 << (8*y+x) 
        }
    }

    pub fn from_square(square: u8) -> Bitboard {
        Bitboard {
            b: 1 << square
        }
    }

    pub fn from_u64(bb: u64) -> Bitboard {
        Bitboard {
            b: bb
        }
    }

    pub fn squares(&self) -> BitboardIterator {
        BitboardIterator {
            b: self.b
        }
    }

    pub fn contains(&self, square: u8) -> bool {
        ((1 << square) & self.b) != 0
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

    // rhs is the "right-hand side" of the expression `a | b`
    fn not(self) -> Self::Output {
        Bitboard {b: !self.b }
    }
}