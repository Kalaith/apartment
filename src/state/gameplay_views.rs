//! Game view rendering - split from gameplay.rs for maintainability

use macroquad::prelude::*;
use crate::assets::AssetManager;
use crate::ui::{
    Selection, FloatingText, colors,
    draw_header, draw_building_view, draw_apartment_panel, draw_hallway_panel,
    draw_application_panel, draw_notifications, draw_ownership_panel,
};
use crate::narrative::NotificationCategory;
use crate::ui::layout::HEADER_HEIGHT;

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// Main draw function - dispatches to appropriate view
    pub fn draw(&mut self, assets: &AssetManager) {
        match self.view_mode {
            ViewMode::Building => {
                self.draw_building_mode(assets);
            }
            ViewMode::CityMap => {
                if let Some(action) = crate::ui::city_view::draw_city_map(&self.city, assets, &self.narrative_events) {
                    self.handle_city_action(action);
                }
                
                 if let Some(action) = crate::ui::city_view::draw_portfolio_panel(&self.city, self.city.active_building_index, assets) {
                     self.handle_city_action(action);
                 }
            }
            ViewMode::Market => {
                 let listings: Vec<&crate::city::PropertyListing> = self.city.market.listings.iter().collect();
                 if let Some(action) = crate::ui::city_view::draw_market_panel(&listings, &self.city.neighborhoods, self.funds.balance, assets) {
                     self.handle_city_action(action);
                 }
            }
            ViewMode::Mail => {
                self.draw_mail_view(assets);
            }
            ViewMode::CareerSummary => {
                if let Some(action) = crate::ui::career_summary::draw_career_summary(self) {
                     self.pending_actions.push(action);
                }
            }
        }

        // Draw blocking narrative event modal (Phase 4)
        // Find first unread event that requires response
        let blocking_event = self.narrative_events.events.iter()
            .find(|e| !e.read && e.requires_response);
            
        if let Some(event) = blocking_event {
            if let Some(action) = crate::ui::event_modal::draw_event_modal(event) {
                self.pending_actions.push(action);
            }
        }

        // Draw notifications (hints, etc) on top
        if !self.tutorial.active {
            self.draw_notification_overlay();
        }
        
        draw_notifications(&self.event_log, self.current_tick, assets);
        
        // Floating text
        for text in &self.floating_texts {
            text.draw();
        }
        
        // Tutorial overlay (takes precedence)
        if self.tutorial.active && !self.tutorial.pending_messages.is_empty() {
            self.draw_tutorial_overlay(assets);
        }
        // Notification overlay (shows when tutorial is done/empty)
        else if self.notifications.has_pending() {
            self.draw_notification_overlay();
        }
        
        // Draw pause menu on top of everything if active
        if self.show_pause_menu {
            self.draw_pause_menu_overlay();
        }
    }

    pub(super) fn draw_building_mode(&mut self, assets: &AssetManager) {
        // Draw Header
        if let Some(action) = draw_header(
            self.funds.balance, 
            self.current_tick, 
            &self.building.name, 
            self.building.occupancy_count(), 
            self.building.apartments.len(), 
            assets
        ) {
            self.pending_actions.push(action);
        }
        
        // Draw Building View
        if let Some(action) = draw_building_view(&self.building, &self.tenants, &self.selection, assets) {
            self.pending_actions.push(action);
        }
        
        match self.selection {
            Selection::Apartment(id) => {
                 if let Some(apt) = self.building.get_apartment(id) {
                     let (action, new_scroll) = draw_apartment_panel(apt, &self.building, &self.tenants, self.funds.balance, 0.0, self.panel_scroll_offset, assets, &self.config, &self.tenant_network, &self.tenant_stories);
                     self.panel_scroll_offset = new_scroll;
                     if let Some(action) = action {
                         self.pending_actions.push(action);
                     }
                 }
            }
            Selection::Hallway => {
                let (action, new_scroll) = draw_hallway_panel(&self.building, self.funds.balance, 0.0, self.panel_scroll_offset, assets, &self.config);
                self.panel_scroll_offset = new_scroll;
                if let Some(action) = action {
                    self.pending_actions.push(action);
                }
            }
            Selection::Applications(filter) => {
                if let Some(action) = draw_application_panel(&self.applications, &self.building, filter, 0.0, assets) {
                    self.pending_actions.push(action);
                }
            }
            Selection::Ownership => {
                if let Some(action) = draw_ownership_panel(&self.building) {
                    self.pending_actions.push(action);
                }
            }
            _ => {}
        }
    }
    
    /// Draw mail view
    pub(super) fn draw_mail_view(&self, assets: &AssetManager) {
        // Use assets to check if textures are loaded
        let has_assets = assets.loaded;
        draw_rectangle(0.0, 0.0, screen_width(), HEADER_HEIGHT, colors::PANEL_HEADER);
        
        // Show a loading indicator if assets aren't ready
        if !has_assets {
            draw_text_ex(
                "Loading...",
                screen_width() - 100.0,
                35.0,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT_DIM,
                    ..Default::default()
                },
            );
        }
        
        draw_text_ex(
            "Mailbox",
            20.0,
            35.0,
            TextParams {
                font_size: 28,
                color: colors::TEXT,
                ..Default::default()
            },
        );
        
        // Unread count
        let unread = self.mailbox.unread_count();
        if unread > 0 {
            draw_text_ex(
                &format!("{} unread", unread),
                150.0,
                35.0,
                TextParams {
                    font_size: 16,
                    color: colors::WARNING,
                    ..Default::default()
                },
            );
        }
        
        // Mail list
        let start_y = HEADER_HEIGHT + 20.0;
        let mail_height = 80.0;
        
        let mail_to_show = self.mailbox.recent(10);
        
        for (i, mail) in mail_to_show.iter().enumerate() {
            let y = start_y + i as f32 * (mail_height + 10.0);
            
            let bg_color = if mail.read {
                Color::from_rgba(40, 40, 45, 255)
            } else {
                Color::from_rgba(50, 55, 70, 255)
            };
            draw_rectangle(20.0, y, screen_width() - 40.0, mail_height, bg_color);
            
            // Icon
            draw_text_ex(
                mail.mail_type.icon(),
                30.0,
                y + 30.0,
                TextParams {
                    font_size: 24,
                    color: colors::TEXT,
                    ..Default::default()
                },
            );
            
            // Subject
            draw_text_ex(
                &mail.subject,
                60.0,
                y + 25.0,
                TextParams {
                    font_size: 18,
                    color: if mail.read { colors::TEXT_DIM } else { colors::TEXT },
                    ..Default::default()
                },
            );
            
            // Sender
            draw_text_ex(
                &format!("From: {}", mail.sender),
                60.0,
                y + 45.0,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT_DIM,
                    ..Default::default()
                },
            );
            
            // Month
            draw_text_ex(
                &format!("Month {}", mail.month_received),
                screen_width() - 120.0,
                y + 25.0,
                TextParams {
                    font_size: 12,
                    color: colors::TEXT_DIM,
                    ..Default::default()
                },
            );
        }
        
        // Back hint
        draw_text_ex(
            "[Esc] Back to Building",
            20.0,
            screen_height() - 30.0,
            TextParams {
                font_size: 14,
                color: colors::TEXT_DIM,
                ..Default::default()
            },
        );
    }
    
    /// Draw the pause menu overlay (called from draw())
    pub(super) fn draw_pause_menu_overlay(&mut self) {
        // Semi-transparent overlay
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.7));
        
        // Menu panel
        let panel_w = 300.0;
        let panel_h = 280.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;
        
        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, colors::ACCENT);
        
        // Title
        let title = "PAUSED";
        let title_width = measure_text(title, None, 32, 1.0).width;
        draw_text(title, panel_x + (panel_w - title_width) / 2.0, panel_y + 40.0, 32.0, colors::TEXT_BRIGHT);
        
        let btn_w = 200.0;
        let btn_h = 40.0;
        let btn_x = panel_x + (panel_w - btn_w) / 2.0;
        let mut btn_y = panel_y + 70.0;
        
        // Resume button
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Resume") {
            self.show_pause_menu = false;
        }
        btn_y += 50.0;
        
        // Fullscreen toggle
        let fs_label = if self.is_fullscreen { "Windowed Mode" } else { "Fullscreen" };
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, fs_label) {
            self.is_fullscreen = !self.is_fullscreen;
            set_fullscreen(self.is_fullscreen);
        }
        btn_y += 50.0;
        
        // Save button  
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Save Game") {
            if crate::save::save_game(self).is_ok() {
                self.floating_texts.push(FloatingText::new(
                    "Game Saved!",
                    screen_width() / 2.0,
                    screen_height() / 2.0,
                    colors::POSITIVE,
                ));
            }
            self.show_pause_menu = false;
        }
        btn_y += 50.0;
        
        // Quit button
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Quit to Menu") {
            self.pending_quit_to_menu = true;
        }
        
        // ESC hint
        draw_text(
            "Press ESC to resume",
            panel_x + (panel_w - 140.0) / 2.0,
            panel_y + panel_h - 20.0,
            14.0,
            colors::TEXT_DIM
        );
    }
    
    /// Helper for drawing menu buttons
    pub(super) fn menu_button(&self, x: f32, y: f32, w: f32, h: f32, text: &str) -> bool {
        let mouse = mouse_position();
        let hovered = mouse.0 >= x && mouse.0 <= x + w && mouse.1 >= y && mouse.1 <= y + h;
        let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);
        
        let bg_color = if hovered {
            colors::HOVERED
        } else {
            Color::new(0.25, 0.25, 0.3, 1.0)
        };
        
        draw_rectangle(x, y, w, h, bg_color);
        draw_rectangle_lines(x, y, w, h, 2.0, colors::ACCENT);
        
        let text_width = measure_text(text, None, 20, 1.0).width;
        draw_text(text, x + (w - text_width) / 2.0, y + h / 2.0 + 6.0, 20.0, colors::TEXT);
        
        clicked
    }
    
    /// Draw the tutorial overlay
    pub(super) fn draw_tutorial_overlay(&mut self, assets: &AssetManager) {
        if self.tutorial.pending_messages.is_empty() {
            return;
        }
        
        let message = &self.tutorial.pending_messages[0];
        // "Uncle Artie"
        let npc_name = "Uncle Artie";
        
        // Layout - increased width
        let panel_w = 650.0;
        let panel_h = 180.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = screen_height() - panel_h - 20.0;
        
        // Background
        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 3.0, colors::ACCENT);
        
        // Portrait placeholder (use assets if available)
        let portrait_size = 100.0;
        let portrait_x = panel_x + 20.0;
        let portrait_y = panel_y + 40.0;
        
        if assets.loaded {
            // Draw a nice golden border around portrait when assets loaded
            draw_rectangle_lines(portrait_x - 2.0, portrait_y - 2.0, portrait_size + 4.0, portrait_size + 4.0, 2.0, colors::ACCENT);
        }
        draw_rectangle(portrait_x, portrait_y, portrait_size, portrait_size, GRAY);
        draw_text("ARTIE", portrait_x + 20.0, portrait_y + 50.0, 20.0, DARKGRAY);
            
        // Name
        draw_text(npc_name, panel_x + 140.0, panel_y + 30.0, 24.0, colors::TEXT_BRIGHT);
        
        // Message (with better wrapping)
        let max_chars_per_line = 55;
        let mut y = panel_y + 60.0;
        let words: Vec<&str> = message.split(' ').collect();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.len() + word.len() > max_chars_per_line {
                draw_text(&current_line, panel_x + 140.0, y, 18.0, colors::TEXT);
                y += 22.0;
                current_line.clear();
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        if !current_line.is_empty() {
            draw_text(&current_line, panel_x + 140.0, y, 18.0, colors::TEXT);
        }
        
        // Next Button - draw and check click
        let btn_w = 120.0;
        let btn_h = 35.0;
        let btn_x = panel_x + panel_w - btn_w - 20.0;
        let btn_y = panel_y + panel_h - btn_h - 15.0;
        
        let mouse = mouse_position();
        let hovered = mouse.0 >= btn_x && mouse.0 <= btn_x + btn_w && mouse.1 >= btn_y && mouse.1 <= btn_y + btn_h;
        
        let bg_color = if hovered {
            colors::HOVERED
        } else {
            colors::ACCENT
        };
        
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, bg_color);
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, colors::TEXT_BRIGHT);
        
        let text = "Next >";
        let text_width = measure_text(text, None, 22, 1.0).width;
        draw_text(text, btn_x + (btn_w - text_width) / 2.0, btn_y + btn_h / 2.0 + 7.0, 22.0, colors::TEXT_BRIGHT);
        // Click handling is done in gameplay.rs update function
    }
    
    /// Draw the notification overlay (similar to tutorial but for game hints and relationship changes)
    pub(super) fn draw_notification_overlay(&self) {
        if self.notifications.pending.is_empty() {
            return;
        }
        
        let notification = &self.notifications.pending[0];
        
        // Layout - similar to tutorial overlay
        let panel_w = 550.0;
        let panel_h = 150.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = screen_height() - panel_h - 20.0;
        
        // Category-based border color
        let border_color = match notification.category {
            NotificationCategory::Positive => colors::POSITIVE,
            NotificationCategory::Warning => colors::WARNING,
            NotificationCategory::Info => colors::ACCENT,
            NotificationCategory::Hint => colors::TEXT_DIM,
        };
        
        // Background
        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 3.0, border_color);
        
        // Icon
        draw_text(&notification.icon, panel_x + 20.0, panel_y + 50.0, 40.0, WHITE);
        
        // Message (with word wrapping)
        let max_chars_per_line: usize = 50;
        let mut y = panel_y + 40.0;
        let words: Vec<&str> = notification.message.split(' ').collect();
        let mut current_line = String::new();
        
        for word in words {
            if current_line.len() + word.len() > max_chars_per_line {
                draw_text(&current_line, panel_x + 80.0, y, 20.0, colors::TEXT_BRIGHT);
                y += 24.0;
                current_line.clear();
            }
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        if !current_line.is_empty() {
            draw_text(&current_line, panel_x + 80.0, y, 20.0, colors::TEXT_BRIGHT);
            y += 24.0;
        }
        
        // Description (if any)
        if let Some(desc) = &notification.description {
            draw_text(desc, panel_x + 80.0, y + 5.0, 14.0, colors::TEXT_DIM);
        }
        
        // OK Button
        let btn_w = 100.0;
        let btn_h = 32.0;
        let btn_x = panel_x + panel_w - btn_w - 15.0;
        let btn_y = panel_y + panel_h - btn_h - 12.0;
        
        let mouse = mouse_position();
        let hovered = mouse.0 >= btn_x && mouse.0 <= btn_x + btn_w && mouse.1 >= btn_y && mouse.1 <= btn_y + btn_h;
        
        let bg_color = if hovered {
            colors::HOVERED
        } else {
            border_color
        };
        
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, bg_color);
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 2.0, colors::TEXT_BRIGHT);
        
        let text = "OK";
        let text_width = measure_text(text, None, 20, 1.0).width;
        draw_text(text, btn_x + (btn_w - text_width) / 2.0, btn_y + btn_h / 2.0 + 6.0, 20.0, colors::TEXT_BRIGHT);
    }
}
