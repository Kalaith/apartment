
use macroquad::prelude::*;

// Import toolkit utilities
pub use macroquad_toolkit::input::{is_hovered, was_clicked};
pub use macroquad_toolkit::ui::progress_bar;

/// Color palette (extends toolkit colors with game-specific colors)
pub mod colors {
    use macroquad::prelude::Color;

    // Re-export common colors from toolkit
    pub use macroquad_toolkit::colors::dark::{
        BACKGROUND, PANEL, PANEL_HEADER, TEXT, TEXT_DIM,
        ACCENT, POSITIVE, NEGATIVE
    };

    // Game-specific colors that toolkit doesn't have
    pub const TEXT_BRIGHT: Color = Color::new(1.0, 1.0, 1.0, 1.0);  // Pure white
    pub const WARNING: Color = Color::new(0.9, 0.7, 0.2, 1.0);

    pub const VACANT: Color = Color::new(0.3, 0.3, 0.35, 1.0);
    pub const OCCUPIED: Color = Color::new(0.25, 0.35, 0.45, 1.0);
    pub const SELECTED: Color = Color::new(0.35, 0.5, 0.7, 1.0);
    pub const HOVERED: Color = Color::new(0.3, 0.4, 0.55, 1.0);

    // Archetype colors
    pub const STUDENT: Color = Color::new(0.8, 0.5, 0.3, 1.0);      // Orange-ish
    pub const PROFESSIONAL: Color = Color::new(0.3, 0.5, 0.8, 1.0); // Blue-ish
    pub const ARTIST: Color = Color::new(0.8, 0.3, 0.7, 1.0);       // Purple-ish
}

use crate::tenant::TenantArchetype;

/// Get color for tenant archetype
pub fn archetype_color(archetype: &TenantArchetype) -> macroquad::prelude::Color {
    match archetype {
        TenantArchetype::Student => colors::STUDENT,
        TenantArchetype::Professional => colors::PROFESSIONAL,
        TenantArchetype::Artist => colors::ARTIST,
        TenantArchetype::Family => Color::new(0.4, 0.8, 0.4, 1.0),   // Green-ish
        TenantArchetype::Elderly => Color::new(0.7, 0.7, 0.7, 1.0),  // Grey-ish
    }
}

/// Get icon for happiness level
pub fn happiness_icon(happiness: i32) -> &'static str {
    match happiness {
        85..=100 => "😃", // Ecstatic
        70..=84 => "🙂",  // Happy
        50..=69 => "😐",  // Neutral
        30..=49 => "☹️",  // Unhappy
        0..=29 => "😭",   // Miserable
        _ => "😶",
    }
}

/// Layout constants
pub mod layout {
    pub const HEADER_HEIGHT: f32 = 60.0;
    pub const FOOTER_HEIGHT: f32 = 100.0;
    pub const PANEL_SPLIT: f32 = 0.6;  // Building view takes 60%
    pub const PADDING: f32 = 10.0;
    pub const UNIT_WIDTH: f32 = 120.0;
    pub const UNIT_HEIGHT: f32 = 80.0;
    pub const UNIT_GAP: f32 = 15.0;
    pub const FLOOR_HEIGHT: f32 = 100.0;
}

/// Draw a simple button, returns true if clicked
///
/// Wrapper around toolkit button that maintains apartment's API (enabled parameter)
/// and click behavior (triggers on press, not release)
/// Draw a simple button, returns true if clicked
///
/// Wrapper around toolkit button that maintains apartment's API (enabled parameter)
/// and click behavior (triggers on press, not release)
pub fn button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool) -> bool {
    // If disabled, use a disabled style but don't check for clicks
    let style = if !enabled {
        macroquad_toolkit::ui::ButtonStyle {
             normal: Color::new(0.2, 0.2, 0.2, 1.0),
             hovered: Color::new(0.2, 0.2, 0.2, 1.0),
             pressed: Color::new(0.2, 0.2, 0.2, 1.0),
             border: colors::ACCENT,
             text_color: colors::TEXT_DIM,
             disabled: Color::new(0.1, 0.1, 0.1, 1.0),
        }
    } else {
        macroquad_toolkit::ui::ButtonStyle {
            normal: colors::PANEL,
            hovered: colors::HOVERED,
            pressed: Color::new(0.25, 0.35, 0.5, 1.0),
            border: colors::ACCENT,
            text_color: colors::TEXT,
            disabled: Color::new(0.1, 0.1, 0.1, 1.0),
        }
    };

    if !enabled {
         macroquad_toolkit::ui::button_on_press(x, y, w, h, text, &style);
         return false;
    }

    macroquad_toolkit::ui::button_on_press(x, y, w, h, text, &style)
}

/// Draw a button with custom colors
pub fn colored_button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool, bg_color: Color, text_color: Color) -> bool {
    if !enabled {
        // Draw disabled version using generic button logic logic above or custom draw
        draw_rectangle(x, y, w, h, Color::new(0.2, 0.2, 0.2, 1.0));
        draw_rectangle_lines(x, y, w, h, 2.0, colors::TEXT_DIM);
        
        let text_size = 20.0;
        let text_width = measure_text(text, None, text_size as u16, 1.0).width;
        draw_text(text, x + (w - text_width) / 2.0, y + h / 2.0 + 6.0, text_size, colors::TEXT_DIM);
        return false;
    }
    
    // Use toolkit's colored_button logic but we need to respect the text_color param
    // Toolkit's colored_button uses standard text colors. 
    // Since this needs custom text color, we'll manually construct the style and call button_on_release (or press to match apartment)
    
    // Apartment uses "on press" mostly
    let style = macroquad_toolkit::ui::ButtonStyle {
        normal: bg_color,
        hovered: Color::new(bg_color.r * 1.2, bg_color.g * 1.2, bg_color.b * 1.2, bg_color.a),
        pressed: Color::new(bg_color.r * 0.8, bg_color.g * 0.8, bg_color.b * 0.8, bg_color.a),
        border: colors::TEXT_DIM,
        text_color: text_color,
        disabled: Color::new(0.1, 0.1, 0.1, 1.0),
    };
    
    macroquad_toolkit::ui::button_on_press(x, y, w, h, text, &style)
}

/// Draw a panel with header
///
/// Wrapper around toolkit panel that maintains apartment's API (title is not optional)
pub fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    macroquad_toolkit::ui::panel(x, y, w, h, Some(title));
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
