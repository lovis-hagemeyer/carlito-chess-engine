use std::mem::size_of;

use crate::chess_move::Move;

use super::score::Score;

pub struct TTable {
    table: Vec<Bucket> 
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EntryType {
    Exact,
    Upper,
    Lower
}

#[derive(Clone, Copy)]
pub struct TTableEntry {
    pub hash: u64,
    pub entry_type: EntryType,
    pub score: Score,
    pub best_move: Move,
    pub depth: u16
}

#[derive(Clone)]
#[repr(align(64))] // buckets should align with cache lines
struct Bucket {
    pub entries: [TTableEntry; 4]
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

impl Bucket {
    fn new() -> Bucket {
        Bucket {
            entries: [TTableEntry::new(); 4]
        }
    }
}

impl TTable {
    pub fn new(mb_size: usize) -> TTable {
        TTable {
            table: vec![Bucket::new(); mb_size * (1<<20) / size_of::<Bucket>()] //new table with size MiB
        }
    }

    pub fn mb_size(&self) -> usize {
        self.table.len() / (1<<20) * size_of::<Bucket>()
    }

    pub fn lookup(&self, hash: u64) -> Option<&TTableEntry> {
        let index = hash as usize % self.table.len();

        for entry in self.table[index].entries.iter() {
            if entry.hash == hash {
                return Some(entry);
            } 
        }

        None
    }

    pub fn insert(&mut self, hash: u64, entry_type: EntryType, score: Score, best_move: Move, depth: u16) {
        let index = hash as usize % self.table.len();
        
        //replacement strategy: always replace entry with smallest depth
 
        let (mut bucket_index, mut smallest_depth) = (0, u16::MAX);
    
        for (i, entry) in self.table[index].entries.iter().enumerate() {
            if entry.hash == hash {
                bucket_index = i;
                break;
            }
            
            if entry.depth < smallest_depth {
                smallest_depth = entry.depth;
                bucket_index = i;
            }
        }

        self.table[index].entries[bucket_index] = TTableEntry {
            hash,
            entry_type,
            score,
            best_move,
            depth
        }
    }
    
    pub fn clear(&mut self) {
        for b in self.table.iter_mut() {
            *b = Bucket::new();
        }
    }
}