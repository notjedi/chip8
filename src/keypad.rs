use sdl2;
use sdl2::keyboard::KeyCode;

struct Keypad {
    event_pump: sdl2::EventPump,
    keys: [bool; 16],
}

impl Keypad {
    fn new(sdl_context: &sdl2::Sdl) -> Self {
        Keypad {
            event_pump = sdl_context.event_pump().unwrap(),
            keys = [false; 16],
        }
    }

    fn is_pressed(&self, key_idx: usize) -> bool {
        self.keys[key_idx]
    }

    fn wait_key_press(&self) -> KeyCode {
    }

    fn map_key(key: Keycode) -> Option<i32> {
        match key {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xC),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xD),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xE),
            Keycode::Z => Some(0xA),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xB),
            Keycode::V => Some(0xF),
            _ => None,
        }
    }

    fn set_key_state(&mut self, key: KeyCode, state: bool) {
        if let Some(idx) = Keypad::map_key(key) {
            self.keys[idx] = state;
        }
    }

}
