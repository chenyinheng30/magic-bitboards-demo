mod cannon;
pub mod generate;
mod king;
mod knight;
pub mod rng;
mod rook;

use cannon::*;
use king::King;
use knight::*;
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

pub fn king() -> King {
    King::new()
}
