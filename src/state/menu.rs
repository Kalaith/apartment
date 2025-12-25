use macroquad::prelude::*;
use super::{GameplayState, StateTransition};
use crate::save::{has_save_game, load_game};
use crate::assets::AssetManager;

pub struct MenuState {
    has_save: bool,
}

impl MenuState {
    pub fn new() -> Self {
        Self {
            has_save: has_save_game(),
        }
    }

    pub fn update(&mut self, _assets: &AssetManager) -> Option<StateTransition> {
        let button_w = 200.0;
        let button_h = 50.0;
        let center_x = screen_width() / 2.0 - button_w / 2.0;
        let start_y = screen_height() / 2.0;
        
        // Continue Game Button (if save exists)
        if self.has_save {
            let continue_y = start_y - 60.0;
            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                if mx >= center_x && mx <= center_x + button_w
                    && my >= continue_y && my <= continue_y + button_h
                {
                    if let Ok(state) = load_game() {
                        return Some(StateTransition::ToGameplay(state));
                    } else {
                        // Failed to load, maybe delete save?
                        // For now just ignore and maybe user clicks New Game
                        eprintln!("Failed to load save");
                    }
                }
            }
        }

        // New Game Button
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if mx >= center_x && mx <= center_x + button_w
                && my >= start_y && my <= start_y + button_h
            {
                return Some(StateTransition::ToGameplay(GameplayState::new()));
            }
        }

        None
    }

    pub fn draw(&self, assets: &AssetManager) {
        // Draw title
        let title = "APARTMENT";
        let title_size = 60.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        
        // Use background if available
        if let Some(bg) = assets.get_texture("title_background") {
             draw_texture_ex(
                bg,
                0.0,
                0.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(screen_width(), screen_height())),
                    ..Default::default()
                }
            );
        } else {
             clear_background(Color::from_rgba(30, 30, 35, 255));
        }

       // Use logo if available
        if let Some(logo) = assets.get_texture("title_logo") {
             draw_texture_ex(
                logo,
                screen_width() / 2.0 - 256.0,
                screen_height() / 4.0 - 128.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(512.0, 256.0)),
                    ..Default::default()
                }
            );
        } else {
            draw_text(
                title,
                screen_width() / 2.0 - title_width / 2.0,
                screen_height() / 3.0,
                title_size,
                WHITE,
            );
        }

        let button_w = 200.0;
        let button_h = 50.0;
        let center_x = screen_width() / 2.0 - button_w / 2.0;
        let start_y = screen_height() / 2.0;

        // Continue Button
        if self.has_save {
            let continue_y = start_y - 60.0;
            draw_rectangle(center_x, continue_y, button_w, button_h, Color::from_rgba(60, 100, 60, 255));
            
            let label = "Continue";
            let label_size = 30.0;
            let label_width = measure_text(label, None, label_size as u16, 1.0).width;
            draw_text(
                label,
                center_x + button_w / 2.0 - label_width / 2.0,
                continue_y + button_h / 2.0 + 10.0,
                label_size,
                WHITE,
            );
        }

        // New Game Button
        draw_rectangle(center_x, start_y, button_w, button_h, Color::from_rgba(70, 70, 80, 255));

        let label = "New Game";
        let label_size = 30.0;
        let label_width = measure_text(label, None, label_size as u16, 1.0).width;
        draw_text(
            label,
            center_x + button_w / 2.0 - label_width / 2.0,
            start_y + button_h / 2.0 + 10.0,
            label_size,
            WHITE,
        );
    }
}
