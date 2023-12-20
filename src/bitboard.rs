
#[derive(Copy, Clone)]
pub struct Bitboard {
    b: u64
}

#[derive(Copy, Clone)]
pub struct BitboardIterator {
    b: u64
}

impl Bitboard {
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

    pub fn is_set(&self, square: u8) -> bool {
        ((1 << square) & self.b) != 0
    }

    pub fn print(&self) {
        for i in 0..8 {
            for j in 0..8 {
                if self.is_set(8*i+j) {
                    print!("1 ");
                } else {
                    print!("0 ");
                }
            }
            println!();
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