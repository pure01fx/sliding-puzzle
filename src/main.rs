mod draw_tree;
mod logic;
mod ui;

use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use draw_tree::{IntRectBound, RcRefDrawTreeNode};
use logic::{solve_from_initial, AStarHeuristic, BfsHeuristic, Heuristic, Puzzle, SearchTree};
use raylib::{prelude::*, rgui::RaylibDrawGui, rstr};
use ui::{elements::draw_puzzle, interactive_input::SetPuzzle};

pub trait AsMapSearchTree {
    fn goal(&self) -> &Puzzle;
    fn initial(&self) -> &Puzzle;
    fn map(&self) -> &HashMap<Puzzle, (Puzzle, i32)>;
    fn map_mut(&mut self) -> &mut HashMap<Puzzle, (Puzzle, i32)>;
    fn step_callback(&mut self, _: &Puzzle, _: (&Puzzle, bool));

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

pub struct MapSearchTree<'a, T>
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
    fn step_callback(&mut self, _: &Puzzle, _: (&Puzzle, bool)) {}
}

struct AnimatedSearchTree<'handle> {
    goal: Puzzle,
    initial: Puzzle,
    map: HashMap<Puzzle, (Puzzle, i32)>,
    handle: &'handle mut RaylibHandle,
    thread: &'handle RaylibThread,
    max_nodes: usize,
}

struct AnimatingSearchTree<'handle: 'draw, 'draw, 'data> {
    goal: &'data Puzzle,
    initial: &'data Puzzle,
    map: &'data HashMap<Puzzle, (Puzzle, i32)>,
    draw_handle: RaylibDrawHandle<'draw>,
    thread: &'handle RaylibThread,
}

impl<'handle: 'draw, 'draw, 'data> AnimatingSearchTree<'handle, 'draw, 'data> {
    fn from_animated_tree<'a: 'draw + 'data>(tree: &'a mut AnimatedSearchTree<'handle>) -> Self {
        let AnimatedSearchTree {
            goal,
            initial,
            map,
            handle,
            thread,
            ..
        } = tree;
        let mut draw_handle = RaylibHandle::begin_drawing(handle, thread);
        draw_handle.clear_background(raylib::color::Color::WHITE);
        AnimatingSearchTree {
            goal,
            initial,
            map,
            draw_handle,
            thread,
        }
    }
}

impl AsMapSearchTree for AnimatingSearchTree<'_, '_, '_> {
    fn goal(&self) -> &Puzzle {
        self.goal
    }
    fn initial(&self) -> &Puzzle {
        self.initial
    }
    fn map(&self) -> &HashMap<Puzzle, (Puzzle, i32)> {
        self.map
    }
    fn map_mut(&mut self) -> &mut HashMap<Puzzle, (Puzzle, i32)> {
        panic!("AnimatingSearchTree is read-only");
    }
    fn step_callback(&mut self, _: &Puzzle, _: (&Puzzle, bool)) {}
}

