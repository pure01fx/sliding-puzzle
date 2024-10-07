use super::{Heuristic, Puzzle};

pub struct AStarHeuristic1 {}

impl Heuristic for AStarHeuristic1 {
    fn new() -> Self {
        AStarHeuristic1 {}
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

pub struct AStarHeuristic2 {}

impl Heuristic for AStarHeuristic2 {
    fn new() -> Self {
        AStarHeuristic2 {}
    }

    fn estimate_h(&mut self, current: &Puzzle, goal: &Puzzle) -> i32 {
        let mut count = 0;

        for i in 0..3 {
            for j in 0..3 {
                if current.get_value(i, j) != goal.get_value(i, j) && current.get_value(i, j) != 0 {
                    count += 1;
                }
            }
        }

        count
    }
}
