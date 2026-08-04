# CHIP-8 Emulator

[![CI](https://github.com/an0n7mdoy/chip8/actions/workflows/ci.yml/badge.svg)](https://github.com/an0n7mdoy/chip8/actions/workflows/ci.yml)  [![Live Demo](https://img.shields.io/badge/play-online-brightgreen)](https://an0n7mdoy.github.io/chip8/)  ![Code size](https://img.shields.io/github/languages/code-size/an0n7mdoy/chip8)  ![Website](https://img.shields.io/website?url=https%3A%2F%2Fan0n7mdoy.github.io%2Fchip8%2F&label=site)  ![License](https://img.shields.io/github/license/an0n7mdoy/chip8)

A CHIP-8 emulator written in Rust. It runs as a **native desktop app** and, from the exact same code, compiles to **WebAssembly** so it plays in the browser — [try it live](https://an0n7mdoy.github.io/chip8/). It ships with a built-in library of 20+ classic games you can flip through without hunting down ROM files.

## What is CHIP-8?

CHIP-8 is an interpreted virtual machine from the mid-1970s, designed to make simple games easier to write on early hobbyist computers. It's tiny — 35 instructions, 4 KB of memory, a 64×32 monochrome display, a 16-key hex keypad, and two timers — which makes it the traditional "hello world" of emulator writing. The games (Pong, Space Invaders, Tetris, Breakout, and friends) are public-domain classics that have been passed around for decades.

This project implements the full instruction set and runs those original ROMs.

## Features

- **Full CHIP-8 instruction set** — all 35 opcodes, verified by a 47-test suite.
- **Two front-ends, one core** — the same emulator core drives both the desktop binary and the WebAssembly build.
- **Built-in game library** — 20+ games bundled into the binary; switch between them at runtime, no files to manage.
- **Sound** — the CHIP-8 buzzer is emulated (a tone while the sound timer is active).
- **Accurate timers** — delay and sound timers tick at 60 Hz; the CPU runs ~540 instructions/second.
- **Responsive display** — the picture scales to any window or phone screen while keeping the 2:1 aspect ratio.
- **On-screen keypad** (web) — the hex keypad is clickable/tappable, so it's playable on a phone with no keyboard.

## Project structure

This is a Cargo workspace with two crates:

| Crate | What it is |
|-------|-----------|
| [`chip8-core`](chip8-core) | The emulator itself — a pure library that models the CHIP-8 machine (memory, registers, opcodes, timers, display buffer). It does **no** I/O: no rendering, no input reading, no file access. This is what the tests exercise. |
| [`chip8-frontend`](chip8-frontend) | The playable app, built on the [macroquad](https://macroquad.rs) game framework. It owns the window, reads the keyboard, draws the display buffer, plays sound, and feeds input into the core. |

Keeping all machine logic in an I/O-free core is what lets the *same* emulator run on the desktop and in the browser — only the thin front-end layer changes.

## Running on desktop

### Prerequisites

You need the Rust toolchain. If you don't have it, install [rustup](https://rustup.rs):

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and run

```sh
git clone https://github.com/an0n7mdoy/chip8.git
cd chip8
cargo run --release -p chip8-frontend
```

The `--release` flag is worth it — the debug build runs noticeably slower. The window opens on the built-in library's first entry, and you switch games from the keyboard (see below). There's nothing else to configure and no ROM files to download; the games are compiled into the binary.

## Controls

### Emulator controls

These drive the emulator itself (not the game):

| Key | Action |
|-----|--------|
| `←` / `→` | Previous / next game |
| `P` | Restart the current game |
| `N` | Jump back to the first entry (the logo) |

### The keypad

CHIP-8's controller is a 16-key hex keypad (`0`–`F`). It's mapped onto the left side of your keyboard in the same physical 4×4 shape:

```
 Keyboard          CHIP-8 keypad
 1  2  3  4          1  2  3  C
 Q  W  E  R          4  5  6  D
 A  S  D  F          7  8  9  E
 Z  X  C  V          A  0  B  F
```

So pressing **`Q`** sends CHIP-8 key **`4`**, **`W`** sends **`5`**, and so on.

### ⚠️ Every game has its own controls

**CHIP-8 has no standard control scheme.** Each ROM was written independently and hardcodes whatever keys its author chose, using some subset of the 16 hex keys. There is no universal "left/right/fire" — a key that steers a paddle in one game does nothing in another.

That means: **when you start a game, you often have to experiment** to find its controls. Try the keys clustered around the pad (`Q W E`, `1 2 3`) — most games use just two or three of them.

A few of the well-known ones (mapped to the keyboard keys you'd actually press):

| Game | Controls |
|------|----------|
| **Invaders** (Space Invaders) | `Q` = move left, `E` = move right, `W` = fire / start |
| **Brix**, **Breakout** | `Q` = paddle left, `E` = paddle right |
| **Pong** | `1` / `Q` = left paddle up / down, `4` / `R` = right paddle up / down |
| **Blitz** | `W` = drop bomb |
| **Tank** | `Q` `E` = turn, `W` = fire (and the number row to move) |

These come from the games' original documentation and may not be exhaustive — when in doubt, experiment. On the [web version](https://an0n7mdoy.github.io/chip8/) the on-screen keypad makes trial-and-error easy: just tap keys and watch what happens.

## Game library

The bundled games, in menu order:

`Logo` · `15 Puzzle` · `Blitz` · `Breakout` · `Brix` · `Cave` · `Connect 4` · `Filter` · `Flight Runner` · `Guess` · `Hidden` · `Invaders` · `Maze` · `Missile` · `Pong` · `Puzzle` · `Rocket` · `Squash` · `Syzygy` · `Tank` · `Tapeworm` · `Tetris`

## Playing in the browser

The web build is already deployed — just open **<https://an0n7mdoy.github.io/chip8/>**. It works on desktop and mobile; on a phone, use the on-screen keypad.

### Building the web version yourself

The front-end compiles to WebAssembly against macroquad's browser backend:

```sh
# one-time: add the WebAssembly target
rustup target add wasm32-unknown-unknown

# build (linker flags for the browser live in .cargo/config.toml)
cargo build --release --target wasm32-unknown-unknown -p chip8-frontend
```

The resulting `.wasm` is loaded by [`docs/index.html`](docs/index.html), which pulls in macroquad's JavaScript bundle and provides the styled page and on-screen keypad. The `docs/` folder is what GitHub Pages serves. To preview locally, serve that folder over HTTP (browsers won't load `.wasm` from `file://`):

```sh
cp target/wasm32-unknown-unknown/release/chip8-frontend.wasm docs/chip8-frontend.wasm
python3 -m http.server 8080 --directory docs
# then open http://localhost:8080
```

## Implementation notes

CHIP-8 has a handful of ambiguous instructions where different historical interpreters disagree ("quirks"). This emulator makes these choices:

- **`8XY6` / `8XYE` (shifts)** — shift `VX` in place (ignore `VY`).
- **`BNNN` (jump with offset)** — offset by `V0` (the original COSMAC VIP behavior).
- **`FX55` / `FX65` (register store/load)** — do **not** increment the index register `I` afterwards.
- **`FX1E` (add to I)** — sets the carry flag `VF` if `I` overflows past `0x0FFF`.
- **Sprite drawing (`DXYN`)** — the starting position wraps around the screen edges, but sprites drawn near an edge are clipped, not wrapped.

Randomness (`CXNN`) uses a small seedable PRNG so the core stays dependency-light and works on WebAssembly without needing OS entropy; the front-end seeds it from the clock at launch.

## Testing

The core has a full unit-test suite covering every opcode (both branches of each conditional, flag behavior, memory round-trips, and property tests for the random instruction):

```sh
cargo test --all
```

Continuous integration runs the tests plus Clippy (with warnings denied) on every push.

## Acknowledgements

- Built with the [macroquad](https://macroquad.rs) game framework.
- Conformance validated against [Timendus's CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite).
- The bundled games are the traditional public-domain CHIP-8 ROM collection.

## License

See [`LICENSE`](LICENSE).
