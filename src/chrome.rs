use crate::{
    buttons::{push_button, toggle_button},
    interaction::{handle_continuous_interaction, DragConfig},
    scale::ParamScale,
    theme::{AudioTheme, Skin},
};
use egui::{Align2, Color32, FontId, Response, Sense, Stroke, TextEdit, Ui, Vec2};

// ---------------- 1. SCRUBBABLE VALUE BOX ----------------

pub fn scrub_value_box(
    ui: &mut Ui,
    value: &mut f32,
    default_value: f32,
    scale: ParamScale,
    unit: &str,
    width: f32,
    theme: &AudioTheme,
) -> Response {
    let id = ui.next_auto_id();
    let editing = ui.data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false));

    if editing {
        let mut buf = ui.data_mut(|d| {
            d.get_temp::<String>(id.with("buf"))
                .unwrap_or_else(|| scale.format(*value, unit))
        });
        let needs_focus = ui.data_mut(|d| d.get_temp::<bool>(id.with("focus")).unwrap_or(false));
        let mut resp = ui.add_sized(Vec2::new(width, 22.0), TextEdit::singleline(&mut buf));
        if needs_focus {
            resp.request_focus();
            ui.data_mut(|d| d.insert_temp(id.with("focus"), false));
            ui.data_mut(|d| d.insert_temp(id.with("buf"), buf));
        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            ui.data_mut(|d| d.insert_temp(id, false));
        } else if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(parsed) = scale.parse(&buf) {
                *value = parsed;
                resp.mark_changed();
            }
            ui.data_mut(|d| d.insert_temp(id, false));
        } else {
            ui.data_mut(|d| d.insert_temp(id.with("buf"), buf));
        }
        return resp;
    }

    let (rect, mut response) = ui.allocate_exact_size(Vec2::new(width, 22.0), Sense::click_and_drag());
    if response.double_clicked() {
        ui.data_mut(|d| {
            d.insert_temp(id, true);
            d.insert_temp(id.with("focus"), true);
            d.insert_temp(id.with("buf"), format!("{:.2}", *value));
        });
        return response;
    }
    if handle_continuous_interaction(ui, &response, value, default_value, scale, DragConfig::default()) {
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        theme.paint_recessed_rect(painter, rect, 3.0);
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            scale.format(*value, unit),
            FontId::monospace(11.0),
            if response.dragged() { theme.palette.accent } else { theme.palette.text_primary },
        );
    }
    response
}

// ---------------- 2. STYLED TEXT INPUT ----------------

pub fn styled_text_input(ui: &mut Ui, text: &mut String, hint: &str, width: f32, theme: &AudioTheme) -> Response {
    let frame = egui::Frame::none()
        .fill(theme.palette.bg_recessed)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .rounding(theme.rounding(3.0))
        .inner_margin(egui::Margin::symmetric(6.0, 4.0));
    frame
        .show(ui, |ui| {
            ui.add_sized(
                Vec2::new(width, 18.0),
                TextEdit::singleline(text)
                    .frame(false)
                    .hint_text(hint)
                    .text_color(theme.palette.text_primary),
            )
        })
        .inner
}

// ---------------- 3. PRESET SELECTOR BAR ----------------

