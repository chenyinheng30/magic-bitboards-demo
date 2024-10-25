use types::{bitboard, BitBoard, Square};

use crate::generate::ChessMove;

pub const RED_PALACE: BitBoard = bitboard![
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . X X X . . .
    . . . X X X . . .
    . . . X X X . . .
];

pub const BLACK_PALACE: BitBoard = bitboard![
    . . . X X X . . .
    . . . X X X . . .
    . . . X X X . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
];

pub struct King;

impl ChessMove for King {
    fn moves(&self, mut square: Square, blockers: BitBoard) -> BitBoard {
        let mut moves = BitBoard::EMPTY;
        let (another, delta) = if BLACK_PALACE.has(square) {
            (RED_PALACE, -1)
        } else {
            (BLACK_PALACE, 1)
        };
        while let Some(x) = square.try_offset(0, delta) {
            square = x;
            if blockers.has(square) {
                break;
            }
        }
        while another.has(square) {
            moves ^= square.bitboard();
            if let Some(x) = square.try_offset(0, -delta) {
                square = x;
            } else {
                break;
            }
        }
        moves
    }

    fn relevant_blockers(&self, mut square: Square) -> BitBoard {
        let mut mask = BitBoard::EMPTY;
        let delta = if square.rank() as usize >= 7 { -1 } else { 1 };
        while let Some(x) = square.try_offset(0, delta) {
            mask |= x.bitboard();
            square = x;
        }
        mask ^ square.bitboard()
    }

    fn possible_squares(&self) -> Vec<Square> {
        let start_range = RED_PALACE | BLACK_PALACE;
        start_range.into_iter().collect()
    }
}

impl King {
    pub fn new() -> Self {
        King {}
    }
}
