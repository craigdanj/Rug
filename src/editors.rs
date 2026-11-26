use crate::theme::AudioTheme;
use egui::{Color32, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

// ---------------- 1. WAVEFORM VIEWER / EDITOR ----------------

pub struct WaveformState {
    pub loop_start: f32,
    pub loop_end: f32,
    pub playhead: f32,
}

impl Default for WaveformState {
    fn default() -> Self {
        Self { loop_start: 0.15, loop_end: 0.85, playhead: 0.35 }
    }
}

pub fn waveform_editor(
    ui: &mut Ui,
    samples: &[f32],
    state: &mut WaveformState,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let drag_id = response.id.with("wf_drag");

    if response.drag_started() {
        if let Some(p) = response.interact_pointer_pos() {
            let t = ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let mode = if (t - state.loop_start).abs() < 0.04 {
                1u8
            } else if (t - state.loop_end).abs() < 0.04 {
                2u8
            } else {
                0u8
            };
            ui.data_mut(|d| d.insert_temp(drag_id, mode));
        }
    }
    if response.dragged() || response.clicked() {
        if let Some(p) = response.interact_pointer_pos() {
            let t = ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            let mode = ui.data_mut(|d| d.get_temp::<u8>(drag_id).unwrap_or(0));
            match mode {
                1 => state.loop_start = t.min(state.loop_end - 0.05),
                2 => state.loop_end = t.max(state.loop_start + 0.05),
                _ => state.playhead = t,
            }
            response.mark_changed();
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);

        let lx0 = rect.min.x + state.loop_start * rect.width();
        let lx1 = rect.min.x + state.loop_end * rect.width();
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(lx0, rect.min.y), Pos2::new(lx1, rect.max.y)),
            0.0,
            AudioTheme::with_alpha(theme.palette.accent, 28),
        );

        let cols = (rect.width() as usize).max(1);
        if !samples.is_empty() {
            let spp = (samples.len() / cols).max(1);
            for c in 0..cols {
                let start = (c * samples.len()) / cols;
                let end = (start + spp).min(samples.len());
                let (mut mn, mut mx) = (0.0f32, 0.0f32);
                for &s in &samples[start..end] {
                    mn = mn.min(s);
                    mx = mx.max(s);
                }
                let x = rect.min.x + c as f32;
                let y0 = rect.center().y - mx.clamp(-1.0, 1.0) * rect.height() * 0.45;
                let y1 = rect.center().y - mn.clamp(-1.0, 1.0) * rect.height() * 0.45;
                painter.line_segment([Pos2::new(x, y0), Pos2::new(x, y1)], Stroke::new(1.0, theme.palette.accent));
            }
        }

        for lx in [lx0, lx1] {
            painter.line_segment([Pos2::new(lx, rect.min.y), Pos2::new(lx, rect.max.y)], Stroke::new(1.5, theme.palette.accent_secondary));
        }
        let px = rect.min.x + state.playhead * rect.width();
        painter.line_segment([Pos2::new(px, rect.min.y), Pos2::new(px, rect.max.y)], Stroke::new(1.5, theme.palette.text_primary));
        theme.paint_screen_overlay(painter, rect);
    }
    response
}

// ---------------- 2. ENVELOPE EDITOR (ADSR) ----------------

#[derive(Clone, Copy, Debug)]
pub struct AdsrState {
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
}

impl Default for AdsrState {
    fn default() -> Self {
        Self { attack: 0.2, decay: 0.3, sustain: 0.65, release: 0.35 }
    }
}

