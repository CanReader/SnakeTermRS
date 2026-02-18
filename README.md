# Snake — Terminal Edition (Rust)

> A compact, configurable, terminal-based Snake game rewritten in Rust.

A complete rewrite of [SnakeTerm](../README.md) (originally C++) using idiomatic Rust with `crossterm` for cross-platform terminal handling and `clap` for CLI parsing.

---

## Improvements over the C++ version

- **No raw pthreads** — uses `crossterm`'s event polling instead of a background input thread with semaphores
- **Alternate screen** — game runs in an alternate terminal buffer, leaving your scrollback clean
- **Visible borders** — the map is drawn with a box border for better visual clarity
- **Cross-platform** — `crossterm` works on Linux, macOS, and Windows
- **Proper CLI parsing** — `clap` with derive macros, automatic `--help` generation
- **Memory safe** — no raw pointers, no manual memory management
- **VecDeque** for the snake body instead of `vector::erase(begin())` (O(1) vs O(n) pop)

---

## Build

```bash
cargo build --release
```

The binary is at `target/release/snake-term`.

---

## Run

```bash
cargo run --release
# or
./target/release/snake-term
```

### Controls

- **Move**: `W A S D` or Arrow keys
- **Quit**: `Q`, `Esc`, or `Ctrl+C`
- **On game over**: `R` to restart, `Q` to quit

---

## Command-line options

```
Usage: snake-term [OPTIONS]

Options:
      --speed <SPEED>      Frame delay in milliseconds (default: 200)
      --body <BODY>        Snake body character (default: @)
      --head-w <HEAD_W>    Head glyph when moving west
      --head-n <HEAD_N>    Head glyph when moving north
      --head-e <HEAD_E>    Head glyph when moving east
      --head-s <HEAD_S>    Head glyph when moving south
      --head <HEAD>        Set all 4 head chars as WNES (e.g. '<^>v')
      --food <FOOD>        Food glyph (default: *)
      --seed <SEED>        RNG seed (0 = random)
      --hide-score         Hide the score display
      --auto-restart       Automatically restart on game over
      --invert-controls    Invert movement controls
      --disable-borders    Enable wrap-around
  -h, --help               Print help
```

### Examples

```bash
# Faster game with custom look
./target/release/snake-term --speed 120 --body '#' --head '<^>v' --food '$'

# Auto-restart with wrap-around
./target/release/snake-term --auto-restart --disable-borders
```
