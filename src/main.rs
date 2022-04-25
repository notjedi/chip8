use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::fs::File;

use display::Display;
use keypad::Keypad;
use processor::Processor;

mod display;
mod keypad;
mod processor;

// TODO: try using static
const OPCODE_SIZE: usize = 2;
const CHIP8_RAM: usize = 4096;
const SCALE_FACTOR: u32 = 10;
const CHIP8_SCREEN_WIDTH: usize = 64;
const CHIP8_SCREEN_HEIGHT: usize = 32;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let mut display = Display::new(&sdl_context);
    let mut keypad = Keypad::new(&sdl_context);
    let mut processor = Processor::new();

    let mut rom =
        File::open("/home/jedi/.local/src/chip8/rust-chip8/data/PONG").expect("File not found!");
    processor.load(&mut rom);

    loop {
        let (vram, display_flag, clear_flag) = processor.emulate_cycle(&mut keypad);
        if display_flag {
            display.render(vram);
        } else if clear_flag {
            display.clear();
        }
        // processor.pretty_print();

        let event = keypad.wait_key_press_until(1);
        if let Some(event) = event {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break,
                _ => {}
            }
        }
    }
}
