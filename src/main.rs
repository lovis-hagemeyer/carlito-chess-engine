pub mod bitboard;
pub mod position;
pub mod chess_move;
pub mod uci;
pub mod engine;


fn main() {
    uci::input_loop()
}