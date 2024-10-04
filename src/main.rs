mod logic;

use std::collections::HashMap;

use logic::{solve_from_initial, Heuristic, Puzzle, SearchTree, SolutionMap};
use raylib::{prelude::*, rgui::RaylibDrawGui, rstr};

trait MapSearchTree {
    fn new(goal: &Puzzle, initial: &Puzzle) -> Self;
    fn initial(&self) -> &Puzzle;
    fn goal(&self) -> &Puzzle;
    fn map(&self) -> &HashMap<Puzzle, (Puzzle, i32)>;
    fn map_mut(&mut self) -> &mut HashMap<Puzzle, (Puzzle, i32)>;
}

fn print_map_search_tree(m: &impl MapSearchTree) {
    if m.goal_reached() {
        let mut vec = Vec::new();
        let mut current = m.goal().clone();

        while &current != m.initial() {
            if let Some((next, _)) = m.get(&current) {
                vec.push(current);
                current = next;
            }
        }

        vec.push(m.initial().clone());
        vec.reverse();

        for step in vec {
            println!("{}", step);
        }
    }

    println!("Total nodes: {}", m.map().len());
}

struct NativeSearchTree {
    goal: Puzzle,
    initial: Puzzle,
    map: HashMap<Puzzle, (Puzzle, i32)>,
}

impl MapSearchTree for NativeSearchTree {
    fn new(goal: &Puzzle, initial: &Puzzle) -> Self {
        NativeSearchTree {
            goal: goal.clone(),
            initial: initial.clone(),
            map: HashMap::new(),
        }
    }
    fn goal(&self) -> &Puzzle {
        &self.goal
    }
    fn initial(&self) -> &Puzzle {
        &self.initial
    }
    fn map(&self) -> &HashMap<Puzzle, (Puzzle, i32)> {
        &self.map
    }
    fn map_mut(&mut self) -> &mut HashMap<Puzzle, (Puzzle, i32)> {
        &mut self.map
    }
}

impl<T> SearchTree for T
where
    T: MapSearchTree,
{
    fn new(goal: &Puzzle, initial: &Puzzle) -> Self {
        Self::new(goal, initial)
    }

    fn goal_reached(&self) -> bool {
        self.map().contains_key(&self.goal())
    }

    fn get(&self, key: &Puzzle) -> Option<(Puzzle, i32)> {
        self.map().get(key).cloned()
    }

    fn set(&mut self, key: Puzzle, value: (Puzzle, i32)) {
        self.map_mut().insert(key, value);
    }
}

struct BfsHeuristic {}

impl Heuristic for BfsHeuristic {
    fn new() -> Self {
        BfsHeuristic {}
    }

    fn estimate(&mut self, _current: &Puzzle, _goal: &Puzzle) -> i32 {
        0
    }
}

struct SetGoal {
    current: u8,
    content: [u8; 9],
}

impl SetGoal {
    pub fn new() -> Self {
        SetGoal {
            current: 0,
            content: [9; 9], // 9 represents empty
        }
    }

    pub fn draw(&self, draw_handle: &mut RaylibDrawHandle, x: i32, y: i32) {
        for i in 0..3 {
            for j in 0..3 {
                if i * 3 + j == self.current as usize {
                    draw_handle.draw_rectangle_lines(
                        x + j as i32 * 30,
                        y + i as i32 * 30,
                        25,
                        25,
                        raylib::color::Color::RED,
                    );
                } else {
                    let content = self.content[i * 3 + j];
                    if content > 0 && content < 9 {
                        draw_sq_box(
                            draw_handle,
                            x + j as i32 * 30,
                            y + i as i32 * 30,
                            &format!("{}", content),
                        );
                    }
                }
            }
        }
    }

    fn set_value(&mut self, value: u8) -> bool {
        for i in 0..9 {
            if self.content[i] == value {
                return false;
            }
        }
        self.content[self.current as usize] = value;
        self.current = self.current + 1;
        true
    }

