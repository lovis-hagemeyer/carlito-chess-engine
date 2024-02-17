use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::{bitboard::{Bitboard, Direction::*}, position::{Color::{self, *}, Position}};
use super::{score::Score, Piece::*};

pub struct Evaluator {
    params: EvalParams,
    pawn_attacks: [Bitboard; 2],
    king_safety: [i32; 2],
    king_ring: [Bitboard; 2],
    outpost_squares: [Bitboard; 2]
}

pub struct EvalParams {
    pub material: [i32; 5],
    bishop_pair: P,
    piece_square: [[P; 64]; 6],
    knight_mobility: [P; 9],
    bishop_mobility: [P; 14],
    rook_mobility: [P; 15],
    queen_mobility: [P; 28],
    stacked_pawns: P,
    isolated_pawn:  P,
    passed_pawn: [P; 6],
    king_attack_ray: [i32; 8], //TODO: check if seperate parameters for file and diagonal are better
    king_ring_attacker: [i32; 5],
    king_ring_defender: [i32; 5],
    bishop_outpost: P,
    knight_outpost: P,
    open_rook_file: P,
    half_open_rook_file: P,

}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct P(i32, i32);

impl Add for P {
    type Output = P;

    fn add(self, rhs: Self) -> Self::Output {
        P(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Sub for P {
    type Output = P;

    fn sub(self, rhs: Self) -> Self::Output {
        P(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl AddAssign for P {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl SubAssign for P {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl Neg for P {
    type Output = P;

    fn neg(self) -> Self::Output {
        P(0,0) - self
    }
}

impl Mul<i32> for P {
    type Output = P;

    fn mul(self, rhs: i32) -> Self::Output {
        P(self.0 * rhs, self.1 * rhs)
    }
}

impl Evaluator {

    const DEFAULT_PARAMS: EvalParams = EvalParams {
        material: [100, 290, 310, 500, 900],

        bishop_pair: P(10, 10),

        piece_square: [
            //pawn:
            [
                P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), 
                P( 23, 28), P( 59, 23), P(-14,  8), P( 20,-16), P( -7, -3), P( 51,-18), P(-41, 15), P(-86, 37),
                P(-16, 54), P( -3, 60), P( 16, 45), P( 21, 27), P( 55, 16), P( 46, 13), P( 15, 42), P(-30, 44), 
                P(-14, 12), P( 13, 14), P(  6,  3), P( 21, -5), P( 23,-12), P( 12, -6), P( 17,  7), P(-23,  7), 
                P(-27, 13), P( -2,  9), P( -5, -3), P( 12, -7), P( 17, -7), P(  6, -8), P( 10,  3), P(-25, -1), 
                P(-26,  4), P( -4,  7), P( -4, -6), P(-10,  1), P(  3,  0), P(  3, -5), P( 33, -1), P(-12, -8), 
                P(-35, 13), P( -1,  8), P(-20,  8), P(-23, 10), P(-15, 13), P( 24,  0), P( 38,  2), P(-22, -7), 
                P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), P(  0,  0), 
            ],

            //knight:
            [
                P(-167,-58), P(-89,-38), P(-34,-13), P(-49,-28), P( 61,-31), P(-97,-27), P(-15,-63), P(-107,-99), 
                P(-73,-25), P(-41, -8), P( 72,-25), P( 36, -2), P( 23, -9), P( 62,-25), P(  7,-24), P(-17,-52), 
                P(-47,-24), P( 60,-20), P( 37, 10), P( 65,  9), P( 84, -1), P(129, -9), P( 73,-19), P( 44,-41), 
                P( -9,-17), P( 17,  3), P( 19, 22), P( 53, 22), P( 37, 22), P( 69, 11), P( 18,  8), P( 22,-18), 
                P(-13,-18), P(  4, -6), P( 16, 16), P( 13, 25), P( 28, 16), P( 19, 17), P( 21,  4), P( -8,-18), 
                P(-23,-23), P( -9, -3), P( 12, -1), P( 10, 15), P( 19, 10), P( 17, -3), P( 25,-20), P(-16,-22), 
                P(-29,-42), P(-53,-20), P(-12,-10), P( -3, -5), P( -1, -2), P( 18,-20), P(-14,-23), P(-19,-44), 
                P(-105,-29), P(-21,-51), P(-58,-23), P(-33,-15), P(-17,-22), P(-28,-18), P(-19,-50), P(-23,-64),
            ],

            //bishop:
            [
                P(-29,-14), P(  4,-21), P(-82,-11), P(-37, -8), P(-25, -7), P(-42, -9), P(  7,-17), P( -8,-24), 
                P(-26, -8), P( 16, -4), P(-18,  7), P(-13,-12), P( 30, -3), P( 59,-13), P( 18, -4), P(-47,-14), 
                P(-16,  2), P( 37, -8), P( 43,  0), P( 40, -1), P( 35, -2), P( 50,  6), P( 37,  0), P( -2,  4), 
                P( -4, -3), P(  5,  9), P( 19, 12), P( 50,  9), P( 37, 14), P( 37, 10), P(  7,  3), P( -2,  2), 
                P( -6, -6), P( 13,  3), P( 13, 13), P( 26, 19), P( 34,  7), P( 12, 10), P( 10, -3), P(  4, -9), 
                P(  0,-12), P( 15, -3), P( 15,  8), P( 15, 10), P( 14, 13), P( 27,  3), P( 18, -7), P( 10,-15), 
                P(  4,-14), P( 15,-18), P( 16, -7), P(  0, -1), P(  7,  4), P( 21, -9), P( 33,-15), P(  1,-27), 
                P(-33,-23), P( -3, -9), P(-14,-23), P(-21, -5), P(-13, -9), P(-12,-16), P(-39, -5), P(-21,-17), 
            ],

            //rook:
            [
                P( 32, 13), P( 42, 10), P( 32, 18), P( 51, 15), P( 63, 12), P(  9, 12), P( 31,  8), P( 43,  5), 
                P( 27, 11), P( 32, 13), P( 58, 13), P( 62, 11), P( 80, -3), P( 67,  3), P( 26,  8), P( 44,  3), 
                P( -5,  7), P( 19,  7), P( 26,  7), P( 36,  5), P( 17,  4), P( 45, -3), P( 61, -5), P( 16, -3), 
                P(-24,  4), P(-11,  3), P(  7, 13), P( 26,  1), P( 24,  2), P( 35,  1), P( -8, -1), P(-20,  2), 
                P(-36,  3), P(-26,  5), P(-12,  8), P( -1,  4), P(  9, -5), P( -7, -6), P(  6, -8), P(-23,-11), 
                P(-45, -4), P(-25,  0), P(-16, -5), P(-17, -1), P(  3, -7), P(  0,-12), P( -5, -8), P(-33,-16), 
                P(-44, -6), P(-16, -6), P(-20,  0), P( -9,  2), P( -1, -9), P( 11, -9), P( -6,-11), P(-71, -3), 
                P(-19, -9), P(-13,  2), P(  1,  3), P( 17, -1), P( 16, -5), P(  7,-13), P(-37,  4), P(-26,-20), 
            ],

            //queen:
            [
                P(-28, -9), P(  0, 22), P( 29, 22), P( 12, 27), P( 59, 27), P( 44, 19), P( 43, 10), P( 45, 20), 
                P(-24,-17), P(-39, 20), P( -5, 32), P(  1, 41), P(-16, 58), P( 57, 25), P( 28, 30), P( 54,  0), 
                P(-13,-20), P(-17,  6), P(  7,  9), P(  8, 49), P( 29, 47), P( 56, 35), P( 47, 19), P( 57,  9), 
                P(-27,  3), P(-27, 22), P(-16, 24), P(-16, 45), P( -1, 57), P( 17, 40), P( -2, 57), P(  1, 36), 
                P( -9,-18), P(-26, 28), P( -9, 19), P(-10, 47), P( -2, 31), P( -4, 34), P(  3, 39), P( -3, 23), 
                P(-14,-16), P(  2,-27), P(-11, 15), P( -2,  6), P( -5,  9), P(  2, 17), P( 14, 10), P(  5,  5), 
                P(-35,-22), P( -8,-23), P( 11,-30), P(  2,-16), P(  8,-16), P( 15,-23), P( -3,-36), P(  1,-32), 
                P( -1,-33), P(-18,-28), P( -9,-22), P( 10,-43), P(-15, -5), P(-25,-32), P(-31,-20), P(-50,-41), 
            ],

            //king:
            [
                P(-65,-74), P( 23,-35), P( 16,-18), P(-15,-18), P(-56,-11), P(-34, 15), P(  2,  4), P( 13,-17), 
                P( 29,-12), P( -1, 17), P(-20, 14), P( -7, 17), P( -8, 17), P( -4, 38), P(-38, 23), P(-29, 11), 
                P( -9, 10), P( 24, 17), P(  2, 23), P(-16, 15), P(-20, 20), P(  6, 45), P( 22, 44), P(-22, 13), 
                P(-17, -8), P(-20, 22), P(-12, 24), P(-27, 27), P(-30, 26), P(-25, 33), P(-14, 26), P(-36,  3), 
                P(-49,-18), P( -1, -4), P(-27, 21), P(-39, 24), P(-46, 27), P(-44, 23), P(-33,  9), P(-51,-11), 
                P(-14,-19), P(-14, -3), P(-22, 11), P(-46, 21), P(-44, 23), P(-30, 16), P(-15,  7), P(-27, -9), 
                P(  1,-27), P(  7,-11), P( -8,  4), P(-64, 13), P(-43, 14), P(-16,  4), P(  9, -5), P(  8,-17), 
                P(-15,-53), P( 36,-34), P( 12,-21), P(-54,-11), P(  8,-28), P(-28,-14), P( 24,-24), P( 14,-43), 
            ]
        ],
        knight_mobility: [ 
            P(-30,-30), P( -7, -7), P(  0,  0), P(  3,  3), P(  6,  6), P( 10, 10), P( 13, 13), P( 15, 15), P( 16, 16) 
        ],

        bishop_mobility: [
            P(-30,-30), P(-10,-10), P( -5, -5), P(  0,  0), P(  2,  2), P(  5,  5), P(  8,  8), P( 11, 11), P( 13, 13), P( 14, 14), 
            P( 15, 15), P( 16, 16), P( 16, 16), P( 16, 16)
        ],

        rook_mobility: [
            P(-30,-30), P(-10,-10), P( -5, -5), P(  0,  0), P(  2,  2), P(  5,  5), P(  8,  8), P( 11, 11), P( 13, 13), P( 14, 14), 
            P( 15, 15), P( 16, 16), P( 16, 16), P( 16, 16), P( 16, 16)
        ],

        queen_mobility: [
            P(-50,-50), P(-10,-10), P( -4, -4), P(  0,  0), P(  1,  1), P(  2,  2), P(  3,  3), P(  4,  4), P(  4,  4), P(  5,  5), 
            P(  5,  5), P(  6,  6), P(  6,  6), P(  6,  6), P(  7,  7), P(  7,  7), P(  7,  7), P(  7,  7), P(  8,  8), P(  8,  8), 
            P(  8,  8), P(  8,  8), P(  9,  9), P(  9,  9), P(  9,  9), P(  9,  9), P(  9,  9), P(  9,  9)
        ],

        stacked_pawns: P(-10, -15),

        isolated_pawn: P(-10, -15),

        passed_pawn: [P(10, 15), P(15, 20), P(20, 25), P(25, 35), P(50, 60), P(75 ,150)],

        king_attack_ray: [0, -3, -5, -7, -8, -9, -10, -10],

        king_ring_attacker: [-6, -5, -5, -8, -12],

        king_ring_defender: [0, 0, 0, 0, 0], //TODO: remove if automatic tuning does not show significant change here.
        
        bishop_outpost: P(5, 5),

        knight_outpost: P(7, 7),

        open_rook_file: P(15, 7),

        half_open_rook_file: P(10, 7),
    };


    
    pub fn new() -> Evaluator {
        Evaluator {
            params: Evaluator::DEFAULT_PARAMS,
            pawn_attacks: [Bitboard::new(); 2],
            king_safety: [0,0],
            king_ring: [Bitboard::new(); 2],
            outpost_squares: [Bitboard::new(); 2]
        }
    }
    
    pub fn evaluate(&mut self, pos: &mut Position) -> Score {
        
        //initialise fields for king danger evaluation.
        self.king_safety = [0,0];

        let king_square = pos.king_square(White);
        self.king_ring[White as usize] = Bitboard::king_attacks(king_square) | Bitboard::from_square(king_square);
        let king_square = pos.king_square(Black);
        self.king_ring[Black as usize] = Bitboard::king_attacks(king_square) | Bitboard::from_square(king_square);


        let mut eval = self.material(pos, White) - self.material(pos, Black);

        eval += self.eval_pawns(pos, White) - self.eval_pawns(pos, Black);
       
        for s in pos.pieces_by_player(White) {
            eval += self.eval_piece(pos, s);
        }

        for s in pos.pieces_by_player(Black) {
            eval -= self.eval_piece(pos, s);
        }

        eval.0 += self.king_safety[White as usize] - self.king_safety[Black as usize];


        let game_phase = self.game_phase(pos);

        let mut score = ((game_phase as i32 * eval.0) + ((24-game_phase.clamp(0, 24)) as i32 * eval.1)) / 24;

        if pos.current_player() == Black {
            score = -score;
        }
        
        Score::from_centi_pawns(score)
    }

    pub fn params(&self) -> &EvalParams {
        &self.params
    }

    fn material(&mut self, pos: &mut Position, player: Color) -> P {

        let material_value = pos.pieces(Pawn, player).count_squares() as i32 * self.params.material[Pawn as usize]
                            + pos.pieces(Knight, player).count_squares() as i32 * self.params.material[Knight as usize]
                            + pos.pieces(Bishop, player).count_squares() as i32 * self.params.material[Bishop as usize]
                            + pos.pieces(Rook, player).count_squares() as i32 * self.params.material[Rook as usize]
                            + pos.pieces(Queen, player).count_squares() as i32 * self.params.material[Queen as usize];

        let mut score = P(material_value, material_value);

        if pos.pieces(Bishop, player).count_squares() >= 2 {
            score += self.params.bishop_pair;
        }

        score
    }

    fn eval_pawns(&mut self, pos: &mut Position, player: Color) -> P {
        let mut score = P(0,0);

        let pawn_attacks = match player {
            White => pos.pieces(Pawn, player).shift(UpLeft) | pos.pieces(Pawn, player).shift(UpRight),
            Black => pos.pieces(Pawn, player).shift(DownLeft) | pos.pieces(Pawn, player).shift(DownRight)
        };

        self.pawn_attacks[player as usize] = pawn_attacks;

        //stacked pawns
        let mut in_front_of_pawns;

        match player {
            White => {
                in_front_of_pawns = pos.pieces(Pawn, player) >> 8;
                in_front_of_pawns |= in_front_of_pawns >> 8;
                in_front_of_pawns |= in_front_of_pawns >> 16;
                in_front_of_pawns |= in_front_of_pawns >> 32;
            },
            Black => {
                in_front_of_pawns = pos.pieces(Pawn, player) << 8;
                in_front_of_pawns |= in_front_of_pawns << 8;
                in_front_of_pawns |= in_front_of_pawns << 16;
                in_front_of_pawns |= in_front_of_pawns << 32;
            }
        }

        let stacked_pawns = in_front_of_pawns & pos.pieces(Pawn, player);

        score += self.params.stacked_pawns * (stacked_pawns.count_squares() as i32);

        //isolated pawns
        let pawn_files = match player {
            Black => in_front_of_pawns.b >> 56,
            White => in_front_of_pawns.b & 0xff
        };

        let mut stacked_pawn_files = stacked_pawns.b;
        stacked_pawn_files |= stacked_pawn_files >> 8;
        stacked_pawn_files |= stacked_pawn_files >> 16;
        stacked_pawn_files |= stacked_pawn_files >> 32;
        stacked_pawn_files &= 0xff;

        let isolated_pawn_files = pawn_files & !(pawn_files << 1) & !(pawn_files >> 1) & !(stacked_pawn_files);

        score += self.params.isolated_pawn * (isolated_pawn_files.count_ones() as i32);

        //enemy passed pawns
        let stoppable = match player {
            Black => in_front_of_pawns | in_front_of_pawns.shift(DownLeft) | in_front_of_pawns.shift(DownRight),
            White => in_front_of_pawns | in_front_of_pawns.shift(UpLeft) | in_front_of_pawns.shift(UpRight)
        };
        
        let enemy_passed_pawns = pos.pieces(Pawn, !player) & !stoppable;

        for s in enemy_passed_pawns {
            let rank = match player {
                Black => 7-s/8,
                White => s/8
            };
            score -= self.params.passed_pawn[(rank - 1) as usize];
        }

        //king ring attacks
        let king_ring_attack_squares = match player {
            Black => self.king_ring[White as usize].shift(UpLeft) | self.king_ring[White as usize].shift(UpRight),
            White => self.king_ring[Black as usize].shift(DownLeft) | self.king_ring[Black as usize].shift(DownRight),
        };

        self.king_safety[!player as usize] += self.params.king_ring_attacker[Pawn as usize] * (pos.pieces(Pawn, player) & king_ring_attack_squares).count_squares() as i32;

        //king ring defenders
        let king_ring_defender_squares = match player {
            Black => self.king_ring[Black as usize].shift(UpLeft) | self.king_ring[Black as usize].shift(UpRight),
            White => self.king_ring[White as usize].shift(DownLeft) | self.king_ring[White as usize].shift(DownRight),
        };

        self.king_safety[player as usize] += self.params.king_ring_defender[Pawn as usize] * (pos.pieces(Pawn, player) & king_ring_defender_squares).count_squares() as i32;

        //calculate outpost squares
        self.outpost_squares[!player as usize] = !(in_front_of_pawns.shift(Left) | in_front_of_pawns.shift(Right));

        score
    }

    fn eval_piece(&mut self, pos: &mut Position, square: u8) -> P {
        let mut score = P(0,0);
        let piece = pos.piece_on(square);
        let player = pos.square_color(square).unwrap();

        //piece square

        let mut piece_square_value_index = square;
        if player == Black {
            piece_square_value_index ^= 56;
        }

        score += self.params.piece_square[piece as usize][piece_square_value_index as usize];

        if piece == King {
            //count the number of squares between the king and the next own piece in diagonal, antidiagonal and vertical direction.

            let diagonal_rays = Bitboard::diagonal_attacks(square, pos.pieces_by_player(player)) & !pos.pieces_by_player(player);
            let antidiagonal_rays = Bitboard::antidiagonal_attacks(square, pos.pieces_by_player(player)) & !pos.pieces_by_player(player);
            let file_rays = Bitboard::file_attacks(square, pos.pieces_by_player(player)) & !pos.pieces_by_player(player);

            self.king_safety[player as usize] += self.params.king_attack_ray[diagonal_rays.count_squares() as usize];
            self.king_safety[player as usize] += self.params.king_attack_ray[antidiagonal_rays.count_squares() as usize];
            self.king_safety[player as usize] += self.params.king_attack_ray[file_rays.count_squares() as usize];

        } else if piece != Pawn {

            let attacks = match piece {
                Knight => Bitboard::knight_attacks(square),
                Bishop => Bitboard::bishop_attacks(square, pos.occupied()),
                Rook => Bitboard::rook_attacks(square, pos.occupied()),
                Queen => Bitboard::bishop_attacks(square, pos.occupied()) | Bitboard::rook_attacks(square, pos.occupied()),
                _ => panic!()
            };
            
            //mobility
            let moves = attacks & !pos.pieces_by_player(player) & !self.pawn_attacks[!player as usize];

            score += match piece {
                Knight => self.params.knight_mobility[moves.count_squares() as usize],
                Bishop => self.params.bishop_mobility[moves.count_squares() as usize],
                Rook => self.params.rook_mobility[moves.count_squares() as usize],
                Queen => self.params.queen_mobility[moves.count_squares() as usize],
                _ => panic!()
            };


            //king ring attacks
            if !(self.king_ring[!player as usize] & attacks).is_empty() {
                self.king_safety[!player as usize] += self.params.king_ring_attacker[piece as usize];
            }

            //king ring defenders
            if !(self.king_ring[player as usize] & attacks).is_empty() {
                self.king_safety[player as usize] += self.params.king_ring_defender[piece as usize];
            }

            //outposts
            if piece == Knight || piece == Bishop {
                if (self.outpost_squares[player as usize] & self.pawn_attacks[player as usize]).contains(square) {
                    score += if piece == Knight {
                        self.params.knight_outpost
                    } else {
                        self.params.bishop_outpost
                    }
                }
            }

            if piece == Rook {
                let file = Bitboard::file(square % 8);

                if (pos.pieces_by_type(Pawn) & file).is_empty() {
                    score += self.params.open_rook_file;
                } else if (pos.pieces(Pawn, player) & file).is_empty() {
                    score += self.params.half_open_rook_file;
                }
            }
        }

        score
    }

    fn game_phase(&mut self, pos: &mut Position) -> u32 {
          pos.pieces_by_type(Bishop).count_squares()
        + pos.pieces_by_type(Knight).count_squares()
        + pos.pieces_by_type(Rook).count_squares() * 2
        + pos.pieces_by_type(Queen).count_squares() * 4
    }
}