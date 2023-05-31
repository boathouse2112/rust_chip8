use std::{
    collections::{HashMap, HashSet},
    io::{stdout, Stdout, Write},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Print,
    terminal::{self, ClearType},
    ExecutableCommand, QueueableCommand,
};
use log::debug;

use crate::{
    chip_8::{self, Chip8},
    globals::Err,
};

use super::Interface;

const KEY_HOLD_DURATION: Duration = Duration::from_millis(600);

lazy_static! {
    static ref KEY_CODE_TO_CHIP_8_KEY: HashMap<KeyCode, u8> = HashMap::from([
        (KeyCode::Char('1'), 1),
        (KeyCode::Char('2'), 2),
        (KeyCode::Char('3'), 3),
        (KeyCode::Char('4'), 0xC),
        (KeyCode::Char('q'), 4),
        (KeyCode::Char('w'), 5),
        (KeyCode::Char('e'), 6),
        (KeyCode::Char('r'), 0xD),
        (KeyCode::Char('a'), 7),
        (KeyCode::Char('s'), 8),
        (KeyCode::Char('d'), 9),
        (KeyCode::Char('f'), 0xE),
        (KeyCode::Char('z'), 0xA),
        (KeyCode::Char('x'), 0),
        (KeyCode::Char('c'), 0xB),
        (KeyCode::Char('v'), 0xF),
    ]);
}

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Self {
        Terminal {}
    }
}

impl Interface for Terminal {
    fn run(&mut self, chip_8: &mut Chip8) -> Result<(), crate::globals::Err> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;

        stdout.execute(terminal::Clear(ClearType::All))?;
        stdout.execute(cursor::Hide)?;

        let ns_per_frame: u64 = hertz::fps_to_ns_per_frame(chip_8::FRAMES_PER_SECOND as usize);
        let mut last_frame_end = Instant::now();

        // Game loop
        let mut held_keys: HashMap<u8, Instant> = HashMap::new();
        let mut last_frame_display = chip_8.display.clone();

        'game_loop: loop {
            let start_time = Instant::now();

            let frame_start = Instant::now();

            // Read keys
            while event::poll(Duration::ZERO)? {
                match event::read()? {
                    Event::Key(KeyEvent {
                        code: KeyCode::Esc, ..
                    })
                    | Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }) => break 'game_loop,

                    // The release event never happens, so we use duration since last keypress to mimic it.
                    Event::Key(KeyEvent { code, kind, .. }) => {
                        if let Some(&chip_8_key) = KEY_CODE_TO_CHIP_8_KEY.get(&code) {
                            held_keys.insert(chip_8_key, frame_start);
                        }
                    }

                    _ => {}
                }
            }

            // Remove stale keypresses
            held_keys = held_keys
                .into_iter()
                .filter(|(_, timestamp)| (frame_start - *timestamp) < KEY_HOLD_DURATION)
                .collect();

            let held_keys_set: HashSet<u8> = held_keys.keys().copied().collect();

            // Decrement counters
            chip_8.decrement_counters();

            // Run n cycles
            for _ in 0..chip_8::INSTRUCTIONS_PER_FRAME {
                chip_8.run_cycle(&held_keys_set);
            }

            draw(
                &chip_8.display,
                &last_frame_display,
                &held_keys_set,
                &mut stdout,
            )?;

            let time_remaining =
                Duration::from_nanos(ns_per_frame).saturating_sub(last_frame_end.elapsed());

            debug!("Time remaining: {} ms", time_remaining.as_millis());
            thread::sleep(time_remaining);

            last_frame_display = chip_8.display.clone();
            last_frame_end = Instant::now();

            let end_time = Instant::now();
            debug!("Frame duration: {} ms", (end_time - start_time).as_millis());
        }

        stdout.execute(cursor::Show)?;
        debug!("disable_raw_mode");
        terminal::disable_raw_mode()?;
        Ok(())
    }
}

fn draw(
    display: &HashSet<(i32, i32)>,
    last_frame_display: &HashSet<(i32, i32)>,
    held_keys: &HashSet<u8>,
    stdout: &mut Stdout,
) -> Result<(), Err> {
    // Get a diff between last_frame_display and display,
    // And execute changes to get to the current frame

    stdout
        .queue(cursor::MoveTo(0, 0))?
        .queue(Print(format!("{:?}              ", held_keys)))?
        .queue(cursor::MoveToNextLine(2))?;

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
