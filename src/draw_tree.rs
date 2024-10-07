use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ops::Deref,
    rc::Rc,
};

use raylib::{color::Color, prelude::*};

use crate::{
    logic::Puzzle,
    ui::elements::{draw_small_puzzle, SmallPuzzleCenter},
    AsMapSearchTree, MapSearchTree,
};

pub struct IntRectBound {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Visibility {
    Full,
    Partial,
    None,
}

pub struct DrawTreeNode {
    puzzle: Puzzle,
    depth: u32,
    children: Vec<RcRefDrawTreeNode>,
    pub center_x: i32,
    min_x: i32,
    max_x: i32,
    draw_x: Cell<i32>,
    visibility: Cell<Visibility>,
    on_path: Cell<bool>,
}

#[derive(Clone)]
pub struct RcRefDrawTreeNode(Rc<RefCell<DrawTreeNode>>);

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
            draw_x: Cell::new(0),
            visibility: Cell::new(Visibility::None),
            on_path: Cell::new(false),
        })))
    }
}

impl RcRefDrawTreeNode {
    const SCALE: i32 = 3;

    const PUZZLE_CELL: i32 = 2 * Self::SCALE + 1;
    const PUZZLE_CENTER_OFFSET: i32 = Self::PUZZLE_CELL * 3 / 2;

    fn build_depth(&self, depth: u32, path_end: &Puzzle) -> (bool, Option<Self>) {
        let xx = self
            .borrow()
            .children
            .iter()
            .map(|child| child.build_depth(depth + 1, path_end))
            .fold((false, None), |acc, x| match x {
                (_, node @ Some(_)) => (true, node),
                (x, _) => (acc.0 || x, acc.1),
            });

        self.borrow_mut().depth = depth;

        if self.borrow().puzzle == *path_end {
            self.borrow_mut().on_path.set(true);
            (true, Some(self.clone()))
        } else {
            self.borrow_mut().on_path.set(xx.0);
            xx
        }
    }

    fn build_coord(&self) {
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
            self.borrow_mut().min_x = -Self::PUZZLE_CENTER_OFFSET;
            self.borrow_mut().max_x = Self::PUZZLE_CENTER_OFFSET;
        } else {
            let mut width = (self.borrow().children.len() - 1) as i32 * Self::PUZZLE_CELL;
            for child in self.borrow().children.iter() {
                child.build_width_phase1();
                width += child.borrow().max_x - child.borrow().min_x + 1;
            }

            let mut inner = self.borrow_mut();
            inner.min_x = -(width - 1) / 2;
            inner.max_x = (width - 1) / 2; // TODO
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

                left = new_max + Self::PUZZLE_CELL + 1;
            }

