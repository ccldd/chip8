use std::path::Path;

use chip8::Chip8;
use macroquad::{
    color::{BLACK, BLUE, DARKGRAY, GREEN, RED, WHITE},
    input::KeyCode,
    shapes::{draw_line, draw_rectangle},
    text::draw_text,
    window::{clear_background, next_frame, request_new_screen_size, screen_width},
};
use tracing::{debug, info};
use tracing_subscriber::fmt::SubscriberBuilder;

mod chip8;

const SCALE: f32 = 10.0;
const PIXEL_COLOR: macroquad::color::Color = WHITE;

#[macroquad::main("Chip-8")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut chip8 = Chip8::new();
    chip8.load_rom(Path::new("roms/1-chip8-logo.ch8")).unwrap();
    info!("Loaded ROM {}", "roms/1-chip8-logo.ch8");

    let scale = 10.0;
    clear_background(BLACK);
    request_new_screen_size(
        chip8::display::WIDTH as f32 * scale,
        chip8::display::HEIGHT as f32 * scale,
    );

    loop {
        chip8.keypad.release();
        macroquad::input::get_keys_down()
            .iter()
            .take(1)
            .for_each(|keycode| {
                let key = map_keycode_to_chip8_key(*keycode);
                chip8.keypad.press(key);
            });

        let next_instruction = chip8.fetch();
        debug!("Next instruction: {:#06X}", next_instruction);

        chip8.execute(next_instruction);

        for y in 0..chip8::display::HEIGHT {
            for x in 0..chip8::display::WIDTH {
                let should_draw = chip8.display[x as usize][y as usize] == 1;
                let colour = if should_draw { PIXEL_COLOR } else { BLACK };
                draw_pixel(x, y, colour);
            }
        }

        next_frame().await
    }
}

fn draw_pixel(x: u32, y: u32, color: macroquad::color::Color) {
    let x = x as f32 * SCALE;
    let y = y as f32 * SCALE;
    draw_rectangle(x, y, SCALE, SCALE, color);
}

fn map_keycode_to_chip8_key(keycode: KeyCode) -> chip8::keypad::Key {
    match keycode {
        KeyCode::Key1 => chip8::keypad::Key::Key1,
        KeyCode::Key2 => chip8::keypad::Key::Key2,
        KeyCode::Key3 => chip8::keypad::Key::Key3,
        KeyCode::Key4 => chip8::keypad::Key::KeyC,
        KeyCode::Q => chip8::keypad::Key::Key4,
        KeyCode::W => chip8::keypad::Key::Key5,
        KeyCode::E => chip8::keypad::Key::Key6,
        KeyCode::R => chip8::keypad::Key::KeyD,
        KeyCode::A => chip8::keypad::Key::Key7,
        KeyCode::S => chip8::keypad::Key::Key8,
        KeyCode::D => chip8::keypad::Key::Key9,
        KeyCode::F => chip8::keypad::Key::KeyE,
        KeyCode::Z => chip8::keypad::Key::KeyA,
        KeyCode::X => chip8::keypad::Key::Key0,
        KeyCode::C => chip8::keypad::Key::KeyB,
        KeyCode::V => chip8::keypad::Key::KeyF,
        _ => chip8::keypad::Key::Key0,
    }
}
