use crate::state::GameplayState;
use crate::ui::theme::{color, scale, space, Tone};
use crate::ui::widgets::{button_at, draw_card};
use crate::ui::{colors, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_career_summary(state: &GameplayState) -> Option<UiAction> {
    let screen_w = screen_width();
    let screen_h = screen_height();

    // Background
    draw_rectangle(0., 0., screen_w, screen_h, colors::BACKGROUND);

    // Calculate Score
    let funds = state.funds.balance;
    let avg_happiness = if state.tenants.is_empty() {
        0
    } else {
        state.tenants.iter().map(|t| t.happiness).sum::<i32>() / state.tenants.len() as i32
    };
    let reputation = state
        .city
        .neighborhoods
        .iter()
        .map(|n| n.reputation)
        .sum::<i32>()
        / state.city.neighborhoods.len().max(1) as i32;
    let achievements_unlocked = state.achievements.unlocked.len();

    let score =
        funds + (avg_happiness * 100) + (reputation * 50) + (achievements_unlocked as i32 * 1000);

    // Determine Rank
    let rank = if score > 50000 {
        "Real Estate Tycoon"
    } else if score > 25000 {
        "Successful Landlord"
    } else if score > 10000 {
        "Property Manager"
    } else if score > 0 {
        "Struggling Owner"
    } else {
        "Slumlord"
    };

    let color = if score > 25000 {
        colors::POSITIVE
    } else if score > 0 {
        colors::WARNING
    } else {
        colors::NEGATIVE
    };

    // Header
    let cx = screen_w / 2.0;
    let mut y = 60.0;

    draw_text_centered("CAREER SUMMARY", cx, y, 50.0, colors::TEXT_BRIGHT);
    y += 60.0;

    draw_text_centered(&format!("Rank: {}", rank), cx, y, 40.0, color);
    y += 50.0;

    draw_text_centered(
        &format!("Final Score: {}", score),
        cx,
        y,
        30.0,
        colors::TEXT,
    );
    y += 60.0;

    // Stats Grid
    let stats_y = y;
    let col_w = 200.0;
    let start_x = cx - (col_w * 2.5); // 5 columns
                                      // Funds, Happiness, Reputation, Months, Missions

    draw_stat(
        "Funds",
        &format!("${}", funds),
        start_x,
        stats_y,
        colors::POSITIVE,
    );
    draw_stat(
        "Happiness",
        &format!("{}%", avg_happiness),
        start_x + col_w,
        stats_y,
        colors::TEXT,
    );
    draw_stat(
        "Avg Rep",
        &format!("{}", reputation),
        start_x + col_w * 2.0,
        stats_y,
        colors::ACCENT,
    );
    draw_stat(
        "Months",
        &format!("{}", state.current_tick),
        start_x + col_w * 3.0,
        stats_y,
        colors::TEXT_DIM,
    );
    draw_stat(
        "Missions",
        &format!("{}", state.missions.completed_missions().len()),
        start_x + col_w * 4.0,
        stats_y,
        colors::TEXT_BRIGHT,
    );

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
        let rect = Rect::new(rect_x, ach_y, ach_w, ach_h);
        draw_card(rect, unlocked);

        if unlocked {
            draw_ui_text(
                &achievement.name,
                rect_x + space::SM,
                ach_y + 25.0,
                scale::HEADING,
                color::TEXT_BRIGHT,
            );
            // Wrap description roughly
            draw_ui_text(
                &achievement.description,
                rect_x + space::SM,
                ach_y + 50.0,
                scale::CAPTION,
                color::TEXT_DIM,
            );
        } else {
            draw_ui_text(
                "???",
                rect_x + space::SM,
                ach_y + 25.0,
                scale::HEADING,
                color::TEXT_DIM,
            );
            draw_ui_text(
                "Locked",
                rect_x + space::SM,
                ach_y + 50.0,
                scale::CAPTION,
                color::TEXT_DIM,
            );
        }

        col += 1;
        if col >= cols {
            col = 0;
            ach_y += ach_h + gap;
        }
    }

    // Back to Menu Button - positioned below all achievements
    // Add one more row height if there was a partial last row
    let final_ach_y = if col > 0 { ach_y + ach_h + gap } else { ach_y };

    let btn_w = 250.0;
    let btn_h = 55.0;
    let btn_x = cx - btn_w / 2.0;
    let btn_y = final_ach_y + 30.0; // After all achievements

    let rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
    if button_at(rect, "RETURN TO MENU", true, Tone::Positive) {
        return Some(UiAction::ReturnToMenu);
    }

    None
}

fn draw_text_centered(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    macroquad_toolkit::ui::draw_text_centered(
        text,
        cx,
        y,
        macroquad_toolkit::ui::TextStyle::new(size, color),
    );
}

fn draw_stat(label: &str, value: &str, x: f32, y: f32, color: Color) {
    draw_ui_text(label, x, y, 16.0, colors::TEXT_DIM);
    draw_ui_text(value, x, y + 25.0, 24.0, color);
}
