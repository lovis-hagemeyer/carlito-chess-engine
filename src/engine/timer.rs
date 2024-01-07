use std::sync::mpsc::RecvTimeoutError;
use std::sync::{atomic, Arc, mpsc};
use std::time::Duration;
use std::thread;

use crate::position::Color;

use super::ThreadData;

pub struct Timer {
    timer_thread: Option<thread::JoinHandle<()>>,
    abort: mpsc::Sender<()>
}


impl Timer {
    
    pub fn new(thread_data: Arc<ThreadData>) -> Option<Timer> {

        if thread_data.options.infinite || thread_data.ponder.load(atomic::Ordering::Acquire) {
            None 
        } else {
            let ms = if let Some(move_time) = thread_data.options.move_time {
                move_time
            } else {
                let moves_to_go = thread_data.options.moves_to_go.unwrap_or(50);
                
                let clock_time = match thread_data.position.current_player() {
                    Color::White => thread_data.options.wtime,
                    Color::Black => thread_data.options.btime,
                }.unwrap_or(1000); // if no time given think for 1 second

                let increment = match thread_data.position.current_player() {
                    Color::White => thread_data.options.winc,
                    Color::Black => thread_data.options.binc,
                }.unwrap_or(0);

                let max_time = (clock_time / moves_to_go + increment) * 2;

                if clock_time <= 1 {
                    1
                } else if clock_time < 100 {
                    clock_time / 2
                } else if max_time < clock_time - 50 {
                    clock_time - 50
                } else {
                    max_time
                }
            };

            *(thread_data.max_time.lock().unwrap()) = Some(Duration::from_millis(ms as u64));


            let (sender, reciever) = mpsc::channel::<()>();

            let t = thread::spawn(move || {
                match reciever.recv_timeout(thread_data.max_time.lock().unwrap().unwrap()) {
                    Ok(()) => (),
                    Err(RecvTimeoutError::Disconnected) => panic!("timing abort sender dropped while timer thread still active"),
                    Err(RecvTimeoutError::Timeout) => thread_data.stop.store(true, atomic::Ordering::Release) //time is up, set stop flag
                }
            });

            Some(Timer {
                timer_thread: Some(t),
                abort: sender
            })
        } 
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        if self.abort.send(()).is_err() { }; //the timer thread may have already finished
        self.timer_thread.take().unwrap().join().unwrap();
    }
}