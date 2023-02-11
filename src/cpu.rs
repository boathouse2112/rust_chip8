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
}
