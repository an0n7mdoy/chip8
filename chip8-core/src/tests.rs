use crate::hardware::{Hardware, FONT_SET};

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
    fn reset_restores_fresh_state() {
        let mut machine = Hardware::new();

        // Dirty every field so a partial reset would be caught.
        machine.memory[0x300] = 0xAB;
        machine.registers[3]  = 0x7F;
        machine.index         = 0x0FFF;
        machine.pc            = 0x400;
        machine.stack.push(0x222);
        machine.d_timer       = 30;
        machine.s_timer       = 15;
        machine.display[10][20] = true;
        machine.keypad[5]     = true;

        machine.reset(0);

        let fresh = Hardware::new();
        assert_eq!(machine.memory,    fresh.memory   );
        assert_eq!(machine.registers, fresh.registers);
        assert_eq!(machine.index,     fresh.index    );
        assert_eq!(machine.pc,        fresh.pc       );
        assert_eq!(machine.stack,     fresh.stack    );
        assert_eq!(machine.d_timer,   fresh.d_timer  );
        assert_eq!(machine.s_timer,   fresh.s_timer  );
        assert_eq!(machine.display,   fresh.display  );
        assert_eq!(machine.keypad,    fresh.keypad   );
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

        let mut machine = Hardware::new();
        let mut seen = HashSet::new();
        for _ in 0..1000 {
            machine.execute(0xCAFF); // V[A] = random & 0xFF
            seen.insert(machine.registers[0xA]);
        }
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

    // EX9E: skip the next instruction if the key in VX is pressed.
    #[test]
    fn skips_if_key_pressed() {
        // key pressed -> skip
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x5; // VX names key 5
        machine.keypad[0x5] = true;
        let op = 0xEA9E; // EX9E with X = A
        let pc_before = machine.pc;
        machine.execute(op);
        assert_eq!(machine.pc, pc_before + 2); // skipped

        // key not pressed -> no skip
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x5;
        machine.keypad[0x5] = false;
        let pc_before = machine.pc;
        machine.execute(0xEA9E);
        assert_eq!(machine.pc, pc_before); // not skipped
    }

    // EXA1: skip the next instruction if the key in VX is NOT pressed.
    #[test]
    fn skips_if_key_not_pressed() {
        // key not pressed -> skip
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x5;
        machine.keypad[0x5] = false;
        let pc_before = machine.pc;
        machine.execute(0xEAA1); // EXA1 with X = A
        assert_eq!(machine.pc, pc_before + 2); // skipped

        // key pressed -> no skip
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x5;
        machine.keypad[0x5] = true;
        let pc_before = machine.pc;
        machine.execute(0xEAA1);
        assert_eq!(machine.pc, pc_before); // not skipped
    }

    // A VX value above 0x0F must be masked to the low nibble, not panic.
    #[test]
    fn key_index_is_masked() {
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x1F; // low nibble is 0xF
        machine.keypad[0xF] = true;
        let pc_before = machine.pc;
        machine.execute(0xEA9E); // must index key 0xF, not panic on 0x1F
        assert_eq!(machine.pc, pc_before + 2);
    }

    // FX07: read the delay timer into VX.
    #[test]
    fn reads_delay_timer() {
        let mut machine = Hardware::new();
        machine.d_timer = 0x42;
        machine.execute(0xFA07); // VA = delay timer
        assert_eq!(machine.registers[0xA], 0x42);
    }

    // FX15: set the delay timer from VX.
    #[test]
    fn sets_delay_timer() {
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x42;
        machine.execute(0xFA15); // delay timer = VA
        assert_eq!(machine.d_timer, 0x42);
    }

    // FX18: set the sound timer from VX.
    #[test]
    fn sets_sound_timer() {
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x42;
        machine.execute(0xFA18); // sound timer = VA
        assert_eq!(machine.s_timer, 0x42);
    }

    // FX1E: add VX to the index register.
    #[test]
    fn adds_vx_to_index() {
        // plain add, no overflow past 0x0FFF
        let mut machine = Hardware::new();
        machine.index = 0x200;
        machine.registers[0xA] = 0x05;
        machine.execute(0xFA1E);
        assert_eq!(machine.index, 0x205);

        // overflow past 0x0FFF raises VF
        let mut machine = Hardware::new();
        machine.index = 0x0FFF;
        machine.registers[0xA] = 0x01;
        machine.execute(0xFA1E);
        assert_eq!(machine.index, 0x1000);
        assert_eq!(machine.registers[0xF], 1);
    }

    // FX0A: with no key down, rewind PC so the instruction re-runs (blocking wait).
    #[test]
    fn waits_for_key_press() {
        let mut machine = Hardware::new();
        machine.pc = 0x300;
        machine.execute(0xFA0A); // FX0A with X = A, no keys pressed
        assert_eq!(machine.pc, 0x2FE); // rewound by 2
        assert_eq!(machine.registers[0xA], 0); // nothing captured
    }

    // FX0A: with a key down, store its index in VX and leave PC alone.
    #[test]
    fn captures_key_press() {
        let mut machine = Hardware::new();
        machine.pc = 0x300;
        machine.keypad[0x7] = true;
        machine.execute(0xFA0A);
        assert_eq!(machine.registers[0xA], 0x7); // captured key 7
        assert_eq!(machine.pc, 0x300); // did not rewind
    }

    // FX29: point I at the font sprite for the digit in VX (0x050 + digit*5).
    #[test]
    fn sets_index_to_font_char() {
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0xB; // digit B
        machine.execute(0xFA29); // FX29 with X = A
        assert_eq!(machine.index, 0x087); // matches the font table comment for B

        // high nibble is ignored: 0x1B still selects digit B
        let mut machine = Hardware::new();
        machine.registers[0xA] = 0x1B;
        machine.execute(0xFA29);
        assert_eq!(machine.index, 0x087);
    }

    // FX33: store the decimal digits of VX at memory[I], I+1, I+2.
    #[test]
    fn stores_bcd() {
        // 254 -> 2, 5, 4
        let mut machine = Hardware::new();
        machine.index = 0x300;
        machine.registers[0xA] = 254;
        machine.execute(0xFA33); // FX33 with X = A
        assert_eq!(machine.memory[0x300], 2);
        assert_eq!(machine.memory[0x301], 5);
        assert_eq!(machine.memory[0x302], 4);

        // leading zeros are still written: 7 -> 0, 0, 7
        let mut machine = Hardware::new();
        machine.index = 0x300;
        machine.registers[0xA] = 7;
        machine.execute(0xFA33);
        assert_eq!(machine.memory[0x300], 0);
        assert_eq!(machine.memory[0x301], 0);
        assert_eq!(machine.memory[0x302], 7);
    }

    // FX55: store V0..=VX into memory starting at I.
    #[test]
    fn stores_registers_to_memory() {
        let mut machine = Hardware::new();
        machine.index = 0x300;
        machine.registers[0..=5].copy_from_slice(&[10, 20, 30, 40, 50, 60]);
        machine.execute(0xF555); // FX55 with X = 5 -> store V0..=V5
        assert_eq!(machine.memory[0x300..=0x305], [10, 20, 30, 40, 50, 60]);
        assert_eq!(machine.memory[0x306], 0); // one past the end untouched
        assert_eq!(machine.index, 0x300); // non-incrementing variant leaves I alone
    }

    // FX65: load V0..=VX from memory starting at I.
    #[test]
    fn loads_registers_from_memory() {
        let mut machine = Hardware::new();
        machine.index = 0x300;
        machine.memory[0x300..=0x305].copy_from_slice(&[11, 22, 33, 44, 55, 66]);
        machine.execute(0xF565); // FX65 with X = 5 -> load V0..=V5
        assert_eq!(machine.registers[0..=5], [11, 22, 33, 44, 55, 66]);
        assert_eq!(machine.registers[6], 0); // register past X untouched
    }

    // FX55 then FX65 must round-trip all 16 registers unchanged.
    #[test]
    fn store_load_round_trips() {
        let mut machine = Hardware::new();
        machine.index = 0x300;
        let values: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        machine.registers = values;

        machine.execute(0xFF55); // store V0..=VF
        machine.registers = [0; 16]; // wipe registers
        machine.execute(0xFF65); // load them back

        assert_eq!(machine.registers, values);
    }

    // step: fetch + execute in one call — the opcode runs and PC advances.
    #[test]
    fn step_fetches_and_executes() {
        let mut machine = Hardware::new();
        machine.load(&[0x6A, 0x42]); // 0x6A42: set VA = 0x42
        machine.step();
        assert_eq!(machine.registers[0xA], 0x42); // execute ran
        assert_eq!(machine.pc, 0x202); // fetch advanced PC past the opcode
    }

    // timer_decrement: both timers count down by 1 and floor at 0.
    #[test]
    fn timers_count_down() {
        let mut machine = Hardware::new();
        machine.d_timer = 5;
        machine.s_timer = 2;
        machine.timer_decrement();
        assert_eq!(machine.d_timer, 4);
        assert_eq!(machine.s_timer, 1);
    }

    #[test]
    fn timers_do_not_underflow() {
        let mut machine = Hardware::new();
        machine.d_timer = 0;
        machine.s_timer = 0;
        machine.timer_decrement(); // must stay at 0, not wrap to 255 or panic
        assert_eq!(machine.d_timer, 0);
        assert_eq!(machine.s_timer, 0);
    }
}