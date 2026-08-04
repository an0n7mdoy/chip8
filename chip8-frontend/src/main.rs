use macroquad::{audio::*, prelude::*};
use chip8_core::Hardware;

const STEPS_PER_FRAME: usize = 9;

const SCALE: f32 = 10.0;

const KEYMAP: [(KeyCode, u8); 16] = [
    (KeyCode::Key1, 0x1), (KeyCode::Key2, 0x2), (KeyCode::Key3, 0x3), (KeyCode::Key4, 0xC),
    (KeyCode::Q,    0x4), (KeyCode::W,    0x5), (KeyCode::E,    0x6), (KeyCode::R,    0xD),
    (KeyCode::A,    0x7), (KeyCode::S,    0x8), (KeyCode::D,    0x9), (KeyCode::F,    0xE),
    (KeyCode::Z,    0xA), (KeyCode::X,    0x0), (KeyCode::C,    0xB), (KeyCode::V,    0xF),
];

// The switchable game library. include_bytes! paths are relative to this file
// and case-sensitive; each glyph coerces from &[u8; N] to &'static [u8].
const ROMS: &[(&str, &[u8])] = &[
    ("Logo",          include_bytes!("../../roms/games/C8pic.ch8")),
    ("15 Puzzle",     include_bytes!("../../roms/games/15puzzle.ch8")),
    ("Blitz",         include_bytes!("../../roms/games/Blitz.ch8")),
    ("Breakout",      include_bytes!("../../roms/games/Breakout.ch8")),
    ("Brix",          include_bytes!("../../roms/games/Brix.ch8")),
    ("Cave",          include_bytes!("../../roms/games/Cave.ch8")),
    ("Connect 4",     include_bytes!("../../roms/games/Connect4.ch8")),
    ("Filter",        include_bytes!("../../roms/games/Filter.ch8")),
    ("Flight Runner", include_bytes!("../../roms/games/flightrunner.ch8")),
    ("Guess",         include_bytes!("../../roms/games/Guess.ch8")),
    ("Hidden",        include_bytes!("../../roms/games/Hidden.ch8")),
    ("Invaders",      include_bytes!("../../roms/games/Invaders.ch8")),
    ("Maze",          include_bytes!("../../roms/games/Maze.ch8")),
    ("Missile",       include_bytes!("../../roms/games/Missile.ch8")),
    ("Pong",          include_bytes!("../../roms/games/Pong.ch8")),
    ("Puzzle",        include_bytes!("../../roms/games/Puzzle.ch8")),
    ("Rocket",        include_bytes!("../../roms/games/Rocket.ch8")),
    ("Squash",        include_bytes!("../../roms/games/Squash.ch8")),
    ("Syzygy",        include_bytes!("../../roms/games/Syzygy.ch8")),
    ("Tank",          include_bytes!("../../roms/games/Tank.ch8")),
    ("Tapeworm",      include_bytes!("../../roms/games/Tapeworm.ch8")),
    ("Tetris",        include_bytes!("../../roms/games/Tetris.ch8")),
];


fn boot(machine: &mut Hardware, rom: &[u8]) {
    machine.reset((macroquad::miniquad::date::now() * 1000.0) as u64);
    machine.load(rom);
}

#[macroquad::main("CHIP-8")]
async fn main() {
    let mut selected = 0usize;

    let seed = (macroquad::miniquad::date::now() * 1000.00) as u64;
    let mut machine = Hardware::new_with_seed(seed);
    machine.load(ROMS[selected].1);

    let beep = load_sound_from_bytes(include_bytes!("../../roms/beep.wav"))
        .await
        .expect("failed to load beep");
    let mut beeping = false;
    loop {

        if is_key_pressed(KeyCode::P) {
            boot(&mut machine, ROMS[selected].1);
        }

        if is_key_pressed(KeyCode::Right) {
            selected = (selected + 1) % ROMS.len();
            boot(&mut machine, ROMS[selected].1);
        }
        if is_key_pressed(KeyCode::Left) {
            selected = (selected + ROMS.len() - 1) % ROMS.len();
            boot(&mut machine, ROMS[selected].1);
        }

        if is_key_pressed(KeyCode::N) {
            selected = 0usize;
            boot(&mut machine, ROMS[selected].1);
        }

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

        if machine.is_beeping() && !beeping {
            play_sound(&beep, PlaySoundParams { looped: true, volume: 0.2 });
            beeping = true;
        } else if !machine.is_beeping() && beeping {
            stop_sound(&beep);
            beeping = false;
        }


        next_frame().await;   
    }
}
