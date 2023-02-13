mod chip_8;
mod display_canvas;

use chip_8::Chip8;
use crossterm::{
    event::{
        self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent,
        KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use display_canvas::DisplayCanvas;
use std::{
    collections::{HashMap, HashSet},
    fs, io,
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

fn main() -> Result<(), io::Error> {
    // Set up terminal
    // enable_raw_mode()?;
    // let mut stdout = io::stdout();
    // execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    // let backend = CrosstermBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;

    // Load ROM into CPU memory
    let rom = fs::read("roms/test_suite.ch8").expect("Can read ROM file");
    let mut chip_8 = Chip8::new();
    chip_8.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);
    chip_8.memory[0x1FF] = 1;

    // Test run_instruction
    for _ in 0..1_000 {
        chip_8.run_cycle();
    }
    // println!("{:?}", cpu.display);
    print_grid(chip_8.display);

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
    // disable_raw_mode()?;
    // execute!(
    //     terminal.backend_mut(),
    //     LeaveAlternateScreen,
    //     DisableMouseCapture
    // )?;
    // terminal.show_cursor()?;

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
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &chip_8))?;
        if event::poll(Duration::new(1, 0))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    _ => {}
                }
            }
        }
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
