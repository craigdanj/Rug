//! Shared pointer/keyboard gesture handling for continuous audio controls.

use crate::scale::ParamScale;
use egui::{Response, Ui};

/// Drag sensitivity and axis settings for [`handle_continuous_interaction`].
pub struct DragConfig {
    /// Normalized delta applied per pixel of pointer travel.
    pub sensitivity: f32,
    /// Multiplier applied while holding `Shift` for precision fine-tuning.
    pub fine_multiplier: f32,
    /// When `true`, uses horizontal pointer delta (`+x`); otherwise vertical (`-y`).
    pub horizontal: bool,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.005,
            fine_multiplier: 0.15,
            horizontal: false,
        }
    }
}

/// Applies standard audio UX: drag, Shift fine-tune, double-click / Ctrl+click reset, scroll nudge.
pub fn handle_continuous_interaction(
    ui: &Ui,
    response: &Response,
    value: &mut f32,
    default_value: f32,
    scale: ParamScale,
    cfg: DragConfig,
) -> bool {
    let mut changed = false;
    let modifiers = ui.input(|i| i.modifiers);

    if response.double_clicked() || (response.clicked() && modifiers.command) {
        *value = default_value;
        return true;
    }

    if response.dragged() {
        let delta = response.drag_delta();
        let axis = if cfg.horizontal { delta.x } else { -delta.y };
        if axis.abs() > 0.0 {
            let mult = if modifiers.shift { cfg.fine_multiplier } else { 1.0 };
            let norm = (scale.normalize(*value) + axis * cfg.sensitivity * mult).clamp(0.0, 1.0);
            *value = scale.denormalize(norm);
            changed = true;
        }
    }

    if response.hovered() {
        let scroll = ui.input(|i| i.raw_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let step = if modifiers.shift { 0.005 } else { 0.025 };
            let norm = (scale.normalize(*value) + scroll.signum() * step).clamp(0.0, 1.0);
            *value = scale.denormalize(norm);
            changed = true;
        }
    }

    changed
}
