use sdl2;
use sdl2::rect::Rect;
use sdl2::pixels::Color;
use sdl2::{render::Canvas, video::Window};

use crate::SCALE_FACTOR;
use crate::CHIP8_SCREEN_HEIGHT;
use crate::CHIP8_SCREEN_WIDTH;

pub struct Display {
    canvas: Canvas<Window>,
}

impl Display {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "CHIP-8",
                CHIP8_SCREEN_WIDTH as u32 * SCALE_FACTOR,
                CHIP8_SCREEN_HEIGHT as u32 * SCALE_FACTOR,
            )
            .position_centered()
            .build()
            .unwrap();
            // TOOD: try .opengl()

        let mut canvas = window.into_canvas().build().unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        Display {
            canvas,
        }
    }

    // TODO: try without &mut for self
    pub fn render(&mut self, vram: &[[u8; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT]) {
        let scale = SCALE_FACTOR as usize;
        for x in 0..CHIP8_SCREEN_WIDTH {
            for y in 0..CHIP8_SCREEN_HEIGHT {
                self.canvas.set_draw_color(Display::get_color(vram[x][y]));
                let x = (x * scale) as i32;
                let y = (y * scale) as i32;
                self.canvas.fill_rect(Rect::new(x, y, SCALE_FACTOR, SCALE_FACTOR)).unwrap();
            }
        }
        self.canvas.present();
    }

    // TODO: try without &mut for self
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        self.canvas.present();
    }

    fn get_color(pixel: u8) -> Color {
        if pixel == 0 {
            Color::RGB(0, 0, 0)
        } else {
            Color::RGB(255, 255, 255)
        }
    }
}
