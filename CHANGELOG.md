# Changelog

All notable changes to `egui_audio_ui` are documented in this file.

## [0.1.0] - 2026-11-27

### Added
- **Theme & Skin Engine (`theme`):** Three switchable skins (`Skin::Skeuomorphic`, `Skin::Hybrid`, `Skin::Flat`), `egui::Mesh` gradient/glow primitives, five built-in `ColorPalette` presets (`studio_cyan`, `vintage_amber`, `acid_emerald`, `bassmaker_slate`, `light_nordic`), adaptive light-mode shading, and `AudioTheme::apply_to_ctx`.
- **Continuous Controls (`controls`, `scale`, `interaction`):** `RotaryKnob`, vertical/horizontal `Fader`, `XyPad`, and `RangeSlider` with `ParamScale` (`Linear`, `Log`, `Decibel`, `Bipolar`), Shift fine-tune, scroll nudging, hover tooltips, and right-click context menus.
- **Buttons & Chrome (`buttons`, `chrome`):** `push_button`, `toggle_button`, `multi_switch`, `segmented_control`, `led_indicator`, `scrub_value_box` (with unit parsing, auto-focus, and Escape cancel), `styled_text_input`, `preset_bar` / `preset_bar_ex` (dropdown popup, A/B compare, Save, Init), `rack_module`, and `transport_bar`.
- **Meters & DSP Analysis (`meters`, `dsp_widgets`):** `level_meter`, `gain_reduction_meter`, `correlation_meter`, `oscilloscope`, `spectrum_analyzer`, `oled_screen`, `vu_meter`, `lufs_meter`, `compressor_curve`, `goniometer`, `wavetable_view`, `chromatic_tuner`, and `MeterBallistics`.
- **Interactive Editors & Pro Modules (`editors`, `pro`):** `waveform_editor` (cursor zoom & pan), `envelope_editor` (ADSR), `eq_curve_editor` (5 filter types, add/remove bands, live HUD), `step_sequencer`, `mini_keyboard`, `channel_strip`, `saturation_display`, `mod_matrix`, and `piano_roll` (drag, resize, loop brace, synced velocity lane).
- **Demos & Tests:** `showcase` and `synth_mixer` examples plus unit test suites across `scale`, `theme`, `editors`, `pro`, and `dsp_widgets`.
