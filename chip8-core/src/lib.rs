use rand::RngExt;
use bit_iter::*;
struct Hardware {
    memory:    [u8; 4096],
    registers: [u8; 16],
    index:      u16,
    pc:         u16,
    stack:      Vec<u16>,
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
            stack:      Vec::new(),
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
        let i1:  u16  = self.memory[self.pc as usize] as u16;
        let i2:  u16  = self.memory[(self.pc + 1) as usize] as u16;
        let res: u16 = (i1 << 8) | i2;
        self.pc += 2;
        res
    }

    fn execute(&mut self, opcode: u16) {
        let d:   u16   =  (opcode & 0xF000) >> 12;
        let x:   usize = ((opcode & 0x0F00) >> 8) as usize;
        let y:   usize = ((opcode & 0x00F0) >> 4) as usize;
        let n:   u8    =  (opcode & 0x000F)       as u8;
        let nn:  u8    =  (opcode & 0x00FF)       as u8;
        let nnn: u16   =   opcode & 0x0FFF;

        match (d, x, y, n) {
            (0x0, 0x0, 0xE, 0x0) => { // 00E0: Clear screen
                self.display = [[false; 32]; 64]; 
            } 
            (0x1, _,   _,   _  ) => { // 1NNN: Set PC to NNN
                self.pc = nnn; 
            } 
            (0x2, _,   _,   _  ) => { // 2NNN: Call subroutine at NNN
                self.stack.push(self.pc);
                self.pc = nnn; 
            } 
            (0x0, 0x0, 0xE, 0xE) => { // 00EE: Return to pushed address
                match self.stack.pop() {
                    Some(address) => { self.pc = address },
                    None => {}
                }
            } 
            (0x3, _,   _,   _  ) => { // 3XNN: Skip one instruction if VX == NN
                if self.registers[x] == nn {self.pc += 2}
            } 
            (0x4, _,   _,   _  ) => { // 4XNN: Skip one instruction if VX != NN
                if self.registers[x] != nn {self.pc += 2}
            } 
            (0x5, _,   _,   0x0) => { // 5XY0: Skip one instruction if VX == VY
                if self.registers[x] == self.registers[y] {self.pc += 2}
            } 
            (0x9, _,   _,   0x0) => { // 9XY0: Skip one instruction if VX != VY
                if self.registers[x] != self.registers[y] {self.pc += 2}
            } 
            (0x6, _,   _,   _  ) => { //6XNN: Set VX to NN
                self.registers[x] = nn; 
            } 
            (0x7, _,   _,   _  ) => { //7XNN: Add NN to VX
                self.registers[x] = self.registers[x].wrapping_add(nn); 
            } 
            (0x8, _,   _,   0x0) => { // 8XY0: set VX to value VY
                self.registers[x] = self.registers[y];
            } 
            (0x8, _,   _,   0x1) => { // 8XY1: set VX to bitwise VX or VY
                self.registers[x] = self.registers[x] | self.registers[y];
            } 
            (0x8, _,   _,   0x2) => { // 8XY2: set VX to bitwise VX and VY
                self.registers[x] = self.registers[x] & self.registers[y];
            } 
            (0x8, _,   _,   0x3) => { // 8XY3: set VX to bitwise VX xor VY
                self.registers[x] = self.registers[x] ^ self.registers[y];
            } 
            (0x8, _,   _,   0x4) => { // 8XY4: VX = VX + VY
                let (result, overflow) = self.registers[x].overflowing_add(self.registers[y]);
                self.registers[x] = result;
                self.registers[0xF] = overflow as u8;
            } 
            (0x8, _,   _,   0x5) => { // 8XY5: VX = VX - VY
                let (result, overflow) = self.registers[x].overflowing_sub(self.registers[y]);
                self.registers[x] = result;
                self.registers[0xF] = !overflow as u8;
            } 
            (0x8, _,   _,   0x7) => { // 8XY7: VX = VY - VX
                let (result, overflow) = self.registers[y].overflowing_sub(self.registers[x]);
                self.registers[x] = result;
                self.registers[0xF] = !overflow as u8;
            } 
            // for shifts, add option to set VX to value of VY before shift
            (0x8, _,   _,   0x6) => { // 8XY6: Shift VX one bit right
                //self.registers[x] = self.registers[y];
                let shift_out = self.registers[x] & 1;
                self.registers[x] = self.registers[x] >> 1;
                self.registers[0xF] = shift_out;
            } 
            (0x8, _,   _,   0xE) => { // 8XYE: Shift VX one bit left
                //self.registers[x] = self.registers[y];
                let shift_out = (self.registers[x] >> 7) & 1;
                self.registers[x] = self.registers[x] << 1;
                self.registers[0xF] = shift_out;
            } 
            (0xA, _,   _,   _  ) => { // ANNN: Set index register I to value NNN
                self.index = nnn;
            } 
            (0xB, _,   _,   _  ) => { // BNNN: Set pc to (nnn + V0) | BXNN: Set pc to (nnn + VX)
                // self.pc = nnn + (self.registers[x] as u16);
                self.pc = nnn + (self.registers[0] as u16);
            } 
            (0xC, _,   _,   _  ) => { // CXNN: Gen rand num R, bitwise R and NN, put result into VX
                let mut rng = rand::rng();
                let random = rng.random::<u8>();
                self.registers[x] = random & nn;
            } 
            (0xD, _,   _,   _  ) => { // DXYN: Display
                let vx = self.registers[x] % 64;  // starting X coordinate
                let vy = self.registers[y] % 32;  // starting Y coordinate
                self.registers[0xF] = 0;

                for i in 0..n {
                    let cy = (vy + i) as usize;
                    if cy > 31 { break; }
                    let ca = self.index + i as u16;
                    let row = self.memory[ca as usize];

                    for j in BitIter::from(row).rev() {
                        let cx = vx as usize + (7-j);
                        if cx > 63 { break; }
                        if self.display[cx][cy] {
                            self.display[cx][cy] = false;
                            self.registers[0xF] = 1;
                        } else {
                            self.display[cx][cy] = true;
                        }
                    }
                }
            } 

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
        assert_eq!(machine.stack.len(),      0     );
        assert_eq!(machine.display.len(),    64    );
        assert_eq!(machine.display[0].len(), 32    );
        assert_eq!(machine.keypad.len(),     16    );
        assert_eq!(machine.index,            0     );
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

    // 00E0: clear the screen.
    #[test]
    fn clears_screen() {
        let mut machine = Hardware::new();
        machine.load(&[0x00, 0xE0]);
        machine.display = [[true; 32]; 64]; // dirty the screen first
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.display, [[false; 32]; 64]);
    }

    // 1NNN: jump to NNN.
    #[test]
    fn jumps() {
        let mut machine = Hardware::new();
        machine.load(&[0x12, 0x34]); // 0x1234: jump to 0x234
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x234);
    }

    // 2NNN: call a subroutine at NNN (jump + push the return address).
    #[test]
    fn calls_subroutine() {
        let mut machine = Hardware::new();
        machine.load(&[0x24, 0x00]); // 0x2400: call 0x400
        let op = machine.fetch();    // reads 0x2400, pc: 0x200 -> 0x202
        machine.execute(op);
        assert_eq!(machine.pc, 0x400);          // jumped into the subroutine
        assert_eq!(machine.stack, vec![0x202]); // pushed the advanced pc as return address
    }

    // 00EE: return from a subroutine (pop the return address back into pc).
    #[test]
    fn returns_from_subroutine() {
        let mut machine = Hardware::new();
        machine.load(&[0x24, 0x00]); // 0x2400 at 0x200: call 0x400
        machine.memory[0x400] = 0x00; // 0x00EE at 0x400: return
        machine.memory[0x401] = 0xEE;

        let call = machine.fetch();
        machine.execute(call);       // pc = 0x400, stack = [0x202]

        let ret = machine.fetch();   // reads 0x00EE, pc: 0x400 -> 0x402
        machine.execute(ret);
        assert_eq!(machine.pc, 0x202);     // returned to right after the call
        assert!(machine.stack.is_empty()); // stack popped back to empty
    }

    // 3XNN: skip the next instruction if VX == NN.
    #[test]
    fn skips_if_vx_eq_nn() {
        // condition true -> skip (pc advances an extra 2)
        let mut machine = Hardware::new();
        machine.load(&[0x3A, 0x42]); // 0x3A42: skip if V[A] == 0x42
        machine.registers[0xA] = 0x42;
        let op = machine.fetch(); // pc: 0x200 -> 0x202
        machine.execute(op);
        assert_eq!(machine.pc, 0x204); // skipped

        // condition false -> no skip
        let mut machine = Hardware::new();
        machine.load(&[0x3A, 0x42]);
        machine.registers[0xA] = 0x00;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x202); // not skipped
    }

    // 4XNN: skip the next instruction if VX != NN.
    #[test]
    fn skips_if_vx_ne_nn() {
        // condition true (values differ) -> skip
        let mut machine = Hardware::new();
        machine.load(&[0x4A, 0x42]); // 0x4A42: skip if V[A] != 0x42
        machine.registers[0xA] = 0x00;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x204); // skipped

        // condition false (values equal) -> no skip
        let mut machine = Hardware::new();
        machine.load(&[0x4A, 0x42]);
        machine.registers[0xA] = 0x42;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x202); // not skipped
    }

    // 5XY0: skip the next instruction if VX == VY.
    #[test]
    fn skips_if_vx_eq_vy() {
        // equal -> skip
        let mut machine = Hardware::new();
        machine.load(&[0x5A, 0xB0]); // 0x5AB0: skip if V[A] == V[B]
        machine.registers[0xA] = 0x11;
        machine.registers[0xB] = 0x11;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x204); // skipped

        // not equal -> no skip
        let mut machine = Hardware::new();
        machine.load(&[0x5A, 0xB0]);
        machine.registers[0xA] = 0x11;
        machine.registers[0xB] = 0x22;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x202); // not skipped
    }

    // 9XY0: skip the next instruction if VX != VY.
    #[test]
    fn skips_if_vx_ne_vy() {
        // not equal -> skip
        let mut machine = Hardware::new();
        machine.load(&[0x9A, 0xB0]); // 0x9AB0: skip if V[A] != V[B]
        machine.registers[0xA] = 0x11;
        machine.registers[0xB] = 0x22;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x204); // skipped

        // equal -> no skip
        let mut machine = Hardware::new();
        machine.load(&[0x9A, 0xB0]);
        machine.registers[0xA] = 0x11;
        machine.registers[0xB] = 0x11;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x202); // not skipped
    }

    // 6XNN: set VX to NN.
    #[test]
    fn sets_vx_to_nn() {
        let mut machine = Hardware::new();
        machine.load(&[0x6A, 0x42]); // 0x6A42: V[A] = 0x42
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0x42);
    }

    // 7XNN: add NN to VX, wrapping, without touching VF.
    #[test]
    fn adds_nn_to_vx() {
        // plain add
        let mut machine = Hardware::new();
        machine.load(&[0x7A, 0x05]); // 0x7A05: V[A] += 5
        machine.registers[0xA] = 10;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 15);

        // wraps past 255 and leaves VF untouched
        let mut machine = Hardware::new();
        machine.load(&[0x7A, 0x02]); // 0x7A02: V[A] += 2
        machine.registers[0xA] = 0xFF;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0x01); // 255 + 2 wraps to 1
        assert_eq!(machine.registers[0xF], 0); // 7XNN does not set the carry flag
    }

    // 8XY0: set VX to the value of VY.
    #[test]
    fn sets_vx_to_vy() {
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB0]); // 0x8AB0: V[A] = V[B]
        machine.registers[0xB] = 0x33;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0x33);
    }

    // 8XY1: VX = VX | VY.
    #[test]
    fn ors_vx_vy() {
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB1]); // 0x8AB1: V[A] |= V[B]
        machine.registers[0xA] = 0b1100;
        machine.registers[0xB] = 0b1010;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b1110);
    }

    // 8XY2: VX = VX & VY.
    #[test]
    fn ands_vx_vy() {
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB2]); // 0x8AB2: V[A] &= V[B]
        machine.registers[0xA] = 0b1100;
        machine.registers[0xB] = 0b1010;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b1000);
    }

    // 8XY3: VX = VX ^ VY.
    #[test]
    fn xors_vx_vy() {
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB3]); // 0x8AB3: V[A] ^= V[B]
        machine.registers[0xA] = 0b1100;
        machine.registers[0xB] = 0b1010;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b0110);
    }

    // 8XY4: VX = VX + VY, with VF = carry.
    #[test]
    fn adds_vx_vy_with_carry() {
        // no overflow -> VF = 0
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB4]); // 0x8AB4: V[A] += V[B]
        machine.registers[0xA] = 10;
        machine.registers[0xB] = 20;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 30);
        assert_eq!(machine.registers[0xF], 0);

        // overflow -> VF = 1, result wraps
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB4]);
        machine.registers[0xA] = 200;
        machine.registers[0xB] = 100;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 44); // 300 wraps to 44
        assert_eq!(machine.registers[0xF], 1);
    }

    // 8XY5: VX = VX - VY, with VF = 1 when there is NO borrow.
    #[test]
    fn subs_vy_from_vx_with_borrow() {
        // no borrow (VX >= VY) -> VF = 1
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB5]); // 0x8AB5: V[A] -= V[B]
        machine.registers[0xA] = 50;
        machine.registers[0xB] = 20;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 30);
        assert_eq!(machine.registers[0xF], 1);

        // borrow (VX < VY) -> VF = 0, result wraps
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB5]);
        machine.registers[0xA] = 20;
        machine.registers[0xB] = 50;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 226); // 20 - 50 wraps to 226
        assert_eq!(machine.registers[0xF], 0);
    }

    // 8XY7: VX = VY - VX, with VF = 1 when there is NO borrow.
    #[test]
    fn subs_vx_from_vy_with_borrow() {
        // no borrow (VY >= VX) -> VF = 1
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB7]); // 0x8AB7: V[A] = V[B] - V[A]
        machine.registers[0xA] = 20;
        machine.registers[0xB] = 50;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 30);
        assert_eq!(machine.registers[0xF], 1);

        // borrow (VY < VX) -> VF = 0, result wraps
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB7]);
        machine.registers[0xA] = 50;
        machine.registers[0xB] = 20;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 226); // 20 - 50 wraps to 226
        assert_eq!(machine.registers[0xF], 0);
    }

    // 8XY6: shift VX right by 1; VF = the bit shifted out (old bit 0).
    #[test]
    fn shifts_vx_right() {
        // low bit set -> VF = 1
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB6]); // 0x8AB6: V[A] >>= 1
        machine.registers[0xA] = 0b0000_1011; // 0x0B
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b0000_0101); // 0x0B >> 1 = 0x05
        assert_eq!(machine.registers[0xF], 1); // old bit 0 was 1

        // low bit clear -> VF = 0
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xB6]);
        machine.registers[0xA] = 0b0000_1010; // 0x0A
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b0000_0101); // 0x0A >> 1 = 0x05
        assert_eq!(machine.registers[0xF], 0); // old bit 0 was 0
    }

    // 8XYE: shift VX left by 1; VF = the bit shifted out (old bit 7).
    #[test]
    fn shifts_vx_left() {
        // high bit set -> VF = 1, top bit is lost
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xBE]); // 0x8ABE: V[A] <<= 1
        machine.registers[0xA] = 0b1000_0011; // 0x83
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b0000_0110); // 0x83 << 1 wraps to 0x06
        assert_eq!(machine.registers[0xF], 1); // old bit 7 was 1

        // high bit clear -> VF = 0
        let mut machine = Hardware::new();
        machine.load(&[0x8A, 0xBE]);
        machine.registers[0xA] = 0b0000_0011; // 0x03
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0b0000_0110); // 0x03 << 1 = 0x06
        assert_eq!(machine.registers[0xF], 0); // old bit 7 was 0
    }

    // ANNN: Set index register I to value NNN
    #[test]
    fn sets_index_to_nnn() {
        let mut machine = Hardware::new();
        machine.load(&[0xA1, 0x23]);
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.index, 0x123);
    }

    // BNNN: jump to NNN + V0.
    #[test]
    fn jumps_with_offset() {
        let mut machine = Hardware::new();
        machine.load(&[0xB3, 0x00]); // 0xB300: jump to 0x300 + V0
        machine.registers[0] = 0x12;
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.pc, 0x312); // 0x300 + 0x12
    }

    // CXNN: VX = random_byte & NN.
    // Property 1: the result never has a bit set outside the mask NN.
    #[test]
    fn random_stays_within_mask() {
        let nn = 0x0F;
        for _ in 0..1000 {
            let mut machine = Hardware::new();
            machine.load(&[0xCA, nn]); // 0xCA0F: V[A] = random & 0x0F
            let op = machine.fetch();
            machine.execute(op);
            // no bit may be set outside the mask
            assert_eq!(machine.registers[0xA] & !nn, 0);
        }

        // Edge case: a zero mask must always produce 0.
        let mut machine = Hardware::new();
        machine.load(&[0xCA, 0x00]); // 0xCA00: V[A] = random & 0x00
        let op = machine.fetch();
        machine.execute(op);
        assert_eq!(machine.registers[0xA], 0);
    }

    // Property 2: with a full mask, the output actually varies
    // (catches a stuck generator that always returns the same value).
    #[test]
    fn random_actually_varies() {
        use std::collections::HashSet;

        let mut seen = HashSet::new();
        for _ in 0..1000 {
            let mut machine = Hardware::new();
            machine.load(&[0xCA, 0xFF]); // 0xCAFF: V[A] = random & 0xFF
            let op = machine.fetch();
            machine.execute(op);
            seen.insert(machine.registers[0xA]);
        }
        // 1000 draws should yield more than one distinct value.
        assert!(seen.len() > 1);
    }

    // DXYN: draw an N-row sprite from memory[I] at (VX, VY), XOR-ing pixels.
    // Uses a 3-row diagonal so a wrong bit-order (mirror) or wrong row address
    // would land pixels in the wrong place.
    #[test]
    fn draws_sprite() {
        let mut machine = Hardware::new();
        // sprite: one pixel per row, stepping right each row (a diagonal)
        machine.memory[0x300] = 0x80; // 0b1000_0000 -> offset 0
        machine.memory[0x301] = 0x40; // 0b0100_0000 -> offset 1
        machine.memory[0x302] = 0x20; // 0b0010_0000 -> offset 2
        machine.index = 0x300;
        machine.registers[0] = 0; // VX -> start x = 0
        machine.registers[1] = 0; // VY -> start y = 0

        machine.execute(0xD013); // draw 3 rows at (V0, V1)

        // the diagonal is on
        assert!(machine.display[0][0]);
        assert!(machine.display[1][1]);
        assert!(machine.display[2][2]);
        // neighbours that must stay off (catches a mirrored draw)
        assert!(!machine.display[7][0]);
        assert!(!machine.display[1][0]);
        assert!(!machine.display[0][1]);
        // nothing was already on, so no collision
        assert_eq!(machine.registers[0xF], 0);
    }

    // DXYN: drawing the same sprite twice erases it (XOR) and flags a collision.
    #[test]
    fn draw_collision_sets_vf() {
        let mut machine = Hardware::new();
        machine.memory[0x300] = 0x80; // single pixel at offset 0
        machine.index = 0x300;
        machine.registers[0] = 0;
        machine.registers[1] = 0;

        machine.execute(0xD011); // first draw: pixel turns on
        assert!(machine.display[0][0]);
        assert_eq!(machine.registers[0xF], 0); // nothing collided

        machine.execute(0xD011); // second draw: same pixel flips back off
        assert!(!machine.display[0][0]); // erased
        assert_eq!(machine.registers[0xF], 1); // collision detected
    }
}
