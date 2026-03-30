use crossterm::event::KeyCode;
use rand::Rng;
use std::time::Instant;

// ── Lane Configuration ──────────────────────────────────────────────────────

pub const MAX_LANES: usize = 8;

/// Full 8-lane color palette (a s d f h j k l)
pub const ALL_LANE_COLORS: [(u8, u8, u8); MAX_LANES] = [
    (255, 70, 100),  // Hot pink  — a
    (255, 180, 50),  // Orange    — s
    (120, 255, 100), // Green     — d
    (80, 170, 255),  // Blue      — f
    (200, 120, 255), // Purple    — h
    (255, 220, 80),  // Yellow    — j
    (80, 220, 220),  // Cyan      — k
    (255, 140, 50),  // Amber     — l
];

const BEGINNER_KEYS: [char; 4] = ['h', 'j', 'k', 'l'];
const STANDARD_KEYS: [char; 8] = ['a', 's', 'd', 'f', 'h', 'j', 'k', 'l'];

/// Maps 5-lane chart indices to 8 Standard lanes (spread evenly across the board)
const STANDARD_REMAP: [usize; 5] = [0, 2, 4, 5, 7];
/// Maps 5-lane chart indices to 4 Beginner lanes
const BEGINNER_REMAP: [usize; 5] = [0, 1, 2, 2, 3];

// ── Timing Constants ────────────────────────────────────────────────────────

/// How many seconds a note is visible before it reaches the hit zone
pub const APPROACH_TIME: f64 = 2.5;

/// Hit windows in seconds (± from perfect hit time)
pub const WINDOW_PERFECT: f64 = 0.045;
pub const WINDOW_GREAT: f64 = 0.090;
pub const WINDOW_GOOD: f64 = 0.140;
pub const WINDOW_MISS: f64 = 0.200;

/// How long past the hit zone before a note counts as missed
pub const MISS_CUTOFF: f64 = 0.200;

// ── Difficulty ──────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Beginner,
    Standard,
    Hardcore,
}

impl Difficulty {
    pub fn label(&self) -> &'static str {
        match self {
            Difficulty::Beginner => "Beginner",
            Difficulty::Standard => "Standard",
            Difficulty::Hardcore => "Hardcore",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Difficulty::Beginner => "4 lanes  ·  fixed keys: H J K L",
            Difficulty::Standard => "8 lanes  ·  fixed keys: A S D F H J K L",
            Difficulty::Hardcore => "5 lanes  ·  random key on every note",
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            Difficulty::Beginner => (80, 220, 120),
            Difficulty::Standard => (80, 170, 255),
            Difficulty::Hardcore => (255, 80, 80),
        }
    }
}

pub const ALL_DIFFICULTIES: [Difficulty; 3] =
    [Difficulty::Beginner, Difficulty::Standard, Difficulty::Hardcore];

// ── Types ───────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Title,
    DifficultySelect,
    SongSelect,
    Playing,
    Results,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HitGrade {
    Perfect,
    Great,
    Good,
    Miss,
}

impl HitGrade {
    pub fn label(&self) -> &'static str {
        match self {
            HitGrade::Perfect => "PERFECT",
            HitGrade::Great => "GREAT",
            HitGrade::Good => "GOOD",
            HitGrade::Miss => "MISS",
        }
    }

    pub fn color(&self) -> (u8, u8, u8) {
        match self {
            HitGrade::Perfect => (255, 255, 80),
            HitGrade::Great => (80, 255, 120),
            HitGrade::Good => (80, 180, 255),
            HitGrade::Miss => (255, 60, 60),
        }
    }

    pub fn points(&self) -> u32 {
        match self {
            HitGrade::Perfect => 100,
            HitGrade::Great => 75,
            HitGrade::Good => 50,
            HitGrade::Miss => 0,
        }
    }
}

// ── Note ────────────────────────────────────────────────────────────────────

pub struct Note {
    pub lane: usize,
    /// The key the player must press to hit this note
    pub key: char,
    /// Absolute game time (seconds) when this note should be hit
    pub hit_time: f64,
    pub active: bool,
    pub hit_grade: Option<HitGrade>,
}

impl Note {
    /// Returns the note's z-position: 0.0 = far (just spawned), 1.0 = at hit zone
    pub fn z_position(&self, game_time: f64) -> f64 {
        let time_until_hit = self.hit_time - game_time;
        1.0 - (time_until_hit / APPROACH_TIME)
    }
}

// ── Song ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct Song {
    pub name: String,
    pub artist: String,
    pub bpm: f64,
    /// (beat_number, lane_index) — charts are always authored for 5 lanes (0–4)
    pub chart: Vec<(f64, usize)>,
    pub total_beats: f64,
}

impl Song {
    pub fn beat_duration(&self) -> f64 {
        60.0 / self.bpm
    }

    pub fn duration_secs(&self) -> f64 {
        self.total_beats * self.beat_duration()
    }
}

