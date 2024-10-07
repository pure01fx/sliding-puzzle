mod draw_tree;
mod logic;
pub mod name;
mod ui;

use std::{
    cell::Cell, collections::{hash_map, HashMap}, iter::Chain, marker::PhantomData, ops::{Deref, DerefMut}
};

use draw_tree::{ElementPainter, IntRectBound, IterableSearchTree, PuzzleSizer, RcRefDrawTreeNode};
use logic::{
    solve_from_initial, AStarHeuristic1, AStarHeuristic2, BfsHeuristic, Heuristic, OpenSet, Puzzle, SearchTree
};
use name::AUTHOR_NOTE;
use raylib::{prelude::*, rgui::RaylibDrawGui, rstr};
use ui::{
    elements::draw_puzzle,
    gif::{load_alice, ALICE_HEIGHT, ALICE_WIDTH},
    interactive_input::SetPuzzle,
};

pub trait AsMapSearchTree {
    fn goal(&self) -> &Puzzle;
    fn initial(&self) -> &Puzzle;
    fn map(&self) -> &HashMap<Puzzle, (Puzzle, i32)>;
    fn map_mut(&mut self) -> &mut HashMap<Puzzle, (Puzzle, i32)>;
    fn step_callback(&mut self, _: &Puzzle, _: (&Puzzle, bool), _: &OpenSet);

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
    fn step_callback(&mut self, _: &Puzzle, _: (&Puzzle, bool), _: &OpenSet) {}
}

struct AnimatedSearchTree<'handle> {
    goal: Puzzle,
    initial: Puzzle,
    map: HashMap<Puzzle, (Puzzle, i32)>,
    handle: &'handle mut RaylibHandle,
    thread: &'handle RaylibThread,
    max_nodes: Cell<usize>,
    alice: Vec<Texture2D>,
}

struct AnimatingSearchTree<'handle: 'draw, 'draw, 'data> {
    goal: &'data Puzzle,
    initial: &'data Puzzle,
    map: &'data HashMap<Puzzle, (Puzzle, i32)>,
    draw_handle: RaylibDrawHandle<'draw>,
    thread: PhantomData<&'handle RaylibThread>,
    alice: &'data Texture2D,
    max_nodes: &'data mut Cell<usize>,
}

impl<'handle: 'draw, 'draw, 'data> AnimatingSearchTree<'handle, 'draw, 'data> {
    fn from_animated_tree<'a: 'draw + 'data>(
        tree: &'a mut AnimatedSearchTree<'handle>,
        alice_id: usize,
    ) -> Self {
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
            thread: PhantomData,
            alice: &tree.alice[alice_id],
            max_nodes: &mut tree.max_nodes,
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
    fn step_callback(&mut self, _: &Puzzle, _: (&Puzzle, bool), _: &OpenSet) {}
}

struct MixedIterativeSearchTree<'a, 'b, 'c, 'd> {
    map_search_tree: &'d AnimatingSearchTree<'a, 'b, 'c>,
    open_set: &'d OpenSet,
}

impl<'a, 'b: 'a, 'c: 'a, 'd: 'a, 'e: 'a> IterableSearchTree<'a, Chain<hash_map::Iter<'a, Puzzle, (Puzzle, i32)>, hash_map::Iter<'a, Puzzle, (Puzzle, i32)>>> for MixedIterativeSearchTree<'b, 'c, 'd, 'e> {
    fn initial(&self) -> &Puzzle {
        self.map_search_tree.initial()
    }

    fn goal(&self) -> &Puzzle {
        self.map_search_tree.goal()
    }
    
    fn iter(&'a self) -> Chain<hash_map::Iter<'a, Puzzle, (Puzzle, i32)>, hash_map::Iter<'a, Puzzle, (Puzzle, i32)>> {
        self.map_search_tree.map().iter().chain(self.open_set.iter())
    }
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
    fn step_callback(&mut self, current: &Puzzle, _: (&Puzzle, bool), open_set: &OpenSet) {
        if self.map.len() > self.max_nodes.get() {
            return;
        }
        // draw alice in loop
        let total = 1024 + ALICE_WIDTH;
        let single = total / 50;
        let left = (single * (self.map.len() as u32 % 50)) as i32 - ALICE_WIDTH as i32;
        let alice_id = self.map.len() % self.alice.len();

        let mut animating = AnimatingSearchTree::from_animated_tree(self, alice_id);

        animating.draw_handle.draw_texture(
            animating.alice,
            left as i32,
            768 - ALICE_HEIGHT as i32,
            raylib::color::Color::WHITE,
        );

        if animating.draw_handle.gui_button(
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 20.0,
            },
            Some(rstr!("Skip animation")),
        ) {
            animating.max_nodes.set(0);
        }

        let mixed_iterative = MixedIterativeSearchTree {
            map_search_tree: &animating,
            open_set,
        };
        let (a, _) = RcRefDrawTreeNode::new_from_map_search_tree(&mixed_iterative, current);
        let mut painter = ElementPainter {
            draw_handle: &mut animating.draw_handle,
            bound: ANIM_BOUND,
            offset: (
                1024 / 2,
                ANIM_BOUND.top + 20,
            ),
            sizer: PuzzleSizer { scale: 3 },
        };
        a.draw(&mut painter);
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

