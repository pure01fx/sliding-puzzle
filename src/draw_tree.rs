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
    pub depth: u32,
    children: Vec<RcRefDrawTreeNode>,
    pub center_x: i32,
    min_x: i32,
    max_x: i32,
    draw_x: Cell<i32>,
    visibility: Cell<Visibility>,
    on_path: Cell<bool>,
    coord_built_for: Cell<Option<PuzzleSizer>>,
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
            coord_built_for: Cell::new(None),
        })))
    }
}

impl RcRefDrawTreeNode {
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

    pub fn build_coord(&self, sizer: &PuzzleSizer) {
        if self.borrow().coord_built_for.get() == Some(*sizer) {
            return;
        }

        self.build_width_phase1(sizer);
        self.build_width_phase2(sizer);
        self.borrow().coord_built_for.set(Some(*sizer));
    }

    fn build_width_phase1(&self, sizer: &PuzzleSizer) {
        // The leaf nodes makes their min_x = -4 and max_x = 4
        // The parent nodes accumulate their children's min_x and max_x
        // and allocate 3px between each child.
        // The parent node's min_x and max_x are calculated
        // based on the width of the children.
        if self.borrow().children.is_empty() {
            self.borrow_mut().min_x = -sizer.puzzle_center_offset();
            self.borrow_mut().max_x = sizer.puzzle_center_offset();
        } else {
            let mut width = (self.borrow().children.len() - 1) as i32 * sizer.puzzle_cell();
            for child in self.borrow().children.iter() {
                child.build_width_phase1(sizer);
                width += child.borrow().max_x - child.borrow().min_x + 1;
            }

            let mut inner = self.borrow_mut();
            inner.min_x = -(width - 1) / 2;
            inner.max_x = (width - 1) / 2; // TODO
        }
    }

    fn build_width_phase2(&self, sizer: &PuzzleSizer) {
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

                left = new_max + sizer.puzzle_cell() + 1;
            }

            child.build_width_phase2(sizer);
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

    pub fn draw(&self, painter: &mut ElementPainter) {
        self.build_coord(&painter.sizer);
        self.update_visibility(&painter.bound, painter.offset);
        self.draw_phase_2(painter, false);
        self.draw_phase_3(painter);
    }

    pub fn draw_phase_2(&self, painter: &mut ElementPainter, fully_visible: bool) {
        if !fully_visible && self.borrow().visibility.get() == Visibility::None {
            return;
        }

        let fully_visible = fully_visible || self.borrow().visibility.get() == Visibility::Full;

        let inner = self.borrow();
        let center_x = inner.center_x;

        if let Some(y) = painter.get_draw_y(inner.depth) {
            let x = painter.get_draw_x(center_x);

            painter.draw_small_puzzle(&inner.puzzle, x, y, inner.on_path.get());

            inner.draw_x.set(x);
            inner.visibility.set(match fully_visible {
                true => Visibility::Full,
                false => Visibility::Partial,
            });

            if !inner.children.is_empty() {
                for child in inner.children.iter() {
                    child.draw_phase_2(painter, fully_visible);
                }
            }
        }
    }

