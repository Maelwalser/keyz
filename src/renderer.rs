use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    Frame,
};

use crate::game::*;

// ── Perspective Math ────────────────────────────────────────────────────────
//
// The track is rendered with a vanishing-point perspective:
//   - z = 0.0  →  far away (vanishing point at top of screen)
//   - z = 1.0  →  hit zone (bottom of track)
//
// Screen-Y mapping uses z² for foreshortening:
//   y = vanish_y + (hit_y - vanish_y) · z²
//
// Track width scales linearly with z:
//   half_width = max_half_width · z

fn z_to_screen_y(z: f64, vanish_y: f64, hit_y: f64) -> f64 {
    vanish_y + (hit_y - vanish_y) * z * z
}

fn screen_y_to_z(y: f64, vanish_y: f64, hit_y: f64) -> f64 {
    let t = ((y - vanish_y) / (hit_y - vanish_y)).clamp(0.0, 1.0);
    t.sqrt()
}

// ── Main Render Dispatch ────────────────────────────────────────────────────

pub fn render(frame: &mut Frame, game: &Game) {
    let area = frame.area();
    match game.phase {
        GamePhase::Title => render_title(frame.buffer_mut(), area),
        GamePhase::DifficultySelect => render_difficulty_select(frame.buffer_mut(), area, game),
        GamePhase::SongSelect => render_song_select(frame.buffer_mut(), area, game),
        GamePhase::Playing => render_gameplay(frame.buffer_mut(), area, game),
        GamePhase::Results => render_results(frame.buffer_mut(), area, game),
        GamePhase::Quit => {}
    }
}

// ── Title Screen ────────────────────────────────────────────────────────────

fn render_title(buf: &mut Buffer, area: Rect) {
    let cx = area.width / 2;
    let cy = area.height / 2;

    let logo = [
        r"  ╦═╗ ╦ ╦ ╦ ╦ ╔╦╗ ╦ ╦ ╔╦╗ ",
        r"  ╠╦╝ ╠═╣ ╚╦╝  ║  ╠═╣ ║║║ ",
        r"  ╩╚═ ╩ ╩  ╩   ╩  ╩ ╩ ╩ ╩ ",
        r"     ╦╔═ ╔═╗ ╦ ╦ ╔═╗       ",
        r"     ╠╩╗ ║╣  ╚╦╝ ╚═╗       ",
        r"     ╩ ╩ ╚═╝  ╩  ╚═╝       ",
    ];

    let logo_width = logo[0].chars().count() as u16;
    let start_y = cy.saturating_sub(5);
    let start_x = cx.saturating_sub(logo_width / 2);

    for (i, line) in logo.iter().enumerate() {
        let y = start_y + i as u16;
        if y >= area.height {
            break;
        }
        let hue = (i as f64 / logo.len() as f64 * 120.0) as u8;
        let color = Color::Rgb(255, 100 + hue, 200 - hue);
        draw_str(buf, start_x, y, line, Style::default().fg(color));
    }

    let subtitle = "── A 3D Terminal Rhythm Game ──";
    let sub_x = cx.saturating_sub(subtitle.len() as u16 / 2);
    draw_str(
        buf,
        sub_x,
        start_y + logo.len() as u16 + 1,
        subtitle,
        Style::default().fg(Color::Rgb(180, 140, 220)),
    );

    let prompt = "[ Press ENTER to start ]";
    let px = cx.saturating_sub(prompt.len() as u16 / 2);
    let blink = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 600)
        % 2
        == 0;
    if blink {
        draw_str(
            buf,
            px,
            start_y + logo.len() as u16 + 4,
            prompt,
            Style::default().fg(Color::Rgb(200, 200, 200)),
        );
    }

    // Key guide at bottom
    let guide = "Type the key shown on each incoming note   │   Quit: Esc";
    let gx = cx.saturating_sub(guide.len() as u16 / 2);
    draw_str(
        buf,
        gx,
        area.height.saturating_sub(2),
        guide,
        Style::default().fg(Color::Rgb(100, 100, 120)),
    );
}

// ── Difficulty Select ────────────────────────────────────────────────────────