    fn step_callback(&mut self, current: &Puzzle, next: (&Puzzle, bool), open_set: &OpenSet) {
        self.inner.step_callback(current, next, open_set);
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

const PLUS_BUTTON: Rectangle = Rectangle {
    x: 1024.0 - (10.0 + 24.0),
    y: 200.0 + 10.0,
    width: 24.0,
    height: 24.0,
};

const MINUS_BUTTON: Rectangle = Rectangle {
    x: 1024.0 - (10.0 + 24.0) * 2.0,
    y: 200.0 + 10.0,
    width: 24.0,
    height: 24.0,
};

const MAIN_BOUND: IntRectBound = IntRectBound {
    left: 0 + 10,
    top: 200,
    right: 1024 - 10,
    bottom: 800,
};

const ANIM_BOUND: IntRectBound = IntRectBound {
    left: 0 + 10,
    top: 10,
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
    let mut display_scale = 2;

    while !handle.window_should_close() {
        if handle.is_mouse_button_down(raylib::consts::MouseButton::MOUSE_BUTTON_LEFT) {
            if let Some(start) = start_pos {
                offset_xy = (
                    handle.get_mouse_x() - start.0 + offset_xy_old.0,
                    handle.get_mouse_y() - start.1 + offset_xy_old.1,
                );
            } else if handle.get_mouse_y() > 200 {
                start_pos = Some((handle.get_mouse_x(), handle.get_mouse_y()));
                offset_xy_old = offset_xy;
            }
        } else if start_pos.is_some() {
            start_pos = None;
        }

        let request_solve = {
            let mut draw_handle = handle.begin_drawing(&thread);
            draw_handle.clear_background(raylib::color::Color::WHITE);

            if show_result {
                if let Some(((solution, _), _)) = &solution_tree {
                    let mut painter = ElementPainter {
                        draw_handle: &mut draw_handle,
                        bound: MAIN_BOUND,
                        offset: (500 + offset_xy.0, 220 + offset_xy.1),
                        sizer: PuzzleSizer {
                            scale: display_scale,
                        },
                    };
                    solution.draw(&mut painter);
                }
            }

            draw_handle.draw_rectangle(0, 0, 1100, 200, raylib::color::Color::RAYWHITE);

            if animation_edit {
                draw_handle.gui_lock();
            }
            if draw_handle.gui_dropdown_box(
                STRATEGY_LIST,
                Some(rstr!("BFS;A* (1);A* (2)")),
                &mut selected_strategy,
                strategy_edit,
            ) {
                strategy_edit = !strategy_edit;
            }
            if animation_edit {
                draw_handle.gui_unlock();
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
                if let Some(((_, goal_node), count)) = &solution_tree {
                    draw_handle.draw_text(
                        &*format!(
                            "Solution found, {} nodes, take {} step(s).",
                            count,
                            goal_node.as_ref().unwrap().borrow().depth
                        ),
                        500,
                        52,
                        20,
                        raylib::color::Color::GREEN,
                    );
                } else {
                    draw_handle.draw_text(
                        "No solution found",
                        500,
                        52,
                        20,
                        raylib::color::Color::RED,
                    );
                }
            }

            draw_handle.draw_text(AUTHOR_NOTE, 500, 82, 20, raylib::color::Color::DARKCYAN);

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
                &mut display_scale,
            )
        };

        if request_solve {
            handle.set_target_fps((animate_fps_x5 * 5) as u32);

            let max_nodes = (150 * animate_fps_x5) as usize;

            if let Some(mut s) = {
                match selected_strategy {
                    1 => solve::<AStarHeuristic1>(initial, goal, &mut handle, &thread, max_nodes),
                    2 => solve::<AStarHeuristic2>(initial, goal, &mut handle, &thread, max_nodes),
                    _ => solve::<BfsHeuristic>(initial, goal, &mut handle, &thread, max_nodes),
                }
            } {
                let count = s.map.len();
                let s = s.as_map_search_tree();
                print_map_search_tree(&s);
                solution_tree = Some((s.into(), count));

                if let Some(((init_node, Some(goal_node)), _)) = &solution_tree {
                    init_node.build_coord(&PuzzleSizer {
                        scale: display_scale,
                    });
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
    display_scale: &mut i32,
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

    if draw_handle.gui_button(PLUS_BUTTON, Some(rstr!("+"))) {
        *display_scale += 1;
    }

    if *display_scale > 1 && draw_handle.gui_button(MINUS_BUTTON, Some(rstr!("-"))) {
        *display_scale -= 1;
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
    let alice = load_alice(handle, &thread);
    let mut tree = OwnedMapSearchTree {
        inner: AnimatedSearchTree {
            goal,
            initial,
            map: HashMap::new(),
            handle,
            thread,
            max_nodes: Cell::new(max_nodes),
            alice,
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
