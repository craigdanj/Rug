//! Next-wave DSP & studio analysis widgets: [`compressor_curve`], [`goniometer`],
//! [`wavetable_view`], [`chromatic_tuner`], and [`MeterBallistics`].

use crate::theme::{AudioTheme, Skin};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

// ---------------- 1. METER BALLISTICS SMOOTHER ----------------

/// Zero-allocation attack/release meter smoother with peak-hold decay.
///
/// Feed raw instantaneous amplitude (`0.0..=1.0`) each UI frame via [`MeterBallistics::update`]
/// to drive [`crate::level_meter`], [`crate::vu_meter`], or [`crate::gain_reduction_meter`]
/// with smooth studio ballistics.
#[derive(Clone, Copy, Debug)]
pub struct MeterBallistics {
    /// Current smoothed level (`0.0..=1.0`).
    pub level: f32,
    /// Current peak-hold marker (`0.0..=1.0`).
    pub peak_hold: f32,
    /// Remaining hold time in seconds before `peak_hold` begins decaying.
    hold_timer: f32,
    /// Attack time constant in seconds (smaller = faster rise).
    pub attack_sec: f32,
    /// Release time constant in seconds (larger = slower fall).
    pub release_sec: f32,
    /// Peak hold duration in seconds before fallback decay starts.
    pub hold_sec: f32,
    /// Peak hold decay rate in normalized units per second.
    pub peak_decay_per_sec: f32,
}

impl Default for MeterBallistics {
    fn default() -> Self {
        Self {
            level: 0.0,
            peak_hold: 0.0,
            hold_timer: 0.0,
            attack_sec: 0.015,
            release_sec: 0.28,
            hold_sec: 1.2,
            peak_decay_per_sec: 0.35,
        }
    }
}

impl MeterBallistics {
    /// Advances the smoother by `dt` seconds toward `input_norm` (`0.0..=1.0`).
    pub fn update(&mut self, input_norm: f32, dt: f32) {
        let target = input_norm.clamp(0.0, 1.0);
        let dt = dt.max(0.0);
        let tau = if target > self.level {
            self.attack_sec.max(1e-4)
        } else {
            self.release_sec.max(1e-4)
        };
        let coeff = 1.0 - (-dt / tau).exp();
        self.level = (self.level + (target - self.level) * coeff).clamp(0.0, 1.0);

        if target >= self.peak_hold {
            self.peak_hold = target;
            self.hold_timer = self.hold_sec;
        } else if self.hold_timer > 0.0 {
            self.hold_timer = (self.hold_timer - dt).max(0.0);
        } else {
            self.peak_hold = (self.peak_hold - self.peak_decay_per_sec * dt).max(self.level);
        }
    }

    /// Resets smoothed level and peak hold to zero.
    pub fn reset(&mut self) {
        self.level = 0.0;
        self.peak_hold = 0.0;
        self.hold_timer = 0.0;
    }
}

// ---------------- 2. COMPRESSOR TRANSFER CURVE ----------------

/// Interactive state for [`compressor_curve`].
#[derive(Clone, Copy, Debug)]
pub struct CompressorState {
    /// Compression threshold in dB (`-60.0..=0.0`).
    pub threshold_db: f32,
    /// Compression ratio (`1.0..=20.0`, where `1.0` is bypass and `20.0` is limiting).
    pub ratio: f32,
    /// Soft knee width in dB (`0.0..=24.0`).
    pub knee_db: f32,
    /// Post-compression makeup gain in dB (`0.0..=24.0`).
    pub makeup_db: f32,
}

impl Default for CompressorState {
    fn default() -> Self {
        Self {
            threshold_db: -18.0,
            ratio: 4.0,
            knee_db: 6.0,
            makeup_db: 2.0,
        }
    }
}

impl CompressorState {
    /// Computes the compressor output level in dB for a given `input_db` (`-60.0..=0.0`).
    pub fn transfer_db(&self, input_db: f32) -> f32 {
        let t = self.threshold_db;
        let r = self.ratio.max(1.0);
        let w = self.knee_db.max(0.0);
        let slope = 1.0 - 1.0 / r;

        let gr = if w <= 0.01 {
            if input_db <= t {
                0.0
            } else {
                slope * (input_db - t)
            }
        } else if 2.0 * (input_db - t) < -w {
            0.0
        } else if 2.0 * (input_db - t) <= w {
            let x = input_db - t + w * 0.5;
            slope * (x * x) / (2.0 * w)
        } else {
            slope * (input_db - t)
        };

        (input_db - gr + self.makeup_db).clamp(-60.0, 0.0)
    }

