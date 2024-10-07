use raylib::prelude::*;

use crate::logic::Puzzle;

pub fn map_color(num: &str) -> Color {
    match num {
        "1" => Color::DARKRED,
        "2" => Color::DARKBLUE,
        "3" => Color::DARKGREEN,
        "4" => Color::DARKCYAN,
        "5" => Color::DARKORANGE,
        "6" => Color::DARKPURPLE,
        "7" => Color::DARKBROWN,
        "8" => Color::DARKGOLDENROD,
        _ => Color::BLACK,
    }
}

pub fn draw_sq_box(draw_handle: &mut RaylibDrawHandle, x: i32, y: i32, number: &str) {
    draw_handle.draw_rectangle(x, y, 25, 25, map_color(number));
    draw_handle.draw_text(
        number,
        x + 7 + (if number == "1" { 3 } else { 0 }),
        y + 4,
        20,
        raylib::color::Color::WHITE,
    );
}

pub fn draw_puzzle(draw_handle: &mut RaylibDrawHandle, puzzle: &Puzzle, x: i32, y: i32) {
    for i in 0..3 {
        for j in 0..3 {
            let s = format!("{}", puzzle.get_value(i, j));
            if s != "0" {
                draw_sq_box(draw_handle, x + j as i32 * 30, y + i as i32 * 30, &s);
            }
        }
    }
}

pub trait PuzzleCoord {
    fn get_top_left(&self) -> (i32, i32);
    fn get_cell_size(&self) -> i32;
}

pub struct SmallPuzzleCenter {
    pub x: i32,
    pub y: i32,
    pub cell_size: i32,
}

impl PuzzleCoord for SmallPuzzleCenter {
    fn get_top_left(&self) -> (i32, i32) {
        (
            self.x - self.cell_size * 3 / 2,
            self.y - self.cell_size * 3 / 2,
        )
    }

    fn get_cell_size(&self) -> i32 {
        self.cell_size
    }
}

pub fn draw_small_puzzle(
    draw_handle: &mut RaylibDrawHandle,
    puzzle: &Puzzle,
    coord: impl PuzzleCoord,
    with_border: bool,
) {
    let (x, y) = coord.get_top_left();
    let cell_size = coord.get_cell_size();
    for i in 0..3 {
        for j in 0..3 {
            let value = puzzle.get_value(i, j);
            if value != 0 {
                draw_handle.draw_rectangle(
                    x + j as i32 * cell_size,
                    y + i as i32 * cell_size,
                    cell_size,
                    cell_size,
                    map_color(&value.to_string()),
                );
            }
        }
    }

    if with_border {
        draw_handle.draw_rectangle_lines(x - 2, y - 2, 4 + cell_size * 3, 4 + cell_size * 3, Color::RED);
    } else {
        draw_handle.draw_rectangle_lines(x - 1, y - 1, 2 + cell_size * 3, 2 + cell_size * 3, Color::BLACK);
    }
}
