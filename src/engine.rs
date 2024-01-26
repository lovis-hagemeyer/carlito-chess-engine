use std::collections::btree_map::Entry;
use std::sync::{atomic, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};


mod timer;
mod eval;
mod score;
mod ttable;


use timer::Timer;
use score::Score;
use ttable::{TTable, EntryType};

use crate::{chess_move::*, position};
use crate::position::*;

use self::eval::Evaluator;
use self::ttable::TTableEntry;

pub struct Engine {
    thread_data: Option<Arc<ThreadData>>,
    worker_thread: Option<thread::JoinHandle<TTable>>,
    timer: Option<Timer>,
    ttable: Option<TTable>
}

pub struct ThreadData {
    stop: atomic::AtomicBool,
    ponder: atomic::AtomicBool,

    start_time: Instant,
    max_time: Mutex<Option<Duration>>,
    min_time: Mutex<Option<Duration>>,

    position: Position,
    options: EngineOptions
}

struct SearchData {
    pv: Vec<Move>,
    evaluator: Evaluator,
    nodes: u64,
    move_sorter: MoveSorter,
    ttable: TTable
}

struct MoveSorter {
    killer_moves: Vec<(Move, Move)>,
}

const DRAW_SCORE: Score = Score { s: 0 };

#[derive(Debug, Clone)]
pub struct EngineOptions {
    pub search_moves: Vec<Move>,
    pub ponder: bool,
    pub infinite: bool,
    pub wtime: Option<u32>,
    pub btime: Option<u32>,
    pub winc: Option<u32>,
    pub binc: Option<u32>,
    pub moves_to_go: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u32>,
    pub mate_in: Option<u32>,
    pub move_time: Option<u32>
}

impl Engine {
    pub fn new(mb_table_size: usize) -> Engine {
        Engine {
            thread_data: None,
            worker_thread: None,
            timer: None,
            ttable: Some(TTable::new(mb_table_size))
        }
    }

    pub fn start(&mut self, position: Position, options: EngineOptions) {
        self.stop();


        self.thread_data = Some(Arc::new(ThreadData {
            stop: atomic::AtomicBool::new(false),
            ponder: atomic::AtomicBool::new(options.ponder),

            start_time: Instant::now(),
            max_time: Mutex::new(None),
            min_time: Mutex::new(None),

            position,
            options
        }));


        self.timer = Timer::new(self.thread_data.clone().unwrap());
        

        let thread_data_ref = self.thread_data.clone().unwrap();
        let ttable = self.ttable.take().unwrap();

        self.worker_thread = Some(thread::spawn(move || {
            Engine::analyze(thread_data_ref, ttable)
        }));
    }

    pub fn stop(&mut self) {    
        match self.worker_thread.take() {
            None => (),
            Some(handle) => {
                self.thread_data.as_ref().unwrap().stop.store(true, atomic::Ordering::Release);
                let ttable = handle.join().expect("error when joining worker thread");
                if self.ttable.is_none() {
                    self.ttable = Some(ttable);
                }
            }
        }

        self.thread_data = None;
        self.timer = None;
    }

    pub fn ponderhit(&mut self) {
        if let Some(thread_data) = self.thread_data.as_ref() {
            thread_data.ponder.store(false, atomic::Ordering::Release);
            self.timer = Timer::new(thread_data.clone());
        }
    }

    pub fn set_table_size(&mut self, size_in_mb: usize) {
        if self.ttable.as_ref().map(|t| t.mb_size()).filter(|n| *n == size_in_mb).is_none() {
            self.ttable = Some(TTable::new(size_in_mb));
        }
    }

    fn analyze(thread_data: Arc<ThreadData>, ttable: TTable) -> TTable {       

        //TODO check for mate in start position

        let mut position = thread_data.position.clone();

        let mut depth: u16 = 1;

        let mut data = SearchData {
            pv: Vec::new(),
            evaluator: Evaluator::new(),
            nodes: 0,
            move_sorter: MoveSorter::new(),
            ttable
        };

        let mut pv: Vec<Move> = Vec::new();

        loop {
            data.pv = Vec::new();

            let score = match Engine::search(&mut position, depth, 0, Score::NEGATIVE_INFTY, Score::POSITIVE_INFTY, true, &mut data, &thread_data) {
                None => break,
                Some(s) => s
            };

            pv = data.pv;

            let search_time = Instant::now().duration_since(thread_data.start_time);
            let search_time_ms = search_time.as_millis() as u64;

            //print info about search depth

            print!("info depth {depth}");
            if let Some(s) = score.centi_pawns() {
                print!(" score cp {s}");
            } else {
                print!(" score mate {}", score.mate().unwrap());
            }

            print!(" nodes {}", data.nodes);
            print!(" time {}", search_time_ms);
            if search_time_ms > 50 {
                print!(" nps {}", data.nodes * 1000 / search_time_ms);
            }

            print!(" pv");
            for m in pv.iter() {
                print!(" {m}");
            }

            println!();

            //end search if we found a mate
            if score.mate().is_some() {
                break;
            }

            //end search if we have been searching longer than min_time
            if let Some(min_time) = *thread_data.min_time.lock().unwrap() {
                if search_time >= min_time {
                    break;
                }
            }

            if let Some(max_depth) = thread_data.options.depth {
                if max_depth == depth as u32{
                    break;
                }
            }

            depth += 1;
        }

        print!("bestmove {}", pv[0]);
        if pv.len() > 1 {
            print!(" ponder {}", pv[1]);
        }
        println!();

        data.ttable
    }