fn render_difficulty_select(buf: &mut Buffer, area: Rect, game: &Game) {
    let cx = area.width / 2;
    let start_y = area.height / 4;

    let header = "─── SELECT DIFFICULTY ───";
    draw_str(
        buf,
        cx.saturating_sub(header.len() as u16 / 2),
        start_y,
        header,
        Style::default().fg(Color::Rgb(255, 200, 100)),
    );

    for (i, diff) in ALL_DIFFICULTIES.iter().enumerate() {
        let y = start_y + 2 + i as u16 * 4;
        if y + 2 >= area.height {
            break;
        }

        let selected = i == game.selected_difficulty;
        let arrow = if selected { "▸ " } else { "  " };
        let (dr, dg, db) = diff.color();

        let name_color = if selected {
            Color::Rgb(dr, dg, db)
        } else {
            Color::Rgb(dr / 2, dg / 2, db / 2)
        };
        let desc_color = if selected {
            Color::Rgb(180, 180, 200)
        } else {
            Color::Rgb(80, 80, 100)
        };

        let x = cx.saturating_sub(22);
        let name_line = format!("{}{}", arrow, diff.label());
        draw_str(buf, x, y, &name_line, Style::default().fg(name_color));
        draw_str(
            buf,
            x + 2,
            y + 1,
            diff.description(),
            Style::default().fg(desc_color),
        );

        if selected {
            for col in x.saturating_sub(1)..=(x + 45).min(area.width - 1) {
                if let Some(cell) = buf.cell_mut((col, y)) {
                    cell.set_bg(Color::Rgb(20, 20, 35));
                }
                if let Some(cell) = buf.cell_mut((col, y + 1)) {
                    cell.set_bg(Color::Rgb(20, 20, 35));
                }
            }
        }
    }

    let hint = "↑/↓ Navigate   ENTER Select   Esc Back";
    draw_str(
        buf,
        cx.saturating_sub(hint.len() as u16 / 2),
        area.height.saturating_sub(2),
        hint,
        Style::default().fg(Color::Rgb(100, 100, 120)),
    );
}

// ── Song Select ─────────────────────────────────────────────────────────────

fn render_song_select(buf: &mut Buffer, area: Rect, game: &Game) {
    let cx = area.width / 2;
    let start_y = area.height / 4;

    // Show selected difficulty at top
    let (dr, dg, db) = game.difficulty.color();
    let diff_label = format!(
        "── {} ─ {} ──",
        game.difficulty.label(),
        game.difficulty.description()
    );
    draw_str(
        buf,
        cx.saturating_sub(diff_label.len() as u16 / 2),
        start_y.saturating_sub(2),
        &diff_label,
        Style::default().fg(Color::Rgb(dr, dg, db)),
    );

    let header = "─── SELECT SONG ───";
    draw_str(
        buf,
        cx.saturating_sub(header.len() as u16 / 2),
        start_y,
        header,
        Style::default().fg(Color::Rgb(255, 200, 100)),
    );

    for (i, song) in game.songs.iter().enumerate() {
        let y = start_y + 2 + i as u16 * 3;
        if y + 1 >= area.height {
            break;
        }

        let selected = i == game.selected_song;
        let arrow = if selected { "▸ " } else { "  " };
        let name_color = if selected {
            Color::Rgb(255, 255, 255)
        } else {
            Color::Rgb(120, 120, 140)
        };
        let detail_color = if selected {
            Color::Rgb(180, 180, 200)
        } else {
            Color::Rgb(80, 80, 100)
        };

        let name_line = format!("{}{}  —  {}", arrow, song.name, song.artist);
        let detail_line = format!(
            "    BPM: {}   Notes: {}   Duration: {:.0}s",
            song.bpm,
            song.chart.len(),
            song.duration_secs()
        );

        let x = cx.saturating_sub(25);
        draw_str(buf, x, y, &name_line, Style::default().fg(name_color));
        draw_str(buf, x, y + 1, &detail_line, Style::default().fg(detail_color));

        if selected {
            for col in x.saturating_sub(1)..=(x + 55).min(area.width - 1) {
                if let Some(cell) = buf.cell_mut((col, y)) {
                    cell.set_bg(Color::Rgb(30, 25, 45));
                }
            }
        }
    }

    let hint = "↑/↓ Navigate   ENTER Play   Esc Back   Q Quit";
    draw_str(
        buf,
        cx.saturating_sub(hint.len() as u16 / 2),
        area.height.saturating_sub(2),
        hint,
        Style::default().fg(Color::Rgb(100, 100, 120)),
    );
}

