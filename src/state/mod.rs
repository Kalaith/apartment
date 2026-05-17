//! # State Module
//!
//! Manages the global application state machine:
//! - `GameState`: The top-level enum for game modes (Menu, Gameplay, Results).
//! - Transitions between these high-level states.
//! - Specific state structs for each mode.

mod gameplay;
mod gameplay_actions; // Action processing (end_turn, process_action, etc.)
mod gameplay_views; // Drawing functions (draw, draw_building_mode, etc.)
mod menu;
pub mod mission_system;
pub mod tutorial_system; // Tutorial logic // Mission logic

pub use gameplay::GameplayState;
pub use menu::MenuState;

pub enum GameState {
    Menu(MenuState),
    Gameplay(GameplayState),
}

pub enum StateTransition {
    ToMenu,
    ToGameplay(GameplayState),
}
