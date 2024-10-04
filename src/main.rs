mod logic;
mod ui;

use std::collections::HashMap;

use logic::{solve_from_initial, Heuristic, Puzzle, SearchTree, SolutionMap};
use raylib::{prelude::*, rgui::RaylibDrawGui, rstr};
use ui::{elements::draw_puzzle, interactive_input::SetGoal};

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

const SET_GOAL_BUTTON: Rectangle = Rectangle {
    x: 350.0,
    y: 50.0,
    width: 100.0,
    height: 24.0,
};

const SOLVE_BUTTON: Rectangle = Rectangle {
    x: 350.0,
    y: 80.0,
    width: 100.0,
    height: 24.0,
};

const RANDOM_INIT_BUTTON: Rectangle = Rectangle {
    x: 350.0,
    y: 110.0,
    width: 100.0,
    height: 24.0,
};

fn main() {
    let (mut handle, thread) = raylib::init().size(1024, 768).build();
    let mut goal = Puzzle::new([[1, 2, 3], [8, 0, 4], [7, 6, 5]]);
    let mut initial = Puzzle::from_random();
    let mut setting_goal: Option<SetGoal> = None;

    let mut show_result = false;
    let mut has_solution = false;

    handle.gui_enable();

    while !handle.window_should_close() {
        let request_solve = {
            let mut draw_handle = handle.begin_drawing(&thread);
            draw_handle.clear_background(raylib::color::Color::RAYWHITE);
            draw_puzzle(&mut draw_handle, &initial, 200, 50);

            // show result
            if show_result {
                result_draw(has_solution, &mut draw_handle);
            }

            // interactive buttons
            button_draw(
                &mut setting_goal,
                draw_handle,
                &mut goal,
                &mut initial,
                &mut show_result,
                &mut has_solution,
            )
        };

        if request_solve {
            if let Some(s) = solve(initial, goal) {
                print_map_search_tree(&s);
                has_solution = true;
            } else {
                has_solution = false;
            }

            show_result = true;
        }
    }
}

fn button_draw(
    setting_goal: &mut Option<SetGoal>,
    mut draw_handle: RaylibDrawHandle<'_>,
    goal: &mut Puzzle,
    initial: &mut Puzzle,
    show_result: &mut bool,
    has_solution: &mut bool,
) -> bool {
    if let Some(set_goal) = setting_goal {
        set_goal.read_event(&draw_handle);
        set_goal.draw(&mut draw_handle, 50, 50);

        if let Some(puzzle) = set_goal.get_puzzle() {
            *goal = puzzle;
            *setting_goal = None;
        }
    } else {
        draw_puzzle(&mut draw_handle, &*goal, 50, 50);
    }

    if draw_handle.gui_button(RANDOM_INIT_BUTTON, Some(rstr!("Random init"))) {
        *initial = Puzzle::from_random();
        *show_result = false;
    }

    if draw_handle.gui_button(SET_GOAL_BUTTON, Some(rstr!("Set goal"))) {
        *setting_goal = Some(SetGoal::new());
        *show_result = false;
    }

    setting_goal.is_none() && draw_handle.gui_button(SOLVE_BUTTON, Some(rstr!("Solve")))
}

fn result_draw(has_solution: bool, draw_handle: &mut RaylibDrawHandle<'_>) {
    if has_solution {
        draw_handle.draw_text("Solution found", 500, 50, 20, raylib::color::Color::GREEN);
    } else {
        draw_handle.draw_text("No solution found", 500, 50, 20, raylib::color::Color::RED);
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
