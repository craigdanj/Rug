//! Momentary push buttons, latching toggle switches, multi-position selectors,
//! segmented controls, and status LED indicators.

use crate::theme::{AudioTheme, Skin};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

// ---------------- 1. PUSH / MOMENTARY BUTTON ----------------

/// Tactile momentary push button with hover highlight and pressed travel offset.
pub fn push_button(ui: &mut Ui, label: &str, size: Vec2, theme: &AudioTheme) -> Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let pressed = response.is_pointer_button_down_on();
        let base = if pressed {
            AudioTheme::darken(theme.palette.bg_panel, 0.2)
        } else if response.hovered() {
            AudioTheme::lighten(theme.palette.bg_panel, 0.08)
        } else {
            theme.palette.bg_panel
        };
        let draw_rect = if pressed { rect.translate(Vec2::new(0.0, 1.0)) } else { rect };
        theme.paint_raised_rect(painter, draw_rect, base, 4.0);
        painter.text(
            draw_rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(12.0),
            if pressed { theme.palette.accent } else { theme.palette.text_primary },
        );
    }
    response
}

// ---------------- 2. TOGGLE / LATCH BUTTON ----------------

/// Latching toggle button with an integrated glowing LED indicator.
///
/// Pass `active_color: None` to default to `theme.palette.accent`.
pub fn toggle_button(
    ui: &mut Ui,
    active: &mut bool,
    label: &str,
    size: Vec2,
    active_color: Option<Color32>,
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(size, Sense::click());
    if response.clicked() {
        *active = !*active;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let col = active_color.unwrap_or(theme.palette.accent);
        let bg = if *active {
            AudioTheme::with_alpha(col, if theme.skin == Skin::Flat { 45 } else { 32 })
        } else {
            theme.palette.bg_panel
        };
        theme.paint_raised_rect(painter, rect, bg, 4.0);
        if *active {
            painter.rect_stroke(rect, theme.rounding(4.0), Stroke::new(1.2, col));
        }

        let led_pos = Pos2::new(rect.min.x + 10.0, rect.center().y);
        if *active {
            theme.paint_glow_circle(painter, led_pos, 3.0, col);
            painter.circle_filled(led_pos, 3.0, col);
        } else {
            painter.circle_filled(led_pos, 2.5, theme.palette.bg_recessed);
        }

        painter.text(
            Pos2::new(rect.center().x + 5.0, rect.center().y),
            Align2::CENTER_CENTER,
            label,
            FontId::proportional(11.5),
            if *active { col } else { theme.palette.text_muted },
        );
    }
    response
}

// ---------------- 3. MULTI-POSITION HARDWARE SWITCH ----------------

/// Vertical multi-position lever switch that cycles through `options` on each click.
pub fn multi_switch(
    ui: &mut Ui,
    index: &mut usize,
    options: &[&str],
    theme: &AudioTheme,
) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(44.0, 56.0), Sense::click());
    if response.clicked() && !options.is_empty() {
        *index = (*index + 1) % options.len();
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let well = Rect::from_center_size(Pos2::new(rect.center().x, rect.center().y - 6.0), Vec2::new(18.0, 34.0));
        theme.paint_recessed_rect(painter, well, 9.0);

        let steps = options.len().max(2) - 1;
        let t = (*index as f32 / steps as f32).clamp(0.0, 1.0);
        let lever_y = well.min.y + 7.0 + t * (well.height() - 14.0);
        let lever_pos = Pos2::new(well.center().x, lever_y);

        match theme.skin {
            Skin::Skeuomorphic => {
                painter.circle_filled(lever_pos + Vec2::new(0.0, 2.0), 7.5, Color32::from_black_alpha(120));
                painter.circle_filled(lever_pos, 7.5, AudioTheme::lighten(theme.palette.bg_panel, 0.35));
                painter.circle_filled(lever_pos - Vec2::new(1.5, 1.5), 3.0, Color32::from_white_alpha(80));
            }
            Skin::Hybrid => {
                painter.circle_filled(lever_pos, 6.5, theme.palette.accent);
                painter.circle_stroke(lever_pos, 6.5, Stroke::new(1.0, theme.palette.bg_base));
            }
            Skin::Flat => {
                let knob = Rect::from_center_size(lever_pos, Vec2::new(12.0, 10.0));
                painter.rect_filled(knob, 1.0, theme.palette.accent);
            }
        }

        if let Some(lbl) = options.get(*index) {
            painter.text(
                Pos2::new(rect.center().x, rect.max.y - 6.0),
                Align2::CENTER_CENTER,
                *lbl,
                FontId::monospace(10.0),
                theme.palette.text_primary,
            );
        }
    }
    response
}

// ---------------- 4. SEGMENTED CONTROL ----------------

/// Horizontal mutually-exclusive mode selector bar (e.g., `MONO` / `STEREO` / `M/S`).
pub fn segmented_control(
    ui: &mut Ui,
    selected: &mut usize,
    labels: &[&str],
    theme: &AudioTheme,
) -> Response {
    let seg_w = 54.0;
    let total = Vec2::new(seg_w * labels.len() as f32, 26.0);
    let (rect, mut response) = ui.allocate_exact_size(total, Sense::click());

    if response.clicked() {
        if let Some(pos) = response.interact_pointer_pos() {
            let idx = ((pos.x - rect.min.x) / seg_w).floor() as usize;
            if idx < labels.len() && idx != *selected {
                *selected = idx;
                response.mark_changed();
            }
        }
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);
        for (i, lbl) in labels.iter().enumerate() {
            let seg = Rect::from_min_size(
                Pos2::new(rect.min.x + i as f32 * seg_w, rect.min.y),
                Vec2::new(seg_w, rect.height()),
            );
            let is_sel = i == *selected;
            if is_sel {
                theme.paint_raised_rect(painter, seg.shrink(2.0), theme.palette.accent, 3.0);
            }
            painter.text(
                seg.center(),
                Align2::CENTER_CENTER,
                *lbl,
                FontId::proportional(11.0),
                if is_sel { theme.palette.bg_base } else { theme.palette.text_muted },
            );
        }
    }
    response
}

// ---------------- 5. LED INDICATOR ----------------

/// Non-interactive glowing status LED with a monospace caption (e.g., `CLIP`, `SYNC`).
pub fn led_indicator(ui: &mut Ui, lit: bool, color: Color32, label: &str, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(42.0, 20.0), Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let center = Pos2::new(rect.min.x + 8.0, rect.center().y);
        if lit {
            theme.paint_glow_circle(painter, center, 4.5, color);
            painter.circle_filled(center, 4.5, color);
            if theme.skin == Skin::Skeuomorphic {
                painter.circle_filled(center - Vec2::new(1.2, 1.2), 1.5, Color32::from_white_alpha(170));
            }
        } else {
            painter.circle_filled(center, 4.5, theme.palette.bg_recessed);
            painter.circle_stroke(center, 4.5, Stroke::new(1.0, theme.palette.border));
        }
        painter.text(
            Pos2::new(rect.min.x + 18.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::monospace(10.0),
            theme.palette.text_muted,
        );
    }
}
