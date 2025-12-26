mod menu;
mod gameplay;
mod gameplay_actions;  // Action processing (end_turn, process_action, etc.)
mod gameplay_views;    // Drawing functions (draw, draw_building_mode, etc.)
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
