use crate::{
    chip_8::Chip8,
    globals::{Err, Keys},
};

/// A GUI/TUI interface for the emulator
pub trait Interface {
    fn run(&mut self, chip_8: &mut Chip8) -> Result<(), Err>;

    /// Set up the interface for the run loop.
    /// Called just before the run loop.
    fn setup(&mut self) -> Result<(), Err>;

    fn load_rom(&mut self, chip_8: &mut Chip8) -> Result<(), Err>;

    /// Read held chip8 keys, or a break signal
    fn read_keys(&mut self) -> Result<Keys, Err>;

    /// Draw the chip8's display buffer to the screen
    fn draw(&mut self, chip_8: &mut Chip8) -> Result<(), Err>;

    fn cleanup(&mut self) -> Result<(), Err>;
}
