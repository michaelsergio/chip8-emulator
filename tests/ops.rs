#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    /*
    #[test]
    fn add_op() {
        let memory: [u8; 4096] = [0; 4096];
        let mut chip8 = Chip8 {
            memory: memory,
            v: [0; 16],
            address: 0,
            timer_delay: 0,
            timer_sound: 0,
            pc: 0,
            sp: 0
        };
        chip8.load(0, 42);
        assert_eq!(chip8.v[0], 42);
    }
    */
}
