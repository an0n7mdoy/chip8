use super::Hardware;
use bit_iter::BitIter;
use rand::RngExt;

impl Hardware {
    pub(crate) fn execute(&mut self, opcode: u16) {
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
                if let Some(address) = self.stack.pop() { self.pc = address }
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
                self.registers[x] |= self.registers[y];
            }
            (0x8, _,   _,   0x2) => { // 8XY2: set VX to bitwise VX and VY
                self.registers[x] &= self.registers[y];
            }
            (0x8, _,   _,   0x3) => { // 8XY3: set VX to bitwise VX xor VY
                self.registers[x] ^= self.registers[y];
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
                self.registers[x] >>= 1;
                self.registers[0xF] = shift_out;
            }
            (0x8, _,   _,   0xE) => { // 8XYE: Shift VX one bit left
                //self.registers[x] = self.registers[y];
                let shift_out = (self.registers[x] >> 7) & 1;
                self.registers[x] <<= 1;
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
                self.registers[x] = self.rng.random::<u8>() & nn;
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
            (0xE, _,   0x9, 0xE) => { // EX9E: Skip instruction if key VX pressed
                let vx = self.registers[x] & 0x0F;
                if self.keypad[vx as usize] { self.pc += 2; }
            }
            (0xE, _,   0xA, 0x1) => { // EXA1: Skip instruction if key VX not pressed
                let vx = self.registers[x] & 0x0F;
                if !self.keypad[vx as usize] { self.pc += 2; }
            }
            (0xF, _,   0x0, 0x7) => { // FX07: Set VX to current value of delay timer
                self.registers[x] = self.d_timer;
            }
            (0xF, _,   0x1, 0x5) => { // FX15: Set delay timer to value in VX
                self.d_timer = self.registers[x];
            }
            (0xF, _,   0x1, 0x8) => { // FX18: Set sound timer to value in VX
                self.s_timer = self.registers[x];
            }
            (0xF, _,   0x1, 0xE) => { // FX1E: Add VX to index register
                self.index += self.registers[x] as u16; // unguarded index
                self.registers[0xF] = (self.index > 0x0FFF) as u8;
            }
            (0xF, _,   0x0, 0xA) => { // FX0A: Get key, decrement pc othewise
                match self.keypad.iter().position(|&r| r) {
                     Some(i) => self.registers[x] = i as u8,
                     None => self.pc -= 2
                }
            }
            (0xF, _,   0x2, 0x9) => { // FX29: Set index to the address of char in VX
                self.index = 0x050 + (self.registers[x] & 0x0F) as u16 * 5;
            }
            (0xF, _,   0x3, 0x3) => { // FX33: Take number from  vx, store digit 1 at I, digit 2 at I+1, digit 3 at i+2
                let num = self.registers[x];
                let d1 = num / 100;
                let d2 = (num % 100) / 10;
                let d3 = num % 10;
                let i = self.index as usize;
                self.memory[i] = d1;
                self.memory[i+1] = d2;
                self.memory[i+2] = d3;
            }
            (0xF, _,   0x5, 0x5) => { // FX55: Store V0..VX in succ mem addrss starting with I. If VX == 0, store only V0
                for i in 0..=x {
                    self.memory[self.index as usize + i ] = self.registers[i];
                }
                //self.index += x as u16 + 1;
            }
            (0xF, _,   0x6, 0x5) => { // FX65: Store values from I..I+X+1 in V0..VX
                for i in 0..=x {
                    self.registers[i] = self.memory[self.index as usize + i ];
                }
            }
            (0x0, 0x0, 0x0, 0x0) => { // 0000: Buffer
                self.index = 0x050 + (self.registers[x] & 0x0F) as u16 * 5;
            }
            _ => { /* unknown opcode */
                //dbg!{"unknown opcode!!!", d, x, y, n};
            }
        }
    }
}