// ── Game State ──────────────────────────────────────────────────────────────

pub struct Game {
    pub phase: GamePhase,
    pub difficulty: Difficulty,
    pub selected_difficulty: usize,

    pub songs: Vec<Song>,
    pub selected_song: usize,

    // Active game state
    pub notes: Vec<Note>,
    pub game_time: f64,
    pub start_instant: Option<Instant>,
    pub last_frame: Option<Instant>,

    // Pre-play countdown
    pub countdown: Option<f64>,

    // Scoring
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub total_notes: u32,
    pub hit_counts: [u32; 4], // perfect, great, good, miss

    // Visual feedback
    pub last_hit: Option<(HitGrade, Instant)>,
    pub lane_flash: [Option<Instant>; MAX_LANES],
}

impl Game {
    pub fn new() -> Self {
        let songs = crate::songs::builtin_songs();
        Game {
            phase: GamePhase::Title,
            difficulty: Difficulty::Standard,
            selected_difficulty: 1, // default to Standard
            songs,
            selected_song: 0,
            notes: Vec::new(),
            game_time: 0.0,
            start_instant: None,
            last_frame: None,
            countdown: None,
            score: 0,
            combo: 0,
            max_combo: 0,
            total_notes: 0,
            hit_counts: [0; 4],
            last_hit: None,
            lane_flash: [None; MAX_LANES],
        }
    }

    // ── Difficulty helpers ────────────────────────────────────────────────

    pub fn num_lanes(&self) -> usize {
        match self.difficulty {
            Difficulty::Beginner => 4,
            Difficulty::Standard => 8,
            Difficulty::Hardcore => 5,
        }
    }

    pub fn lane_color(&self, lane: usize) -> (u8, u8, u8) {
        match self.difficulty {
            // Beginner uses hjkl → ALL_LANE_COLORS indices 4-7
            Difficulty::Beginner => ALL_LANE_COLORS[lane + 4],
            Difficulty::Standard => ALL_LANE_COLORS[lane],
            Difficulty::Hardcore => ALL_LANE_COLORS[lane], // indices 0-4
        }
    }

    /// Returns the key to display for a lane.
    /// Fixed-key modes always return the lane key; Hardcore returns the next note's key.
    pub fn display_key_for_lane(&self, lane: usize) -> char {
        match self.difficulty {
            Difficulty::Beginner => BEGINNER_KEYS[lane],
            Difficulty::Standard => STANDARD_KEYS[lane],
            Difficulty::Hardcore => self.next_key_for_lane(lane).unwrap_or('·'),
        }
    }

    // ── Song / game lifecycle ─────────────────────────────────────────────

    pub fn start_song(&mut self) {
        let song = &self.songs[self.selected_song];
        let beat_dur = song.beat_duration();

        let offset = APPROACH_TIME + 1.5; // extra 1.5s lead-in
        let mut rng = rand::thread_rng();

        self.notes = song
            .chart
            .iter()
            .map(|&(beat, chart_lane)| {
                let src = chart_lane.min(4);
                let lane = match self.difficulty {
                    Difficulty::Beginner => BEGINNER_REMAP[src],
                    Difficulty::Standard => STANDARD_REMAP[src],
                    Difficulty::Hardcore => src,
                };
                let key = match self.difficulty {
                    Difficulty::Beginner => BEGINNER_KEYS[lane],
                    Difficulty::Standard => STANDARD_KEYS[lane],
                    Difficulty::Hardcore => (b'a' + rng.gen_range(0u8..26)) as char,
                };
                Note {
                    lane,
                    key,
                    hit_time: beat * beat_dur + offset,
                    active: true,
                    hit_grade: None,
                }
            })
            .collect();

        self.total_notes = self.notes.len() as u32;
        self.score = 0;
        self.combo = 0;
        self.max_combo = 0;
        self.hit_counts = [0; 4];
        self.last_hit = None;
        self.lane_flash = [None; MAX_LANES];
        self.game_time = 0.0;
        self.countdown = Some(3.0);
        self.start_instant = Some(Instant::now());
        self.last_frame = Some(Instant::now());
        self.phase = GamePhase::Playing;
    }

