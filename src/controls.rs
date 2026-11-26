use crate::{
    interaction::{handle_continuous_interaction, DragConfig},
    scale::ParamScale,
    theme::{AudioTheme, Skin},
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2};
use std::f32::consts::PI;

// ---------------- 1. ROTARY KNOB ----------------

pub struct RotaryKnob<'a> {
    value: &'a mut f32,
    default_value: f32,
    scale: ParamScale,
    label: &'a str,
    unit: &'a str,
    size: f32,
    modulation: Option<f32>,
    theme: &'a AudioTheme,
}

impl<'a> RotaryKnob<'a> {
    pub fn new(
        value: &'a mut f32,
        default_value: f32,
        scale: ParamScale,
        label: &'a str,
        theme: &'a AudioTheme,
    ) -> Self {
        Self {
            value,
            default_value,
            scale,
            label,
            unit: "",
            size: 56.0,
            modulation: None,
            theme,
        }
    }

    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn modulation(mut self, mod_amount: f32) -> Self {
        self.modulation = Some(mod_amount);
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let total_size = Vec2::new(self.size + 14.0, self.size + 28.0);
        let (rect, mut response) = ui.allocate_exact_size(total_size, Sense::click_and_drag());

        if handle_continuous_interaction(
            ui,
            &response,
            self.value,
            self.default_value,
            self.scale,
            DragConfig::default(),
        ) {
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = Pos2::new(rect.center().x, rect.min.y + self.size * 0.5 + 4.0);
            let radius = self.size * 0.38;
            let arc_radius = radius + 6.5;
            let start_ang = PI * 0.75;
            let sweep = PI * 1.5;
            let norm = self.scale.normalize(*self.value);
            let val_ang = start_ang + norm * sweep;

            if self.theme.skin == Skin::Hybrid {
                let ticks = 10;
                for i in 0..=ticks {
                    let t = i as f32 / ticks as f32;
                    let a = start_ang + t * sweep;
                    let is_major = i == 0 || i == ticks / 2 || i == ticks;
                    let r0 = arc_radius + 3.0;
                    let r1 = arc_radius + if is_major { 6.0 } else { 4.5 };
                    let col = if is_major {
                        AudioTheme::with_alpha(self.theme.palette.text_muted, 150)
                    } else {
                        AudioTheme::with_alpha(self.theme.palette.border, 150)
                    };
                    painter.line_segment(
                        [center + Vec2::angled(a) * r0, center + Vec2::angled(a) * r1],
                        Stroke::new(if is_major { 1.2 } else { 1.0 }, col),
                    );
                }
            }

            draw_arc(
                painter,
                center,
                arc_radius,
                start_ang,
                start_ang + sweep,
                Stroke::new(3.0, self.theme.palette.bg_recessed),
            );

            let (a0, a1) = if self.scale.is_bipolar() {
                let mid = start_ang + sweep * 0.5;
                if val_ang >= mid { (mid, val_ang) } else { (val_ang, mid) }
            } else {
                (start_ang, val_ang)
            };
            if (a1 - a0).abs() > 0.01 {
                if self.theme.skin == Skin::Hybrid {
                    draw_arc(
                        painter,
                        center,
                        arc_radius,
                        a0,
                        a1,
                        Stroke::new(5.5, AudioTheme::with_alpha(self.theme.palette.accent, 36)),
                    );
                }
                draw_arc(
                    painter,
                    center,
                    arc_radius,
                    a0,
                    a1,
                    Stroke::new(3.0, self.theme.palette.accent),
                );
            }

            if let Some(m) = self.modulation {
                let mod_norm = (norm + m).clamp(0.0, 1.0);
                let mod_ang = start_ang + mod_norm * sweep;
                let (m0, m1) = if mod_ang >= val_ang { (val_ang, mod_ang) } else { (mod_ang, val_ang) };
                draw_arc(
                    painter,
                    center,
                    arc_radius + 3.5,
                    m0,
                    m1,
                    Stroke::new(2.0, self.theme.palette.accent_secondary),
                );
            }

            match self.theme.skin {
                Skin::Skeuomorphic => {
                    painter.circle_filled(center + Vec2::new(0.0, 3.0), radius, Color32::from_black_alpha(130));
                    painter.circle_filled(center, radius, AudioTheme::darken(self.theme.palette.bg_panel, 0.25));
                    painter.circle_filled(center, radius * 0.84, AudioTheme::lighten(self.theme.palette.bg_panel, 0.08));
                    for i in 0..16 {
                        let a = (i as f32 / 16.0) * PI * 2.0;
                        let p1 = center + Vec2::angled(a) * (radius * 0.85);
                        let p2 = center + Vec2::angled(a) * radius;
                        painter.line_segment([p1, p2], Stroke::new(1.0, Color32::from_white_alpha(25)));
                    }
                    painter.circle_stroke(center, radius, Stroke::new(1.0, Color32::from_black_alpha(180)));
                }
                Skin::Hybrid => {
                    AudioTheme::paint_radial_gradient_circle(
                        painter,
                        center + Vec2::new(0.0, 2.5),
                        radius * 1.15,
                        Color32::from_black_alpha(105),
                        Color32::TRANSPARENT,
                    );
                    painter.circle_filled(center, radius, AudioTheme::darken(self.theme.palette.bg_panel, 0.14));
                    AudioTheme::paint_directional_circle(
                        painter,
                        center,
                        radius * 0.86,
                        AudioTheme::lighten(self.theme.palette.bg_panel, 0.12),
                        AudioTheme::darken(self.theme.palette.bg_panel, 0.08),
                    );
                    painter.circle_stroke(
                        center,
                        radius * 0.86,
                        Stroke::new(0.8, Color32::from_white_alpha(24)),
                    );
                    let rim_col = if response.hovered() || response.dragged() {
                        AudioTheme::lerp_color(self.theme.palette.border, self.theme.palette.accent, 0.45)
                    } else {
                        self.theme.palette.border
                    };
                    painter.circle_stroke(center, radius, Stroke::new(1.2, rim_col));
                }
                Skin::Flat => {
                    painter.circle_filled(center, radius, self.theme.palette.bg_panel);
                    painter.circle_stroke(center, radius, Stroke::new(1.0, self.theme.palette.border));
                }
            }

            let dir = Vec2::angled(val_ang);
            if self.theme.skin == Skin::Hybrid {
                painter.line_segment(
                    [center + dir * (radius * 0.28), center + dir * (radius * 0.82)],
                    Stroke::new(4.0, AudioTheme::with_alpha(self.theme.palette.accent, 50)),
                );
                painter.circle_filled(center, radius * 0.16, AudioTheme::darken(self.theme.palette.bg_panel, 0.22));
            }
            painter.line_segment(
                [center + dir * (radius * 0.3), center + dir * (radius * 0.85)],
                Stroke::new(2.2, self.theme.palette.accent),
            );

            let text = if response.hovered() || response.dragged() {
                self.scale.format(*self.value, self.unit)
            } else {
                self.label.to_string()
            };
            painter.text(
                Pos2::new(rect.center().x, rect.max.y - 6.0),
                Align2::CENTER_CENTER,
                text,
                FontId::monospace(10.5),
                if response.dragged() {
                    self.theme.palette.accent
                } else {
                    self.theme.palette.text_muted
                },
            );
        }

        response
    }
}