// ── Gameplay ────────────────────────────────────────────────────────────────

fn render_gameplay(buf: &mut Buffer, area: Rect, game: &Game) {
    if area.width < 40 || area.height < 15 {
        draw_str(
            buf,
            1,
            1,
            "Terminal too small! Need 40×15+",
            Style::default().fg(Color::Red),
        );
        return;
    }

    let vanish_y = area.y as f64 + 2.0;
    let hit_y = (area.y + area.height) as f64 - 4.0;
    let center_x = area.width as f64 / 2.0;
    let max_half_width = (area.width as f64 * 0.38).min(40.0);
    let num_lanes = game.num_lanes();

    // Handle countdown
    if let Some(cd) = game.countdown {
        let count_num = cd.ceil() as u32;
        let count_str = if count_num == 0 {
            "GO!".to_string()
        } else {
            count_num.to_string()
        };
        let cx = (center_x - count_str.len() as f64 / 2.0) as u16;
        let cy = area.height / 2;
        let pulse = ((cd.fract() * std::f64::consts::PI * 2.0).sin() * 0.3 + 0.7) as f64;
        let brightness = (pulse * 255.0) as u8;
        draw_str(
            buf,
            cx,
            cy,
            &count_str,
            Style::default().fg(Color::Rgb(brightness, brightness, brightness)),
        );
        return;
    }

    draw_track_background(buf, area, vanish_y, hit_y, center_x, max_half_width);
    draw_lane_dividers(buf, area, vanish_y, hit_y, center_x, max_half_width, num_lanes);
    draw_depth_grid(buf, area, vanish_y, hit_y, center_x, max_half_width, game.game_time);
    draw_notes(buf, area, vanish_y, hit_y, center_x, max_half_width, game);
    draw_hit_zone(buf, area, vanish_y, hit_y, center_x, max_half_width, game);

    // ── Next-key indicators below hit zone ─────────────────────────────
    let label_y = (hit_y + 2.0) as u16;
    if label_y < area.y + area.height {
        let hw = max_half_width;
        let lane_w = (2.0 * hw) / num_lanes as f64;
        for lane in 0..num_lanes {
            let lane_center = center_x - hw + lane_w * (lane as f64 + 0.5);
            let x = lane_center as u16;
            let (r, g, b) = game.lane_color(lane);

            let flash = game.lane_flash[lane]
                .map(|t| {
                    let elapsed = t.elapsed().as_secs_f64();
                    if elapsed < 0.15 { 1.0 - elapsed / 0.15 } else { 0.0 }
                })
                .unwrap_or(0.0);

            let br = ((r as f64 + (255.0 - r as f64) * flash) as u8).min(255);
            let bg_c = ((g as f64 + (255.0 - g as f64) * flash) as u8).min(255);
            let bb = ((b as f64 + (255.0 - b as f64) * flash) as u8).min(255);

            if x > 0 && x < area.width - 1 {
                let key_char = game.display_key_for_lane(lane).to_ascii_uppercase();
                let label = format!("[{}]", key_char);
                let lx = x.saturating_sub(1);
                draw_str(
                    buf,
                    lx,
                    label_y,
                    &label,
                    Style::default().fg(Color::Rgb(br, bg_c, bb)),
                );
            }
        }
    }

    draw_hud(buf, area, game);
}

fn draw_track_background(
    buf: &mut Buffer,
    area: Rect,
    vanish_y: f64,
    hit_y: f64,
    center_x: f64,
    max_hw: f64,
) {
    let y_start = vanish_y.max(area.y as f64) as u16;
    let y_end = hit_y.min((area.y + area.height - 1) as f64) as u16;

    for row in y_start..=y_end {
        let z = screen_y_to_z(row as f64, vanish_y, hit_y);
        let hw = max_hw * z;
        let left = ((center_x - hw).max(area.x as f64)) as u16;
        let right = ((center_x + hw).min((area.x + area.width - 1) as f64)) as u16;

        let depth_shade = (z * 18.0) as u8 + 4;
        let bg_color = Color::Rgb(depth_shade, depth_shade, depth_shade + 6);

        for col in left..=right {
            if let Some(c) = buf.cell_mut((col, row)) {
                c.set_char(' ').set_bg(bg_color);
            }
        }
    }
}

