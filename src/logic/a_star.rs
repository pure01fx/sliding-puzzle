use super::{Heuristic, Puzzle};

pub struct AStarHeuristic {}

impl Heuristic for AStarHeuristic {
    fn new() -> Self {
        AStarHeuristic {}
    }

    fn estimate_h(&mut self, current: &Puzzle, goal: &Puzzle) -> i32 {
        let mut count = 0;

        for i in 0..current.board.len() {
            if current.board[i] != goal.board[i] {
                count += 1;
            }
        }

        count
    }
}