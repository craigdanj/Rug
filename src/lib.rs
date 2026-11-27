//! `egui_audio_ui` — A custom `egui` widget suite for audio plugins, synthesizers,
//! and desktop DAW applications.
//!
//! # Features
//! - **Three switchable skins** ([`Skin::Skeuomorphic`], [`Skin::Hybrid`], [`Skin::Flat`])
//! - **Configurable [`ColorPalette`]** presets (`studio_cyan`, `vintage_amber`, `acid_emerald`,
//!   `bassmaker_slate`, `light_nordic`) with one-call [`AudioTheme::apply_to_ctx`] sync
//! - **Continuous controls** ([`RotaryKnob`], [`Fader`], [`XyPad`], [`RangeSlider`]) with
//!   audio-native scaling ([`ParamScale`]), Shift fine-tune, double-click reset, tooltips,
//!   and right-click context menus
//! - **Metering & DSP analysis** ([`level_meter`], [`vu_meter`], [`lufs_meter`],
//!   [`oscilloscope`], [`spectrum_analyzer`], [`oled_screen`], [`compressor_curve`],
//!   [`goniometer`], [`wavetable_view`], [`chromatic_tuner`], [`MeterBallistics`])
//! - **Interactive editors & pro modules** ([`waveform_editor`], [`envelope_editor`],
//!   [`eq_curve_editor`], [`step_sequencer`], [`piano_roll`], [`channel_strip`],
//!   [`saturation_display`], [`mod_matrix`])
//!
//! # Quick Start
//! ```no_run
//! use egui_audio_ui::prelude::*;
//!
//! let theme = AudioTheme::new(Skin::Skeuomorphic, ColorPalette::bassmaker_slate());
//! // Inside eframe::App::update:
//! // theme.apply_to_ctx(ctx);
//! ```

pub mod buttons;
pub mod chrome;
pub mod controls;
pub mod dsp_widgets;
pub mod editors;
pub mod interaction;
pub mod meters;
pub mod pro;
pub mod scale;
pub mod theme;

/// Convenient glob import for all commonly used widgets, state types, scales, and themes.
pub mod prelude {
    pub use crate::buttons::*;
    pub use crate::chrome::*;
    pub use crate::controls::*;
    pub use crate::dsp_widgets::*;
    pub use crate::editors::*;
    pub use crate::interaction::*;
    pub use crate::meters::*;
    pub use crate::pro::*;
    pub use crate::scale::*;
    pub use crate::theme::*;
}

pub use prelude::*;
