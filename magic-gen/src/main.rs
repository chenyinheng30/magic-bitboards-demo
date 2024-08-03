mod rng;
mod threadpool;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use clap::Parser;
use rng::*;
use threadpool::*;
use types::*;

trait Move {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard;
    fn relevant_blockers(&self, square: Square) -> BitBoard;
    fn square_range(&self) -> &[Square];
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

    fn square_range(&self) -> &[Square] {
        &Square::ALL
    }
}

struct Cannon {
    deltas: [(i8, i8); 4],
    file_move_buffer: [u16; 10 * 1024],
    rank_move_buffer: [u16; 9 * 512],
}

impl Move for Cannon {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let rank_moves = self.rank_move_buffer[Cannon::to_rank_index(blockers, square)];
        let file_moves = self.file_move_buffer[Cannon::to_file_index(blockers, square)];
        let moves = Cannon::from_rank_moves(rank_moves, square.rank());
        moves | Cannon::from_file_moves(file_moves, square.file())
    }

    fn relevant_blockers(&self, square: Square) -> BitBoard {
        let mut blockers = BitBoard::EMPTY;
        for &(df, dr) in &self.deltas {
            let mut ray = square;
            while let Some(shifted) = ray.try_offset(df, dr) {
                blockers |= ray.bitboard();
                ray = shifted;
            }
            blockers |= ray.bitboard();
        }
        blockers &= !square.bitboard();
        blockers
    }

    fn square_range(&self) -> &[Square] {
        &Square::ALL
    }
}

impl Cannon {
    fn from_rank_moves(moves: u16, rank: Rank) -> BitBoard {
        let rank = rank as u8 * 9;
        BitBoard((moves as u128) << rank)
    }

    fn from_file_moves(moves: u16, file: File) -> BitBoard {
        let mut moves = moves;
        let mut bitboard = BitBoard::EMPTY;
        loop {
            let log2 = (moves & moves.wrapping_neg()).trailing_zeros();
            bitboard |= BitBoard(1u128 << (log2 * 9 + file as u32));
            moves &= moves.wrapping_sub(1);
            if moves == 0 {
                break;
            }
        }
        bitboard
    }

    fn to_rank_index(blockers: BitBoard, square: Square) -> usize {
        let rank = square.rank() as u8 * 9;
        let blockers = (blockers.0 as usize >> rank) & 0x1ff;
        ((square.file() as usize) << 9) + blockers
    }

    fn to_file_index(blockers: BitBoard, square: Square) -> usize {
        let blockers = blockers.0;
        let mut index = 0usize;
        let mut scan = 1 << square.file() as u8;
        for i in 0..10 {
            if blockers & scan != 0 {
                index |= 1usize << i
            }
            scan <<= 9;
        }
        ((square.rank() as usize) << 10) + index
    }

    fn _init_buffer(buffer: &mut [u16], size: usize) {
        let range = 1 << size;
        for blocker in 0..range {
            let blocker = blocker;
            for rank in 0..size {
                let mut moves = 1u16 << rank;
                let mut scan = 1u16 << rank;
                while scan != 0 && scan & blocker == 0 {
                    moves |= scan;
                    scan >>= 1;
                }
                scan >>= 1;
                while scan != 0 && scan & blocker == 0 {
                    scan >>= 1;
                }
                moves |= scan & blocker;
                let mut scan = 1u16 << rank;
                while scan < (1u16 << size) && scan & blocker == 0 {
                    moves |= scan;
                    scan <<= 1;
                }
                scan <<= 1;
                while scan < (1u16 << size) && scan & blocker == 0 {
                    scan <<= 1;
                }
                moves |= scan & blocker;
                moves ^= 1u16 << rank;
                buffer[(rank << size) + blocker as usize] = moves;
            }
        }
    }
    fn init_file_move(&mut self) {
        Cannon::_init_buffer(&mut self.file_move_buffer, 10);
    }
    fn init_rank_move(&mut self) {
        Cannon::_init_buffer(&mut self.rank_move_buffer, 9);
    }
}

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

static GIL: AtomicBool = AtomicBool::new(true);

