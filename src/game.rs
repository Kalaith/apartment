use crate::state::{GameState, GameplayState, MenuState, ResultsState, StateTransition};

pub struct Game {
    pub state: GameState,
}

impl Game {
    pub async fn new() -> Self {
        Self {
            state: GameState::Menu(MenuState::new()),
        }
    }

    pub fn update(&mut self) {
        let transition = match &mut self.state {
            GameState::Menu(s) => s.update(),
            GameState::Gameplay(s) => s.update(),
            GameState::Results(s) => s.update(),
        };

        if let Some(t) = transition {
            self.transition(t);
        }
    }

    pub fn draw(&self) {
        match &self.state {
            GameState::Menu(s) => s.draw(),
            GameState::Gameplay(s) => s.draw(),
            GameState::Results(s) => s.draw(),
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToMenu => GameState::Menu(MenuState::new()),
            StateTransition::ToGameplay(s) => GameState::Gameplay(s),
            StateTransition::ToResults(s) => GameState::Results(s),
        };
    }
}
