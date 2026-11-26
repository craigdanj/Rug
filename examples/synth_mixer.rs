use eframe::egui;
use egui::Vec2;
use egui_audio_ui::*;

fn main() -> eframe::Result<()> {
    let opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1140.0, 740.0]),
        ..Default::default()
    };
    eframe::run_native(
        "egui_audio_ui — Synth & Mixer Demo",
        opts,
        Box::new(|_| Ok(Box::new(SynthMixerApp::default()))),
    )
}

struct SynthMixerApp {
    theme: AudioTheme,
    transport: TransportState,
    preset_idx: usize,
    preset_dirty: bool,
    slot_b: bool,
    saved_voice: (f32, f32, f32),
    // Synth voice controls
    cutoff: f32,
    resonance: f32,
    sub_level: f32,
    filter_slope: usize,
    voice_on: bool,
    adsr: AdsrState,
    sat: SaturationState,
    eq_bands: Vec<EqBand>,
    mod_routes: Vec<ModRoute>,
    selected_mod_src: usize,
    piano_roll: PianoRollState,
    // 4-channel mixer
    strips: Vec<ChannelStripState>,
    time: f32,
}

impl Default for SynthMixerApp {
    fn default() -> Self {
        Self {
            theme: AudioTheme::new(Skin::Skeuomorphic, ColorPalette::bassmaker_slate()),
            transport: TransportState {
                playing: true,
                recording: false,
                bpm: 126.0,
                bar_beat: "001:01:00".into(),
            },
            preset_idx: 0,
            preset_dirty: false,
            slot_b: false,
            saved_voice: (520.0, 0.55, 0.8),
            cutoff: 1450.0,
            resonance: 0.32,
            sub_level: 0.72,
            filter_slope: 1,
            voice_on: true,
            adsr: AdsrState {
                attack: 0.08,
                decay: 0.34,
                sustain: 0.58,
                release: 0.42,
            },
            sat: SaturationState::default(),
            eq_bands: vec![
                EqBand { freq_norm: 0.16, gain_norm: 0.42, q: 1.1, filter_type: EqFilterType::LowShelf },
                EqBand { freq_norm: 0.48, gain_norm: -0.28, q: 2.0, filter_type: EqFilterType::Peak },
                EqBand { freq_norm: 0.82, gain_norm: 0.30, q: 1.0, filter_type: EqFilterType::HighShelf },
            ],
            mod_routes: vec![
                ModRoute { source_idx: 0, dest_idx: 0, depth: 0.32 },
                ModRoute { source_idx: 1, dest_idx: 2, depth: 0.25 },
            ],
            selected_mod_src: 0,
            piano_roll: PianoRollState::default(),
            strips: vec![
                ChannelStripState::new("SUB", -2.5, 0.0),
                ChannelStripState::new("LEAD", -5.0, -0.22),
                ChannelStripState::new("PAD", -7.5, 0.28),
                ChannelStripState::new("MASTER", -0.5, 0.0),
            ],
            time: 0.0,
        }
    }
}

