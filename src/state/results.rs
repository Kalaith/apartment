use macroquad::prelude::*;
use super::StateTransition;

pub struct ResultsState {
    pub total_income: i32,
    pub tenants_housed: u32,
    pub tenants_left: u32,
    pub final_reputation: f32,
    pub won: bool,
}

impl ResultsState {
    pub fn new(total_income: i32, tenants_housed: u32, tenants_left: u32, won: bool) -> Self {
        Self {
            total_income,
            tenants_housed,
            tenants_left,
            final_reputation: 0.0,
            won,
        }
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        // Check for Return to Menu click
        let button_x = screen_width() / 2.0 - 100.0;
        let button_y = screen_height() / 2.0 + 100.0;
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

    pub fn draw(&self) {
        let title = if self.won { "SUCCESS!" } else { "BANKRUPT" };
        let title_color = if self.won { GREEN } else { RED };
        let title_size = 60.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;

        draw_text(
            title,
            screen_width() / 2.0 - title_width / 2.0,
            screen_height() / 4.0,
            title_size,
            title_color,
        );

        // Draw summary stats
        let stats_x = screen_width() / 2.0 - 150.0;
        let stats_y = screen_height() / 2.0 - 50.0;
        draw_text(&format!("Total Income: ${}", self.total_income), stats_x, stats_y, 24.0, WHITE);
        draw_text(&format!("Tenants Housed: {}", self.tenants_housed), stats_x, stats_y + 30.0, 24.0, WHITE);
        draw_text(&format!("Tenants Left: {}", self.tenants_left), stats_x, stats_y + 60.0, 24.0, WHITE);

        // Draw Return to Menu button
        let button_x = screen_width() / 2.0 - 100.0;
        let button_y = screen_height() / 2.0 + 100.0;
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
