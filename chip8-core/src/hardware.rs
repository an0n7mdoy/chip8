use rand::rngs::SmallRng;
use rand::SeedableRng;

mod fetch;
mod execute;

pub struct Hardware {
    pub(crate) memory:    [u8; 4096],
    pub(crate) registers: [u8; 16],
    pub(crate) index:      u16,
    pub(crate) pc:         u16,
    pub(crate) stack:      Vec<u16>,
    pub(crate) d_timer:    u8,
    pub(crate) s_timer:    u8,
    pub(crate) display:  [[bool; 32]; 64],
    pub(crate) keypad:    [bool; 16],
    pub(crate) rng: SmallRng,
}

// Loaded into memory at 0x050; each glyph is 5 bytes, so digit N lives at 0x050 + N*5.
pub(crate) const FONT_SET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0  @ 0x050
    0x20, 0x60, 0x20, 0x20, 0x70, // 1  @ 0x055
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2  @ 0x05A
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3  @ 0x05F
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4  @ 0x064
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5  @ 0x069
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6  @ 0x06E
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7  @ 0x073
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8  @ 0x078
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9  @ 0x07D
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A  @ 0x082
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B  @ 0x087
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C  @ 0x08C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D  @ 0x091
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E  @ 0x096
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F  @ 0x09B
];

impl Default for Hardware {
    fn default() -> Self {
        Self::new()
    }
}

impl Hardware {
    pub fn new() -> Self {
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
            rng: SmallRng::seed_from_u64(0),
        };
        machine.memory[0x050..0x0A0].copy_from_slice(&FONT_SET);
        machine
    }

    pub fn new_with_seed(seed: u64) -> Self {
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
            rng: SmallRng::seed_from_u64(seed),
        };
        machine.memory[0x050..0x0A0].copy_from_slice(&FONT_SET);
        machine
    }

    pub fn load(&mut self, rom: &[u8]) {
        let end: usize = 0x200 + rom.len();
        self.memory[0x200..end].copy_from_slice(rom);
    }

    pub fn step(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    pub fn timer_decrement(&mut self) {
        self.d_timer = self.d_timer.saturating_sub(1);
        self.s_timer = self.s_timer.saturating_sub(1);
    }

    pub fn display(&self) -> &[[bool; 32]; 64] {
        &self.display
    }

    pub fn set_key(&mut self, key: u8, state: bool) {
        let k = key & 0x0F;
        self.keypad[k as usize] = state;
    }

    pub fn is_beeping(&self) -> bool {
        self.s_timer > 0
    }

    pub fn reset(&mut self, seed: u64) {
        *self = Self::new_with_seed(seed);
    }
}
