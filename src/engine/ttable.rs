use std::mem::size_of;

use crate::chess_move::Move;

use super::score::Score;

pub struct TTable {
    table: Vec<TTableEntry> 
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntryType {
    Exact,
    Upper,
    Lower
}

#[derive(Copy, Clone)]
pub struct TTableEntry {
    pub hash: u64,
    pub entry_type: EntryType,
    pub score: Score,
    pub best_move: Move,
    pub depth: u16
}

impl TTableEntry {
    fn new() -> TTableEntry {
        TTableEntry {
            hash: 0,
            entry_type: EntryType::Upper,
            score: Score::NEGATIVE_INFTY,
            best_move: Move::new(0,0),
            depth: 0
        }
    }
}

impl TTable {
    pub fn new(mb_size: usize) -> TTable {
        TTable {
            table: vec![TTableEntry::new(); mb_size * (1<<20) / size_of::<TTableEntry>()] //new table with size MiB
        }
    }

    pub fn mb_size(&self) -> usize {
        self.table.len() / (1<<20) * size_of::<TTableEntry>()
    }

    pub fn lookup(&self, hash: u64) -> Option<&TTableEntry> {
        let index = hash as usize % self.table.len();

        if self.table[index].hash == hash {
            Some(&self.table[index])
        } else {
            None
        }
    }

    pub fn insert(&mut self, hash: u64, entry_type: EntryType, score: Score, best_move: Move, depth: u16) {
        let index = hash as usize % self.table.len();
        
        self.table[index as usize] = TTableEntry {
            hash,
            entry_type,
            score,
            best_move,
            depth
        }
    }
}