use crate::{
    buttons::{push_button, toggle_button},
    chrome::styled_text_input,
    controls::{Fader, RotaryKnob},
    meters::level_meter,
    scale::ParamScale,
    theme::{AudioTheme, Skin},
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Rounding, Sense, Shape, Stroke, Ui, Vec2};

// ---------------- 1. MIXER CHANNEL STRIP ----------------

#[derive(Clone, Debug)]
pub struct ChannelStripState {
    pub name: String,
    pub volume_db: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
    pub rec_arm: bool,
    pub peak_l: f32,
    pub peak_r: f32,
}

impl ChannelStripState {
    pub fn new(name: impl Into<String>, volume_db: f32, pan: f32) -> Self {
        Self {
            name: name.into(),
            volume_db,
            pan,
            mute: false,
            solo: false,
            rec_arm: false,
            peak_l: 0.78,
            peak_r: 0.74,
        }
    }
}

pub fn channel_strip(
    ui: &mut Ui,
    state: &mut ChannelStripState,
    level_l: f32,
    level_r: f32,
    theme: &AudioTheme,
) -> bool {
    let mut changed = false;
    let frame = egui::Frame::none()
        .fill(AudioTheme::darken(theme.palette.bg_panel, 0.08))
        .stroke(Stroke::new(1.0, theme.palette.border))
        .rounding(theme.rounding(6.0))
        .inner_margin(egui::Margin::same(7.0));

    frame.show(ui, |ui| {
        ui.set_width(84.0);
        ui.vertical_centered(|ui| {
            if styled_text_input(ui, &mut state.name, "Track", 70.0, theme).changed() {
                changed = true;
            }
            ui.add_space(4.0);

            if RotaryKnob::new(&mut state.pan, 0.0, ParamScale::Bipolar { max_abs: 1.0 }, "PAN", theme)
                .size(36.0)
                .show(ui)
                .changed()
            {
                changed = true;
            }
            ui.add_space(2.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 3.0;
                if toggle_button(ui, &mut state.mute, "M", Vec2::new(25.0, 18.0), Some(theme.palette.meter_mid), theme).clicked() {
                    changed = true;
                }
                if toggle_button(ui, &mut state.solo, "S", Vec2::new(25.0, 18.0), Some(theme.palette.accent), theme).clicked() {
                    changed = true;
                }
                if toggle_button(ui, &mut state.rec_arm, "R", Vec2::new(25.0, 18.0), Some(theme.palette.danger), theme).clicked() {
                    changed = true;
                }
            });
            ui.add_space(6.0);

            let effective_l = if state.mute { 0.0 } else { level_l };
            let effective_r = if state.mute { 0.0 } else { level_r };
            state.peak_l = state.peak_l.max(effective_l);
            state.peak_r = state.peak_r.max(effective_r);

            ui.horizontal(|ui| {
                if Fader::vertical(
                    &mut state.volume_db,
                    0.0,
                    ParamScale::Decibel { min_db: -60.0, max_db: 6.0 },
                    "dB",
                    theme,
                )
                .length(112.0)
                .show(ui)
                .changed()
                {
                    changed = true;
                }
                level_meter(
                    ui,
                    &[effective_l, effective_r],
                    &[state.peak_l, state.peak_r],
                    Vec2::new(24.0, 112.0),
                    theme,
                );
            });

            let db_str = ParamScale::Decibel { min_db: -60.0, max_db: 6.0 }.format(state.volume_db, "dB");
            let (readout_rect, _) = ui.allocate_exact_size(Vec2::new(76.0, 18.0), Sense::hover());
            theme.paint_recessed_rect(ui.painter(), readout_rect, 3.0);
            ui.painter().text(
                readout_rect.center(),
                Align2::CENTER_CENTER,
                db_str,
                FontId::monospace(10.0),
                if state.mute { theme.palette.text_muted } else { theme.palette.accent },
            );
        });
    });

    changed
}

// ---------------- 2. TRANSFER-FUNCTION / SATURATION DISPLAY ----------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaturationMode {
    Tube,
    Tape,
    SoftClip,
    Wavefold,
}

impl SaturationMode {
    pub const ALL: [Self; 4] = [Self::Tube, Self::Tape, Self::SoftClip, Self::Wavefold];

