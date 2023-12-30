mod bitboard;
mod position;
mod chess_move;
mod uci;

use position::*;


fn tmp_perft(pos: &mut Position, depth: u8, root_node: bool) -> u64 {
    if depth == 0 {
        1
    } else {
        let mut result = 0;
        for m in pos.legal_moves().into_iter() {

            pos.make_move(m);
            let child_nodes = tmp_perft(pos, depth-1,false);
            if root_node {
                println!("{}: {child_nodes}", m);
            }
            result += child_nodes;
            pos.unmake_move(m);
        }

        result
    }
}


fn main() {
    let mut pos = Position::from_fen_string("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1").unwrap();

    //println!("{:#?}", pos);

    println!("{}", tmp_perft(&mut pos, 4, true));


    //uci::input_loop();
}
