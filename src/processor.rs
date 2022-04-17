use rand::Rng;

const CHIP8_RAM: usize = 4096;
const OPCODE_SIZE: usize = 2;
const CHIP8_SCREEN_WIDTH: usize = 64;
const CHIP8_SCREEN_HEIGHT: usize = 32;

static FONTSET: [u8; 80]= [0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
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
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
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
}

impl Processor {
    pub fn new() -> Self {
        let mut ram = [0u8; CHIP8_RAM];
        for i in 0..FONTSET.len() {
            ram[i] = FONTSET[i];
        }

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
        }
    }

    pub fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);
    }

    fn fetch_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16
    }

    fn execute_opcode(&mut self, opcode: u16) {
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
            0xE000 => self.op_e(opcode),
            0xF000 => self.op_f(opcode),
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
        /*
        0x0NNN = Deprecated?
        0x00E0(CLS) = Clear the screen.
        0x00EE(RET) = Return from subroutine.
        */
        match opcode & 0x00FF {
            0x00E0 => {
                self.vram = [[0; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT];
                ProgramCounter::Next
            }
            0x00EE => {
                self.sp -= 1;
                ProgramCounter::Jump(self.stack[self.sp] as usize)
            }
            _ => {
                println!(
                    "Invalid operation in OPCODE {} with args {}",
                    opcode & 0xF000,
                    Processor::get_nnn(opcode)
                );
                ProgramCounter::Next
            }
        }
    }

    fn op_1(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x1NNN(JP addr) = Jump to location NNN.
        */
        ProgramCounter::Jump(Processor::get_nnn(opcode) as usize)
    }

    fn op_2(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x2NNN(CALL addr) = Call subroutine at NNN.
        */
        // https://old.reddit.com/r/EmuDev/comments/5so1bo/chip8_emu_questions/ddibkkp/
        self.stack[self.sp] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        ProgramCounter::Jump(Processor::get_nnn(opcode) as usize)
    }

    fn op_3(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x3XKK = Skip next instruction if Vx == kk.
        */
        ProgramCounter::skip_if(
            self.reg[Processor::get_x(opcode) as usize] == Processor::get_0nn(opcode),
        )
    }

    fn op_4(&mut self, opcode: u16) -> ProgramCounter {
        /*
        0x4XKK = Skip next instruction if Vx != kk.
        */
        ProgramCounter::skip_if(
            self.reg[Processor::get_x(opcode) as usize] != Processor::get_0nn(opcode),
        )
    }

    fn op_5(&mut self, opcode: u16) -> ProgramCounter {
        ProgramCounter::skip_if(
            self.reg[Processor::get_x(opcode) as usize]
                == self.reg[Processor::get_y(opcode) as usize],
        )
    }

    fn op_6(&mut self, opcode: u16) -> ProgramCounter {
        self.reg[Processor::get_x(opcode) as usize] = Processor::get_0nn(opcode);
        ProgramCounter::Next
    }

    fn op_7(&mut self, opcode: u16) -> ProgramCounter {
        // TODO: sanity check
        let x = Processor::get_x(opcode) as usize;
        self.reg[x] = self.reg[x].wrapping_add(Processor::get_0nn(opcode));
        ProgramCounter::Next
    }

    fn op_8(&mut self, opcode: u16) -> ProgramCounter {
        let x = Processor::get_x(opcode) as usize;
        let y = Processor::get_y(opcode) as usize;

        match Processor::get_00n(opcode) {
            0x00 => {
                self.reg[x] = self.reg[y];
                ProgramCounter::Next
            }
            0x01 => {
                self.reg[x] |= self.reg[y];
                ProgramCounter::Next
            }
            0x02 => {
                self.reg[x] &= self.reg[y];
                ProgramCounter::Next
            }
            0x03 => {
                self.reg[x] ^= self.reg[y];
                ProgramCounter::Next
            }
            0x04 => {
                let (vx, vy) = (self.reg[x], self.reg[y]);
                let result = vx + vy;
                self.reg[x] = result as u8;
                self.reg[0x0F] = if result > 0xFF { 1 } else { 0 };
                ProgramCounter::Next
            }
            0x05 => {
                let (vx, vy) = (self.reg[x], self.reg[y]);
                self.reg[0x0F] = if vx > vy { 1 } else { 0 };
                self.reg[x] = self.reg[x].wrapping_sub(self.reg[y]);
                ProgramCounter::Next
            }
            0x06 => {
                self.reg[0x0F] = self.reg[x] & 0x01;
                self.reg[x] >>= 1;
                ProgramCounter::Next
            }
            0x07 => {
                let (vx, vy) = (self.reg[x], self.reg[y]);
                self.reg[0x0F] = if vy > vx { 1 } else { 0 };
                self.reg[x] = self.reg[y].wrapping_sub(self.reg[x]);
                ProgramCounter::Next
            }
            0x0E => {
                // TODO: should i change this to 0b10000000?
                self.reg[0x0F] = (self.reg[x] & 256) >> 7;
                self.reg[x] <<= 1;
                ProgramCounter::Next
            }
            _ => {
                println!(
                    "Invalid operation in OPCODE {} with args {}",
                    opcode & 0xF000,
                    Processor::get_nnn(opcode)
                );
                ProgramCounter::Next
            }
        }
    }

    fn op_9(&mut self, opcode: u16) -> ProgramCounter {
        ProgramCounter::skip_if(
            self.reg[Processor::get_x(opcode) as usize]
                != self.reg[Processor::get_y(opcode) as usize],
        )
    }

    fn op_a(&mut self, opcode: u16) -> ProgramCounter {
        self.i = Processor::get_nnn(opcode) as usize;
        ProgramCounter::Next
    }

    fn op_b(&mut self, opcode: u16) -> ProgramCounter {
        ProgramCounter::Jump((Processor::get_nnn(opcode) + self.reg[0] as u16) as usize)
    }

    fn op_c(&mut self, opcode: u16) -> ProgramCounter {
        let mut rng = rand::thread_rng();
        self.reg[Processor::get_x(opcode) as usize] = Processor::get_0nn(opcode) & rng.gen::<u8>();
        ProgramCounter::Next
    }

    fn op_d(&mut self, opcode: u16) -> ProgramCounter {
        let x = Processor::get_x(opcode) as usize;
        let y = Processor::get_y(opcode) as usize;
        let vx = Processor::get_x(opcode) as usize;
        let vy = Processor::get_y(opcode) as usize;
        let n = Processor::get_00n(opcode) as usize;
        self.reg[0x0F] = 0;

        for byte in 0..n {
            let data = self.ram[self.i + byte];
            let sy = (vy + byte) % CHIP8_SCREEN_HEIGHT;
            for bit in 0..8 {
                // let bit_to_draw = (data >> (7 - bit)) & 1;
                let bit_to_draw = data & (1 << (7 - bit));
                let sx = (vx + bit) % CHIP8_SCREEN_WIDTH;
                self.reg[0x0F] |= bit_to_draw & self.vram[sy][sx];
                self.vram[sy][sx] ^= bit_to_draw;
            }
        }
        ProgramCounter::Next
    }

    fn op_e(&mut self, opcode: u16) -> ProgramCounter {
        // TODO: later
    }

    fn op_f(&mut self, opcode: u16) -> ProgramCounter {
        let x = Processor::get_x(opcode) as usize;
        let y = Processor::get_y(opcode) as usize;
        match Processor::get_nnn(opcode) {
            0x07 => {
                self.reg[x] = self.delay_timer;
                ProgramCounter::Next
            }
            0x0A => {
                // TODO: later
                ProgramCounter::Next
            }
            0x15 => {
                self.delay_timer = self.reg[x];
                ProgramCounter::Next
            }
            0x18 => {
                self.sound_timer = self.reg[x];
                ProgramCounter::Next
            }
            0x1E => {
                self.i += self.reg[x] as usize;
                ProgramCounter::Next
            }
            0x29 => {
                // The program doesn't know where we stored the fontset, it can be anywhere.
                // It just requests the char(0-F) that it wants and we give it that.
                // So as each char takes up 5 bytes,
                // we calculate the offset by multiplying V[x] by 5 to get the font addr.
                self.i = (self.ram[x] as usize) * 5;
                ProgramCounter::Next
            }
            0x33 => {
                self.ram[self.i] = self.reg[x] / 100;
                self.ram[self.i + 1] = (self.reg[x] / 10) % 10;
                self.ram[self.i + 2] = (self.reg[x]) % 10;
                ProgramCounter::Next
            }
            0x55 => {
                for i in 0..=x {
                    self.ram[self.i + i] = self.reg[i];
                }
                ProgramCounter::Next
            }
            0x65 => {
                for i in 0..=x {
                    self.reg[i] = self.ram[self.i + i];
                }
                ProgramCounter::Next
            }
            _ => {
                println!(
                    "Invalid operation in OPCODE {} with args {}",
                    opcode & 0xF000,
                    Processor::get_nnn(opcode)
                );
                ProgramCounter::Next
            }
        }
    }

    fn get_x(opcode: u16) -> u8 {
        (opcode & 0x0F00) as u8
    }

    fn get_y(opcode: u16) -> u8 {
        (opcode & 0x00F0) as u8
    }

    fn get_00n(opcode: u16) -> u8 {
        (opcode & 0x000F) as u8
    }

    fn get_0nn(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    fn get_nnn(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }
}