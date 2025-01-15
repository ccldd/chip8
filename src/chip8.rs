#[derive(Debug)]
pub struct Chip8 {
    memory: [u8; 4096],
    display: [[u8; 64]; 32],
    pc: u16,
    I: u16,
    stack: [u16; 16],
    sp: u8,
    delay_timer: u8,
    sound_timer: u8,
    V: [u8; 16], // registers
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            memory: [0; 4096],
            display: [[0; 64]; 32],
            pc: 0x200,
            I: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            V: [0; 16],
        }
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
                    self.unknown_instruction(instruction);
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
                        self.unknown_instruction(instruction);
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
            _ => {
                self.unknown_instruction(instruction);
            }
        }
    }

    fn unknown_instruction(&self, instruction: u16) {
        println!("Unknown instruction: {:x}", instruction);
    }
}
