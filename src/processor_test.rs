use super::*;

fn build_processor() -> Processor {
    let mut processor = Processor::new();
    processor.pc = 0x200;
    processor
}

fn get_keypad() -> Keypad {
    let sdl_context = sdl2::init().unwrap();
    Keypad::new(&sdl_context)
}

#[test]
fn test_load_rom() {
    // TODO: test a ROM with more than the capacity of RAM
    let mut processor = Processor::new();
    processor.load(&[1, 2, 3]);
    assert_eq!(processor.ram[0x200], 1);
    assert_eq!(processor.ram[0x201], 2);
    assert_eq!(processor.ram[0x202], 3);
}

#[test]
fn test_emulate_cycle() {
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.delay_timer = 10;
    processor.sound_timer = 10;
    processor.emulate_cycle(&mut keypad);

    assert_eq!(processor.delay_timer, 9);
    assert_eq!(processor.delay_timer, 9);
    assert_eq!(processor.pc, 0x200 + OPCODE_SIZE);

    processor.delay_timer = 0;
    processor.sound_timer = 0;
    processor.emulate_cycle(&mut keypad);

    assert_eq!(processor.delay_timer, 0);
    assert_eq!(processor.delay_timer, 0);
    assert_eq!(processor.pc, 0x200 + (2 * OPCODE_SIZE));

    assert_eq!(processor.display_flag, false);
    assert_eq!(processor.clear_flag, false);
}

#[test]
fn test_op_00e0() {
    // CLS
    let mut processor = Processor::new();
    let mut keypad = get_keypad();
    processor.execute_opcode(0x00E0, &mut keypad);

    assert_eq!(
        processor.vram,
        [[0; CHIP8_SCREEN_WIDTH]; CHIP8_SCREEN_HEIGHT]
    );
    assert_eq!(processor.clear_flag, true);
    assert_eq!(processor.pc, 0x200 + OPCODE_SIZE);
}

#[test]
fn test_op_00ee() {
    // RET
    let mut processor = Processor::new();
    let mut keypad = get_keypad();
    processor.sp = 4;
    processor.stack[3] = 0x400;

    processor.execute_opcode(0x00EE, &mut keypad);
    assert_eq!(processor.sp, 3);
    assert_eq!(processor.pc, 0x400);
}

#[test]
fn test_op_1nnn() {
    // JP addr
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.execute_opcode(0x1444, &mut keypad);
    assert_eq!(processor.pc, 0x444);
}

#[test]
fn test_op_2nnn() {
    // CALL addr
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.execute_opcode(0x2777, &mut keypad);
    assert_eq!(processor.stack[0], 0x200 + OPCODE_SIZE);
    assert_eq!(processor.sp, 1);
    assert_eq!(processor.pc, 0x777);
}

#[test]
fn test_3xkk() {
    // 0x3xkk(SE Vx, byte) = Skip next instruction if Vx == kk.
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.reg[7] = 0x22;
    processor.execute_opcode(0x3744, &mut keypad);
    assert_eq!(processor.pc, 0x200 + OPCODE_SIZE);

    processor.execute_opcode(0x3722, &mut keypad);
    assert_eq!(processor.pc, 0x202 + (2 * OPCODE_SIZE));
}

#[test]
fn test_4xkk() {
    // 0x4xkk(SNE Vx, byte) = Skip next instruction if Vx != kk.
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.reg[7] = 0x22;
    processor.execute_opcode(0x4744, &mut keypad);
    assert_eq!(processor.pc, 0x200 + (2 * OPCODE_SIZE));

    processor.execute_opcode(0x4722, &mut keypad);
    assert_eq!(processor.pc, 0x204 + OPCODE_SIZE);
}

#[test]
fn test_5xy0() {
    // 0x5xy0(SE Vx, Vy) = Skip next instruction if Vx == Vy.
    let mut processor = Processor::new();
    let mut keypad = get_keypad();
    let x = 2;
    let y = 8;

    processor.reg[x] = 0x10;
    processor.reg[y] = 0x10;
    processor.execute_opcode(0x5280, &mut keypad);
    assert_eq!(processor.pc, 0x200 + (2 * OPCODE_SIZE));

    processor.reg[y] = 0x20;
    processor.execute_opcode(0x5280, &mut keypad);
    assert_eq!(processor.pc, 0x204 + OPCODE_SIZE);
}

#[test]
fn test_6xkk() {
    // 0x6xkk(LD Vx, byte) = Load value kk into register Vx.
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.execute_opcode(0x6522, &mut keypad);
    assert_eq!(processor.reg[0x5], 0x22);
}

#[test]
fn test_7xkk() {
    // 0x7xkk(LD Vx, byte) = Add value kk to register Vx.
    let mut processor = Processor::new();
    let mut keypad = get_keypad();

    processor.execute_opcode(0x7588, &mut keypad);
    assert_eq!(processor.reg[0x5], 0x88);

    // TODO: check this
    processor.execute_opcode(0x75FE, &mut keypad);
    assert_eq!(processor.reg[0x5], 0x87);
}
