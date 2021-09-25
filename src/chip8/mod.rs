extern crate rand;

// http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.1

pub const MEMORY_SIZE: usize = 4096; // 4k memory
pub const FONT_DATA: usize = 0x000;  // first 512 for font data
pub const DATA: usize = 0x200;       // start at 512 0x200
pub const DISPLAY: usize = 0xF00;    // upper 256 0xF00 - 0xFFF display refresh
pub const CALLSTACK: usize = 0xEA0;  // 96 below that are call stack 0xEA0 - 0xEFF

pub const ROWS: usize = 32;
pub const COLS: usize = 64;
pub const ROW_LEN: usize = COLS / 8;

pub const ROW_SIZE_BYTE: usize = ROWS / 8;
pub const COL_SIZE_BYTE: usize = COLS / 8;


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

        pub v: [u8; 16], // 16 8bit registers V0-VF // VF is a flag, do not use

        pub address: u16, // Address Register I is 16 bis

        // Stack used for return address
        // 48 bytes for 24 levels of nesting

        // Timers - 60 hz count down until 0
        pub timer_delay: u8, // Delay Timer: get/set
        pub timer_sound: u8, // Sound timer: when non-zero makes beep

        pub pc: u16,
        pub sp: u8,
        pub i: u16,

        pub should_draw: bool, // Custom regster to know if drw has been invoked.
        pub should_sound: bool, // Custom regster to know if drw has been invoked.
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
                for i in 0..(COLS * ROWS / 8) {
                        self.memory[DISPLAY + i] = 0
                }
        }
        // Not an instruction but for debugging
        pub fn fill_screen(&mut self) {
                for i in 0..(COLS * ROWS / 8) {
                        self.memory[DISPLAY + i] = 0xFF
                }
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
                if self.v[v_x] == byte {
                        self.pc += 2;
                }
        }
        // 4xkk
        pub fn sne(&mut self, v_x: usize, byte: u8) {
                if self.v[v_x] != byte {
                        self.pc += 2;
                }
        }
        // 5xy0
        pub fn se_reg(&mut self, v_x: usize, v_y: usize) {
                if self.v[v_x] == self.v[v_y] {
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
            self.v[v_x] >>= 1;
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
            self.v[v_x] <<= 1;
        }
        // 9xy0
        pub fn sne_v(&mut self, v_x: usize, v_y: usize) {
                if self.v[v_x] != self.v[v_y] {
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


        
        // X and Y must be inside the screen bounds.
        fn screen_bit_write(&mut self, x: usize, y:usize, set: bool) -> bool {

                // start with x
                // which byte are we in?


                let byte_offset = DISPLAY + (y * COL_SIZE_BYTE) + (x/8);
                let byte_sector = self.memory[byte_offset];
                let bit_offset = x % 8;
                let old_bit = bit_value(byte_sector, bit_offset);
                let new_value = old_bit ^ set;
                let new_byte_sector = byte_with_replaced_bit(byte_sector, bit_offset, new_value);

                self.memory[byte_offset] = new_byte_sector;

                // Need to return if any bits erased (old set to unset)
                return !old_bit & new_value;
        }

        //Dxyn
        pub fn draw(&mut self, v_x: usize, v_y:usize, n: usize) {
                // draw at coord (vx, vy) a sprite from (I) that is 8 pixels (bits) wide and N pixels high.
                // We must keep track of collisions.
                // We must wrap around outside x and y coordinates (i think)

                let mut collision_flag = false;

                // Read one byte up to n-times. This is the vertical position.
                for y_i in 0..n {
                        // read a byte of data from I location
                        let byte_read = self.memory[(self.i as usize) + y_i];

                        // Lets go bit by bit
                        for bit_i in 0..8 {
                                // get bit at position 
                                let is_set = bit_value(byte_read, bit_i);

                                // if its not set we can safely ignore xor-ing it
                                // if it is set we must write the bit to memory

                                if is_set {
                                        // Need to adjust actual x/y position if outside bounds
                                        let adj_x = (v_x + bit_i) % ROWS;
                                        let adj_y = (v_y + y_i) % COLS;

                                        // if any pixels are erased (1^1) = 0. we must set flag
                                        // TODO: Need to figure out where to write the bit to
                                        let collision = self.screen_bit_write(adj_x, adj_y, is_set);
                                        collision_flag |= collision;
                                }
                        }
                }

                self.v[0xF] = match collision_flag {
                        true => 1,
                        false => 0,
                };
                self.should_draw = true;
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
                let x = (b0 & 0x0F) as usize;
                let y = (b1 >> 4) as usize;
                let n = b1 & 0x0F;
                // println!("{} {:01x} {:01x} {:01x} ", opcode, arg0, arg1, arg2);
                if opcode == 0 { 
                        if arg0 == 0 && arg1 == 0xE {
                                if arg2 == 0 { self.clear_screen(); }
                                else if arg2 == 0xE { self.ret(); }
                                else { println!("SYS???"); }
                        } 
                        else { println!("SYS0???"); }
                }
                else if opcode == 1 { self.jump(arg3(b0, b1)); }
                else if opcode == 2 { self.call(arg3(b0, b1)) }
                else if opcode == 3 { self.se_byte(x, b1)}
                else if opcode == 4 { self.sne(x, b1) }
                else if opcode == 5 { self.se_reg(x, y) }
                else if opcode == 6 { self.load(x, b1) }
                else if opcode == 7 { self.add(x, b1) }
                else if opcode == 8 { 
                        if arg2 == 0 { self.load_reg(x, y)}
                        else if arg2 == 1 { self.or(x, y) }
                        else if arg2 == 2 { self.and(x, y) }
                        else if arg2 == 3 { self.xor(x, y) }
                        else if arg2 == 4 { self.add_with_carry(x, y) }
                        else if arg2 == 5 { self.sub_with_borrow(x, y) }
                        else if arg2 == 6 { self.shr(x, y) }
                        else if arg2 == 7 { self.subn(x, y) }
                        else if arg2 == 0xE { self.shl(x) }
                        else { println!("MATH???") }
                }
                else if opcode == 9 { self.sne_v(x, y) }
                else if opcode == 0xA { self.load_i(arg3(b0, b1)) }
                else if opcode == 0xB { self.jump_to_v0(n as u16) }
                else if opcode == 0xC { self.rand(x, b1) }
                else if opcode == 0xD { self.draw(x, y, n as usize) }
                else if opcode == 0xE { 
                        if b1 == 0x9E { self.skip_if_key_pressed(x) }
                        else if b1 == 0xA1 { self.skip_if_key_not_pressed(x) }
                }
                else if opcode == 0xF { 
                        if b1 == 0x07 { self.load_delay_timer(x) }
                        else if b1 == 0x0A { self.load_key_press(x) }
                        else if b1 == 0x15 { self.set_delay_timer(x) }
                        else if b1 == 0x18 { self.set_sound_timer(x) }
                        else if b1 == 0x1E { self.add_i(x) }
                        else if b1 == 0x29 { self.load_font(x) }
                        else if b1 == 0x33 { self.load_bcd(x) }
                        else if b1 == 0x55 { self.store_registers() }
                        else if b1 == 0x65 { self.recall_registers() }
                        else { println!("??")}
                }
                else { println!("???") }
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

fn bit_value(byte: u8, bit_index_ltr: usize) -> bool {
        // Shift it over up to 7 positions and mask the right most bit
        return byte >> (7 - bit_index_ltr) & 0x01 == 1;
}

fn byte_with_replaced_bit(byte: u8, bit_offset_ltr: usize, set: bool) -> u8 {
        let offset_x = 7 - bit_offset_ltr;
        match set {
                true => byte | (1 << offset_x),
                false => byte & (!(1 << offset_x)),
        }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_value_test() {
        let five: u8 = 5;
        assert_eq!(false, bit_value(five, 0));
        assert_eq!(false, bit_value(five, 1));
        assert_eq!(false, bit_value(five, 2));
        assert_eq!(false, bit_value(five, 3));
        assert_eq!(false, bit_value(five, 4));
        assert_eq!(true , bit_value(five, 5));
        assert_eq!(false, bit_value(five, 6));
        assert_eq!(true , bit_value(five, 7));
    }

    #[test]
    fn byte_with_replaced_bit_set_test() {
        let five: u8 = 5;
        let seven: u8 = byte_with_replaced_bit(five, 6, true);
        assert_eq!(7, seven);
    }

    #[test]
    fn byte_with_replaced_bit_unset_test() {
        let seven: u8 = 7;
        let five: u8 = byte_with_replaced_bit(seven, 6, false);
        assert_eq!(5, five);
    }
}
