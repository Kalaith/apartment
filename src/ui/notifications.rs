use super::common::*;
use super::theme::{scale, space};
use crate::assets::AssetManager;
use crate::simulation::{EventLog, EventSeverity};
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, truncate_text_to_width};

pub fn draw_notifications(event_log: &EventLog, _current_tick: u32, _assets: &AssetManager) {
    let y = screen_height() - layout::FOOTER_HEIGHT();
    let w = screen_width();
    let h = layout::FOOTER_HEIGHT();

    // Background
    draw_rectangle(0.0, y, w, h, colors::SURFACE_HEADER());
    draw_line(0.0, y, w, y, 1.0, colors::BORDER_STRONG());

    // Title
    draw_ui_text(
        "EVENTS",
        space::LG,
        y + 22.0,
        scale::LABEL,
        colors::TEXT_DIM(),
    );

    // Recent events (single-line each, truncated to the footer width).
    let recent = event_log.recent_events(5);
    let mut event_y = y + 44.0;
    let max_w = w - space::LG * 2.0;

    for event in recent {
        let color = match event.severity() {
            EventSeverity::Positive => colors::POSITIVE(),
            EventSeverity::Info => colors::TEXT_DIM(),
            EventSeverity::Warning => colors::WARNING(),
            EventSeverity::Negative => colors::NEGATIVE(),
        };

        let display_msg = truncate_text_to_width(&event.message(), max_w, scale::BODY);
        draw_ui_text(&display_msg, space::LG, event_y, scale::BODY, color);
        event_y += scale::BODY + space::XS;

        if event_y > y + h - space::SM {
            break;
        }
    }
}
