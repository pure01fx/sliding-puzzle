use std::collections::{BinaryHeap, HashMap, VecDeque};

use rand::Rng;

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

    fn find_zero(&self) -> Option<(usize, usize)> {
        for (i, row) in self.board.iter().enumerate() {
            for (j, &val) in row.iter().enumerate() {
                if val == 0 {
                    return Some((i, j));
                }
            }
        }
        None
    }

    fn move_zero(&self, direction: Direction) -> Option<Puzzle> {
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
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    const fn all() -> [Direction; 4] {
        [
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ]
    }

    const fn reverse(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

pub struct SolutionMap {
    goal: Puzzle,
    steps: HashMap<Puzzle, Direction>,
}

impl SolutionMap {
    pub fn new(goal: Puzzle) -> Self {
        let mut queue = VecDeque::new();
        let mut steps = HashMap::new();

        queue.push_back((goal, 0));

        while let Some((current, step)) = queue.pop_front() {
            for direction in Direction::all() {
                if let Some(next) = current.move_zero(direction) {
                    if !steps.contains_key(&next) {
                        steps.insert(next, direction.reverse());
                        queue.push_back((next, step + 1));
                    }
                }
            }

            if steps.len() % 1000 == 0 {
                println!(
                    "Building BFS step tree. Current {} items, depth {}",
                    steps.len(),
                    step
                );
            }
        }

        println!(
            "BFS step tree built. Total {} items, memory usage {} bytes",
            steps.len(),
            std::mem::size_of::<Puzzle>() * steps.len()
        );

        SolutionMap { goal, steps }
    }

    pub fn reconstruct_path(&self, current: Puzzle) -> Vec<Puzzle> {
        let mut path = Vec::new();
        let mut current_state = current;

        if !self.steps.contains_key(&current_state) {
            return path;
        }

        path.push(current_state);

        while let Some(dir) = self.steps.get(&current_state) {
            let next = current_state.move_zero(*dir).unwrap();
            path.push(next);
            current_state = next;

            if current_state == self.goal {
                break;
            }
        }

        path
    }

    pub fn goal(&self) -> &Puzzle {
        &self.goal
    }
}

// f = g + h

pub trait Heuristic {
    fn new() -> Self;
    fn estimate(&mut self, current: &Puzzle, goal: &Puzzle) -> i32;
}

pub trait SearchTree {
    fn new(goal: &Puzzle, initial: &Puzzle) -> Self;
    fn goal_reached(&self) -> bool;
    fn get(&self, key: &Puzzle) -> Option<(Puzzle, i32)>;
    fn set(&mut self, key: Puzzle, value: (Puzzle, i32));
    fn step_callback(&self, _current: &Puzzle, _next: (&Puzzle, bool)) {}
}

#[derive(Clone, Debug)]
struct BinaryHeapNode {
    puzzle: Puzzle,
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

pub fn solve_from_initial<S: SearchTree, H: Heuristic>(initial: Puzzle, goal: Puzzle) -> S {
    let mut open_set = BinaryHeap::new();
    let mut closed_set: S = S::new(&goal, &initial);
    let mut h_estimator = H::new();

    open_set.push(BinaryHeapNode {
        puzzle: initial,
        g: 0,
        h: 0,
    });
    closed_set.set(initial, (initial, 0));

    while let Some(current) = open_set.pop() {
        if current.puzzle == goal {
            break;
        }

        let current_g = closed_set.get(&current.puzzle).unwrap().1;

        for direction in Direction::all() {
            if let Some(next) = current.puzzle.move_zero(direction) {
                let g = current_g + 1;
                let h = h_estimator.estimate(&next, &goal);

                let update = match closed_set.get(&next) {
                    Some((_, prev_g)) => g < prev_g,
                    _ => true,
                };

                if update {
                    closed_set.set(next, (current.puzzle, g));
                    open_set.push(BinaryHeapNode { puzzle: next, g, h });
                }

                closed_set.step_callback(&current.puzzle, (&next, update));
            }
        }
    }

    closed_set
}
