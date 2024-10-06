use raylib::{texture::{Image, Texture2D}, RaylibHandle, RaylibThread};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "assets/alice"]
struct Asset;

pub const ALICE_WIDTH: u32 = 640 / 4;
pub const ALICE_HEIGHT: u32 = 650 / 4;

pub fn load_alice(handle: &mut RaylibHandle, thread: &RaylibThread) -> Vec<Texture2D> {
    let mut frames = Vec::new();
    for i in 0.. {
        let frame = Asset::get(&*format!("{}.png", i));
        if let Some(frame) = frame {
            let mut frame = Image::load_image_from_mem(".png", &frame.data).unwrap();
            frame.resize(ALICE_HEIGHT as i32, ALICE_WIDTH as i32);
            frames.push(handle.load_texture_from_image(thread, &frame).unwrap());
        } else {
            break;
        }
    }
    frames
}
