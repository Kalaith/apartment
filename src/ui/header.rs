use macroquad::prelude::*;
use super::{common::*, UiAction};

pub fn draw_header(
    money: i32,
    tick: u32,
    building_name: &str,
    occupancy: usize,
    total_units: usize,
) -> Option<UiAction> {
    let mut action = None;
    let w = screen_width();
    
    // Background
    draw_rectangle(0.0, 0.0, w, layout::HEADER_HEIGHT, colors::PANEL_HEADER);
    
    // Building name
    draw_text(building_name, 20.0, 38.0, 28.0, colors::TEXT);
    
    // Money
    let money_text = format!("${}", money);
    let money_color = if money < 500 { colors::WARNING } 
                      else if money < 0 { colors::NEGATIVE }
                      else { colors::POSITIVE };
    draw_text(&money_text, 250.0, 38.0, 24.0, money_color);
    
    // Month
    let month_text = format!("Month {}", tick);
    draw_text(&month_text, 400.0, 38.0, 20.0, colors::TEXT_DIM);
    
    // Occupancy
    let occ_text = format!("Occupancy: {}/{}", occupancy, total_units);
    draw_text(&occ_text, 520.0, 38.0, 20.0, colors::TEXT_DIM);
    
    // End Turn button (mouse-clickable)
    let btn_x = w - 150.0;
    let btn_y = 10.0;
    if button(btn_x, btn_y, 130.0, 40.0, "End Month", true) {
        action = Some(UiAction::EndTurn);
    }
    
    // Keyboard hint
    draw_text("(Space)", btn_x + 30.0, btn_y + 55.0, 12.0, colors::TEXT_DIM);
    
    // Bottom border
    draw_line(0.0, layout::HEADER_HEIGHT, w, layout::HEADER_HEIGHT, 2.0, colors::ACCENT);
    
    action
}
