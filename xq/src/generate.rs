use crate::rng::Rng;
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};
use types::{BitBoard, Square};

pub trait ChessMove {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard;
    fn relevant_blockers(&self, square: Square) -> BitBoard;
    fn start_range(&self) -> Vec<Square>;
}

pub struct MagicEntry {
    pub mask: BitBoard,
    pub magic: u128,
    pub shift: u8,
}

pub struct MagicEntryGen {
    pub square: Square,
    pub magic: u128,
    pub shift: u8,
    pub size: usize,
}

impl Display for MagicEntryGen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MagicEntryGen {{ square: {:?}, magic: 0x{:032X}, shift: {}, size: {} }}",
            self.square, self.magic, self.shift, self.size
        )
    }
}

pub fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic as u128);
    let index = (hash >> entry.shift) as usize;
    index
}

// Given a sliding piece and a square, finds a magic number that
// perfectly maps input blockers into its solution in a hash table
pub fn find_magic(
    slider: &dyn ChessMove,
    square: Square,
    index_bits: u8,
    rng: Arc<Mutex<Rng>>,
) -> (MagicEntryGen, Vec<BitBoard>) {
    let mask = slider.relevant_blockers(square);
    let shift = 128 - index_bits;
    loop {
        // Magics require a low number of active bits, so we AND
        // by two more random values to cut down on the bits set.
        let magic = {
            let mut rng = rng.lock().unwrap();
            rng.next_u128() & rng.next_u128() & rng.next_u128()
        };
        let magic_entry = MagicEntry { mask, magic, shift };
        if let Ok(table) = try_make_table(slider, square, &magic_entry) {
            let magic_entry_gen = MagicEntryGen {
                square,
                magic,
                shift,
                size: table.len(),
            };
            return (magic_entry_gen, table);
        }
    }
}

struct TableFillError;

// Attempt to fill in a hash table using a magic number.
// Fails if there are any non-constructive collisions.
fn try_make_table(
    slider: &dyn ChessMove,
    square: Square,
    magic_entry: &MagicEntry,
) -> Result<Vec<BitBoard>, TableFillError> {
    let index_bits = 128 - magic_entry.shift;
    let mut table = vec![BitBoard::EMPTY; 1 << index_bits];
    // Iterate all configurations of blockers
    let mut blockers = BitBoard::EMPTY;
    loop {
        let moves = slider.moves(square, blockers);
        let index = magic_index(magic_entry, blockers);
        let table_entry = &mut table[index];
        if table_entry.is_empty() {
            // Write to empty slot
            *table_entry = moves;
        } else if *table_entry != moves {
            // Having two different move sets in the same slot is a hash collision
            return Err(TableFillError);
        }

        // Carry-Rippler trick that enumerates all subsets of the mask, getting us all blockers.
        // https://www.chessprogramming.org/Traversing_Subsets_of_a_Set#All_Subsets_of_any_Set
        blockers.0 = blockers.0.wrapping_sub(magic_entry.mask.0) & magic_entry.mask.0;
        if blockers.is_empty() {
            // Finished enumerating all blocker configurations
            break;
        }
    }
    Ok(table)
}