// Given a sliding piece and a square, finds a magic number that
// perfectly maps input blockers into its solution in a hash table
fn find_magic(
    slider: &dyn Move,
    square: Square,
    index_bits: u8,
    rng: Arc<Mutex<Rng>>,
) -> (MagicEntry, Vec<BitBoard>) {
    let mask = slider.relevant_blockers(square);
    let shift = 128 - index_bits;
    let mut _i = 0;
    loop {
        // Magics require a low number of active bits, so we AND
        // by two more random values to cut down on the bits set.
        let magic = {
            let mut rng = rng.lock().unwrap();
            rng.next_u128() & rng.next_u128() & rng.next_u128()
        };
        let magic_entry = MagicEntry { mask, magic, shift };
        if let Ok(table) = try_make_table(slider, square, &magic_entry) {
            if GIL.load(Ordering::Relaxed) {
                GIL.fetch_and(false, Ordering::Acquire);
                println!("// {:?}: {:<12}", square, _i);
                _i = 0;
                GIL.fetch_or(true, Ordering::Release);
            }
            return (magic_entry, table);
        }
        if GIL.load(Ordering::Relaxed) {
            GIL.fetch_and(false, Ordering::Acquire);
            if _i % 127 == 0 {
                print!("// {:?}: {:<12}\r", square, _i);
            }
            _i += 1;
            GIL.fetch_or(true, Ordering::Release);
        }
    }
}

struct TableFillError;

