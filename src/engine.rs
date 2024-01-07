use std::sync::{atomic, Arc, Mutex};
use std::thread;
use std::time::{self, Duration, Instant};


mod timer;
mod evaluate;

use crate::position::*;
use timer::*;
use crate::chess_move::*;

pub struct Engine {
    thread_data: Option<Arc<ThreadData>>,
    worker_thread: Option<thread::JoinHandle<()>>,
    timer: Option<Timer>,
}

pub struct ThreadData {
    stop: atomic::AtomicBool,
    ponder: atomic::AtomicBool,

    start_time: Instant,
    max_time: Mutex<Option<Duration>>,

    position: Position,
    options: EngineOptions
}

struct SearchData {
    pv: Vec<Move>,

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
    pub fn new() -> Engine {
        Engine {
            thread_data: None,
            worker_thread: None,
            timer: None,
        }
    }

    pub fn start(&mut self, position: Position, options: EngineOptions) {
        self.stop();


        self.thread_data = Some(Arc::new(ThreadData {
            stop: atomic::AtomicBool::new(false),
            ponder: atomic::AtomicBool::new(options.ponder),

            start_time: Instant::now(),
            max_time: Mutex::new(None),

            position,
            options
        }));


        self.timer = Timer::new(self.thread_data.clone().unwrap());
        

        let thread_data_ref = self.thread_data.clone().unwrap();

        self.worker_thread = Some(thread::spawn(move || {
            Engine::analyze(thread_data_ref);
        }));
    }

    pub fn stop(&mut self) {    
        match self.worker_thread.take() {
            None => (),
            Some(handle) => {
                self.thread_data.as_ref().unwrap().stop.store(true, atomic::Ordering::Release);
                handle.join().expect("error when joining worker thread");
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

    pub fn set_table_size(&mut self, size_in_mb: u32) {
        //TODO
        println!("setting table size to {size_in_mb}");
    }

    fn analyze(thread_data: Arc<ThreadData>) {

        let mut position = thread_data.position.clone();

        let mut data = SearchData {
            pv: Vec::new(),
        };

        //TODO

        Engine::search(&mut position, 1, &mut data);

        println!("bestmove {}", data.pv[0]);
    }

    fn search(position: &mut Position, depth: u32, data: &mut SearchData) -> i16 {
        let moves = position.legal_moves();
        
        data.pv.push(moves[moves.len()/2]);
        
        0
    }
}


impl Drop for Engine {
    fn drop(&mut self) {
        self.stop();
    }
}