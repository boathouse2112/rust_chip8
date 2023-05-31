mod chip_8;
mod globals;
mod interface;

use chip_8::Chip8;
use globals::Err;
use interface::{Graphical, Interface, Terminal};
use log::LevelFilter;

use std::{fs, thread};

#[macro_use]
extern crate lazy_static;

fn main() -> Result<(), Err> {
    // Init logger
    simple_logging::log_to_file("test.log", LevelFilter::Debug)?;

    // Load ROM into CPU memory
    let rom = fs::read("roms/brick.ch8").expect("Can read ROM file");
    let mut chip_8 = Chip8::new();
    chip_8.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);
    chip_8.memory[0x1FF] = 5;
    chip_8.memory[0x1FE] = 2;

    // let mut interface = Graphical::new()?;
    let mut interface = Terminal::new();
    interface.run(&mut chip_8)?;

    Ok(())

    // let mut held_keys: HashSet<u8> = HashSet::new();
    // loop {
    //     engine.wait_frame();

    //     // Quit if CTRL-C is held down
    //     if engine.is_key_pressed_with_modifier(KeyCode::Char('c'), KeyModifiers::CONTROL) {
    //         break;
    //     }

    //     let mut held_keys_changed = false;
    //     for (&key_code, &chip_8_key) in key_code_to_chip_8_key.into_iter() {
    //         if engine.is_key_pressed(KeyCode::Char(key_code)) {
    //             held_keys_changed = true;
    //             held_keys.insert(chip_8_key);
    //         }
    //         if engine.is_key_released(KeyCode::Char(key_code)) {
    //             held_keys_changed = true;
    //             held_keys.remove(&chip_8_key);
    //         }
    //     }

    //     draw(&mut engine, &chip_8, &held_keys, held_keys_changed);

    //     // Decrement counters
    //     chip_8.decrement_counters();

    //     // Run n cycles
    //     for _ in 0..chip_8::INSTRUCTIONS_PER_FRAME {
    //         chip_8.run_cycle(&held_keys);
    //     }
    // }

    // Ok(())
}

// fn print_grid(display: HashSet<(i32, i32)>) {
//     for y in 0..chip_8::DISPLAY_HEIGHT {
//         for x in 0..chip_8::DISPLAY_WIDTH {
//             let char = if display.contains(&(x, y)) {
//                 "█"
//             } else {
//                 " "
//             };
//             print!("{} ", char);
//         }
//         println!();
//     }
// }