    fn draw_phase_3(&self, painter: &mut ElementPainter) {
        if self.borrow().visibility.get() == Visibility::None {
            return;
        }

        let inner = self.borrow();
        let on_path = inner.on_path.get();

        if let Some(y) = painter.get_draw_y(inner.depth) {
            let x = inner.draw_x.get();

            if inner.depth != 0 {
                painter.draw_line_up(x, y, on_path);
            }

            if !inner.children.is_empty() {
                painter.draw_line_down(x, y, on_path);

                let left_x = inner.children.first().unwrap().borrow().draw_x.get();
                let right_x = inner.children.last().unwrap().borrow().draw_x.get();

                painter.draw_line_across(left_x, right_x, y);

                let mut child_on_path_id = -1;

                for (child, id) in inner.children.iter().zip(0..) {
                    child.draw_phase_3(painter);
                    if child.borrow().on_path.get() {
                        child_on_path_id = id as i32;
                    }
                }

                if on_path && child_on_path_id >= 0 {
                    let other_end = inner.children[child_on_path_id as usize]
                        .borrow()
                        .draw_x
                        .get();

                    painter.draw_line_across_on_path(x, y, other_end);
                }
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

        (root_node, goal_node)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PuzzleSizer {
    pub scale: i32,
}

impl PuzzleSizer {
    fn puzzle_cell(&self) -> i32 {
        2 * self.scale - 1
    }

    fn puzzle_center_offset(&self) -> i32 {
        self.puzzle_cell() * 3 / 2
    }
}

pub struct ElementPainter<'a, 'b> {
    pub draw_handle: &'b mut RaylibDrawHandle<'a>,
    pub bound: IntRectBound,
    pub offset: (i32, i32),
    pub sizer: PuzzleSizer,
}

impl Deref for ElementPainter<'_, '_> {
    type Target = PuzzleSizer;

    fn deref(&self) -> &Self::Target {
        &self.sizer
    }
}

impl ElementPainter<'_, '_> {
    fn get_draw_x(&self, center_x: i32) -> i32 {
        match center_x + self.offset.0 {
            x if x >= self.bound.left && x <= self.bound.right => x,
            _ => match center_x + self.offset.0 {
                x if x < self.bound.left => self.bound.left,
                _ => self.bound.right,
            },
        }
    }

    fn get_draw_y(&self, depth: u32) -> Option<i32> {
        match self.offset.1
            + depth as i32 * ((4 * self.puzzle_cell() + 1) + self.puzzle_cell() * 3)
            + self.puzzle_center_offset()
        {
            x if x > self.bound.bottom => None,
            x => Some(x),
        }
    }

    fn draw_small_puzzle(&mut self, puzzle: &Puzzle, x: i32, y: i32, on_path: bool) {
        draw_small_puzzle(
            self.draw_handle,
            puzzle,
            SmallPuzzleCenter {
                x,
                y,
                cell_size: self.puzzle_cell(),
            },
            on_path,
        );
    }

    fn draw_line_up(&mut self, x: i32, y: i32, on_path: bool) {
        self.draw_handle.draw_rectangle(
            x,
            y - self.puzzle_cell() * 2 - self.puzzle_center_offset(),
            1,
            self.puzzle_cell(),
            if on_path { Color::RED } else { Color::BLACK },
        );
    }

    fn draw_line_down(&mut self, x: i32, y: i32, on_path: bool) {
        self.draw_handle.draw_rectangle(
            x,
            y + 1 + self.puzzle_cell() + self.puzzle_center_offset(),
            1,
            self.puzzle_cell(),
            if on_path { Color::RED } else { Color::BLACK },
        );
    }

    fn draw_line_across(&mut self, left_x: i32, right_x: i32, y: i32) {
        let line_y = y + 1 + 2 * self.puzzle_cell() + self.puzzle_center_offset();

        self.draw_handle
            .draw_rectangle(left_x, line_y, right_x - left_x + 1, 1, Color::BLACK);
    }

    fn draw_line_across_on_path(&mut self, x: i32, y: i32, other_end: i32) {
        let line_y = y + 1 + 2 * self.puzzle_cell() + self.puzzle_center_offset();
        let (start_x, end_x) = match x < other_end {
            true => (x, other_end),
            false => (other_end, x),
        };

        self.draw_handle
            .draw_rectangle(start_x, line_y, end_x - start_x + 1, 1, Color::RED);
    }
}

impl<T: AsMapSearchTree> From<MapSearchTree<'_, T>>
    for (RcRefDrawTreeNode, Option<RcRefDrawTreeNode>)
{
    fn from(tree: MapSearchTree<T>) -> Self {
        RcRefDrawTreeNode::new_from_map_search_tree(&tree, tree.goal())
    }
}