    /// Computes instantaneous gain reduction in dB (`>= 0.0`) for `input_db`.
    pub fn gain_reduction_db(&self, input_db: f32) -> f32 {
        let out_no_makeup = self.transfer_db(input_db) - self.makeup_db;
        (input_db - out_no_makeup).max(0.0)
    }
}

/// Interactive dynamics transfer-curve plot (`-60..0 dB`) with live input ball.
///
/// - **Drag horizontally/vertically:** adjusts `threshold_db` and `ratio`.
/// - **Scroll wheel:** adjusts soft `knee_db`.
/// - **Double-click:** resets to default studio settings.
pub fn compressor_curve(
    ui: &mut Ui,
    state: &mut CompressorState,
    input_db: f32,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let pad = 8.0;
    let inner = rect.shrink(pad);

    if response.double_clicked() {
        *state = CompressorState::default();
        response.mark_changed();
    } else if response.dragged() {
        let d = response.drag_delta();
        state.threshold_db = (state.threshold_db + d.x * 0.25).clamp(-54.0, -2.0);
        state.ratio = (state.ratio - d.y * 0.08).clamp(1.0, 20.0);
        response.mark_changed();
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            state.knee_db = (state.knee_db + scroll.signum() * 1.5).clamp(0.0, 24.0);
            response.mark_changed();
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 5.0);

        let db_to_x = |db: f32| inner.min.x + ((db + 60.0) / 60.0).clamp(0.0, 1.0) * inner.width();
        let db_to_y = |db: f32| inner.max.y - ((db + 60.0) / 60.0).clamp(0.0, 1.0) * inner.height();

        // Subtle 15 dB grid + 1:1 unity reference diagonal
        let grid_col = AudioTheme::with_alpha(theme.palette.border, 55);
        for db in [-45.0, -30.0, -15.0] {
            let x = db_to_x(db);
            let y = db_to_y(db);
            painter.line_segment([Pos2::new(x, inner.min.y), Pos2::new(x, inner.max.y)], Stroke::new(1.0, grid_col));
            painter.line_segment([Pos2::new(inner.min.x, y), Pos2::new(inner.max.x, y)], Stroke::new(1.0, grid_col));
        }
        painter.line_segment(
            [inner.left_bottom(), inner.right_top()],
            Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.text_muted, 55)),
        );

        // Threshold guide
        let tx = db_to_x(state.threshold_db);
        painter.line_segment(
            [Pos2::new(tx, inner.min.y), Pos2::new(tx, inner.max.y)],
            Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.accent_secondary, 90)),
        );

        // Transfer curve + translucent gradient fill
        let steps = 64;
        let pts: Vec<Pos2> = (0..=steps)
            .map(|i| {
                let db = -60.0 + (i as f32 / steps as f32) * 60.0;
                Pos2::new(db_to_x(db), db_to_y(state.transfer_db(db)))
            })
            .collect();

        if matches!(theme.skin, Skin::Hybrid | Skin::Skeuomorphic) {
            let mut mesh = egui::Mesh::default();
            let top_col = AudioTheme::with_alpha(theme.palette.accent, if theme.skin == Skin::Skeuomorphic { 85 } else { 55 });
            let bot_col = AudioTheme::with_alpha(theme.palette.accent, 6);
            for (i, &p) in pts.iter().enumerate() {
                mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: egui::epaint::WHITE_UV, color: top_col });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p.x, inner.max.y), uv: egui::epaint::WHITE_UV, color: bot_col });
                if i > 0 {
                    let b = ((i - 1) * 2) as u32;
                    mesh.add_triangle(b, b + 1, b + 2);
                    mesh.add_triangle(b + 1, b + 3, b + 2);
                }
            }
            painter.add(Shape::mesh(mesh));
            painter.add(Shape::line(pts.clone(), Stroke::new(4.2, AudioTheme::with_alpha(theme.palette.accent, 38))));
        }
        painter.add(Shape::line(pts, Stroke::new(2.0, theme.palette.accent)));

        // Threshold handle dot
        let th_pos = Pos2::new(tx, db_to_y(state.transfer_db(state.threshold_db)));
        painter.circle_filled(th_pos, 4.0, theme.palette.accent_secondary);
        painter.circle_stroke(th_pos, 4.0, Stroke::new(1.2, theme.palette.bg_base));

        // Live input ball + gain-reduction drop line
        let in_clamped = input_db.clamp(-60.0, 0.0);
        let out_db = state.transfer_db(in_clamped);
        let ball = Pos2::new(db_to_x(in_clamped), db_to_y(out_db));
        let unity_pt = Pos2::new(ball.x, db_to_y(in_clamped));
        if state.gain_reduction_db(in_clamped) > 0.2 {
            painter.line_segment([unity_pt, ball], Stroke::new(1.5, theme.palette.danger));
        }
        theme.paint_glow_circle(painter, ball, 5.0, theme.palette.accent);
        painter.circle_filled(ball, 4.5, theme.palette.text_primary);

        // Compact HUD readout
        painter.text(
            Pos2::new(rect.min.x + 6.0, rect.min.y + 8.0),
            Align2::LEFT_CENTER,
            format!("TH {:.0}dB  {:.1}:1  KN {:.0}", state.threshold_db, state.ratio, state.knee_db),
            FontId::monospace(8.5),
            theme.palette.text_muted,
        );
        theme.paint_screen_overlay(painter, rect);
    }

    response.on_hover_text("Drag: threshold & ratio | Scroll: knee | Double-click: reset")
}

