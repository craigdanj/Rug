use egui::{epaint::Vertex, Color32, Mesh, Pos2, Rect, Rounding, Shape, Stroke, Vec2};

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

    pub fn bassmaker_slate() -> Self {
        Self {
            bg_base: Color32::from_rgb(22, 25, 32),
            bg_panel: Color32::from_rgb(38, 42, 52),
            bg_recessed: Color32::from_rgb(12, 14, 18),
            border: Color32::from_rgb(66, 72, 86),
            text_primary: Color32::from_rgb(236, 240, 248),
            text_muted: Color32::from_rgb(142, 150, 168),
            accent: Color32::from_rgb(249, 138, 66),
            accent_secondary: Color32::from_rgb(138, 108, 255),
            ..Self::studio_cyan()
        }
    }

    pub fn is_light(&self) -> bool {
        let luma = 0.2126 * self.bg_base.r() as f32
            + 0.7152 * self.bg_base.g() as f32
            + 0.0722 * self.bg_base.b() as f32;
        luma > 128.0
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

    /// Syncs native egui popups, tooltips, menus, and scrollbars to match this AudioTheme.
    pub fn apply_to_ctx(&self, ctx: &egui::Context) {
        let mut visuals = if self.palette.is_light() {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        let p = self.palette;
        visuals.panel_fill = p.bg_base;
        visuals.window_fill = p.bg_panel;
        visuals.window_stroke = Stroke::new(1.0, p.border);
        visuals.window_rounding = self.rounding(6.0);
        visuals.extreme_bg_color = p.bg_recessed;
        visuals.faint_bg_color = Self::lerp_color(p.bg_base, p.bg_panel, 0.5);
        visuals.selection.bg_fill = Self::with_alpha(p.accent, 65);
        visuals.selection.stroke = Stroke::new(1.0, p.accent);

        visuals.widgets.noninteractive.bg_fill = p.bg_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.text_primary);

        visuals.widgets.inactive.bg_fill = p.bg_panel;
        visuals.widgets.inactive.weak_bg_fill = p.bg_panel;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, p.border);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, p.text_primary);

        visuals.widgets.hovered.bg_fill = if p.is_light() {
            Self::darken(p.bg_panel, 0.06)
        } else {
            Self::lighten(p.bg_panel, 0.08)
        };
        visuals.widgets.hovered.weak_bg_fill = visuals.widgets.hovered.bg_fill;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, p.accent);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, p.text_primary);

        visuals.widgets.active.bg_fill = Self::with_alpha(p.accent, 90);
        visuals.widgets.active.weak_bg_fill = Self::with_alpha(p.accent, 90);
        visuals.widgets.active.bg_stroke = Stroke::new(1.2, p.accent);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, p.text_primary);

        ctx.set_visuals(visuals);
    }

    pub fn rounding(&self, base: f32) -> Rounding {
        match self.skin {
            Skin::Skeuomorphic => Rounding::same(base + 2.0),
            Skin::Hybrid => Rounding::same(base),
            Skin::Flat => Rounding::same((base * 0.35).min(2.0)),
        }
    }

    pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
        let t = t.clamp(0.0, 1.0);
        let mix = |c0: u8, c1: u8| (c0 as f32 + (c1 as f32 - c0 as f32) * t).round().clamp(0.0, 255.0) as u8;
        Color32::from_rgba_unmultiplied(
            mix(a.r(), b.r()),
            mix(a.g(), b.g()),
            mix(a.b(), b.b()),
            mix(a.a(), b.a()),
        )
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

    /// Paints a rounded rectangle with a smooth vertical vertex-color gradient.
    pub fn paint_vertical_gradient_rect(
        painter: &egui::Painter,
        rect: Rect,
        rounding: Rounding,
        top_color: Color32,
        bottom_color: Color32,
    ) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let max_r = (rect.width().min(rect.height()) * 0.5).max(0.0);
        let r_tl = rounding.nw.min(max_r);
        let r_tr = rounding.ne.min(max_r);
        let r_br = rounding.se.min(max_r);
        let r_bl = rounding.sw.min(max_r);

        let mut perimeter: Vec<Pos2> = Vec::with_capacity(24);
        let steps = 4usize;
        let corners = [
            (Pos2::new(rect.max.x - r_tr, rect.min.y + r_tr), r_tr, -std::f32::consts::FRAC_PI_2, 0.0),
            (Pos2::new(rect.max.x - r_br, rect.max.y - r_br), r_br, 0.0, std::f32::consts::FRAC_PI_2),
            (Pos2::new(rect.min.x + r_bl, rect.max.y - r_bl), r_bl, std::f32::consts::FRAC_PI_2, std::f32::consts::PI),
            (Pos2::new(rect.min.x + r_tl, rect.min.y + r_tl), r_tl, std::f32::consts::PI, std::f32::consts::PI * 1.5),
        ];
        for (center, r, a0, a1) in corners {
            if r <= 0.1 {
                perimeter.push(center);
            } else {
                for i in 0..=steps {
                    let a = a0 + (a1 - a0) * (i as f32 / steps as f32);
                    perimeter.push(center + Vec2::angled(a) * r);
                }
            }
        }

        let mut mesh = Mesh::default();
        let center = rect.center();
        let center_col = Self::lerp_color(top_color, bottom_color, 0.5);
        mesh.vertices.push(Vertex { pos: center, uv: egui::epaint::WHITE_UV, color: center_col });
        for &p in &perimeter {
            let t = ((p.y - rect.min.y) / rect.height()).clamp(0.0, 1.0);
            mesh.vertices.push(Vertex {
                pos: p,
                uv: egui::epaint::WHITE_UV,
                color: Self::lerp_color(top_color, bottom_color, t),
            });
        }
        let count = perimeter.len() as u32;
        for i in 0..count {
            let next = (i + 1) % count;
            mesh.add_triangle(0, 1 + i, 1 + next);
        }
        painter.add(Shape::mesh(mesh));
    }

    /// Paints a smooth radial gradient circle (for soft shadows, LED halos, and domes).
    pub fn paint_radial_gradient_circle(
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        inner_color: Color32,
        outer_color: Color32,
    ) {
        if radius <= 0.0 {
            return;
        }
        let steps = 28u32;
        let mut mesh = Mesh::default();
        mesh.vertices.push(Vertex { pos: center, uv: egui::epaint::WHITE_UV, color: inner_color });
        for i in 0..steps {
            let a = (i as f32 / steps as f32) * std::f32::consts::TAU;
            mesh.vertices.push(Vertex {
                pos: center + Vec2::angled(a) * radius,
                uv: egui::epaint::WHITE_UV,
                color: outer_color,
            });
        }
        for i in 0..steps {
            let next = (i + 1) % steps;
            mesh.add_triangle(0, 1 + i, 1 + next);
        }
        painter.add(Shape::mesh(mesh));
    }

    /// Paints a circle with a smooth top-to-bottom directional lighting gradient.
    pub fn paint_directional_circle(
        painter: &egui::Painter,
        center: Pos2,
        radius: f32,
        top_color: Color32,
        bottom_color: Color32,
    ) {
        if radius <= 0.0 {
            return;
        }
        let steps = 32u32;
        let mut mesh = Mesh::default();
        let mid_col = Self::lerp_color(top_color, bottom_color, 0.5);
        mesh.vertices.push(Vertex { pos: center, uv: egui::epaint::WHITE_UV, color: mid_col });
        for i in 0..steps {
            let a = (i as f32 / steps as f32) * std::f32::consts::TAU;
            let t = (a.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
            mesh.vertices.push(Vertex {
                pos: center + Vec2::angled(a) * radius,
                uv: egui::epaint::WHITE_UV,
                color: Self::lerp_color(top_color, bottom_color, t),
            });
        }
        for i in 0..steps {
            let next = (i + 1) % steps;
            mesh.add_triangle(0, 1 + i, 1 + next);
        }
        painter.add(Shape::mesh(mesh));
    }

    /// Paints a raised surface (panel, button cap, fader cap) according to active skin.
    pub fn paint_raised_rect(&self, painter: &egui::Painter, rect: Rect, base: Color32, radius: f32) {
        let r = self.rounding(radius);
        let light = self.palette.is_light();
        match self.skin {
            Skin::Skeuomorphic => {
                // Soft layered ambient shadow for machined tactile depth (lighter on Nordic)
                painter.rect_filled(
                    rect.translate(Vec2::new(0.0, 3.0)),
                    r,
                    Color32::from_black_alpha(if light { 26 } else { 70 }),
                );
                painter.rect_filled(
                    rect.translate(Vec2::new(0.0, 1.5)),
                    r,
                    Color32::from_black_alpha(if light { 38 } else { 95 }),
                );
                // Smooth matte anodized vertical gradient
                Self::paint_vertical_gradient_rect(
                    painter,
                    rect,
                    r,
                    Self::lighten(base, if light { 0.06 } else { 0.11 }),
                    Self::darken(base, if light { 0.07 } else { 0.13 }),
                );
                // Crisp outer contour
                let stroke_col = if light { self.palette.border } else { Self::darken(base, 0.45) };
                painter.rect_stroke(rect, r, Stroke::new(1.0, stroke_col));
                // Subtle top specular rim highlight
                let inset = (r.nw + 1.0).min(rect.width() * 0.4);
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + inset, rect.min.y + 1.0),
                        Pos2::new(rect.max.x - inset, rect.min.y + 1.0),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(if light { 160 } else { 52 })),
                );
            }
            Skin::Hybrid => {
                painter.rect_filled(
                    rect.translate(Vec2::new(0.0, 1.5)),
                    r,
                    Color32::from_black_alpha(if light { 24 } else { 65 }),
                );
                Self::paint_vertical_gradient_rect(
                    painter,
                    rect,
                    r,
                    Self::lighten(base, if light { 0.05 } else { 0.08 }),
                    Self::darken(base, if light { 0.05 } else { 0.08 }),
                );
                painter.rect_stroke(rect, r, Stroke::new(1.0, self.palette.border));
                let inset = (radius + 1.0).min(rect.width() * 0.4);
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + inset, rect.min.y + 1.0),
                        Pos2::new(rect.max.x - inset, rect.min.y + 1.0),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(if light { 130 } else { 34 })),
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
        let light = self.palette.is_light();
        match self.skin {
            Skin::Skeuomorphic => {
                Self::paint_vertical_gradient_rect(
                    painter,
                    rect,
                    r,
                    Self::darken(self.palette.bg_recessed, if light { 0.08 } else { 0.35 }),
                    Self::lighten(self.palette.bg_recessed, 0.04),
                );
                let top_band = Rect::from_min_max(
                    rect.min + Vec2::new(1.0, 1.0),
                    Pos2::new(rect.max.x - 1.0, (rect.min.y + 8.0).min(rect.max.y)),
                );
                Self::paint_vertical_gradient_rect(
                    painter,
                    top_band,
                    Rounding::same(r.nw.min(4.0)),
                    Color32::from_black_alpha(if light { 35 } else { 95 }),
                    Color32::TRANSPARENT,
                );
                let rim_col = if light { self.palette.border } else { Color32::from_black_alpha(175) };
                painter.rect_stroke(rect, r, Stroke::new(1.0, rim_col));
                let inset = (r.sw + 1.0).min(rect.width() * 0.4);
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + inset, rect.max.y + 0.5),
                        Pos2::new(rect.max.x - inset, rect.max.y + 0.5),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(if light { 145 } else { 36 })),
                );
            }
            Skin::Hybrid => {
                Self::paint_vertical_gradient_rect(
                    painter,
                    rect,
                    r,
                    Self::darken(self.palette.bg_recessed, if light { 0.06 } else { 0.22 }),
                    Self::lighten(self.palette.bg_recessed, 0.03),
                );
                painter.rect_stroke(rect, r, Stroke::new(1.0, self.palette.border));
                let inset = (radius + 1.0).min(rect.width() * 0.4);
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + inset, rect.max.y + 0.5),
                        Pos2::new(rect.max.x - inset, rect.max.y + 0.5),
                    ],
                    Stroke::new(0.8, Color32::from_white_alpha(if light { 120 } else { 18 })),
                );
            }
            Skin::Flat => {
                painter.rect_filled(rect, r, self.palette.bg_recessed);
                painter.rect_stroke(rect, r, Stroke::new(1.0, self.palette.border));
            }
        }
    }

    /// Ambient glass finish for screens: soft bottom accent glow + top bezel vignette for Skeuomorphic.
    pub fn paint_screen_overlay(&self, painter: &egui::Painter, rect: Rect) {
        let light = self.palette.is_light();
        match self.skin {
            Skin::Skeuomorphic => {
                let glow_h = (rect.height() * 0.38).min(36.0);
                let bottom_glow = Rect::from_min_max(
                    Pos2::new(rect.min.x + 1.0, (rect.max.y - glow_h).max(rect.min.y)),
                    rect.max - Vec2::new(1.0, 1.0),
                );
                Self::paint_vertical_gradient_rect(
                    painter,
                    bottom_glow,
                    Rounding::same(3.0),
                    Color32::TRANSPARENT,
                    Self::with_alpha(self.palette.accent, if light { 22 } else { 34 }),
                );
                let inset = (rect.width() * 0.12).min(10.0);
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + inset, rect.max.y - 1.0),
                        Pos2::new(rect.max.x - inset, rect.max.y - 1.0),
                    ],
                    Stroke::new(1.0, Self::with_alpha(self.palette.accent, if light { 55 } else { 75 })),
                );
                let top_shadow = Rect::from_min_max(
                    rect.min + Vec2::new(1.0, 1.0),
                    Pos2::new(rect.max.x - 1.0, (rect.min.y + 10.0).min(rect.max.y)),
                );
                Self::paint_vertical_gradient_rect(
                    painter,
                    top_shadow,
                    Rounding::same(3.0),
                    Color32::from_black_alpha(if light { 24 } else { 65 }),
                    Color32::TRANSPARENT,
                );
            }
            Skin::Hybrid => {
                let top_shadow = Rect::from_min_max(
                    rect.min + Vec2::new(1.0, 1.0),
                    Pos2::new(rect.max.x - 1.0, (rect.min.y + 10.0).min(rect.max.y)),
                );
                Self::paint_vertical_gradient_rect(
                    painter,
                    top_shadow,
                    Rounding::same(2.0),
                    Color32::from_black_alpha(if light { 20 } else { 48 }),
                    Color32::TRANSPARENT,
                );
            }
            Skin::Flat => {}
        }
    }

    /// Paints a glow halo behind active LEDs or arcs in Skeuomorphic & Hybrid skins.
    pub fn paint_glow_circle(&self, painter: &egui::Painter, center: Pos2, radius: f32, color: Color32) {
        match self.skin {
            Skin::Skeuomorphic => {
                Self::paint_radial_gradient_circle(
                    painter,
                    center,
                    radius * 2.4,
                    Self::with_alpha(color, 110),
                    Self::with_alpha(color, 0),
                );
            }
            Skin::Hybrid => {
                Self::paint_radial_gradient_circle(
                    painter,
                    center,
                    radius * 2.1,
                    Self::with_alpha(color, 90),
                    Self::with_alpha(color, 0),
                );
            }
            Skin::Flat => {}
        }
    }
}
