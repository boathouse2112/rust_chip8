use std::{
    collections::{HashMap, HashSet},
    io::{stdout, Stdout, Write},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    style::Print,
    terminal::{self, ClearType},
    QueueableCommand,
};
use device_query::{DeviceQuery, DeviceState, Keycode};
use log::debug;

use crate::{
    chip_8::{self, Chip8},
    globals::Err,
};

use super::Interface;

lazy_static! {
    static ref KEY_CODE_TO_CHIP_8_KEY: HashMap<Keycode, u8> = HashMap::from([
        (Keycode::Key1, 1),
        (Keycode::Key2, 2),
        (Keycode::Key3, 3),
        (Keycode::Key4, 0xC),
        (Keycode::Q, 4),
        (Keycode::W, 5),
        (Keycode::E, 6),
        (Keycode::R, 0xD),
        (Keycode::A, 7),
        (Keycode::S, 8),
        (Keycode::D, 9),
        (Keycode::F, 0xE),
        (Keycode::Z, 0xA),
        (Keycode::X, 0),
        (Keycode::C, 0xB),
        (Keycode::V, 0xF),
    ]);
}

pub struct Terminal {
    stdout: Stdout,
    device_state: DeviceState,
}

impl Terminal {
    pub fn new() -> Self {
        let stdout = stdout();
        let device_state = DeviceState::new();
        Terminal {
            stdout,
            device_state,
        }
    }
}

impl Interface for Terminal {
    fn run(&mut self, chip_8: &mut Chip8) -> Result<(), crate::globals::Err> {
        // Terminal setup
        terminal::enable_raw_mode()?;
        self.stdout
            .queue(terminal::EnterAlternateScreen)?
            .queue(terminal::Clear(ClearType::All))?
            .queue(cursor::Hide)?;

        let ns_per_frame: u64 = hertz::fps_to_ns_per_frame(chip_8::FRAMES_PER_SECOND as usize);

        // Game loop
        let mut last_frame_end = Instant::now();
        let mut last_frame_display = chip_8.display.clone();
        loop {
            let frame_start = Instant::now();

            let mut held_keys = HashSet::new();

            // Read keys
            let keys = self.device_state.get_keys();

            // Break out if ESC or CTRL-C are pressed
            if keys.contains(&Keycode::Escape)
                || (keys.contains(&Keycode::LControl) && keys.contains(&Keycode::C))
            {
                break;
            }

            for key_code in keys {
                if let Some(&chip_8_key) = KEY_CODE_TO_CHIP_8_KEY.get(&key_code) {
                    held_keys.insert(chip_8_key);
                }
            }

            // Decrement counters
            chip_8.decrement_counters();

            // Run n cycles
            for _ in 0..chip_8::INSTRUCTIONS_PER_FRAME {
                chip_8.run_cycle(&held_keys);
            }

            draw(&chip_8.display, &last_frame_display, &mut self.stdout)?;

            let time_remaining =
                Duration::from_nanos(ns_per_frame).saturating_sub(last_frame_end.elapsed());
            debug!("Time remaining: {} ms", time_remaining.as_millis());

            thread::sleep(time_remaining);
            let frame_end = Instant::now();

            debug!(
                "Frame duration: {} ms",
                (frame_end - frame_start).as_millis()
            );

            last_frame_display = chip_8.display.clone();
            last_frame_end = frame_end;
        }

        // Cleanup
        self.stdout
            .queue(cursor::Show)?
            .queue(terminal::LeaveAlternateScreen)?;
        terminal::disable_raw_mode()?;

        Ok(())
    }
}

fn draw(
    display: &HashSet<(i32, i32)>,
    last_frame_display: &HashSet<(i32, i32)>,
    stdout: &mut Stdout,
) -> Result<(), Err> {
    // Get a diff between last_frame_display and display,
    // And execute changes to get to the current frame

    #[derive(Clone, Copy, Debug)]
    enum Op {
        Add(i32, i32),
        Remove(i32, i32),
    }

    let removed = last_frame_display - display;
    let removed = removed.iter().map(|&(x, y)| Op::Remove(x, y));

    let added = display - last_frame_display;
    let added = added.iter().map(|&(x, y)| Op::Add(x, y));

    let mut ops: Vec<_> = removed.chain(added).collect();
    ops.sort_by_key(|&op| match op {
        Op::Add(_, y) => y,
        Op::Remove(_, y) => y,
    });

    debug!("{:?}", ops);

    for op in ops.into_iter() {
        match op {
            Op::Add(x, y) => {
                stdout
                    .queue(cursor::MoveTo(x as u16, y as u16))?
                    .queue(Print("█"))?;
            }
            Op::Remove(x, y) => {
                stdout
                    .queue(cursor::MoveTo(x as u16, y as u16))?
                    .queue(Print(" "))?;
            }
        }
    }

    stdout.flush()?;

    Ok(())
}