fn draw_lane_dividers(
    buf: &mut Buffer,
    area: Rect,
    vanish_y: f64,
    hit_y: f64,
    center_x: f64,
    max_hw: f64,
    num_lanes: usize,
) {
    let y_start = vanish_y.max(area.y as f64) as u16;
    let y_end = hit_y.min((area.y + area.height - 1) as f64) as u16;

    for row in y_start..=y_end {
        let z = screen_y_to_z(row as f64, vanish_y, hit_y);
        let hw = max_hw * z;

        for i in 0..=num_lanes {
            let x = center_x - hw + (2.0 * hw * i as f64 / num_lanes as f64);
            let col = x as u16;
            if col >= area.x && col < area.x + area.width {
                let brightness = (z * 55.0) as u8 + 25;
                if let Some(c) = buf.cell_mut((col, row)) {
                    c.set_char('│')
                        .set_fg(Color::Rgb(brightness, brightness, brightness + 20))
                        .set_bg(Color::Rgb(8, 8, 12));
                }
            }
        }
    }
}

fn draw_depth_grid(
    buf: &mut Buffer,
    area: Rect,
    vanish_y: f64,
    hit_y: f64,
    center_x: f64,
    max_hw: f64,
    game_time: f64,
) {
    let num_grid_lines = 12;
    let scroll_offset = (game_time * 0.4) % 1.0;

    for i in 0..num_grid_lines {
        let z_base = (i as f64 + scroll_offset) / num_grid_lines as f64;
        if z_base < 0.05 || z_base > 0.98 {
            continue;
        }

        let y = z_to_screen_y(z_base, vanish_y, hit_y);
        let row = y as u16;
        if row < area.y || row >= area.y + area.height {
            continue;
        }

        let hw = max_hw * z_base;
        let left = ((center_x - hw).max(area.x as f64)) as u16 + 1;
        let right = ((center_x + hw).min((area.x + area.width - 1) as f64)) as u16;

        let brightness = (z_base * 25.0) as u8 + 10;
        let style = Style::default().fg(Color::Rgb(brightness, brightness, brightness + 8));

        for col in left..right {
            if let Some(cell) = buf.cell_mut((col, row)) {
                if cell.symbol() == " " {
                    cell.set_char('·').set_style(style);
                }
            }
        }
    }
}

