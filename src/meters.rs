use crate::theme::AudioTheme;
use egui::{Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

// ---------------- 1. LEVEL METER (STEREO PEAK / RMS + PEAK HOLD) ----------------

pub fn level_meter(
    ui: &mut Ui,
    levels: &[f32],
    peak_hold: &[f32],
    size: Vec2,
    theme: &AudioTheme,
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) || levels.is_empty() {
        return;
    }
    let painter = ui.painter();
    theme.paint_recessed_rect(painter, rect, 3.0);

    let ch_count = levels.len();
    let gap = 2.0;
    let ch_w = (rect.width() - gap * (ch_count as f32 + 1.0)) / ch_count as f32;
    let segments = 24;

    for (ch, &lvl) in levels.iter().enumerate() {
        let x0 = rect.min.x + gap + ch as f32 * (ch_w + gap);
        let seg_h = (rect.height() - 4.0) / segments as f32;

        for s in 0..segments {
            let norm = (s + 1) as f32 / segments as f32;
            let lit = lvl >= norm - 0.02;
            let seg_y = rect.max.y - 2.0 - (s + 1) as f32 * seg_h;
            let seg_rect = Rect::from_min_size(Pos2::new(x0, seg_y), Vec2::new(ch_w, (seg_h - 1.2).max(1.0)));

            let col = if norm > 0.88 {
                theme.palette.meter_high
            } else if norm > 0.68 {
                theme.palette.meter_mid
            } else {
                theme.palette.meter_low
            };
            let fill = if lit { col } else { AudioTheme::with_alpha(col, 24) };
            painter.rect_filled(seg_rect, 1.0, fill);
        }

        if let Some(&pk) = peak_hold.get(ch) {
            if pk > 0.01 {
                let py = rect.max.y - 2.0 - pk.clamp(0.0, 1.0) * (rect.height() - 4.0);
                let col = if pk > 0.95 { theme.palette.danger } else { theme.palette.text_primary };
                painter.line_segment([Pos2::new(x0, py), Pos2::new(x0 + ch_w, py)], Stroke::new(2.0, col));
            }
        }
    }
    theme.paint_screen_overlay(painter, rect);
}

// ---------------- 2. GAIN REDUCTION METER ----------------

pub fn gain_reduction_meter(ui: &mut Ui, reduction_norm: f32, size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 3.0);
        let h = (rect.height() - 4.0) * reduction_norm.clamp(0.0, 1.0);
        let bar = Rect::from_min_size(
            Pos2::new(rect.min.x + 2.0, rect.min.y + 2.0),
            Vec2::new(rect.width() - 4.0, h),
        );
        painter.rect_filled(bar, 1.5, theme.palette.accent_secondary);
    }
}

// ---------------- 3. CORRELATION / PHASE METER ----------------

pub fn correlation_meter(ui: &mut Ui, correlation: f32, size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 3.0);
        let cx = rect.center().x;
        let x = cx + correlation.clamp(-1.0, 1.0) * (rect.width() * 0.45);
        let (x0, x1) = if x >= cx { (cx, x) } else { (x, cx) };
        let col = if correlation < 0.0 { theme.palette.danger } else { theme.palette.accent };
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(x0, rect.min.y + 3.0), Pos2::new(x1, rect.max.y - 3.0)),
            1.0,
            col,
        );
        painter.line_segment([Pos2::new(cx, rect.min.y), Pos2::new(cx, rect.max.y)], Stroke::new(1.0, theme.palette.text_muted));
    }
}

// ---------------- 4. OSCILLOSCOPE ----------------

