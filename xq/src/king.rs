use std::io::Empty;

use types::BitBoard;

use crate::{generate::ChessMove, Slider, ToNeighbour, BLACK_PALACE, RED_PALACE};

pub struct King {
    to_neighbour: ToNeighbour,
}

impl ChessMove for King {
    fn moves(&self, mut square: types::Square, blockers: types::BitBoard) -> types::BitBoard {
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

    fn relevant_blockers(&self, mut square: types::Square) -> types::BitBoard {
        let mut mask = self.to_neighbour.relevant_blockers(square);
        let delta = if square.rank() as usize >= 7 { -1 } else { 1 };
        while let Some(x) = square.try_offset(0, delta) {
            mask |= x.bitboard();
            square = x;
        }
        mask ^ square.bitboard()
    }

    fn possible_squares(&self) -> Vec<types::Square> {
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
