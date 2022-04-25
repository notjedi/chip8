use crate::Keypad;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Scancode;
use std::fs::File;
use std::io::Read;

use crate::CHIP8_RAM;
use crate::CHIP8_SCREEN_HEIGHT;
use crate::CHIP8_SCREEN_WIDTH;
use crate::OPCODE_SIZE;

static FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

impl ProgramCounter {
    fn skip_if(cond: bool) -> ProgramCounter {
        if cond {
            return ProgramCounter::Skip;
        }
        ProgramCounter::Next
    }
}

// * 16 x 8-bit general purpose registers (V0 - VF)
// * 16 x 16-bit stack implemented as an array
// * 1 x 16-bit index register (I)
// * 1 x 16-bit stack pointer (SP)
// * 1 x 16-bit program counter (PC)
// * 1 x 8-bit delay timer (DT)
// * 1 x 8-bit sound timer (ST)
// * 4096 bytes of RAM

pub struct Processor {
    reg: [u8; 16],
    stack: [usize; 16],
    ram: [u8; CHIP8_RAM],
    vram: [[u8; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT],
    pc: usize,
    sp: usize,
    i: usize,
    delay_timer: u8,
    sound_timer: u8,
    display_flag: bool,
    clear_flag: bool,
}

impl Processor {
    pub fn new() -> Self {
        let mut ram = [0u8; CHIP8_RAM];
        ram[..FONTSET.len()].clone_from_slice(&FONTSET[..]);

        Processor {
            reg: [0; 16],
            stack: [0; 16],
            ram,
            vram: [[0; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT],
            pc: 0x200,
            sp: 0,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,
            display_flag: false,
            clear_flag: false,
        }
    }

    pub fn load(&mut self, rom: &mut File) {
        let mut buf = [0u8; 3584]; // 0x1000 - 0x200 = 3584
        if rom.read(&mut buf).is_ok() {
            for (i, &byte) in buf.iter().enumerate() {
                self.ram[0x200 + i] = byte;
            }
        }
        // rom.read_exact(&mut self.ram[0x200..]).expect("Unable to read file!");
    }

    pub fn emulate_cycle(
        &mut self,
        keypad: &mut Keypad,
    ) -> (&[[u8; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT], bool, bool) {
        self.display_flag = false;
        self.clear_flag = false;
        let opcode = self.fetch_opcode();

        self.execute_opcode(opcode, keypad);
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }

        (&self.vram, self.display_flag, self.clear_flag)
    }

    fn fetch_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16
    }

    fn execute_opcode(&mut self, opcode: u16, keypad: &mut Keypad) {
        let pc_update = match opcode & 0xF000 {
            0x0000 => self.op_0(opcode),
            0x1000 => self.op_1(opcode),
            0x2000 => self.op_2(opcode),
            0x3000 => self.op_3(opcode),
            0x4000 => self.op_4(opcode),
            0x5000 => self.op_5(opcode),
            0x6000 => self.op_6(opcode),
            0x7000 => self.op_7(opcode),
            0x8000 => self.op_8(opcode),
            0x9000 => self.op_9(opcode),
            0xA000 => self.op_a(opcode),
            0xB000 => self.op_b(opcode),
            0xC000 => self.op_c(opcode),
            0xD000 => self.op_d(opcode),
            0xE000 => self.op_e(opcode, keypad),
            0xF000 => self.op_f(opcode, keypad),
            _ => {
                println!("Invalid OPCODE {}", opcode);
                ProgramCounter::Next
            }
        };

        match pc_update {
            ProgramCounter::Next => self.pc += OPCODE_SIZE,
            ProgramCounter::Skip => self.pc += 2 * OPCODE_SIZE,
            ProgramCounter::Jump(addr) => self.pc = addr,
        };
    }

