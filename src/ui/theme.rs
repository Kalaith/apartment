//! # Theme
//!
//! Single source of truth for the game's visual design tokens: palette, type
//! scale, spacing scale, and reusable style builders. Panels compose these
//! instead of hand-picking colors and font sizes, which keeps the look
//! consistent and eliminates the ad-hoc magic numbers that caused overlaps.
//!
//! The palette mirrors the `theme` block in `assets/config.json`
//! (`ThemeConfig`); these consts are the compile-time defaults the whole UI
//! reads through `common::colors`.

use macroquad::prelude::Color;
use macroquad_toolkit::ui::{ButtonStyle, SurfaceStyle};

/// Bolder restyle palette: deep desaturated indigo base with a warm amber
/// primary, plus clear semantic accents.
pub mod color {
    use macroquad::prelude::Color;

    // Base surfaces (dark -> light)
    pub const BACKGROUND: Color = Color::new(0.07, 0.08, 0.11, 1.0);
    pub const SURFACE: Color = Color::new(0.12, 0.13, 0.17, 1.0);
    pub const SURFACE_ALT: Color = Color::new(0.16, 0.17, 0.22, 1.0);
    pub const SURFACE_HEADER: Color = Color::new(0.10, 0.11, 0.15, 1.0);

    // Back-compat aliases used across the existing UI (map onto the new base).
    pub const PANEL: Color = SURFACE;
    pub const PANEL_HEADER: Color = SURFACE_HEADER;

    // Hairlines / outlines
    pub const BORDER: Color = Color::new(0.24, 0.26, 0.33, 1.0);
    pub const BORDER_STRONG: Color = Color::new(0.38, 0.41, 0.50, 1.0);

    // Text
    pub const TEXT: Color = Color::new(0.90, 0.91, 0.94, 1.0);
    pub const TEXT_BRIGHT: Color = Color::new(1.0, 1.0, 1.0, 1.0);
    pub const TEXT_DIM: Color = Color::new(0.58, 0.61, 0.69, 1.0);

    // Primary (warm amber) — the headline accent for this restyle.
    pub const PRIMARY: Color = Color::new(0.96, 0.70, 0.30, 1.0);
    pub const PRIMARY_HOVER: Color = Color::new(1.0, 0.78, 0.42, 1.0);
    pub const PRIMARY_PRESSED: Color = Color::new(0.78, 0.55, 0.20, 1.0);

    // Secondary accent (cool teal) — selection glow / info highlights.
    pub const ACCENT: Color = Color::new(0.35, 0.78, 0.82, 1.0);

    // Semantic status colors
    pub const POSITIVE: Color = Color::new(0.42, 0.80, 0.48, 1.0);
    pub const WARNING: Color = Color::new(0.95, 0.68, 0.25, 1.0);
    pub const NEGATIVE: Color = Color::new(0.92, 0.36, 0.38, 1.0);

    // Apartment-unit states
    pub const VACANT: Color = Color::new(0.20, 0.21, 0.26, 1.0);
    pub const OCCUPIED: Color = Color::new(0.18, 0.28, 0.34, 1.0);
    pub const SELECTED: Color = Color::new(0.30, 0.44, 0.52, 1.0);
    pub const HOVERED: Color = Color::new(0.24, 0.30, 0.40, 1.0);

    // Tenant archetype accents
    pub const STUDENT: Color = Color::new(0.90, 0.60, 0.35, 1.0);
    pub const PROFESSIONAL: Color = Color::new(0.40, 0.60, 0.92, 1.0);
    pub const ARTIST: Color = Color::new(0.82, 0.42, 0.78, 1.0);
    pub const FAMILY: Color = Color::new(0.45, 0.82, 0.50, 1.0);
    pub const ELDERLY: Color = Color::new(0.72, 0.74, 0.80, 1.0);

    /// A translucent shadow used under raised surfaces.
    pub const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.35);
}