    pub fn label(self) -> &'static str {
        match self {
            Self::Tube => "TUBE",
            Self::Tape => "TAPE",
            Self::SoftClip => "SOFT",
            Self::Wavefold => "FOLD",
        }
    }

    pub fn shape(self, x: f32, drive: f32, bias: f32) -> f32 {
        let g = 1.0 + drive * 5.0;
        let v = x * g + bias * 0.6;
        match self {
            Self::Tube => {
                let y = if v >= 0.0 {
                    (v * 1.15).tanh()
                } else {
                    (v * 0.85).tanh() * 0.92
                };
                y.clamp(-1.0, 1.0)
            }
            Self::Tape => (v / (1.0 + v.abs() * 0.65)).clamp(-1.0, 1.0),
            Self::SoftClip => (v - (v.clamp(-1.0, 1.0).powi(3)) / 3.0).clamp(-1.0, 1.0),
            Self::Wavefold => (v * std::f32::consts::FRAC_PI_2).sin().clamp(-1.0, 1.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SaturationState {
    pub mode: SaturationMode,
    pub drive: f32,
    pub bias: f32,
}

impl Default for SaturationState {
    fn default() -> Self {
        Self {
            mode: SaturationMode::Tube,
            drive: 0.45,
            bias: 0.1,
        }
    }
}

pub fn saturation_display(
    ui: &mut Ui,
    state: &mut SaturationState,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let mut changed = false;
    let resp = ui
        .vertical(|ui| {
            ui.horizontal(|ui| {
                let idx = SaturationMode::ALL.iter().position(|&m| m == state.mode).unwrap_or(0);
                if push_button(ui, "<", Vec2::new(20.0, 18.0), theme).clicked() {
                    state.mode = SaturationMode::ALL[(idx + SaturationMode::ALL.len() - 1) % SaturationMode::ALL.len()];
                    changed = true;
                }
                let (lbl_rect, lbl_resp) = ui.allocate_exact_size(Vec2::new((size.x - 48.0).max(52.0), 18.0), Sense::click());
                if lbl_resp.clicked() {
                    state.mode = SaturationMode::ALL[(idx + 1) % SaturationMode::ALL.len()];
                    changed = true;
                }
                theme.paint_recessed_rect(ui.painter(), lbl_rect, 3.0);
                ui.painter().text(
                    lbl_rect.center(),
                    Align2::CENTER_CENTER,
                    state.mode.label(),
                    FontId::monospace(10.5),
                    theme.palette.accent,
                );
                if push_button(ui, ">", Vec2::new(20.0, 18.0), theme).clicked() {
                    state.mode = SaturationMode::ALL[(idx + 1) % SaturationMode::ALL.len()];
                    changed = true;
                }
            });
            ui.add_space(4.0);

            let (rect, mut plot_resp) = ui.allocate_exact_size(size, Sense::click_and_drag());
            if plot_resp.double_clicked() {
                state.drive = 0.45;
                state.bias = 0.0;
                changed = true;
            } else if plot_resp.dragged() {
                let d = plot_resp.drag_delta();
                state.drive = (state.drive - d.y * 0.008).clamp(0.0, 1.0);
                state.bias = (state.bias + d.x * 0.008).clamp(-1.0, 1.0);
                changed = true;
            }
            if changed {
                plot_resp.mark_changed();
            }

            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                theme.paint_recessed_rect(painter, rect, 5.0);

                // Subtle center crosshair + linear reference diagonal
                let grid_col = AudioTheme::with_alpha(theme.palette.border, 70);
                painter.line_segment(
                    [Pos2::new(rect.min.x, rect.center().y), Pos2::new(rect.max.x, rect.center().y)],
                    Stroke::new(1.0, grid_col),
                );
                painter.line_segment(
                    [Pos2::new(rect.center().x, rect.min.y), Pos2::new(rect.center().x, rect.max.y)],
                    Stroke::new(1.0, grid_col),
                );
                painter.line_segment(
                    [Pos2::new(rect.min.x + 8.0, rect.max.y - 8.0), Pos2::new(rect.max.x - 8.0, rect.min.y + 8.0)],
                    Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.text_muted, 40)),
                );

                let pad = 8.0;
                let inner = rect.shrink(pad);
                let steps = 72;
                let curve: Vec<Pos2> = (0..=steps)
                    .map(|i| {
                        let t = i as f32 / steps as f32;
                        let x_in = t * 2.0 - 1.0;
                        let y_out = state.mode.shape(x_in, state.drive, state.bias);
                        Pos2::new(
                            inner.min.x + t * inner.width(),
                            inner.center().y - y_out * (inner.height() * 0.48),
                        )
                    })
                    .collect();

                if matches!(theme.skin, Skin::Hybrid | Skin::Skeuomorphic) {
                    let mut mesh = egui::Mesh::default();
                    let base_y = inner.center().y;
                    let top_col = AudioTheme::with_alpha(theme.palette.accent, if theme.skin == Skin::Skeuomorphic { 95 } else { 60 });
                    let zero_col = AudioTheme::with_alpha(theme.palette.accent, 6);
                    for (i, &p) in curve.iter().enumerate() {
                        mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: egui::epaint::WHITE_UV, color: top_col });
                        mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p.x, base_y), uv: egui::epaint::WHITE_UV, color: zero_col });
                        if i > 0 {
                            let b = ((i - 1) * 2) as u32;
                            mesh.add_triangle(b, b + 1, b + 2);
                            mesh.add_triangle(b + 1, b + 3, b + 2);
                        }
                    }
                    painter.add(Shape::mesh(mesh));
                    painter.add(Shape::line(curve.clone(), Stroke::new(4.5, AudioTheme::with_alpha(theme.palette.accent, 42))));
                }
                painter.add(Shape::line(curve, Stroke::new(2.0, theme.palette.accent)));

                painter.text(
                    Pos2::new(rect.min.x + 6.0, rect.min.y + 8.0),
                    Align2::LEFT_CENTER,
                    format!("DRV {:>3.0}%", state.drive * 100.0),
                    FontId::monospace(9.0),
                    theme.palette.text_muted,
                );
                painter.text(
                    Pos2::new(rect.max.x - 6.0, rect.max.y - 8.0),
                    Align2::RIGHT_CENTER,
                    format!("BIAS {:+.2}", state.bias),
                    FontId::monospace(9.0),
                    theme.palette.text_muted,
                );
                theme.paint_screen_overlay(painter, rect);
            }
            plot_resp
        })
        .inner;
    resp
}

