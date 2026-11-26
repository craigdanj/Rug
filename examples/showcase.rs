use eframe::egui;
use egui::Vec2;
use egui_audio_ui::*;

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions::default();
    eframe::run_native(
        "egui_audio_ui — Skin & Palette Showcase",
        opts,
        Box::new(|_| Ok(Box::new(AudioShowcase::default()))),
    )
}

struct AudioShowcase {
    theme: AudioTheme,
    cutoff: f32,
    resonance: f32,
    pan: f32,
    fader_db: f32,
    xy: (f32, f32),
    range: (f32, f32),
    bypass: bool,
    switch_idx: usize,
    seg_idx: usize,
    preset_idx: usize,
    search_text: String,
    transport: TransportState,
    wf_state: WaveformState,
    adsr: AdsrState,
    eq_bands: Vec<EqBand>,
    steps: Vec<f32>,
    active_note: Option<(u8, f32)>,
    time: f32,
}

impl Default for AudioShowcase {
    fn default() -> Self {
        Self {
            theme: AudioTheme::default(),
            cutoff: 1200.0,
            resonance: 0.35,
            pan: 0.0,
            fader_db: -6.0,
            xy: (0.5, 0.5),
            range: (0.2, 0.8),
            bypass: true,
            switch_idx: 1,
            seg_idx: 1,
            preset_idx: 0,
            search_text: String::new(),
            transport: TransportState::default(),
            wf_state: WaveformState::default(),
            adsr: AdsrState::default(),
            eq_bands: vec![
                EqBand { freq_norm: 0.2, gain_norm: 0.35, q: 1.4 },
                EqBand { freq_norm: 0.52, gain_norm: -0.4, q: 2.2 },
                EqBand { freq_norm: 0.8, gain_norm: 0.5, q: 1.1 },
            ],
            steps: vec![0.9, 0.0, 0.5, 0.0, 0.85, 0.0, 0.4, 0.6, 0.9, 0.0, 0.3, 0.0, 0.8, 0.4, 0.0, 0.7],
            active_note: None,
            time: 0.0,
        }
    }
}

