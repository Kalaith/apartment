//! # State Module
//! 
//! Manages the global application state machine:
//! - `GameState`: The top-level enum for game modes (Menu, Gameplay, Results).
//! - Transitions between these high-level states.
//! - Specific state structs for each mode.

mod menu;
mod gameplay;
mod gameplay_actions;  // Action processing (end_turn, process_action, etc.)
mod gameplay_views;    // Drawing functions (draw, draw_building_mode, etc.)
pub mod tutorial_system; // Tutorial logic
pub mod mission_system;  // Mission logic
mod results;

pub use menu::MenuState;
pub use gameplay::GameplayState;
pub use results::ResultsState;

pub enum GameState {
    Menu(MenuState),
    Gameplay(GameplayState),
    Results(ResultsState),
}

pub enum StateTransition {
    ToMenu,
    ToGameplay(GameplayState),
    ToResults(ResultsState),
}
