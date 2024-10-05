mod logic;
mod ui;

use std::{cell::RefCell, collections::HashMap, iter::Map, ops::{Deref, DerefMut}, rc::Rc};

use logic::{solve_from_initial, Heuristic, Puzzle, SearchTree, SolutionMap};
use raylib::{prelude::*, rgui::RaylibDrawGui, rstr};
use ui::{
    elements::{draw_puzzle, draw_small_puzzle, SmallPuzzleCenter},
    interactive_input::SetPuzzle,
};

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
        MapSearchTree { inner: &mut self.inner }
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

impl<T: AsMapSearchTree> MapSearchTree<'_, T> {
    fn inner(&self) -> &T {
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

struct IntRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

struct DrawTreeNode {
    puzzle: Puzzle,
    depth: u32,
    children: Vec<RcRefDrawTreeNode>,
    center_x: i32,
    min_x: i32,
    max_x: i32,
}

#[derive(Clone)]
struct RcRefDrawTreeNode(Rc<RefCell<DrawTreeNode>>);

impl Deref for RcRefDrawTreeNode {
    type Target = Rc<RefCell<DrawTreeNode>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DrawTreeNode {
    fn new_rc_ref(puzzle: Puzzle) -> RcRefDrawTreeNode {
        RcRefDrawTreeNode(Rc::new(RefCell::new(DrawTreeNode {
            puzzle,
            depth: 0,
            children: Vec::new(),
            center_x: 0,
            min_x: 0,
            max_x: 0,
        })))
    }
}

impl RcRefDrawTreeNode {
    fn build_depth(&self, depth: u32) {
        self.borrow_mut().depth = depth;
        for child in self.borrow().children.iter() {
            assert!(child.borrow().puzzle != self.borrow().puzzle);
            child.build_depth(depth + 1);
        }
    }

    pub fn build_coord(&self) {
        self.build_width_phase1();
        self.build_width_phase2();
    }

    fn build_width_phase1(&self) {
        // The leaf nodes makes their min_x = -4 and max_x = 4
        // The parent nodes accumulate their children's min_x and max_x
        // and allocate 3px between each child.
        // The parent node's min_x and max_x are calculated
        // based on the width of the children.
        if self.borrow().children.is_empty() {
            self.borrow_mut().min_x = -4;
            self.borrow_mut().max_x = 4;
        } else {
            let mut width = (self.borrow().children.len() - 1) as i32 * 3;
            for child in self.borrow().children.iter() {
                child.build_width_phase1();
                width += child.borrow().max_x - child.borrow().min_x + 1;
            }

            let mut inner = self.borrow_mut();
            inner.min_x = -(width - 1) / 2;
            inner.max_x = (width - 1) / 2;
        }
    }

    fn build_width_phase2(&self) {
        // The parent node adjusts its children's center_x, min_x and max_x
        // based on the parent's center_x, min_x and max_x to make the parent
        // is at the center of its children.
        let mut left = self.borrow().min_x;

        for child in self.borrow().children.iter() {
            {
                let mut child = child.borrow_mut();
                child.center_x = left + (child.max_x - child.min_x) / 2;

                let new_max = child.center_x + (child.max_x - child.min_x) / 2;
                let new_min = child.center_x - (child.max_x - child.min_x) / 2;
                child.min_x = new_min;
                child.max_x = new_max;

                left = new_max + 3 + 1;
            }

            child.build_width_phase2();
        }
    }

    fn draw(&self, draw_handle: &mut RaylibDrawHandle, canvas: &IntRect) {
        let inner = self.borrow();
        let center_x = inner.center_x;
        let depth = inner.depth;

        let x = canvas.x + canvas.width / 2 + center_x;
        let y = canvas.y + depth as i32 * (13 + 9) + 4;

        if y - canvas.y > canvas.height {
            return;
        }

        draw_small_puzzle(draw_handle, &inner.puzzle, SmallPuzzleCenter { x, y });

        if !inner.children.is_empty() {
            // draw line down
            draw_handle.draw_rectangle(x, y + 8, 1, 3, raylib::color::Color::BLACK);
            // draw line across
            let left_center_x =
                canvas.x + canvas.width / 2 + inner.children.first().unwrap().borrow().center_x;
            let right_center_x =
                canvas.x + canvas.width / 2 + inner.children.last().unwrap().borrow().center_x;
            draw_handle.draw_rectangle(
                left_center_x,
                y + 8 + 3,
                right_center_x - left_center_x + 1,
                1,
                raylib::color::Color::BLACK,
            );

            for child in inner.children.iter() {
                // draw line down
                draw_handle.draw_rectangle(
                    canvas.x + canvas.width / 2 + child.borrow().center_x,
                    y + 8 + 4,
                    1,
                    3,
                    raylib::color::Color::BLACK,
                );
                child.draw(draw_handle, canvas);
            }
        }
    }
}

impl<T: AsMapSearchTree> From<MapSearchTree<'_, T>> for RcRefDrawTreeNode {
    fn from(tree: MapSearchTree<T>) -> Self {
        let root_node = DrawTreeNode::new_rc_ref(*tree.initial());
        let mut temp_nodes = HashMap::new();

        temp_nodes.insert(tree.initial(), root_node.clone());

        for (puzzle, (parent, _)) in tree.map() {
            if puzzle == parent {
                continue;
            }
            let puzzle_node = temp_nodes
                .entry(puzzle)
                .or_insert_with(|| DrawTreeNode::new_rc_ref(*puzzle))
                .clone();
            temp_nodes
                .entry(parent)
                .or_insert_with(|| DrawTreeNode::new_rc_ref(*parent))
                .borrow_mut()
                .children
                .push(puzzle_node.clone());
        }

        root_node.build_depth(0);
        root_node.build_coord();

        root_node
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
    let mut solution_tree = None;

    handle.gui_enable();

    while !handle.window_should_close() {
        let request_solve = {
            let mut draw_handle = handle.begin_drawing(&thread);
            draw_handle.clear_background(raylib::color::Color::RAYWHITE);

            // show result
            if show_result {
                result_draw(&solution_tree, &mut draw_handle);
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

fn result_draw(has_solution: &Option<RcRefDrawTreeNode>, draw_handle: &mut RaylibDrawHandle<'_>) {
    if let Some(solution) = has_solution {
        draw_handle.draw_text("Solution found", 500, 50, 20, raylib::color::Color::GREEN);
        solution.draw(
            draw_handle,
            &IntRect {
                x: 50,
                y: 350,
                width: 900,
                height: 400,
            },
        );
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
