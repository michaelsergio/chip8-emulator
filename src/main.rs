use crate::chip8::Font;
use crate::chip8::BLOCK;
use crate::chip8::DISPLAY;
use crate::chip8::COLS;
use crate::chip8::ROWS;
use crate::chip8::FONT_DATA;
use crate::chip8::FONT_SPRITES;
use crate::chip8::MEMORY_SIZE;
use crate::chip8::Chip8;

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