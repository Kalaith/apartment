use macroquad::prelude::*;
use super::{common::*, UiAction};
use crate::assets::AssetManager;

pub fn draw_header(
    money: i32,
    tick: u32,
    building_name: &str,
    occupancy: usize,
    total_units: usize,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;
    let w = screen_width();
    
    // Background
    draw_rectangle(0.0, 0.0, w, layout::HEADER_HEIGHT, colors::PANEL_HEADER);
    
    // Building name
    draw_text(building_name, 20.0, 38.0, 28.0, colors::TEXT);
    
    // Money
    let mut current_x = 250.0;
    if let Some(icon) = assets.get_texture("icon_money") {
        draw_texture_ex(icon, current_x, 15.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(32.0, 32.0)),
            ..Default::default()
        });
        current_x += 35.0;
    }
    
    let money_text = format!("${}", money);
    let money_color = if money < 500 { colors::WARNING } 
                      else if money < 0 { colors::NEGATIVE }
                      else { colors::POSITIVE };
    draw_text(&money_text, current_x, 38.0, 24.0, money_color);
    
    // Month
    let mut month_x = 420.0;
    if let Some(icon) = assets.get_texture("icon_calendar") {
        draw_texture_ex(icon, month_x, 18.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(28.0, 28.0)),
            ..Default::default()
        });
        month_x += 32.0;
    }
    
    let month_text = format!("Month {}", tick);
    draw_text(&month_text, month_x, 38.0, 20.0, colors::TEXT_DIM);
    
    // Occupancy
    let mut occ_x = 560.0;
     if let Some(icon) = assets.get_texture("icon_key") {
        draw_texture_ex(icon, occ_x, 18.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(28.0, 28.0)),
            ..Default::default()
        });
        occ_x += 32.0;
    }
    let occ_text = format!("{}/{}", occupancy, total_units);
    draw_text(&occ_text, occ_x, 38.0, 20.0, colors::TEXT_DIM);
    
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
