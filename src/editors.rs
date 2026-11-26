use crate::theme::AudioTheme;
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};

// ---------------- 1. WAVEFORM VIEWER / EDITOR ----------------

pub struct WaveformState {
    pub loop_start: f32,
    pub loop_end: f32,
    pub playhead: f32,
    pub zoom: f32,
    pub pan: f32,
}

impl Default for WaveformState {
    fn default() -> Self {
        Self {
            loop_start: 0.15,
            loop_end: 0.85,
            playhead: 0.35,
            zoom: 1.0,
            pan: 0.0,
        }
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
    state.zoom = state.zoom.clamp(1.0, 16.0);
    let view_span = 1.0 / state.zoom;
    state.pan = state.pan.clamp(0.0, (1.0 - view_span).max(0.0));

    let view_start = state.pan;
    let view_end = (state.pan + view_span).min(1.0);
    let x_to_norm = |x: f32| -> f32 {
        let local = ((x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
        (view_start + local * (view_end - view_start)).clamp(0.0, 1.0)
    };
    let norm_to_x = |t: f32| -> f32 {
        rect.min.x + ((t - view_start) / (view_end - view_start).max(1e-6)) * rect.width()
    };

    // Scroll wheel zoom anchored at cursor X
    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let anchor_local = ui
                .input(|i| i.pointer.hover_pos())
                .map(|p| ((p.x - rect.min.x) / rect.width()).clamp(0.0, 1.0))
                .unwrap_or(0.5);
            let anchor_norm = view_start + anchor_local * view_span;
            let factor = if scroll > 0.0 { 1.18 } else { 1.0 / 1.18 };
            state.zoom = (state.zoom * factor).clamp(1.0, 16.0);
            let new_span = 1.0 / state.zoom;
            state.pan = (anchor_norm - anchor_local * new_span).clamp(0.0, (1.0 - new_span).max(0.0));
            response.mark_changed();
        }
    }

    if response.double_clicked() {
        state.zoom = 1.0;
        state.pan = 0.0;
        response.mark_changed();
    }

    let shift_held = ui.input(|i| i.modifiers.shift);
    let middle_down = ui.input(|i| i.pointer.middle_down());
    if response.drag_started() {
        if let Some(p) = response.interact_pointer_pos() {
            let mode = if (shift_held || middle_down) && state.zoom > 1.01 {
                3u8 // pan
            } else {
                let t = x_to_norm(p.x);
                let tol = 0.04 * view_span;
                if (t - state.loop_start).abs() < tol {
                    1u8
                } else if (t - state.loop_end).abs() < tol {
                    2u8
                } else {
                    0u8
                }
            };
            ui.data_mut(|d| d.insert_temp(drag_id, mode));
        }
    }
    if response.dragged() || response.clicked() {
        let mode = ui.data_mut(|d| d.get_temp::<u8>(drag_id).unwrap_or(0));
        if mode == 3 && response.dragged() {
            let dx_norm = (response.drag_delta().x / rect.width()) * view_span;
            state.pan = (state.pan - dx_norm).clamp(0.0, (1.0 - view_span).max(0.0));
            response.mark_changed();
        } else if let Some(p) = response.interact_pointer_pos() {
            let t = x_to_norm(p.x);
            match mode {
                1 => state.loop_start = t.min(state.loop_end - 0.02),
                2 => state.loop_end = t.max(state.loop_start + 0.02),
                _ => state.playhead = t,
            }
            response.mark_changed();
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);

        let view_start = state.pan;
        let view_span = 1.0 / state.zoom;
        let view_end = (view_start + view_span).min(1.0);
        let to_x = |t: f32| rect.min.x + ((t - view_start) / (view_end - view_start).max(1e-6)) * rect.width();

        // Loop region highlight clipped to visible viewport
        let lx0 = to_x(state.loop_start).clamp(rect.min.x, rect.max.x);
        let lx1 = to_x(state.loop_end).clamp(rect.min.x, rect.max.x);
        if lx1 > lx0 {
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(lx0, rect.min.y), Pos2::new(lx1, rect.max.y)),
                0.0,
                AudioTheme::with_alpha(theme.palette.accent, 28),
            );
        }

