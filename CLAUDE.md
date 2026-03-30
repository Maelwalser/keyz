# CLAUDE.md — Rhythm Keys

## Project overview

Rhythm Keys is a 3D terminal rhythm game written in Rust. Notes fly towards the player along a perspective-projected track and must be hit with the correct key when they reach the strike zone. Think Guitar Hero / Beat Saber, but in the terminal with piano-style key bindings.

## Tech stack

- **Language:** Rust (edition 2021, MSRV 1.80)
- **TUI framework:** ratatui 0.29 with crossterm 0.28 backend
- **Rendering:** Direct buffer manipulation via `ratatui::buffer::Buffer` — no widgets, all custom draw calls
- **Build:** `cargo build --release` (LTO enabled in release profile)

## Architecture

```
src/
├── main.rs        # Entry point, terminal setup/teardown, 60fps game loop
├── game.rs        # Game state machine, hit detection, scoring, note/song types
├── renderer.rs    # 3D perspective rendering engine, all screen drawing
└── songs.rs       # Built-in song chart definitions
```

### Module responsibilities

**main.rs** — Owns the terminal lifecycle (raw mode, alternate screen, cursor hide). Runs the game loop: poll input → update state → draw frame → sleep. Delegates all logic to `Game` and all drawing to `renderer::render()`. The loop targets 60fps via `Duration`-based sleep.

**game.rs** — The `Game` struct holds all mutable game state. It implements a phase-based state machine:

- `Title` → `SongSelect` → `Playing` (with countdown) → `Results` → back to `SongSelect`

Key types: `Game`, `GamePhase`, `Note`, `Song`, `HitGrade`. All lane configuration lives here as constants (`NUM_LANES`, `LANE_KEYS`, `LANE_COLORS`, timing windows). Hit detection uses closest-note-in-window matching with graded timing windows (Perfect ±45ms, Great ±90ms, Good ±140ms, Miss ±200ms).

**renderer.rs** — Stateless rendering: each function takes `&mut Buffer`, `Rect`, and `&Game`, draws directly into the buffer. The 3D perspective system is the core of the project (see below). Each game phase has its own render function (`render_title`, `render_song_select`, `render_gameplay`, `render_results`). All text drawing goes through the `draw_str()` helper.

**songs.rs** — Each song is a function returning a `Song` struct. Charts are `Vec<(f64, usize)>` — beat number (fractional) and lane index. Currently has 3 built-in songs at increasing difficulty.

### 3D perspective system (renderer.rs)

The track uses vanishing-point perspective with two core functions:

```
z_to_screen_y(z, vanish_y, hit_y) = vanish_y + (hit_y - vanish_y) * z²
screen_y_to_z(y, vanish_y, hit_y) = sqrt((y - vanish_y) / (hit_y - vanish_y))
```

- `z = 0.0` is the vanishing point (top of screen, far away)
- `z = 1.0` is the hit zone (bottom of track, closest to player)
- The z² mapping creates foreshortening: far notes are compressed, near notes are spread out
- Track width scales linearly with z: `half_width = max_half_width * z`
- Lane dividers converge at the vanishing point
- Notes use depth-based brightness (dim when far, bright when near, glow near hit zone)
- Bevel shading: `█` for note centers, `▓` for edges
- Scrolling horizontal grid lines provide depth/motion cues

### Note timing model

Notes have a `hit_time` (absolute seconds). Their z-position at any game time is:

```
z = 1.0 - (hit_time - game_time) / APPROACH_TIME
```

`APPROACH_TIME` (2.5s) is how long a note is visible before reaching the hit zone. The z-to-screen mapping makes notes appear to accelerate as they approach.

### Song chart format

```rust
Song {
    name: String,
    artist: String,
    bpm: f64,
    chart: Vec<(f64, usize)>,  // (beat_number, lane_index)
    total_beats: f64,
}
```

Beat numbers are fractional (e.g., 4.5 = the "and" of beat 4). Lane indices are 0–4 mapping to keys A, S, D, J, K. Chords are represented as multiple entries at the same beat with different lanes.

## Key constants (game.rs)

| Constant | Value | Purpose |
|---|---|---|
| `NUM_LANES` | 5 | A S D J K |
| `APPROACH_TIME` | 2.5s | Note visibility duration |
| `WINDOW_PERFECT` | ±0.045s | Perfect hit window |
| `WINDOW_GREAT` | ±0.090s | Great hit window |
| `WINDOW_GOOD` | ±0.140s | Good hit window |
| `WINDOW_MISS` | ±0.200s | Max hit window / miss cutoff |

## Development

```bash
cargo run              # Debug build (may stutter)
cargo run --release    # Smooth 60fps, use for playtesting
```

The game requires a real TTY — it will error gracefully without one. Minimum terminal size is 40×15; bigger terminals produce a more dramatic 3D effect. True color (24-bit RGB) support is assumed throughout — all colors use `Color::Rgb(r, g, b)`.

## Common modifications

**Adding a new song:** Create a function in `songs.rs` that returns a `Song`, add it to the `builtin_songs()` vec. Chart notes as `(beat_number, lane_index)` pairs.

**Adding/changing lanes:** Update `NUM_LANES`, `LANE_KEYS`, `LANE_LABELS`, and `LANE_COLORS` arrays in `game.rs`. The renderer and song system derive lane count from `NUM_LANES` automatically.

**Tuning difficulty:** Adjust `APPROACH_TIME` (lower = harder, less reaction time) and the `WINDOW_*` constants in `game.rs`. The scoring weights in `HitGrade::points()` and `Game::accuracy()` may also need rebalancing.

**Visual tweaks:** All rendering constants are local to `render_gameplay()` in `renderer.rs`: `vanish_y`, `hit_y`, `max_half_width`, note `note_z_thickness` (0.035), grid line count (12), flash durations, brightness curves. The depth grid scroll speed is `game_time * 0.4`.

**Adding a song file loader:** The `Song` struct is ready for deserialization — add `serde` derives and a JSON/TOML loader in `songs.rs`. The `rand` dependency is currently unused (available for future procedural generation or shuffle features).

## Style conventions

- Section headers use `// ── Title ───...` box-drawing separators
- All rendering is done via direct `Buffer::get_mut()` calls, not ratatui widgets
- Colors are always `Color::Rgb()` — no named colors, no indexed colors
- Game state flows through `&Game` (immutable borrows for rendering)
- Input handling and state mutation are strictly separated from rendering
- `f64` for all timing and coordinate math; `u16` only at the final screen-coordinate stage
