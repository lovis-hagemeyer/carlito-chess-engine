use std::io;

use crate::position::*;

const NAME: &str = "Carlito Chess Engine";
const AUTHOR: &str = "Lovis Hagemeyer";

pub struct UciOptions {

}

impl UciOptions {

}

pub fn input_loop() {
    setup();


    let mut pos = Position::new();

    for line in io::stdin().lines().map(|r| r.expect("error reading stdin")) {
        let mut tokens = line.split(' ');

        match tokens.next().unwrap() { //unwrap never panics, because str.split always returns at least one element
            "setoption" => (),
            "isready" => println!("readyok"),
            "position" => (),
            "go" => (),
            "stop" => (),
            "ponderhit" => (),
            "quit" => return,
            "uci" => (),
            s => eprintln!("unknown command: {s}")
        }
    }
}

fn setup() {
    for line in io::stdin().lines().map(|r| r.expect("error reading stdin")) {
        if line == "uci" {
            break;
        }   
    }

    println!("id name {NAME}");
    println!("id author {AUTHOR}");

    //print supported options
}

fn parse_position_command<T>(args: T) -> Result<Position, ()> 
where 
    T: Iterator<Item = String>
{
    Err(())
}