use std::path::PathBuf;

use chip8::Chip8;
use clap::{command, Parser};
use macroquad::{
    color::{BLACK, WHITE},
    input::KeyCode,
    shapes::draw_rectangle,
    window::{clear_background, next_frame, request_new_screen_size},
};
use tracing::{debug, info};

mod chip8;

const SCALE: f32 = 10.0;
const PIXEL_COLOR: macroquad::color::Color = WHITE;
const TICKS_PER_SECOND: u8 = 10;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    rom: PathBuf,
}

#[macroquad::main("Chip-8")]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args = Args::parse();

    let mut chip8 = Chip8::new();
    chip8.load_rom(&args.rom).unwrap();
    info!("Loaded ROM {rom}", rom = "roms/1-chip8-logo.ch8");

    let scale = 10.0;
    clear_background(BLACK);
    request_new_screen_size(
        chip8::display::WIDTH as f32 * scale,
        chip8::display::HEIGHT as f32 * scale,
    );

    let mut ticks: u128 = 0;
    loop {
        for _ in 0..TICKS_PER_SECOND {
            chip8.keypad.release();
            macroquad::input::get_keys_down()
                .iter()
                .take(1)
                .for_each(|keycode| {
                    chip8.keypad.press((*keycode).into());
                });

            debug!("tick: {:#08X}, {:?}", ticks, chip8);
            chip8.tick();
            ticks += 1;
        }

        draw_display(&chip8);

        next_frame().await;

        chip8.decrement_delay_timer();
    }
}

fn draw_display(chip8: &Chip8) {
    for y in 0..chip8::display::HEIGHT {
        for x in 0..chip8::display::WIDTH {
            let is_pixel_on = chip8.display[x as usize][y as usize];
            let colour = if is_pixel_on { PIXEL_COLOR } else { BLACK };
            draw_pixel(x, y, colour);
        }
    }
}

fn draw_pixel(x: u8, y: u8, color: macroquad::color::Color) {
    let x = x as f32 * SCALE;
    let y = y as f32 * SCALE;
    draw_rectangle(x, y, SCALE, SCALE, color);
}

impl From<KeyCode> for chip8::keypad::Key {
    fn from(keycode: KeyCode) -> Self {
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
}
