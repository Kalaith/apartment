
use macroquad::prelude::*;

/// Color palette
pub mod colors {
    use macroquad::prelude::Color;
    
    pub const BACKGROUND: Color = Color::new(0.12, 0.12, 0.14, 1.0);
    pub const PANEL: Color = Color::new(0.18, 0.18, 0.22, 1.0);
    pub const PANEL_HEADER: Color = Color::new(0.22, 0.22, 0.28, 1.0);
    pub const TEXT: Color = Color::new(0.9, 0.9, 0.9, 1.0);
    pub const TEXT_BRIGHT: Color = Color::new(1.0, 1.0, 1.0, 1.0);  // Pure white
    pub const TEXT_DIM: Color = Color::new(0.6, 0.6, 0.6, 1.0);
    pub const ACCENT: Color = Color::new(0.3, 0.6, 0.9, 1.0);
    pub const POSITIVE: Color = Color::new(0.3, 0.8, 0.4, 1.0);
    pub const WARNING: Color = Color::new(0.9, 0.7, 0.2, 1.0);
    pub const NEGATIVE: Color = Color::new(0.9, 0.3, 0.3, 1.0);
    
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
pub fn button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool) -> bool {
    let mouse = mouse_position();
    let rect = Rect::new(x, y, w, h);
    let hovered = rect.contains(Vec2::new(mouse.0, mouse.1));
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left) && enabled;
    
    let is_pressed = hovered && is_mouse_button_down(MouseButton::Left) && enabled;
    
    let bg_color = if !enabled {
        Color::new(0.2, 0.2, 0.2, 1.0)
    } else if is_pressed {
        Color::new(0.25, 0.35, 0.5, 1.0) // Darker when pressed
    } else if hovered {
        colors::HOVERED
    } else {
        colors::PANEL
    };
    
    // Draw background
    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, 2.0, colors::ACCENT);
    
    // Text offset
    let y_offset = if is_pressed { 2.0 } else { 0.0 };
    
    let text_color = if enabled { colors::TEXT } else { colors::TEXT_DIM };
    let text_size = 20.0;
    let text_width = measure_text(text, None, text_size as u16, 1.0).width;
    draw_text(text, x + (w - text_width) / 2.0, y + h / 2.0 + 6.0 + y_offset, text_size, text_color);
    
    clicked
}

/// Draw a progress bar
pub fn progress_bar(x: f32, y: f32, w: f32, h: f32, value: f32, max: f32, color: Color) {
    let fill_width = (value / max).clamp(0.0, 1.0) * w;
    
    draw_rectangle(x, y, w, h, Color::new(0.15, 0.15, 0.15, 1.0));
    draw_rectangle(x, y, fill_width, h, color);
    draw_rectangle_lines(x, y, w, h, 1.0, colors::TEXT_DIM);
}

/// Draw a panel with header
pub fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    // Background
    draw_rectangle(x, y, w, h, colors::PANEL);
    
    // Header
    draw_rectangle(x, y, w, 30.0, colors::PANEL_HEADER);
    draw_text(title, x + 10.0, y + 22.0, 20.0, colors::TEXT);
    
    // Border
    draw_rectangle_lines(x, y, w, h, 1.0, colors::TEXT_DIM);
}

/// Check if mouse is over a rectangle
pub fn is_hovered(x: f32, y: f32, w: f32, h: f32) -> bool {
    let mouse = mouse_position();
    let rect = Rect::new(x, y, w, h);
    rect.contains(Vec2::new(mouse.0, mouse.1))
}

/// Check if rectangle was clicked
pub fn was_clicked(x: f32, y: f32, w: f32, h: f32) -> bool {
    is_hovered(x, y, w, h) && is_mouse_button_pressed(MouseButton::Left)
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
