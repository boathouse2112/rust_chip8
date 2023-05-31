use std::{
    collections::{HashMap, HashSet},
    thread,
    time::{Duration, Instant},
};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};

use crate::{
    chip_8::{self, Chip8},
    Err,
};

use super::Interface;

const SQUARE_SIZE: i32 = 16;

lazy_static! {
    static ref KEY_CODE_TO_CHIP_8_KEY: HashMap<Keycode, u8> = HashMap::from([
        (Keycode::Num1, 1),
        (Keycode::Num2, 2),
        (Keycode::Num3, 3),
        (Keycode::Num4, 0xC),
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

pub struct Graphical {
    sdl_context: Sdl,
    canvas: Canvas<Window>,
}

impl Graphical {
    pub fn new() -> Result<Self, Err> {
        // Initialize SDL2
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "chip_8",
                (chip_8::DISPLAY_WIDTH * SQUARE_SIZE) as u32,
                (chip_8::DISPLAY_HEIGHT * SQUARE_SIZE) as u32,
            )
            .position_centered()
            .build()?;

        let mut canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()?;

        println!("Using SDL_Renderer \"{}\"", canvas.info().name);
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let graphical = Graphical {
            sdl_context,
            canvas,
        };

        Ok(graphical)
    }
}

impl Interface for Graphical {
    fn run(&mut self, chip_8: &mut crate::chip_8::Chip8) -> Result<(), Err> {
        // Make a little texture for our white square
        let texture_creator = self.canvas.texture_creator();
        let white_square_texture = dummy_texture(&mut self.canvas, &texture_creator)?;

        // Game loop
        let mut event_pump = self.sdl_context.event_pump()?;
        let mut held_keys: HashSet<u8> = HashSet::new();

        let ns_per_frame: u64 = hertz::fps_to_ns_per_frame(chip_8::FRAMES_PER_SECOND as usize);
        let mut last_frame = Instant::now();
        'running: loop {
            // Get inputs
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        // Add chip_8-relevant keys to held_keys
                        if let Some(&chip_8_key) = KEY_CODE_TO_CHIP_8_KEY.get(&keycode) {
                            held_keys.insert(chip_8_key);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        if let Some(&chip_8_key) = KEY_CODE_TO_CHIP_8_KEY.get(&keycode) {
                            held_keys.remove(&chip_8_key);
                        }
                    }
                    _ => {}
                }
            }

            // Decrement counters
            chip_8.decrement_counters();

            // Run n cycles
            for _ in 0..chip_8::INSTRUCTIONS_PER_FRAME {
                chip_8.run_cycle(&held_keys);
            }

            draw(&mut self.canvas, &white_square_texture, &chip_8)?;

            let time_remaining =
                Duration::from_nanos(ns_per_frame).saturating_sub(last_frame.elapsed());

            thread::sleep(time_remaining);

            last_frame = Instant::now();
        }
        Ok(())
    }
}

fn dummy_texture<'a>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
) -> Result<Texture<'a>, Err> {
    let mut white_square =
        texture_creator.create_texture_target(None, SQUARE_SIZE as u32, SQUARE_SIZE as u32)?;

    canvas.with_texture_canvas(&mut white_square, |texture_canvas| {
        texture_canvas.set_draw_color(Color::RGB(255, 255, 255));
        texture_canvas.clear();
    })?;

    Ok(white_square)
}

fn draw(canvas: &mut Canvas<Window>, square_texture: &Texture, chip_8: &Chip8) -> Result<(), Err> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for (x, y) in chip_8.display.iter() {
        canvas.copy(
            square_texture,
            None,
            Rect::new(
                x * SQUARE_SIZE as i32,
                y * SQUARE_SIZE as i32,
                SQUARE_SIZE as u32,
                SQUARE_SIZE as u32,
            ),
        )?;
    }

    canvas.present();
    Ok(())

    // for x in 0..engine.get_width() {
    //     for y in 0..engine.get_height() {
    //         let cell_on = chip_8.display.contains(&(x as i32, y as i32));
    //         let color = if cell_on { Color::White } else { Color::Black };
    //         engine.set_pxl(x as i32, y as i32, pixel::pxl_bg(' ', color))
    //     }
    // }

    // engine.print(0, 0, &format!("{:?}", held_keys));

    // engine.draw();
}
