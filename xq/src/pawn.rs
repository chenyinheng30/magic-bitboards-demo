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
        self.moves(square, BitBoard(0))
    }

    fn start_range(&self) -> Vec<Square> {
        self.start_range.into_iter().collect()
    }
}

const RED_PALACE: BitBoard = bitboard![
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

const BLACK_PALACE: BitBoard = bitboard![
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

pub const RED_PAWN: BitBoard = bitboard![
    X X X X X X X X X
    X X X X X X X X X
    X X X X X X X X X
    X X X X X X X X X
    X X X X X X X X X
    X . X . X . X . X
    X . X . X . X . X
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
];

pub const BLACK_PAWN: BitBoard = bitboard![
    . . . . . . . . .
    . . . . . . . . .
    . . . . . . . . .
    X . X . X . X . X
    X . X . X . X . X
    X X X X X X X X X
    X X X X X X X X X
    X X X X X X X X X
    X X X X X X X X X
    X X X X X X X X X
];

pub struct Pawn {
    pub begin: usize,
    pub end: usize,
    pub offset: usize,
    pub range: BitBoard,
    pub end_mask: BitBoard,
}

impl ToNeighbour {
    pub fn pawn(info: Pawn) -> Self {
        let m = 0x3ffffu128;
        let mut masks = [BitBoard::EMPTY; 10];
        let begin = info.begin;
        let end = info.end;
        let offset = info.offset;
        for i in begin.min(end + offset)..end.max(begin + offset) {
            masks[i] = BitBoard(m << (9 * (i - offset))) & info.range;
        }
        masks[end] = info.end_mask;
        ToNeighbour {
            deltas: vec![(1, 0), (0, 1), (-1, 0), (0, -1)],
            start_range: info.range,
            masks,
        }
    }

    pub fn advisor() -> Self {
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
            deltas: vec![(1, 1), (1, -1), (-1, 1), (-1, -1)],
            start_range: RED_PALACE | BLACK_PALACE,
            masks,
        }
    }

    pub fn king() -> Self {
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