        // Waveform peaks within zoomed range
        let cols = (rect.width() as usize).max(1);
        if !samples.is_empty() {
            let total = samples.len();
            let s0 = ((view_start * total as f32).floor() as usize).min(total - 1);
            let s1 = ((view_end * total as f32).ceil() as usize).clamp(s0 + 1, total);
            let slice = &samples[s0..s1];
            let spp = (slice.len() / cols).max(1);
            for c in 0..cols {
                let start = (c * slice.len()) / cols;
                let end = (start + spp).min(slice.len());
                let (mut mn, mut mx) = (0.0f32, 0.0f32);
                for &s in &slice[start..end] {
                    mn = mn.min(s);
                    mx = mx.max(s);
                }
                let x = rect.min.x + c as f32;
                let y0 = rect.center().y - mx.clamp(-1.0, 1.0) * rect.height() * 0.43;
                let y1 = rect.center().y - mn.clamp(-1.0, 1.0) * rect.height() * 0.43;
                painter.line_segment([Pos2::new(x, y0), Pos2::new(x, y1)], Stroke::new(1.0, theme.palette.accent));
            }
        }

        for t in [state.loop_start, state.loop_end] {
            if t >= view_start && t <= view_end {
                let lx = to_x(t);
                painter.line_segment([Pos2::new(lx, rect.min.y), Pos2::new(lx, rect.max.y)], Stroke::new(1.5, theme.palette.accent_secondary));
            }
        }
        if state.playhead >= view_start && state.playhead <= view_end {
            let px = to_x(state.playhead);
            painter.line_segment([Pos2::new(px, rect.min.y), Pos2::new(px, rect.max.y)], Stroke::new(1.5, theme.palette.text_primary));
        }

        // Zoom badge + bottom viewport indicator when zoomed in
        if state.zoom > 1.01 {
            painter.text(
                Pos2::new(rect.max.x - 6.0, rect.min.y + 9.0),
                Align2::RIGHT_CENTER,
                format!("{:.1}x", state.zoom),
                FontId::monospace(9.0),
                theme.palette.text_muted,
            );
            let bar_y = rect.max.y - 3.0;
            let vx0 = rect.min.x + view_start * rect.width();
            let vx1 = rect.min.x + view_end * rect.width();
            painter.line_segment(
                [Pos2::new(vx0, bar_y), Pos2::new(vx1, bar_y)],
                Stroke::new(2.0, AudioTheme::with_alpha(theme.palette.accent, 170)),
            );
        }
        theme.paint_screen_overlay(painter, rect);
    }
    response.on_hover_text("Scroll: zoom | Shift+drag: pan | Double-click: reset")
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
        if matches!(theme.skin, crate::theme::Skin::Hybrid | crate::theme::Skin::Skeuomorphic) {
            for i in 1..4 {
                let t = i as f32 / 4.0;
                let gx = rect.min.x + rect.width() * t;
                let gy = rect.min.y + rect.height() * t;
                let grid_col = AudioTheme::with_alpha(theme.palette.border, 45);
                painter.line_segment([Pos2::new(gx, rect.min.y), Pos2::new(gx, rect.max.y)], Stroke::new(1.0, grid_col));
                painter.line_segment([Pos2::new(rect.min.x, gy), Pos2::new(rect.max.x, gy)], Stroke::new(1.0, grid_col));
            }
        }
        let pts = vec![p0, p_a, p_d, p_s, p_r];
        if matches!(theme.skin, crate::theme::Skin::Hybrid | crate::theme::Skin::Skeuomorphic) {
            let mut mesh = egui::Mesh::default();
            let top_alpha = if theme.skin == crate::theme::Skin::Skeuomorphic { 105 } else { 70 };
            let top_col = AudioTheme::with_alpha(theme.palette.accent_secondary, top_alpha);
            let bot_col = AudioTheme::with_alpha(theme.palette.accent_secondary, 8);
            for (i, &p) in pts.iter().enumerate() {
                let t_y = ((p.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
                let top_c = AudioTheme::lerp_color(top_col, bot_col, t_y * 0.5);
                mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: egui::epaint::WHITE_UV, color: top_c });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p.x, inner.max.y), uv: egui::epaint::WHITE_UV, color: bot_col });
                if i > 0 {
                    let base = ((i - 1) * 2) as u32;
                    mesh.add_triangle(base, base + 1, base + 2);
                    mesh.add_triangle(base + 1, base + 3, base + 2);
                }
            }
            painter.add(Shape::mesh(mesh));
            painter.add(Shape::line(pts.clone(), Stroke::new(4.5, AudioTheme::with_alpha(theme.palette.accent_secondary, 40))));
        }
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EqFilterType {
    Peak,
    LowShelf,
    HighShelf,
    LowPass,
    HighPass,
}

