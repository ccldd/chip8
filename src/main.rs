use std::path::PathBuf;

use chip8::{
    keypad::{Key, KeyState},
    Chip8,
};
use clap::{command, Parser};
use macroquad::{
    audio::{self, PlaySoundParams},
    color::{BLACK, WHITE},
    input::KeyCode,
    shapes::draw_rectangle,
    time,
    window::{next_frame, request_new_screen_size},
};
use strum::IntoEnumIterator;
use tracing::info;
use tracing::{debug, Level};

mod chip8;

const SCALE: f32 = 15.0;
const PIXEL_COLOR: macroquad::color::Color = WHITE;
const TICKS_PER_SECOND: u16 = 700;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    rom: PathBuf,
}

#[macroquad::main("Chip-8")]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .with_max_level(Level::DEBUG)
        .without_time()
        .init();

    let args = Args::parse();

    let mut chip8 = Chip8::new();
    chip8.load_rom(&args.rom).expect("error loading rom");
    info!("Loaded ROM {rom}", rom = args.rom.display());

    let beep = audio::load_sound("beep.wav")
        .await
        .expect("error loading sound");

    request_new_screen_size(
        chip8::display::WIDTH as f32 * SCALE,
        chip8::display::HEIGHT as f32 * SCALE,
    );

    let mut ticks: u128 = 0;
    loop {
        for _ in 0..TICKS_PER_SECOND {
            update_keypad(&mut chip8);

            // debug!(ticks, fps = time::get_fps(), ?chip8);
            chip8.tick();
            ticks += 1;
        }

        draw_display(&chip8);
        next_frame().await;

        chip8.tick_timers();
        if chip8.should_play_sound() {
            audio::play_sound(
                &beep,
                PlaySoundParams {
                    looped: false,
                    volume: 1.0,
                },
            );
        } else {
            audio::stop_sound(&beep);
        }
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

fn update_keypad(chip8: &mut Chip8) {
    for key in Key::iter() {
        if macroquad::input::is_key_down(key.into()) {
            chip8.keypad.set_key_state(key, KeyState::Down);
        } else {
            chip8.keypad.set_key_state(key, KeyState::Up);
        }
    }
}

impl From<Key> for KeyCode {
    fn from(value: Key) -> Self {
        match value {
            Key::Key1 => KeyCode::Key1,
            Key::Key2 => KeyCode::Key2,
            Key::Key3 => KeyCode::Key3,
            Key::KeyC => KeyCode::Key4,
            Key::Key4 => KeyCode::Q,
            Key::Key5 => KeyCode::W,
            Key::Key6 => KeyCode::E,
            Key::KeyD => KeyCode::R,
            Key::Key7 => KeyCode::A,
            Key::Key8 => KeyCode::S,
            Key::Key9 => KeyCode::D,
            Key::KeyE => KeyCode::F,
            Key::KeyA => KeyCode::Z,
            Key::Key0 => KeyCode::X,
            Key::KeyB => KeyCode::C,
            Key::KeyF => KeyCode::V,
        }
    }
}
