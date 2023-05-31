mod chip_8;
mod globals;
mod interface;

use chip_8::Chip8;
use clap::{Parser, ValueEnum};
use globals::Err;
use interface::{Graphical, Interface, Terminal};
use log::LevelFilter;

use std::fs;

#[macro_use]
extern crate lazy_static;

#[derive(Copy, Clone, ValueEnum)]
enum InterfaceType {
    /// Display in terminal
    Terminal,
    /// Display in new window
    Graphical,
}

/// Chip8 emulator
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Interface type
    #[arg(value_enum, default_value_t = InterfaceType::Terminal)]
    interface: InterfaceType,
}

fn main() -> Result<(), Err> {
    let args = Args::parse();

    // Init logger
    simple_logging::log_to_file("test.log", LevelFilter::Debug)?;

    // Load ROM into CPU memory
    let rom = fs::read("roms/brick.ch8").expect("Can read ROM file");
    let mut chip_8 = Chip8::new();
    chip_8.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);
    chip_8.memory[0x1FF] = 5;
    chip_8.memory[0x1FE] = 2;

    let mut interface: Box<dyn Interface> = match args.interface {
        InterfaceType::Graphical => Box::new(Graphical::new().unwrap()),
        InterfaceType::Terminal => Box::new(Terminal::new()),
    };
    interface.run(&mut chip_8)?;

    Ok(())
}
