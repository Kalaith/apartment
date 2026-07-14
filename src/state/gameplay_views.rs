//! Game view rendering - split from gameplay.rs for maintainability

use crate::assets::AssetManager;
use crate::narrative::NotificationCategory;
use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::{
    colors, draw_apartment_panel, draw_application_panel, draw_building_view, draw_hallway_panel,
    draw_header, draw_notifications, draw_ownership_panel, FloatingText, Selection,
};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};
use macroquad_toolkit::ui::{draw_ui_text, draw_ui_text_ex, measure_ui_text};

impl GameplayState {
    /// Main draw function - dispatches to appropriate view
    pub fn draw(&mut self, assets: &AssetManager) {
        match self.view_mode {
            ViewMode::Building => {
                self.draw_building_mode(assets);
            }
            ViewMode::CityMap => {
                if let Some(action) =
                    crate::ui::city_view::draw_city_map(&self.city, assets, &self.narrative_events)
                {
                    self.handle_city_action(action);
                }

                if let Some(action) = crate::ui::city_view::draw_portfolio_panel(
                    &self.city,
                    self.city.active_building_index,
                    assets,
                ) {
                    self.handle_city_action(action);
                }
            }
            ViewMode::Market => {
                let listings: Vec<&crate::city::PropertyListing> =
                    self.city.market.listings.iter().collect();
                if let Some(action) = crate::ui::city_view::draw_market_panel(
                    &listings,
                    &self.city.neighborhoods,
                    self.funds.balance,
                    assets,
                ) {
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
        let blocking_event = self
            .narrative_events
            .events
            .iter()
            .find(|e| !e.read && e.requires_response);

        if let Some(event) = blocking_event {
            if let Some(action) = crate::ui::event_modal::draw_event_modal(event) {
                self.pending_actions.push(action);
            }
        }

        // Footer event log.
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
            assets,
        ) {
            self.pending_actions.push(action);
        }

        // Draw Building View
        if let Some(action) =
            draw_building_view(&self.building, &self.tenants, &self.selection, assets)
        {
            self.pending_actions.push(action);
        }

        match self.selection {
            Selection::Apartment(id) => {
                if let Some(apt) = self.building.get_apartment(id) {
                    let (action, new_scroll) = draw_apartment_panel(
                        apt,
                        &self.building,
                        &self.tenants,
                        self.funds.balance,
                        0.0,
                        self.panel_scroll_offset,
                        assets,
                        &self.config,
                        &self.tenant_network,
                        &self.tenant_stories,
                    );
                    self.panel_scroll_offset = new_scroll;
                    if let Some(action) = action {
                        self.pending_actions.push(action);
                    }
                }
            }
            Selection::Hallway => {
                let (action, new_scroll) = draw_hallway_panel(
                    &self.building,
                    self.funds.balance,
                    0.0,
                    self.panel_scroll_offset,
                    assets,
                    &self.config,
                );
                self.panel_scroll_offset = new_scroll;
                if let Some(action) = action {
                    self.pending_actions.push(action);
                }
            }
            Selection::Applications(filter) => {
                if let Some(action) =
                    draw_application_panel(&self.applications, &self.building, filter, 0.0, assets)
                {
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
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            HEADER_HEIGHT,
            colors::PANEL_HEADER,
        );

        // Show a loading indicator if assets aren't ready
        if !has_assets {
            draw_ui_text_ex(
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

        draw_ui_text_ex(
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
            draw_ui_text_ex(
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
            draw_ui_text_ex(
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
            draw_ui_text_ex(
                &mail.subject,
                60.0,
                y + 25.0,
                TextParams {
                    font_size: 18,
                    color: if mail.read {
                        colors::TEXT_DIM
                    } else {
                        colors::TEXT
                    },
                    ..Default::default()
                },
            );

            // Sender
            draw_ui_text_ex(
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
            draw_ui_text_ex(
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
        draw_ui_text_ex(
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
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            Color::new(0.0, 0.0, 0.0, 0.7),
        );

        // Menu panel
        let panel_w = 300.0;
        let panel_h = 330.0;
        let panel_x = (screen_width() - panel_w) / 2.0;
        let panel_y = (screen_height() - panel_h) / 2.0;

        draw_rectangle(panel_x, panel_y, panel_w, panel_h, colors::PANEL);
        draw_rectangle_lines(panel_x, panel_y, panel_w, panel_h, 2.0, colors::ACCENT);

        // Title
        let title = "PAUSED";
        let title_width = measure_ui_text(title, None, 32, 1.0).width;
        draw_ui_text(
            title,
            panel_x + (panel_w - title_width) / 2.0,
            panel_y + 40.0,
            32.0,
            colors::TEXT_BRIGHT,
        );

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
        let fs_label = if self.is_fullscreen {
            "Windowed Mode"
        } else {
            "Fullscreen"
        };
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

        // Quit to Menu button
        if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Quit to Menu") {
            self.pending_quit_to_menu = true;
        }

        // Quit Game button (exits completely) — native only; a browser tab has
        // nothing to exit and std::process::exit is unsupported on wasm.
        #[cfg(not(target_arch = "wasm32"))]
        {
            btn_y += 50.0;
            if self.menu_button(btn_x, btn_y, btn_w, btn_h, "Quit Game") {
                std::process::exit(0);
            }
        }

        // ESC hint
        draw_ui_text(
            "Press ESC to resume",
            panel_x + (panel_w - 140.0) / 2.0,
            panel_y + panel_h - 20.0,
            14.0,
            colors::TEXT_DIM,
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
            colors::SURFACE_ALT
        };

        draw_rectangle(x, y, w, h, bg_color);
        draw_rectangle_lines(
            x,
            y,
            w,
            h,
            1.0,
            if hovered {
                colors::PRIMARY
            } else {
                colors::BORDER_STRONG
            },
        );

        let text_width = measure_ui_text(text, None, 20, 1.0).width;
        draw_ui_text(
            text,
            x + (w - text_width) / 2.0,
            y + h / 2.0 + 6.0,
            20.0,
            colors::TEXT,
        );

        clicked
    }

    /// Draw the tutorial overlay as a bottom toast. Dismisses on "Next".
    pub(super) fn draw_tutorial_overlay(&mut self, _assets: &AssetManager) {
        if self.tutorial.pending_messages.is_empty() {
            return;
        }
        let message = self.tutorial.pending_messages[0].clone();
        if crate::ui::widgets::draw_toast(
            "",
            "Uncle Artie",
            &message,
            crate::ui::widgets::ToastKind::Info,
            "Next",
        ) {
            self.tutorial.pending_messages.remove(0);
        }
    }

    /// Draw the hint/relationship notification as a bottom toast. Dismisses on
    /// "OK".
    pub(super) fn draw_notification_overlay(&mut self) {
        let Some(notification) = self.notifications.pending.first() else {
            return;
        };
        let kind = match notification.category {
            NotificationCategory::Positive => crate::ui::widgets::ToastKind::Positive,
            NotificationCategory::Warning => crate::ui::widgets::ToastKind::Warning,
            NotificationCategory::Info => crate::ui::widgets::ToastKind::Info,
            NotificationCategory::Hint => crate::ui::widgets::ToastKind::Hint,
        };
        let icon = notification.icon.clone();
        let mut body = notification.message.clone();
        if let Some(desc) = &notification.description {
            body.push('\n');
            body.push_str(desc);
        }
        if crate::ui::widgets::draw_toast(&icon, "", &body, kind, "OK") {
            self.notifications.pop();
        }
    }
}
