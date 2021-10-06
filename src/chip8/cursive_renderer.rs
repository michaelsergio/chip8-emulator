use cursive::CursiveRunnable;
use crate::DISPLAY;
use crate::COLS;
use crate::ROWS;
use crate::ROW_LEN;
use crate::chip8::DATA;
use crate::chip8::MEMORY_SIZE;
use crate::Chip8;
use cursive::view::Resizable;
use cursive::views::DummyView;
use cursive::views::LinearLayout;
use cursive::views::TextContent;
use cursive::views::{TextView};
use std::fs;
use c8_disasm_lib::decode;
use std::path::Path;

pub fn display_render_gui(chip8: &Chip8, glyph: char, tv: &TextContent) {
    // 32 rows x 64 cols
    // aka. 32 rows with 8 sections of 8 bits (1 byte each)
    //abcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGH
    //abcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGHabcdefghABCDEFGH
    // ... 30 more times

    tv.set_content("");
    for row_i in 0..ROWS {
        let row_start = DISPLAY + (row_i * ROW_LEN);
        // print each col for row
        for i in 0..(COLS / 8) {
                let byte: u8 = chip8.memory[row_start + i];
                for b in 0..8 {
                        let bit = byte & (0x1 << 7 - b);
                        let draw = if bit == 0 { ' ' } else { glyph };
                        tv.append(draw);
                }
        }
        tv.append("\n");
    }
}

pub fn decode_print_byte_gui(tv: &TextContent, b0: u8, b1: u8, should_show_ascii: bool) {
    let opcode = decode(b0, b1);

    let b0_printable = b0 == b' ' || b0.is_ascii_alphanumeric();
    let b1_printable = b1 == b' ' || b1.is_ascii_alphanumeric();
    if should_show_ascii && b0_printable && b1_printable {
        tv.set_content(format!("{} \"{}{}\"", opcode, b0 as char, b1 as char));
    } else {
        tv.set_content(format!("{}", opcode));
    }
}

pub fn gui_debug_registers(chip8: &Chip8, tv: &TextContent) {
    tv.set_content("PC    SP    I\n");
    tv.append(format!("{:#X} {:#X} {:#X}\n", chip8.pc, chip8.sp, chip8.i));
    tv.append("v0 v1 v2 v3 v4 v5 v6 v7 v8 v9 va vb vc vd ve vf dt st  k\n");
    tv.append(format!("{:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X} {:2X}", 
            chip8.v[0], chip8.v[1], chip8.v[2], chip8.v[3],
            chip8.v[4], chip8.v[5], chip8.v[6], chip8.v[7],
            chip8.v[8], chip8.v[9], chip8.v[10], chip8.v[11],
            chip8.v[12], chip8.v[13], chip8.v[14], chip8.v[15],
            chip8.timer_delay, chip8.timer_sound, chip8.keyboard
    ));
}

pub fn run_gui_emulator(path: &Path, debug_registers: bool, glyph: char, should_autorun: bool) {
    let mut siv = cursive::default();
    let mut display_tv = TextView::new("Waiting to draw to display...");
    let mut op_tv = TextView::new("Press \"n\" to run!");
    let mut register_tv = TextView::new("Registers");
    let display_content = display_tv.get_shared_content();
    let op_content = op_tv.get_shared_content();
    let register_content = register_tv.get_shared_content();
    // siv.add_layer(op_tv);
    // siv.add_layer(display_tv);
    siv.add_layer(LinearLayout::vertical()
        .child(display_tv)
        .child(DummyView.fixed_height(1))
        .child(op_tv)
        .child(DummyView.fixed_height(1))
        .child(register_tv)
    );
    siv.add_global_callback('q', |s| s.quit());
        //Dialog::around(tv).title("Cursive").button("Quit", |s| s.quit()));
    setup_gui(&mut siv);

    let memory: [u8; MEMORY_SIZE] =  [0; MEMORY_SIZE];
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
    let start = DATA;
    for i in 0..bytes.len() {
        chip8.memory[start + i] = bytes[i];
    }

    chip8.pc = 0x200;

    // Starts the event loop.

    // siv.set_user_data(chip8);
    // siv.add_global_callback('n', move |s| {
    // });

    siv.set_autorefresh(should_autorun);
    std::thread::spawn(move|| {
        loop {
            // Next step
            let (b0, b1) = chip8.fetch();
            chip8.decode_execute(b0, b1);
            decode_print_byte_gui(&op_content, b0, b1, true);
            gui_debug_registers(&chip8, &register_content);
            if chip8.should_draw {
                display_render_gui(&chip8, glyph, &display_content);
                chip8.should_draw = false;
                // TODO Need to implement timers for sounds and delays
            }
            std::thread::sleep_ms(167)
        }
    });
    siv.run();
}

fn setup_gui(siv: &mut CursiveRunnable) {
    // Creates a dialog with a single "Quit" button
    // siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
    //                      .title("Cursive")
    //                      .button("Quit", |s| s.quit()));
}
