mod rng;
mod threadpool;
mod rook;
mod generate;

use std::sync::{Arc, Mutex};

use generate::*;
use rng::*;
use rook::Slider;
use threadpool::*;
use types::*;

fn find_and_print_all_magics(
    slider: Arc<dyn ChessMove + Send + Sync + 'static>,
    slider_name: &str,
    rng: Arc<Mutex<Rng>>,
) {
    println!(
        "pub const {}_MAGICS: &[MagicEntry; Square::NUM] = &[",
        slider_name
    );
    let total_table_size = Arc::new(Mutex::new(0usize));
    let mut pool = ThreadPool::new(8);
    for &square in &Square::ALL {
        let slider = Arc::clone(&slider);
        let rng = Arc::clone(&rng);
        let total_table_size = Arc::clone(&total_table_size);
        pool.execute(move || {
            find_and_print_step(slider, square, rng, total_table_size);
        });
    }
    pool.wait();
    println!("];");
    println!(
        "pub const {}_TABLE_SIZE: usage = {} KiB;",
        slider_name,
        *total_table_size.lock().unwrap() / 1024 * 16
    );
}

fn find_and_print_step(
    slider: Arc<dyn ChessMove + Send + Sync + 'static>,
    square: Square,
    rng: Arc<Mutex<Rng>>,
    total_table_size: Arc<Mutex<usize>>,
) {
    let index_bits = slider.relevant_blockers(square).popcnt() as u8;
    let (entry, table) = find_magic(&*slider, square, index_bits, rng);
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
    let rook = Slider::new([(1, 0), (0, -1), (-1, 0), (0, 1)]);
    let rook = Arc::new(rook);
    find_and_print_all_magics(rook, "ROOK", Arc::clone(&rng));
}