pub fn oscilloscope(ui: &mut Ui, samples: &[f32], size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    theme.paint_recessed_rect(painter, rect, 4.0);

    painter.line_segment(
        [Pos2::new(rect.min.x, rect.center().y), Pos2::new(rect.max.x, rect.center().y)],
        Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.border, 80)),
    );

    if samples.len() >= 2 {
        let start = samples
            .windows(2)
            .position(|w| w[0] <= 0.0 && w[1] > 0.0)
            .unwrap_or(0);
        let view = &samples[start..];
        let count = view.len().min(256);
        let pts: Vec<Pos2> = (0..count)
            .map(|i| {
                let x = rect.min.x + (i as f32 / (count - 1).max(1) as f32) * rect.width();
                let y = rect.center().y - view[i].clamp(-1.0, 1.0) * (rect.height() * 0.44);
                Pos2::new(x, y)
            })
            .collect();
        if matches!(theme.skin, crate::theme::Skin::Hybrid | crate::theme::Skin::Skeuomorphic) {
            painter.add(Shape::line(
                pts.clone(),
                Stroke::new(4.5, AudioTheme::with_alpha(theme.palette.accent, 42)),
            ));
        }
        painter.add(Shape::line(pts, Stroke::new(1.8, theme.palette.accent)));
    }
    theme.paint_screen_overlay(painter, rect);
}

// ---------------- 5. SPECTRUM ANALYZER (FFT) ----------------

pub fn spectrum_analyzer(ui: &mut Ui, bins_norm: &[f32], size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    theme.paint_recessed_rect(painter, rect, 4.0);

    if bins_norm.len() >= 2 {
        let mut pts = Vec::with_capacity(bins_norm.len());
        for (i, &v) in bins_norm.iter().enumerate() {
            let t = ((i + 1) as f32).ln() / (bins_norm.len() as f32).ln();
            let x = rect.min.x + t * rect.width();
            let y = rect.max.y - v.clamp(0.0, 1.0) * (rect.height() - 6.0) - 2.0;
            pts.push(Pos2::new(x, y));
        }
        if matches!(theme.skin, crate::theme::Skin::Hybrid | crate::theme::Skin::Skeuomorphic) {
            let mut mesh = egui::Mesh::default();
            let is_skeuo = theme.skin == crate::theme::Skin::Skeuomorphic;
            let bot_col = AudioTheme::with_alpha(theme.palette.accent, 8);
            let n = (pts.len() - 1).max(1) as f32;
            for (i, &p) in pts.iter().enumerate() {
                let top_col = if is_skeuo {
                    let blend = AudioTheme::lerp_color(theme.palette.accent_secondary, theme.palette.accent, i as f32 / n);
                    AudioTheme::with_alpha(blend, 115)
                } else {
                    AudioTheme::with_alpha(theme.palette.accent, 75)
                };
                mesh.vertices.push(egui::epaint::Vertex { pos: p, uv: egui::epaint::WHITE_UV, color: top_col });
                mesh.vertices.push(egui::epaint::Vertex { pos: Pos2::new(p.x, rect.max.y - 1.0), uv: egui::epaint::WHITE_UV, color: bot_col });
                if i > 0 {
                    let base = ((i - 1) * 2) as u32;
                    mesh.add_triangle(base, base + 1, base + 2);
                    mesh.add_triangle(base + 1, base + 3, base + 2);
                }
            }
            painter.add(Shape::mesh(mesh));
            painter.add(Shape::line(pts.clone(), Stroke::new(4.2, AudioTheme::with_alpha(theme.palette.accent, 42))));
        } else {
            for p in &pts {
                painter.line_segment(
                    [*p, Pos2::new(p.x, rect.max.y - 1.0)],
                    Stroke::new(2.0, AudioTheme::with_alpha(theme.palette.accent, 35)),
                );
            }
        }
        painter.add(Shape::line(pts, Stroke::new(1.6, theme.palette.accent)));
    }
    theme.paint_screen_overlay(painter, rect);
}

// ---------------- 6. OLED / LCD INFO SCREEN ----------------

pub fn oled_screen(ui: &mut Ui, title: &str, value_text: &str, subtitle: &str, size: Vec2, theme: &AudioTheme) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 4.0);
        painter.text(
            Pos2::new(rect.min.x + 8.0, rect.min.y + 10.0),
            Align2::LEFT_CENTER,
            title.to_uppercase(),
            FontId::monospace(9.5),
            theme.palette.text_muted,
        );
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            value_text,
            FontId::monospace(16.0),
            theme.palette.accent,
        );
        painter.text(
            Pos2::new(rect.max.x - 8.0, rect.max.y - 9.0),
            Align2::RIGHT_CENTER,
            subtitle,
            FontId::monospace(9.0),
            theme.palette.text_muted,
        );
        theme.paint_screen_overlay(painter, rect);
    }
}
