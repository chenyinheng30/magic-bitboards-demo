use types::{BitBoard, Square};

use crate::{rook::SLIDER_ONE_STEP, generate::ChessMove};

pub struct CannonAttack {
    deltas: [(i8, i8); 4],
}

impl CannonAttack {
    pub fn new() -> Self {
        CannonAttack { deltas: SLIDER_ONE_STEP }
    }
}

impl ChessMove for CannonAttack {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let mut moves = BitBoard::EMPTY;
        for (df, dr) in self.deltas {
            let mut ray = square;
            while !blockers.has(ray) {
                if let Some(shifted) = ray.try_offset(df, dr) {
                    ray = shifted;
                } else {
                    break;
                }
            }
            if let Some(mut ray) = ray.try_offset(df, dr) {
                while !blockers.has(ray) {
                    if let Some(shifted) = ray.try_offset(df, dr) {
                        ray = shifted;
                    } else {
                        break;
                    }
                }
                moves |= ray.bitboard();
            }
        }
        moves
    }

    fn relevant_blockers(&self, square: Square) -> BitBoard {
        let mut blockers = BitBoard::EMPTY;
        for (df, dr) in self.deltas {
            let mut ray = square;
            while let Some(shifted) = ray.try_offset(df, dr) {
                blockers |= ray.bitboard();
                ray = shifted;
            }
        }
        blockers &= !square.bitboard();
        blockers
    }

    fn possible_squares(&self) -> Vec<Square> {
        Vec::from(Square::ALL)
    }
}