pub fn envelope_editor(ui: &mut Ui, adsr: &mut AdsrState, size: Vec2, theme: &AudioTheme) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let pad = 10.0;
    let inner = rect.shrink(pad);

    let p0 = inner.left_bottom();
    let p_a = Pos2::new(inner.min.x + adsr.attack * inner.width() * 0.3, inner.min.y);
    let p_d = Pos2::new(p_a.x + adsr.decay * inner.width() * 0.3, inner.max.y - adsr.sustain * inner.height());
    let p_s = Pos2::new(inner.max.x - adsr.release * inner.width() * 0.35, p_d.y);
    let p_r = inner.right_bottom();

    let active_id = response.id.with("adsr_node");
    if response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            let nodes = [p_a, p_d, p_s];
            let best = nodes
                .iter()
                .enumerate()
                .min_by(|a, b| a.1.distance(pos).total_cmp(&b.1.distance(pos)))
                .filter(|(_, p)| p.distance(pos) < 18.0)
                .map(|(i, _)| i as u8);
            ui.data_mut(|d| d.insert_temp(active_id, best));
        }
    }
    if response.dragged() {
        if let (Some(pos), Some(node)) = (
            response.interact_pointer_pos(),
            ui.data_mut(|d| d.get_temp::<Option<u8>>(active_id).flatten()),
        ) {
            match node {
                0 => adsr.attack = ((pos.x - inner.min.x) / (inner.width() * 0.3)).clamp(0.02, 1.0),
                1 => {
                    adsr.decay = ((pos.x - p_a.x) / (inner.width() * 0.3)).clamp(0.02, 1.0);
                    adsr.sustain = (1.0 - (pos.y - inner.min.y) / inner.height()).clamp(0.0, 1.0);
                }
                2 => {
                    adsr.release = ((inner.max.x - pos.x) / (inner.width() * 0.35)).clamp(0.02, 1.0);
                    adsr.sustain = (1.0 - (pos.y - inner.min.y) / inner.height()).clamp(0.0, 1.0);
                }
                _ => {}
            }
            response.mark_changed();
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);
        let pts = vec![p0, p_a, p_d, p_s, p_r];
        painter.add(Shape::line(pts, Stroke::new(2.0, theme.palette.accent_secondary)));
        for pt in [p_a, p_d, p_s] {
            theme.paint_glow_circle(painter, pt, 5.0, theme.palette.accent_secondary);
            painter.circle_filled(pt, 4.5, theme.palette.accent_secondary);
            painter.circle_stroke(pt, 4.5, Stroke::new(1.5, theme.palette.bg_base));
        }
        theme.paint_screen_overlay(painter, rect);
    }
    response
}

// ---------------- 3. EQ / FILTER CURVE EDITOR ----------------

#[derive(Clone, Copy, Debug)]
pub struct EqBand {
    pub freq_norm: f32,
    pub gain_norm: f32,
    pub q: f32,
}

pub fn eq_curve_editor(ui: &mut Ui, bands: &mut [EqBand], size: Vec2, theme: &AudioTheme) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let active_id = response.id.with("eq_band");

    if response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            let best = bands
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    let pa = Pos2::new(rect.min.x + a.freq_norm * rect.width(), rect.center().y - a.gain_norm * rect.height() * 0.42);
                    let pb = Pos2::new(rect.min.x + b.freq_norm * rect.width(), rect.center().y - b.gain_norm * rect.height() * 0.42);
                    pa.distance(pos).total_cmp(&pb.distance(pos))
                })
                .map(|(i, _)| i);
            ui.data_mut(|d| d.insert_temp(active_id, best));
        }
    }
    if response.dragged() {
        if let (Some(pos), Some(idx)) = (
            response.interact_pointer_pos(),
            ui.data_mut(|d| d.get_temp::<Option<usize>>(active_id).flatten()),
        ) {
            if let Some(b) = bands.get_mut(idx) {
                b.freq_norm = ((pos.x - rect.min.x) / rect.width()).clamp(0.02, 0.98);
                b.gain_norm = ((rect.center().y - pos.y) / (rect.height() * 0.42)).clamp(-1.0, 1.0);
                response.mark_changed();
            }
        }
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                if let Some(b) = bands.iter_mut().min_by(|a, b| {
                    let pa = Pos2::new(rect.min.x + a.freq_norm * rect.width(), rect.center().y - a.gain_norm * rect.height() * 0.42);
                    let pb = Pos2::new(rect.min.x + b.freq_norm * rect.width(), rect.center().y - b.gain_norm * rect.height() * 0.42);
                    pa.distance(pos).total_cmp(&pb.distance(pos))
                }) {
                    b.q = (b.q + scroll.signum() * 0.25).clamp(0.3, 8.0);
                    response.mark_changed();
                }
            }
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);
        painter.line_segment(
            [Pos2::new(rect.min.x, rect.center().y), Pos2::new(rect.max.x, rect.center().y)],
            Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.border, 90)),
        );

        let steps = 96;
        let curve: Vec<Pos2> = (0..=steps)
            .map(|i| {
                let x_norm = i as f32 / steps as f32;
                let mut sum = 0.0f32;
                for b in bands.iter() {
                    let d = (x_norm - b.freq_norm) * b.q * 4.5;
                    sum += b.gain_norm * (-d * d).exp();
                }
                Pos2::new(
                    rect.min.x + x_norm * rect.width(),
                    rect.center().y - sum.clamp(-1.0, 1.0) * rect.height() * 0.42,
                )
            })
            .collect();
        painter.add(Shape::line(curve, Stroke::new(2.0, theme.palette.accent)));

        for b in bands.iter() {
            let p = Pos2::new(
                rect.min.x + b.freq_norm * rect.width(),
                rect.center().y - b.gain_norm * rect.height() * 0.42,
            );
            theme.paint_glow_circle(painter, p, 5.0, theme.palette.accent);
            painter.circle_filled(p, 5.0, theme.palette.accent);
            painter.circle_stroke(p, 5.0, Stroke::new(1.5, theme.palette.bg_base));
        }
        theme.paint_screen_overlay(painter, rect);
    }
    response
}

