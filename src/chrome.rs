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
    let id = ui.make_persistent_id(("scrub_box", unit, width.to_bits()));
    let editing = ui.data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false));

    if editing {
        let mut buf = ui.data_mut(|d| {
            d.get_temp::<String>(id.with("buf"))
                .unwrap_or_else(|| scale.format(*value, unit))
        });
        let resp = ui.add_sized(Vec2::new(width, 22.0), TextEdit::singleline(&mut buf));
        if resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Some(parsed) = scale.parse(&buf) {
                *value = parsed;
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
        let (rect, _) = ui.allocate_exact_size(Vec2::new(160.0, 24.0), Sense::hover());
        theme.paint_recessed_rect(ui.painter(), rect, 3.0);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            display,
            FontId::monospace(11.5),
            theme.palette.text_primary,
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
    let frame = egui::Frame::none()
        .fill(theme.palette.bg_panel)
        .stroke(Stroke::new(1.0, theme.palette.border))
        .rounding(theme.rounding(6.0))
        .inner_margin(egui::Margin::same(10.0));

    let resp = frame.show(ui, |ui| {
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
        ui.add_space(6.0);
        add_contents(ui);
    });

    if theme.skin == Skin::Skeuomorphic {
        let r = resp.response.rect;
        let painter = ui.painter();
        for c in [
            r.left_top() + Vec2::new(6.0, 6.0),
            r.right_top() + Vec2::new(-6.0, 6.0),
            r.left_bottom() + Vec2::new(6.0, -6.0),
            r.right_bottom() + Vec2::new(-6.0, -6.0),
        ] {
            painter.circle_filled(c, 2.5, AudioTheme::darken(theme.palette.bg_panel, 0.45));
            painter.line_segment([c - Vec2::new(1.5, 1.5), c + Vec2::new(1.5, 1.5)], Stroke::new(0.8, Color32::from_white_alpha(45)));
        }
    }
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
