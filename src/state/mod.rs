mod menu;
mod gameplay;
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
