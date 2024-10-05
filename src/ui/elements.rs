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
}

pub struct SmallPuzzleCenter {
    pub x: i32,
    pub y: i32,
}

impl PuzzleCoord for SmallPuzzleCenter {
    fn get_top_left(&self) -> (i32, i32) {
        (self.x - 4, self.y - 4)
    }
}

pub fn draw_small_puzzle(
    draw_handle: &mut RaylibDrawHandle,
    puzzle: &Puzzle,
    coord: impl PuzzleCoord,
) {
    let (x, y) = coord.get_top_left();
    for i in 0..3 {
        for j in 0..3 {
            let value = puzzle.get_value(i, j);
            if value != 0 {
                draw_handle.draw_rectangle(
                    x + j as i32 * 3,
                    y + i as i32 * 3,
                    3,
                    3,
                    map_color(&value.to_string()),
                );
            }
        }
    }
}