/// Type scale (font sizes in logical px). Replaces the ad-hoc 11..32 sizes.
pub mod scale {
    pub const TITLE: f32 = 22.0;
    pub const HEADING: f32 = 18.0;
    pub const BODY: f32 = 15.0;
    pub const LABEL: f32 = 13.0;
    pub const CAPTION: f32 = 11.0;
}

/// Spacing scale (logical px). Replaces magic 10/20/25/... offsets.
pub mod space {
    pub const XS: f32 = 4.0;
    pub const SM: f32 = 8.0;
    pub const MD: f32 = 12.0;
    pub const LG: f32 = 16.0;
    pub const XL: f32 = 24.0;
    /// Default inner padding for panels/cards.
    pub const PAD: f32 = 16.0;
}

// --- Surface style builders ----------------------------------------------

/// A raised card: filled surface with a soft drop shadow and hairline border.
pub fn card_style() -> SurfaceStyle {
    SurfaceStyle::new(color::SURFACE)
        .with_shadow(macroquad::prelude::vec2(0.0, 3.0), color::SHADOW)
        .with_border(1.0, color::BORDER)
}

/// A selected/active card variant with the primary accent edge.
pub fn card_selected_style() -> SurfaceStyle {
    SurfaceStyle::new(color::SURFACE_ALT)
        .with_shadow(macroquad::prelude::vec2(0.0, 3.0), color::SHADOW)
        .with_border(2.0, color::PRIMARY)
        .with_left_accent(4.0, color::PRIMARY)
}

/// A titled panel surface (header strip + divider).
pub fn panel_style() -> SurfaceStyle {
    SurfaceStyle::new(color::SURFACE)
        .with_shadow(macroquad::prelude::vec2(0.0, 3.0), color::SHADOW)
        .with_border(1.0, color::BORDER)
        .with_header(38.0, color::SURFACE_HEADER)
        .with_header_divider(1.0, color::BORDER_STRONG)
}

// --- Button style builders ------------------------------------------------

/// Semantic button tone drawn with the theme palette (not the toolkit's
/// default blue), so buttons match the restyle.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Primary,
    Secondary,
    Positive,
    Danger,
}

fn dim(c: Color, f: f32) -> Color {
    Color::new(c.r * f, c.g * f, c.b * f, c.a)
}

fn lift(c: Color, f: f32) -> Color {
    Color::new(
        (c.r * f).min(1.0),
        (c.g * f).min(1.0),
        (c.b * f).min(1.0),
        c.a,
    )
}

/// Build a `ButtonStyle` for a tone from the theme palette.
pub fn button_style(tone: Tone) -> ButtonStyle {
    match tone {
        Tone::Primary => ButtonStyle {
            normal: color::PRIMARY,
            hovered: color::PRIMARY_HOVER,
            pressed: color::PRIMARY_PRESSED,
            border: color::PRIMARY_HOVER,
            text_color: Color::new(0.10, 0.08, 0.04, 1.0),
            disabled: Color::new(0.18, 0.16, 0.12, 1.0),
        },
        Tone::Secondary => ButtonStyle {
            normal: color::SURFACE_ALT,
            hovered: color::HOVERED,
            pressed: dim(color::SURFACE_ALT, 0.8),
            border: color::BORDER_STRONG,
            text_color: color::TEXT,
            disabled: Color::new(0.12, 0.12, 0.14, 1.0),
        },
        Tone::Positive => tone_style(color::POSITIVE),
        Tone::Danger => tone_style(color::NEGATIVE),
    }
}

fn tone_style(base: Color) -> ButtonStyle {
    ButtonStyle {
        normal: base,
        hovered: lift(base, 1.15),
        pressed: dim(base, 0.75),
        border: lift(base, 1.2),
        text_color: Color::new(0.08, 0.09, 0.06, 1.0),
        disabled: dim(base, 0.3),
    }
}
