use keypad::KeyPad;
use tracing::error;

mod font;
pub mod keypad;

#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; 4096],
    display: [[u8; 64]; 32],
    pc: u16,

    #[allow(non_snake_case)]
    I: u16,

    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,

    #[allow(non_snake_case)]
    V: [u8; 16], // registers

    keypad: KeyPad,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        let mut c = Chip8 {
            memory: [0; 4096],
            display: [[0; 64]; 32],
            pc: 0x200,
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

    pub fn execute(&mut self, instruction: u16) {
        let opcode = instruction & 0xF000;
        match opcode {
            0x0000 => match instruction & 0x00FF {
                0x00E0 => {
                    // CLS
                    for y in 0..32 {
                        for x in 0..64 {
                            self.display[x][y] = 0;
                        }
                    }
                }
                0x00EE => {
                    // RET
                    self.pc = self.stack[self.sp as usize];
                    self.sp -= 1;
                }
                _ => {
                    Self::unknown_instruction(instruction);
                }
            },
            0x1000 => {
                // JP addr
                let addr = instruction & 0x0FFF;
                self.pc = addr;
            }
            0x2000 => {
                // CALL addr
                let addr = instruction & 0x0FFF;
                self.sp += 1;
                self.stack[self.sp as usize] = self.pc;
                self.pc = addr;
            }
            0x3000 => {
                // SE Vx, byte
                let x = instruction & 0x0F00 >> 8;
                let byte = instruction & 0x00FF;
                let val = self.V[x as usize];
                if val == byte as u8 {
                    self.pc += 2;
                }
            }
            0x4000 => {
                // SNE Vx, byte
                let x = instruction & 0x0F00 >> 8;
                let byte = instruction & 0x00FF;
                let val = self.V[x as usize];
                if val != byte as u8 {
                    self.pc += 2;
                }
            }
            0x5000 => {
                // SE Vx, Vy
                let x = instruction & 0x0F00 >> 8;
                let y = instruction & 0x00F0 >> 4;
                let vx = self.V[x as usize];
                let vy = self.V[y as usize];
                if vx == vy {
                    self.pc += 2;
                }
            }
            0x6000 => {
                // LD Vx, byte
                let x = instruction & 0x0F00 >> 8;
                let byte = instruction & 0x00FF;
                self.V[x as usize] = byte as u8;
            }
            0x7000 => {
                // ADD Vx, byte
                let x = instruction & 0x0F00 >> 8;
                let byte = instruction & 0x00FF;
                let val = self.V[x as usize];
                self.V[x as usize] = val.wrapping_add(byte as u8);
            }
            0x8000 => {
                let x = instruction & 0x0F00 >> 8;
                let y = instruction & 0x00F0 >> 4;
                match instruction & 0x000F {
                    0x0000 => {
                        // LD Vx, Vy
                        self.V[x as usize] = self.V[y as usize];
                    }
                    0x0001 => {
                        // OR Vx, Vy
                        self.V[x as usize] |= self.V[y as usize];
                    }
                    0x0002 => {
                        // AND Vx, Vy
                        self.V[x as usize] &= self.V[y as usize];
                    }
                    0x0003 => {
                        // XOR Vx, Vy
                        self.V[x as usize] ^= self.V[y as usize];
                    }
                    0x0004 => {
                        // ADD Vx, Vy
                        let sum = self.V[x as usize] as u16 + self.V[y as usize] as u16;
                        self.V[0xF] = if sum > 0xFF { 1 } else { 0 };
                        self.V[x as usize] = sum as u8;
                    }
                    0x0005 => {
                        // SUB Vx, Vy
                        self.V[0xF] = if self.V[x as usize] > self.V[y as usize] {
                            1
                        } else {
                            0
                        };
                        self.V[x as usize] = self.V[x as usize].wrapping_sub(self.V[y as usize]);
                    }
                    0x0006 => {
                        // SHR Vx {, Vy}
                        self.V[0xF] = self.V[x as usize] & 0x1;
                        self.V[x as usize] /= 2;
                    }
                    0x0007 => {
                        // SUBN Vx, Vy
                        self.V[0xF] = if self.V[y as usize] > self.V[x as usize] {
                            1
                        } else {
                            0
                        };
                        self.V[x as usize] = self.V[y as usize].wrapping_sub(self.V[x as usize]);
                    }
                    0x000E => {
                        // SHL Vx {, Vy}
                        self.V[0xF] = self.V[x as usize] >> 7;
                        self.V[x as usize] *= 2;
                    }
                    _ => {
                        Self::unknown_instruction(instruction);
                    }
                }
                // LD Vx, Vy
                self.V[x as usize] = self.V[y as usize];
            }
            0x9000 => {
                // SNE Vx, Vy
                let x = instruction & 0x0F00 >> 8;
                let y = instruction & 0x00F0 >> 4;
                if x != y {
                    self.pc += 2;
                }
            }
            0xA000 => {
                // LD I, addr
                let addr = instruction & 0x0FFF;
                self.I = addr;
            }
            0xB000 => {
                // JP V0, addr
                let addr = instruction & 0x0FFF;
                self.pc = addr + self.V[0] as u16;
            }
            0xC000 => {
                // RND Vx, byte
                let x = instruction & 0x0F00 >> 8;
                let byte = instruction & 0x00FF;
                let rand = rand::random::<u8>();
                self.V[x as usize] = rand & byte as u8;
            }
            0xD000 => {
                // DRW Vx, Vy, nibble
                let x = (instruction & 0x0F00 >> 8) % 64;
                let y = (instruction & 0x00F0 >> 4) % 32;
                self.V[0xF] = 0;

                let n = instruction & 0x000F;
                for i in 0..n {
                    let byte = self.memory[self.I as usize + i as usize];
                    for j in 0..8 {
                        let bit = byte & (0x80 >> j);
                        if bit == 1 && self.display[(x + j) as usize][(y + i) as usize] == 1 {
                            self.V[0xF] = 1;
                        } else if bit == 1 && self.display[(x + j) as usize][(y + i) as usize] == 0
                        {
                            self.display[(x + j) as usize][(y + i) as usize] = 1;
                        }

                        if x + j >= 64 {
                            break;
                        }
                    }
                }
            }
            0xE000 => {
                let x = instruction & 0x0F00 >> 8;
                let key = self.V[x as usize];
                match instruction & 0x00FF {
                    0x009E => {
                        // SKP Vx
                        if self.keypad.is_pressed(key.into()) {
                            self.pc += 2;
                        }
                    }
                    0x00A1 => {
                        // SKNP Vx
                        if !self.keypad.is_pressed(key.into()) {
                            self.pc += 2;
                        }
                    }
                    _ => {
                        Self::unknown_instruction(instruction);
                    }
                }
            }
            0xF000 => {
                let x = instruction & 0x0F00 >> 8;
                match instruction & 0x00FF {
                    0x0007 => {
                        // LD Vx, DT
                        self.V[x as usize] = self.delay_timer;
                    }
                    0x000A => {
                        // LD Vx, K
                        let key = self.keypad.wait_for_key_press();
                        self.V[x as usize] = key as u8;
                    }
                    0x0015 => {
                        // LD DT, Vx
                        self.delay_timer = self.V[x as usize];
                    }
                    0x0018 => {
                        // LD ST, Vx
                        self.sound_timer = self.V[x as usize];
                    }
                    0x001E => {
                        // ADD I, Vx
                        self.I += self.V[x as usize] as u16;
                    }
                    0x0029 => {
                        // LD F, Vx
                        let sprite = self.V[x as usize];
                        self.I = font::get_sprite_addr(sprite);
                    }
                    0x0033 => {
                        // LD B, Vx
                        let val = self.V[x as usize];
                        self.memory[self.I as usize] = val / 100;
                        self.memory[self.I as usize + 1] = (val / 10) % 10;
                        self.memory[self.I as usize + 2] = val % 10;
                    }
                    0x0055 => {
                        // LD [I], Vx
                        for i in 0..=x {
                            self.memory[self.I as usize + i as usize] = self.V[i as usize];
                        }
                    }
                    0x0065 => {
                        // LD Vx, [I]
                        for i in 0..=x {
                            self.V[i as usize] = self.memory[self.I as usize + i as usize];
                        }
                    }
                    _ => {
                        Self::unknown_instruction(instruction);
                    }
                }
            }
            _ => {
                Self::unknown_instruction(instruction);
            }
        }
    }

    fn unknown_instruction(instruction: u16) {
        error!("Unknown instruction: {:x}", instruction);
    }
}
