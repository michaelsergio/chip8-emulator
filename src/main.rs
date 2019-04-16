// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1

// 4096 memory
// start at 512 0x200
// upper 256 0xF00 - 0xFFF display refresh
// 96 below that are call stack 0xEA0 - 0xEFF
// first 512 for font data
const MEMORY_SIZE: usize = 4096;
const FONT_DATA: usize = 0x000;
const DATA: usize = 0x200;
const DISPLAY: usize = 0xF00;
const CALLSTACK: usize = 0xEA0;

const BLOCK: char = '\u{2588}';

const ROWS:usize = 32;
const COLS:usize = 64;

type Font = [u8; 5];

const FONT_SPRITES: [Font; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xF0, 0x10, 0x20, 0x40, 0x40],
    [0xF0, 0x90, 0xF0, 0x90, 0xF0],
    [0xF0, 0x90, 0xF0, 0x10, 0xF0],
    [0xF0, 0x90, 0xF0, 0x90, 0x90],
    [0xE0, 0x90, 0xE0, 0x90, 0xE0],
    [0xF0, 0x80, 0x80, 0x80, 0xF0],
    [0xE0, 0x90, 0x90, 0x90, 0xE0],
    [0xF0, 0x80, 0xF0, 0x80, 0xF0],
    [0xF0, 0x80, 0xF0, 0x80, 0x80]
];

struct Chip8 {
   memory: [u8; MEMORY_SIZE],
    
    // 16 8bit registers V0-VF
    // VF is a flag, do not use
    v: [u8; 16],

    // Address Register I is 16 bis
    address: u16,

    // Stack used for return address
    // 48 bytes for 24 levels of nesting

    // Timers - 60 hz count down until 0
    timer_delay: u8,
    // Delay Timer: get/set
    // Sound timer: when non-zero makes beep
    timer_sound: u8,

    pc: u16,
    sp: u8
}


fn main() {

    let memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

    let mut chip8 = Chip8 { 
        memory: memory,
        v: [0; 16],
        address: 0,
        timer_delay: 0,
        timer_sound: 0,
        pc: 0,
        sp: 0
    };
    

    // Input - hex keyboard: 16 keys 0-F.
    // 8,4,6,2 for directional input

    // Graphics is 64x32 monochrome
    // Sprites are 8 wide 1-15 in height
    // xor'd to screen pixels
    // Carry flag VF is set to 1 if pixels are flipped when sprite drawn or else 0

    // load fonts
    let mut i:usize = 0;
    for font in &FONT_SPRITES {
        for font_byte in &*font {
            chip8.memory[FONT_DATA + i] = *font_byte;
            i += 1;
        }
    }

    for i in 0..16 {
        let f = FONT_SPRITES[i];
        draw_font(f);
        println!("");
    }

    //draw(&chip8);

    // Opcode table
    // 35 ops 2 bytes long big endian

    // NNN: address
    //  NN: 8-bit K
    //   N: 4 bit K
    // X,Y: 4 bit register
    //  PC: program counter
    //   I: 16 bit register for memory address

    // 00E0 CLS
    // 00EE RET
    // 1NNN JMP  NNN
    // 2NNN CALL NNN *(0xNNN)()
    // 3XNN SE skip if VX == NN (then JMP)
    // 4XNN SNE skip if VX != NN (then JMP)
    // 5XY0 SE skip if X == Y
    // 6XNN LD VX to NN
    // 7XNN ADD NN to VX
    // 8XY0 LD VX to VY
    // 8XY1 OR  VX |= VY
    // 8XY2 AND VX &= VY
    // 8XY3 XOR VX ^= VY
    // 8XY4 ADD VX += VY  VF Carry
    // 8XY5 SUB VX -= VY  VF Borrow
    // 8XY6 SHR VX >>=1  lsb in VF
    // 8XY7 SUBN VX=VY-VX  VF borrow
    // 8XYE SHL VX<<=1    msb in VF
    // 9XY0 SNE skip if vx != vy
    // ANNN LD  I=NNN
    // BNNN JMP to NNN+V0
    // CXNN RND VX=rnd(0, 255) & NN
    // DXYN DRW draw at coord(vx,vy, width=8, height=n) VF=pixels flipped
    // EX9E SKP skip if key() == Vx
    // EXA1 SKNP skip if key() !=Vy
    // FX07 LD Vx=delay timer  
    // FX0A LD VX=key
    // FX15 LD delay=VX
    // FX18 LD sound=VX
    // FX1E ADD I+=VX
    // FX29 LD  I=sprite_addr[Vx] set sprite char (4x5 sprite)
    // FX33 LD set_BCD(Vx) => *(I+0)=3 *(I+1)=2 *(I+2)=1
    // FX55 LD reg_dump(Vx, &I)
    // FX65 LD reg_load(Vxm &I)
}

fn draw_font(font: Font) {
    let first: u8 = 0x80;
    for part in &font {
        for i in 0..4 {
            let val = (part << i) & first;
            let glyph = if val != (0 as u8) { BLOCK } else { ' ' };
            print!("{}", glyph);
        }
        print!("\n");
    }
}


fn draw(chip8: &Chip8) {
    for y in 0..ROWS - 1 {
        for x in 0..COLS - 1 {
            let i = y * ROWS + x;
            let val = chip8.memory[DISPLAY + i];
            let glyph = if val == 0 { BLOCK } else { ' ' };
            print!("{}", glyph);
        }
        print!("\n");
    }
}