    pub fn read_event(&mut self, r: &RaylibHandle) {
        if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_ZERO) {
            self.set_value(0);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_ONE) {
            self.set_value(1);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_TWO) {
            self.set_value(2);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_THREE) {
            self.set_value(3);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_FOUR) {
            self.set_value(4);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_FIVE) {
            self.set_value(5);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_SIX) {
            self.set_value(6);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_SEVEN) {
            self.set_value(7);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_EIGHT) {
            self.set_value(8);
        }
    }

    pub fn get_puzzle(&self) -> Option<Puzzle> {
        if self.content.iter().all(|&x| x != 9) {
            Some(Puzzle::new([
                [self.content[0], self.content[1], self.content[2]],
                [self.content[3], self.content[4], self.content[5]],
                [self.content[6], self.content[7], self.content[8]],
            ]))
        } else {
            None
        }
    }
}

fn main() {
    let (mut handle, thread) = raylib::init().size(1024, 768).build();
    let mut goal = Puzzle::new([[1, 2, 3], [8, 0, 4], [7, 6, 5]]);
    let mut initial = Puzzle::from_random();
    let mut setting_goal = None;

    let mut show_result = false;
    let mut has_solution = false;

    handle.gui_enable();

    while !handle.window_should_close() {
        let mut draw_handle = handle.begin_drawing(&thread);
        draw_handle.clear_background(raylib::color::Color::RAYWHITE);
        draw_puzzle(&mut draw_handle, &initial, 200, 50);

        // interactive buttons

        if draw_handle.gui_button(
            Rectangle {
                x: 350.0,
                y: 50.0,
                width: 100.0,
                height: 24.0,
            },
            Some(rstr!("Set goal")),
        ) {
            setting_goal = Some(SetGoal::new());
            show_result = false;
        }

        if draw_handle.gui_button(
            Rectangle {
                x: 350.0,
                y: 110.0,
                width: 100.0,
                height: 24.0,
            },
            Some(rstr!("Solve")),
        ) {
            if let Some(s) = solve(initial, goal) {
                print_map_search_tree(&s);
                has_solution = true;
            } else {
                has_solution = false;
            }

            show_result = true;
        }

        if let Some(set_goal) = &mut setting_goal {
            set_goal.read_event(&draw_handle);
            set_goal.draw(&mut draw_handle, 50, 50);

            if let Some(puzzle) = set_goal.get_puzzle() {
                goal = puzzle;
                setting_goal = None;
            }
        } else {
            draw_puzzle(&mut draw_handle, &goal, 50, 50);
        }

        if draw_handle.gui_button(
            Rectangle {
                x: 350.0,
                y: 80.0,
                width: 100.0,
                height: 24.0,
            },
            Some(rstr!("Random init")),
        ) {
            initial = Puzzle::from_random();
            show_result = false;
        }

        // show result
        if show_result {
            if has_solution {
                draw_handle.draw_text("Solution found", 500, 50, 20, raylib::color::Color::GREEN);
            } else {
                draw_handle.draw_text("No solution found", 500, 50, 20, raylib::color::Color::RED);
            }
        }
    }
}

fn solve(initial: Puzzle, goal: Puzzle) -> Option<NativeSearchTree> {
    let bfs_solution = SolutionMap::new(goal);

    let goal_map = bfs_solution.reconstruct_path(initial);

    if goal_map.is_empty() {
        return None;
    }

    println!("Initial state:");
    println!("{}", initial);

    println!("Goal state:");
    println!("{}", bfs_solution.goal());

    for step in goal_map {
        println!("{}", step);
    }

    println!("\n----------------\n");

    let s = solve_from_initial::<NativeSearchTree, BfsHeuristic>(initial, goal);
    assert!(s.goal_reached());
    print_map_search_tree(&s);

    Some(s)
}

fn draw_sq_box(draw_handle: &mut RaylibDrawHandle, x: i32, y: i32, number: &str) {
    draw_handle.draw_rectangle(x, y, 25, 25, raylib::color::Color::BLACK);
    draw_handle.draw_text(
        number,
        x + 7 + (if number == "1" { 3 } else { 0 }),
        y + 4,
        20,
        raylib::color::Color::WHITE,
    );
}

fn draw_puzzle(draw_handle: &mut RaylibDrawHandle, puzzle: &Puzzle, x: i32, y: i32) {
    for i in 0..3 {
        for j in 0..3 {
            let s = format!("{}", puzzle.get_value(i, j));
            if s != "0" {
                draw_sq_box(draw_handle, x + j as i32 * 30, y + i as i32 * 30, &s);
            }
        }
    }
}
