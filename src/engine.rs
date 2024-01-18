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

use crate::chess_move::*;
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
    search_aborted: bool,
    nodes: u64,
    killer_moves: Vec<(Move, Move)>,
    ttable: TTable
}

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
            search_aborted: false,
            nodes: 0,
            killer_moves: Vec::new(),
            ttable
        };

        let mut pv: Vec<Move> = Vec::new();

        loop {
            data.pv = Vec::new();

            let score = Engine::search(&mut position, depth, 0, Score::NEGATIVE_INFTY, Score::POSITIVE_INFTY, true, &mut data, &thread_data);

            if data.search_aborted {
                break;
            }

            pv = data.pv;

            let search_time = Instant::now().duration_since(thread_data.start_time);
            let search_time_ms = search_time.as_millis() as u64;

            //print info about search depth

            print!("info depth {depth}");
            if let Some(s) = score.centi_pawns() {
                print!(" cp {s}");
            } else {
                print!(" mate {}", score.mate().unwrap());
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

    fn search(position: &mut Position, depth: u16, ply: u32, mut alpha: Score, beta: Score, pv_node: bool, data: &mut SearchData, thread_data: &ThreadData) -> Score {
        
        if thread_data.stop.load(atomic::Ordering::Acquire) {
            data.search_aborted = true;
            return Score::from_centi_pawns(0);
        }

        data.nodes += 1;

        //TODO nodes option should be u64
        if (thread_data.options.nodes.unwrap_or(u32::MAX) as u64) < data.nodes {
            data.search_aborted = true;
            return Score::from_centi_pawns(0);
        }

        let mut moves = position.legal_moves();
        
        if moves.len() == 0 {
            if position.is_attacked(position.king_square(position.current_player()), position.current_player()) {
                return Score::from_mate_distance(-(((ply+1)/2) as i16));
            } else {
                return Score::from_centi_pawns(0);
            }
        }

        if depth == 0 {
            return data.evaluator.evaluate(position);
        }


        let mut sorted_moves = 0;

        //transposition table look up
        if let Some(table_entry) = data.ttable.lookup(position.hash()) {
            if table_entry.depth == depth {
                //TODO account for search instabilities when using pvs. (see Calito)
                match table_entry.entry_type {
                    ttable::EntryType::Exact => {
                        if ply == 0 {
                            data.pv.push(table_entry.best_move);
                        }
                        return table_entry.score
                    },
                    ttable::EntryType::Upper => {
                        if alpha >= table_entry.score {
                            return table_entry.score;
                        }
                    },
                    ttable::EntryType::Lower => {
                        if table_entry.score >= beta {
                            return table_entry.score;
                        }
                    }
                }
            }

            if table_entry.entry_type == EntryType::Lower || table_entry.entry_type == EntryType::Exact {
                for i in 0..moves.len() {
                    if moves[i] == table_entry.best_move {
                        moves.swap(sorted_moves, i);
                        sorted_moves += 1;
                        break;
                    }
                }
            }
        }


        //killer moves
        if data.killer_moves.len() > ply as usize {
            for i in sorted_moves..moves.len() {
                if moves[i] == data.killer_moves[ply as usize].0 {
                    moves.swap(sorted_moves, i);
                    sorted_moves += 1;
                    break;
                }
            }

            for i in sorted_moves..moves.len() {
                if moves[i] == data.killer_moves[ply as usize].1 {
                    moves.swap(sorted_moves, i);
                    //sorted_moves += 1;
                    break;
                }
            }

        } else {
            data.killer_moves.push((Move::new(0,0), Move::new(0,0)));
        }


        let mut best_move = moves[0];

        for (i, m) in moves.into_iter().enumerate() {
            position.make_move(m);

            let mut move_score;
            
            if pv_node {
                if i == 0 {
                    move_score = -Engine::search(position, depth - 1, ply + 1, -beta, -alpha, true,  data, thread_data);
                } else {
                    move_score = -Engine::search(position, depth - 1, ply + 1, - (Score { s: (alpha.s + 1) }), -alpha, false,  data, thread_data);
                    if move_score > alpha {
                        move_score = -Engine::search(position, depth - 1, ply + 1, -beta, -alpha, true,  data, thread_data);
                    }
                }
            } else {
                move_score = -Engine::search(position, depth - 1, ply + 1, -beta, -alpha, false,  data, thread_data)
            }


            position.unmake_move(m);

            if move_score > alpha {
                alpha = move_score;
                best_move = m;

                if alpha >= beta {

                    if data.killer_moves[ply as usize].0 != m {
                        data.killer_moves[ply as usize].1 = data.killer_moves[ply as usize].0;
                        data.killer_moves[ply as usize].0 = m;
                    }

                    data.ttable.insert(position.hash(), EntryType::Lower, move_score, best_move, depth);
                    
                    return beta;
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

        alpha
    }
}


impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}