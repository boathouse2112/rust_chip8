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
        // Check 1st nibble
        match instruction & 0xF000 {
            // 1XXX -- JMP to XXX
            x if x == 0x1000 => self.pc = instruction & 0x0FFF,
            // 2XXX -- Subroutine: push PC to stack, JMP to XXX
            x if x == 0x2000 => {
                self.stack.push(self.pc);
                self.pc = instruction & 0x0FFF;
            }
            // 6XNN -- Store NN in register vX
            x if x == 0x6000 => {
                let register = ((instruction & 0x0F00) >> 8) as usize;
                let value = instruction as u8;
                self.v[register] = value;
            }
            _ => println!("no"),
        }
    }
}
