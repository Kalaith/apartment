//! Gameplay input dispatch and overlay ownership.

use crate::assets::AssetManager;
use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::{colors, Selection, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

use super::gameplay::{GameplayState, ViewMode};
use super::StateTransition;

impl GameplayState {
    fn has_blocking_narrative_event(&self) -> bool {
        self.narrative_events
            .events
            .iter()
            .any(|event| !event.read && event.requires_response)
    }

    fn has_blocking_tutorial_message(&self) -> bool {
        self.tutorial.active && !self.tutorial.pending_messages.is_empty()
    }

    /// Main update function - handles game logic and input.
    pub fn update(&mut self, assets: &AssetManager) -> Option<StateTransition> {
        if !assets.loaded {
            return None;
        }

        // Resolve pause before dispatching clicks or keyboard shortcuts from
        // this frame. Opening or closing the menu consumes the frame's input.
        let escape_pressed = is_key_pressed(KeyCode::Escape);
        let was_paused = self.show_pause_menu;
        if escape_pressed {
            self.show_pause_menu = !self.show_pause_menu;
        }

        // A modal is drawn over the base view, so base-view clicks from that
        // same frame must not leak through. Its own response remains allowed.
        let paused_on_entry = was_paused || self.show_pause_menu;
        let blocked_on_entry =
            self.has_blocking_narrative_event() || self.has_blocking_tutorial_message();
        let actions: Vec<UiAction> = self.pending_actions.drain(..).collect();
        for action in actions {
            if action_allowed_while_blocked(paused_on_entry, blocked_on_entry, &action) {
                self.process_action(action);
            }
        }

        let dt = get_frame_time();
        self.floating_texts.update(dt);
        self.dialogue_system.tick(self.current_tick);

        if matches!(self.selection, Selection::None) {
            self.panel_tween.set_target(0.0);
        } else {
            self.panel_tween.set_target(1.0);
        }
        self.panel_tween.update(dt);

        if self.game_outcome.is_some() && self.view_mode != ViewMode::CareerSummary {
            self.view_mode = ViewMode::CareerSummary;
            let new_unlocks = self.achievements.check_new_unlocks(
                &self.city,
                &self.building,
                &self.tenants,
                &self.funds,
                self.current_tick,
                &self.config,
            );
            for id in new_unlocks {
                self.achievements.unlock(&id);
            }
        }

        self.update_tutorial();

        if self.show_pause_menu {
            if self.pending_quit_to_menu {
                self.pending_quit_to_menu = false;
                return Some(StateTransition::ToMenu);
            }
            return None;
        }

        // Space advances only when no modal owns input. Processing Escape first
        // also prevents a simultaneous pause keypress from advancing a month.
        if !escape_pressed
            && !self.has_blocking_narrative_event()
            && !self.has_blocking_tutorial_message()
            && is_key_pressed(KeyCode::Space)
            && matches!(self.view_mode, ViewMode::Building)
        {
            self.end_turn();
        }

        if self.pending_quit_to_menu {
            self.pending_quit_to_menu = false;
            return Some(StateTransition::ToMenu);
        }

        draw_status_header(self);
        None
    }
}

fn draw_status_header(state: &GameplayState) {
    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        HEADER_HEIGHT(),
        colors::SURFACE_HEADER(),
    );
    draw_ui_text_ex(
        &format!("{} - City Overview", state.city.name),
        20.0,
        35.0,
        TextParams {
            font_size: 28,
            color: colors::TEXT(),
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        &format!("${}", state.funds.balance),
        screen_width() - 200.0,
        35.0,
        TextParams {
            font_size: 24,
            color: colors::POSITIVE(),
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        &format!(
            "{} Buildings | Month {}",
            state.city.buildings.len(),
            state.current_tick
        ),
        screen_width() - 400.0,
        35.0,
        TextParams {
            font_size: 16,
            color: colors::TEXT_DIM(),
            ..Default::default()
        },
    );
    draw_ui_text_ex(
        &format!(
            "Gentrification Score: {} | Affordable Units: {}",
            state.gentrification.gentrification_score, state.gentrification.affordable_units
        ),
        20.0,
        55.0,
        TextParams {
            font_size: 12,
            color: colors::TEXT_DIM(),
            ..Default::default()
        },
    );

    let nav_hint = match state.view_mode {
        ViewMode::Building => "Building workspace",
        ViewMode::Tenants => "Resident roster and leasing",
        ViewMode::Finances => "Ledger and operating policies",
        ViewMode::CityMap => "City portfolio",
        ViewMode::Market => "Property market",
        ViewMode::Mail => "Letters and conversations",
        ViewMode::Tasks => "Missions and resident requests",
        ViewMode::CareerSummary => "",
    };
    draw_ui_text_ex(
        nav_hint,
        20.0,
        55.0,
        TextParams {
            font_size: 14,
            color: colors::TEXT_DIM(),
            ..Default::default()
        },
    );
}

fn action_allowed_while_blocked(paused: bool, blocked: bool, action: &UiAction) -> bool {
    !paused && (!blocked || matches!(action, UiAction::ResolveEventChoice { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlays_reject_gameplay_actions_but_accept_event_responses() {
        let gameplay_action = UiAction::EndTurn;
        let response = UiAction::ResolveEventChoice {
            event_id: 1,
            choice_index: 0,
        };

        assert!(!action_allowed_while_blocked(false, true, &gameplay_action));
        assert!(action_allowed_while_blocked(false, true, &response));
        assert!(!action_allowed_while_blocked(true, false, &gameplay_action));
    }
}