// ---------------- 3. MODULATION MATRIX ----------------

#[derive(Clone, Debug)]
pub struct ModRoute {
    pub source_idx: usize,
    pub dest_idx: usize,
    pub depth: f32, // -1.0..=1.0
}

pub fn mod_matrix(
    ui: &mut Ui,
    sources: &[(&str, f32)],
    destinations: &[&str],
    routes: &mut Vec<ModRoute>,
    selected_source: &mut usize,
    theme: &AudioTheme,
) -> bool {
    let mut changed = false;

    // Source pills row
    ui.horizontal(|ui| {
        for (i, &(name, live_val)) in sources.iter().enumerate() {
            let is_sel = i == *selected_source;
            let (rect, resp) = ui.allocate_exact_size(Vec2::new(56.0, 22.0), Sense::click());
            if resp.clicked() {
                *selected_source = i;
            }
            let bg = if is_sel {
                AudioTheme::with_alpha(theme.palette.accent_secondary, 55)
            } else {
                theme.palette.bg_panel
            };
            theme.paint_raised_rect(ui.painter(), rect, bg, 10.0);
            if is_sel {
                ui.painter().rect_stroke(rect, Rounding::same(10.0), Stroke::new(1.2, theme.palette.accent_secondary));
            }
            let dot = Pos2::new(rect.min.x + 9.0, rect.center().y);
            let alpha = (60.0 + live_val.abs().clamp(0.0, 1.0) * 195.0) as u8;
            let dot_col = AudioTheme::with_alpha(theme.palette.accent_secondary, alpha);
            theme.paint_glow_circle(ui.painter(), dot, 3.0, dot_col);
            ui.painter().circle_filled(dot, 2.8, dot_col);
            ui.painter().text(
                Pos2::new(rect.center().x + 5.0, rect.center().y),
                Align2::CENTER_CENTER,
                name,
                FontId::monospace(9.5),
                if is_sel { theme.palette.text_primary } else { theme.palette.text_muted },
            );
        }

        if !sources.is_empty() && !destinations.is_empty() {
            if push_button(ui, "+", Vec2::new(22.0, 22.0), theme).clicked() {
                routes.push(ModRoute {
                    source_idx: (*selected_source).min(sources.len() - 1),
                    dest_idx: routes.len() % destinations.len(),
                    depth: 0.35,
                });
                changed = true;
            }
        }
    });

    ui.add_space(6.0);

    let mut remove_idx: Option<usize> = None;
    for (r_idx, route) in routes.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            let src_name = sources.get(route.source_idx).map(|s| s.0).unwrap_or("SRC");
            let dst_name = destinations.get(route.dest_idx).copied().unwrap_or("DST");

            if push_button(ui, src_name, Vec2::new(50.0, 20.0), theme).clicked() && !sources.is_empty() {
                route.source_idx = (route.source_idx + 1) % sources.len();
                changed = true;
            }
            ui.label(egui::RichText::new("->").monospace().size(10.0).color(theme.palette.text_muted));
            if push_button(ui, dst_name, Vec2::new(64.0, 20.0), theme).clicked() && !destinations.is_empty() {
                route.dest_idx = (route.dest_idx + 1) % destinations.len();
                changed = true;
            }

            // Bipolar depth bar
            let (bar_rect, bar_resp) = ui.allocate_exact_size(Vec2::new(96.0, 20.0), Sense::click_and_drag());
            if bar_resp.double_clicked() {
                route.depth = 0.0;
                changed = true;
            } else if bar_resp.dragged() {
                route.depth = (route.depth + bar_resp.drag_delta().x * 0.012).clamp(-1.0, 1.0);
                changed = true;
            }
            if ui.is_rect_visible(bar_rect) {
                let painter = ui.painter();
                theme.paint_recessed_rect(painter, bar_rect, 3.0);
                let cx = bar_rect.center().x;
                let x = cx + route.depth * (bar_rect.width() * 0.46);
                let (x0, x1) = if x >= cx { (cx, x) } else { (x, cx) };
                let col = if route.depth >= 0.0 { theme.palette.accent_secondary } else { theme.palette.accent };
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(x0, bar_rect.min.y + 3.0), Pos2::new(x1, bar_rect.max.y - 3.0)),
                    1.5,
                    AudioTheme::with_alpha(col, 190),
                );
                painter.line_segment(
                    [Pos2::new(cx, bar_rect.min.y + 1.0), Pos2::new(cx, bar_rect.max.y - 1.0)],
                    Stroke::new(1.0, theme.palette.text_muted),
                );
                painter.text(
                    bar_rect.center(),
                    Align2::CENTER_CENTER,
                    format!("{:+.0}%", route.depth * 100.0),
                    FontId::monospace(9.5),
                    theme.palette.text_primary,
                );
            }

            if push_button(ui, "x", Vec2::new(20.0, 20.0), theme).clicked() {
                remove_idx = Some(r_idx);
                changed = true;
            }
        });
    }

    if let Some(idx) = remove_idx {
        routes.remove(idx);
    }

    changed
}

