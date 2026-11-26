# egui_audio_ui

A custom `egui` widget library for audio plugins and desktop audio applications, featuring **3 switchable skins** (`Skeuomorphic`, `Hybrid`, `Flat`) and a **fully configurable `ColorPalette`**.

## Running the Showcase

```bash
cargo run --example showcase
```

## Included Widgets

- **Continuous Controls:** `RotaryKnob`, `Fader` (vertical & horizontal), `XyPad`, `RangeSlider`
- **Buttons & Switches:** `push_button`, `toggle_button`, `multi_switch`, `segmented_control`, `led_indicator`
- **Meters & Screens:** `level_meter`, `gain_reduction_meter`, `correlation_meter`, `oscilloscope`, `spectrum_analyzer`, `oled_screen`, `vu_meter`, `lufs_meter`
- **Interactive Editors:** `waveform_editor`, `envelope_editor` (ADSR), `eq_curve_editor`, `step_sequencer`, `mini_keyboard`, `piano_roll`
- **Pro-Audio Modules:** `channel_strip`, `saturation_display`, `mod_matrix`
- **Chrome & Inputs:** `scrub_value_box`, `styled_text_input`, `preset_bar`, `rack_module`, `transport_bar`

## Customizing the Theme

```rust
use egui::Color32;
use egui_audio_ui::{AudioTheme, ColorPalette, Skin};

let mut palette = ColorPalette::studio_cyan();
palette.accent = Color32::from_rgb(255, 94, 0);
palette.bg_panel = Color32::from_rgb(24, 24, 28);

let theme = AudioTheme::new(Skin::Skeuomorphic, palette);
```