fn draw_notes(
    buf: &mut Buffer,
    area: Rect,
    vanish_y: f64,
    hit_y: f64,
    center_x: f64,
    max_hw: f64,
    game: &Game,
) {
    let note_z_thickness = 0.035;
    let num_lanes = game.num_lanes();

    for note in &game.notes {
        if !note.active {
            continue;
        }

        let z = note.z_position(game.game_time);
        if z < -0.05 || z > 1.15 {
            continue;
        }

        let z_front = z.min(1.05);
        let z_back = (z - note_z_thickness).max(0.0);

        let y_front = z_to_screen_y(z_front, vanish_y, hit_y);
        let y_back = z_to_screen_y(z_back, vanish_y, hit_y);

        let row_top = (y_back as u16).max(area.y);
        let row_bot = (y_front as u16).min(area.y + area.height - 1);

        let (base_r, base_g, base_b) = game.lane_color(note.lane);

        for row in row_top..=row_bot {
            let z_at_row = screen_y_to_z(row as f64, vanish_y, hit_y);
            let hw = max_hw * z_at_row;

            let lane_w = (2.0 * hw) / num_lanes as f64;
            let lane_left = center_x - hw + lane_w * note.lane as f64;
            let lane_right = lane_left + lane_w;

            let pad = (lane_w * 0.12).max(0.3);
            let note_left = ((lane_left + pad).max(area.x as f64)) as u16;
            let note_right = ((lane_right - pad).min((area.x + area.width - 1) as f64)) as u16;

            if note_left >= note_right {
                continue;
            }

            let brightness = (z_at_row * 0.65 + 0.35).min(1.0);
            let glow = if z_at_row > 0.85 {
                (z_at_row - 0.85) / 0.15
            } else {
                0.0
            };

            let r = ((base_r as f64 * brightness) + (255.0 - base_r as f64) * glow * 0.3) as u8;
            let g = ((base_g as f64 * brightness) + (255.0 - base_g as f64) * glow * 0.3) as u8;
            let b = ((base_b as f64 * brightness) + (255.0 - base_b as f64) * glow * 0.3) as u8;

            for col in note_left..=note_right {
                let is_edge_row = row == row_top || row == row_bot;
                let is_edge_col = col == note_left || col == note_right;

                let ch = if is_edge_row || is_edge_col { '▓' } else { '█' };
                let edge_dim = if is_edge_row || is_edge_col { 0.75 } else { 1.0 };
                let er = (r as f64 * edge_dim) as u8;
                let eg = (g as f64 * edge_dim) as u8;
                let eb = (b as f64 * edge_dim) as u8;

                if let Some(c) = buf.cell_mut((col, row)) {
                    c.set_char(ch)
                        .set_fg(Color::Rgb(er, eg, eb))
                        .set_bg(Color::Rgb(er / 5, eg / 5, eb / 5));
                }
            }
        }

        // Draw the key letter on the note center (only when close enough to be readable)
        if z > 0.25 {
            let row_mid = (row_top + row_bot) / 2;
            let z_mid = screen_y_to_z(row_mid as f64, vanish_y, hit_y);
            let hw_mid = max_hw * z_mid;
            let lane_w_mid = (2.0 * hw_mid) / num_lanes as f64;
            let lane_left_mid = center_x - hw_mid + lane_w_mid * note.lane as f64;
            let col_mid = (lane_left_mid + lane_w_mid / 2.0) as u16;

            if col_mid >= area.x
                && col_mid < area.x + area.width
                && row_mid >= area.y
                && row_mid < area.y + area.height
            {
                let key_char = note.key.to_ascii_uppercase();
                let dim = ((z * 0.7 + 0.3) * 255.0) as u8;
                if let Some(c) = buf.cell_mut((col_mid, row_mid)) {
                    c.set_char(key_char)
                        .set_fg(Color::Rgb(255, 255, dim))
                        .set_bg(Color::Rgb(0, 0, 0));
                }
            }
        }
    }
}

fn draw_hit_zone(
    buf: &mut Buffer,
    area: Rect,
    _vanish_y: f64,
    hit_y: f64,
    center_x: f64,
    max_hw: f64,
    game: &Game,
) {
    let row = hit_y as u16;
    if row >= area.y + area.height {
        return;
    }

    let left = ((center_x - max_hw).max(area.x as f64)) as u16;
    let right = ((center_x + max_hw).min((area.x + area.width - 1) as f64)) as u16;

    // Main hit line
    for col in left..=right {
        if let Some(c) = buf.cell_mut((col, row)) {
            c.set_char('═')
                .set_fg(Color::Rgb(100, 100, 130))
                .set_bg(Color::Rgb(15, 12, 25));
        }
    }

    let num_lanes = game.num_lanes();
    let lane_w = (2.0 * max_hw) / num_lanes as f64;

    for lane in 0..num_lanes {
        let lane_center = center_x - max_hw + lane_w * (lane as f64 + 0.5);
        let (r, g, b) = game.lane_color(lane);

        let flash = game.lane_flash[lane]
            .map(|t| {
                let elapsed = t.elapsed().as_secs_f64();
                if elapsed < 0.12 { 1.0 - elapsed / 0.12 } else { 0.0 }
            })
            .unwrap_or(0.0);

        let base_bright = 0.4;
        let bright = base_bright + flash * (1.0 - base_bright);
        let cr = (r as f64 * bright) as u8;
        let cg = (g as f64 * bright) as u8;
        let cb = (b as f64 * bright) as u8;

        let target_left =
            (center_x - max_hw + lane_w * lane as f64 + lane_w * 0.15) as u16;
        let target_right =
            (center_x - max_hw + lane_w * (lane as f64 + 1.0) - lane_w * 0.15) as u16;

        for col in target_left..=target_right {
            if col >= area.x && col < area.x + area.width {
                if let Some(c) = buf.cell_mut((col, row)) {
                    c.set_char('▬')
                        .set_fg(Color::Rgb(cr, cg, cb))
                        .set_bg(Color::Rgb(cr / 6, cg / 6, cb / 6));
                }
            }
        }

        // Show key for this lane centered on the target
        let key_col = lane_center as u16;
        if key_col >= area.x && key_col < area.x + area.width {
            let key_char = game.display_key_for_lane(lane).to_ascii_uppercase();
            let (kr, kg, kb) = if flash > 0.0 {
                (255u8, 255u8, 255u8)
            } else {
                (
                    cr.saturating_add(60),
                    cg.saturating_add(60),
                    cb.saturating_add(60),
                )
            };
            if let Some(c) = buf.cell_mut((key_col, row)) {
                c.set_char(key_char)
                    .set_fg(Color::Rgb(kr, kg, kb))
                    .set_bg(Color::Rgb(cr / 8, cg / 8, cb / 8));
            }
        }

        // Lane flash column effect (brief vertical glow on keypress)
        if flash > 0.3 {
            let glow_alpha = ((flash - 0.3) / 0.7).min(1.0);
            let glow_height = (glow_alpha * 8.0) as u16;
            for dy in 1..=glow_height {
                let gy = row.saturating_sub(dy);
                if gy <= area.y {
                    break;
                }
                let fade = 1.0 - (dy as f64 / glow_height as f64);
                let alpha = glow_alpha * fade * 0.25;
                let gr = (r as f64 * alpha) as u8;
                let gg = (g as f64 * alpha) as u8;
                let gb = (b as f64 * alpha) as u8;
                let x = lane_center as u16;
                if x >= area.x && x < area.x + area.width {
                    if let Some(cell) = buf.cell_mut((x, gy)) {
                        cell.set_fg(Color::Rgb(
                            gr.saturating_add(40),
                            gg.saturating_add(40),
                            gb.saturating_add(40),
                        ));
                    }
                }
            }
        }
    }
}