impl EqFilterType {
    pub const ALL: [Self; 5] = [
        Self::Peak,
        Self::LowShelf,
        Self::HighShelf,
        Self::LowPass,
        Self::HighPass,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Peak => "PK",
            Self::LowShelf => "LS",
            Self::HighShelf => "HS",
            Self::LowPass => "LP",
            Self::HighPass => "HP",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Peak => "Peak",
            Self::LowShelf => "Low Shelf",
            Self::HighShelf => "High Shelf",
            Self::LowPass => "Low Pass",
            Self::HighPass => "High Pass",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EqBand {
    pub freq_norm: f32,
    pub gain_norm: f32,
    pub q: f32,
    pub filter_type: EqFilterType,
}

impl EqBand {
    pub fn response_at(&self, x_norm: f32) -> f32 {
        let diff = x_norm - self.freq_norm;
        match self.filter_type {
            EqFilterType::Peak => {
                let d = diff * self.q * 4.5;
                self.gain_norm * (-d * d).exp()
            }
            EqFilterType::LowShelf => {
                let slope = (self.q * 6.0).clamp(2.5, 16.0);
                let s = 1.0 / (1.0 + (diff * slope).exp());
                self.gain_norm * s
            }
            EqFilterType::HighShelf => {
                let slope = (self.q * 6.0).clamp(2.5, 16.0);
                let s = 1.0 / (1.0 + (-diff * slope).exp());
                self.gain_norm * s
            }
            EqFilterType::LowPass => {
                let roll = if diff > 0.0 { -(diff * (3.0 + self.q * 1.5)).min(1.2) } else { 0.0 };
                let res = ((self.q - 0.7).max(0.0) * 0.14) * (-(diff * self.q * 5.0).powi(2)).exp();
                roll + res
            }
            EqFilterType::HighPass => {
                let roll = if diff < 0.0 { (diff * (3.0 + self.q * 1.5)).max(-1.2) } else { 0.0 };
                let res = ((self.q - 0.7).max(0.0) * 0.14) * (-(diff * self.q * 5.0).powi(2)).exp();
                roll + res
            }
        }
    }
}

pub fn eq_curve_editor(ui: &mut Ui, bands: &mut Vec<EqBand>, size: Vec2, theme: &AudioTheme) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let active_id = response.id.with("eq_band");
    let ctx_id = response.id.with("eq_ctx_band");

    let band_pos = |b: &EqBand| {
        Pos2::new(
            rect.min.x + b.freq_norm * rect.width(),
            rect.center().y - b.gain_norm * rect.height() * 0.42,
        )
    };
    let nearest_band = |bands: &[EqBand], pos: Pos2| -> Option<(usize, f32)> {
        bands
            .iter()
            .enumerate()
            .map(|(i, b)| (i, band_pos(b).distance(pos)))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    };

    // Double-click: delete band if near one (keeping at least 1), otherwise add a new Peak band
    if response.double_clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if let Some((idx, dist)) = nearest_band(bands, pos) {
                if dist < 14.0 && bands.len() > 1 {
                    bands.remove(idx);
                    response.mark_changed();
                } else if dist >= 14.0 && bands.len() < 8 {
                    let freq_norm = ((pos.x - rect.min.x) / rect.width()).clamp(0.03, 0.97);
                    let gain_norm = ((rect.center().y - pos.y) / (rect.height() * 0.42)).clamp(-1.0, 1.0);
                    bands.push(EqBand {
                        freq_norm,
                        gain_norm,
                        q: 1.4,
                        filter_type: EqFilterType::Peak,
                    });
                    response.mark_changed();
                }
            }
        }
    }

    if response.secondary_clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let pick = nearest_band(bands, pos).map(|(i, _)| i);
            ui.data_mut(|d| d.insert_temp(ctx_id, pick));
        }
    }

    if response.drag_started() {
        if let Some(pos) = response.interact_pointer_pos() {
            let best = nearest_band(bands, pos).map(|(i, _)| i);
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
                if let Some((idx, _)) = nearest_band(bands, pos) {
                    if let Some(b) = bands.get_mut(idx) {
                        b.q = (b.q + scroll.signum() * 0.25).clamp(0.3, 8.0);
                        response.mark_changed();
                    }
                }
            }
        }
    }

    // Right-click context menu for filter type + delete
    let mut ctx_changed = false;
    response.context_menu(|ui| {
        let idx = ui
            .data_mut(|d| d.get_temp::<Option<usize>>(ctx_id).flatten())
            .filter(|&i| i < bands.len());
        if let Some(i) = idx {
            ui.label(egui::RichText::new(format!("Band {}", i + 1)).strong());
            ui.separator();
            for ft in EqFilterType::ALL {
                let sel = bands[i].filter_type == ft;
                if ui.selectable_label(sel, ft.name()).clicked() {
                    bands[i].filter_type = ft;
                    ctx_changed = true;
                    ui.close_menu();
                }
            }
            if bands.len() > 1 {
                ui.separator();
                if ui.button("Delete band").clicked() {
                    bands.remove(i);
                    ctx_changed = true;
                    ui.close_menu();
                }
            }
        }
    });
    if ctx_changed {
        response.mark_changed();
    }

    let focused_idx = if response.dragged() {
        ui.data_mut(|d| d.get_temp::<Option<usize>>(active_id).flatten())
    } else if response.hovered() {
        ui.input(|i| i.pointer.hover_pos())
            .and_then(|pos| nearest_band(bands, pos))
            .filter(|(_, d)| *d < 22.0)
            .map(|(i, _)| i)
    } else {
        None
    };

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);
        if matches!(theme.skin, crate::theme::Skin::Hybrid | crate::theme::Skin::Skeuomorphic) {
            for i in 1..4 {
                let gx = rect.min.x + rect.width() * (i as f32 / 4.0);
                painter.line_segment(
                    [Pos2::new(gx, rect.min.y), Pos2::new(gx, rect.max.y)],
                    Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.border, 45)),
                );
            }
        }
        painter.line_segment(
            [Pos2::new(rect.min.x, rect.center().y), Pos2::new(rect.max.x, rect.center().y)],
            Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.border, 90)),
        );

        let steps = 96;
        let curve: Vec<Pos2> = (0..=steps)
            .map(|i| {
                let x_norm = i as f32 / steps as f32;
                let sum: f32 = bands.iter().map(|b| b.response_at(x_norm)).sum();
                Pos2::new(
                    rect.min.x + x_norm * rect.width(),
                    rect.center().y - sum.clamp(-1.0, 1.0) * rect.height() * 0.42,
                )
            })
            .collect();
        if matches!(theme.skin, crate::theme::Skin::Hybrid | crate::theme::Skin::Skeuomorphic) {
            let mut mesh = egui::Mesh::default();
            let base_y = rect.center().y;
            let is_skeuo = theme.skin == crate::theme::Skin::Skeuomorphic;
            let zero_col = AudioTheme::with_alpha(theme.palette.accent, 6);
            let n = (curve.len() - 1).max(1) as f32;
            for (i, &p) in curve.iter().enumerate() {
                let peak_col = if is_skeuo {
                    let blend = AudioTheme::lerp_color(theme.palette.accent_secondary, theme.palette.accent, i as f32 / n);
                    AudioTheme::with_alpha(blend, 110)
                } else {
                    AudioTheme::with_alpha(theme.palette.accent, 65)
                };
                mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: egui::epaint::WHITE_UV, color: peak_col });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p.x, base_y), uv: egui::epaint::WHITE_UV, color: zero_col });
                if i > 0 {
                    let base = ((i - 1) * 2) as u32;
                    mesh.add_triangle(base, base + 1, base + 2);
                    mesh.add_triangle(base + 1, base + 3, base + 2);
                }
            }
            painter.add(Shape::mesh(mesh));
            painter.add(Shape::line(curve.clone(), Stroke::new(4.8, AudioTheme::with_alpha(theme.palette.accent, 42))));
        }
        painter.add(Shape::line(curve, Stroke::new(2.0, theme.palette.accent)));

        for (i, b) in bands.iter().enumerate() {
            let p = band_pos(b);
            let is_foc = focused_idx == Some(i);
            let col = if is_foc { theme.palette.accent_secondary } else { theme.palette.accent };
            theme.paint_glow_circle(painter, p, if is_foc { 6.5 } else { 5.0 }, col);
            painter.circle_filled(p, if is_foc { 6.0 } else { 5.0 }, col);
            painter.circle_stroke(p, if is_foc { 6.0 } else { 5.0 }, Stroke::new(1.5, theme.palette.bg_base));
        }

        // Live HUD readout for focused/hovered band
        if let Some(i) = focused_idx {
            if let Some(b) = bands.get(i) {
                let freq_hz = 20.0 * (20000.0f32 / 20.0).powf(b.freq_norm);
                let freq_str = if freq_hz >= 1000.0 {
                    format!("{:.2}kHz", freq_hz / 1000.0)
                } else {
                    format!("{:.0}Hz", freq_hz)
                };
                let gain_db = b.gain_norm * 18.0;
                painter.text(
                    Pos2::new(rect.min.x + 6.0, rect.min.y + 9.0),
                    Align2::LEFT_CENTER,
                    format!("B{} {} {} {:+.1}dB Q{:.1}", i + 1, b.filter_type.label(), freq_str, gain_db, b.q),
                    FontId::monospace(9.0),
                    theme.palette.text_primary,
                );
            }
        }
        theme.paint_screen_overlay(painter, rect);
    }
    response.on_hover_text("Drag: freq/gain | Scroll: Q | Double-click: add/remove | Right-click: filter type")
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
