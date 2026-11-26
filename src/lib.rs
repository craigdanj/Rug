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
//! - **Metering & displays** ([`level_meter`], [`vu_meter`], [`lufs_meter`], [`oscilloscope`],
//!   [`spectrum_analyzer`], [`oled_screen`])
//! - **Interactive editors & pro modules** ([`waveform_editor`], [`envelope_editor`],
//!   [`eq_curve_editor`], [`step_sequencer`], [`piano_roll`], [`channel_strip`],
//!   [`saturation_display`], [`mod_matrix`])
//!
//! # Quick Start
//! ```no_run
//! use egui_audio_ui::{AudioTheme, ColorPalette, Skin};
//!
//! let theme = AudioTheme::new(Skin::Skeuomorphic, ColorPalette::bassmaker_slate());
//! // Inside eframe::App::update:
//! // theme.apply_to_ctx(ctx);
//! ```

pub mod buttons;
pub mod chrome;
pub mod controls;
pub mod editors;
pub mod interaction;
pub mod meters;
pub mod pro;
pub mod scale;
pub mod theme;

pub use buttons::*;
pub use chrome::*;
pub use controls::*;
pub use editors::*;
pub use interaction::*;
pub use meters::*;
pub use pro::*;
pub use scale::*;
pub use theme::*;
