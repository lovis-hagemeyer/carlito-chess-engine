mod bitboard;
mod position;

use crate::bitboard::*;

fn main() {
    let b = Bitboard::from_square(63);
    b.print();
}