    fn search(position: &mut Position, depth: u16, ply: u16, mut alpha: Score, beta: Score, pv_node: bool, data: &mut SearchData, thread_data: &ThreadData) -> Option<Score> {
        
        if thread_data.stop.load(atomic::Ordering::Acquire) {
            return None;
        }

        data.nodes += 1;

        //TODO nodes option should be u64
        if (thread_data.options.nodes.unwrap_or(u32::MAX) as u64) < data.nodes {
            return None
        }

        if depth == 0 {
            return Self::qsearch(position, ply, alpha, beta, pv_node, data, thread_data);
        }

        let moves = position.legal_moves();
        
        if moves.is_empty() {
            if position.is_attacked(position.king_square(position.current_player()), position.current_player()) {
                return Some(Score::from_mate_distance(-(((ply+1)/2) as i16)));
            } else {
                return Some(DRAW_SCORE);
            }
        }

        if position.insufficient_material() || position.has_repetition(ply) || position.half_move_clock() >= 100 {
            return Some(DRAW_SCORE);
        }  

        let ttable_move;

        //transposition table look up
        if let Some(table_entry) = data.ttable.lookup(position.hash()) {
            if table_entry.depth == depth {
                match table_entry.entry_type {
                    ttable::EntryType::Exact => {
                        if ply == 0 {
                            data.pv.push(table_entry.best_move);
                        }
                        return Some(table_entry.score)
                    },
                    ttable::EntryType::Upper => {
                        if alpha >= table_entry.score {
                            return Some(table_entry.score);
                        }
                    },
                    ttable::EntryType::Lower => {
                        if table_entry.score >= beta {
                            return Some(table_entry.score);
                        }
                    }
                }
            }

            ttable_move = Some(table_entry.best_move);
        } else {
            ttable_move = None;
        }

        let mut best_move = moves[0];

        for (i, m) in data.move_sorter.sort(position, moves, ply, ttable_move).into_iter().enumerate() {
            position.make_move(m);

            let mut move_score;
            

            if pv_node && i != 0 {
                move_score = -Engine::search(position, depth - 1, ply + 1, - (Score { s: (alpha.s + 1) }), -alpha, false, data, thread_data)?;
                if move_score > alpha {
                    move_score = -Engine::search(position, depth - 1, ply + 1, -beta, -alpha, true,  data, thread_data)?;
                }
            } else {
                move_score = -Engine::search(position, depth - 1, ply + 1, -beta, -alpha, pv_node, data, thread_data)?;
            }


            position.unmake_move(m);

            if move_score > alpha {
                alpha = move_score;
                best_move = m;

                if alpha >= beta {

                    data.move_sorter.cut_off_move(m, ply);

                    data.ttable.insert(position.hash(), EntryType::Lower, move_score, best_move, depth);
                    
                    return Some(alpha);
                }
            }
        }

        if pv_node {
            data.ttable.insert(position.hash(), EntryType::Exact, alpha, best_move, depth);
        } else {
            data.ttable.insert(position.hash(), EntryType::Upper, alpha, Move::new(0,0), depth);
        }

        if ply == 0 {
            data.pv.push(best_move);
        }

        Some(alpha)
    }

