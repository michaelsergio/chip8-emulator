use crate::chip8::Chip8;
use crate::chip8::Font;
use crate::chip8::COLS;
use crate::chip8::DISPLAY;
use crate::chip8::FONT_SPRITES;
use crate::chip8::MEMORY_SIZE;
use crate::chip8::ROWS;
use crate::chip8::ROW_LEN;

pub fn display_text(chip8: &mut Chip8, glyph: char) {
    chip8.clear_screen();

    // draw to (0,0) a "1"
    draw_font_to_buffer(chip8, 0, 0, 1);
    display_render(&chip8, true, glyph);

    draw_font_to_buffer(chip8, 8, 0, 3);
    display_render(&chip8, true, glyph);

    draw_font_to_buffer(chip8, 0, 6, 5);
    display_render(&chip8, true, glyph);

    draw_font_to_buffer(chip8, 62, 12, 7);
    display_render(&chip8, true, glyph);

    // mem_dump(chip8);

    chip8.clear_screen();
}

// Render as stdout
pub fn display_render(chip8: &Chip8, debug: bool, glyph: char) {
    // 32 rows x 64 cols
    // aka. 32 rows with 8 sections of 8 bits (1 byte each)
    //abcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGH
    //abcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGH
    // ... 30 more times

    for row_i in 0..ROWS {
        if debug {
            print!("{:02}:", row_i);
        }
        let row_start = DISPLAY + (row_i * ROW_LEN);
        // print each col for row
        for i in 0..(COLS / 8) {
            let byte: u8 = chip8.memory[row_start + i];
            for b in 0..8 {
                let bit = byte & (0x1 << 7 - b);
                let draw = if bit == 0 { ' ' } else { glyph };
                print!("{}", draw);
                // section >>= 1;
            }
        }
        println!("");
    }
    if debug {
        print!("   "); // padding for 01:
    }
    for _i in 0..COLS {
        print!("_");
    }
    print!("\n");
}

fn draw_font_to_buffer(chip8: &mut Chip8, x: u8, y: u8, val: u8) {
    // Put at 0,0
    chip8.v[0] = x;
    chip8.v[1] = y;
    // Draw "0"
    chip8.v[2] = val;
    chip8.load_font(2);
    chip8.draw(0, 1, 5);
}
pub fn dump_fonts(glyph: char) {
    for i in 0..16 {
        let f = FONT_SPRITES[i];
        debug_font(f, glyph);
        println!("");
    }
}

pub fn mem_dump(chip8: &Chip8) {
    for i in 0..MEMORY_SIZE {
        if i % 32 == 0 {
            print!("\n")
        } else if i % 4 == 0 {
            print!(" ")
        }
        print!("{:02X}", chip8.memory[i])
    }
}

/*
fn debug_screen(chip8: &Chip8) {
    for y in 0..32 {
        for x in 0..16 {
            let i = y * 32 + x;
            println!("\n{}, {}, {}", i, x, y);
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

fn bios_draw(chip8: &Chip8) {
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
*/

pub fn debug_font(font: Font, block: char) {
    for part in &font {
        for i in 0..4 {
            let val = (part << i) & (0x80 as u8);
            let glyph = if val != (0 as u8) { block } else { ' ' };
            print!("{}", glyph);
        }
        print!("\n");
    }
}
