use chip8::Chip8;
use macroquad::{
    color::{BLACK, BLUE, DARKGRAY, GREEN, RED, WHITE},
    shapes::{draw_line, draw_rectangle},
    text::draw_text,
    window::{clear_background, next_frame, request_new_screen_size, screen_width},
};

mod chip8;

const SCALE: f32 = 10.0;
const PIXEL_COLOR: macroquad::color::Color = WHITE;

#[macroquad::main("Chip-8")]
async fn main() {
    let mut chip8 = Chip8::new();

    let scale = 10.0;
    clear_background(BLACK);
    request_new_screen_size(
        chip8::display::WIDTH as f32 * scale,
        chip8::display::HEIGHT as f32 * scale,
    );

    let mut x = 0;
    let mut y = 0;

    loop {
        let next_instruction = chip8.fetch();
        chip8.execute(next_instruction);

        next_frame().await
    }
}

fn draw_pixel(x: u32, y: u32, color: macroquad::color::Color) {
    let x = x as f32 * SCALE;
    let y = y as f32 * SCALE;
    draw_rectangle(x, y, SCALE, SCALE, color);
}
