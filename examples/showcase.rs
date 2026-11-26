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
    preset_dirty: bool,
    slot_b_active: bool,
    saved_slot: (f32, f32, f32),
    search_text: String,
    transport: TransportState,
    wf_state: WaveformState,
    adsr: AdsrState,
    eq_bands: Vec<EqBand>,
    steps: Vec<f32>,
    active_note: Option<(u8, f32)>,
    strips: Vec<ChannelStripState>,
    sat_state: SaturationState,
    mod_routes: Vec<ModRoute>,
    selected_mod_src: usize,
    piano_roll: PianoRollState,
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
            preset_dirty: true,
            slot_b_active: false,
            saved_slot: (480.0, 0.65, -0.25),
            search_text: String::new(),
            transport: TransportState::default(),
            wf_state: WaveformState::default(),
            adsr: AdsrState::default(),
            eq_bands: vec![
                EqBand { freq_norm: 0.18, gain_norm: 0.35, q: 1.2, filter_type: EqFilterType::LowShelf },
                EqBand { freq_norm: 0.52, gain_norm: -0.4, q: 2.2, filter_type: EqFilterType::Peak },
                EqBand { freq_norm: 0.82, gain_norm: 0.45, q: 1.1, filter_type: EqFilterType::HighShelf },
            ],
            steps: vec![0.9, 0.0, 0.5, 0.0, 0.85, 0.0, 0.4, 0.6, 0.9, 0.0, 0.3, 0.0, 0.8, 0.4, 0.0, 0.7],
            active_note: None,
            strips: vec![
                ChannelStripState::new("BASS", -3.5, -0.1),
                ChannelStripState::new("LEAD", -6.0, 0.25),
            ],
            sat_state: SaturationState::default(),
            mod_routes: vec![
                ModRoute { source_idx: 0, dest_idx: 0, depth: 0.35 },
                ModRoute { source_idx: 1, dest_idx: 1, depth: -0.25 },
            ],
            selected_mod_src: 0,
            piano_roll: PianoRollState::default(),
            time: 0.0,
        }
    }
}

impl eframe::App for AudioShowcase {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.time += ctx.input(|i| i.unstable_dt).min(0.05);
        ctx.request_repaint();

        self.theme.apply_to_ctx(ctx);

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
                if ui.button("Slate").clicked() { self.theme.palette = ColorPalette::bassmaker_slate(); }
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
                match preset_bar_ex(
                    ui,
                    &["Warm Analog Pad", "Sub Bass 808", "Tape Delay"],
                    &mut self.preset_idx,
                    self.preset_dirty,
                    &mut self.slot_b_active,
                    &self.theme,
                ) {
                    PresetAction::Changed(_) => self.preset_dirty = false,
                    PresetAction::ToggleAB => {
                        let cur = (self.cutoff, self.resonance, self.pan);
                        (self.cutoff, self.resonance, self.pan) = self.saved_slot;
                        self.saved_slot = cur;
                    }
                    PresetAction::Save => self.preset_dirty = false,
                    PresetAction::Init => {
                        self.cutoff = 1000.0;
                        self.resonance = 0.2;
                        self.pan = 0.0;
                        self.fader_db = 0.0;
                        self.preset_dirty = false;
                    }
                    PresetAction::None => {}
                }
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
        let lfo1 = (self.time * 2.0).sin();
        let env2 = ((self.time * 1.4).sin() * 0.5 + 0.5).powf(2.0);
        let cutoff_mod: f32 = self
            .mod_routes
            .iter()
            .filter(|r| r.dest_idx == 0)
            .map(|r| {
                let src = match r.source_idx {
                    0 => lfo1,
                    1 => env2,
                    2 => 0.75,
                    _ => 0.5,
                };
                src * r.depth * 0.4
            })
            .sum();

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Continuous Controls", Some(&mut self.bypass), &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            RotaryKnob::new(&mut self.cutoff, 1000.0, ParamScale::Log { min: 20.0, max: 20000.0 }, "CUTOFF", &self.theme)
                                .unit("Hz")
                                .modulation(cutoff_mod)
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
                    rack_module(ui, "VU & Loudness", None, &self.theme, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                vu_meter(ui, lvl_l * 0.92, "VU L", Vec2::new(105.0, 62.0), &self.theme);
                                vu_meter(ui, lvl_r * 0.92, "VU R", Vec2::new(105.0, 62.0), &self.theme);
                            });
                            ui.add_space(4.0);
                            lufs_meter(
                                ui,
                                -14.5 + 3.2 * (self.time * 2.4).sin(),
                                -13.8 + 1.1 * (self.time * 0.8).sin(),
                                -0.8 + 0.7 * (self.time * 3.1).sin().abs(),
                                Vec2::new(218.0, 42.0),
                                &self.theme,
                            );
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
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Mixer Strips", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            for (i, strip) in self.strips.iter_mut().enumerate() {
                                let phase = i as f32 * 0.9;
                                let sl = (0.45 + 0.4 * (self.time * 4.2 + phase).sin().abs()).clamp(0.0, 1.0);
                                let sr = (0.42 + 0.42 * (self.time * 4.5 + phase + 0.5).sin().abs()).clamp(0.0, 1.0);
                                channel_strip(ui, strip, sl, sr, &self.theme);
                            }
                        });
                    });
                    rack_module(ui, "Saturation", None, &self.theme, |ui| {
                        saturation_display(ui, &mut self.sat_state, Vec2::new(150.0, 118.0), &self.theme);
                    });
                    rack_module(ui, "Modulation Matrix", None, &self.theme, |ui| {
                        mod_matrix(
                            ui,
                            &[("LFO 1", lfo1), ("ENV 2", env2), ("VEL", 0.75), ("MAC", 0.5)],
                            &["CUTOFF", "RESO", "DRIVE", "PAN"],
                            &mut self.mod_routes,
                            &mut self.selected_mod_src,
                            &self.theme,
                        );
                    });
                });

                ui.add_space(8.0);
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Sequencer & Keyboard", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            let step_idx = (self.time * 6.0) as usize;
                            step_sequencer(ui, &mut self.steps, step_idx, Vec2::new(260.0, 92.0), &self.theme);
                            mini_keyboard(ui, &mut self.active_note, Vec2::new(180.0, 92.0), &self.theme);
                        });
                    });
                    rack_module(ui, "Piano Roll", None, &self.theme, |ui| {
                        piano_roll(ui, &mut self.piano_roll, self.time * 1.5, Vec2::new(300.0, 136.0), &self.theme);
                    });
                });
            });
        });
    }
}