// ---------------- 4. STEP SEQUENCER ----------------

pub fn step_sequencer(
    ui: &mut Ui,
    steps: &mut [f32],
    playhead: usize,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let n = steps.len().max(1);
    let cell_w = rect.width() / n as f32;

    if response.clicked() || response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let idx = ((pos.x - rect.min.x) / cell_w).floor() as usize;
            if idx < steps.len() {
                let vel = (1.0 - (pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                steps[idx] = if vel < 0.12 { 0.0 } else { vel };
                response.mark_changed();
            }
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        for (i, &vel) in steps.iter().enumerate() {
            let cell = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * cell_w + 1.5, rect.min.y),
                Vec2::new(cell_w - 3.0, rect.height()),
            );
            theme.paint_recessed_rect(painter, cell, 3.0);
            if vel > 0.0 {
                let h = (cell.height() - 4.0) * vel;
                let fill = Rect::from_min_max(
                    Pos2::new(cell.min.x + 2.0, cell.max.y - 2.0 - h),
                    Pos2::new(cell.max.x - 2.0, cell.max.y - 2.0),
                );
                let col = if i % 4 == 0 { theme.palette.accent } else { theme.palette.accent_secondary };
                painter.rect_filled(fill, 2.0, col);
            }
            if i == playhead % n {
                painter.rect_stroke(cell, theme.rounding(3.0), Stroke::new(1.5, theme.palette.text_primary));
            }
        }
    }
    response
}

// ---------------- 5. MINI PIANO KEYBOARD ----------------

pub fn mini_keyboard(
    ui: &mut Ui,
    active_note: &mut Option<(u8, f32)>,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let white_notes = [0u8, 2, 4, 5, 7, 9, 11];
    let black_notes = [(1u8, 0.7), (3, 1.7), (6, 3.7), (8, 4.7), (10, 5.7)];
    let kw = rect.width() / 7.0;

    if response.dragged() || response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let vel = ((pos.y - rect.min.y) / rect.height()).clamp(0.1, 1.0);
            let mut hit = None;
            for &(n, offset) in &black_notes {
                let kr = Rect::from_min_size(Pos2::new(rect.min.x + offset * kw, rect.min.y), Vec2::new(kw * 0.6, rect.height() * 0.62));
                if kr.contains(pos) {
                    hit = Some((n, vel));
                    break;
                }
            }
            if hit.is_none() {
                let idx = ((pos.x - rect.min.x) / kw).floor().clamp(0.0, 6.0) as usize;
                hit = Some((white_notes[idx], vel));
            }
            *active_note = hit;
            response.mark_changed();
        }
    } else if response.drag_stopped() {
        *active_note = None;
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        for (i, &n) in white_notes.iter().enumerate() {
            let kr = Rect::from_min_size(Pos2::new(rect.min.x + i as f32 * kw, rect.min.y), Vec2::new(kw - 1.0, rect.height()));
            let is_active = active_note.map(|(a, _)| a == n).unwrap_or(false);
            let col = if is_active { theme.palette.accent } else { Color32::from_rgb(235, 238, 244) };
            theme.paint_raised_rect(painter, kr, col, 2.0);
        }
        for &(n, offset) in &black_notes {
            let kr = Rect::from_min_size(Pos2::new(rect.min.x + offset * kw, rect.min.y), Vec2::new(kw * 0.6, rect.height() * 0.62));
            let is_active = active_note.map(|(a, _)| a == n).unwrap_or(false);
            let col = if is_active { theme.palette.accent_secondary } else { Color32::from_rgb(24, 26, 30) };
            theme.paint_raised_rect(painter, kr, col, 2.0);
        }
    }
    response
}
