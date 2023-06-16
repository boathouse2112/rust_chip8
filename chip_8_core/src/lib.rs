mod chip_8;
pub use chip_8::Chip8;

pub mod globals;

pub mod interface;
pub use interface::Interface;

pub mod runner;

#[macro_use]
extern crate lazy_static;
