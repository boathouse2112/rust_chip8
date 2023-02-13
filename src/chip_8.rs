use std::{collections::HashSet, num::Wrapping};

use byteorder::{BigEndian, ByteOrder};

pub const DISPLAY_WIDTH: i32 = 64;
pub const DISPLAY_HEIGHT: i32 = 32;

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

pub struct Chip8 {
    // Memory 4096 bytes
    pub memory: [u8; 4096],
    // Dixplay 64 x 32 black & white pixels
    pub display: HashSet<(i32, i32)>,
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

impl Chip8 {
    pub fn new() -> Chip8 {
        // Read the sprite font into memory
        let mut memory = [0u8; 4096];
        memory[FONT_START_LOCATION..FONT_START_LOCATION + FONT.len()].clone_from_slice(&FONT[..]);

        return Chip8 {
            memory: memory,
            display: HashSet::default(),
            v: [0; 16],
            pc: 0x200, // Programs start at 0x200
            i: 0,
            stack: Vec::new(),
            dt: 0,
            st: 0,
        };
    }

    // Actually this runs a cycle
    pub fn run_cycle(&mut self) {
        // Read instruction
        let pc_idx = self.pc as usize;
        let instruction = BigEndian::read_u16(&self.memory[pc_idx..pc_idx + 2]);

        // Increment pc
        self.pc += 2;

        // Get instruction nibble values
        let x_nibble = (instruction & 0x0F00) >> 8;
        let y_nibble = (instruction & 0x00F0) >> 4;
        let n_nibble = instruction & 0x000F;
        let nn_nibble = instruction & 0x00FF; // Technically not a nibble.
        let nnn_nibble = instruction & 0x0FFF;

        // Check 1st nibble
        match instruction & 0xF000 {
            // 00E0 -- Clear screen
            _ if instruction == 0x00E0 => self.display.clear(),
            // 00EE -- End subroutine
            _ if instruction == 0x00EE => self.pc = self.stack.pop().unwrap(),
            // 1XXX -- JMP to XXX
            a if a == 0x1000 => self.pc = instruction & 0x0FFF,
            // 2XXX -- Subroutine: push PC to stack, JMP to XXX
            a if a == 0x2000 => {
                self.stack.push(self.pc);
                self.pc = instruction & 0x0FFF;
            }
            // 3XNN -- Skip the following instruction if vX == NN
            a if a == 0x3000 => {
                if self.v[x_nibble as usize] == nn_nibble as u8 {
                    self.pc += 2;
                };
            }
            // 4XNN -- Skip the following instruction if vX != NN
            a if a == 0x4000 => {
                if self.v[x_nibble as usize] != nn_nibble as u8 {
                    self.pc += 2;
                };
            }
            // 5XY0 -- Skip the following instruction if vX == vY
            a if a == 0x5000 && instruction & 0x000F == 0 => {
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
                    (Wrapping(self.v[x_nibble as usize]) + Wrapping(nn_nibble as u8)).0
            }
            // 8XY0 -- Store the value of register VY in register VX
            _ if instruction & 0xF00F == 0x8000 => {
                self.v[x_nibble as usize] = self.v[y_nibble as usize]
            }
            // 8XY1 -- Set VX to VX OR VY
            _ if instruction & 0xF00F == 0x8001 => {
                self.v[x_nibble as usize] = self.v[x_nibble as usize] | self.v[y_nibble as usize];
            }
            // 8XY2 -- Set VX to VX AND VY
            _ if instruction & 0xF00F == 0x8002 => {
                self.v[x_nibble as usize] = self.v[x_nibble as usize] & self.v[y_nibble as usize];
            }
            // 8XY3 -- Set VX to VX XOR VY
            _ if instruction & 0xF00F == 0x8003 => {
                self.v[x_nibble as usize] = self.v[x_nibble as usize] ^ self.v[y_nibble as usize];
            }
            // 9XY0 -- Skip the following instruction if vX != vY
            _ if instruction & 0xF00F == 0x9000 => {
                if self.v[x_nibble as usize] != self.v[y_nibble as usize] {
                    self.pc += 2;
                };
            }
            // ANNN -- Store memory address NNN in register I
            a if a == 0xA000 => self.i = nnn_nibble,
            // DXYN -- Draw a sprite at vX, vY with N bytes of sprite data starting at the address stored in I
            a if a == 0xD000 => {
                self.v[0xf] = self
                    .draw_sprite(
                        (self.v[x_nibble as usize] % 64) as i32,
                        (self.v[y_nibble as usize] % 32) as i32,
                        n_nibble as usize,
                        self.i as usize,
                    )
                    .into();
            }
            // FX29 -- Set I to the memory address of the sprite for the digit stored in vX
            a if a == 0xF000 && instruction & 0x00FF == 0x0029 => {
                self.i = (FONT_START_LOCATION + (5 * self.v[x_nibble as usize]) as usize) as u16
            }
            // FX65 -- Fill registers V0 to VX inclusive with the values stored in memory starting at address I
            //         I is set to I + X + 1 after operation
            a if a == 0xF000 && instruction & 0x00FF == 0x0065 => {
                let mut addr = self.i;
                for register in 0..x_nibble + 1 {
                    let value = self.memory[addr as usize];
                    self.v[register as usize] = value;
                    addr += 1;
                }
                self.i = self.i + x_nibble + 1;
            }
            _ => {}
        }
    }