fn draw_arc(painter: &egui::Painter, center: Pos2, radius: f32, a0: f32, a1: f32, stroke: Stroke) {
    let steps = 28;
    let pts: Vec<Pos2> = (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            let a = a0 + (a1 - a0) * t;
            center + Vec2::angled(a) * radius
        })
        .collect();
    painter.add(Shape::line(pts, stroke));
}

// ---------------- 2. VERTICAL / HORIZONTAL FADER ----------------

pub struct Fader<'a> {
    value: &'a mut f32,
    default_value: f32,
    scale: ParamScale,
    label: &'a str,
    vertical: bool,
    length: f32,
    meter_level: Option<f32>,
    theme: &'a AudioTheme,
}

impl<'a> Fader<'a> {
    pub fn vertical(
        value: &'a mut f32,
        default_value: f32,
        scale: ParamScale,
        label: &'a str,
        theme: &'a AudioTheme,
    ) -> Self {
        Self {
            value,
            default_value,
            scale,
            label,
            vertical: true,
            length: 140.0,
            meter_level: None,
            theme,
        }
    }

    pub fn horizontal(mut self) -> Self {
        self.vertical = false;
        self
    }

    pub fn length(mut self, len: f32) -> Self {
        self.length = len;
        self
    }

    pub fn meter_level(mut self, level: f32) -> Self {
        self.meter_level = Some(level.clamp(0.0, 1.0));
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let size = if self.vertical {
            Vec2::new(42.0, self.length + 22.0)
        } else {
            Vec2::new(self.length, 32.0)
        };
        let (rect, mut response) = ui.allocate_exact_size(size, Sense::click_and_drag());

        let cfg = DragConfig {
            horizontal: !self.vertical,
            sensitivity: 1.0 / self.length,
            ..Default::default()
        };
        if handle_continuous_interaction(ui, &response, self.value, self.default_value, self.scale, cfg) {
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let norm = self.scale.normalize(*self.value);

            if self.vertical {
                let track_rect = Rect::from_center_size(
                    Pos2::new(rect.center().x, rect.min.y + self.length * 0.5),
                    Vec2::new(6.0, self.length - 12.0),
                );
                self.theme.paint_recessed_rect(painter, track_rect, 3.0);

                for i in 0..=8 {
                    let y = track_rect.min.y + (i as f32 / 8.0) * track_rect.height();
                    let w = if i % 2 == 0 { 6.0 } else { 3.5 };
                    painter.line_segment(
                        [Pos2::new(track_rect.min.x - 4.0 - w, y), Pos2::new(track_rect.min.x - 4.0, y)],
                        Stroke::new(1.0, self.theme.palette.border),
                    );
                }

                if let Some(lvl) = self.meter_level {
                    let h = track_rect.height() * lvl;
                    let m_rect = Rect::from_min_max(
                        Pos2::new(track_rect.min.x + 1.0, track_rect.max.y - h),
                        Pos2::new(track_rect.max.x - 1.0, track_rect.max.y),
                    );
                    painter.rect_filled(m_rect, 1.0, self.theme.palette.meter_low);
                }

                let cap_y = track_rect.max.y - norm * track_rect.height();
                let cap_rect = Rect::from_center_size(Pos2::new(track_rect.center().x, cap_y), Vec2::new(24.0, 15.0));
                let cap_col = match self.theme.skin {
                    Skin::Skeuomorphic => AudioTheme::lighten(self.theme.palette.bg_panel, 0.25),
                    Skin::Hybrid => AudioTheme::lighten(self.theme.palette.bg_panel, 0.14),
                    Skin::Flat => self.theme.palette.bg_panel,
                };
                self.theme.paint_raised_rect(painter, cap_rect, cap_col, 2.5);
                if self.theme.skin == Skin::Hybrid {
                    for offset in [-3.5f32, 3.5] {
                        painter.line_segment(
                            [Pos2::new(cap_rect.min.x + 5.0, cap_y + offset), Pos2::new(cap_rect.max.x - 5.0, cap_y + offset)],
                            Stroke::new(1.0, Color32::from_black_alpha(65)),
                        );
                    }
                }
                painter.line_segment(
                    [Pos2::new(cap_rect.min.x + 3.0, cap_y), Pos2::new(cap_rect.max.x - 3.0, cap_y)],
                    Stroke::new(2.0, self.theme.palette.accent),
                );

                painter.text(
                    Pos2::new(rect.center().x, rect.max.y - 8.0),
                    Align2::CENTER_CENTER,
                    self.label,
                    FontId::monospace(10.0),
                    self.theme.palette.text_muted,
                );
            } else {
                let track_rect = Rect::from_center_size(rect.center(), Vec2::new(self.length - 16.0, 6.0));
                self.theme.paint_recessed_rect(painter, track_rect, 3.0);
                let fill_rect = Rect::from_min_max(
                    track_rect.min,
                    Pos2::new(track_rect.min.x + norm * track_rect.width(), track_rect.max.y),
                );
                painter.rect_filled(fill_rect, 2.0, self.theme.palette.accent);
                let cap_x = track_rect.min.x + norm * track_rect.width();
                let cap_rect = Rect::from_center_size(Pos2::new(cap_x, track_rect.center().y), Vec2::new(12.0, 20.0));
                self.theme.paint_raised_rect(painter, cap_rect, self.theme.palette.bg_panel, 2.0);
            }
        }

        response
    }
}