impl eframe::App for SynthMixerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.unstable_dt).min(0.05);
        if self.transport.playing {
            self.time += dt;
            let beats = self.time * (self.transport.bpm / 60.0);
            let bar = (beats as u32 / 4) + 1;
            let beat = (beats as u32 % 4) + 1;
            let sub = ((beats.fract()) * 100.0) as u32;
            self.transport.bar_beat = format!("{:03}:{:02}:{:02}", bar, beat, sub);
        }
        ctx.request_repaint();
        self.theme.apply_to_ctx(ctx);

        // Top global transport, preset bar, and theme switcher
        egui::TopBottomPanel::top("synth_mixer_header").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                transport_bar(ui, &mut self.transport, &self.theme);
                ui.separator();
                match preset_bar_ex(
                    ui,
                    &["Bass Under Attack", "Analog Reese", "Warm Tape Pluck", "Init Patch"],
                    &mut self.preset_idx,
                    self.preset_dirty,
                    &mut self.slot_b,
                    &self.theme,
                ) {
                    PresetAction::Changed(_) => self.preset_dirty = false,
                    PresetAction::ToggleAB => {
                        let cur = (self.cutoff, self.resonance, self.sub_level);
                        (self.cutoff, self.resonance, self.sub_level) = self.saved_voice;
                        self.saved_voice = cur;
                    }
                    PresetAction::Save => self.preset_dirty = false,
                    PresetAction::Init => {
                        self.cutoff = 1000.0;
                        self.resonance = 0.2;
                        self.sub_level = 0.5;
                        self.preset_dirty = false;
                    }
                    PresetAction::None => {}
                }
                ui.separator();
                for (skin, name) in [
                    (Skin::Skeuomorphic, "Skeuo"),
                    (Skin::Hybrid, "Hybrid"),
                    (Skin::Flat, "Flat"),
                ] {
                    if ui.selectable_label(self.theme.skin == skin, name).clicked() {
                        self.theme.skin = skin;
                    }
                }
                ui.separator();
                if ui.button("Slate").clicked() { self.theme.palette = ColorPalette::bassmaker_slate(); }
                if ui.button("Cyan").clicked() { self.theme.palette = ColorPalette::studio_cyan(); }
                if ui.button("Amber").clicked() { self.theme.palette = ColorPalette::vintage_amber(); }
                if ui.button("Nordic").clicked() { self.theme.palette = ColorPalette::light_nordic(); }
            });
        });

        let lfo1 = (self.time * 2.2).sin();
        let env2 = ((self.time * 1.5).sin() * 0.5 + 0.5).powf(2.0);
        let cutoff_mod: f32 = self
            .mod_routes
            .iter()
            .filter(|r| r.dest_idx == 0)
            .map(|r| (if r.source_idx == 0 { lfo1 } else { env2 }) * r.depth * 0.4)
            .sum();

        let fft_bins: Vec<f32> = (0..64)
            .map(|i| {
                let f = i as f32 / 64.0;
                let base = (1.0 - f).powf(1.4) * (0.45 + 0.35 * (self.time * 3.5 + f * 9.0).sin().abs());
                (base * self.sub_level.max(0.2)).clamp(0.0, 1.0)
            })
            .collect();
        let scope_samples: Vec<f32> = (0..256)
            .map(|i| {
                let x = (i as f32 * 0.12 + self.time * 9.0).sin() * 0.75;
                self.sat.mode.shape(x, self.sat.drive, self.sat.bias) * 0.85
            })
            .collect();

        let master_l = (0.58 + 0.32 * (self.time * 4.8).sin().abs()).clamp(0.0, 1.0);
        let master_r = (0.55 + 0.34 * (self.time * 5.1 + 0.5).sin().abs()).clamp(0.0, 1.0);

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Row 1: Wide Spectrum + Master EQ + Scope
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Master Spectrum & Scope", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            spectrum_analyzer(ui, &fft_bins, Vec2::new(340.0, 128.0), &self.theme);
                            oscilloscope(ui, &scope_samples, Vec2::new(210.0, 128.0), &self.theme);
                        });
                    });
                    rack_module(ui, "Master EQ", None, &self.theme, |ui| {
                        eq_curve_editor(ui, &mut self.eq_bands, Vec2::new(280.0, 128.0), &self.theme);
                    });
                });

                ui.add_space(10.0);

                // Row 2: Synth Voice + Tone/Saturation + Mod Matrix
                ui.horizontal_top(|ui| {
                    rack_module(ui, "Sub Voice & Filter", Some(&mut self.voice_on), &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            if RotaryKnob::new(
                                &mut self.cutoff,
                                1000.0,
                                ParamScale::Log { min: 20.0, max: 20000.0 },
                                "CUTOFF",
                                &self.theme,
                            )
                            .unit("Hz")
                            .modulation(cutoff_mod)
                            .show(ui)
                            .changed()
                            {
                                self.preset_dirty = true;
                            }
                            if RotaryKnob::new(
                                &mut self.resonance,
                                0.2,
                                ParamScale::Linear { min: 0.0, max: 1.0 },
                                "RESO",
                                &self.theme,
                            )
                            .show(ui)
                            .changed()
                            {
                                self.preset_dirty = true;
                            }
                            if RotaryKnob::new(
                                &mut self.sub_level,
                                0.7,
                                ParamScale::Linear { min: 0.0, max: 1.0 },
                                "SUB",
                                &self.theme,
                            )
                            .show(ui)
                            .changed()
                            {
                                self.preset_dirty = true;
                            }
                            multi_switch(ui, &mut self.filter_slope, &["12dB", "24dB", "36dB"], &self.theme);
                        });
                        ui.add_space(6.0);
                        envelope_editor(ui, &mut self.adsr, Vec2::new(265.0, 82.0), &self.theme);
                    });

                    rack_module(ui, "Tone / Saturation", None, &self.theme, |ui| {
                        saturation_display(ui, &mut self.sat, Vec2::new(165.0, 146.0), &self.theme);
                    });

                    rack_module(ui, "Modulation", None, &self.theme, |ui| {
                        mod_matrix(
                            ui,
                            &[("LFO 1", lfo1), ("ENV 2", env2), ("VEL", 0.8)],
                            &["CUTOFF", "RESO", "DRIVE", "SUB"],
                            &mut self.mod_routes,
                            &mut self.selected_mod_src,
                            &self.theme,
                        );
                        ui.add_space(8.0);
                        piano_roll(
                            ui,
                            &mut self.piano_roll,
                            self.time * (self.transport.bpm / 60.0),
                            Vec2::new(280.0, 118.0),
                            &self.theme,
                        );
                    });
                });

                ui.add_space(10.0);

                // Row 3: 4-Channel Mixer + Master Bus VU/LUFS Metering
                ui.horizontal_top(|ui| {
                    rack_module(ui, "4-Channel Mixer", None, &self.theme, |ui| {
                        ui.horizontal(|ui| {
                            for (i, strip) in self.strips.iter_mut().enumerate() {
                                let p = i as f32 * 0.85;
                                let l = if self.transport.playing {
                                    (0.45 + 0.42 * (self.time * 4.1 + p).sin().abs()).clamp(0.0, 1.0)
                                } else {
                                    0.0
                                };
                                let r = if self.transport.playing {
                                    (0.42 + 0.44 * (self.time * 4.4 + p + 0.4).sin().abs()).clamp(0.0, 1.0)
                                } else {
                                    0.0
                                };
                                channel_strip(ui, strip, l, r, &self.theme);
                            }
                        });
                    });

                    rack_module(ui, "Master Bus Metering", None, &self.theme, |ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                vu_meter(ui, master_l * 0.9, "VU L", Vec2::new(124.0, 74.0), &self.theme);
                                vu_meter(ui, master_r * 0.9, "VU R", Vec2::new(124.0, 74.0), &self.theme);
                            });
                            ui.add_space(8.0);
                            lufs_meter(
                                ui,
                                -14.2 + 2.8 * (self.time * 2.2).sin(),
                                -13.9 + 0.9 * (self.time * 0.7).sin(),
                                -0.6 + 0.5 * (self.time * 2.9).sin().abs(),
                                Vec2::new(256.0, 46.0),
                                &self.theme,
                            );
                            ui.add_space(8.0);
                            correlation_meter(
                                ui,
                                0.72 + 0.22 * (self.time * 1.4).sin(),
                                Vec2::new(256.0, 16.0),
                                &self.theme,
                            );
                        });
                    });
                });
            });
        });
    }
}
