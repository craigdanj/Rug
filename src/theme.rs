use egui::{Color32, Pos2, Rect, Rounding, Shape, Stroke, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skin {
    Skeuomorphic,
    Hybrid,
    Flat,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorPalette {
    pub bg_base: Color32,
    pub bg_panel: Color32,
    pub bg_recessed: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_secondary: Color32,
    pub meter_low: Color32,
    pub meter_mid: Color32,
    pub meter_high: Color32,
    pub danger: Color32,
}

impl ColorPalette {
    pub fn studio_cyan() -> Self {
        Self {
            bg_base: Color32::from_rgb(16, 18, 22),
            bg_panel: Color32::from_rgb(28, 31, 38),
            bg_recessed: Color32::from_rgb(11, 13, 16),
            border: Color32::from_rgb(56, 62, 74),
            text_primary: Color32::from_rgb(232, 236, 242),
            text_muted: Color32::from_rgb(138, 146, 160),
            accent: Color32::from_rgb(0, 212, 255),
            accent_secondary: Color32::from_rgb(168, 85, 247),
            meter_low: Color32::from_rgb(34, 197, 94),
            meter_mid: Color32::from_rgb(250, 204, 21),
            meter_high: Color32::from_rgb(239, 68, 68),
            danger: Color32::from_rgb(239, 68, 68),
        }
    }

    pub fn vintage_amber() -> Self {
        Self {
            bg_base: Color32::from_rgb(22, 19, 16),
            bg_panel: Color32::from_rgb(38, 33, 28),
            bg_recessed: Color32::from_rgb(15, 13, 11),
            border: Color32::from_rgb(78, 67, 56),
            text_primary: Color32::from_rgb(245, 236, 220),
            text_muted: Color32::from_rgb(164, 148, 128),
            accent: Color32::from_rgb(245, 158, 11),
            accent_secondary: Color32::from_rgb(56, 189, 248),
            ..Self::studio_cyan()
        }
    }

    pub fn acid_emerald() -> Self {
        Self {
            accent: Color32::from_rgb(16, 235, 145),
            accent_secondary: Color32::from_rgb(250, 204, 21),
            ..Self::studio_cyan()
        }
    }

    pub fn light_nordic() -> Self {
        Self {
            bg_base: Color32::from_rgb(226, 230, 236),
            bg_panel: Color32::from_rgb(242, 244, 248),
            bg_recessed: Color32::from_rgb(208, 214, 224),
            border: Color32::from_rgb(174, 182, 196),
            text_primary: Color32::from_rgb(24, 28, 36),
            text_muted: Color32::from_rgb(96, 106, 122),
            accent: Color32::from_rgb(14, 116, 214),
            accent_secondary: Color32::from_rgb(217, 70, 239),
            ..Self::studio_cyan()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioTheme {
    pub skin: Skin,
    pub palette: ColorPalette,
}

impl Default for AudioTheme {
    fn default() -> Self {
        Self {
            skin: Skin::Hybrid,
            palette: ColorPalette::studio_cyan(),
        }
    }
}

impl AudioTheme {
    pub fn new(skin: Skin, palette: ColorPalette) -> Self {
        Self { skin, palette }
    }

    pub fn rounding(&self, base: f32) -> Rounding {
        match self.skin {
            Skin::Skeuomorphic => Rounding::same(base + 2.0),
            Skin::Hybrid => Rounding::same(base),
            Skin::Flat => Rounding::same((base * 0.35).min(2.0)),
        }
    }

    pub fn lighten(c: Color32, amt: f32) -> Color32 {
        let mix = |ch: u8| (ch as f32 + (255.0 - ch as f32) * amt).clamp(0.0, 255.0) as u8;
        Color32::from_rgba_unmultiplied(mix(c.r()), mix(c.g()), mix(c.b()), c.a())
    }

    pub fn darken(c: Color32, amt: f32) -> Color32 {
        let mix = |ch: u8| (ch as f32 * (1.0 - amt)).clamp(0.0, 255.0) as u8;
        Color32::from_rgba_unmultiplied(mix(c.r()), mix(c.g()), mix(c.b()), c.a())
    }

    pub fn with_alpha(c: Color32, a: u8) -> Color32 {
        Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
    }

    /// Paints a raised surface (panel, button cap, fader cap) according to active skin.
    pub fn paint_raised_rect(&self, painter: &egui::Painter, rect: Rect, base: Color32, radius: f32) {
        let r = self.rounding(radius);
        match self.skin {
            Skin::Skeuomorphic => {
                painter.rect_filled(
                    rect.translate(Vec2::new(0.0, 2.0)),
                    r,
                    Color32::from_black_alpha(120),
                );
                painter.rect_filled(rect, r, base);
                let top_half = Rect::from_min_max(rect.min, Pos2::new(rect.max.x, rect.center().y));
                painter.rect_filled(top_half, r, Color32::from_white_alpha(18));
                painter.rect_stroke(rect, r, Stroke::new(1.0, Self::darken(base, 0.45)));
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + radius, rect.min.y + 1.0),
                        Pos2::new(rect.max.x - radius, rect.min.y + 1.0),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(55)),
                );
            }
            Skin::Hybrid => {
                painter.rect_filled(
                    rect.translate(Vec2::new(0.0, 1.0)),
                    r,
                    Color32::from_black_alpha(60),
                );
                painter.rect_filled(rect, r, base);
                painter.rect_stroke(rect, r, Stroke::new(1.0, self.palette.border));
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + radius, rect.min.y + 0.5),
                        Pos2::new(rect.max.x - radius, rect.min.y + 0.5),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(28)),
                );
            }
            Skin::Flat => {
                painter.rect_filled(rect, r, base);
                painter.rect_stroke(rect, r, Stroke::new(1.0, self.palette.border));
            }
        }
    }

    /// Paints a recessed screen, track, or meter well.
    pub fn paint_recessed_rect(&self, painter: &egui::Painter, rect: Rect, radius: f32) {
        let r = self.rounding(radius);
        painter.rect_filled(rect, r, self.palette.bg_recessed);
        match self.skin {
            Skin::Skeuomorphic => {
                painter.line_segment(
                    [Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0), Pos2::new(rect.max.x - 1.0, rect.min.y + 1.0)],
                    Stroke::new(2.0, Color32::from_black_alpha(140)),
                );
                painter.line_segment(
                    [Pos2::new(rect.min.x + 1.0, rect.max.y), Pos2::new(rect.max.x - 1.0, rect.max.y)],
                    Stroke::new(1.0, Color32::from_white_alpha(30)),
                );
                painter.rect_stroke(rect, r, Stroke::new(1.0, Color32::from_black_alpha(160)));
            }
            Skin::Hybrid | Skin::Flat => {
                painter.rect_stroke(rect, r, Stroke::new(1.0, self.palette.border));
            }
        }
    }

    /// Optional glass glare + subtle scanlines for Skeuomorphic screens.
    pub fn paint_screen_overlay(&self, painter: &egui::Painter, rect: Rect) {
        if self.skin == Skin::Skeuomorphic {
            let mut y = rect.min.y + 2.0;
            while y < rect.max.y {
                painter.line_segment(
                    [Pos2::new(rect.min.x, y), Pos2::new(rect.max.x, y)],
                    Stroke::new(1.0, Color32::from_black_alpha(28)),
                );
                y += 4.0;
            }
            let glare = vec![
                rect.left_top(),
                Pos2::new(rect.min.x + rect.width() * 0.55, rect.min.y),
                Pos2::new(rect.min.x + rect.width() * 0.25, rect.max.y),
                rect.left_bottom(),
            ];
            painter.add(Shape::convex_polygon(
                glare,
                Color32::from_white_alpha(12),
                Stroke::NONE,
            ));
        }
    }

    /// Paints a glow halo behind active LEDs or arcs in Skeuomorphic & Hybrid skins.
    pub fn paint_glow_circle(&self, painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
        match self.skin {
            Skin::Skeuomorphic => {
                painter.circle_filled(center, radius * 2.1, Self::with_alpha(color, 35));
                painter.circle_filled(center, radius * 1.45, Self::with_alpha(color, 75));
            }
            Skin::Hybrid => {
                painter.circle_filled(center, radius * 1.5, Self::with_alpha(color, 45));
            }
            Skin::Flat => {}
        }
    }
}
