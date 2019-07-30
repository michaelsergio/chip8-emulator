extern crate rand;
use rand::random;

// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1

// 4096 memory
// start at 512 0x200
// upper 256 0xF00 - 0xFFF display refresh
// 96 below that are call stack 0xEA0 - 0xEFF
// first 512 for font data
pub const MEMORY_SIZE: usize = 4096;
pub const FONT_DATA: usize = 0x000;
pub const DATA: usize = 0x200;
pub const DISPLAY: usize = 0xF00;
pub const CALLSTACK: usize = 0xEA0;

pub const BLOCK: char = '\u{2588}';

pub const ROWS:usize = 32;
pub const COLS:usize = 64;

pub const ECHO_SOUND: char = 7 as char;

pub type Font = [u8; 5];

pub const FONT_SPRITES: [Font; 16] = [
    [0xF0, 0x90, 0x90, 0x90, 0xF0],
    [0x20, 0x60, 0x20, 0x20, 0x70],
    [0xF0, 0x10, 0xF0, 0x80, 0xF0],
    [0xF0, 0x10, 0xF0, 0x10, 0xF0],
    [0x90, 0x90, 0xF0, 0x10, 0x10],
    [0xF0, 0x80, 0xF0, 0x10, 0xF0],
    [0xF0, 0x80, 0xF0, 0x90, 0xF0],
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

pub struct Chip8 {
        pub memory: [u8; MEMORY_SIZE],

        // 16 8bit registers V0-VF
        // VF is a flag, do not use
        pub v: [u8; 16],

        // Address Register I is 16 bis
        pub address: u16,

        // Stack used for return address
        // 48 bytes for 24 levels of nesting

        // Timers - 60 hz count down until 0
        pub timer_delay: u8,
        // Delay Timer: get/set
        // Sound timer: when non-zero makes beep
        pub timer_sound: u8,

        pub pc: u16,
        pub sp: u8,
        pub i: u16
}

impl Chip8 {
        pub fn fetch(&mut self) -> (u8, u8) {
                // fetch
                let b0 = self.memory[(self.pc + 0) as usize];
                let b1 = self.memory[(self.pc + 1) as usize];
                // increment the pc
                self.pc += 2;
                return (b0, b1);
        }

        pub fn load_fonts(&mut self) {
            let mut i = 0;
            for font in &FONT_SPRITES {
                for font_byte in &*font {
                    self.memory[FONT_DATA + i] = *font_byte;
                    i += 1;
                }
            }
        }

        // 00E0
        pub fn clear_screen(&mut self) {
        // TODO cls

        }

        // 00EE
        pub fn ret(&mut self) {
            let pc0 = self.memory[CALLSTACK + (self.sp as usize * 2)];
            let pc1 = self.memory[CALLSTACK + (self.sp as usize * 2 + 1)];
            self.pc = ((pc0 as u16) << 8) | pc1 as u16;
            self.sp -= 1;
        }
        // 1nnn
        pub fn jump(&mut self, nnn: u16) {
                self.pc = nnn;
        }
        // 2nnn
        pub fn call(&mut self, nnn: u16) {
            self.sp += 1;
            let pc0 = (self.pc >> 8) as u8;
            let pc1 = (self.pc & 0xFF) as u8;
            self.memory[CALLSTACK + (self.sp as usize * 2)] = pc0;
            self.memory[CALLSTACK + (self.sp as usize * 2 + 1)] = pc1;
            self.pc = nnn
        }
        // 3xkk
        pub fn se_byte(&mut self, v_x: usize, byte: u8) {
                if (self.v[v_x] == byte) {
                        self.pc += 2;
                }
        }
        // 4xkk
        pub fn sne(&mut self, v_x: usize, byte: u8) {
                if (self.v[v_x] != byte) {
                        self.pc += 2;
                }
        }
        // 5xy0
        pub fn se_reg(&mut self, v_x: usize, v_y: usize) {
                if (self.v[v_x] == self.v[v_y]) {
                        self.pc += 2;
                }
        }
        // 6xkk
        pub fn load(&mut self, v_x: usize, kk: u8) {
                self.v[v_x] = kk;
        }
        // 7xkk
        pub fn add(&mut self, v_x: usize, kk: u8) {
                self.v[v_x] = self.v[v_x].wrapping_add(kk);
        }
        // 8xy0
        pub fn load_reg(&mut self, v_x: usize, v_y: usize) {
                self.v[v_x] = self.v[v_y];
        }
        // 8xy1
        pub fn or(&mut self, v_x: usize, v_y: usize) {
                self.v[v_x] |= self.v[v_y];
        }
        // 8xy2
        pub fn and(&mut self, v_x: usize, v_y: usize) {
                self.v[v_x] &= self.v[v_y];
        }
        // 8xy3
        pub fn xor(&mut self, v_x: usize, v_y: usize) {
                self.v[v_x] ^= self.v[v_y];
        }
        // 8xy4
        pub fn add_with_carry(&mut self, v_x: usize, v_y: usize) {
                match self.v[v_x].checked_add(self.v[v_y]) {
                        None => {
                                self.v[v_x] = self.v[v_x].wrapping_add(self.v[v_y]);
                                self.v[0xF] = 1;
                        },
                        Some (x) => {
                                self.v[v_x] = x;
                                self.v[0xF] = 0
                        },
                }
        }
        // 8xy5
        pub fn sub_with_borrow(&mut self, v_x: usize, v_y: usize) {
                match self.v[v_x].checked_sub(self.v[v_y]) {
                        None => {
                                self.v[v_x] = self.v[v_x].wrapping_sub(self.v[v_y]);
                                self.v[0xF] = 1;
                        },
                        Some (x) => {
                                self.v[v_x] = x;
                                self.v[0xF] = 0;
                        },
                }
        }
        // 8xy6
        pub fn shr(&mut self, v_x: usize, v_y: usize) {
            let x = self.v[v_x];
            self.v[0xF] = x & 0x1; // lsb underflow
            self.v[v_x] >> 1;
        }
        // 8xy7
        pub fn subn(&mut self, v_x: usize, v_y: usize) {
            let carry = match self.v[v_y] > self.v[v_x] {
                false => 0,
                true => 1
            };
            self.v[0xF] = carry;
            self.v[v_x] -= self.v[v_y];
        }
        // 8xyE
        pub fn shl(&mut self, v_x: usize) {
            let x = self.v[v_x];
            self.v[0xF] = x >> 7; // msb overflow
            self.v[v_x] << 1;
        }
        // 9xy0
        pub fn sne_v(&mut self, v_x: usize, v_y: usize) {
                if (self.v[v_x] != self.v[v_y]) {
                        self.pc += 2;
                }
        }

        // Annn
        pub fn load_i(&mut self, nnn: u16) {
                self.i = nnn;
        }

        // Bnnn
        pub fn jump_to_v0(&mut self, offset: u16) {
            self.pc = self.v[0] as u16 + offset;
        }

        //Cxkk
        pub fn rand(&mut self, v_x: usize, kk: u8) {
                let random = rand::random::<u8>();
                self.v[v_x] = random & kk;
        }

        //Dxyn
        pub fn draw(&mut self, v_x: usize, v_y:usize, n: usize) {
                // read n bytes from memory I
                // nibble is max 16 
                let mut read: [u8; 16] = [0; 16];
                for i in 0..n {
                        read[i as usize] = self.memory[(self.i + (i as u16)) as usize];
                }
                let is_erased = self.set_screen(self.v[v_x], self.v[v_y], &read[0..n]);
                if is_erased { self.v[0xF] = 1; }
                else { self.v[0xF] = 0; }
        }

        fn set_screen(&mut self, x: u8, y:u8, read: &[u8]) -> bool {
                // display at (x,y) on screen.
                // xor the bytes onto the screen
                // be sure to wrap around dispay.
                // return true if xor erases
                return false
        }



        // Ex9E
        pub fn skip_if_key_pressed(&mut self, v_x: usize) {
                if is_key_down(self.v[v_x]) { 
                    self.pc += 2;
                }
        }
        // ExA1
        pub fn skip_if_key_not_pressed(&mut self, v_x: usize) {
                if !is_key_down(self.v[v_x]) { 
                    self.pc += 2;
                }
        }
        // Fx07
        pub fn load_delay_timer(&mut self, v_x: usize) {
            self.v[v_x] = self.timer_delay;
        }
        // Fx0A
        pub fn load_key_press(&mut self, v_x: usize) {
            self.v[v_x] = get_key();
        }
        // Fx15
        pub fn set_delay_timer(&mut self, v_x: usize) {
            self.timer_delay = self.v[v_x];
        }
        // Fx18
        pub fn set_sound_timer(&mut self, v_x: usize) {
            self.timer_sound = self.v[v_x];
        }
        // Fx1E
        pub fn add_i(&mut self, v_x: usize) {
            self.i += self.v[v_x] as u16;
        }
        // Fx29
        pub fn load_font(&mut self, v_x: usize) {
            // TODO 
            // Fx29 - LD F, Vx
            // Set I = location of sprite for digit Vx.
            // The value of I is set to the location for the hexadecimal sprite 
            // corresponding to the value of Vx. 
            // See section 2.4, Display, for more information on the Chip-8 hexadecimal font.
        }

        // Fx33
        pub fn load_bcd(&mut self, v_x: usize) {
            let val =self.v[v_x];
            let hundreds = val / 100;
            let tens = val % 100 / 10;
            let ones = val % 10;
            self.memory[(self.i + 0) as usize] =  hundreds;
            self.memory[(self.i + 1) as usize] =  tens;
            self.memory[(self.i + 2) as usize] =  ones;
        }
        // Fx55
        pub fn store_registers(&mut self) {
            for i in 0..16 {
                self.memory[self.i as usize + i] = self.v[i as usize]
            }
        }
        // Fx65
        pub fn recall_registers(&mut self) {
            for i in 0..16 {
                self.v[i as usize] = self.memory[self.i as usize + i] 
            }
        }


        // Given a fetched instruction, decode and execute the function
        pub fn decode_execute(&mut self, b0: u8, b1: u8) {
                let opcode = b0 >> 4;
                let arg0 = b0 & 0x0F;
                let arg1 = b1 >> 4;
                let arg2 = b1 & 0x0F;
                let x = b0 & 0x0F;
                let y = b1 >> 4;
                let n = b1 & 0x0F;
                // println!("{} {:01x} {:01x} {:01x} ", opcode, arg0, arg1, arg2);
                if opcode == 0 { 
                        if arg0 == 0 && arg1 == 0xE {
                                if arg2 == 0 { self.clear_screen(); }
                                else if arg2 == 0xE { self.ret(); }
                        }
                        println!("SYS???");
                }
                else if opcode == 1 { self.jump(arg3(b0, b1)); }
                else if opcode == 2 { println!("CALL {:#X}", arg3(b0, b1)) }
                else if opcode == 3 { self.se_byte(x as usize, b1)}
                else if opcode == 4 { println!("SNE V{:X}, {:#X} ({})", x, b1, b1) }
                else if opcode == 5 { println!("SE V{:X}, V{:X}", x, y) }
                else if opcode == 6 { self.load(x as usize, b1) }
                else if opcode == 7 { self.add(x as usize, b1) }
                else if opcode == 8 { 
                        if arg2 == 0 { self.load_reg(x as usize, y as usize)}
                        else if arg2 == 1 { println!("OR, V{:X}, V{:X}", x, y) }
                        else if arg2 == 2 { println!("AND, V{:X} V{:X}", x, y) }
                        else if arg2 == 3 { println!("XOR, V{:X} V{:X}", x, y) }
                        else if arg2 == 4 { println!("ADD, V{:X} V{:X}", x, y) }
                        else if arg2 == 5 { println!("SUB, V{:X} V{:X}", x, y) }
                        else if arg2 == 6 { println!("SHR, V{:X} >> 1", x) }
                        else if arg2 == 7 { println!("SUBN, V{:X} V{:X}", x, y) }
                        else if arg2 == 0xE { println!("SHL, V{:X} << 1", x) }
                        else { println!("MATH???") }
                }
                else if opcode == 9 { println!("SNE V{:X}, V{:X}", x, y) }
                else if opcode == 0xA { self.load_i(arg3(b0, b1)) }
                else if opcode == 0xB { println!("JP V0, {:#X}", arg3(b0, b1)) }
                else if opcode == 0xC { self.rand(x as usize, b1) }
                else if opcode == 0xD { self.draw(x as usize, y as usize, n as usize) }
                else if opcode == 0xE { println!("SKP V{:X}", x) }
                else if opcode == 0xF { 
                        if b1 == 0x07 { println!("LD V{}, DT", x) }
                        if b1 == 0x0A { println!("LD V{}, KEY", x) }
                        if b1 == 0x15 { println!("LD DT, V{:X}", x) }
                        if b1 == 0x18 { println!("LD, ST, V{:X}", x) }
                        if b1 == 0x1E { println!("ADD I, V{:X}", x) }
                        if b1 == 0x29 { println!("LD F, V{:X}", x) }
                        if b1 == 0x33 { println!("LD BCD, V{:X}", x) }
                        if b1 == 0x55 { println!("LD [I], V{}", x) }
                        if b1 == 0x65 { println!("LD V{}, [I]", x) }
                        else { println!("??")}
                }
                else { println!("??") }
        }
}

fn arg3(b0: u8, b1: u8) -> u16 {
        return (((b0 & 0x0F) as u16) << 8) | b1 as u16;
}

fn is_key_down(key: u8) -> bool {
    // TODO
    false
}
fn get_key() -> u8 {
    // TODO
    1
}
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