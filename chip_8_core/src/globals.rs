use std::collections::HashSet;

pub type Err = Box<dyn std::error::Error>;

pub enum Keys {
    Break,
    Keys(HashSet<u8>),
}

pub const FRAMES_PER_SECOND: i32 = 60;
pub const INSTRUCTIONS_PER_FRAME: i32 = 16;

pub const DISPLAY_WIDTH: i32 = 64;
pub const DISPLAY_HEIGHT: i32 = 32;
