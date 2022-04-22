use sdl2;

use display::Display;
use processor::Processor;

mod display;
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
    let mut processor = Processor::new();

    loop {
        let (vram, display_flag, clear_flag) = processor.emulate_cycle();

        if display_flag {
            display.render(vram);
        } else if clear_flag {
            display.clear();
        }
    }
}
