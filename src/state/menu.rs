use macroquad::prelude::*;
use super::{GameplayState, StateTransition};
use crate::save::{has_save_game, load_game, load_player_progress, PlayerProgress};
use crate::assets::AssetManager;
use crate::data::templates::{load_templates, BuildingTemplate};

pub struct MenuState {
    has_save: bool,
    progress: PlayerProgress,
    templates: Vec<BuildingTemplate>,
}

impl MenuState {
    pub fn new() -> Self {
        let templates = load_templates()
            .map(|t| t.templates)
            .unwrap_or_default();
        
        Self {
            has_save: has_save_game(),
            progress: load_player_progress(),
            templates,
        }
    }

    pub fn update(&mut self, _assets: &AssetManager, config: &crate::data::config::GameConfig) -> Option<StateTransition> {
        let (mx, my) = mouse_position();
        let clicked = is_mouse_button_pressed(MouseButton::Left);
        
        // Layout constants
        let card_w = 280.0;
        let card_h = 120.0;
        let card_spacing = 20.0;
        let start_y = screen_height() * 0.45;
        
        // Calculate total width to center cards
        let total_width = self.templates.len() as f32 * card_w + (self.templates.len() - 1) as f32 * card_spacing;
        let start_x = (screen_width() - total_width) / 2.0;
        
        // Building cards
        for (i, template) in self.templates.iter().enumerate() {
            let x = start_x + i as f32 * (card_w + card_spacing);
            let y = start_y;
            
            let is_unlocked = self.progress.is_unlocked(&template.id);
            
            if is_unlocked && clicked && mx >= x && mx <= x + card_w && my >= y && my <= y + card_h {
                // Start game with this building template
                let state = GameplayState::new_with_template(config.clone(), template.clone());
                return Some(StateTransition::ToGameplay(state));
            }
        }
        
        // Continue button (if save exists)
        if self.has_save {
            let btn_w = 200.0;
            let btn_h = 45.0;
            let btn_x = screen_width() / 2.0 - btn_w / 2.0;
            let btn_y = start_y + card_h + 40.0;
            
            if clicked && mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h {
                if let Ok(state) = load_game() {
                    return Some(StateTransition::ToGameplay(state));
                } else {
                    eprintln!("Failed to load save");
                }
            }
        }
        
        // Quit button
        let quit_btn_w = 150.0;
        let quit_btn_h = 40.0;
        let quit_btn_x = screen_width() / 2.0 - quit_btn_w / 2.0;
        let quit_btn_y = screen_height() - 80.0;
        
        if clicked && mx >= quit_btn_x && mx <= quit_btn_x + quit_btn_w && my >= quit_btn_y && my <= quit_btn_y + quit_btn_h {
            std::process::exit(0);
        }

        None
    }

    pub fn draw(&self, assets: &AssetManager) {
        // Background
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
            clear_background(Color::from_rgba(25, 25, 30, 255));
        }

