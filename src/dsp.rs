// ── DSP: Audio Analysis Pipeline ────────────────────────────────────────────
//
// Converts a WAV file into a rhythm game chart via:
//   1. WAV decode (hound) → mono f32 samples
//   2. Spectral flux onset detection (rustfft)
//   3. BPM estimation via IOI histogram
//   4. Beat quantization (sixteenth-note grid)
//   5. Lane mapping via frequency band energy

use rustfft::{FftPlanner, num_complex::Complex};
use std::path::Path;

pub struct AnalysisResult {
    pub bpm: f64,
    pub chart: Vec<(f64, usize)>,       // (beat_number, lane 0-4)
    pub hold_durations: Vec<f64>,        // seconds to keep track unmuted after each hit
    pub total_beats: f64,
}

pub fn analyze_audio(path: &Path) -> Result<AnalysisResult, String> {
    // ── Step A: Decode WAV ───────────────────────────────────────────────────
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV: {}", e))?;

    let spec = reader.spec();
    let sample_rate = spec.sample_rate as f64;
    let channels = spec.channels as usize;

    let samples_raw: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map_err(|e| format!("Read error: {}", e)))
            .collect::<Result<Vec<_>, _>>()?,
        hound::SampleFormat::Int => {
            let max_val = (1i64 << (spec.bits_per_sample - 1)) as f32;
            match spec.bits_per_sample {
                16 => reader
                    .samples::<i16>()
                    .map(|s| s.map(|v| v as f32 / max_val).map_err(|e| format!("Read error: {}", e)))
                    .collect::<Result<Vec<_>, _>>()?,
                24 | 32 => reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / max_val).map_err(|e| format!("Read error: {}", e)))
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err(format!("Unsupported bit depth: {}", spec.bits_per_sample)),
            }
        }
    };

    // Mix to mono by averaging channels
    let mono: Vec<f32> = if channels == 1 {
        samples_raw
    } else {
        samples_raw
            .chunks(channels)
            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
            .collect()
    };

    let duration_secs = mono.len() as f64 / sample_rate;
    if duration_secs < 1.0 {
        return Err("Audio too short (< 1 second)".to_string());
    }

    // ── Step B: Spectral flux onset detection ───────────────────────────────
    const FRAME_SIZE: usize = 1024;
    const HOP_SIZE: usize = 512;

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(FRAME_SIZE);

    let mut prev_mag = vec![0.0f32; FRAME_SIZE / 2];
    let mut flux_values: Vec<f32> = Vec::new();
    // Store per-frame magnitude spectra for lane analysis later
    let mut frame_spectra: Vec<Vec<f32>> = Vec::new();

    let num_frames = (mono.len().saturating_sub(FRAME_SIZE)) / HOP_SIZE + 1;

    for frame_idx in 0..num_frames {
        let start = frame_idx * HOP_SIZE;
        let end = (start + FRAME_SIZE).min(mono.len());

        // Build windowed frame (zero-pad if needed)
        let mut buf: Vec<Complex<f32>> = (0..FRAME_SIZE)
            .map(|i| {
                let sample = if start + i < end { mono[start + i] } else { 0.0 };
                // Hann window
                let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (FRAME_SIZE - 1) as f32).cos());
                Complex::new(sample * w, 0.0)
            })
            .collect();

        fft.process(&mut buf);

        // Magnitude spectrum (positive frequencies only)
        let mag: Vec<f32> = buf[..FRAME_SIZE / 2]
            .iter()
            .map(|c| c.norm())
            .collect();

        // Half-wave rectified spectral flux
        let flux: f32 = mag
            .iter()
            .zip(prev_mag.iter())
            .map(|(m, p)| (m - p).max(0.0))
            .sum();

        flux_values.push(flux);
        frame_spectra.push(mag.clone());
        prev_mag = mag;
    }

    // ── Step C: Peak-picking with adaptive threshold ─────────────────────────
    let window_half = 4usize;
    let threshold_multiplier = 1.5f32;

    let onsets: Vec<(usize, f32)> = flux_values
        .iter()
        .enumerate()
        .filter(|&(i, &flux)| {
            // Must be a local maximum
            let lo = i.saturating_sub(1);
            let hi = (i + 1).min(flux_values.len() - 1);
            if flux <= flux_values[lo] || flux <= flux_values[hi] {
                return false;
            }
            // Must exceed adaptive threshold
            let win_lo = i.saturating_sub(window_half);
            let win_hi = (i + window_half + 1).min(flux_values.len());
            let mean = flux_values[win_lo..win_hi].iter().sum::<f32>()
                / (win_hi - win_lo) as f32;
            flux > mean * threshold_multiplier
        })
        .map(|(i, &f)| (i, f))
        .collect();

    if onsets.is_empty() {
        return Err("No onsets detected in audio".to_string());
    }

    // Convert frame indices to seconds
    let onset_times: Vec<f64> = onsets
        .iter()
        .map(|&(frame, _)| frame as f64 * HOP_SIZE as f64 / sample_rate)
        .collect();

    // ── Step D: BPM estimation via IOI histogram ─────────────────────────────
    let iois: Vec<f64> = onset_times
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|&ioi| ioi >= 0.2 && ioi <= 2.0) // 30–300 BPM range
        .collect();

    let bpm = if iois.is_empty() {
        120.0
    } else {
        // Histogram with 1ms bins
        const BIN_MS: f64 = 0.001;
        const NUM_BINS: usize = 1800; // 0.2s to 2.0s
        let mut hist = vec![0u32; NUM_BINS];
        for &ioi in &iois {
            let bin = ((ioi - 0.2) / BIN_MS) as usize;
            if bin < NUM_BINS {
                hist[bin] += 1;
            }
        }
        let modal_bin = hist
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(i, _)| i)
            .unwrap_or(0);
        let modal_ioi = 0.2 + modal_bin as f64 * BIN_MS;
        (60.0 / modal_ioi).clamp(40.0, 300.0)
    };

    // ── Step E: Quantize onsets to beats and map to lanes ───────────────────
    let beat_duration = 60.0 / bpm;
    const GRID: f64 = 0.25; // sixteenth note

    // For each onset, compute beat position and frequency band
    struct OnsetEntry {
        beat_slot: i64, // quantized beat * 4 (sixteenth note grid)
        beat_num: f64,
        lane: usize,
        flux: f32,
    }

    let entries: Vec<OnsetEntry> = onsets
        .iter()
        .zip(onset_times.iter())
        .map(|(&(frame_idx, flux), &t)| {
            let raw_beat = t / beat_duration;
            let slot = (raw_beat / GRID).round() as i64;
            let beat_num = slot as f64 * GRID;

            // Lane from frequency band energy
            let mag = &frame_spectra[frame_idx];
            let bands = [
                mag[1..12.min(mag.len())].iter().map(|&v| v as f64).sum::<f64>(),
                mag[12..30.min(mag.len())].iter().map(|&v| v as f64).sum::<f64>(),
                mag[30..80.min(mag.len())].iter().map(|&v| v as f64).sum::<f64>(),
                mag[80..200.min(mag.len())].iter().map(|&v| v as f64).sum::<f64>(),
                mag[200..mag.len()].iter().map(|&v| v as f64).sum::<f64>(),
            ];
            let lane = bands
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .map(|(i, _)| i)
                .unwrap_or(2);

            OnsetEntry { beat_slot: slot, beat_num, lane, flux }
        })
        .collect();

    // Deduplicate: keep highest-flux onset per grid slot
    use std::collections::HashMap;
    let mut best_per_slot: HashMap<i64, OnsetEntry> = HashMap::new();
    for entry in entries {
        let existing = best_per_slot.get(&entry.beat_slot);
        if existing.map_or(true, |e: &OnsetEntry| entry.flux > e.flux) {
            best_per_slot.insert(entry.beat_slot, entry);
        }
    }

    // Collect into a vec of (beat_num, lane, flux) sorted by beat
    let mut sorted_entries: Vec<(f64, usize, f32)> = best_per_slot
        .values()
        .filter(|e| e.beat_num >= 0.0)
        .map(|e| (e.beat_num, e.lane, e.flux))
        .collect();
    sorted_entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    // ── Per-lane merge: runs of same-lane notes → hold notes ───────────────
    // Merge same-lane consecutive notes within 2 beats into one hold note.
    // Single notes with a long IOI (≥ 0.5s) also become short holds.
    const MERGE_BEATS: f64 = 2.0;
    const HOLD_IOI_MIN_SECS: f64 = 0.5;

    // Group indices per lane
    let mut lane_buckets: Vec<Vec<usize>> = vec![Vec::new(); 5];
    for (i, &(_, lane, _)) in sorted_entries.iter().enumerate() {
        lane_buckets[lane].push(i);
    }

    let mut hold_secs_per_entry = vec![0.0f64; sorted_entries.len()];
    let mut to_remove: std::collections::HashSet<usize> = std::collections::HashSet::new();

    for lane in 0..5 {
        let bucket = &lane_buckets[lane];
        if bucket.len() < 2 {
            continue;
        }
        let mut i = 0;
        while i < bucket.len() {
            let idx_start = bucket[i];
            let beat_start = sorted_entries[idx_start].0;
            let mut run_end = beat_start;
            let mut j = i + 1;
            while j < bucket.len() {
                let (beat_j, _, _) = sorted_entries[bucket[j]];
                if beat_j - run_end <= MERGE_BEATS {
                    run_end = beat_j;
                    to_remove.insert(bucket[j]);
                    j += 1;
                } else {
                    break;
                }
            }
            if j > i + 1 {
                // Merged group: hold from beat_start to run_end + half a beat
                let hold = ((run_end - beat_start + 0.5) * beat_duration).clamp(0.2, 4.0);
                hold_secs_per_entry[idx_start] = hold;
            }
            i = j;
        }
    }

    // Single (unmerged) notes: if IOI to next is long enough, make a short hold
    for i in 0..sorted_entries.len() {
        if to_remove.contains(&i) || hold_secs_per_entry[i] > 0.0 {
            continue;
        }
        if let Some(j) = (i + 1..sorted_entries.len()).find(|&j| !to_remove.contains(&j)) {
            let ioi = (sorted_entries[j].0 - sorted_entries[i].0) * beat_duration;
            if ioi >= HOLD_IOI_MIN_SECS {
                hold_secs_per_entry[i] = (ioi * 0.70).clamp(0.2, 2.0);
            }
        }
    }

    // Build final chart and hold_durations (0.0 = tap)
    let mut chart: Vec<(f64, usize)> = Vec::new();
    let mut hold_durations: Vec<f64> = Vec::new();
    for (i, &(beat, lane, _)) in sorted_entries.iter().enumerate() {
        if to_remove.contains(&i) {
            continue;
        }
        chart.push((beat, lane));
        hold_durations.push(hold_secs_per_entry[i]);
    }

    let total_beats = (duration_secs / beat_duration).ceil() + 4.0;

    Ok(AnalysisResult {
        bpm,
        chart,
        hold_durations,
        total_beats,
    })
}
