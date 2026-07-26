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

const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

impl Hardware {
    fn new() -> Self {
        let mut machine = Self {
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
        };
        machine.memory[0x050..0x0A0].copy_from_slice(&FONT_SET);
        machine
    }

    fn load(&mut self, rom: &[u8]) {
        let end: usize = 0x200 + rom.len();
        self.memory[0x200..end].copy_from_slice(rom);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let machine = Hardware::new();
        assert_eq!(machine.memory.len(), 4096);
        assert_eq!(machine.registers.len(), 16);
        assert_eq!(machine.stack.len(), 16);
        assert_eq!(machine.display.len(), 64);
        assert_eq!(machine.display[0].len(), 32);
        assert_eq!(machine.keypad.len(), 16);
        assert_eq!(machine.index, 0);
        assert_eq!(machine.stack_p, 0);
        assert_eq!(machine.d_timer, 0);
        assert_eq!(machine.s_timer, 0);
        assert_eq!(machine.pc, 0x200)
    }

    #[test]
    fn memory_load() {
        let mut machine = Hardware::new();
        let that = [0x12u8, 0x34u8, 0xABu8];
        machine.load(&that);
        assert_eq!(machine.memory[0x200..0x203], that);
    }
}
