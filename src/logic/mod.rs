mod a_star;
mod bfs;

use std::collections::{BinaryHeap, HashMap};

use rand::Rng;

pub use a_star::{AStarHeuristic1, AStarHeuristic2};
pub use bfs::BfsHeuristic;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Puzzle {
    board: [[u8; 3]; 3],
}
use std::fmt;

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for row in self.board.iter() {
            for &val in row.iter() {
                write!(f, "{} ", val)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl Puzzle {
    pub fn new(board: [[u8; 3]; 3]) -> Self {
        Puzzle { board }
    }

    pub fn from_random() -> Self {
        let mut rnd = rand::thread_rng();
        let mut numbers = (1..=8).collect::<Vec<u8>>();
        let mut board = [[0; 3]; 3];

        let zero_pos = rnd.gen_range(0..9);
        board[zero_pos / 3][zero_pos % 3] = 0;
        for i in 0..9 {
            if i != zero_pos {
                board[i / 3][i % 3] = numbers.remove(rnd.gen_range(0..numbers.len()));
            }
        }

        Puzzle { board }
    }

    pub fn find_zero(&self) -> Option<(usize, usize)> {
        for (i, row) in self.board.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                if val == 0 {
                    return Some((i, j));
                }
            }
        }
        None
    }

    pub fn move_zero(&self, direction: Direction) -> Option<Puzzle> {
        if let Some((i, j)) = self.find_zero() {
            let new_board = self.board;
            let (new_i, new_j) = match direction {
                Direction::Up => (i as i32 - 1, j as i32),
                Direction::Down => (i as i32 + 1, j as i32),
                Direction::Left => (i as i32, j as i32 - 1),
                Direction::Right => (i as i32, j as i32 + 1),
            };
            if new_i >= 0 && new_i < 3 && new_j >= 0 && new_j < 3 {
                let mut new_board = new_board;
                new_board[i][j] = new_board[new_i as usize][new_j as usize];
                new_board[new_i as usize][new_j as usize] = 0;
                Some(Puzzle::new(new_board))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_value(&self, i: usize, j: usize) -> u8 {
        self.board[i][j]
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }

    pub const fn reverse(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

pub trait Heuristic {
    fn new() -> Self;
    fn estimate_h(&mut self, current: &Puzzle, goal: &Puzzle) -> i32;
}

pub trait SearchTree {
    fn goal_reached(&self) -> bool;
    fn get(&self, key: &Puzzle) -> Option<(Puzzle, i32)>;
    fn set(&mut self, key: Puzzle, value: (Puzzle, i32));
    fn step_callback(&mut self, _current: &Puzzle, _next: (&Puzzle, bool), _open_set: &OpenSet) {}
}

#[derive(Clone, Debug)]
struct BinaryHeapNode {
    puzzle: Puzzle,
    parent: Puzzle,
    g: i32,
    h: i32,
}

impl PartialEq for BinaryHeapNode {
    fn eq(&self, other: &Self) -> bool {
        (self.g + self.h) == (other.g + other.h)
    }
}

impl PartialOrd for BinaryHeapNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for BinaryHeapNode {}

impl Ord for BinaryHeapNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (other.g + other.h).cmp(&(self.g + self.h))
    }
}

pub struct OpenSet {
    set: BinaryHeap<BinaryHeapNode>,
    map: HashMap<Puzzle, (Puzzle, i32)>,
}

impl OpenSet {
    fn new() -> Self {
        OpenSet {
            set: BinaryHeap::new(),
            map: HashMap::new(),
        }
    }

    fn push(&mut self, node: BinaryHeapNode) {
        // self.map.insert(node.puzzle, (from, node.h));
        let old = self.map.get(&node.puzzle);
        if old.is_none() || old.unwrap().1 > node.g {
            self.map.insert(node.puzzle, (node.parent, node.g));
            self.set.push(node);
        }
    }

    fn pop(&mut self) -> Option<BinaryHeapNode> {
        loop {
            let heap_node = self.set.pop();
            if let Some(node) = &heap_node {
                if let Some(real) = self.map.remove(&node.puzzle) {
                    break Some(BinaryHeapNode {
                        puzzle: node.puzzle,
                        parent: real.0,
                        g: real.1,
                        h: node.h, // This value is not used outside of the binary heap
                    });
                }
            } else {
                break None;
            }
        }
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Puzzle, (Puzzle, i32)> {
        self.map.iter()
    }
}

pub fn solve_from_initial<S: SearchTree, H: Heuristic>(
    initial: Puzzle,
    goal: Puzzle,
    closed_set: &mut S,
) {
    let mut open_set = OpenSet::new();
    let mut h_estimator = H::new();

    open_set.push(BinaryHeapNode {
        puzzle: initial,
        parent: initial,
        g: 0,
        h: 0,
    });

    while let Some(current) = open_set.pop() {
        closed_set.set(current.puzzle, (current.parent, current.g));

        if current.puzzle == goal {
            break;
        }

        let current_g = closed_set.get(&current.puzzle).unwrap().1;

        for direction in Direction::all() {
            if let Some(next) = current.puzzle.move_zero(direction) {
                if closed_set.get(&next).is_some() {
                    continue;
                }
                
                let g = current_g + 1;
                let h = h_estimator.estimate_h(&next, &goal);

                open_set.push(BinaryHeapNode {
                    puzzle: next,
                    parent: current.puzzle,
                    g,
                    h,
                });
                closed_set.step_callback(&current.puzzle, (&next, false), &open_set);
                // TODO: remove this false
            }
        }
    }
}
