use raylib::prelude::*;

use crate::logic::Puzzle;

use super::elements::draw_sq_box;

pub struct SetPuzzle {
    current: u8,
    content: [u8; 9],
}

impl SetPuzzle {
    pub fn new() -> Self {
        SetPuzzle {
            current: 0,
            content: [9; 9], // 9 represents empty
        }
    }

    pub fn draw(&self, draw_handle: &mut RaylibDrawHandle, x: i32, y: i32) {
        for i in 0..3 {
            for j in 0..3 {
                if i * 3 + j == self.current as usize {
                    draw_handle.draw_rectangle_lines(
                        x + j as i32 * 30,
                        y + i as i32 * 30,
                        25,
                        25,
                        raylib::color::Color::RED,
                    );
                } else {
                    let content = self.content[i * 3 + j];
                    if content > 0 && content < 9 {
                        draw_sq_box(
                            draw_handle,
                            x + j as i32 * 30,
                            y + i as i32 * 30,
                            &format!("{}", content),
                        );
                    }
                }
            }
        }
    }

    fn set_value(&mut self, value: u8) -> bool {
        for i in 0..9 {
            if self.content[i] == value {
                return false;
            }
        }
        self.content[self.current as usize] = value;
        self.current = self.current + 1;
        true
    }

    pub fn read_event(&mut self, r: &RaylibHandle) {
        if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_ZERO) {
            self.set_value(0);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_ONE) {
            self.set_value(1);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_TWO) {
            self.set_value(2);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_THREE) {
            self.set_value(3);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_FOUR) {
            self.set_value(4);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_FIVE) {
            self.set_value(5);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_SIX) {
            self.set_value(6);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_SEVEN) {
            self.set_value(7);
        } else if r.is_key_pressed(raylib::consts::KeyboardKey::KEY_EIGHT) {
            self.set_value(8);
        }
    }

    pub fn get_puzzle(&self) -> Option<Puzzle> {
        if self.content.iter().all(|&x| x != 9) {
            Some(Puzzle::new([
                [self.content[0], self.content[1], self.content[2]],
                [self.content[3], self.content[4], self.content[5]],
                [self.content[6], self.content[7], self.content[8]],
            ]))
        } else {
            None
        }
    }
}