        // Logo or Title
        if let Some(logo) = assets.get_texture("title_logo") {
            draw_texture_ex(
                logo,
                screen_width() / 2.0 - 200.0,
                40.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(400.0, 180.0)),
                    ..Default::default()
                }
            );
        } else {
            let title = "APARTMENT";
            let title_size = 60.0;
            let title_width = measure_text(title, None, title_size as u16, 1.0).width;
            draw_text(title, screen_width() / 2.0 - title_width / 2.0, 120.0, title_size, WHITE);
        }
        
        // Section title
        let section_title = "Select Building";
        let section_size = 28.0;
        let section_width = measure_text(section_title, None, section_size as u16, 1.0).width;
        draw_text(section_title, screen_width() / 2.0 - section_width / 2.0, screen_height() * 0.40, section_size, Color::from_rgba(200, 200, 200, 255));

        // Layout constants
        let card_w = 280.0;
        let card_h = 120.0;
        let card_spacing = 20.0;
        let start_y = screen_height() * 0.45;
        
        let total_width = self.templates.len() as f32 * card_w + (self.templates.len() - 1) as f32 * card_spacing;
        let start_x = (screen_width() - total_width) / 2.0;
        
        let (mx, my) = mouse_position();
        
        // Draw building cards
        for (i, template) in self.templates.iter().enumerate() {
            let x = start_x + i as f32 * (card_w + card_spacing);
            let y = start_y;
            
            let is_unlocked = self.progress.is_unlocked(&template.id);
            let is_completed = self.progress.completed_buildings.contains(&template.id);
            let is_hovered = mx >= x && mx <= x + card_w && my >= y && my <= y + card_h;
            
            // Card background
            let bg_color = if !is_unlocked {
                Color::from_rgba(40, 40, 45, 200) // Locked - dark
            } else if is_hovered {
                Color::from_rgba(70, 80, 100, 255) // Hovered
            } else {
                Color::from_rgba(50, 55, 65, 255) // Normal unlocked
            };
            
            draw_rectangle(x, y, card_w, card_h, bg_color);
            
            // Border color based on difficulty
            let border_color = match template.difficulty.as_str() {
                "Easy" => Color::from_rgba(80, 180, 80, 255),
                "Medium" => Color::from_rgba(200, 180, 60, 255),
                "Hard" => Color::from_rgba(200, 80, 80, 255),
                _ => Color::from_rgba(100, 100, 100, 255),
            };
            draw_rectangle_lines(x, y, card_w, card_h, 3.0, border_color);
            
            // Building name
            let name_color = if is_unlocked { WHITE } else { Color::from_rgba(100, 100, 100, 255) };
            draw_text(&template.name, x + 15.0, y + 30.0, 22.0, name_color);
            
            // Difficulty badge
            let diff_color = border_color;
            draw_text(&template.difficulty, x + 15.0, y + 52.0, 14.0, diff_color);
            
            // Description (truncated)
            let desc = if template.description.len() > 40 {
                format!("{}...", &template.description[..37])
            } else {
                template.description.clone()
            };
            let desc_color = if is_unlocked { Color::from_rgba(180, 180, 180, 255) } else { Color::from_rgba(80, 80, 80, 255) };
            draw_text(&desc, x + 15.0, y + 75.0, 12.0, desc_color);
            
            // Units count
            let units = template.apartments.len();
            draw_text(&format!("{} units", units), x + 15.0, y + 100.0, 14.0, desc_color);
            
            // Locked overlay
            if !is_unlocked {
                draw_text("🔒 LOCKED", x + card_w - 90.0, y + 30.0, 16.0, Color::from_rgba(150, 100, 100, 255));
            }
            
            // Completed checkmark
            if is_completed {
                draw_text("✓", x + card_w - 30.0, y + 30.0, 24.0, Color::from_rgba(80, 200, 80, 255));
            }
        }
        
        // Continue button (if save exists)
        if self.has_save {
            let btn_w = 200.0;
            let btn_h = 45.0;
            let btn_x = screen_width() / 2.0 - btn_w / 2.0;
            let btn_y = start_y + card_h + 40.0;
            
            let hovered = mx >= btn_x && mx <= btn_x + btn_w && my >= btn_y && my <= btn_y + btn_h;
            let bg = if hovered { Color::from_rgba(60, 100, 60, 255) } else { Color::from_rgba(50, 80, 50, 255) };
            
            draw_rectangle(btn_x, btn_y, btn_w, btn_h, bg);
            draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, Color::from_rgba(80, 140, 80, 255));
            
            let label = "Continue Saved Game";
            let label_width = measure_text(label, None, 18, 1.0).width;
            draw_text(label, btn_x + (btn_w - label_width) / 2.0, btn_y + 28.0, 18.0, WHITE);
        }
        
        // Quit button
        let quit_btn_w = 150.0;
        let quit_btn_h = 40.0;
        let quit_btn_x = screen_width() / 2.0 - quit_btn_w / 2.0;
        let quit_btn_y = screen_height() - 80.0;
        
        let quit_hovered = mx >= quit_btn_x && mx <= quit_btn_x + quit_btn_w && my >= quit_btn_y && my <= quit_btn_y + quit_btn_h;
        let quit_bg = if quit_hovered { Color::from_rgba(100, 60, 60, 255) } else { Color::from_rgba(70, 45, 45, 255) };
        
        draw_rectangle(quit_btn_x, quit_btn_y, quit_btn_w, quit_btn_h, quit_bg);
        draw_rectangle_lines(quit_btn_x, quit_btn_y, quit_btn_w, quit_btn_h, 2.0, Color::from_rgba(140, 80, 80, 255));
        
        let label = "Quit";
        let label_width = measure_text(label, None, 18, 1.0).width;
        draw_text(label, quit_btn_x + (quit_btn_w - label_width) / 2.0, quit_btn_y + 26.0, 18.0, WHITE);
    }
}
