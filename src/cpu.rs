use bit_iter::BitIter;
use std::num::Wrapping;

use byteorder::{BigEndian, ByteOrder};

const FONT_START_LOCATION: usize = 0x50;
const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct CPU {
    // Memory 4096 bytes
    pub memory: [u8; 4096],
    // Dixplay 64 x 32 black & white pixels
    pub display: [[bool; 64]; 32],
    // Registers 0 through F
    pub v: [u8; 16],
    // Program counter
    pub pc: u16,
    // Index register
    pub i: u16,
    // Stack of 16-bit addresses
    pub stack: Vec<u16>,
    // Delay timer, sound timer
    pub dt: u8,
    pub st: u8,
}

impl CPU {
    pub fn new() -> CPU {
        // Read the sprite font into memory
        let mut memory = [0u8; 4096];
        memory[FONT_START_LOCATION..FONT_START_LOCATION + 0x50].clone_from_slice(&FONT[..]);

        return CPU {
            memory: memory,
            display: [[false; 64]; 32],
            v: [0; 16],
            pc: 0x200, // Programs start at 0x200
            i: 0,
            stack: Vec::new(),
            dt: 0,
            st: 0,
        };
    }

    // Draw a sprite at x and y on the display, using sprite data of the given length,
    // drawn from the given memory location
    // Returns whether any cells were turned off
    fn draw_sprite(
        &mut self,
        x_start: usize,
        y_start: usize,
        length: usize,
        sprite_location: usize,
    ) -> bool {
        let sprite_data = self.memory[sprite_location..sprite_location + length].to_vec();
        println!("{:02X?}", sprite_data);

        let mut cells_turned_off = false;
        for (y_offset, byte) in sprite_data.iter().enumerate() {
            for x_offset in 0..8 {
                if x_start + x_offset >= self.display[y_start + y_offset].len() {
                    break;
                }
                let bit = (byte >> (7 - x_offset)) % 2;
                println!("offset: {:?}, bit: {:b}", x_offset, bit);
                if bit != 0 {
                    let x = x_start + x_offset;
                    let y = y_start + y_offset;
                    let current_cell = self.display[y][x];
                    if current_cell {
                        cells_turned_off = true;
                    }
                    self.display[y][x] = !current_cell;
                }
            }
        }
        return cells_turned_off;
    }

    // Actually this runs a cycle
    pub fn run_cycle(&mut self, _instruction: u16) {
        // Read instruction
        let pc_idx = self.pc as usize;
        let instruction = BigEndian::read_u16(&self.memory[pc_idx..pc_idx + 2]);
        // println!("{:X}", instruction);

        // Increment pc
        self.pc += 2;

        // Get instruction nibble values
        let x_nibble = (instruction & 0x0F00) >> 8;
        let y_nibble = (instruction & 0x00F0) >> 4;
        let n_3_nibble = instruction & 0x0FFF;
        let n_2_nibble = instruction & 0x00FF;
        let n_1_nibble = instruction & 0x000F;

        // Check 1st nibble
        match instruction & 0xF000 {
            // 00EE -- End subroutine
            a if a == 0x00EE => self.pc = self.stack.pop().unwrap(),
            // 1XXX -- JMP to XXX
            a if a == 0x1000 => self.pc = instruction & 0x0FFF,
            // 2XXX -- Subroutine: push PC to stack, JMP to XXX
            a if a == 0x2000 => {
                self.stack.push(self.pc);
                self.pc = instruction & 0x0FFF;
            }
            // 3XNN -- Skip the following instruction if vX == NN
            a if a == 0x3000 => {
                if self.v[x_nibble as usize] == n_2_nibble as u8 {
                    self.pc += 2;
                };
            }
            // 4XNN -- Skip the following instruction if vX != NN
            a if a == 0x4000 => {
                if self.v[x_nibble as usize] != n_2_nibble as u8 {
                    self.pc += 2;
                };
            }
            // 5XY0 -- Skip the following instruction if vX == vY
            a if a == 0x4000 && instruction & 0x000F == 0 => {
                if self.v[x_nibble as usize] == self.v[y_nibble as usize] {
                    self.pc += 2;
                };
            }
            // 6XNN -- Store NN in register vX
            a if a == 0x6000 => {
                let value = instruction as u8;
                self.v[x_nibble as usize] = value;
            }
            // 7XNN -- Add the value NN to register vX -- Use wrapping overflow
            a if a == 0x7000 => {
                self.v[x_nibble as usize] =
                    (Wrapping(self.v[x_nibble as usize]) + Wrapping(n_2_nibble as u8)).0
            }
            // 9XY0 -- Skip the following instruction if vX != vY
            a if a == 0x9000 && instruction & 0x000F == 0 => {
                if self.v[x_nibble as usize] != self.v[y_nibble as usize] {
                    self.pc += 2;
                };
            }
            // ANNN -- Store memory address NNN in register I
            a if a == 0xA000 => self.i = n_3_nibble,
            // DXYN -- Draw a sprite at vX, vY with N bytes of sprite data starting at the address stored in I
            a if a == 0xD000 => {
                self.v[0xf] = self
                    .draw_sprite(
                        (self.v[x_nibble as usize] % 64) as usize,
                        (self.v[y_nibble as usize] % 32) as usize,
                        n_1_nibble as usize,
                        self.i as usize,
                    )
                    .into();
            }
            // FX29 -- Set I to the memory address of the sprite for the digit stored in vX
            a if a == 0xF000 && instruction & 0x00FF == 0x0029 => {
                self.i = (FONT_START_LOCATION + (5 * self.v[x_nibble as usize]) as usize) as u16
            }
            _ => {}
        }
    }
}