impl eframe::App for AudioShowcase {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.time += ctx.input(|i| i.unstable_dt).min(0.05);
        ctx.request_repaint();

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = self.theme.palette.bg_base;
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Skin:");
                for (skin, name) in [
                    (Skin::Skeuomorphic, "Skeuomorphic"),
                    (Skin::Hybrid, "Hybrid"),
                    (Skin::Flat, "Flat"),
                ] {
                    if ui.selectable_label(self.theme.skin == skin, name).clicked() {
                        self.theme.skin = skin;
                    }
                }
                ui.separator();
                ui.label("Palette:");
                if ui.button("Cyan").clicked() { self.theme.palette = ColorPalette::studio_cyan(); }
                if ui.button("Amber").clicked() { self.theme.palette = ColorPalette::vintage_amber(); }
                if ui.button("Emerald").clicked() { self.theme.palette = ColorPalette::acid_emerald(); }
                if ui.button("Nordic").clicked() { self.theme.palette = ColorPalette::light_nordic(); }
                ui.separator();
                ui.label("Accent:");
                ui.color_edit_button_srgba(&mut self.theme.palette.accent);
                ui.label("Mod:");
                ui.color_edit_button_srgba(&mut self.theme.palette.accent_secondary);
                ui.label("Panel:");
                ui.color_edit_button_srgba(&mut self.theme.palette.bg_panel);
            });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                transport_bar(ui, &mut self.transport, &self.theme);
                ui.separator();
                preset_bar(ui, &["Warm Analog Pad", "Sub Bass 808", "Tape Delay"], &mut self.preset_idx, true, &self.theme);
                styled_text_input(ui, &mut self.search_text, "Search presets...", 130.0, &self.theme);
            });
        });

        let scope_samples: Vec<f32> = (0..256)
            .map(|i| ((i as f32 * 0.14 + self.time * 8.0).sin() * 0.7))
            .collect();
        let fft_bins: Vec<f32> = (0..48)
            .map(|i| ((1.0 - i as f32 / 48.0) * (0.5 + 0.4 * (self.time * 4.0 + i as f32 * 0.3).sin())).clamp(0.0, 1.0))
            .collect();
        let lvl_l = 0.55 + 0.35 * (self.time * 5.0).sin().abs();
        let lvl_r = 0.50 + 0.38 * (self.time * 5.3 + 0.6).sin().abs();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Continuous Controls", Some(&mut self.bypass), &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            RotaryKnob::new(&mut self.cutoff, 1000.0, ParamScale::Log { min: 20.0, max: 20000.0 }, "CUTOFF", &self.theme)
                                .unit("Hz")
                                .modulation(0.18 * (self.time * 2.0).sin())
                                .show(ui);
                            RotaryKnob::new(&mut self.resonance, 0.2, ParamScale::Linear { min: 0.0, max: 1.0 }, "RESO", &self.theme)
                                .show(ui);
                            RotaryKnob::new(&mut self.pan, 0.0, ParamScale::Bipolar { max_abs: 1.0 }, "PAN", &self.theme)
                                .show(ui);
                            Fader::vertical(&mut self.fader_db, 0.0, ParamScale::Decibel { min_db: -60.0, max_db: 6.0 }, "OUT", &self.theme)
                                .meter_level(lvl_l)
                                .show(ui);
                            XyPad::new(&mut self.xy.0, &mut self.xy.1, &self.theme).size(Vec2::splat(115.0)).show(ui);
                        });
                        ui.add_space(6.0);
                        RangeSlider::new(&mut self.range.0, &mut self.range.1, &self.theme).show(ui);
                    });

                    rack_module(ui, "Switches & Metering", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            multi_switch(ui, &mut self.switch_idx, &["12dB", "24dB", "36dB"], &self.theme);
                            ui.vertical(|ui| {
                                segmented_control(ui, &mut self.seg_idx, &["MONO", "STEREO", "M/S"], &self.theme);
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    led_indicator(ui, lvl_l > 0.85, self.theme.palette.danger, "CLIP", &self.theme);
                                    led_indicator(ui, true, self.theme.palette.accent, "SYNC", &self.theme);
                                });
                                ui.add_space(6.0);
                                correlation_meter(ui, 0.65 * (self.time * 1.5).sin(), Vec2::new(150.0, 14.0), &self.theme);
                            });
                            level_meter(ui, &[lvl_l, lvl_r], &[0.88, 0.84], Vec2::new(28.0, 120.0), &self.theme);
                            gain_reduction_meter(ui, 0.35 * (self.time * 3.0).sin().abs(), Vec2::new(14.0, 120.0), &self.theme);
                        });
                    });
                });

                ui.add_space(8.0);
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Screens & Telemetry", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            oled_screen(ui, "Cutoff", &ParamScale::Log { min: 20.0, max: 20000.0 }.format(self.cutoff, "Hz"), "LPF 24dB", Vec2::new(130.0, 90.0), &self.theme);
                            oscilloscope(ui, &scope_samples, Vec2::new(180.0, 90.0), &self.theme);
                            spectrum_analyzer(ui, &fft_bins, Vec2::new(180.0, 90.0), &self.theme);
                        });
                    });
                });

                ui.add_space(8.0);
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Waveform & ADSR", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            waveform_editor(ui, &scope_samples, &mut self.wf_state, Vec2::new(240.0, 100.0), &self.theme);
                            envelope_editor(ui, &mut self.adsr, Vec2::new(220.0, 100.0), &self.theme);
                        });
                    });
                    rack_module(ui, "EQ Curve", None, &self.theme, |ui| {
                        eq_curve_editor(ui, &mut self.eq_bands, Vec2::new(240.0, 100.0), &self.theme);
                    });
                });

                ui.add_space(8.0);
                rack_module(ui, "Sequencer & Keyboard", None, &self.theme, |ui| {
                    ui.horizontal(|ui| {
                        let step_idx = (self.time * 6.0) as usize;
                        step_sequencer(ui, &mut self.steps, step_idx, Vec2::new(380.0, 72.0), &self.theme);
                        mini_keyboard(ui, &mut self.active_note, Vec2::new(220.0, 72.0), &self.theme);
                    });
                });
            });
        });
    }
}
