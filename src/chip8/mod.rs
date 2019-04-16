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
        pub sp: u8
}

impl Chip8 {
        // TODO cls
        pub fn ret(&mut self) {
                self.pc = self.memory[CALLSTACK + (self.sp as usize)] as u16;
                self.sp -= 1;
        }
        pub fn jump(&mut self, nnn: u8) {
                self.pc = nnn as u16;
        }
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