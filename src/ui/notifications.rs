use macroquad::prelude::*;
use crate::simulation::{EventLog, EventSeverity};
use super::common::*;
use crate::assets::AssetManager;

pub fn draw_notifications(event_log: &EventLog, _current_tick: u32, _assets: &AssetManager) {
    let y = screen_height() - layout::FOOTER_HEIGHT;
    let w = screen_width();
    let h = layout::FOOTER_HEIGHT;
    
    // Background
    draw_rectangle(0.0, y, w, h, colors::PANEL);
    draw_line(0.0, y, w, y, 2.0, colors::TEXT_DIM);
    
    // Title
    draw_text("EVENTS", 15.0, y + 22.0, 16.0, colors::TEXT_DIM);
    
    // Recent events
    let recent = event_log.recent_events(5);
    let mut event_y = y + 45.0;
    
    for event in recent {
        let color = match event.severity() {
            EventSeverity::Positive => colors::POSITIVE,
            EventSeverity::Info => colors::TEXT_DIM,
            EventSeverity::Warning => colors::WARNING,
            EventSeverity::Negative => colors::NEGATIVE,
        };
        
        let msg = event.message();
        let display_msg = if msg.len() > 80 {
            format!("{}...", &msg[..77])
        } else {
            msg
        };
        
        draw_text(&display_msg, 15.0, event_y, 14.0, color);
        event_y += 18.0;
        
        if event_y > y + h - 10.0 {
            break;
        }
    }
}

/// Draw a floating notification for important events
pub fn draw_toast(message: &str, severity: EventSeverity, progress: f32) {
    if progress <= 0.0 {
        return;
    }
    
    let alpha = progress.min(1.0);
    let w = 400.0;
    let h = 50.0;
    let x = (screen_width() - w) / 2.0;
    let y = layout::HEADER_HEIGHT + 20.0;
    
    let bg_color = match severity {
        EventSeverity::Positive => Color::new(0.2, 0.5, 0.3, alpha),
        EventSeverity::Info => Color::new(0.2, 0.3, 0.5, alpha),
        EventSeverity::Warning => Color::new(0.5, 0.4, 0.2, alpha),
        EventSeverity::Negative => Color::new(0.5, 0.2, 0.2, alpha),
    };
    
    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(1.0, 1.0, 1.0, alpha * 0.5));
    
    let text_color = Color::new(1.0, 1.0, 1.0, alpha);
    let text_width = measure_text(message, None, 20, 1.0).width;
    draw_text(message, x + (w - text_width) / 2.0, y + 32.0, 20.0, text_color);
}
