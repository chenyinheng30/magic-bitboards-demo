use crate::generate::ChessMove;
use types::{BitBoard, Square};

pub struct LameLeaper<const N: usize> {
    deltas: [(i8, i8); N],
    lamels: [(i8, i8); N],
    start_range: Vec<Square>,
}

impl<const N: usize> LameLeaper<N> {
    pub fn new(deltas: [(i8, i8); N], lamels: [(i8, i8); N], start_range: Vec<Square>) -> Self {
        LameLeaper {
            deltas,
            lamels,
            start_range,
        }
    }
}

impl<const N: usize> ChessMove for LameLeaper<N> {
    fn moves(&self, square: types::Square, blockers: types::BitBoard) -> types::BitBoard {
        let mut moves = BitBoard::EMPTY;
        for i in 0..N {
            if let (Some(lamel), Some(dst)) = (
                square.try_offset(self.lamels[i].0, self.lamels[i].1),
                square.try_offset(self.deltas[i].0, self.deltas[i].1),
            ) {
                if !blockers.has(lamel) {
                    moves |= dst.bitboard();
                }
            }
        }
        moves
    }

    fn relevant_blockers(&self, square: types::Square) -> types::BitBoard {
        let mut blockers = BitBoard::EMPTY;
        for lamel in self.lamels {
            if let Some(lamel) = square.try_offset(lamel.0, lamel.1) {
                blockers |= lamel.bitboard();
            }
        }
        blockers
    }

    fn possible_squares(&self) -> Vec<types::Square> {
        self.start_range.clone()
    }
}

pub const KNIGHT_DELTAS: [(i8, i8); 8] = [
    (2, 1),
    (2, -1),
    (1, 2),
    (-1, 2),
    (-2, 1),
    (-2, -1),
    (1, -2),
    (-1, -2),
];

pub const KNIGHT_LAMELS: [(i8, i8); 8] = [
    (1, 0),
    (1, 0),
    (0, 1),
    (0, 1),
    (-1, 0),
    (-1, 0),
    (0, -1),
    (0, -1),
];

pub const BISHOP_DELTAS: [(i8, i8); 4] = [(2, 2), (2, -2), (-2, 2), (-2, -2)];

pub const BISHOP_LAMELS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

pub const BISHOP_START_RANGE: [Square; 14] = [
    Square::C1,
    Square::G1,
    Square::A3,
    Square::E3,
    Square::I3,
    Square::C5,
    Square::G5,
    Square::C6,
    Square::G6,
    Square::A8,
    Square::E8,
    Square::I8,
    Square::C0,
    Square::G0,
];
