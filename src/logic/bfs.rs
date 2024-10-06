use super::{Heuristic, Puzzle};

pub struct BfsHeuristic {}

impl Heuristic for BfsHeuristic {
    fn new() -> Self {
        BfsHeuristic {}
    }

    fn estimate_h(&mut self, _current: &Puzzle, _goal: &Puzzle) -> i32 {
        0
    }
}
