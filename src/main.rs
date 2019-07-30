use crate::chip8::Font;
use crate::chip8::BLOCK;
use crate::chip8::DISPLAY;
use crate::chip8::COLS;
use crate::chip8::ROWS;
use crate::chip8::FONT_DATA;
use crate::chip8::FONT_SPRITES;
use crate::chip8::MEMORY_SIZE;
use crate::chip8::Chip8;
use std::env;
use std::fs;


mod chip8;

fn main() {

    let memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];

    let mut chip8 = Chip8 { 
        memory: memory,
        v: [0; 16],
        address: 0,
        timer_delay: 0,
        timer_sound: 0,
        pc: 0,
        sp: 0,
        i: 0
    };
    

    // Input - hex keyboard: 16 keys 0-F.
    // 8,4,6,2 for directional input

    // Graphics is 64x32 monochrome
    // Sprites are 8 wide 1-15 in height
    // xor'd to screen pixels
    // Carry flag VF is set to 1 if pixels are flipped when sprite drawn or else 0

    // load fonts
    load_fonts(&chip8)

/*
    dump_fonts();
    chip8.load(2, 250);
    chip8.load(3, 20);
    println!("{}", chip8.v[2]);
    println!("{}", chip8.v[3]);
    chip8.add_with_carry(2, 3);
    println!("{}", chip8.v[2]);
    println!("{}", chip8.v[0xF]);
    //println!("{}", ECHO_SOUND);
    //debug_screen(&chip8);

    //draw(&chip8);
    */

    // fetch 
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: disasm ???? file");
        return
    }
    let path = &args[1];
    let bytes = fs::read(path).unwrap();

    println!("Loading {} into memory", path);
    let start = 0x200;
    for i in 0..bytes.len() {
        chip8.memory[start + i] = bytes[i];
    }

    chip8.pc = 0x200;
    debug_registers(&chip8);
    let iterations = 12;
    for _ in 0..iterations {
        run_with_debug(&mut chip8);
    }
}

fn load_fonts(chip8: &mut Chip8) {
    let mut i = 0;
    for font in &FONT_SPRITES {
        for font_byte in &*font {
            chip8.memory[FONT_DATA + i] = *font_byte;
            i += 1;
        }
    }
}


fn run_with_debug(chip8: &mut Chip8) {
    let (b0, b1) = chip8.fetch();
    
    // decode / execute
    chip8.decode_execute(b0, b1);
    debug_registers(&chip8);
}

fn debug_registers(chip8: &Chip8) {
    println!("PC    SP    I");
    println!("{:#X} {:#X} {:#X}", chip8.pc, chip8.sp, chip8.i);
    println!("v0 v1 v2 v3 v4 v5 v6 v7 v8 v9 va vb vc vd ve vf");
    println!("{:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}", 
             chip8.v[0],
             chip8.v[1],
             chip8.v[2],
             chip8.v[3],
             chip8.v[4],
             chip8.v[5],
             chip8.v[6],
             chip8.v[7],
             chip8.v[8],
             chip8.v[9],
             chip8.v[10],
             chip8.v[11],
             chip8.v[12],
             chip8.v[13],
             chip8.v[14],
             chip8.v[15]);
}

fn debug_font(font: Font) {
    for part in &font {
        for i in 0..4 {
            let val = (part << i) & (0x80 as u8);
            let glyph = if val != (0 as u8) { BLOCK } else { ' ' };
            print!("{}", glyph);
        }
        print!("\n");
    }
}


fn debug_screen(chip8: &Chip8) {
    for y in 0..32 {
        for x in 0..16 {
            let i = y * 32 + x;
            let val = chip8.memory[DISPLAY + i];
            for z in 0..4 {
                //let glyph = if z == 0 { BLOCK } else { ' ' };
                let glyph = 'x';
                print!("{}", glyph);
            }
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

fn dump_fonts() {
    for i in 0..16 {
        let f = FONT_SPRITES[i];
        debug_font(f);
        println!("");
    }
}