// Attempt to fill in a hash table using a magic number.
// Fails if there are any non-constructive collisions.
fn try_make_table(
    slider: &dyn Move,
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

struct FindMagicsWorker {
    pool: ThreadPool,
    rng: Arc<Mutex<Rng>>,
}

impl FindMagicsWorker {
    fn new(size: usize) -> Self {
        FindMagicsWorker {
            pool: ThreadPool::new(size),
            rng: Arc::new(Mutex::new(Rng::default())),
        }
    }

    fn find_and_print_all_magics(
        &mut self,
        slider: Arc<dyn Move + Send + Sync + 'static>,
        slider_name: &str,
    ) {
        println!(
            "pub const {}_MAGICS: &[MagicEntry; Square::NUM] = &[",
            slider_name
        );
        let total_table_size = Arc::new(Mutex::new(0usize));
        for &square in slider.square_range() {
            let slider = Arc::clone(&slider);
            let rng = Arc::clone(&self.rng);
            let total_table_size = Arc::clone(&total_table_size);
            self.pool.execute(move || {
                find_and_print_step(slider, square, rng, total_table_size);
            });
        }
        self.pool.wait();
        println!("];");
        println!(
            "pub const {}_TABLE_SIZE: usage = {} KiB;",
            slider_name,
            *total_table_size.lock().unwrap() / 1024 * 16
        );
    }
}

fn find_and_print_step(
    slider: Arc<dyn Move + Send + Sync + 'static>,
    square: Square,
    rng: Arc<Mutex<Rng>>,
    total_table_size: Arc<Mutex<usize>>,
) {
    let index_bits = slider.relevant_blockers(square).popcnt() as u8;
    let (entry, table) = find_magic(&*slider, square, index_bits, rng);
    // In the final move generator, each table is concatenated into one contiguous table
    // for convenience, so an offset is added to denote the start of each segment.
    let mut total_table_size = total_table_size.lock().unwrap();
    if GIL.load(Ordering::Relaxed) {
        GIL.fetch_and(false, Ordering::Acquire);
        println!(
            "    MagicEntry {{ mask: {:?}, magic: 0x{:032X}, shift: {}, offset: {} }},",
            square, entry.magic, entry.shift, total_table_size
        );
        GIL.fetch_or(true, Ordering::Release);
    }
    *total_table_size += table.len();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional task_name to operate on
    task_name: Option<String>,
    /// number of thread
    #[arg(short, long, value_name = "THREAD_COUNT")]
    thread_count: Option<usize>,
}

fn main() {
    let cli = Cli::parse();
    let task_name = cli.task_name.as_deref();
    let mut worker;
    if let Some(thread_count) = cli.thread_count {
        worker = FindMagicsWorker::new(thread_count);
    } else {
        worker = FindMagicsWorker::new(1);
    }
    if task_name == None || task_name == Some("rook") {
        let rook = Slider {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
        };
        let rook = Arc::new(rook);
        worker.find_and_print_all_magics(rook, "ROOK");
    }
    if task_name == None || task_name == Some("cannon") {
        let mut cannon = Cannon {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
            file_move_buffer: [0; 10 * 1024],
            rank_move_buffer: [0; 9 * 512],
        };
        cannon.init_file_move();
        cannon.init_rank_move();
        let cannon = Arc::new(cannon);
        worker.find_and_print_all_magics(cannon, "CANNON");
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_cannon_moves0() {
        let mut cannon = Cannon {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
            file_move_buffer: [0; 10 * 1024],
            rank_move_buffer: [0; 9 * 512],
        };
        cannon.init_file_move();
        cannon.init_rank_move();
        let blocker = 0b0000000000;
        assert!(cannon.file_move_buffer[blocker] == 0b1111111110);
        assert!(cannon.file_move_buffer[(7 << 10) + blocker] == 0b1101111111);
        let blocker = 0b0010001000;
        assert!(cannon.file_move_buffer[blocker] == 0b010000110);
        assert!(cannon.file_move_buffer[(4 << 10) + blocker] == 0b001100000);
        let blocker = 0b0000001100;
        assert!(cannon.file_move_buffer[(7 << 10) + blocker] == 0b1101110100);
    }

    #[test]
    fn test_cannon_moves1() {
        let correct = bitboard! {
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            X X X . X X X X X
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
        };
        let file_bb = Cannon::from_file_moves(0b1111110111, File::D);
        let rank_bb = Cannon::from_rank_moves(0b111110111, Rank::Fourth);
        assert!(file_bb | rank_bb == correct);
    }

    #[test]
    fn test_cannon_moves2() {
        let correct = bitboard! {
            . . . X . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . X . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
        };
        let file_bb = Cannon::from_file_moves(0b1000001111, File::D);
        let rank_bb = Cannon::from_rank_moves(0b001000000, Rank::Fifth);
        assert!(file_bb | rank_bb == correct);
    }

    #[test]
    fn test_cannon_moves3() {
        let blockers = bitboard! {
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            X X X . X X X X X
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
        };
        let file_index = Cannon::to_file_index(blockers, Square::D4);
        let rank_index = Cannon::to_rank_index(blockers, Square::D4);
        assert!(file_index == 4087);
        assert!(rank_index == 2039)
    }

    #[test]
    fn test_cannon_moves4() {
        let mut cannon = Cannon {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
            file_move_buffer: [0; 10 * 1024],
            rank_move_buffer: [0; 9 * 512],
        };
        cannon.init_file_move();
        cannon.init_rank_move();
        let blockers = bitboard! {
            . . . X . . . . .
            . . . X . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . X . . . . X X .
            . . . X . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . . . . . . .
        };
        let moves = cannon.moves(Square::D5, blockers);
        let correct = bitboard! {
            . . . X . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . X . X X . X .
            . . . . . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . . . . . . .
        };
        assert!(moves == correct, "fail with {:?}", moves);
        let blockers = bitboard! {
            . . . X . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . . . . . . .
            . . . . X X X X X
            . . . . . . . . .
            . . . . . . . . .
            . . . . . . . . .
        };
        let moves = cannon.moves(Square::D4, blockers);
        let correct = bitboard! {
            . . . . . . . . .
            . . . . . . . . .
            . . . X . . . . .
            . . . . . . . . .
            . . . . . . . . .
            . . . X . . . . .
            X X X . . X . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
        };
        assert!(moves == correct, "fail with {:?}", moves)
    }

    #[test]
    fn test_cannon_mask0() {
        let cannon = Cannon {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
            file_move_buffer: [0; 10 * 1024],
            rank_move_buffer: [0; 9 * 512],
        };
        let mask: BitBoard = cannon.relevant_blockers(Square::D4);
        let correct = bitboard! {
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            X X X . X X X X X
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
        };
        assert!(mask == correct, "fail with {:?}", mask)
    }
    #[test]
    fn test_cannon_mask1() {
        let cannon = Cannon {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
            file_move_buffer: [0; 10 * 1024],
            rank_move_buffer: [0; 9 * 512],
        };
        let mask: BitBoard = cannon.relevant_blockers(Square::A0);
        let correct = bitboard! {
            . X X X X X X X X
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
            X . . . . . . . .
        };
        assert!(mask == correct, "fail with {:?}", mask)
    }
    #[test]
    fn test_cannon_mask2() {
        let cannon = Cannon {
            deltas: [(1, 0), (0, -1), (-1, 0), (0, 1)],
            file_move_buffer: [0; 10 * 1024],
            rank_move_buffer: [0; 9 * 512],
        };
        let mask: BitBoard = cannon.relevant_blockers(Square::D0);
        let correct = bitboard! {
            X X X . X X X X X
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
            . . . X . . . . .
        };
        assert!(mask == correct, "fail with {:?}", mask)
    }
}
