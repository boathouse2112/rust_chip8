mod interface;

use chip_8_core::{globals::Err, runner, Interface};
use clap::{command, Parser, ValueEnum};
use interface::{Graphical, Terminal};
use log::LevelFilter;

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

    let mut interface: Box<dyn Interface> = match args.interface {
        InterfaceType::Graphical => Box::new(Graphical::new().unwrap()),
        InterfaceType::Terminal => Box::new(Terminal::new()),
    };
    runner::run(&mut interface)?;

    Ok(())
}
