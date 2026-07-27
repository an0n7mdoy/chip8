struct Hardware {
    memory:    [u8; 4096],
    registers: [u8; 16],
    index:      u16,
    pc:         u16,
    stack:     [u16; 16],
    stack_p:    usize,
    d_timer:    u8,
    s_timer:    u8,
    display:  [[bool; 32]; 64],
    keypad:    [bool; 16],
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
            memory:    [0; 4096],
            registers: [0; 16],
            index:      0,
            pc:         0x200,
            stack:     [0; 16],
            stack_p:    0,
            d_timer:    0,
            s_timer:    0,
            display:  [[false; 32]; 64],
            keypad:    [false; 16],
        };
        machine.memory[0x050..0x0A0].copy_from_slice(&FONT_SET);
        machine
    }

    fn load(&mut self, rom: &[u8]) {
        let end: usize = 0x200 + rom.len();
        self.memory[0x200..end].copy_from_slice(rom);
    }

    fn fetch(&mut self) -> u16 {
        let i1  = self.memory[self.pc as usize] as u16;
        let i2  = self.memory[(self.pc + 1) as usize] as u16;
        let res = (i1 << 8) | i2;
        self.pc += 2;
        res
    }

    fn execute(&mut self, opcode: u16) {
        let d   =  (opcode & 0xF000) >> 12;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n    =  (opcode & 0x000F)       as u8;
        let nn   =  (opcode & 0x00FF)       as u8;
        let nnn =   opcode & 0x0FFF;

        match (d, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => { self.display = [[false; 32]; 64]; }
            (0x1, _,   _,   _  ) => { self.pc = nnn; }
            (0x6, _,   _,   _  ) => { /* 6XNN: set VX = nn   */ }
            (0x7, _,   _,   _  ) => { /* 7XNN: Add NN to VX  */ }
            (0x8, _,   _,   0x0) => { /* 8XY0: set VX to value VY */ }
            (0x8, _,   _,   0x1) => { /* 8XY1: set VX to bitwise VX or VY */ }
            (0x8, _,   _,   0x2) => { /* 8XY2: set VX to bitwise VX and VY */ }
            (0x8, _,   _,   0x3) => { /* 8XY3: set VX to bitwise VX xor VY */ }
            (0x8, _,   _,   0x4) => { /* 8XY4: VX = VX + VY */ }
            (0x8, _,   _,   0x5) => { /* 8XY5: VX = VX - VY */ }
            (0x8, _,   _,   0x7) => { /* 8XY7: VX = VY - VX */ }
            // for shifts, add option to set VX to value of VY before shift
            (0x8, _,   _,   0x6) => { /* 8XY6: Shift VX one bit right */ }
            (0x8, _,   _,   0xE) => { /* 8XYE: Shift VX one bit left */ }
            (0xA, _,   _,   _  ) => { /* ANNN: Set index register I to value NNN */ }
            (0xB, _,   _,   _  ) => { /* BNNN: Set pc to (nnn + V0) || BXNN: Set pc to (nnn + VX) */ }
            (0xC, _,   _,   _  ) => { /* CXNN: Gen rand num R, bitwise R and NN, put result into VX*/ }
            (0xD, _,   _,   _  ) => { /* DXYN: Display */ }

            _ => { /* unknown opcode */ }
        }  
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_loads() {
        let machine = Hardware::new();
        assert_eq!(machine.memory.len(),     4096  );
        assert_eq!(machine.registers.len(),  16    );
        assert_eq!(machine.stack.len(),      16    );
        assert_eq!(machine.display.len(),    64    );
        assert_eq!(machine.display[0].len(), 32    );
        assert_eq!(machine.keypad.len(),     16    );
        assert_eq!(machine.index,            0     );
        assert_eq!(machine.stack_p,          0     );
        assert_eq!(machine.d_timer,          0     );
        assert_eq!(machine.s_timer,          0     );
        assert_eq!(machine.pc,               0x200 );
    }

    #[test]
    fn font_loads() {
        let machine = Hardware::new();
        assert_eq!(machine.memory[0x050..0x0A0], FONT_SET);
    }

    #[test]
    fn memory_loads() {
        let mut machine = Hardware::new();
        let that = [0x12u8, 0x34u8, 0xABu8];
        machine.load(&that);
        assert_eq!(machine.memory[0x200..0x203], that);
    }

    #[test]
    fn fetches() {
        let mut machine = Hardware::new();
        let that = [0x12u8, 0x34u8];
        machine.load(&that);
        assert_eq!(machine.fetch(), 0x1234);
        assert_eq!(machine.pc, 0x202);
    }

    #[test]
    fn executes() {
        let mut machine = Hardware::new();
        let that = [0x00u8, 0xE0u8, 0x12u8, 0x34u8];
        machine.load(&that);

        let i1 = machine.fetch();
        machine.display = [[true; 32]; 64];
        machine.execute(i1); 
        assert_eq!(machine.display, [[false; 32]; 64]);

        let i2 = machine.fetch();
        machine.execute(i2); 
        assert_eq!(machine.pc, 0x234);
    }
}