// ---------------- 4. PIANO ROLL / CLIP EDITOR ----------------

#[derive(Clone, Copy, Debug)]
pub struct PianoNote {
    pub pitch: u8,      // 0..12 (C..B within displayed octave)
    pub start: f32,     // in beats (0.0..total_beats)
    pub length: f32,    // in beats
    pub velocity: f32,  // 0.0..=1.0
}

#[derive(Clone, Debug)]
pub struct PianoRollState {
    pub notes: Vec<PianoNote>,
    pub total_beats: f32,
    pub loop_start: f32,
    pub loop_end: f32,
}

impl Default for PianoRollState {
    fn default() -> Self {
        Self {
            notes: vec![
                PianoNote { pitch: 0, start: 0.0, length: 0.75, velocity: 0.9 },
                PianoNote { pitch: 7, start: 0.75, length: 0.5, velocity: 0.75 },
                PianoNote { pitch: 3, start: 1.5, length: 0.75, velocity: 0.85 },
                PianoNote { pitch: 5, start: 2.25, length: 0.5, velocity: 0.7 },
                PianoNote { pitch: 10, start: 3.0, length: 0.75, velocity: 0.92 },
            ],
            total_beats: 4.0,
            loop_start: 0.0,
            loop_end: 4.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PianoDragMode {
    None,
    MoveNote { idx: usize, grab_offset_beats: f32 },
    ResizeNote { idx: usize },
    LoopStart,
    LoopEnd,
    Velocity { idx: usize },
}

pub fn piano_roll(
    ui: &mut Ui,
    state: &mut PianoRollState,
    playhead_beat: f32,
    size: Vec2,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());
    let key_w = 24.0;
    let loop_h = 14.0;
    let vel_h = (rect.height() * 0.24).clamp(22.0, 34.0);
    let loop_bar = Rect::from_min_max(
        Pos2::new(rect.min.x + key_w, rect.min.y),
        Pos2::new(rect.max.x, rect.min.y + loop_h),
    );
    let grid = Rect::from_min_max(
        Pos2::new(rect.min.x + key_w, rect.min.y + loop_h),
        Pos2::new(rect.max.x, rect.max.y - vel_h),
    );
    let vel_lane = Rect::from_min_max(
        Pos2::new(rect.min.x + key_w, grid.max.y),
        rect.max,
    );

    let rows = 12usize;
    let row_h = (grid.height() / rows as f32).max(1.0);
    let total_beats = state.total_beats.max(1.0);
    let beat_w = grid.width() / total_beats;
    let snap = 0.25f32;
    let drag_id = response.id.with("pr_drag");

    let hit_note = |notes: &[PianoNote], pos: Pos2| -> Option<(usize, bool)> {
        let beat = (pos.x - grid.min.x) / beat_w;
        let row_from_top = ((pos.y - grid.min.y) / row_h).floor() as isize;
        if row_from_top < 0 || row_from_top >= rows as isize {
            return None;
        }
        let pitch = (rows - 1 - row_from_top as usize) as u8;
        for (i, n) in notes.iter().enumerate() {
            if n.pitch == pitch && beat >= n.start && beat <= n.start + n.length {
                let note_right_x = grid.min.x + ((n.start + n.length) / total_beats) * grid.width();
                let on_right_edge = (note_right_x - pos.x).abs() <= 6.0;
                return Some((i, on_right_edge));
            }
        }
        None
    };

    if response.double_clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if grid.contains(pos) {
                if let Some((idx, _)) = hit_note(&state.notes, pos) {
                    state.notes.remove(idx);
                    response.mark_changed();
                }
            }
        }
    }

