mod chip_8;

use chip_8::Chip8;
use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    video::Window,
    video::WindowContext,
};
use std::{
    collections::{HashMap, HashSet},
    fs, thread,
    time::{Duration, Instant},
};

const SQUARE_SIZE: i32 = 16;

type Err = Box<dyn std::error::Error>;

fn main() -> Result<(), Err> {
    let key_code_to_chip_8_key: HashMap<Keycode, u8> = HashMap::from([
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

    // Make a little texture for our white square
    let texture_creator = canvas.texture_creator();
    let white_square_texture = dummy_texture(&mut canvas, &texture_creator)?;

    // Load ROM into CPU memory
    let rom = fs::read("roms/brick.ch8").expect("Can read ROM file");
    let mut chip_8 = Chip8::new();
    chip_8.memory[0x200..0x200 + rom.len()].clone_from_slice(&rom[..]);
    chip_8.memory[0x1FF] = 5;
    chip_8.memory[0x1FE] = 2;

    // Game loop
    let mut event_pump = sdl_context.event_pump()?;
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
                    if let Some(&chip_8_key) = key_code_to_chip_8_key.get(&keycode) {
                        held_keys.insert(chip_8_key);
                    }
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    repeat: false,
                    ..
                } => {
                    if let Some(&chip_8_key) = key_code_to_chip_8_key.get(&keycode) {
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

        draw(&mut canvas, &white_square_texture, &chip_8)?;

        let time_remaining =
            Duration::from_nanos(ns_per_frame).saturating_sub(last_frame.elapsed());

        thread::sleep(time_remaining);
    }

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