            child.build_width_phase2();
        }
    }

    fn update_visibility(&self, bound: &IntRectBound, offset: (i32, i32)) {
        let left = offset.0 + self.borrow().min_x;
        let right = offset.0 + self.borrow().max_x;

        if bound.left <= left && right <= bound.right {
            self.borrow().visibility.set(Visibility::Full);
        } else if bound.left <= right && left <= bound.right {
            self.borrow().visibility.set(Visibility::Partial);
            for child in self.borrow().children.iter() {
                child.update_visibility(bound, offset);
            }
        } else {
            self.borrow().visibility.set(Visibility::None);

            if left > bound.right {
                self.borrow().draw_x.set(bound.right + 20);
            } else {
                self.borrow().draw_x.set(bound.left - 20);
            }
        }
    }

    pub fn draw(
        &self,
        draw_handle: &mut RaylibDrawHandle,
        bound: &IntRectBound,
        offset: (i32, i32),
    ) {
        self.update_visibility(bound, offset);
        self.draw_phase_2(draw_handle, bound, offset, false);
        self.draw_phase_3(draw_handle, bound, offset);
    }

    pub fn draw_phase_2(
        &self,
        draw_handle: &mut RaylibDrawHandle,
        bound: &IntRectBound,
        offset: (i32, i32),
        fully_visible: bool,
    ) {
        if !fully_visible && self.borrow().visibility.get() == Visibility::None {
            return;
        }

        let fully_visible = fully_visible || self.borrow().visibility.get() == Visibility::Full;

        let inner = self.borrow();
        let center_x = inner.center_x;

        let y = offset.1
            + inner.depth as i32 * ((4 * Self::PUZZLE_CELL + 1) + Self::PUZZLE_CELL * 3)
            + Self::PUZZLE_CENTER_OFFSET;

        if y > bound.bottom {
            return;
        }

        let x = match fully_visible {
            true => center_x + offset.0,
            false => match center_x + offset.0 {
                x if x >= bound.left && x <= bound.right => x,
                _ => match center_x + offset.0 {
                    x if x < bound.left => bound.left,
                    _ => bound.right,
                },
            },
        };

        inner.draw_x.set(x);
        inner.visibility.set(match fully_visible {
            true => Visibility::Full,
            false => Visibility::Partial,
        });
        draw_small_puzzle(
            draw_handle,
            &inner.puzzle,
            SmallPuzzleCenter {
                x,
                y,
                cell_size: Self::PUZZLE_CELL,
            },
            inner.on_path.get(),
        );

        if !inner.children.is_empty() {
            for child in inner.children.iter() {
                child.draw_phase_2(draw_handle, bound, offset, fully_visible);
            }
        }
    }

    fn draw_phase_3(
        &self,
        draw_handle: &mut RaylibDrawHandle,
        bound: &IntRectBound,
        offset: (i32, i32),
    ) {
        if self.borrow().visibility.get() == Visibility::None {
            return;
        }

        let inner = self.borrow();
        let color = if inner.on_path.get() {
            Color::RED
        } else {
            Color::BLACK
        };
        let x = inner.draw_x.get();
        let y = offset.1
            + inner.depth as i32 * ((4 * Self::PUZZLE_CELL + 1) + Self::PUZZLE_CELL * 3)
            + Self::PUZZLE_CENTER_OFFSET;

        if y > bound.bottom {
            return;
        }

        if inner.depth != 0 {
            // draw line up
            draw_handle.draw_rectangle(
                x,
                y - Self::PUZZLE_CELL * 2 - Self::PUZZLE_CENTER_OFFSET,
                1,
                Self::PUZZLE_CELL,
                color,
            );
        }

        if !inner.children.is_empty() {
            // draw line down
            draw_handle.draw_rectangle(
                x,
                y + 1 + Self::PUZZLE_CELL + Self::PUZZLE_CENTER_OFFSET,
                1,
                Self::PUZZLE_CELL,
                color,
            );

            // draw line across
            let left_x = inner.children.first().unwrap().borrow().draw_x.get();
            let right_x = inner.children.last().unwrap().borrow().draw_x.get();
            let line_y = y + 1 + Self::PUZZLE_CELL + Self::PUZZLE_CENTER_OFFSET;

            draw_handle.draw_rectangle(left_x, line_y, right_x - left_x + 1, 1, Color::BLACK);

            let mut child_on_path_id = -1;

            for (child, id) in inner.children.iter().zip(0..) {
                child.draw_phase_3(draw_handle, bound, offset);
                if child.borrow().on_path.get() {
                    child_on_path_id = id as i32;
                }
            }

            if inner.on_path.get() && child_on_path_id >= 0 {
                let other_end = inner.children[child_on_path_id as usize]
                    .borrow()
                    .draw_x
                    .get();

                let (start_x, end_x) = match x < other_end {
                    true => (x, other_end),
                    false => (other_end, x),
                };

                draw_handle.draw_rectangle(start_x, line_y, end_x - start_x + 1, 1, Color::RED);
            }
        }
    }

    pub fn new_from_map_search_tree<T: AsMapSearchTree>(
        tree: &MapSearchTree<'_, T>,
        path_end: &Puzzle,
    ) -> (RcRefDrawTreeNode, Option<RcRefDrawTreeNode>) {
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

        let (_, goal_node) = root_node.build_depth(0, path_end);
        root_node.build_coord();

        (root_node, goal_node)
    }
}

impl<T: AsMapSearchTree> From<MapSearchTree<'_, T>>
    for (RcRefDrawTreeNode, Option<RcRefDrawTreeNode>)
{
    fn from(tree: MapSearchTree<T>) -> Self {
        RcRefDrawTreeNode::new_from_map_search_tree(&tree, tree.goal())
    }
}