pub fn preset_bar(
    ui: &mut Ui,
    presets: &[&str],
    current: &mut usize,
    dirty: bool,
    theme: &AudioTheme,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        if push_button(ui, "<", Vec2::new(24.0, 24.0), theme).clicked() && !presets.is_empty() {
            *current = (*current + presets.len() - 1) % presets.len();
            changed = true;
        }
        let name = presets.get(*current).copied().unwrap_or("Init");
        let display = if dirty { format!("{} *", name) } else { name.to_string() };
        let (rect, box_resp) = ui.allocate_exact_size(Vec2::new(160.0, 24.0), Sense::click());
        let popup_id = box_resp.id.with("preset_popup");
        if box_resp.clicked() && !presets.is_empty() {
            ui.memory_mut(|m| m.toggle_popup(popup_id));
        }

        theme.paint_recessed_rect(ui.painter(), rect, 3.0);
        if box_resp.hovered() {
            ui.painter().rect_stroke(
                rect,
                theme.rounding(3.0),
                Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.accent, 130)),
            );
        }
        ui.painter().text(
            egui::Pos2::new(rect.center().x - 6.0, rect.center().y),
            Align2::CENTER_CENTER,
            display,
            FontId::monospace(11.5),
            theme.palette.text_primary,
        );
        ui.painter().text(
            egui::Pos2::new(rect.max.x - 10.0, rect.center().y),
            Align2::CENTER_CENTER,
            "v",
            FontId::monospace(9.5),
            theme.palette.text_muted,
        );

        egui::popup::popup_below_widget(
            ui,
            popup_id,
            &box_resp,
            egui::PopupCloseBehavior::CloseOnClick,
            |ui| {
                ui.set_min_width(160.0);
                for (i, &preset) in presets.iter().enumerate() {
                    if ui.selectable_label(i == *current, preset).clicked() && i != *current {
                        *current = i;
                        changed = true;
                    }
                }
            },
        );

        if push_button(ui, ">", Vec2::new(24.0, 24.0), theme).clicked() && !presets.is_empty() {
            *current = (*current + 1) % presets.len();
            changed = true;
        }
    });
    changed
}

// ---------------- 4. RACK MODULE / SECTION FRAME ----------------

pub fn rack_module(
    ui: &mut Ui,
    title: &str,
    bypass: Option<&mut bool>,
    theme: &AudioTheme,
    add_contents: impl FnOnce(&mut Ui),
) {
    let is_skeuo = theme.skin == Skin::Skeuomorphic;
    let panel_rounding = if is_skeuo {
        egui::Rounding::same(12.0)
    } else {
        theme.rounding(6.0)
    };

    if is_skeuo {
        // Reserve background layer slot so gradient + soft ambient shadow sit behind widgets
        let bg_idx = ui.painter().add(egui::Shape::Noop);
        let frame = egui::Frame::none()
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::NONE)
            .rounding(panel_rounding)
            .inner_margin(egui::Margin::same(12.0));

        let resp = frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(title.to_uppercase())
                        .strong()
                        .size(11.5)
                        .color(theme.palette.text_primary),
                );
                if let Some(b) = bypass {
                    ui.add_space(8.0);
                    toggle_button(ui, b, "ON", Vec2::new(42.0, 18.0), None, theme);
                }
            });
            ui.add_space(8.0);
            add_contents(ui);
        });

        let r = resp.response.rect;
        let mut shapes = Vec::new();
        // Soft pillowed pod drop shadow
        shapes.push(egui::Shape::rect_filled(
            r.translate(Vec2::new(0.0, 5.0)),
            panel_rounding,
            Color32::from_black_alpha(55),
        ));
        shapes.push(egui::Shape::rect_filled(
            r.translate(Vec2::new(0.0, 2.0)),
            panel_rounding,
            Color32::from_black_alpha(85),
        ));
        // Machined matte aluminum gradient body
        let top_col = AudioTheme::lighten(theme.palette.bg_panel, 0.06);
        let bot_col = AudioTheme::darken(theme.palette.bg_panel, 0.08);
        let mut temp_mesh = egui::Mesh::default();
        let max_r = (r.width().min(r.height()) * 0.5).max(0.0);
        let cr = panel_rounding.nw.min(max_r);
        let mut perim = Vec::with_capacity(24);
        let corners = [
            (egui::Pos2::new(r.max.x - cr, r.min.y + cr), -std::f32::consts::FRAC_PI_2, 0.0),
            (egui::Pos2::new(r.max.x - cr, r.max.y - cr), 0.0, std::f32::consts::FRAC_PI_2),
            (egui::Pos2::new(r.min.x + cr, r.max.y - cr), std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
            (egui::Pos2::new(r.min.x + cr, r.min.y + cr), std::f32::consts::PI, std::f32::consts::PI * 1.5),
        ];
        for (c, a0, a1) in corners {
            for i in 0..=4 {
                let a = a0 + (a1 - a0) * (i as f32 / 4.0);
                perim.push(c + Vec2::angled(a) * cr);
            }
        }
        temp_mesh.vertices.push(egui::epaint::Vertex {
            pos: r.center(),
            uv: egui::epaint::WHITE_UV,
            color: AudioTheme::lerp_color(top_col, bot_col, 0.5),
        });
        for &p in &perim {
            let t = ((p.y - r.min.y) / r.height()).clamp(0.0, 1.0);
            temp_mesh.vertices.push(egui::epaint::Vertex {
                pos: p,
                uv: egui::epaint::WHITE_UV,
                color: AudioTheme::lerp_color(top_col, bot_col, t),
            });
        }
        let n = perim.len() as u32;
        for i in 0..n {
            temp_mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
        }
        shapes.push(egui::Shape::mesh(temp_mesh));
        // Outer machined border + top rim catch-light
        shapes.push(egui::Shape::rect_stroke(
            r,
            panel_rounding,
            Stroke::new(1.0, Color32::from_black_alpha(150)),
        ));
        shapes.push(egui::Shape::rect_stroke(
            r.shrink(1.0),
            egui::Rounding::same((cr - 1.0).max(0.0)),
            Stroke::new(1.0, Color32::from_white_alpha(26)),
        ));
        ui.painter().set(bg_idx, egui::Shape::Vec(shapes));
        return;
    }

    let frame = egui::Frame::none()
        .fill(theme.palette.bg_panel)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .rounding(panel_rounding)
        .inner_margin(egui::Margin::same(10.0));

    frame.show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(title.to_uppercase())
                    .strong()
                    .size(11.0)
                    .color(theme.palette.text_muted),
            );
            if let Some(b) = bypass {
                ui.add_space(8.0);
                toggle_button(ui, b, "ON", Vec2::new(42.0, 18.0), None, theme);
            }
        });
        ui.add_space(4.0);
        if theme.skin == Skin::Hybrid {
            let y = ui.cursor().min.y;
            let x0 = ui.max_rect().min.x;
            let x1 = ui.max_rect().max.x;
            ui.painter().line_segment(
                [egui::Pos2::new(x0, y), egui::Pos2::new(x1, y)],
                Stroke::new(1.0, AudioTheme::with_alpha(theme.palette.border, 110)),
            );
            ui.add_space(6.0);
        } else {
            ui.add_space(2.0);
        }
        add_contents(ui);
    });
}

