
use macroquad::prelude::*;
use super::StateTransition;
use crate::assets::AssetManager;
use crate::narrative::{LegacyEvent, BuildingAward};

pub struct ResultsState {
    pub total_income: i32,
    pub tenants_housed: u32,
    pub tenants_left: u32,
    pub won: bool,
    pub months_played: u32,
    pub missions_completed: u32,
    pub legacy_events: Vec<LegacyEvent>,
    pub awards: Vec<BuildingAward>,
}

impl ResultsState {
    /// Create a full career summary
    pub fn career_summary(
        total_income: i32,
        tenants_housed: u32,
        tenants_left: u32,
        won: bool,
        months_played: u32,
        missions_completed: u32,
        legacy_events: Vec<LegacyEvent>,
        awards: Vec<BuildingAward>,
    ) -> Self {
        Self {
            total_income,
            tenants_housed,
            tenants_left,
            won,
            months_played,
            missions_completed,
            legacy_events,
            awards,
        }
    }

    pub fn update(&mut self, assets: &AssetManager) -> Option<StateTransition> {
        // Wait for assets if needed
        if !assets.loaded {
            return None;
        }
        // Check for Return to Menu click
        let button_x = screen_width() / 2.0 - 100.0;
        let button_y = screen_height() - 100.0;
        let button_w = 200.0;
        let button_h = 50.0;

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= button_x && mx <= button_x + button_w
                && my >= button_y && my <= button_y + button_h
            {
                return Some(StateTransition::ToMenu);
            }
        }

        None
    }

    pub fn draw(&self, assets: &AssetManager) {
        // Only draw if assets are loaded
        if !assets.loaded {
            clear_background(Color::from_rgba(20, 20, 30, 255));
            draw_text("Loading...", screen_width() / 2.0 - 50.0, screen_height() / 2.0, 24.0, WHITE);
            return;
        }
        
        clear_background(Color::from_rgba(20, 20, 30, 255));
        
        // Title: Career Summary
        let title = "CAREER SUMMARY";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            screen_width() / 2.0 - title_width / 2.0,
            60.0,
            title_size,
            WHITE,
        );
        
        // Outcome
        let outcome = if self.won { "SUCCESS" } else { "BANKRUPT" };
        let outcome_color = if self.won { GREEN } else { RED };
        let outcome_size = 36.0;
        let outcome_width = measure_text(outcome, None, outcome_size as u16, 1.0).width;
        draw_text(
            outcome,
            screen_width() / 2.0 - outcome_width / 2.0,
            110.0,
            outcome_size,
            outcome_color,
        );

        let col1_x = 50.0;
        let col2_x = screen_width() / 2.0 + 20.0;
        let mut y = 160.0;
        let line_height = 28.0;

        // Stats column
        draw_text("STATISTICS", col1_x, y, 24.0, GOLD);
        y += line_height;
        draw_text(&format!("Months Played: {}", self.months_played), col1_x, y, 20.0, WHITE);
        y += line_height;
        draw_text(&format!("Total Income: ${}", self.total_income), col1_x, y, 20.0, WHITE);
        y += line_height;
        draw_text(&format!("Tenants Housed: {}", self.tenants_housed), col1_x, y, 20.0, WHITE);
        y += line_height;
        draw_text(&format!("Tenants Left: {}", self.tenants_left), col1_x, y, 20.0, WHITE);
        y += line_height;
        draw_text(&format!("Missions Completed: {}", self.missions_completed), col1_x, y, 20.0, WHITE);

        // Awards column
        let mut awards_y = 160.0;
        draw_text("AWARDS", col2_x, awards_y, 24.0, GOLD);
        awards_y += line_height;
        
        if self.awards.is_empty() {
            draw_text("No awards yet", col2_x, awards_y, 18.0, GRAY);
        } else {
            for award in self.awards.iter().take(5) {
                draw_text(&format!("{} - {} ({})", award.year, award.title, award.building_name), col2_x, awards_y, 18.0, WHITE);
                awards_y += line_height;
            }
        }

        // Legacy Events (Story Archive)
        let events_y_start = 340.0;
        draw_text("STORY ARCHIVE", col1_x, events_y_start, 24.0, GOLD);
        let mut event_y = events_y_start + line_height;
        
        if self.legacy_events.is_empty() {
            draw_text("No major events recorded", col1_x, event_y, 18.0, GRAY);
        } else {
            for event in self.legacy_events.iter().take(6) {
                let year = 2024 + (event.month / 12);
                draw_text(&format!("Year {} - {}", year, event.title), col1_x, event_y, 18.0, WHITE);
                event_y += line_height;
            }
        }

        // Draw Return to Menu button
        let button_x = screen_width() / 2.0 - 100.0;
        let button_y = screen_height() - 100.0;
        let button_w = 200.0;
        let button_h = 50.0;

        draw_rectangle(button_x, button_y, button_w, button_h, Color::from_rgba(70, 70, 80, 255));

        let label = "Return to Menu";
        let label_size = 24.0;
        let label_width = measure_text(label, None, label_size as u16, 1.0).width;
        draw_text(
            label,
            button_x + button_w / 2.0 - label_width / 2.0,
            button_y + button_h / 2.0 + 8.0,
            label_size,
            WHITE,
        );
    }
}

