# SnakeTermRS

A feature-rich terminal Snake game written in Rust.

![Rust](https://img.shields.io/badge/language-Rust-orange)
![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-blue)
![License](https://img.shields.io/badge/license-MIT-green)

```
  ╔═══════════════════════════════╗
  ║     SNAKE — Terminal Edition  ║
  ╚═══════════════════════════════╝
```

## Features

- Colored rendering (green snake, yellow head, red food)
- Singleplayer and local multiplayer (2 players, same keyboard)
- Start menu with high score display
- Pause / resume
- Progressive speed (gets faster as you grow)
- Bonus food (`$`) that spawns randomly for extra points
- Random obstacles / walls
- Shrinking border mode
- Death animation
- Wrap-around (borderless) mode
- Inverted controls mode
- Auto-restart mode
- Dynamic map sizing (auto-detects terminal size)
- Input buffering (queue up to 3 fast turns)
- High score persistence
- Game recording and replay
- TOML config file support
- Fully configurable glyphs, speed, and RNG seed
- Terminal bell on food eat and death

---

## Build

Requires [Rust](https://rustup.rs/) (1.70+).

```bash
cargo build --release
```

## Run

```bash
./target/release/snake-term
```

---

## Controls

| Key | Action |
|-----|--------|
| `W A S D` | Move (Player 1) |
| `Arrow keys` | Move (Player 1, or Player 2 in multiplayer) |
| `P` / `Space` | Pause / Resume |
| `Q` / `Esc` | Quit |
| `R` | Restart (on game over) |
| `M` | Back to menu (on game over) |
| `Ctrl+C` | Force quit |

---

## Command-line options

```
Usage: snake-term [OPTIONS]

Options:
      --speed <ms>               Frame delay in milliseconds [default: 200]
      --body <char>              Snake body character [default: @]
      --head-w <char>            Head glyph moving west [default: <]
      --head-n <char>            Head glyph moving north [default: ^]
      --head-e <char>            Head glyph moving east [default: >]
      --head-s <char>            Head glyph moving south [default: v]
      --head <4chars>            All 4 head chars as WNES (e.g. '<^>v')
      --food <char>              Food glyph [default: *]
      --seed <num>               RNG seed, 0 = random [default: 0]
      --hide-score               Hide the score display
      --auto-restart             Auto-restart on game over
      --invert-controls          Invert movement directions
      --disable-borders          Enable wrap-around
      --obstacles <num>          Number of random walls [default: 0]
      --multiplayer              Enable 2-player mode
      --progressive-speed        Speed increases as snake grows
      --shrinking-border         Play area shrinks over time
      --map-width <num>          Map width, 0 = auto [default: 0]
      --map-height <num>         Map height, 0 = auto [default: 0]
      --config <path>            Load settings from a TOML file
      --record <path>            Record game inputs to a file
      --replay <path>            Play back a recorded game
  -h, --help                     Print help
```

---

## Examples

```bash
# Classic game
snake-term

# Fast game with custom glyphs
snake-term --speed 100 --body '#' --head '<^>v' --food '@'

# 2 players with obstacles
snake-term --multiplayer --obstacles 10

# Challenge mode: fast, shrinking, no borders
snake-term --progressive-speed --shrinking-border --disable-borders

# Auto-restart for high score grinding
snake-term --auto-restart --speed 150

# Record a game, then replay it
snake-term --record my_game.rep --seed 42
snake-term --replay my_game.rep --seed 42

# Use a config file
snake-term --config settings.toml
```

---

## TOML config file

Instead of passing flags every time, create a `settings.toml`:

```toml
speed = 150
body = "#"
food = "@"
obstacles = 5
progressive_speed = true
disable_borders = true
map_width = 30
map_height = 20
```

Then run with:

```bash
snake-term --config settings.toml
```

CLI flags override config file values.

---

## Multiplayer

Run with `--multiplayer` for local 2-player on the same keyboard:

- **Player 1**: `W A S D`
- **Player 2**: `Arrow keys`

Players spawn on separate rows. Colliding with the other snake's body kills you.

---

## Project structure

```
src/
├── main.rs        Entry point, game loop, menus
├── config.rs      CLI parsing, TOML config, constants
├── snake.rs       Snake state, movement, collision
├── game_map.rs    Grid rendering, walls, bonus food
├── input.rs       Keyboard input handling
├── highscore.rs   High score persistence
└── replay.rs      Game recording and playback
```

---

## Tests

```bash
cargo test
```

Covers snake movement, collision, wall hits, wrap-around, food eating, direction queue, and reset logic.
