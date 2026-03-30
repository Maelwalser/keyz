use crate::game::Song;

pub fn builtin_songs() -> Vec<Song> {
    vec![first_steps(), neon_cascade(), stardust()]
}

/// Tutorial song — introduces lanes one at a time, then builds
fn first_steps() -> Song {
    let mut chart = Vec::new();

    // Section 1: Left hand only (A, S) — quarter notes
    //   Beats 0–7: just lane 0 (A)
    for i in 0..8 {
        chart.push((i as f64, 0));
    }
    //   Beats 8–15: just lane 1 (S)
    for i in 8..16 {
        chart.push((i as f64, 1));
    }

    // Section 2: Right hand only (J, K)
    for i in 16..24 {
        chart.push((i as f64, if i % 2 == 0 { 3 } else { 4 }));
    }

    // Section 3: Alternating left-right
    for i in 24..40 {
        let lane = match i % 4 {
            0 => 0,
            1 => 2,
            2 => 3,
            3 => 4,
            _ => 0,
        };
        chart.push((i as f64, lane));
    }

    // Section 4: Eighth notes, rolling patterns
    for i in 0..32 {
        let beat = 40.0 + i as f64 * 0.5;
        let lane = match i % 8 {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 3,
            6 => 2,
            7 => 1,
            _ => 0,
        };
        chart.push((beat, lane));
    }

    // Section 5: Mixed rhythms
    for i in 0..16 {
        let beat = 56.0 + i as f64;
        chart.push((beat, i % 5));
        // Add off-beat notes on some
        if i % 3 == 0 {
            chart.push((beat + 0.5, (i + 2) % 5));
        }
    }

    Song {
        name: "First Steps".into(),
        artist: "Tutorial".into(),
        bpm: 110.0,
        chart,
        total_beats: 74.0,
    }
}

/// Medium difficulty — syncopation, faster patterns
fn neon_cascade() -> Song {
    let mut chart = Vec::new();

    // Intro: pulsing center lane
    for i in 0..8 {
        chart.push((i as f64, 2));
    }

    // Build: alternating pairs
    for i in 0..16 {
        let beat = 8.0 + i as f64 * 0.5;
        let lane = match i % 4 {
            0 => 0,
            1 => 4,
            2 => 1,
            3 => 3,
            _ => 2,
        };
        chart.push((beat, lane));
    }

    // Chorus: cascading waterfall
    for i in 0..32 {
        let beat = 16.0 + i as f64 * 0.25;
        let lane = i % 5;
        chart.push((beat, lane));
    }

    // Bridge: syncopated groove
    for measure in 0..8 {
        let base = 24.0 + measure as f64 * 4.0;
        // Kick pattern: beats 1 and 3
        chart.push((base, 0));
        chart.push((base + 2.0, 0));
        // Snare: beats 2 and 4
        chart.push((base + 1.0, 4));
        chart.push((base + 3.0, 4));
        // Hihat: off-beats
        chart.push((base + 0.5, 2));
        chart.push((base + 1.5, 2));
        chart.push((base + 2.5, 2));
        chart.push((base + 3.5, 2));
        // Melody accents
        if measure % 2 == 0 {
            chart.push((base + 0.75, 1));
            chart.push((base + 2.75, 3));
        } else {
            chart.push((base + 1.25, 3));
            chart.push((base + 3.25, 1));
        }
    }

    // Outro: rolls getting faster
    for i in 0..20 {
        let speed = 0.5 - (i as f64 * 0.015); // gets faster
        let beat = 56.0 + i as f64 * speed.max(0.2);
        chart.push((beat, i % 5));
    }

    Song {
        name: "Neon Cascade".into(),
        artist: "Synthwave".into(),
        bpm: 138.0,
        chart,
        total_beats: 66.0,
    }
}

/// Hard difficulty — fast patterns, complex rhythms
fn stardust() -> Song {
    let mut chart = Vec::new();

    // Intro: dramatic build
    for i in 0..4 {
        let beat = i as f64 * 2.0;
        chart.push((beat, 2)); // center hit
        chart.push((beat + 1.0, 0));
        chart.push((beat + 1.0, 4));
    }

    // Fast arpeggios
    for group in 0..8 {
        let base = 8.0 + group as f64 * 2.0;
        let pattern: &[usize] = match group % 4 {
            0 => &[0, 1, 2, 3, 4],
            1 => &[4, 3, 2, 1, 0],
            2 => &[0, 2, 4, 2, 0],
            3 => &[1, 3, 4, 3, 1],
            _ => &[2],
        };
        for (j, &lane) in pattern.iter().enumerate() {
            chart.push((base + j as f64 * 0.25, lane));
        }
    }

    // Chorus: dense patterns with chords
    for measure in 0..8 {
        let base = 24.0 + measure as f64 * 4.0;

        // Main rhythm
        for eighth in 0..8 {
            let beat = base + eighth as f64 * 0.5;
            let lane = match (measure + eighth) % 5 {
                0 => 0,
                1 => 1,
                2 => 2,
                3 => 3,
                4 => 4,
                _ => 2,
            };
            chart.push((beat, lane));
        }

        // Chord accents on strong beats
        chart.push((base, 1));
        chart.push((base, 3));
        chart.push((base + 2.0, 0));
        chart.push((base + 2.0, 4));
    }

    // Break: triplet feel
    for i in 0..24 {
        let beat = 56.0 + i as f64 * 0.333;
        let lane = match i % 6 {
            0 => 0,
            1 => 2,
            2 => 4,
            3 => 3,
            4 => 2,
            5 => 1,
            _ => 2,
        };
        chart.push((beat, lane));
    }

    // Finale: everything
    for i in 0..32 {
        let beat = 64.0 + i as f64 * 0.25;
        let lane = match i % 10 {
            0 => 0,
            1 => 1,
            2 => 2,
            3 => 3,
            4 => 4,
            5 => 4,
            6 => 3,
            7 => 2,
            8 => 1,
            9 => 0,
            _ => 2,
        };
        chart.push((beat, lane));
        // Double notes
        if i % 4 == 0 {
            chart.push((beat, 4 - lane));
        }
    }

    Song {
        name: "Stardust".into(),
        artist: "Hardmode".into(),
        bpm: 160.0,
        chart,
        total_beats: 74.0,
    }
}