fn draw_hud(buf: &mut Buffer, area: Rect, game: &Game) {
    let right_col = area.width.saturating_sub(20);

    // Difficulty indicator
    let (dr, dg, db) = game.difficulty.color();
    draw_str(
        buf,
        right_col,
        0,
        game.difficulty.label(),
        Style::default().fg(Color::Rgb(dr, dg, db)),
    );

    // Score
    let score_str = format!("SCORE {:>8}", game.score);
    draw_str(
        buf,
        right_col,
        1,
        &score_str,
        Style::default().fg(Color::White),
    );

    // Combo
    if game.combo > 1 {
        let combo_str = format!("{}x COMBO", game.combo);
        let combo_color = if game.combo >= 50 {
            Color::Rgb(255, 200, 50)
        } else if game.combo >= 20 {
            Color::Rgb(200, 120, 255)
        } else {
            Color::Rgb(120, 200, 255)
        };
        draw_str(buf, right_col, 2, &combo_str, Style::default().fg(combo_color));
    }

    // Accuracy
    let acc_str = format!("{:.1}%", game.accuracy());
    draw_str(
        buf,
        right_col,
        3,
        &acc_str,
        Style::default().fg(Color::Rgb(180, 180, 200)),
    );

    // Hit grade feedback
    if let Some((grade, instant)) = &game.last_hit {
        let elapsed = instant.elapsed().as_secs_f64();
        if elapsed < 0.6 {
            let alpha = 1.0 - (elapsed / 0.6);
            let (r, g, b) = grade.color();
            let fr = (r as f64 * alpha) as u8;
            let fg = (g as f64 * alpha) as u8;
            let fb = (b as f64 * alpha) as u8;

            let label = grade.label();
            let cx = area.width / 2;
            let label_x = cx.saturating_sub(label.len() as u16 / 2);
            let label_y = (area.height / 2).saturating_sub(1);
            draw_str(
                buf,
                label_x,
                label_y,
                label,
                Style::default().fg(Color::Rgb(fr, fg, fb)),
            );
        }
    }

    // Song progress bar on the left
    let progress = game.song_progress();
    let bar_height = area.height.saturating_sub(6);
    let bar_x = 2u16;
    let bar_y_start = 2u16;

    draw_str(
        buf,
        bar_x,
        bar_y_start.saturating_sub(1),
        "♪",
        Style::default().fg(Color::Rgb(100, 80, 140)),
    );

    for i in 0..bar_height {
        let y = bar_y_start + i;
        let frac = 1.0 - (i as f64 / bar_height as f64);
        let filled = frac <= progress;
        let ch = if filled { '┃' } else { '╎' };
        let color = if filled {
            Color::Rgb(120, 80, 200)
        } else {
            Color::Rgb(40, 35, 55)
        };
        if y < area.height {
            draw_str(buf, bar_x, y, &ch.to_string(), Style::default().fg(color));
        }
    }

    // Song name at top
    let song = &game.songs[game.selected_song];
    let song_name = format!("♫ {} - {}", song.name, song.artist);
    draw_str(
        buf,
        5,
        0,
        &song_name,
        Style::default().fg(Color::Rgb(140, 120, 180)),
    );
}

