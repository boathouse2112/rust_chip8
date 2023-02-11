use byteorder::{BigEndian, ByteOrder};

pub struct CPU {
    // Memory 4096 bytes
    pub memory: [u8; 4096],
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
        return CPU {
            memory: [0; 4096],
            v: [0; 16],
            pc: 0x200, // Programs start at 0x200
            i: 0,
            stack: Vec::new(),
            dt: 0,
            st: 0,
        };
    }

    // Actually this runs a cycle
    pub fn run_cycle(&mut self, instruction: u16) {
        // Read instruction
        let pc_idx = self.pc as usize;
        let _instruction = BigEndian::read_u16(&self.memory[pc_idx..pc_idx + 2]);

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
            x if x == 0x00EE => self.pc = self.stack.pop().unwrap(),
            // 1XXX -- JMP to XXX
            x if x == 0x1000 => self.pc = instruction & 0x0FFF,
            // 2XXX -- Subroutine: push PC to stack, JMP to XXX
            x if x == 0x2000 => {
                self.stack.push(self.pc);
                self.pc = instruction & 0x0FFF;
            }
            // 3XNN -- Skip the following instruction if vX == NN
            x if x == 0x3000 => {
                if self.v[x_nibble as usize] == n_2_nibble as u8 {
                    self.pc += 2;
                };
            }
            // 4XNN -- Skip the following instruction if vX != NN
            x if x == 0x4000 => {
                if self.v[x_nibble as usize] != n_2_nibble as u8 {
                    self.pc += 2;
                };
            }
            // 5XY0 -- Skip the following instruction if vX == vY
            x if x == 0x4000 && instruction & 0x000F == 0 => {
                if self.v[x_nibble as usize] == self.v[y_nibble as usize] {
                    self.pc += 2;
                };
            }
            // 6XNN -- Store NN in register vX
            x if x == 0x6000 => {
                let value = instruction as u8;
                self.v[x_nibble as usize] = value;
            }
            // 9XY0 -- Skip the following instruction if vX != vY
            x if x == 0x9000 && instruction & 0x000F == 0 => {
                if self.v[x_nibble as usize] != self.v[y_nibble as usize] {
                    self.pc += 2;
                };
            }
            _ => println!("no"),
        }
    }
}
