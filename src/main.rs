use crate::chip8::cursive_renderer;
use crate::chip8::raylib_renderer;
use crate::chip8::emu_utils;
use crate::chip8::emu_utils::{display_render, display_text};
use crate::chip8::Chip8;
use crate::chip8::COLS;
use crate::chip8::DISPLAY;
use crate::chip8::ECHO_SOUND;
use crate::chip8::MEMORY_SIZE;
use crate::chip8::ROWS;
use crate::chip8::ROW_LEN;
use c8_disasm_lib::decode;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use structopt::StructOpt;

const GLYPH_BLOCK: char = '\u{2588}';
const GLYPH_X: char = 'x';

#[derive(StructOpt, Debug)]
#[structopt(name = "chip8-emulator")]
struct Opt {
    #[structopt(short = "f", long = "font-check")]
    font_check: bool,

    #[structopt(short = "b", long = "bios-check")]
    bios_check: bool,

    #[structopt(short, long)]
    gui_mode: bool,

    #[structopt(short, long)]
    autorun: bool,

    #[structopt(short = "x", long = "block")]
    override_glyph: Option<char>,

    #[structopt(short, long)]
    registers: bool,

    #[structopt(short = "n", long = "iterations", default_value = "10")]
    iterations: u32,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

mod chip8;

fn main() {
    let opt: Opt = Opt::from_args();
    //println!("{:#?}", opt);

    let lang = env::var("LANG").unwrap_or("".to_string());
    let glyph = determine_display_glyph(opt.override_glyph, lang);

    if opt.font_check {
        emu_utils::dump_fonts(glyph);
        return;
    }

    // Otherwise use 'x'
    if opt.bios_check {
        bios_check(glyph);
        return;
    }

    if opt.gui_mode {
        //cursive_renderer::run_gui_emulator(opt.file.as_path(), false, glyph, opt.autorun);
        raylib_renderer::run();
    } else {
        run_emulator(opt.file.as_path(), opt.iterations, opt.registers, glyph);
    }
}

fn determine_display_glyph(override_glyph: Option<char>, lang: String) -> char {
    if override_glyph.is_some() {
        return override_glyph.unwrap();
    }
    // Return Defaults: Default to unicode BLOCK if env LANG for UTF-8 is supported
    return match lang.to_lowercase().contains("utf-8") {
        true => GLYPH_BLOCK,
        false => GLYPH_X,
    };
}

fn bios_check(glyph: char) {
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
        keyboard: 0,
        should_draw: false,
        wait_key: false,
        wait_key_v_x: 0,
    };
    chip8.load_fonts();

    chip8.load(2, 250);
    chip8.load(3, 20);
    println!("{}", chip8.v[2]);
    println!("{}", chip8.v[3]);
    chip8.add_with_carry(2, 3);
    println!("{}", chip8.v[2]);
    println!("{}", chip8.v[0xF]);
    println!("{}", ECHO_SOUND);
    display_render(&chip8, true, glyph);
    chip8.fill_screen();
    display_render(&chip8, true, glyph);
    chip8.clear_screen();
    chip8.fill_screen_other_row();
    display_render(&chip8, true, glyph);
    chip8.clear_screen();
    chip8.fill_screen_other_col();
    display_render(&chip8, true, glyph);
    chip8.clear_screen();

    display_text(&mut chip8, glyph);
}

fn run_emulator(path: &Path, iterations: u32, debug_registers: bool, glyph: char) {
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
        keyboard: 0,
        should_draw: false,
        wait_key: false,
        wait_key_v_x: 0,
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
    let graphics_state: [bool; ROWS * COLS] = [false; ROWS * COLS];

    // load fonts
    chip8.load_fonts();

    // fetch

    println!(
        "Loading {} into memory",
        path.to_str().unwrap_or("BAD_PATH")
    );
    let bytes = match fs::read(path) {
        Ok(x) => x,
        Err(_e) => {
            println!(
                "Could not read file from path: {}",
                path.to_str().unwrap_or("BAD_PATH")
            );
            return;
        }
    };
    let start = chip8::DATA;
    for i in 0..bytes.len() {
        chip8.memory[start + i] = bytes[i];
    }

    chip8.pc = 0x200;

    if debug_registers {
        console_debug_registers(&chip8);
    }

    for _ in 0..iterations {
        let (b0, b1) = chip8.fetch();
        if debug_registers {
            println!("\nfetch: {:02X}  {:02X}", b0, b1)
        }
        if debug_registers {
            decode_print_byte(b0, b1, true);
        }
        chip8.decode_execute(b0, b1);
        if debug_registers {
            console_debug_registers(&chip8);
        }
        if chip8.should_draw {
            display_render(&chip8, debug_registers, glyph);
            chip8.should_draw = false;
            // TODO Need to implement timers for sounds and delays
        }
    }
}

// taken from the c8_diasm_lib project - main code
fn decode_print_byte(b0: u8, b1: u8, should_show_ascii: bool) {
    let opcode = decode(b0, b1);

    let b0_printable = b0 == b' ' || b0.is_ascii_alphanumeric();
    let b1_printable = b1 == b' ' || b1.is_ascii_alphanumeric();
    if should_show_ascii && b0_printable && b1_printable {
        println!("{} \"{}{}\"", opcode, b0 as char, b1 as char);
    } else {
        println!("{}", opcode);
    }
}

fn console_debug_registers(chip8: &Chip8) {
    println!("PC    SP    I");
    println!("{:#X} {:#X} {:#X}", chip8.pc, chip8.sp, chip8.i);
    println!("v0 v1 v2 v3 v4 v5 v6 v7 v8 v9 va vb vc vd ve vf dt st  k\n");
    println!("{:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X}",
             chip8.v[0], chip8.v[1], chip8.v[2], chip8.v[3],
             chip8.v[4], chip8.v[5], chip8.v[6], chip8.v[7],
             chip8.v[8], chip8.v[9], chip8.v[10], chip8.v[11],
             chip8.v[12], chip8.v[13], chip8.v[14], chip8.v[15],
            chip8.timer_delay, chip8.timer_sound, chip8.keyboard);
}