// ── Results Screen ──────────────────────────────────────────────────────────

fn render_results(buf: &mut Buffer, area: Rect, game: &Game) {
    let cx = area.width / 2;
    let mut y = area.height / 6;

    let header = "── RESULTS ──";
    draw_str(
        buf,
        cx.saturating_sub(header.len() as u16 / 2),
        y,
        header,
        Style::default().fg(Color::Rgb(255, 200, 100)),
    );

    y += 2;

    let song = &game.songs[game.selected_song];
    let song_line = format!("{} — {}", song.name, song.artist);
    draw_str(
        buf,
        cx.saturating_sub(song_line.len() as u16 / 2),
        y,
        &song_line,
        Style::default().fg(Color::Rgb(180, 160, 220)),
    );

    y += 1;

    // Difficulty badge
    let (dr, dg, db) = game.difficulty.color();
    let diff_badge = format!("[{}]", game.difficulty.label());
    draw_str(
        buf,
        cx.saturating_sub(diff_badge.len() as u16 / 2),
        y,
        &diff_badge,
        Style::default().fg(Color::Rgb(dr, dg, db)),
    );

    y += 3;

    // Grade
    let grade = game.grade_letter();
    let grade_display = format!("  {}  ", grade);
    let grade_color = match grade {
        "S" => Color::Rgb(255, 220, 50),
        "A" => Color::Rgb(100, 255, 120),
        "B" => Color::Rgb(80, 180, 255),
        "C" => Color::Rgb(200, 160, 80),
        _ => Color::Rgb(160, 80, 80),
    };
    draw_str(
        buf,
        cx.saturating_sub(grade_display.len() as u16 / 2),
        y,
        &grade_display,
        Style::default().fg(grade_color),
    );

    y += 3;

    let x = cx.saturating_sub(18);
    let lines = [
        (format!("Score       {:>8}", game.score), Color::White),
        (
            format!("Max Combo   {:>8}x", game.max_combo),
            Color::Rgb(200, 180, 255),
        ),
        (
            format!("Accuracy    {:>7.1}%", game.accuracy()),
            Color::Rgb(180, 220, 255),
        ),
        (String::new(), Color::Black),
        (
            format!("Perfect     {:>8}", game.hit_counts[0]),
            Color::Rgb(255, 255, 80),
        ),
        (
            format!("Great       {:>8}", game.hit_counts[1]),
            Color::Rgb(80, 255, 120),
        ),
        (
            format!("Good        {:>8}", game.hit_counts[2]),
            Color::Rgb(80, 180, 255),
        ),
        (
            format!("Miss        {:>8}", game.hit_counts[3]),
            Color::Rgb(255, 60, 60),
        ),
    ];

    for (line, color) in &lines {
        if !line.is_empty() {
            draw_str(buf, x, y, line, Style::default().fg(*color));
        }
        y += 1;
    }

    y += 2;
    let prompt = "[ ENTER to continue ]";
    draw_str(
        buf,
        cx.saturating_sub(prompt.len() as u16 / 2),
        y,
        prompt,
        Style::default().fg(Color::Rgb(160, 160, 180)),
    );
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn draw_str(buf: &mut Buffer, x: u16, y: u16, text: &str, style: Style) {
    let mut col = x;
    for ch in text.chars() {
        if col >= buf.area().width || y >= buf.area().height {
            break;
        }
        if let Some(c) = buf.cell_mut((col, y)) {
            c.set_char(ch).set_style(style);
        }
        col += 1;
    }
}
