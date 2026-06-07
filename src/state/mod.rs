//! # State Module
//!
//! Manages the global application state machine:
//! - `GameState`: The top-level enum for game modes (Menu, Gameplay, Results).
//! - Transitions between these high-level states.
//! - Specific state structs for each mode.

mod gameplay;
mod gameplay_actions; // UI action dispatch and city action handling
mod gameplay_effects; // Narrative event effect application
mod gameplay_turn; // Monthly turn advancement
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
