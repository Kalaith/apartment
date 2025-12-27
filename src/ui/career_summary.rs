use macroquad::prelude::*;
use crate::state::GameplayState;
use crate::ui::{UiAction, colors};
use crate::ui::common::colored_button;

pub fn draw_career_summary(state: &GameplayState) -> Option<UiAction> {
    let screen_w = screen_width();
    let screen_h = screen_height();
    
    // Background
    draw_rectangle(0., 0., screen_w, screen_h, colors::BACKGROUND);
    
    // Calculate Score
    let funds = state.funds.balance;
    let avg_happiness = if state.tenants.is_empty() { 0 } else { state.tenants.iter().map(|t| t.happiness).sum::<i32>() / state.tenants.len() as i32 };
    let reputation = state.city.neighborhoods.iter().map(|n| n.reputation).sum::<i32>() / state.city.neighborhoods.len().max(1) as i32;
    let achievements_unlocked = state.achievements.unlocked.len();
    
    let score = funds + (avg_happiness * 100) + (reputation * 50) + (achievements_unlocked as i32 * 1000);
    
    // Determine Rank
    let rank = if score > 50000 { "Real Estate Tycoon" }
               else if score > 25000 { "Successful Landlord" }
               else if score > 10000 { "Property Manager" }
               else if score > 0 { "Struggling Owner" }
               else { "Slumlord" };
               
    let color = if score > 25000 { colors::POSITIVE } else if score > 0 { colors::WARNING } else { colors::NEGATIVE };
    
    // Header
    let cx = screen_w / 2.0;
    let mut y = 60.0;
    
    draw_text_centered("CAREER SUMMARY", cx, y, 50.0, colors::TEXT_BRIGHT);
    y += 60.0;
    
    draw_text_centered(&format!("Rank: {}", rank), cx, y, 40.0, color);
    y += 50.0;
    
    draw_text_centered(&format!("Final Score: {}", score), cx, y, 30.0, colors::TEXT);
    y += 60.0;
    
    // Stats Grid
    let stats_y = y;
    let col_w = 200.0;
    let start_x = cx - (col_w * 2.5); // 5 columns
    // Funds, Happiness, Reputation, Months, Missions
    
    draw_stat("Funds", &format!("${}", funds), start_x, stats_y, colors::POSITIVE);
    draw_stat("Happiness", &format!("{}%", avg_happiness), start_x + col_w, stats_y, colors::TEXT);
    draw_stat("Avg Rep", &format!("{}", reputation), start_x + col_w * 2.0, stats_y, colors::ACCENT);
    draw_stat("Months", &format!("{}", state.current_tick), start_x + col_w * 3.0, stats_y, colors::TEXT_DIM);
    draw_stat("Missions", &format!("{}", state.missions.completed_missions().len()), start_x + col_w * 4.0, stats_y, colors::TEXT_BRIGHT);
    
    y += 100.0;
    
    // Achievements
    draw_text_centered("Achievements Unlocked", cx, y, 30.0, colors::TEXT_BRIGHT);
    y += 40.0;
    
    let ach_w = 250.0;
    let ach_h = 80.0;
    let gap = 20.0;
    let cols = ((screen_w - 100.0) / (ach_w + gap)).floor() as usize;
    let start_ach_x = (screen_w - (cols as f32 * (ach_w + gap))) / 2.0;
    
    let mut col = 0;
    let mut ach_y = y;
    
    for achievement in &state.achievements.list {
        let unlocked = state.achievements.is_unlocked(&achievement.id);
        let rect_x = start_ach_x + (col as f32 * (ach_w + gap));
        
        // Draw card
        let bg_color = if unlocked { colors::PANEL } else { Color::new(0.1, 0.1, 0.1, 1.0) };
        let border_color = if unlocked { colors::ACCENT } else { colors::TEXT_DIM };
        
        draw_rectangle(rect_x, ach_y, ach_w, ach_h, bg_color);
        draw_rectangle_lines(rect_x, ach_y, ach_w, ach_h, 2.0, border_color);
        
        if unlocked {
            draw_text(&achievement.name, rect_x + 10.0, ach_y + 25.0, 20.0, colors::TEXT_BRIGHT);
            // Wrap description roughly
            draw_text(&achievement.description, rect_x + 10.0, ach_y + 50.0, 14.0, colors::TEXT_DIM);
        } else {
             draw_text("???", rect_x + 10.0, ach_y + 25.0, 20.0, colors::TEXT_DIM);
             draw_text("Locked", rect_x + 10.0, ach_y + 50.0, 14.0, colors::TEXT_DIM);
        }
        
        col += 1;
        if col >= cols {
            col = 0;
            ach_y += ach_h + gap;
        }
    }
    
    // Back to Menu Button
    let btn_w = 200.0;
    let btn_h = 50.0;
    let btn_x = cx - btn_w / 2.0;
    let btn_y = screen_h - 80.0;
    
    if colored_button(btn_x, btn_y, btn_w, btn_h, "MAIN MENU", true, colors::PANEL, colors::TEXT_BRIGHT) {
        return Some(UiAction::ReturnToMenu); 
    }
    
    None
}

fn draw_text_centered(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, cx - dims.width / 2.0, y, size, color);
}

fn draw_stat(label: &str, value: &str, x: f32, y: f32, color: Color) {
    draw_text(label, x, y, 16.0, colors::TEXT_DIM);
    draw_text(value, x, y + 25.0, 24.0, color);
}
