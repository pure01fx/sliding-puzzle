use super::{Heuristic, Puzzle};

pub struct AStarHeuristic {}

impl Heuristic for AStarHeuristic {
    fn new() -> Self {
        AStarHeuristic {}
    }

    fn estimate_h(&mut self, current: &Puzzle, goal: &Puzzle) -> i32 {
        let mut count = 0;

        for i in 0..3 {
            for j in 0..3 {
                if current.get_value(i, j) != goal.get_value(i, j) {
                    count += 1;
                }
            }
        }

        count
    }
}
