use macroquad::prelude::*;
use chip8_core::Hardware;

const STEPS_PER_FRAME: usize = 9;

const SCALE: f32 = 10.0;

const KEYMAP: [(KeyCode, u8); 16] = [
    (KeyCode::Key1, 0x1), (KeyCode::Key2, 0x2), (KeyCode::Key3, 0x3), (KeyCode::Key4, 0xC),
    (KeyCode::Q,    0x4), (KeyCode::W,    0x5), (KeyCode::E,    0x6), (KeyCode::R,    0xD),
    (KeyCode::A,    0x7), (KeyCode::S,    0x8), (KeyCode::D,    0x9), (KeyCode::F,    0xE),
    (KeyCode::Z,    0xA), (KeyCode::X,    0x0), (KeyCode::C,    0xB), (KeyCode::V,    0xF),
];

#[macroquad::main("CHIP-8")]
async fn main() {
    let mut machine = Hardware::new();
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "roms/ibm-logo.ch8".to_string());
    let rom = std::fs::read(&path).expect("failed to read ROM");
    machine.load(&rom);
    loop {
        for (key, chip8) in KEYMAP {
            machine.set_key(chip8, is_key_down(key));
        }   

        for _ in 0..STEPS_PER_FRAME {
             machine.step();
        }
        machine.timer_decrement();

        clear_background(BLACK);
        let disp = machine.display();
        for (x, column) in disp.iter().enumerate() {
            for (y, &pixel) in column.iter().enumerate() {
                if pixel {
                    draw_rectangle(x as f32 * SCALE, y as f32 * SCALE, SCALE, SCALE, WHITE);
                }
            }
        }
        next_frame().await;   
    }
}