    fn qsearch(position: &mut Position, ply: u16, mut alpha: Score, beta: Score, pv_node: bool, data: &mut SearchData, thread_data: &ThreadData) -> Option<Score> {
        if thread_data.stop.load(atomic::Ordering::Acquire) {
            return None;
        }

        data.nodes += 1;

        //TODO nodes option should be u64
        if (thread_data.options.nodes.unwrap_or(u32::MAX) as u64) < data.nodes {
            return None
        }

        let moves = position.legal_moves();
        
        if moves.is_empty() {
            if position.is_attacked(position.king_square(position.current_player()), position.current_player()) {
                return Some(Score::from_mate_distance(-(((ply+1)/2) as i16)));
            } else {
                return Some(DRAW_SCORE);
            }
        }

        if position.insufficient_material() || position.has_repetition(ply) || position.half_move_clock() >= 100 {
            return Some(DRAW_SCORE);
        }

        let standing_pat = data.evaluator.evaluate(position);

        if standing_pat >= beta {
            return Some(standing_pat);
        }
        if standing_pat > alpha {
            alpha = standing_pat;
        }

        let moves = position.legal_moves();
        for m in MoveSorter::sort_qsearch(position, moves).into_iter() {

            if !position.is_capture(m) {
                continue;
            }

            let victim = position.piece_on(m.to());
            let victim_value = data.evaluator.params().material[ if victim == Piece::NoPiece { Piece::Pawn } else { victim } as usize ];
            if standing_pat + Score::from_centi_pawns(victim_value) + Score::from_centi_pawns(200) <= alpha {
                continue;
            }

            position.make_move(m);

            let score = -Self::qsearch(position, ply + 1, -beta, -alpha, pv_node, data, thread_data)?;

            position.unmake_move(m);

            if score > alpha {
                alpha = score;
                if alpha >= beta {
                    return Some(beta);
                }
            }

        }

        Some(alpha)
    }

    //TODO:
    // return draw bound if one player has insufficient material.
    // 0,0 if both player have insufficient material,
    // 0,0 if repetition (try 0,+Inf, because a player actively needs to claim draw)
    // 0,0 if 50-move-rule (-||-)
    /*fn positional_score_bounds(position: &mut Position) -> (Score, Score) {
        if position.has_repetition
        (Score::NEGATIVE_INFTY, Score::POSITIVE_INFTY)
    }*/
}


impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}

impl MoveSorter {
    pub fn new() -> MoveSorter {
        MoveSorter {
            killer_moves: Vec::new()
        }
    }

    fn move_to_front(moves: &mut [Move], m: Move, front_index: &mut usize) {
        for i in *front_index..moves.len() {
            if moves[i] == m 
            {
                moves.swap(*front_index, i);
                *front_index += 1;
            }
        }
    }

    pub fn sort(&mut self, position: &mut Position, mut moves: Vec<Move>, ply: u16, ttable_move: Option<Move>) -> Vec<Move> {
        let mut sorted_moves = 0;
        
        if let Some(m) = ttable_move {
            Self::move_to_front(&mut moves, m, &mut sorted_moves);
        }

        if self.killer_moves.len() > ply as usize {
            Self::move_to_front(&mut moves, self.killer_moves[ply as usize].0, &mut sorted_moves);
            Self::move_to_front(&mut moves, self.killer_moves[ply as usize].1, &mut sorted_moves);
        } else {
            self.killer_moves.push((Move::new(0,0), Move::new(0,0)));
        }

        Self::sort_captures(position, &mut moves[sorted_moves..]);

        moves
    }

    pub fn sort_qsearch(position: &mut Position, mut moves: Vec<Move>) -> Vec<Move> {
        Self::sort_captures(position, &mut moves);
        moves
    }

    pub fn cut_off_move(&mut self, m: Move, ply: u16) {
        if self.killer_moves[ply as usize].0 != m {
            self.killer_moves[ply as usize].1 = self.killer_moves[ply as usize].0;
            self.killer_moves[ply as usize].0 = m;
        }
    }

    fn lva_mvv_values(position: &mut Position, m: Move) -> u8 {
        //println!("move: {}", m);
        let values = [1,3,3,5,9,0,1]; //Piece::NoPiece as usize == 7  =>  en passant captures have a victim value of 1
        
        let victim_value = values[position.piece_on(m.to()) as usize];
        let attacker_value = values[position.piece_on(m.from()) as usize];

        //println!("{victim_value}, {attacker_value}");

        return 16*victim_value - attacker_value;
    }

    fn sort_captures(position: &mut Position, moves: &mut [Move]) {
        let mut move_scores = Vec::new();
        
        for i in 0..moves.len() {
            if position.is_capture(moves[i]) {
                move_scores.push(Self::lva_mvv_values(position, moves[i]));
                moves.swap(i, move_scores.len()-1);
            }
        }

        //println!("{:?}", move_scores);

        for i in 1..move_scores.len() {
            let score = move_scores[i];
            let m = moves[i];
            let mut index = 0;
            for j in (0..i).rev() {
                if move_scores[j] < score {
                    move_scores[j+1] = move_scores[j];
                    moves[j+1] = moves[j];
                } else {
                    index = j+1;
                    break;
                }
            }
            
            moves[index] = m;
            move_scores[index as usize] = score;
        }
    }
}