impl<'a> AsMapSearchTree for AnimatedSearchTree<'a> {
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
    fn step_callback(&mut self, current: &Puzzle, _: (&Puzzle, bool)) {
        if self.map.len() > self.max_nodes {
            return;
        }
        let mut animating = AnimatingSearchTree::from_animated_tree(self);
        let map_search_tree = MapSearchTree {
            inner: &mut animating,
        };
        let (a, b) = RcRefDrawTreeNode::new_from_map_search_tree(&map_search_tree, current);
        a.draw(
            &mut animating.draw_handle,
            &MAIN_BOUND,
            (500 - b.map(|x| x.borrow().center_x).unwrap_or(0), 220),
        );
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

    fn step_callback(&mut self, current: &Puzzle, next: (&Puzzle, bool)) {
        self.inner.step_callback(current, next);
    }
}

const SET_GOAL_BUTTON: Rectangle = Rectangle {
    x: 50.0,
    y: 140.0,
    width: 85.0,
    height: 24.0,
};

const SET_INITIAL_BUTTON: Rectangle = Rectangle {
    x: 200.0,
    y: 140.0,
    width: 85.0,
    height: 24.0,
};

const SOLVE_BUTTON: Rectangle = Rectangle {
    x: 350.0,
    y: 50.0,
    width: 100.0,
    height: 24.0,
};

const RANDOM_INIT_BUTTON: Rectangle = Rectangle {
    x: 350.0,
    y: 80.0,
    width: 100.0,
    height: 24.0,
};

const FPS_LIST: Rectangle = Rectangle {
    x: 350.0,
    y: 110.0,
    width: 100.0,
    height: 24.0,
};

const STRATEGY_LIST: Rectangle = Rectangle {
    x: 350.0,
    y: 140.0,
    width: 100.0,
    height: 24.0,
};

const MAIN_BOUND: IntRectBound = IntRectBound {
    left: 0 + 10,
    top: 200,
    right: 1024 - 10,
    bottom: 800,
};

fn main() {
    let (mut handle, thread) = raylib::init().size(1024, 768).build();
    let mut goal = Puzzle::new([[1, 2, 3], [8, 0, 4], [7, 6, 5]]);
    let mut initial = Puzzle::new([[1, 3, 4], [8, 2, 5], [0, 7, 6]]);
    let mut setting_goal: Option<SetPuzzle> = None;
    let mut setting_initial: Option<SetPuzzle> = None;

    let mut show_result = false;
    let mut solution_tree: Option<((RcRefDrawTreeNode, Option<RcRefDrawTreeNode>), usize)> = None;

    let mut offset_xy = (0, 0);
    let mut offset_xy_old = (0, 0);
    let mut start_pos: Option<(i32, i32)> = None;

    handle.gui_enable();

    let mut selected_strategy = 0;
    let mut strategy_edit = false;
    let mut animate_fps_x5: i32 = 0;
    let mut animation_edit = false;

    while !handle.window_should_close() {
        if handle.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT)
            && handle.get_mouse_y() > 200
        {
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
                if let Some(((solution, _), _)) = &solution_tree {
                    solution.draw(
                        &mut draw_handle,
                        &MAIN_BOUND,
                        (500 + offset_xy.0, 220 + offset_xy.1),
                    );
                }
            }

            draw_handle.draw_rectangle(0, 0, 1100, 200, raylib::color::Color::RAYWHITE);

            if draw_handle.gui_dropdown_box(
                STRATEGY_LIST,
                Some(rstr!("BFS;A*")),
                &mut selected_strategy,
                strategy_edit,
            ) {
                strategy_edit = !strategy_edit;
            }

            if draw_handle.gui_dropdown_box(
                FPS_LIST,
                Some(rstr!("No animation;5 FPS;10 FPS;15 FPS;20 FPS")),
                &mut animate_fps_x5,
                animation_edit,
            ) {
                animation_edit = !animation_edit;
            }

            // show result
            if show_result {
                if let Some((_, count)) = solution_tree {
                    draw_handle.draw_text(
                        &*format!("Solution found, {} nodes", count),
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
            handle.set_target_fps((animate_fps_x5 * 5) as u32);

            let max_nodes = (150 * animate_fps_x5) as usize;

            if let Some(mut s) = {
                match selected_strategy {
                    1 => solve::<AStarHeuristic>(initial, goal, &mut handle, &thread, max_nodes),
                    _ => solve::<BfsHeuristic>(initial, goal, &mut handle, &thread, max_nodes),
                }
            } {
                let count = s.map.len();
                let s = s.as_map_search_tree();
                print_map_search_tree(&s);
                solution_tree = Some((s.into(), count));

                if let Some(((_, Some(goal_node)), _)) = &solution_tree {
                    offset_xy.0 = -goal_node.borrow().center_x;
                }
            } else {
                solution_tree = None;
            }
            handle.set_target_fps(60);

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

fn solve<'a, T: Heuristic>(
    initial: Puzzle,
    goal: Puzzle,
    handle: &'a mut RaylibHandle,
    thread: &'a RaylibThread,
    max_nodes: usize,
) -> Option<AnimatedSearchTree<'a>> {
    let mut tree = OwnedMapSearchTree {
        inner: AnimatedSearchTree {
            goal,
            initial,
            map: HashMap::new(),
            handle,
            thread,
            max_nodes,
        },
    };
    let mut tree_ref: MapSearchTree<'_, AnimatedSearchTree<'_>> = tree.make_ref();
    solve_from_initial::<_, T>(initial, goal, &mut tree_ref);
    match tree_ref.goal_reached() {
        true => {
            print_map_search_tree(&tree_ref);
            Some(tree.inner)
        }
        false => None,
    }
}
