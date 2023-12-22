mod bitboard;
mod position;

use crate::position::*;

fn main() {
    let mut p = Position::from_fen_string("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    //let p = Bitboard::from_square(50);
    println!("{:#?}", p);
    p = Position::from_fen_string("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w KQ - 1234 101234123412341342000");
    println!("{:#?}", p);
    p = Position::from_fen_string("rnbqkbnr/pp1pp1pp/5p2/2p1P3/8/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 3");
    println!("{:#?}", p);
    
}
