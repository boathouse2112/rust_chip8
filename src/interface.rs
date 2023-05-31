use crate::{chip_8::Chip8, globals::Err};

pub use graphical::Graphical;
pub use terminal::Terminal;

pub mod graphical;
pub mod terminal;

/// A GUI/TUI interface for the emulator
pub trait Interface {
    fn run(&mut self, chip_8: &mut Chip8) -> Result<(), Err>;
}
