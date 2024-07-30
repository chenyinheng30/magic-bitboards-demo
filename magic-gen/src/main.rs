mod rng;
mod threadpool;

use std::{borrow::Borrow, sync::{Arc, Mutex}};

use rng::*;
use types::*;
use threadpool::*;

trait Move {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard;
    fn relevant_blockers(&self, square: Square) -> BitBoard;
}

struct Slider {
    deltas: [(i8, i8); 4],
}

impl Move for Slider {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let mut moves = BitBoard::EMPTY;
        for &(df, dr) in &self.deltas {
            let mut ray = square;
            while !blockers.has(ray) {
                if let Some(shifted) = ray.try_offset(df, dr) {
                    ray = shifted;
                    moves |= ray.bitboard();
                } else {
                    break;
                }
            }
        }
        moves
    }

    fn relevant_blockers(&self, square: Square) -> BitBoard {
        let mut blockers = BitBoard::EMPTY;
        for &(df, dr) in &self.deltas {
            let mut ray = square;
            while let Some(shifted) = ray.try_offset(df, dr) {
                blockers |= ray.bitboard();
                ray = shifted;
            }
        }
        blockers &= !square.bitboard();
        blockers
    }
}

const ROOK: Slider = Slider {
    deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
};

struct MagicEntry {
    mask: BitBoard,
    magic: u128,
    shift: u8,
}

fn magic_index(entry: &MagicEntry, blockers: BitBoard) -> usize {
    let blockers = blockers & entry.mask;
    let hash = blockers.0.wrapping_mul(entry.magic as u128);
    let index = (hash >> entry.shift) as usize;
    index
}

// Given a sliding piece and a square, finds a magic number that
// perfectly maps input blockers into its solution in a hash table
fn find_magic(
    slider: &Arc<dyn Move + Send + Sync + 'static>,
    square: Square,
    index_bits: u8,
    rng: Arc<Mutex<Rng>>,
) -> (MagicEntry, Vec<BitBoard>) {
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
            return (magic_entry, table);
        }
    }
}

struct TableFillError;

// Attempt to fill in a hash table using a magic number.
// Fails if there are any non-constructive collisions.
fn try_make_table(
    slider: &Arc<dyn Move + Send + Sync + 'static>,
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

fn find_and_print_all_magics(slider: Arc<dyn Move + Send + Sync + 'static>, slider_name: &str, rng: Arc<Mutex<Rng>>) {
    println!(
        "pub const {}_MAGICS: &[MagicEntry; Square::NUM] = &[",
        slider_name
    );
    let total_table_size  = Arc::new(Mutex::new(0usize));
    let pool = ThreadPool::new(8);
    for &square in &Square::ALL {
        let slider = Arc::clone(&slider);
        let rng = Arc::clone(&rng);
        let total_table_size = Arc::clone(&total_table_size);
        pool.execute(move ||{
            find_and_print_step(slider, square, rng, total_table_size);
        });
    }
    println!("];");
    println!(
        "pub const {}_TABLE_SIZE: usage = {} KiB;",
        slider_name, *total_table_size.lock().unwrap() / 1024 * 16
    );
}

fn find_and_print_step(slider: Arc<dyn Move + Send + Sync + 'static>, square: Square, rng: Arc<Mutex<Rng>>, total_table_size: Arc<Mutex<usize>>) {
    let index_bits = slider.relevant_blockers(square).popcnt() as u8;
    let (entry, table) = find_magic(&slider, square, index_bits, rng);
    // In the final move generator, each table is concatenated into one contiguous table
    // for convenience, so an offset is added to denote the start of each segment.
    let mut total_table_size = total_table_size.lock().unwrap();
    println!(
        "    MagicEntry {{ mask: 0x{:016X}, magic: 0x{:032X}, shift: {}, offset: {} }},",
        entry.mask.0, entry.magic, entry.shift, total_table_size
    );
    *total_table_size += table.len();
}

fn main() {
    let rng = Arc::new(Mutex::new(Rng::default()));
    let rook = Arc::new(ROOK);
    find_and_print_all_magics(rook, "ROOK", Arc::clone(&rng));
}
