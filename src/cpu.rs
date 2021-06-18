
pub struct CPU {
    // Registers 0 through F
    v: [u8; 16],
    // Program counter
    pc: u16,
    // Index register
    i: u16,
    // Delay timer, sound timer
    dt: u8,
    st: u8,
}

impl CPU {
    fn new() -> CPU {
        return CPU {
            v: [0; 16],
            pc: 0x200,      // Programs start at 0x200
            i: 0,
            dt: 0,
            st: 0,
        }
    }
}