    pub fn update(&mut self) {
        if self.phase != GamePhase::Playing {
            return;
        }

        let now = Instant::now();
        if let Some(last) = self.last_frame {
            let dt = now.duration_since(last).as_secs_f64();

            // Handle countdown
            if let Some(ref mut cd) = self.countdown {
                *cd -= dt;
                if *cd <= 0.0 {
                    self.countdown = None;
                    self.start_instant = Some(Instant::now());
                    self.game_time = 0.0;
                } else {
                    self.last_frame = Some(now);
                    return;
                }
            }

            self.game_time += dt;
        }
        self.last_frame = Some(now);

        // Check for missed notes (past the miss cutoff)
        for note in &mut self.notes {
            if note.active && self.game_time > note.hit_time + MISS_CUTOFF {
                note.active = false;
                note.hit_grade = Some(HitGrade::Miss);
                self.combo = 0;
                self.hit_counts[3] += 1;
                self.last_hit = Some((HitGrade::Miss, Instant::now()));
            }
        }

        // Check if song is over
        let song = &self.songs[self.selected_song];
        if self.game_time > song.duration_secs() + APPROACH_TIME + 2.0 {
            self.phase = GamePhase::Results;
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.phase {
            GamePhase::Title => {
                if matches!(key, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.phase = GamePhase::DifficultySelect;
                }
            }
            GamePhase::DifficultySelect => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_difficulty > 0 {
                        self.selected_difficulty -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_difficulty < ALL_DIFFICULTIES.len() - 1 {
                        self.selected_difficulty += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.difficulty = ALL_DIFFICULTIES[self.selected_difficulty];
                    self.phase = GamePhase::SongSelect;
                }
                _ => {}
            },
            GamePhase::SongSelect => match key {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_song > 0 {
                        self.selected_song -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected_song < self.songs.len() - 1 {
                        self.selected_song += 1;
                    }
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    self.start_song();
                }
                _ => {}
            },
            GamePhase::Playing => {
                if self.countdown.is_some() {
                    return;
                }
                if let KeyCode::Char(ch) = key {
                    self.try_hit(ch);
                }
            }
            GamePhase::Results => {
                if matches!(key, KeyCode::Enter | KeyCode::Char(' ')) {
                    self.phase = GamePhase::SongSelect;
                }
            }
            GamePhase::Quit => {}
        }
    }

    fn try_hit(&mut self, ch: char) {
        // Find the closest active note with the matching key within the hit window
        let mut best: Option<(usize, f64, HitGrade)> = None;

        for (i, note) in self.notes.iter().enumerate() {
            if !note.active || note.key != ch {
                continue;
            }
            let diff = (note.hit_time - self.game_time).abs();
            if diff > WINDOW_MISS {
                continue;
            }

            let grade = if diff <= WINDOW_PERFECT {
                HitGrade::Perfect
            } else if diff <= WINDOW_GREAT {
                HitGrade::Great
            } else if diff <= WINDOW_GOOD {
                HitGrade::Good
            } else {
                HitGrade::Miss
            };

            if best.is_none() || diff < best.unwrap().1 {
                best = Some((i, diff, grade));
            }
        }

        if let Some((idx, _, grade)) = best {
            let lane = self.notes[idx].lane;
            self.notes[idx].active = false;
            self.notes[idx].hit_grade = Some(grade);
            self.lane_flash[lane] = Some(Instant::now());

            match grade {
                HitGrade::Perfect => {
                    self.hit_counts[0] += 1;
                    self.combo += 1;
                }
                HitGrade::Great => {
                    self.hit_counts[1] += 1;
                    self.combo += 1;
                }
                HitGrade::Good => {
                    self.hit_counts[2] += 1;
                    self.combo += 1;
                }
                HitGrade::Miss => {
                    self.hit_counts[3] += 1;
                    self.combo = 0;
                }
            }

            if self.combo > self.max_combo {
                self.max_combo = self.combo;
            }

            let multiplier = 1 + self.combo / 10;
            self.score = self.score.saturating_add(grade.points().saturating_mul(multiplier));
            self.last_hit = Some((grade, Instant::now()));
        }
    }

    pub fn accuracy(&self) -> f64 {
        let total_hit =
            self.hit_counts[0] + self.hit_counts[1] + self.hit_counts[2] + self.hit_counts[3];
        if total_hit == 0 {
            return 100.0;
        }
        let weighted = self.hit_counts[0] as f64 * 1.0
            + self.hit_counts[1] as f64 * 0.75
            + self.hit_counts[2] as f64 * 0.5;
        (weighted / total_hit as f64) * 100.0
    }

    pub fn grade_letter(&self) -> &'static str {
        let acc = self.accuracy();
        if acc >= 95.0 {
            "S"
        } else if acc >= 90.0 {
            "A"
        } else if acc >= 80.0 {
            "B"
        } else if acc >= 70.0 {
            "C"
        } else if acc >= 60.0 {
            "D"
        } else {
            "F"
        }
    }

    /// Returns the key char of the next upcoming active note in the given lane, if any
    pub fn next_key_for_lane(&self, lane: usize) -> Option<char> {
        self.notes
            .iter()
            .filter(|n| n.active && n.lane == lane)
            .min_by(|a, b| a.hit_time.partial_cmp(&b.hit_time).unwrap())
            .map(|n| n.key)
    }

    pub fn song_progress(&self) -> f64 {
        let song = &self.songs[self.selected_song];
        let dur = song.duration_secs();
        if dur <= 0.0 {
            return 0.0;
        }
        (self.game_time / dur).clamp(0.0, 1.0)
    }
}
