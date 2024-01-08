pub mod bitboard;
pub mod position;
pub mod chess_move;
pub mod uci;
pub mod engine;

use crate::uci::UciHandler;

fn main() {
    UciHandler::new().input_loop();
}