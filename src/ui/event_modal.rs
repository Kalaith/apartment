
use macroquad::prelude::*;
use crate::ui::{UiAction, colors};
use crate::narrative::events::NarrativeEvent;

pub fn draw_event_modal(event: &NarrativeEvent) -> Option<UiAction> {
    let screen_w = screen_width();
    let screen_h = screen_height();
    
    // Dim background
    draw_rectangle(0., 0., screen_w, screen_h, Color::new(0., 0., 0., 0.6));
    
    let modal_w = 600.0;
    let modal_h = 400.0;
    let x = (screen_w - modal_w) / 2.0;
    let y = (screen_h - modal_h) / 2.0;
    
    // Main Panel
    draw_rectangle(x, y, modal_w, modal_h, colors::PANEL);
    draw_rectangle_lines(x, y, modal_w, modal_h, 3.0, colors::TEXT_DIM);
    
    // Header
    draw_rectangle(x, y, modal_w, 60.0, colors::PANEL_HEADER);
    draw_text(&event.headline, x + 20.0, y + 40.0, 30.0, colors::TEXT_BRIGHT);
    
    // Body Text
    // TODO: Implement proper wrapping. For now, assuming relatively short text or manual breaks.
    let body_y = y + 90.0;
    let mut current_y = body_y;
    let font_size = 20.0;
    let max_width = modal_w - 40.0;
    
    // Very basic wrapping
    let words: Vec<&str> = event.description.split_whitespace().collect();
    let mut line = String::new();
    
    for word in words {
        let test_line = if line.is_empty() {
            String::from(word)
        } else {
            format!("{} {}", line, word)
        };
        
        let dims = measure_text(&test_line, None, font_size as u16, 1.0);
        if dims.width > max_width {
            draw_text(&line, x + 20.0, current_y, font_size, colors::TEXT);
            current_y += 25.0;
            line = String::from(word);
        } else {
            line = test_line;
        }
    }
    if !line.is_empty() {
        draw_text(&line, x + 20.0, current_y, font_size, colors::TEXT);
    }
    
    // Draw Choices
    let mouse_pos = mouse_position();
    let btn_height = 50.0;

    let start_btn_y = y + modal_h - 20.0 - btn_height;
    
    if event.choices.is_empty() {
        // "Continue" Button
        let btn_w = 150.0;
        let btn_rect = Rect::new(x + modal_w - btn_w - 20.0, start_btn_y, btn_w, btn_height);
        let hovered = btn_rect.contains(vec2(mouse_pos.0, mouse_pos.1));
        
        if hovered && is_mouse_button_pressed(MouseButton::Left) {
            return Some(UiAction::ResolveEventChoice { event_id: event.id, choice_index: 0 });
        }
        
        draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, if hovered { colors::HOVERED } else { colors::PANEL });
        draw_text("Continue", btn_rect.x + 30.0, btn_rect.y + 32.0, 20.0, colors::TEXT_BRIGHT);
        
    } else {
        // Multiple choices
        // Render them vertically stacked if descriptions are long?
        // Or horizontally?
        // Let's do horizontal but check width. If many choices, stick to horizontal row.
        // Or vertical stack is safer for text.
        // Let's do Vertical stack for clarity, from bottom up.
        
        let btn_w = modal_w - 40.0;
        let mut btn_y = y + modal_h - 20.0 - btn_height;
        
        // Reverse iterate to stack from bottom?
        for (i, choice) in event.choices.iter().enumerate().rev() {
             let btn_rect = Rect::new(x + 20.0, btn_y, btn_w, btn_height);
             let hovered = btn_rect.contains(vec2(mouse_pos.0, mouse_pos.1));
             
             if hovered && is_mouse_button_pressed(MouseButton::Left) {
                 return Some(UiAction::ResolveEventChoice { event_id: event.id, choice_index: i });
             }
             
             draw_rectangle(btn_rect.x, btn_rect.y, btn_rect.w, btn_rect.h, if hovered { colors::HOVERED } else { colors::PANEL });
             
             draw_text(&choice.label, btn_rect.x + 10.0, btn_rect.y + 32.0, 20.0, colors::TEXT_BRIGHT);
             
             // Draw reputation/cost hint
             if choice.reputation_change != 0 {
                 let rep_text = format!("Rep: {:+}", choice.reputation_change);
                 draw_text(&rep_text, btn_rect.x + btn_w - 100.0, btn_rect.y + 32.0, 18.0, if choice.reputation_change > 0 { colors::POSITIVE } else { colors::NEGATIVE });
             }
             
             btn_y -= btn_height + 10.0;
        }
    }
    
    None
}
