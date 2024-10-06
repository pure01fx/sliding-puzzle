mod draw_tree;
mod logic;
mod ui;

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use draw_tree::{IntRectBound, RcRefDrawTreeNode};
use logic::{bfs::BfsHeuristic, solve_from_initial, Puzzle, SearchTree, SolutionMap};
use raylib::{prelude::*, rgui::RaylibDrawGui, rstr};
use ui::{elements::draw_puzzle, interactive_input::SetPuzzle};

trait AsMapSearchTree {
    fn goal(&self) -> &Puzzle;
    fn initial(&self) -> &Puzzle;
    fn map(&self) -> &HashMap<Puzzle, (Puzzle, i32)>;
    fn map_mut(&mut self) -> &mut HashMap<Puzzle, (Puzzle, i32)>;

    fn as_map_search_tree(&mut self) -> MapSearchTree<'_, Self>
    where
        Self: Sized,
    {
        MapSearchTree { inner: self }
    }
}

struct OwnedMapSearchTree<T>
where
    T: AsMapSearchTree,
{
    inner: T,
}

impl<T: AsMapSearchTree> OwnedMapSearchTree<T> {
    pub fn make_ref<'a>(&'a mut self) -> MapSearchTree<'a, T> {
        MapSearchTree {
            inner: &mut self.inner,
        }
    }
}

struct MapSearchTree<'a, T>
where
    T: AsMapSearchTree,
{
    inner: &'a mut T,
}

impl<T: AsMapSearchTree> Deref for MapSearchTree<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T: AsMapSearchTree> DerefMut for MapSearchTree<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

fn print_map_search_tree<T: AsMapSearchTree>(m: &MapSearchTree<T>) {
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

impl AsMapSearchTree for NativeSearchTree {
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

impl<T: AsMapSearchTree> SearchTree for MapSearchTree<'_, T> {
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

const SET_GOAL_BUTTON: Rectangle = Rectangle {
    x: 50.0,
    y: 150.0,
    width: 85.0,
    height: 24.0,
};

const SET_INITIAL_BUTTON: Rectangle = Rectangle {
    x: 200.0,
    y: 150.0,
    width: 85.0,
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
    let mut initial = Puzzle::new([[1, 3, 0], [8, 2, 4], [7, 6, 5]]);
    let mut setting_goal: Option<SetPuzzle> = None;
    let mut setting_initial: Option<SetPuzzle> = None;

    let mut show_result = false;
    let mut solution_tree: Option<RcRefDrawTreeNode> = None;

    let mut offset_xy = (0, 0);
    let mut offset_xy_old = (0, 0);
    let mut start_pos: Option<(i32, i32)> = None;

    handle.gui_enable();

    while !handle.window_should_close() {
        if handle.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(start) = start_pos {
                offset_xy = (
                    handle.get_mouse_x() - start.0 + offset_xy_old.0,
                    handle.get_mouse_y() - start.1 + offset_xy_old.1,
                );
            } else {
                start_pos = Some((handle.get_mouse_x(), handle.get_mouse_y()));
            }
        } else if start_pos.is_some() {
            offset_xy_old = offset_xy;
            start_pos = None;
        }

        let request_solve = {
            let mut draw_handle = handle.begin_drawing(&thread);
            draw_handle.clear_background(raylib::color::Color::WHITE);

            if show_result {
                if let Some(solution) = &solution_tree {
                    solution.draw(
                        &mut draw_handle,
                        &IntRectBound {
                            left: 0,
                            top: 200,
                            right: 1100,
                            bottom: 800,
                        },
                        (500 + offset_xy.0, 220 + offset_xy.1),
                    );
                }
            }

            draw_handle.draw_rectangle(0, 0, 1100, 200, raylib::color::Color::RAYWHITE);

            // show result
            if show_result {
                if let Some(_) = solution_tree {
                    draw_handle.draw_text(
                        "Solution found",
                        500,
                        50,
                        20,
                        raylib::color::Color::GREEN,
                    );
                } else {
                    draw_handle.draw_text(
                        "No solution found",
                        500,
                        50,
                        20,
                        raylib::color::Color::RED,
                    );
                }
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

            if let Some(set_initial) = &mut setting_initial {
                set_initial.read_event(&draw_handle);
                set_initial.draw(&mut draw_handle, 200, 50);

                if let Some(puzzle) = set_initial.get_puzzle() {
                    initial = puzzle;
                    setting_initial = None;
                }
            } else {
                draw_puzzle(&mut draw_handle, &initial, 200, 50);
            }

            // interactive buttons
            button_draw(
                &mut setting_goal,
                &mut setting_initial,
                draw_handle,
                &mut initial,
                &mut show_result,
            )
        };

        if request_solve {
            if let Some(mut s) = solve(initial, goal) {
                let s = s.as_map_search_tree();
                print_map_search_tree(&s);
                solution_tree = Some(s.into());
            } else {
                solution_tree = None;
            }

            show_result = true;
        }
    }
}

fn button_draw(
    setting_goal: &mut Option<SetPuzzle>,
    setting_initial: &mut Option<SetPuzzle>,
    mut draw_handle: RaylibDrawHandle<'_>,
    initial: &mut Puzzle,
    show_result: &mut bool,
) -> bool {
    if draw_handle.gui_button(RANDOM_INIT_BUTTON, Some(rstr!("Random init"))) {
        *initial = Puzzle::from_random();
        *show_result = false;
    }

    if draw_handle.gui_button(SET_GOAL_BUTTON, Some(rstr!("Set goal"))) {
        *setting_goal = Some(SetPuzzle::new());
        *show_result = false;
    }

    if draw_handle.gui_button(SET_INITIAL_BUTTON, Some(rstr!("Set init"))) {
        *setting_initial = Some(SetPuzzle::new());
        *show_result = false;
    }

    setting_goal.is_none()
        && setting_initial.is_none()
        && draw_handle.gui_button(SOLVE_BUTTON, Some(rstr!("Solve")))
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

    let mut tree = OwnedMapSearchTree {
        inner: NativeSearchTree {
            goal,
            initial,
            map: HashMap::new(),
        },
    };
    let mut tree_ref = tree.make_ref();
    solve_from_initial::<_, BfsHeuristic>(initial, goal, &mut tree_ref);
    assert!(tree_ref.goal_reached());
    print_map_search_tree(&tree_ref);

    Some(tree.inner)
}
