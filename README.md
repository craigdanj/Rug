# egui_audio_ui

A custom `egui` (0.29) widget suite for audio plugins, synthesizers, and desktop DAW applications—featuring **3 switchable skins** (`Skeuomorphic`, `Hybrid`, `Flat`) and a **fully configurable `ColorPalette`**.

## Running the Demos, Tests & Docs

```bash
# Full widget, skin, and palette showcase
cargo run --example showcase

# Focused synth voice + 4-channel mixer console demo
cargo run --example synth_mixer

# Run unit tests
cargo test

# Generate and open API rustdocs
cargo doc --no-deps --open
```

## Included Widgets

- **Continuous Controls (`controls`):** `RotaryKnob`, `Fader` (vertical & horizontal), `XyPad`, `RangeSlider`
- **Buttons & Switches (`buttons`):** `push_button`, `toggle_button`, `multi_switch`, `segmented_control`, `led_indicator`
- **Meters & Displays (`meters`):** `level_meter`, `gain_reduction_meter`, `correlation_meter`, `oscilloscope`, `spectrum_analyzer`, `oled_screen`, `vu_meter`, `lufs_meter`
- **Interactive Editors (`editors`):** `waveform_editor`, `envelope_editor` (ADSR), `eq_curve_editor`, `step_sequencer`, `mini_keyboard`
- **Pro-Audio Modules (`pro`):** `channel_strip`, `saturation_display`, `mod_matrix`, `piano_roll`
- **Chrome & Inputs (`chrome`):** `scrub_value_box`, `styled_text_input`, `preset_bar`, `preset_bar_ex`, `rack_module`, `transport_bar`

## Skins & Palettes

| Skin | Aesthetic |
| :--- | :--- |
| `Skin::Skeuomorphic` | Soft-machined matte anodized pods, sculpted dials with top-arc specular rim, pill-well faders, and ambient bottom-up OLED screen glow. |
| `Skin::Hybrid` | Modern pro-audio plugin look with subtle vertical gradients, dial tick marks, translucent curve fills, and restrained accent glow. |
| `Skin::Flat` | Minimal flat vector surfaces with crisp 1px borders and zero glow or drop shadows. |

Built-in palettes: `ColorPalette::studio_cyan()`, `ColorPalette::vintage_amber()`, `ColorPalette::acid_emerald()`, `ColorPalette::bassmaker_slate()`, and `ColorPalette::light_nordic()`.

Call `theme.apply_to_ctx(ctx)` once per frame to synchronize native `egui` popups, context menus, tooltips, and scrollbars with your active `AudioTheme`.

## Interaction Shortcuts

| Widget | Gestures |
| :--- | :--- |
| `RotaryKnob` / `Fader` | Drag to adjust · `Shift`+drag for fine-tune · Scroll to nudge · Double-click or `Cmd/Ctrl`+click to reset · Right-click for context menu (`Reset`, `Center`, `Copy value`). |
| `scrub_value_box` | Drag vertically to scrub · Double-click to type exact values (`1.2k`, `-6dB`, `-inf`) · `Enter` to commit · `Esc` to cancel. |
| `XyPad` | Drag to move · Hold `Shift` to lock Y / `Alt` to lock X · Double-click to center `(0.5, 0.5)`. |
| `waveform_editor` | Scroll wheel to zoom (`1x–16x`) at cursor · `Shift`+drag (or middle-drag) to pan · Drag loop edges or playhead · Double-click to reset zoom. |
| `eq_curve_editor` | Drag band node for freq/gain · Scroll over band for Q · Double-click empty space to add band / node to remove · Right-click to switch filter type (`Peak`, `LowShelf`, `HighShelf`, `LowPass`, `HighPass`). |
| `piano_roll` | Click empty cell to add note · Drag note body to move pitch/time · Drag right edge to resize length · Double-click note to delete · Drag top loop brace handles · Click/drag bottom `VEL` lane to edit velocity. |
| `saturation_display` | Click header or `<`/`>` to cycle `TUBE` / `TAPE` / `SOFT` / `FOLD` · Drag vertically for `DRV`, horizontally for `BIAS` · Double-click to reset. |

## Quick Usage Example

```rust
use egui::Color32;
use egui_audio_ui::*;

let mut palette = ColorPalette::bassmaker_slate();
palette.accent = Color32::from_rgb(249, 138, 66);
let theme = AudioTheme::new(Skin::Skeuomorphic, palette);

// Inside eframe::App::update(ctx, frame):
theme.apply_to_ctx(ctx);

rack_module(ui, "Filter", Some(&mut bypass), &theme, |ui| {
    RotaryKnob::new(
        &mut cutoff,
        1000.0,
        ParamScale::Log { min: 20.0, max: 20000.0 },
        "CUTOFF",
        &theme,
    )
    .unit("Hz")
    .modulation(0.15)
    .show(ui);
});
```
