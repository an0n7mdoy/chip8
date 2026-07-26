struct Hardware {
    memory: [u8; 4096],
    registers: [u8; 16],
    index: u16,
    pc: u16,
    stack: [u16; 16],
    stack_p: usize,
    d_timer: u8,
    s_timer: u8,
    display: [[bool; 32]; 64],
    keypad: [bool; 16],
}

impl Hardware {
    fn new() -> Self {
        Self {
            memory: [0; 4096],
            registers: [0; 16],
            index: 0,
            pc: 0x200,
            stack: [0; 16],
            stack_p: 0,
            d_timer: 0,
            s_timer: 0,
            display: [[false; 32]; 64],
            keypad: [false; 16],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let machine = Hardware::new();
        assert_eq!(machine.memory, [0; 4096]);
        assert_eq!(machine.registers, [0; 16]);
        assert_eq!(machine.stack, [0; 16]);
        assert_eq!(machine.display, [[false; 32]; 64]);
        assert_eq!(machine.keypad, [false; 16]);
        assert_eq!(machine.index, 0);
        assert_eq!(machine.stack_p, 0);
        assert_eq!(machine.d_timer, 0);
        assert_eq!(machine.s_timer, 0);
        assert_eq!(machine.pc, 0x200)
    }
}
