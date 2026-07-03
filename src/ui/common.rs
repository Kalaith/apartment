use macroquad::prelude::*;

// Import toolkit utilities
pub use macroquad_toolkit::input::{is_hovered, was_clicked};
pub use macroquad_toolkit::ui::progress_bar;

/// Color palette — single source of truth lives in [`crate::ui::theme::color`].
/// Re-exported here so existing `colors::NAME` references keep working while
/// the whole UI picks up the restyle.
pub mod colors {
    pub use crate::ui::theme::color::*;
}

use crate::tenant::TenantArchetype;

/// Get color for tenant archetype
pub fn archetype_color(archetype: &TenantArchetype) -> macroquad::prelude::Color {
    match archetype {
        TenantArchetype::Student => colors::STUDENT,
        TenantArchetype::Professional => colors::PROFESSIONAL,
        TenantArchetype::Artist => colors::ARTIST,
        TenantArchetype::Family => colors::FAMILY,
        TenantArchetype::Elderly => colors::ELDERLY,
    }
}

/// Layout constants
pub mod layout {
    pub const HEADER_HEIGHT: f32 = 60.0;
    pub const FOOTER_HEIGHT: f32 = 100.0;
    pub const PANEL_SPLIT: f32 = 0.6; // Building view takes 60%
    pub const PADDING: f32 = 10.0;
    pub const UNIT_WIDTH: f32 = 120.0;
    pub const UNIT_HEIGHT: f32 = 80.0;
    pub const UNIT_GAP: f32 = 15.0;
    pub const FLOOR_HEIGHT: f32 = 100.0;
}

/// Draw a standard secondary button (returns true on click). Restyled to the
/// theme tone system so every call site picks up the new look at once.
pub fn button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool) -> bool {
    crate::ui::widgets::button_at(
        Rect::new(x, y, w, h),
        text,
        enabled,
        crate::ui::theme::Tone::Secondary,
    )
}

/// Draw a titled panel using the theme's card + header style.
pub fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    crate::ui::widgets::draw_panel(Rect::new(x, y, w, h), title);
}

/// Get color for condition value
pub fn condition_color(condition: i32) -> Color {
    match condition {
        80..=100 => colors::POSITIVE,
        50..=79 => colors::ACCENT,
        30..=49 => colors::WARNING,
        _ => colors::NEGATIVE,
    }
}

/// Get color for happiness value
pub fn happiness_color(happiness: i32) -> Color {
    match happiness {
        70..=100 => colors::POSITIVE,
        40..=69 => colors::ACCENT,
        20..=39 => colors::WARNING,
        _ => colors::NEGATIVE,
    }
}