// ---------------- 5. TRANSPORT BAR ----------------

pub struct TransportState {
    pub playing: bool,
    pub recording: bool,
    pub bpm: f32,
    pub bar_beat: String,
}

impl Default for TransportState {
    fn default() -> Self {
        Self { playing: false, recording: false, bpm: 124.0, bar_beat: "001:01:00".into() }
    }
}

pub fn transport_bar(ui: &mut Ui, state: &mut TransportState, theme: &AudioTheme) {
    ui.horizontal(|ui| {
        toggle_button(ui, &mut state.playing, "PLAY", Vec2::new(56.0, 24.0), Some(theme.palette.meter_low), theme);
        if push_button(ui, "STOP", Vec2::new(50.0, 24.0), theme).clicked() {
            state.playing = false;
            state.recording = false;
        }
        toggle_button(ui, &mut state.recording, "REC", Vec2::new(52.0, 24.0), Some(theme.palette.danger), theme);
        ui.add_space(8.0);
        scrub_value_box(
            ui,
            &mut state.bpm,
            120.0,
            ParamScale::Linear { min: 40.0, max: 240.0 },
            "BPM",
            74.0,
            theme,
        );
        let (rect, _) = ui.allocate_exact_size(Vec2::new(82.0, 24.0), Sense::hover());
        theme.paint_recessed_rect(ui.painter(), rect, 3.0);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            &state.bar_beat,
            FontId::monospace(11.5),
            theme.palette.accent,
        );
    });
}