    // Draw a sprite at x and y on the display, using sprite data of the given length,
    // drawn from the given memory location
    // Returns whether any cells were turned off
    fn draw_sprite(
        &mut self,
        x_start: i32,
        y_start: i32,
        length: usize,
        sprite_location: usize,
    ) -> bool {
        let sprite_data = self.memory[sprite_location..sprite_location + length as usize].to_vec();

        let mut cells_turned_off = false;
        for (y_offset, byte) in sprite_data.iter().enumerate() {
            for x_offset in 0..8 {
                if x_start + x_offset >= DISPLAY_WIDTH {
                    break;
                }
                let bit = (byte >> (7 - x_offset)) % 2;
                if bit != 0 {
                    let x = x_start + x_offset;
                    let y = y_start + y_offset as i32;
                    let current_cell = self.display.contains(&(x, y));
                    if current_cell {
                        cells_turned_off = true;
                        self.display.remove(&(x, y));
                    } else {
                        self.display.insert((x, y));
                    }
                }
            }
        }
        return cells_turned_off;
    }
}

#[cfg(test)]
mod test {
    use super::Chip8;

    #[test]
    fn jump() {
        // 0x1NNN moves the program counter to NNN
        let mut chip_8 = Chip8::new();

        chip_8.memory[0x200] = 0x1E;
        chip_8.memory[0x201] = 0xEE;

        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0xEEE);
    }

    #[test]
    fn store_in_registers() {
        let mut chip_8 = Chip8::new();

        // 0xANNN stores address NNN in register I
        chip_8.memory[0x200] = 0xA0;
        chip_8.memory[0x201] = 0xEF;

        chip_8.run_cycle();

        assert_eq!(chip_8.i, 0x0EF);

        // 0x6XNN stores NN in register vX
        chip_8.memory[0x202] = 0x60;
        chip_8.memory[0x203] = 0xEF;
        chip_8.memory[0x204] = 0x6F;
        chip_8.memory[0x205] = 0x01;

        chip_8.run_cycle();
        chip_8.run_cycle();

        assert_eq!(chip_8.v[0], 0xEF);
        assert_eq!(chip_8.v[0xF], 0x01);
    }

    #[test]
    fn fill_registers() {
        let mut chip_8 = Chip8::new();

        // 0xFX65 fills registers v0 through vX with the values in memory starting at the address stored in I
        // Afterwards, I is set to I + X + 1

        // Instruction
        chip_8.memory[0x200] = 0xFF;
        chip_8.memory[0x201] = 0x65;

        // Values for registers
        chip_8.i = 0x500;
        chip_8.memory[0x500] = 0xFE;
        chip_8.memory[0x501] = 0xEF;
        chip_8.memory[0x502] = 0xDC;
        chip_8.memory[0x503] = 0xCD;
        chip_8.memory[0x504] = 0xBA;
        chip_8.memory[0x505] = 0xAB;
        chip_8.memory[0x506] = 0x98;
        chip_8.memory[0x507] = 0x89;
        chip_8.memory[0x508] = 0x76;
        chip_8.memory[0x509] = 0x67;
        chip_8.memory[0x50A] = 0x54;
        chip_8.memory[0x50B] = 0x45;
        chip_8.memory[0x50C] = 0x32;
        chip_8.memory[0x50D] = 0x23;
        chip_8.memory[0x50E] = 0x10;
        chip_8.memory[0x50F] = 0x01;

        chip_8.run_cycle();

        // Check all registers are correct.
        assert_eq!(chip_8.v[0], 0xFE);
        assert_eq!(chip_8.v[1], 0xEF);
        assert_eq!(chip_8.v[2], 0xDC);
        assert_eq!(chip_8.v[3], 0xCD);
        assert_eq!(chip_8.v[4], 0xBA);
        assert_eq!(chip_8.v[5], 0xAB);
        assert_eq!(chip_8.v[6], 0x98);
        assert_eq!(chip_8.v[7], 0x89);
        assert_eq!(chip_8.v[8], 0x76);
        assert_eq!(chip_8.v[9], 0x67);
        assert_eq!(chip_8.v[0xA], 0x54);
        assert_eq!(chip_8.v[0xB], 0x45);
        assert_eq!(chip_8.v[0xC], 0x32);
        assert_eq!(chip_8.v[0xD], 0x23);
        assert_eq!(chip_8.v[0xE], 0x10);
        assert_eq!(chip_8.v[0xF], 0x01);

        // Check that I has been updated correctly
        assert_eq!(chip_8.i, 0x510);
    }

