# 🎹 Keyz

A **3D terminal rhythm game** where notes fly towards you in perspective — hit them with piano-style key presses as they reach the strike zone.

```
         ╲│╱
        ──┼──        Notes spawn far away and rush towards you
         ╱│╲         with real perspective scaling & depth shading
        / │ \
       /  │  \
      / ░▒▓█  \     ← Notes grow wider and brighter as they approach
     /  ░▒▓█   \
    /   █████    \
   ════▬▬▬▬▬▬════   ← Hit zone — press the right key!
      [A][S][D][J][K]
```

## Install

### From source (recommended)

```bash
cargo install --git https://github.com/YOUR_USERNAME/rhythm-keys
```

### Or clone & build

```bash
git clone https://github.com/YOUR_USERNAME/rhythm-keys.git
cd rhythm-keys
cargo run --release
```

> **Requires:** Rust 1.70+ — install from [rustup.rs](https://rustup.rs)

## How to Play

Notes scroll towards you along a **3D perspective track**. When a note reaches the **hit zone** at the bottom, press the matching key:

| Key | Lane | Color |
|-----|------|-------|
| `A` | 1 (left) | Pink |
| `S` | 2 | Orange |
| `D` | 3 (center) | Green |
| `J` | 4 | Blue |
| `K` | 5 (right) | Purple |

### Timing Grades

- **PERFECT** (±45ms) — 100 pts
- **GREAT** (±90ms) — 75 pts
- **GOOD** (±140ms) — 50 pts
- **MISS** — breaks your combo

Build combos for score multipliers! Every 10 consecutive hits increases your multiplier.

### Controls

| Key | Action |
|-----|--------|
| `↑`/`↓` | Navigate song select |
| `Enter`/`Space` | Select / Start |
| `Q` / `Esc` | Quit (during play: end song) |

## Songs

| Song | BPM | Difficulty |
|------|-----|------------|
| First Steps | 110 | ⭐ Easy — tutorial, introduces each lane |
| Neon Cascade | 138 | ⭐⭐ Medium — syncopation & off-beats |
| Stardust | 160 | ⭐⭐⭐ Hard — fast arpeggios & chords |

## Terminal Requirements

- Minimum **40×15** characters (bigger is better for the 3D effect)
- **True color** support recommended (most modern terminals)
- Works great in: Kitty, Alacritty, WezTerm, iTerm2, Windows Terminal, Konsole

## How the 3D Works

The track uses **vanishing-point perspective projection**:

- Notes spawn at the vanishing point (top center) and travel towards the hit zone
- Screen Y uses a **quadratic mapping** (`z²`) for realistic foreshortening — far notes are compressed, near notes are spread out
- Track width **scales linearly with depth** — lanes converge at the horizon
- Notes use **depth-based brightness** — dim when far, vivid when close, with a glow near the hit zone
- **Scrolling grid lines** add depth perception and motion cues
- **Bevel shading** on note edges (█ center, ▓ edges) creates a 3D block look
- **Lane flash effects** on keypress give satisfying visual feedback