// ---------------- 3. GONIOMETER / STEREO VECTORSCOPE ----------------

/// Real-time stereo Lissajous vectorscope / goniometer.
///
/// Pass paired `(left, right)` audio samples in `-1.0..=1.0`. The display rotates
/// `(L, R)` by 45° so mono content aligns vertically (Mid) and out-of-phase content
/// spreads horizontally (Side).
pub fn goniometer(ui: &mut Ui, stereo_samples: &[(f32, f32)], size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    theme.paint_recessed_rect(painter, rect, 5.0);

    let center = rect.center();
    let r = (rect.width().min(rect.height()) * 0.44).max(8.0);
    let grid_col = AudioTheme::with_alpha(theme.palette.border, 70);

    // Concentric rings + M/S and L/R diagonal axes
    painter.circle_stroke(center, r, Stroke::new(1.0, grid_col));
    painter.circle_stroke(center, r * 0.5, Stroke::new(0.8, AudioTheme::with_alpha(theme.palette.border, 45)));
    painter.line_segment([center - Vec2::new(0.0, r), center + Vec2::new(0.0, r)], Stroke::new(0.8, grid_col));
    painter.line_segment([center - Vec2::new(r, 0.0), center + Vec2::new(r, 0.0)], Stroke::new(0.8, grid_col));
    let diag = r * std::f32::consts::FRAC_1_SQRT_2;
    painter.line_segment(
        [center - Vec2::new(diag, diag), center + Vec2::new(diag, diag)],
        Stroke::new(0.8, AudioTheme::with_alpha(theme.palette.border, 55)),
    );
    painter.line_segment(
        [center + Vec2::new(-diag, diag), center + Vec2::new(diag, -diag)],
        Stroke::new(0.8, AudioTheme::with_alpha(theme.palette.border, 55)),
    );

    painter.text(center + Vec2::new(-diag - 6.0, -diag - 2.0), Align2::CENTER_CENTER, "L", FontId::monospace(8.5), theme.palette.text_muted);
    painter.text(center + Vec2::new(diag + 6.0, -diag - 2.0), Align2::CENTER_CENTER, "R", FontId::monospace(8.5), theme.palette.text_muted);

    if stereo_samples.len() >= 2 {
        let count = stereo_samples.len().min(180);
        let pts: Vec<Pos2> = stereo_samples[..count]
            .iter()
            .map(|&(l, ri)| {
                let mid = (l + ri) * 0.5;
                let side = (ri - l) * 0.5;
                Pos2::new(center.x + side.clamp(-1.0, 1.0) * r, center.y - mid.clamp(-1.0, 1.0) * r)
            })
            .collect();

        if matches!(theme.skin, Skin::Hybrid | Skin::Skeuomorphic) {
            painter.add(Shape::line(pts.clone(), Stroke::new(4.0, AudioTheme::with_alpha(theme.palette.accent, 34))));
        }
        painter.add(Shape::line(pts, Stroke::new(1.5, theme.palette.accent)));
    }
    theme.paint_screen_overlay(painter, rect);
}

