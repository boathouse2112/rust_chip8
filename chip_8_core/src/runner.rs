use crate::globals::{self, Err, Keys};
use crate::interface::Interface;
use crate::Chip8;
use clap::{Parser, ValueEnum};

use std::thread;
use std::time::{Duration, Instant};

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

pub fn run(interface: &mut Box<dyn Interface>) -> Result<(), Err> {
    let mut chip_8 = Chip8::new();

    interface.load_rom(&mut chip_8)?;
    interface.setup()?;

    let ns_per_frame: u64 = hertz::fps_to_ns_per_frame(globals::FRAMES_PER_SECOND as usize);

    // Game loop
    let mut last_frame_end = Instant::now();
    let mut last_frame_display = chip_8.display.clone();
    loop {
        let frame_start = Instant::now();

        let held_keys = interface.read_keys()?;

        let Keys::Keys(held_keys) = held_keys else {
            break;
        };

        chip_8.decrement_counters();

        // Run n cycles
        for _ in 0..globals::INSTRUCTIONS_PER_FRAME {
            chip_8.run_cycle(&held_keys);
        }

        interface.draw(&mut chip_8)?;

        let time_remaining =
            Duration::from_nanos(ns_per_frame).saturating_sub(last_frame_end.elapsed());
        // debug!("Time remaining: {} ms", time_remaining.as_millis());

        thread::sleep(time_remaining);
        let frame_end = Instant::now();

        // debug!(
        //     "Frame duration: {} ms",
        //     (frame_end - frame_start).as_millis()
        // );

        last_frame_display = chip_8.display.clone();
        last_frame_end = frame_end;
    }

    interface.cleanup()?;
    Ok(())
}