    if response.drag_started() {
        let mut mode = PianoDragMode::None;
        if let Some(pos) = response.interact_pointer_pos() {
            if loop_bar.contains(pos) {
                let lx0 = grid.min.x + (state.loop_start / total_beats) * grid.width();
                let lx1 = grid.min.x + (state.loop_end / total_beats) * grid.width();
                mode = if (pos.x - lx0).abs() <= (pos.x - lx1).abs() {
                    PianoDragMode::LoopStart
                } else {
                    PianoDragMode::LoopEnd
                };
            } else if vel_lane.contains(pos) {
                let beat = (pos.x - grid.min.x) / beat_w;
                if let Some((idx, _)) = state
                    .notes
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| (a.start - beat).abs().total_cmp(&(b.start - beat).abs()))
                {
                    mode = PianoDragMode::Velocity { idx };
                }
            } else if grid.contains(pos) {
                if let Some((idx, on_edge)) = hit_note(&state.notes, pos) {
                    if on_edge {
                        mode = PianoDragMode::ResizeNote { idx };
                    } else {
                        let beat = (pos.x - grid.min.x) / beat_w;
                        mode = PianoDragMode::MoveNote {
                            idx,
                            grab_offset_beats: beat - state.notes[idx].start,
                        };
                    }
                }
            }
        }
        ui.data_mut(|d| d.insert_temp(drag_id, mode));
    }

    if response.dragged() {
        if let Some(pos) = response.interact_pointer_pos() {
            let mode = ui.data_mut(|d| d.get_temp::<PianoDragMode>(drag_id).unwrap_or(PianoDragMode::None));
            let beat = ((pos.x - grid.min.x) / beat_w).clamp(0.0, total_beats);
            match mode {
                PianoDragMode::MoveNote { idx, grab_offset_beats } => {
                    if let Some(n) = state.notes.get_mut(idx) {
                        let new_start = (((beat - grab_offset_beats) / snap).round() * snap)
                            .clamp(0.0, (total_beats - n.length).max(0.0));
                        let row_from_top = ((pos.y - grid.min.y) / row_h).floor().clamp(0.0, (rows - 1) as f32) as usize;
                        n.start = new_start;
                        n.pitch = (rows - 1 - row_from_top) as u8;
                        response.mark_changed();
                    }
                }
                PianoDragMode::ResizeNote { idx } => {
                    if let Some(n) = state.notes.get_mut(idx) {
                        let raw_len = (beat - n.start).max(snap);
                        let snapped_len = ((raw_len / snap).round() * snap)
                            .clamp(snap, (total_beats - n.start).max(snap));
                        n.length = snapped_len;
                        response.mark_changed();
                    }
                }
                PianoDragMode::LoopStart => {
                    let s = ((beat / snap).round() * snap).clamp(0.0, state.loop_end - snap);
                    state.loop_start = s;
                    response.mark_changed();
                }
                PianoDragMode::LoopEnd => {
                    let e = ((beat / snap).round() * snap).clamp(state.loop_start + snap, total_beats);
                    state.loop_end = e;
                    response.mark_changed();
                }
                PianoDragMode::Velocity { idx } => {
                    if let Some(n) = state.notes.get_mut(idx) {
                        n.velocity = (1.0 - (pos.y - vel_lane.min.y) / vel_lane.height()).clamp(0.1, 1.0);
                        response.mark_changed();
                    }
                }
                PianoDragMode::None => {}
            }
        }
    } else if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            if vel_lane.contains(pos) {
                let beat = (pos.x - grid.min.x) / beat_w;
                if let Some(n) = state
                    .notes
                    .iter_mut()
                    .min_by(|a, b| (a.start - beat).abs().total_cmp(&(b.start - beat).abs()))
                {
                    n.velocity = (1.0 - (pos.y - vel_lane.min.y) / vel_lane.height()).clamp(0.1, 1.0);
                    response.mark_changed();
                }
            } else if grid.contains(pos) && hit_note(&state.notes, pos).is_none() {
                let beat = ((pos.x - grid.min.x) / beat_w).clamp(0.0, total_beats - snap);
                let row_from_top = ((pos.y - grid.min.y) / row_h).floor().clamp(0.0, (rows - 1) as f32) as usize;
                let pitch = (rows - 1 - row_from_top) as u8;
                let snapped = (beat / snap).floor() * snap;
                state.notes.push(PianoNote {
                    pitch,
                    start: snapped,
                    length: 0.5,
                    velocity: 0.85,
                });
                response.mark_changed();
            }
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);

        let loop_rect = Rect::from_min_max(
            Pos2::new(grid.min.x + (state.loop_start / total_beats) * grid.width(), rect.min.y + 2.0),
            Pos2::new(grid.min.x + (state.loop_end / total_beats) * grid.width(), rect.min.y + loop_h - 1.0),
        );
        painter.rect_filled(loop_rect, 2.0, AudioTheme::with_alpha(theme.palette.accent, 65));
        painter.line_segment(
            [Pos2::new(loop_rect.min.x, loop_rect.min.y), Pos2::new(loop_rect.max.x, loop_rect.min.y)],
            Stroke::new(2.0, theme.palette.accent),
        );
        for hx in [loop_rect.min.x + 2.0, loop_rect.max.x - 2.0] {
            painter.rect_filled(
                Rect::from_center_size(Pos2::new(hx, loop_rect.center().y), Vec2::new(4.0, loop_rect.height() - 2.0)),
                1.0,
                theme.palette.accent,
            );
        }

        let is_black = |p: usize| matches!(p % 12, 1 | 3 | 6 | 8 | 10);
        let note_names = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

        for r in 0..rows {
            let pitch = rows - 1 - r;
            let y0 = grid.min.y + r as f32 * row_h;
            let y1 = y0 + row_h;

            let key_rect = Rect::from_min_max(Pos2::new(rect.min.x + 1.0, y0), Pos2::new(grid.min.x - 1.0, y1));
            let key_col = if is_black(pitch) {
                AudioTheme::darken(theme.palette.bg_recessed, 0.3)
            } else {
                AudioTheme::lighten(theme.palette.bg_panel, 0.1)
            };
            painter.rect_filled(key_rect, 1.0, key_col);
            if pitch == 0 || pitch == 7 {
                painter.text(
                    key_rect.center(),
                    Align2::CENTER_CENTER,
                    note_names[pitch],
                    FontId::monospace(8.0),
                    theme.palette.text_muted,
                );
            }

            if is_black(pitch) {
                painter.rect_filled(
                    Rect::from_min_max(Pos2::new(grid.min.x, y0), Pos2::new(grid.max.x, y1)),
                    0.0,
                    Color32::from_black_alpha(38),
                );
            }
            painter.line_segment(
                [Pos2::new(grid.min.x, y1), Pos2::new(grid.max.x, y1)],
                Stroke::new(0.6, AudioTheme::with_alpha(theme.palette.border, 45)),
            );
        }

        let subdivs = (total_beats * 4.0) as usize;
        for s in 0..=subdivs {
            let x = grid.min.x + (s as f32 / subdivs as f32) * grid.width();
            let is_beat = s % 4 == 0;
            let col = if is_beat {
                AudioTheme::with_alpha(theme.palette.border, 120)
            } else {
                AudioTheme::with_alpha(theme.palette.border, 40)
            };
            painter.line_segment([Pos2::new(x, grid.min.y), Pos2::new(x, rect.max.y)], Stroke::new(1.0, col));
        }

        painter.line_segment(
            [Pos2::new(rect.min.x + 1.0, vel_lane.min.y), Pos2::new(rect.max.x - 1.0, vel_lane.min.y)],
            Stroke::new(1.0, theme.palette.border),
        );
        painter.text(
            Pos2::new(rect.min.x + key_w * 0.5, vel_lane.center().y),
            Align2::CENTER_CENTER,
            "VEL",
            FontId::monospace(7.5),
            theme.palette.text_muted,
        );

        for n in &state.notes {
            let r = (rows - 1).saturating_sub(n.pitch as usize);
            let x0 = grid.min.x + (n.start / total_beats) * grid.width() + 0.5;
            let x1 = grid.min.x + ((n.start + n.length) / total_beats) * grid.width() - 0.5;
            let y0 = grid.min.y + r as f32 * row_h + 1.0;
            let y1 = y0 + row_h - 2.0;
            let note_rect = Rect::from_min_max(Pos2::new(x0, y0), Pos2::new(x1.max(x0 + 5.0), y1));
            let col = AudioTheme::lerp_color(theme.palette.accent_secondary, theme.palette.accent, n.velocity);
            painter.rect_filled(note_rect, 2.0, col);
            painter.rect_stroke(note_rect, 2.0, Stroke::new(1.0, Color32::from_white_alpha(70)));

            let vx = x0 + 3.0;
            let vh = (vel_lane.height() - 5.0) * n.velocity.clamp(0.1, 1.0);
            let vy = vel_lane.max.y - 2.0 - vh;
            painter.line_segment(
                [Pos2::new(vx, vel_lane.max.y - 2.0), Pos2::new(vx, vy)],
                Stroke::new(2.0, col),
            );
            painter.circle_filled(Pos2::new(vx, vy), 2.5, theme.palette.text_primary);
        }

        let loop_span = (state.loop_end - state.loop_start).max(snap);
        let ph_beat = state.loop_start + (playhead_beat % loop_span);
        let px = grid.min.x + (ph_beat / total_beats) * grid.width();
        painter.line_segment(
            [Pos2::new(px, rect.min.y + 1.0), Pos2::new(px, rect.max.y - 1.0)],
            Stroke::new(1.5, theme.palette.text_primary),
        );

        theme.paint_screen_overlay(painter, rect);
    }

    response.on_hover_text("Click: add | Drag: move | Right edge: resize | Double-click: delete | Bottom: velocity")
}
