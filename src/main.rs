mod chip_8;
mod display_canvas;

use chip_8::Chip8;
use crossterm::{
    event::{
        self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyModifiers, ModifierKeyCode,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use display_canvas::DisplayCanvas;
use std::{
    collections::{HashMap, HashSet},
    fs, io,
    sync::mpsc::{self, Sender},
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    style::Color,
    widgets::{
        canvas::{Canvas, Context, Points, Rectangle},
        Block, Borders,
    },
    Frame, Terminal,
};

const TICKS_PER_SECOND: i32 = 700;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Load ROM into CPU memory
    let rom = fs::read("roms/brick.ch8").expect("Can read ROM file");
    let mut chip_8 = Chip8::new();
    chip_8.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);
    // chip_8.memory[0x1FF] = 5;
    // chip_8.memory[0x1FE] = 2;

    // Test run_instruction
    // for _ in 0..1_000 {
    //     chip_8.run_cycle();
    // }
    // println!("{:?}", cpu.display);
    // print_grid(chip_8.display);

    run_chip_8(&mut terminal, chip_8)?;

    // loop {
    //     terminal.draw(|f| ui(f, &chip_8))?;
    //     if event::poll(Duration::new(1, 0))? {
    //         if let Event::Key(key) = event::read()? {
    //             match key.code {
    //                 KeyCode::Char('q') => {
    //                     break;
    //                 }
    //                 _ => {}
    //             }
    //         }
    //     }
    // }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn print_grid(display: HashSet<(i32, i32)>) {
    for y in 0..chip_8::DISPLAY_HEIGHT {
        for x in 0..chip_8::DISPLAY_WIDTH {
            let char = if display.contains(&(x, y)) {
                "█"
            } else {
                " "
            };
            print!("{} ", char);
        }
        println!();
    }
}

fn run_chip_8<B: Backend>(
    terminal: &mut Terminal<B>,
    mut chip_8: Chip8,
) -> Result<(), Box<dyn std::error::Error>> {
    let ns_per_tick = hertz::fps_to_ns_per_frame(TICKS_PER_SECOND as usize);
    let tick_duration = Duration::from_nanos(ns_per_tick);
    let mut last_tick = Instant::now();
    let mut held_key: Option<u8> = None;
    // Draw, tick, wait for input.
    loop {
        // Draw this frame
        terminal.draw(|f| ui(f, &chip_8))?;

        // Run a CPU cycle
        chip_8.run_cycle(held_key);

        // Get MAX(current_time - last_tick, 0)
        let time_remaining = tick_duration
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Spend the rest of the frame waiting on input
        if event::poll(time_remaining)? {
            if let Event::Key(key) = event::read()? {
                // Return on control-c
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    return Ok(());
                }
                match key.code {
                    KeyCode::Char('1') => held_key = Some(1),
                    KeyCode::Char('2') => held_key = Some(2),
                    KeyCode::Char('3') => held_key = Some(3),
                    KeyCode::Char('4') => held_key = Some(0xC),
                    KeyCode::Char('q') => held_key = Some(4),
                    KeyCode::Char('w') => held_key = Some(5),
                    KeyCode::Char('e') => held_key = Some(6),
                    KeyCode::Char('r') => held_key = Some(0xD),
                    KeyCode::Char('a') => held_key = Some(7),
                    KeyCode::Char('s') => held_key = Some(8),
                    KeyCode::Char('d') => held_key = Some(9),
                    KeyCode::Char('f') => held_key = Some(0xE),
                    KeyCode::Char('z') => held_key = Some(0xA),
                    KeyCode::Char('x') => held_key = Some(0),
                    KeyCode::Char('c') => held_key = Some(0xB),
                    KeyCode::Char('v') => held_key = Some(0xF),
                    _ => held_key = None,
                }
            }
        }

        last_tick = Instant::now();
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, chip_8: &Chip8) {
    let display = DisplayCanvas::new(&chip_8.display);

    // let canvas = Canvas::default()
    //     .block(Block::default().borders(Borders::ALL).title("Chip 8"))
    //     .paint(|ctx| draw_rects(ctx, chip_8))
    //     .x_bounds([0.0, chip_8::DISPLAY_WIDTH as f64])
    //     .y_bounds([0.0, chip_8::DISPLAY_HEIGHT as f64]);

    f.render_widget(display, f.size());
}
