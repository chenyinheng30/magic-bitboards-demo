use types::{BitBoard, Square};

use crate::generate::ChessMove;

pub struct Slider {
    deltas: [(i8, i8); 4],
    start_range: Vec<Square>,
}

impl ChessMove for Slider {
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

    fn start_range(&self) -> Vec<Square> {
        self.start_range.clone()
    }
}

impl Slider {
    pub fn new(deltas: [(i8, i8); 4], start_range: Vec<Square>) -> Slider {
        Slider {
            deltas,
            start_range,
        }
    }
}