// ---------------- 3. XY PAD ----------------

pub struct XyPad<'a> {
    x: &'a mut f32,
    y: &'a mut f32,
    size: Vec2,
    theme: &'a AudioTheme,
}

impl<'a> XyPad<'a> {
    pub fn new(x: &'a mut f32, y: &'a mut f32, theme: &'a AudioTheme) -> Self {
        Self {
            x,
            y,
            size: Vec2::new(140.0, 140.0),
            theme,
        }
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(self.size, Sense::click_and_drag());
        let mods = ui.input(|i| i.modifiers);

        if response.double_clicked() {
            *self.x = 0.5;
            *self.y = 0.5;
            response.mark_changed();
        } else if (response.dragged() || response.clicked()) && response.interact_pointer_pos().is_some() {
            let pos = response.interact_pointer_pos().unwrap();
            if !mods.alt {
                *self.x = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            }
            if !mods.shift {
                *self.y = (1.0 - (pos.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
            }
            response.mark_changed();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            self.theme.paint_recessed_rect(painter, rect, 4.0);

            for i in 1..4 {
                let t = i as f32 / 4.0;
                let gx = rect.min.x + rect.width() * t;
                let gy = rect.min.y + rect.height() * t;
                painter.line_segment([Pos2::new(gx, rect.min.y), Pos2::new(gx, rect.max.y)], Stroke::new(1.0, AudioTheme::with_alpha(self.theme.palette.border, 60)));
                painter.line_segment([Pos2::new(rect.min.x, gy), Pos2::new(rect.max.x, gy)], Stroke::new(1.0, AudioTheme::with_alpha(self.theme.palette.border, 60)));
            }

            let puck = Pos2::new(
                rect.min.x + *self.x * rect.width(),
                rect.max.y - *self.y * rect.height(),
            );
            let guide = Stroke::new(1.0, AudioTheme::with_alpha(self.theme.palette.accent, 90));
            painter.line_segment([Pos2::new(puck.x, rect.min.y), Pos2::new(puck.x, rect.max.y)], guide);
            painter.line_segment([Pos2::new(rect.min.x, puck.y), Pos2::new(rect.max.x, puck.y)], guide);

            self.theme.paint_glow_circle(painter, puck, 7.0, self.theme.palette.accent);
            painter.circle_filled(puck, 6.5, self.theme.palette.accent);
            painter.circle_stroke(puck, 6.5, Stroke::new(1.5, self.theme.palette.bg_base));
            self.theme.paint_screen_overlay(painter, rect);
        }

        response
    }
}

// ---------------- 4. RANGE / DUAL-THUMB SLIDER ----------------

pub struct RangeSlider<'a> {
    low: &'a mut f32,
    high: &'a mut f32,
    width: f32,
    theme: &'a AudioTheme,
}

impl<'a> RangeSlider<'a> {
    pub fn new(low: &'a mut f32, high: &'a mut f32, theme: &'a AudioTheme) -> Self {
        Self {
            low,
            high,
            width: 180.0,
            theme,
        }
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, mut response) = ui.allocate_exact_size(Vec2::new(self.width, 24.0), Sense::click_and_drag());
        let id = response.id.with("active_thumb");

        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                let t = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let pick = if (t - *self.low).abs() < (t - *self.high).abs() { 0u8 } else { 1u8 };
                ui.data_mut(|d| d.insert_temp(id, pick));
            }
        }

        if response.dragged() {
            if let Some(pos) = response.interact_pointer_pos() {
                let t = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
                let pick = ui.data_mut(|d| d.get_temp::<u8>(id).unwrap_or(0));
                if pick == 0 {
                    *self.low = t.min(*self.high - 0.02);
                } else {
                    *self.high = t.max(*self.low + 0.02);
                }
                response.mark_changed();
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let track = Rect::from_center_size(rect.center(), Vec2::new(rect.width() - 12.0, 6.0));
            self.theme.paint_recessed_rect(painter, track, 3.0);

            let x0 = track.min.x + *self.low * track.width();
            let x1 = track.min.x + *self.high * track.width();
            let span = Rect::from_min_max(Pos2::new(x0, track.min.y), Pos2::new(x1, track.max.y));
            painter.rect_filled(span, 2.0, self.theme.palette.accent);

            for x in [x0, x1] {
                let thumb = Rect::from_center_size(Pos2::new(x, track.center().y), Vec2::new(10.0, 18.0));
                self.theme.paint_raised_rect(painter, thumb, self.theme.palette.bg_panel, 2.0);
            }
        }

        response
    }
}