    #[test]
    fn skip_instructions() {
        let mut chip_8 = Chip8::new();

        // 0x3XNN skips an instruction if vX == NN
        chip_8.i = 0x100;
        chip_8.v[0] = 0x16;

        chip_8.memory[0x200] = 0x30; // Skip if v0 == 0x16
        chip_8.memory[0x201] = 0x16;
        chip_8.memory[0x202] = 0xAF; // Skipped, would set I to 0xFFF
        chip_8.memory[0x203] = 0xFF;
        chip_8.memory[0x204] = 0x30; // Skip if v0 == 0xFF
        chip_8.memory[0x205] = 0xFF;
        chip_8.memory[0x206] = 0xA2; // Not skipped, sets I to 0x222
        chip_8.memory[0x207] = 0x22;

        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x204);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x206);
        assert_eq!(chip_8.i, 0x100);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x208);
        assert_eq!(chip_8.i, 0x222);

        // 0x4XNN skips an instruction if vX != NN
        chip_8.pc = 0x300;
        chip_8.i = 0x100;
        chip_8.v[0xF] = 0x16;

        chip_8.memory[0x300] = 0x4F; // Skip if vF != 0xFF
        chip_8.memory[0x301] = 0xFF;
        chip_8.memory[0x302] = 0xAF; // Skipped, would set I to 0xFFF
        chip_8.memory[0x303] = 0xFF;
        chip_8.memory[0x304] = 0x4F; // Skip if vF != 0x16
        chip_8.memory[0x305] = 0x16;
        chip_8.memory[0x306] = 0xA3; // Not skipped, sets I to 0x333
        chip_8.memory[0x307] = 0x33;

        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x304);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x306);
        assert_eq!(chip_8.i, 0x100);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x308);
        assert_eq!(chip_8.i, 0x333);

        // 0x5XY0 skips an instruction if vX == vY
        chip_8.pc = 0x400;
        chip_8.i = 0x100;
        chip_8.v[0xA] = 0x16;
        chip_8.v[0xB] = 0x16;
        chip_8.v[0xC] = 0x20;

        chip_8.memory[0x400] = 0x5A; // Skip if vA == vB
        chip_8.memory[0x401] = 0xB0;
        chip_8.memory[0x402] = 0xAF; // Skipped, would set I to 0xFFF
        chip_8.memory[0x403] = 0xFF;
        chip_8.memory[0x404] = 0x5A; // Skip if vA == vC
        chip_8.memory[0x405] = 0xC0;
        chip_8.memory[0x406] = 0xA3; // Not skipped, sets I to 0x333
        chip_8.memory[0x407] = 0x33;

        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x404);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x406);
        assert_eq!(chip_8.i, 0x100);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x408);
        assert_eq!(chip_8.i, 0x333);

        // 0x9XY0 skips an instruction if vX != vY
        chip_8.pc = 0x500;
        chip_8.i = 0x100;
        chip_8.v[0xD] = 0x16;
        chip_8.v[0xE] = 0x20;
        chip_8.v[0xF] = 0x16;

        chip_8.memory[0x500] = 0x9D; // Skip if vD != vE
        chip_8.memory[0x501] = 0xE0;
        chip_8.memory[0x502] = 0xAF; // Skipped, would set I to 0xFFF
        chip_8.memory[0x503] = 0xFF;
        chip_8.memory[0x504] = 0x9D; // Skip if vD != vF
        chip_8.memory[0x505] = 0xF0;
        chip_8.memory[0x506] = 0xA3; // Not skipped, sets I to 0x333
        chip_8.memory[0x507] = 0x33;

        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x504);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x506);
        assert_eq!(chip_8.i, 0x100);
        chip_8.run_cycle();
        assert_eq!(chip_8.pc, 0x508);
        assert_eq!(chip_8.i, 0x333);
    }

    #[test]
    fn clear_screen() {
        // 0x00E0 clears the screen
        let mut chip_8 = Chip8::new();

        chip_8.display.insert((0, 0));
        chip_8.display.insert((10, 10));
        chip_8.display.insert((64, 32));

        chip_8.memory[0x200] = 0x00;
        chip_8.memory[0x201] = 0xE0;

        chip_8.run_cycle();

        assert!(chip_8.display.is_empty());
    }
}
