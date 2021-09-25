use crate::chip8::Font;
use crate::chip8::BLOCK;
use crate::chip8::DISPLAY;
use crate::chip8::COLS;
use crate::chip8::ROWS;
use crate::chip8::FONT_SPRITES;
use crate::chip8::MEMORY_SIZE;
use crate::chip8::ECHO_SOUND;
use crate::chip8::Chip8;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "chip8-emulator")]
struct Opt {

    #[structopt(short = "f", long = "font-check")]
    font_check: bool,

    #[structopt(short = "b", long = "bios-check")]
    bios_check: bool,

    #[structopt(short = "x", long = "block")]
    block: Option<String>,

    #[structopt(short, long)]
    registers: bool,

    #[structopt(short = "n", long = "iterations", default_value="10")]
    iterations: u32,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

mod chip8;

fn main() {
    let opt: Opt = Opt::from_args();
    //println!("{:#?}", opt);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: disasm ???? file");
        return
    }

    if opt.font_check {
        dump_fonts();
        return;
    }
    // TODO: Pass custom glyph along
    // Optional param
    // Default to BLOCK if env LANG for UTF-8 is supported
    // Otherwise use 'x'
    if opt.bios_check {
        bios_check();
        return;
    }

    run_emulator(opt.file.as_path(), opt.iterations, opt.registers);
}

fn bios_check() {
    let memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let mut chip8 = Chip8 { 
        memory: memory,
        v: [0; 16],
        address: 0,
        timer_delay: 0,
        timer_sound: 0,
        pc: 0,
        sp: 0,
        i: 0,
    };
    chip8.load(2, 250);
    chip8.load(3, 20);
    println!("{}", chip8.v[2]);
    println!("{}", chip8.v[3]);
    chip8.add_with_carry(2, 3);
    println!("{}", chip8.v[2]);
    println!("{}", chip8.v[0xF]);
    println!("{}", ECHO_SOUND);
    chip8.display_render();
    chip8.fill_screen();
    chip8.display_render();
   //debug_screen(&chip8);
    // bios_draw(&chip8);
}


fn run_emulator(path: &Path, iterations: u32, debug_registers: bool) {
    let memory: [u8; MEMORY_SIZE] = [0; MEMORY_SIZE];
    let mut chip8 = Chip8 { 
        memory: memory,
        v: [0; 16],
        address: 0,
        timer_delay: 0,
        timer_sound: 0,
        pc: 0,
        sp: 0,
        i: 0,
    };
    
    // Input - hex keyboard: 16 keys 0-F.
    // 1 2 3 C
    // 4 5 6 D
    // 7 8 9 E
    // A 0 B F

    let keyboard_state: [bool; 16] = [false; 16];

    // Graphics is 64x32 monochrome
    // Sprites are 8 wide 1-15 in height
    // xor'd to screen pixels
    // Carry flag VF is set to 1 if pixels are flipped when sprite drawn or else 0
    let graphics_state: [bool; ROWS*COLS] = [false; ROWS*COLS];

    // load fonts
    chip8.load_fonts();

    // fetch 

    println!("Loading {} into memory", path.to_str().unwrap_or("BAD_PATH"));
    let bytes = match fs::read(path) {
        Ok(x) => x,
        Err(_e) => {
            println!("Could not read file from path: {}", path.to_str().unwrap_or("BAD_PATH"));
            return
        }
    };
    let start = chip8::DATA;
    for i in 0..bytes.len() {
        chip8.memory[start + i] = bytes[i];
    }

    chip8.pc = 0x200;

    if debug_registers { console_debug_registers(&chip8); }

    for _ in 0..iterations {
        let (b0, b1) = chip8.fetch();
        chip8.decode_execute(b0, b1);
        if debug_registers { console_debug_registers(&chip8); }
    }
}

fn console_debug_registers(chip8: &Chip8) {
    println!("PC    SP    I");
    println!("{:#X} {:#X} {:#X}", chip8.pc, chip8.sp, chip8.i);
    println!("v0 v1 v2 v3 v4 v5 v6 v7 v8 v9 va vb vc vd ve vf");
    println!("{:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}  {:X}", 
             chip8.v[0], chip8.v[1], chip8.v[2], chip8.v[3],
             chip8.v[4], chip8.v[5], chip8.v[6], chip8.v[7],
             chip8.v[8], chip8.v[9], chip8.v[10], chip8.v[11],
             chip8.v[12], chip8.v[13], chip8.v[14], chip8.v[15]);
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

fn dump_fonts() {
    for i in 0..16 {
        let f = FONT_SPRITES[i];
        debug_font(f);
        println!("");
    }
}