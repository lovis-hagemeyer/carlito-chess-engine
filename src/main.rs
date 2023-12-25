mod bitboard;
mod position;
mod chess_move;
mod uci;

use chess_move::Move;
use position::*;
use bitboard::*;

fn main() {
    /*let mut p = Position::from_fen_string("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    //let p = Bitboard::from_square(50);
    println!("{:#?}", p);
    p = Position::from_fen_string("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w KQ - 1234 101234123412341342000");
    println!("{:#?}", p);
    p = Position::from_fen_string("rnbqkbnr/pp1ppppp/8/8/1Pp5/2PP4/P3PPPP/RNBQKBNR b KQkq b3 0 3");
    println!("{:#?}", p);*/

    //let mut p = Position::new();

    //println!("{:#?}", p);
    //p.make_move(Move::new(Position::parse_square("e2").unwrap(), Position::parse_square("e4").unwrap()));
    //println!("{:#?}", p);

    //println!("{:?}", Bitboard::bishop_attacks(18, Bitboard::from_u64(0x9)));
    //println!("{:?}", Bitboard::from_square(14));
    println!("{:?}", Bitboard::bishop_attacks(21, Bitboard::from_square(12) | Bitboard::from_square(14) | Bitboard::from_square(28)));

    //println!("{:?}", get_diagonals_table());

    //uci::input_loop();
}
