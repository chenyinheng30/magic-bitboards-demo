mod cannon;
pub mod generate;
mod knight;
mod pawn;
pub mod rng;
mod rook;

use cannon::*;
use knight::*;
use pawn::*;
use rook::*;

pub fn rook() -> Slider {
    Slider::new(SLIDER_ONE_STEP, Vec::from(types::Square::ALL))
}

pub fn cannon() -> CannonAttack {
    CannonAttack::new()
}

pub fn knight() -> LameLeaper<8> {
    LameLeaper::new(KNIGHT_DELTAS, KNIGHT_LAMELS, Vec::from(types::Square::ALL))
}

pub fn bishop() -> LameLeaper<4> {
    let start_range = Vec::from(BISHOP_START_RANGE);
    LameLeaper::new(BISHOP_DELTAS, BISHOP_LAMELS, start_range)
}

pub fn pawn(color: types::Color) -> ToNeighbour {
    ToNeighbour::pawn(match color {
        types::Color::Red => Pawn {
            begin: 3,
            end: 9,
            offset: 0,
            range: RED_PAWN,
            end_mask: types::BitBoard(0x1ff << 81),
        },
        types::Color::Black => Pawn {
            begin: 6,
            end: 0,
            offset: 1,
            range: BLACK_PAWN,
            end_mask: types::BitBoard(0x1ff),
        },
    })
}

pub fn advisor() -> ToNeighbour {
    ToNeighbour::advisor()
}

pub fn king() -> ToNeighbour {
    ToNeighbour::king()
}
