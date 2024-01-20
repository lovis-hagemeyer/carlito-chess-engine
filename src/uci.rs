use std::io;

use crate::position;
use crate::position::*;
use crate::chess_move::*;
use crate::engine::*;

const NAME: &str = "Carlito Chess Engine";
const AUTHOR: &str = "Lovis Hagemeyer";

const DEFAULT_TABLE_SIZE: usize = 64;


pub struct UciHandler {
    position: Position,
    engine: Engine
}

impl UciHandler {
    pub fn new() -> UciHandler {
        UciHandler {
            position: Position::new(),
            engine: Engine::new(DEFAULT_TABLE_SIZE)
        }
    }

    pub fn input_loop(&mut self) {
        self.setup();

        for line in io::stdin().lines().map(|r| r.expect("error reading stdin")) {
            let mut tokens = line.split_whitespace();

            match tokens.next() {
                Some("setoption") => self.parse_set_option(&mut tokens),
                Some("isready") => println!("readyok"),
                Some("position") => self.parse_position(&mut tokens),
                Some("go") => self.parse_go(&mut tokens),
                Some("stop") => {
                    if tokens.next().is_some() {
                        eprintln!("invalid arguments for stop command");
                    }
                    self.engine.stop();
                },
                Some("ponderhit") => {
                    if tokens.next().is_some() {
                        eprintln!("invalid arguments for stop command");
                    }
                    self.engine.ponderhit();
                },
                Some("quit") => return,
                Some("uci") => (),
                Some(s) => eprintln!("unknown command: {s}"),
                None => ()
            }
        }
    }

    fn setup(&mut self) {
        for line in io::stdin().lines().map(|r| r.expect("error reading stdin")) {
            if line == "uci" {
                break;
            }   
        }

        println!("id name {NAME}");
        println!("id author {AUTHOR}");

        println!("option name Hash type spin default 256 min 1 max 4096");
        println!("option name Ponder type check default true");

        println!("uciok");
    }

    fn parse_set_option<'a, I: Iterator<Item = &'a str>>(&mut self, tokens: &mut I) {
        if tokens.next() != Some("name") {
            eprintln!("expected 'name' after 'setoption'");
            return;
        }

        match tokens.next().map(|s| s.to_ascii_lowercase()).as_deref() {
            None => { eprintln!("no arguments for setoption command"); }
            Some("hash") => { 
                if tokens.next() != Some("value") {
                    eprintln!("expected 'value' after 'setoption hash'");
                    return;
                }

                if let Some(n) = Self::parse_int_arg(tokens, "value") {
                    self.engine.set_table_size(n as usize); 
                }
            },
            Some("ponder") => { },
            Some(s) => { eprintln!("unsupported options: '{s}'"); }
        }
    }

    fn parse_go<'a, I: Iterator<Item = &'a str>>(&mut self, tokens: &mut I) {
        let mut opt = EngineOptions {
            search_moves: Vec::new(),
            ponder: false,
            infinite: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            moves_to_go: None,
            depth: None,
            nodes: None,
            mate_in: None,
            move_time: None,
        };

        let mut search_moves_flag = false;

        loop {
            let token = match tokens.next() {
                None => break,
                Some(s) => s
            };

            match token {
                "searchmoves" => { search_moves_flag = true; },
                "ponder" => { opt.ponder = true; search_moves_flag = false; },
                "infinite" => { opt.infinite = true; search_moves_flag = false; }
                "wtime" => { opt.wtime = Self::parse_int_arg(tokens, "wtime"); search_moves_flag = false; }
                "btime" => { opt.btime = Self::parse_int_arg(tokens, "btime"); search_moves_flag = false; },
                "winc" => { opt.winc = Self::parse_int_arg(tokens, "winc"); search_moves_flag = false; },
                "binc" => { opt.binc = Self::parse_int_arg(tokens, "binc"); search_moves_flag = false; },
                "movestogo" => { opt.moves_to_go = Self::parse_int_arg(tokens, "movestogo"); search_moves_flag = false; },
                "depth" => { opt.depth = Self::parse_int_arg(tokens, "depth"); search_moves_flag = false; },
                "nodes" => { opt.nodes = Self::parse_int_arg(tokens, "nodes"); search_moves_flag = false; },
                "mate" => { opt.mate_in = Self::parse_int_arg(tokens, "mate"); search_moves_flag = false; },
                "movetime" => { opt.move_time = Self::parse_int_arg(tokens, "movetime"); search_moves_flag = false; },
                "perft" => { } //TODO

                arg => {
                    if search_moves_flag { 
                        match Move::from_string(arg, &mut self.position) {
                            Ok(m) => opt.search_moves.push(m),
                            Err(_) => eprintln!("invalid or illegal move: '{arg}'")
                        }
                    } else {
                        eprintln!("invalid argument for go command: '{arg}'");
                    }
                }
            }
        }

        self.engine.start(self.position.clone(), opt);
    }

    fn parse_position<'a, I: Iterator<Item = &'a str>>(&mut self, tokens: &mut I) {
        let mut new_position = match tokens.next() {
            None => { 
                eprintln!("no arguments for position command"); 
                return; 
            },
            Some("startpos") => {
                if let Some(t) = tokens.next() {
                    if t != "moves" {
                        eprintln!("invalid argument for position command: '{t}', expected 'moves'");
                        self.position = Position::new();
                        return;
                    }
                }

                Position::new() 
            },
            Some("fen") => {

                let mut fen = String::new();
                for t in tokens.by_ref() {
                    if t == "moves" {
                        break;
                    }
                    fen.push_str(t);
                    fen.push(' ');
                }

                fen.pop(); //remove last space character

                match Position::from_fen_string(fen.as_str()) {
                    Ok(p) => p,
                    Err(_) => { 
                        eprintln!("invalid fen: '{fen}'"); 
                        return; 
                    }
                }
            },
            Some(t) => { 
                eprintln!("invalid argument for position command: '{t}'"); 
                return;
            }
        };

        for move_str in tokens {
            match Move::from_string(move_str, &mut new_position) {
                Ok(m) => new_position.make_move(m),
                Err(_) => {
                    eprintln!("invalid move format or illegal move: '{move_str}'");
                    break;
                }
            }
        }

        self.position = new_position;
    }

    fn parse_int_arg<'a, I: Iterator<Item = &'a str>>(tokens: &mut I, last_token: &str) -> Option<u32> {
        match tokens.next() {
            Some(s) => match Position::parse_int(s, u32::MAX/10) {
                Ok(i) => Some(i),
                Err(_) => {
                    eprintln!("expected integer after '{last_token}', got: '{s}");
                    None
                }
            },
            None => {
                eprintln!("expected argument after '{last_token}'");
                None
            }
        }
    }

    fn split_perft(pos: &mut Position, depth: u8, root_node: bool) -> u64 {
        if depth == 0 {
            1
        } else {
            let mut result = 0;
            for m in pos.legal_moves().into_iter() {
    
                pos.make_move(m);
                let child_nodes = Self::split_perft(pos, depth-1,false);
                if root_node {
                    println!("{}: {child_nodes}", m);
                }
                result += child_nodes;
                pos.unmake_move(m);
            }
    
            result
        }
    }
}