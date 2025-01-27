use core::fmt;
use std::{
    error::Error,
    fmt::{Debug, Formatter},
    fs::File,
    io::Read,
    path::Path,
};

use keypad::Keypad;
use tracing::error;

pub mod display;
mod font;
pub mod keypad;

const INITIAL_PC: u16 = 0x200;
const MEMORY_SIZE: usize = 4096;
const MAX_ROM_SIZE: usize = MEMORY_SIZE - INITIAL_PC as usize;

type Instruction = u16;

pub struct Chip8 {
    memory: [u8; MEMORY_SIZE],
    pub display: [[bool; display::HEIGHT as usize]; display::WIDTH as usize],
    pc: u16,
    i: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    v: [u8; 16], // registers
    pub keypad: Keypad,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            memory: [0; 4096],
            display: [[false; display::HEIGHT as usize]; display::WIDTH as usize],
            pc: INITIAL_PC,
            i: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            v: [0; 16],
            keypad: Keypad::new(),
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

    pub fn tick(&mut self) {
        let next_instr = self.fetch();
        self.execute(next_instr);
        self.keypad.clear_keys_released();
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
    
    pub fn should_play_sound(&self) -> bool {
        self.sound_timer > 0
    }

    /// This is the instruction that the program counter is pointing to
    fn current_instruction(&self) -> Instruction {
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16)
    }

    fn fetch(&mut self) -> Instruction {
        let next_instruction = self.current_instruction();
        self.pc += 2;
        next_instruction
    }

    fn execute(&mut self, instruction: Instruction) {
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
                if self.v[x] == byte {
                    self.pc += 2;
                }
            }
            // SNE Vx, byte
            (0x4, _, _, _) => {
                if self.v[x] != byte {
                    self.pc += 2;
                }
            }
            // SE Vx, Vy
            (0x5, _, _, 0x0) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            // LD Vx, byte
            (0x6, _, _, _) => {
                self.v[x] = byte;
            }
            // ADD Vx, byte
            (0x7, _, _, _) => {
                self.v[x] = self.v[x].wrapping_add(byte);
            }
            // LD Vx, Vy
            (0x8, _, _, 0x0) => {
                self.v[x] = self.v[y];
            }
            // OR Vx, Vy
            (0x8, _, _, 0x1) => {
                self.v[x] |= self.v[y];
            }
            // AND Vx, Vy
            (0x8, _, _, 0x2) => {
                self.v[x] &= self.v[y];
            }
            // XOR Vx, Vy
            (0x8, _, _, 0x3) => {
                self.v[x] ^= self.v[y];
            }
            // ADD Vx, Vy
            (0x8, _, _, 0x4) => {
                let sum = self.v[x] as u16 + self.v[y] as u16;
                self.v[x] = sum as u8;
                self.v[0xF] = if sum > 0xFF { 1 } else { 0 };
            }
            // SUB Vx, Vy
            (0x8, _, _, 0x5) => {
                let (diff, borrow) = self.v[x].overflowing_sub(self.v[y]);
                self.v[x] = diff;
                self.v[0xF] = if borrow { 0 } else { 1 };
            }
            // SHR Vx {, Vy}
            (0x8, _, _, 0x6) => {
                let lsb = self.v[x] & 1;
                self.v[x] >>= 1;
                self.v[0xF] = lsb;
            }
            // SUBN Vx, Vy
            (0x8, _, _, 0x7) => {
                let (diff, borrow) = self.v[y].overflowing_sub(self.v[x]);
                self.v[x] = diff;
                self.v[0xF] = if borrow { 0 } else { 1 };
            }
            // SHL Vx {, Vy}
            (0x8, _, _, 0xE) => {
                let msb = (self.v[x] >> 7) & 1;
                self.v[x] <<= 1;
                self.v[0xF] = msb;
            }
            // SNE Vx, Vy
            (0x9, _, _, 0x0) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            // LD I, addr
            (0xA, _, _, _) => {
                self.i = nnn;
            }
            // JP V0, addr
            (0xB, _, _, _) => {
                self.pc = nnn + self.v[0] as u16;
            }
            // RND Vx, byte
            (0xC, _, _, _) => {
                let rand = rand::random::<u8>();
                self.v[x] = rand & byte;
            }
            // DRW Vx, Vy, nibble
            (0xD, _, _, _) => {
                self.v[0x0f] = 0;
                for byte in 0..n {
                    let y = (self.v[y] as usize + byte as usize) % display::HEIGHT as usize;
                    for bit in 0..8 {
                        let x = (self.v[x] as usize + bit) % display::WIDTH as usize;
                        let color = (self.memory[self.i as usize + byte as usize] >> (7 - bit)) & 1;
                        self.v[0x0f] |= color & self.display[x][y] as u8;
                        self.display[x][y] ^= color != 0;
                    }
                }
            }
            // SKP Vx
            (0xE, _, 0x9, 0xE) => {
                if self.keypad.is_key_down(self.v[x].into()) {
                    self.pc += 2;
                }
            }
            // SKNP Vx
            (0xE, _, 0xA, 0x1) => {
                if self.keypad.is_key_up(self.v[x].into()) {
                    self.pc += 2;
                }
            }
            // LD Vx, DT
            (0xF, _, 0x0, 0x7) => {
                self.v[x] = self.delay_timer;
            }
            // LD Vx, K
            (0xF, _, 0x0, 0xA) => {
                if let Some(key) = self.keypad.get_first_key_released() {
                    self.v[x] = key as u8;
                } else {
                    self.pc -= 2;
                }
            }
            // LD DT, Vx
            (0xF, _, 0x1, 0x5) => {
                self.delay_timer = self.v[x];
            }
            // LD ST, Vx
            (0xF, _, 0x1, 0x8) => {
                self.sound_timer = self.v[x];
            }
            // ADD I, Vx
            (0xF, _, 0x1, 0xE) => {
                self.i += self.v[x] as u16;
            }
            // LD F, Vx
            (0xF, _, 0x2, 0x9) => {
                let sprite = self.v[x];
                self.i = font::get_sprite_addr(sprite);
            }
            // LD B, Vx
            (0xF, _, 0x3, 0x3) => {
                let val = self.v[x];
                self.memory[self.i as usize] = val / 100;
                self.memory[self.i as usize + 1] = (val / 10) % 10;
                self.memory[self.i as usize + 2] = val % 10;
            }
            // LD [I], Vx
            (0xF, _, 0x5, 0x5) => {
                for i in 0..=x {
                    self.memory[self.i as usize + i] = self.v[i];
                }
            }
            // LD Vx, [I]
            (0xF, _, 0x6, 0x5) => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i as usize + i];
                }
            }
            _ => {
                error!("Unknown instruction: {:#06X}", instruction);
            }
        }
    }
}

impl Debug for Chip8 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Chip8")
            .field("instr", &format!("{:#06X}", self.current_instruction()))
            .field("pc", &format!("{:#06X}", self.pc))
            .field("I", &format!("{:#06X}", self.i))
            .field("sp", &format!("{:#04X}", self.sp))
            .field("dt", &format!("{:#04X}", self.delay_timer))
            .field("st", &format!("{:#04X}", self.sound_timer))
            .field(
                "V",
                &self
                    .v
                    .iter()
                    .map(|v| format!("{:#04X}", v))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