// ---------------- 4. 3D WAVETABLE MORPH VISUALIZER ----------------

/// Stacked 3D-perspective wavetable visualizer with interactive vertical morph scrubbing.
///
/// `frames` is a slice of wave slices (each sample in `-1.0..=1.0`), and `morph_pos`
/// (`0.0..=1.0`) highlights and interpolates the active table position. Drag vertically
/// or scroll to scrub `morph_pos`.
pub fn wavetable_view(
    ui: &mut Ui,
    frames: &[Vec<f32>],
    morph_pos: &mut f32,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());

    if response.dragged() {
        *morph_pos = (*morph_pos - response.drag_delta().y * 0.008).clamp(0.0, 1.0);
        response.mark_changed();
    }
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            *morph_pos = (*morph_pos + scroll.signum() * 0.04).clamp(0.0, 1.0);
            response.mark_changed();
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 5.0);

        let n = frames.len();
        if n > 0 {
            let active_f = (*morph_pos * (n - 1) as f32).clamp(0.0, (n - 1) as f32);
            let active_idx = active_f.round() as usize;

            for (i, wave) in frames.iter().enumerate().rev() {
                if wave.len() < 2 {
                    continue;
                }
                let depth = i as f32 / (n - 1).max(1) as f32;
                let y_base = rect.max.y - 16.0 - depth * (rect.height() * 0.56);
                let x_inset = 10.0 + depth * 18.0;
                let w = (rect.width() - x_inset * 2.0).max(12.0);
                let amp = (rect.height() * 0.16) * (1.0 - depth * 0.25);

                let pts: Vec<Pos2> = wave
                    .iter()
                    .enumerate()
                    .map(|(j, &s)| {
                        let t = j as f32 / (wave.len() - 1) as f32;
                        Pos2::new(rect.min.x + x_inset + t * w, y_base - s.clamp(-1.0, 1.0) * amp)
                    })
                    .collect();

                let is_active = i == active_idx;
                if is_active {
                    if matches!(theme.skin, Skin::Hybrid | Skin::Skeuomorphic) {
                        painter.add(Shape::line(pts.clone(), Stroke::new(4.8, AudioTheme::with_alpha(theme.palette.accent, 55))));
                    }
                    painter.add(Shape::line(pts, Stroke::new(2.0, theme.palette.accent)));
                } else {
                    let alpha = (40.0 + (1.0 - depth) * 75.0) as u8;
                    let col = AudioTheme::with_alpha(
                        AudioTheme::lerp_color(theme.palette.accent_secondary, theme.palette.text_muted, 0.5),
                        alpha,
                    );
                    painter.add(Shape::line(pts, Stroke::new(1.1, col)));
                }
            }

            painter.text(
                Pos2::new(rect.min.x + 6.0, rect.min.y + 9.0),
                Align2::LEFT_CENTER,
                format!("WT POS {:>3.0}%", *morph_pos * 100.0),
                FontId::monospace(9.0),
                theme.palette.text_muted,
            );
        }
        theme.paint_screen_overlay(painter, rect);
    }

    response.on_hover_text("Drag vertically or scroll to morph wavetable")
}

// ---------------- 5. CHROMATIC PITCH TUNER ----------------

/// Converts a frequency in Hz (relative to `A4 = 440.0 Hz`) into `(note_name, octave, cents)`.
pub fn frequency_to_note(freq_hz: f32) -> (&'static str, i32, f32) {
    const NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
    if !freq_hz.is_finite() || freq_hz <= 0.0 {
        return ("--", 0, 0.0);
    }
    let midi_f = 69.0 + 12.0 * (freq_hz / 440.0).log2();
    let midi_round = midi_f.round() as i32;
    let cents = ((midi_f - midi_round as f32) * 100.0).clamp(-50.0, 50.0);
    let note_idx = midi_round.rem_euclid(12) as usize;
    let octave = midi_round.div_euclid(12) - 1;
    (NAMES[note_idx], octave, cents)
}

