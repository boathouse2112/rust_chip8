use chip_8_core::{
    chip_8,
    globals::{self, Err, Keys},
    Chip8, Interface,
};
use lazy_static::lazy_static;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl,
};
use std::{
    collections::{HashMap, HashSet},
    fs, thread,
    time::{Duration, Instant},
};

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

    held_keys: HashSet<u8>,
}

impl Graphical {
    pub fn new() -> Result<Self, Err> {
        // Initialize SDL2
        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "chip_8",
                (globals::DISPLAY_WIDTH * SQUARE_SIZE) as u32,
                (globals::DISPLAY_HEIGHT * SQUARE_SIZE) as u32,
            )
            .position_centered()
            .build()?;

        let canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()?;

        let graphical = Graphical {
            sdl_context,
            canvas,

            held_keys: HashSet::new(),
        };

        Ok(graphical)
    }
}

impl Interface for Graphical {
    fn run(&mut self, chip_8: &mut Chip8) -> Result<(), Err> {
        // Make a little texture for our white square
        let texture_creator = self.canvas.texture_creator();
        let white_square_texture = dummy_texture(&mut self.canvas, &texture_creator)?;

        // Game loop
        let mut event_pump = self.sdl_context.event_pump()?;
        let mut held_keys: HashSet<u8> = HashSet::new();

        let ns_per_frame: u64 = hertz::fps_to_ns_per_frame(globals::FRAMES_PER_SECOND as usize);
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
            for _ in 0..globals::INSTRUCTIONS_PER_FRAME {
                chip_8.run_cycle(&held_keys);
            }

            draw(&mut self.canvas, &white_square_texture, &chip_8.display)?;

            let time_remaining =
                Duration::from_nanos(ns_per_frame).saturating_sub(last_frame.elapsed());

            thread::sleep(time_remaining);

            last_frame = Instant::now();
        }
        Ok(())
    }

    fn setup(&mut self) -> Result<(), Err> {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.present();
        Ok(())
    }

    fn load_rom(&mut self, chip_8: &mut Chip8) -> Result<(), Err> {
        // Load ROM into CPU memory
        let rom =
            fs::read("roms/brick.ch8").expect("should be able to read ROM file at roms/brick.ch8");
        chip_8.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);
        chip_8.memory[0x1FF] = 5;
        chip_8.memory[0x1FE] = 2;

        Ok(())
    }

    fn read_keys(&mut self) -> Result<Keys, Err> {
        // Get an event pump to read keys
        let mut event_pump = self.sdl_context.event_pump()?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Ok(Keys::Break),
                Event::KeyDown {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    // Add chip_8-relevant keys to held_keys
                    if let Some(&chip_8_key) = KEY_CODE_TO_CHIP_8_KEY.get(&keycode) {
                        self.held_keys.insert(chip_8_key);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    if let Some(&chip_8_key) = KEY_CODE_TO_CHIP_8_KEY.get(&keycode) {
                        self.held_keys.remove(&chip_8_key);
                    }
                }
                _ => {}
            }
        }
        Ok(Keys::Keys(self.held_keys.clone()))
    }

    fn draw(&mut self, chip_8: &mut Chip8) -> Result<(), Err> {
        // Make a little texture for our white square
        let texture_creator = self.canvas.texture_creator();
        let white_square_texture = dummy_texture(&mut self.canvas, &texture_creator)?;

        // Draw with that texture
        draw(&mut self.canvas, &white_square_texture, &chip_8.display)
    }

    fn cleanup(&mut self) -> Result<(), Err> {
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

fn draw(
    canvas: &mut Canvas<Window>,
    square_texture: &Texture,
    display: &chip_8::Display,
) -> Result<(), Err> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    for (x, y) in display.iter() {
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
}
