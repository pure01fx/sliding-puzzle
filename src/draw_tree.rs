use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc};

use raylib::prelude::*;

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

impl IntRectBound {
    pub fn has_intersection_center_rect(
        &self,
        center: (i32, i32),
        width_x2: i32,
        height_x2: i32,
    ) -> bool {
        let left = center.0 - width_x2;
        let right = center.0 + width_x2;
        let top = center.1 - height_x2;
        let bottom = center.1 + height_x2;

        left <= self.right && right >= self.left && top <= self.bottom && bottom >= self.top
    }
}

pub struct DrawTreeNode {
    puzzle: Puzzle,
    depth: u32,
    children: Vec<RcRefDrawTreeNode>,
    center_x: i32,
    min_x: i32,
    max_x: i32,
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

    pub fn draw(
        &self,
        draw_handle: &mut RaylibDrawHandle,
        bound: &IntRectBound,
        offset: (i32, i32),
    ) {
        let inner = self.borrow();
        let center_x = inner.center_x;
        let depth = inner.depth;

        let x = offset.0 + center_x;
        let y = offset.1 + depth as i32 * (13 + 9) + 4;

        if bound.has_intersection_center_rect((x, y), 4, 4) {
            draw_small_puzzle(draw_handle, &inner.puzzle, SmallPuzzleCenter { x, y });
        }

        if !inner.children.is_empty() {
            // draw line down
            draw_handle.draw_rectangle(x, y + 8, 1, 3, raylib::color::Color::BLACK);
            let left_center_x = offset.0 + inner.children.first().unwrap().borrow().center_x;
            let right_center_x = offset.0 + inner.children.last().unwrap().borrow().center_x;
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
                    offset.0 + child.borrow().center_x,
                    y + 8 + 4,
                    1,
                    3,
                    raylib::color::Color::BLACK,
                );
                child.draw(draw_handle, bound, offset);
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
