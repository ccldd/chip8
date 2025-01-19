use core::fmt;
use std::{error::Error, fmt::{Debug, Formatter}, fs::File, io::Read, path::Path};

use keypad::KeyPad;
use tracing::error;

pub mod display;
mod font;
pub mod keypad;

const INITIAL_PC: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;
const MAX_ROM_SIZE: usize = MEMORY_SIZE - INITIAL_PC as usize;

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    pub display: [[bool; display::HEIGHT as usize]; display::WIDTH as usize],
    pc: u16,

    #[allow(non_snake_case)]
    I: u16,

    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,

    #[allow(non_snake_case)]
    V: [u8; 16], // registers

    pub keypad: KeyPad,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            memory: [0; 4096],
            display: [[false; display::HEIGHT as usize]; display::WIDTH as usize],
            pc: INITIAL_PC,
            I: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            V: [0; 16],
            keypad: KeyPad::new(),
        };

        font::load_fonts(&mut c.memory);

        c
    }

    pub fn load_rom(&mut self, file: &Path) -> Result<(), Box<dyn Error>> {
        let metadata = std::fs::metadata(file)?;
        let file_size = metadata.len() as usize;

        if file_size > MAX_ROM_SIZE {
            return Err("ROM too large".into());
        }

        let mut file = File::open(file)?;
        let mut buf = vec![0; file_size];
        file.read_exact(&mut buf)?;

        self.memory[(INITIAL_PC as usize)..(INITIAL_PC as usize + file_size)].copy_from_slice(&buf);

        Ok(())
    }

    pub fn decrement_delay_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    pub fn fetch(&mut self) -> u16 {
        let bytes = (self.memory[self.pc as usize] as u16) << 8
            | (self.memory[self.pc as usize + 1] as u16);
        self.pc += 2;
        bytes
    }

    pub fn execute(&mut self, instruction: u16) {
        let b0 = (instruction & 0xFF00) >> 8;
        let b1 = (instruction & 0x00FF) as u8;

        let opcode = (b0 & 0xF0) >> 4;
        let x = (b0 & 0x0F) as usize;
        let y = ((b1 & 0xF0) >> 4) as usize;
        let n = b1 & 0x0F;
        let byte = b1;
        let nnn = instruction & 0x0FFF;

        match (opcode, x, y, n) {
            // CLS
            (0x0, 0x0, 0xE, 0x0) => {
                for y in 0..display::HEIGHT {
                    for x in 0..display::WIDTH {
                        self.display[x as usize][y as usize] = false;
                    }
                }
            }
            // RET
            (0x0, 0x0, 0xE, 0xE) => {
                self.pc = self.stack[self.sp as usize];
                self.sp -= 1;
            }
            // JP addr
            (0x1, _, _, _) => {
                self.pc = nnn;
            }
            // CALL addr
            (0x2, _, _, _) => {
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = nnn;
            }
            // SE Vx, byte
            (0x3, _, _, _) => {
                if self.V[x] == byte {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte
            (0x4, _, _, _) => {
                if self.V[x] != byte {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy
            (0x5, _, _, 0x0) => {
                if self.V[x] == self.V[y] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte
            (0x6, _, _, _) => {
                self.V[x] = byte;
            }
            // ADD Vx, byte
            (0x7, _, _, _) => {
                self.V[x] = self.V[x].wrapping_add(byte);
            }
            // LD Vx, Vy
            (0x8, _, _, 0x0) => {
                self.V[x] = self.V[y];
            }
            // OR Vx, Vy
            (0x8, _, _, 0x1) => {
                self.V[x] |= self.V[y];
            }
            // AND Vx, Vy
            (0x8, _, _, 0x2) => {
                self.V[x] &= self.V[y];
            }
            // XOR Vx, Vy
            (0x8, _, _, 0x3) => {
                self.V[x] ^= self.V[y];
            }
            // ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                let sum = self.V[x] as u16 + self.V[y] as u16;
                self.V[0xF] = if sum > 0xFF { 1 } else { 0 };
                self.V[x] = sum as u8;
            }
            // SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                self.V[0xF] = if self.V[x] > self.V[y] { 1 } else { 0 };
                self.V[x] = self.V[x].wrapping_sub(self.V[y]);
            }
            // SHR Vx {, Vy}
            (0x8, _, _, 0x6) => {
                self.V[0xF] = self.V[x] & 0x1;
                self.V[x] /= 2;
            }
            // SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                self.V[0xF] = if self.V[y] > self.V[x] { 1 } else { 0 };
                self.V[x] = self.V[y].wrapping_sub(self.V[x]);
            }
            // SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                self.V[0xF] = self.V[x] >> 7;
                self.V[x] *= 2;
            }
            // SNE Vx, Vy
            (0x9, _, _, 0x0) => {
                if self.V[x] != self.V[y] {
                    self.pc += 2;
                }
            }
            // LD I, addr
            (0xA, _, _, _) => {
                self.I = nnn;
            }
            // JP V0, addr
            (0xB, _, _, _) => {
                self.pc = nnn + self.V[0] as u16;
            }
            // RND Vx, byte
            (0xC, _, _, _) => {
                let rand = rand::random::<u8>();
                self.V[x] = rand & byte;
            }
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                let mut x: usize = (self.V[x] % display::WIDTH) as usize;
                let mut y: usize = (self.V[y] % display::HEIGHT) as usize;
                self.V[0xF] = 0;

                for row in 0..n {
                    let byte = self.memory[self.I as usize + row as usize];
                    for j in 0..8 {
                        let bit = byte & (0b1000_0000 >> j);
                        if bit != 0 && self.display[x][y] {
                            self.display[x][y] = false;
                            self.V[0xF] = 1;
                        } else if bit != 0 && !self.display[x][y] {
                            self.display[x][y] = true;
                        }

                        if x == (display::WIDTH - 1) as usize {
                            break;
                        }

                        x += 1;
                    }

                    y += 1;
                    if y == (display::HEIGHT - 1) as usize {
                        break;
                    }
                }
            }
            // SKP Vx
            (0xE, _, 0x9, 0xE) => {
                if self.keypad.is_pressed(self.V[x].into()) {
                    self.pc += 2;
                }
            }
            // SKNP Vx
            (0xE, _, 0xA, 0x1) => {
                if !self.keypad.is_pressed(self.V[x].into()) {
                    self.pc += 2;
                }
            }
            // LD Vx, DT
            (0xF, _, 0x0, 0x7) => {
                self.V[x] = self.delay_timer;
            }
            // LD Vx, K
            (0xF, _, 0x0, 0xA) => {
                let key = self.keypad.wait_for_key_press();
                self.V[x] = key as u8;
            }
            // LD DT, Vx
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.V[x];
            }
            // LD ST, Vx
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.V[x];
            }
            // ADD I, Vx
            (0xF, _, 0x1, 0xE) => {
                self.I += self.V[x] as u16;
            }
            // LD F, Vx
            (0xF, _, 0x2, 0x9) => {
                let sprite = self.V[x];
                self.I = font::get_sprite_addr(sprite);
            }
            // LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                let val = self.V[x];
                self.memory[self.I as usize] = val / 100;
                self.memory[self.I as usize + 1] = (val / 10) % 10;
                self.memory[self.I as usize + 2] = val % 10;
            }
            // LD [I], Vx
            (0xF, _, 0x5, 0x5) => {
                for i in 0..=x {
                    self.memory[self.I as usize + i] = self.V[i];
                }
            }
            // LD Vx, [I]
            (0xF, _, 0x6, 0x5) => {
                for i in 0..=x {
                    self.V[i] = self.memory[self.I as usize + i];
                }
            }
            _ => {
                Self::unknown_instruction(instruction);
            }
        }
    }

    fn unknown_instruction(instruction: u16) {
        error!("Unknown instruction: {:#06X}", instruction);
    }
}

impl Debug for Chip8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip8")
            .field("pc", &self.pc)
            .field("I", &self.I)
            .field("sp", &self.sp)
            .field("delay_timer", &self.delay_timer)
            .field("sound_timer", &self.sound_timer)
            .field("V", &self.V)
            .finish()
    }
}
