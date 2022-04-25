use sdl2::event::Event;
use sdl2::keyboard::Scancode;

pub struct Keypad {
    event_pump: sdl2::EventPump,
}

impl Keypad {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        Keypad {
            event_pump: sdl_context.event_pump().unwrap(),
        }
    }

    pub fn is_pressed(&self, keycode: Scancode) -> bool {
        self.event_pump
            .keyboard_state()
            .is_scancode_pressed(keycode)
    }

    pub fn wait_key_press(&mut self) -> Event {
        self.event_pump.wait_event()
    }

    pub fn wait_key_press_until(&mut self, timeout: u32) -> Option<Event> {
        self.event_pump.wait_event_timeout(timeout)
    }

    pub fn map_key(key: Scancode) -> Option<u8> {
        match key {
            Scancode::Num1 => Some(0x1),
            Scancode::Num2 => Some(0x2),
            Scancode::Num3 => Some(0x3),
            Scancode::Num4 => Some(0xC),
            Scancode::Q => Some(0x4),
            Scancode::W => Some(0x5),
            Scancode::E => Some(0x6),
            Scancode::R => Some(0xD),
            Scancode::A => Some(0x7),
            Scancode::S => Some(0x8),
            Scancode::D => Some(0x9),
            Scancode::F => Some(0xE),
            Scancode::Z => Some(0xA),
            Scancode::X => Some(0x0),
            Scancode::C => Some(0xB),
            Scancode::V => Some(0xF),
            _ => None,
        }
    }

    pub fn unmap_key(key: u8) -> Option<Scancode> {
        match key {
            0x01 => Some(Scancode::Num1),
            0x02 => Some(Scancode::Num2),
            0x03 => Some(Scancode::Num3),
            0x0C => Some(Scancode::Num4),
            0x04 => Some(Scancode::Q),
            0x05 => Some(Scancode::W),
            0x06 => Some(Scancode::E),
            0x0D => Some(Scancode::R),
            0x07 => Some(Scancode::A),
            0x08 => Some(Scancode::S),
            0x09 => Some(Scancode::D),
            0x0E => Some(Scancode::F),
            0x0A => Some(Scancode::Z),
            0x00 => Some(Scancode::X),
            0x0B => Some(Scancode::C),
            0x0F => Some(Scancode::V),
            _ => None,
        }
    }
}
