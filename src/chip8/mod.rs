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
        // TODO cls
        pub fn ret(&mut self) {
                self.pc = self.memory[CALLSTACK + (self.sp as usize)] as u16;
                self.sp -= 1;
        }
        pub fn jump(&mut self, nnn: u16) {
                self.pc = nnn;
        }
        // 3xkk
        pub fn se_byte(&mut self, v_x: usize, byte: u8) {
                if (self.v[v_x] == byte) {
                        self.pc += 2;
                }
        }
        pub fn sne(&mut self, v_x: usize, byte: u8) {
                if (self.v[v_x] != byte) {
                        self.pc += 2;
                }
        }
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
        pub fn and(&mut self, v_x: usize, v_y: usize) {
                self.v[v_x] &= self.v[v_y];
        }
        pub fn xor(&mut self, v_x: usize, v_y: usize) {
                self.v[v_x] ^= self.v[v_y];
        }
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
        // A 
        pub fn loadI(&mut self, nnn: u16) {
                self.i = nnn;
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
                let isErased = self.setScreen(self.v[v_x], self.v[v_y], &read[0..n]);
                if isErased { self.v[0xF] = 1; }
                else { self.v[0xF] = 0; }
        }

        fn setScreen(&mut self, x: u8, y:u8, read: &[u8]) -> bool {
                // display at (x,y) on screen.
                // xor the bytes onto the screen
                // be sure to wrap around dispay.
                // return true if xor erases
                return false
        }

        pub fn clearScreen(&mut self) {}


        // Given a fetched instruction, decode and execute the function
        pub fn decodeExecute(&mut self, b0: u8, b1: u8) {
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
                                if arg2 == 0 { self.clearScreen(); }
                                else if arg2 == 0xE { self.ret(); }
                        }
                        println!("SYS???");
                }
                else if opcode == 1 { self.jump(arg3(b0, b1)); }
                else if opcode == 2 { println!("CALL {:#X}", arg3(b0, b1)) }
                else if opcode == 3 { self.se_byte(x as usize, b1)}
                else if opcode == 4 { println!("SNE V{:X}, {:#X} ({})", x, b1, b1) }
                else if opcode == 5 { println!("SE V{:X}, V{:X}", x, y) }
                else if opcode == 6 { println!("LD V{:X}, {:#X} ({})", b0 & 0x0F, b1, b1) }
                else if opcode == 7 { self.add(x as usize, b1) }
                else if opcode == 8 { 
                        if arg2 == 0 { println!("LD V{:X} = V{:X} ", x, y); }
                        else if arg2 == 1 { println!("OR, V{:X}, V{:X}", x, y) }
                        else if arg2 == 2 { println!("AND, V{:X} V{:X}", x, y) }
                        else if arg2 == 3 { println!("XOR, V{:X} V{:X}", x, y) }
                        else if arg2 == 4 { println!("ADD, V{:X} V{:X}", x, y) }
                        else if arg2 == 5 { println!("SUB, V{:X} V{:X}", x, y) }
                        else if arg2 == 6 { println!("SHR, V{:X} >> 1", x) }
                        else if arg2 == 7 { println!("SUBN, V{:X} V{:X}", x, y) }
                        else if arg2 == 0xE { println!("SHL, V{:X} << 1", x) }
                        println!("MATH???") 
                }
                else if opcode == 9 { println!("SNE V{:X}, V{:X}", x, y) }
                else if opcode == 0xA { self.loadI(arg3(b0, b1)) }
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