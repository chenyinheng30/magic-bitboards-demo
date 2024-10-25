use types::{bitboard, BitBoard, Rank, Square};

use crate::generate::ChessMove;

pub struct ToNeighbour {
    deltas: Vec<(i8, i8)>,
    masks: [BitBoard; 10],
    start_range: BitBoard,
}

impl ChessMove for ToNeighbour {
    fn moves(&self, square: Square, blockers: BitBoard) -> BitBoard {
        let _ = blockers;
        let mut moves = BitBoard::EMPTY;
        for &(df, dr) in &self.deltas {
            if let Some(dst) = square.try_offset(df, dr) {
                moves |= dst.bitboard();
            }
        }
        moves & self.masks[square.rank() as usize]
    }

    fn relevant_blockers(&self, square: Square) -> BitBoard {
        let _ = square;
        BitBoard::EMPTY
    }

    fn possible_squares(&self) -> Vec<Square> {
        self.start_range.into_iter().collect()
    }
}

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

impl ToNeighbour {
    pub fn king_to_neighbour() -> Self {
        let mut masks = [BitBoard::EMPTY; 10];
        for rank in Rank::ALL {
            let rank = rank as usize;
            if rank < 3 {
                masks[rank] = RED_PALACE;
            } else if rank >= 7 {
                masks[rank] = BLACK_PALACE;
            }
        }
        ToNeighbour {
            deltas: vec![(1, 0), (0, 1), (-1, 0), (0, -1)],
            start_range: RED_PALACE | BLACK_PALACE,
            masks,
        }
    }
}

pub struct King {
    to_neighbour: ToNeighbour,
}

impl ChessMove for King {
    fn moves(&self, mut square: Square, blockers: BitBoard) -> BitBoard {
        let mut moves = self.to_neighbour.moves(square, blockers);
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
        let mut mask = self.to_neighbour.relevant_blockers(square);
        let delta = if square.rank() as usize >= 7 { -1 } else { 1 };
        while let Some(x) = square.try_offset(0, delta) {
            mask |= x.bitboard();
            square = x;
        }
        mask ^ square.bitboard()
    }

    fn possible_squares(&self) -> Vec<Square> {
        self.to_neighbour.possible_squares()
    }
}

impl King {
    pub fn new() -> Self {
        King {
            to_neighbour: ToNeighbour::king_to_neighbour(),
        }
    }
}
