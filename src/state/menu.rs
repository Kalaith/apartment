use macroquad::prelude::*;
use super::{GameplayState, StateTransition};

pub struct MenuState {
    // No fields needed for MVP
}

impl MenuState {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        // Check for New Game button click
        let button_x = screen_width() / 2.0 - 100.0;
        let button_y = screen_height() / 2.0;
        let button_w = 200.0;
        let button_h = 50.0;

        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= button_x && mx <= button_x + button_w
                && my >= button_y && my <= button_y + button_h
            {
                return Some(StateTransition::ToGameplay(GameplayState::new()));
            }
        }

        None
    }

    pub fn draw(&self) {
        // Draw title
        let title = "APARTMENT";
        let title_size = 60.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        draw_text(
            title,
            screen_width() / 2.0 - title_width / 2.0,
            screen_height() / 3.0,
            title_size,
            WHITE,
        );

        // Draw "New Game" button
        let button_x = screen_width() / 2.0 - 100.0;
        let button_y = screen_height() / 2.0;
        let button_w = 200.0;
        let button_h = 50.0;

        draw_rectangle(button_x, button_y, button_w, button_h, Color::from_rgba(70, 70, 80, 255));

        let label = "New Game";
        let label_size = 30.0;
        let label_width = measure_text(label, None, label_size as u16, 1.0).width;
        draw_text(
            label,
            button_x + button_w / 2.0 - label_width / 2.0,
            button_y + button_h / 2.0 + 10.0,
            label_size,
            WHITE,
        );
    }
}