    fn op_0(&mut self, opcode: u16) -> ProgramCounter {
        match Processor::get_0nn(opcode) {
            0x00E0 => {
                // 0x00E0(CLS) = Clear the screen.
                self.vram = [[0; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT];
                self.clear_flag = true;
                ProgramCounter::Next
            }
            0x00EE => {
                // 0x00EE(RET) = Return from subroutine.
                self.sp -= 1;
                ProgramCounter::Jump(self.stack[self.sp])
            }
            _ => {
                Processor::print_err(opcode);
                ProgramCounter::Next
            }
        }
    }

    fn op_1(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x1nnn(JP addr) = Jump to location nnn.
        */
        ProgramCounter::Jump(Processor::get_nnn(opcode))
    }

    fn op_2(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x2nnn(CALL addr) = Call subroutine at nnn.
        */
        // https://old.reddit.com/r/EmuDev/comments/5so1bo/chip8_emu_questions/ddibkkp/
        self.stack[self.sp] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        ProgramCounter::Jump(Processor::get_nnn(opcode))
    }

    fn op_3(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x3xkk(SE Vx, byte) = Skip next instruction if Vx == kk.
        */
        ProgramCounter::skip_if(self.reg[Processor::get_x(opcode)] == Processor::get_0nn(opcode))
    }

    fn op_4(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x4xkk(SNE Vx, byte) = Skip next instruction if Vx != kk.
        */
        ProgramCounter::skip_if(self.reg[Processor::get_x(opcode)] != Processor::get_0nn(opcode))
    }

    fn op_5(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x5xy0(SE Vx, Vy) = Skip next instruction if Vx != Vy.
        */
        ProgramCounter::skip_if(
            self.reg[Processor::get_x(opcode)] == self.reg[Processor::get_y(opcode)],
        )
    }

    fn op_6(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x6xkk(LD Vx, byte) = Load value kk into register Vx.
        */
        self.reg[Processor::get_x(opcode)] = Processor::get_0nn(opcode);
        ProgramCounter::Next
    }

    fn op_7(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x7xkk(LD Vx, byte) = Add value kk to register Vx.
        */
        let x = Processor::get_x(opcode);
        self.reg[x] = self.reg[x].wrapping_add(Processor::get_0nn(opcode));
        ProgramCounter::Next
    }

    fn op_8(&mut self, opcode: u16) -> ProgramCounter {
        let x = Processor::get_x(opcode);
        let y = Processor::get_y(opcode);

        match Processor::get_00n(opcode) {
            0x00 => {
                // 0x8xy0(LD Vx, Vy) = Load value of register Vy into Vx.
                self.reg[x] = self.reg[y];
                ProgramCounter::Next
            }
            0x01 => {
                // 0x8xy1(OR Vx, Vy) = Set Vx = Vx OR Vy.
                self.reg[x] |= self.reg[y];
                ProgramCounter::Next
            }
            0x02 => {
                // 0x8xy2(AND Vx, Vy) = Set Vx = Vx AND vy.
                self.reg[x] &= self.reg[y];
                ProgramCounter::Next
            }
            0x03 => {
                // 0x8xy3(XOR Vx, Vy) = Set Vx = Vx XOR vy.
                self.reg[x] ^= self.reg[y];
                ProgramCounter::Next
            }
            0x04 => {
                // 0x8xy4(ADD Vx, Vy) = Set Vx = Vx + Vy, set VF = carry.
                let (vx, vy) = (self.reg[x], self.reg[y]);
                let result = vx as usize + vy as usize;
                self.reg[x] = result as u8;
                self.reg[0x0F] = if result > 0xFF { 1 } else { 0 };
                ProgramCounter::Next
            }
            0x05 => {
                // 0x8xy5(SUB Vx, Vy) = Set Vx = Vx - Vy, set VF = NOT borrow.
                let (vx, vy) = (self.reg[x], self.reg[y]);
                self.reg[0x0F] = if vx > vy { 1 } else { 0 };
                self.reg[x] = self.reg[x].wrapping_sub(self.reg[y]);
                ProgramCounter::Next
            }
            0x06 => {
                // 0x8xy6(SHR Vx, Vy) = VF = Vx & 1. Set Vx = Vx SHR 1. (Shift Right)
                self.reg[0x0F] = self.reg[x] & 0x01;
                self.reg[x] >>= 1;
                ProgramCounter::Next
            }
            0x07 => {
                // 0x8xy7(SUBN Vx, Vy) = Set Vx = Vy - Vx, set VF = NOT borrow.
                let (vx, vy) = (self.reg[x], self.reg[y]);
                self.reg[0x0F] = if vy > vx { 1 } else { 0 };
                self.reg[x] = self.reg[y].wrapping_sub(self.reg[x]);
                ProgramCounter::Next
            }
            0x0E => {
                // 0x8xyE(SHL Vx, Vy) = VF = Vx & 255. Set Vx = Vx SHL 1. (Shift Left)
                self.reg[0x0F] = self.reg[x] >> 7;
                self.reg[x] <<= 1;
                ProgramCounter::Next
            }
            _ => {
                Processor::print_err(opcode);
                ProgramCounter::Next
            }
        }
    }

    fn op_9(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x9xy0(SNE Vx, Vy) = Skip next instruction if Vx != Vy.
        */
        ProgramCounter::skip_if(
            self.reg[Processor::get_x(opcode)] != self.reg[Processor::get_y(opcode)],
        )
    }

    fn op_a(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0xAnnn(LD I, addr) = Load addr nnn into I.
        */
        self.i = Processor::get_nnn(opcode);
        ProgramCounter::Next
    }

    fn op_b(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0xBnnn(JP V0, addr) = Jump to location nnn + V0.
        */
        ProgramCounter::Jump(Processor::get_nnn(opcode) + self.reg[0] as usize)
    }

    fn op_c(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0xCxkk(RND Vx, byte) = Vx = random bytes & kk.
        */
        let mut rng = rand::thread_rng();
        self.reg[Processor::get_x(opcode)] = Processor::get_0nn(opcode) & rng.gen::<u8>();
        ProgramCounter::Next
    }

    fn op_d(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0xDxyn(DRW, Vx, Vy, nibble) = Display n-byte sprite starting at
        memory location I at (Vx, Vy), set VF = collision.
        */
        let vx = self.reg[Processor::get_x(opcode)] as usize % CHIP8_SCREEN_WIDTH;
        let vy = self.reg[Processor::get_y(opcode)] as usize % CHIP8_SCREEN_HEIGHT;
        let n = Processor::get_00n(opcode) as usize;
        self.display_flag = true;
        self.reg[0x0F] = 0;

        // https://tobiasvl.github.io/blog/write-a-chip-8-emulator/#dxyn-display
        for byte in 0..n {
            let data = self.ram[self.i + byte];
            let sy = vy + byte;
            if sy == CHIP8_SCREEN_HEIGHT {
                break;
            }
            for bit in 0..8 {
                let bit_to_draw = data & (1 << (7 - bit));
                let sx = vx + bit;
                if sx == CHIP8_SCREEN_WIDTH {
                    break;
                }
                self.reg[0x0F] |= bit_to_draw & self.vram[sy][sx];
                self.vram[sy][sx] ^= bit_to_draw;
            }
        }
        ProgramCounter::Next
    }

    fn op_e(&mut self, opcode: u16, keypad: &Keypad) -> ProgramCounter {
        let x = Processor::get_x(opcode);
        match Processor::get_0nn(opcode) {
            0x9E => {
                // 0xEX9E(SKP Vx) = Skip next instruction if key with the value of Vx is pressed.
                let keycode = Keypad::unmap_key(self.reg[x]);
                ProgramCounter::skip_if(keypad.is_pressed(keycode.unwrap()))
            }
            0xA1 => {
                // 0xEXA1(SKNP Vx) = Skip next instruction if key with the value of Vx is not
                // pressed.
                let keycode = Keypad::unmap_key(self.reg[x]);
                ProgramCounter::skip_if(!keypad.is_pressed(keycode.unwrap()))
            }
            _ => {
                Processor::print_err(opcode);
                ProgramCounter::Next
            }
        }
    }

    fn op_f(&mut self, opcode: u16, keypad: &mut Keypad) -> ProgramCounter {
        let x = Processor::get_x(opcode);
        match Processor::get_0nn(opcode) {
            0x07 => {
                // Fx07(LD Vx, DT) = Set Vx = delay timer value.
                self.reg[x] = self.delay_timer;
                ProgramCounter::Next
            }
            0x0A => {
                // 0xFx0A(LD Vx, K) = Wait for a key press, store the value of the key in Vx.
                let key_idx = loop {
                    let event: Event = keypad.wait_key_press();
                    let keycode = match event {
                        Event::KeyDown {
                            keycode: Some(code),
                            ..
                        } => Some(code),
                        _ => None,
                    };
                    if let Some(keycode) = keycode {
                        let scancode = Scancode::from_keycode(keycode).unwrap();
                        let key = Keypad::map_key(scancode);
                        if let Some(key) = key {
                            break key;
                        }
                    }
                };

                self.ram[x] = key_idx;
                ProgramCounter::Next
            }
            0x15 => {
                // Fx15(LD DT) = Vx Set delay timer = Vx.
                self.delay_timer = self.reg[x];
                ProgramCounter::Next
            }
            0x18 => {
                // Fx18(LD ST, Vx) =  Set sound timer = Vx.
                self.sound_timer = self.reg[x];
                ProgramCounter::Next
            }
            0x1E => {
                // Fx1E(ADD I, Vx) = Set I = I + Vx.
                self.i += self.reg[x] as usize;
                ProgramCounter::Next
            }
            0x29 => {
                // Fx29(LD F, Vx) = Set I = location of sprite for digit Vx.
                // The program doesn't know where we stored the fontset, it can be anywhere.
                // It just requests the char(0-F) that it wants and we give it that.
                // So as each char takes up 5 bytes,
                // we calculate the offset by multiplying V[x] by 5 to get the font addr.
                self.i = (self.ram[x] as usize) * 5;
                ProgramCounter::Next
            }
            0x33 => {
                // Fx33(LD B, Vx) = Store BCD representation of Vx in memory locations I, I+1, and I+2.
                self.ram[self.i] = self.reg[x] / 100;
                self.ram[self.i + 1] = (self.reg[x] / 10) % 10;
                self.ram[self.i + 2] = (self.reg[x]) % 10;
                ProgramCounter::Next
            }
            0x55 => {
                // Fx55(LD [I], Vx) = Store registers V0 through Vx in memory starting at location I.
                for i in 0..=x {
                    self.ram[self.i + i] = self.reg[i];
                }
                ProgramCounter::Next
            }
            0x65 => {
                // Fx65(LD Vx, [I]) = Read registers V0 through Vx from memory starting at location I.
                for i in 0..=x {
                    self.reg[i] = self.ram[self.i + i];
                }
                ProgramCounter::Next
            }
            _ => {
                Processor::print_err(opcode);
                ProgramCounter::Next
            }
        }
    }

    pub fn pretty_print(&self) {
        for (i, &val) in self.reg.iter().enumerate() {
            print!("V{:01x}: {}", i, val);
            if (i + 1) % 3 == 0 {
                println!();
            } else {
                print!("\t");
            }
        }
        println!("I: {}\n", self.i);
    }

    fn get_x(opcode: u16) -> usize {
        ((opcode & 0x0F00) >> 8) as usize
    }

    fn get_y(opcode: u16) -> usize {
        ((opcode & 0x00F0) >> 4) as usize
    }

    fn get_00n(opcode: u16) -> u8 {
        (opcode & 0x000F) as u8
    }

    fn get_0nn(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    fn get_nnn(opcode: u16) -> usize {
        (opcode & 0x0FFF) as usize
    }

    fn print_err(opcode: u16) {
        println!(
            "Invalid operation in OPCODE {:04x} with args {:04x}",
            opcode & 0xF000,
            Processor::get_nnn(opcode)
        );
    }
}