/// Studio chromatic pitch tuner displaying note name, octave, cent deviation (`-50..+50 ct`),
/// and an illuminated in-tune lock indicator (`±5 ct`).
pub fn chromatic_tuner(ui: &mut Ui, freq_hz: f32, size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    theme.paint_recessed_rect(painter, rect, 5.0);

    let (note, octave, cents) = frequency_to_note(freq_hz);
    let in_tune = freq_hz > 0.0 && cents.abs() <= 5.0;
    let active_col = if in_tune {
        theme.palette.meter_low
    } else if cents.abs() <= 15.0 {
        theme.palette.meter_mid
    } else {
        theme.palette.accent
    };

    // Note + Hz header
    let note_str = if freq_hz > 0.0 { format!("{}{}", note, octave) } else { "--".to_string() };
    painter.text(
        Pos2::new(rect.center().x, rect.min.y + rect.height() * 0.34),
        Align2::CENTER_CENTER,
        note_str,
        FontId::monospace(18.0),
        active_col,
    );
    painter.text(
        Pos2::new(rect.min.x + 8.0, rect.min.y + 11.0),
        Align2::LEFT_CENTER,
        if freq_hz > 0.0 { format!("{:.1} Hz", freq_hz) } else { "NO SIG".to_string() },
        FontId::monospace(8.5),
        theme.palette.text_muted,
    );
    painter.text(
        Pos2::new(rect.max.x - 8.0, rect.min.y + 11.0),
        Align2::RIGHT_CENTER,
        format!("{:+.0} ct", cents),
        FontId::monospace(8.5),
        active_col,
    );

    // Cent meter bar (-50..+50)
    let bar = Rect::from_min_max(
        Pos2::new(rect.min.x + 12.0, rect.max.y - 18.0),
        Pos2::new(rect.max.x - 12.0, rect.max.y - 8.0),
    );
    painter.rect_filled(bar, 2.0, AudioTheme::darken(theme.palette.bg_recessed, 0.35));

    let cx = bar.center().x;
    // Center in-tune lock zone
    let lock_w = bar.width() * 0.1;
    painter.rect_filled(
        Rect::from_center_size(bar.center(), Vec2::new(lock_w, bar.height())),
        1.0,
        AudioTheme::with_alpha(theme.palette.meter_low, if in_tune { 90 } else { 32 }),
    );
    painter.line_segment([Pos2::new(cx, bar.min.y - 2.0), Pos2::new(cx, bar.max.y + 2.0)], Stroke::new(1.2, theme.palette.text_muted));

    if freq_hz > 0.0 {
        let px = cx + (cents / 50.0).clamp(-1.0, 1.0) * (bar.width() * 0.48);
        let needle = Pos2::new(px, bar.center().y);
        theme.paint_glow_circle(painter, needle, 4.0, active_col);
        painter.rect_filled(Rect::from_center_size(needle, Vec2::new(4.0, bar.height() + 2.0)), 1.5, active_col);
    }

    theme.paint_screen_overlay(painter, rect);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ballistics_rises_and_holds_peak() {
        let mut m = MeterBallistics::default();
        m.update(0.9, 0.05);
        assert!(m.level > 0.5);
        assert!((m.peak_hold - 0.9).abs() < 1e-5);
        m.update(0.0, 0.2);
        assert!((m.peak_hold - 0.9).abs() < 1e-5, "peak should hold before hold_sec elapses");
        m.update(0.0, 2.0);
        assert!(m.peak_hold < 0.9, "peak should decay after hold_sec");
    }

    #[test]
    fn compressor_reduces_above_threshold() {
        let c = CompressorState {
            threshold_db: -20.0,
            ratio: 4.0,
            knee_db: 0.0,
            makeup_db: 0.0,
        };
        assert_eq!(c.gain_reduction_db(-30.0), 0.0);
        assert!((c.gain_reduction_db(0.0) - 15.0).abs() < 1e-3);
        assert!((c.transfer_db(0.0) - (-15.0)).abs() < 1e-3);
    }

    #[test]
    fn tuner_identifies_a4_and_c4() {
        let (n, oct, ct) = frequency_to_note(440.0);
        assert_eq!((n, oct), ("A", 4));
        assert!(ct.abs() < 1e-3);

        let (n2, oct2, ct2) = frequency_to_note(261.6256);
        assert_eq!((n2, oct2), ("C", 4));
        assert!(ct2.abs() < 0.2);
    }
}
