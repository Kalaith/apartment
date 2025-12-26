use crate::state::{GameState, MenuState, StateTransition};
use crate::data::config::{GameConfig, load_config};
use crate::assets::AssetManager;

pub struct Game {
    pub state: GameState,
    pub config: GameConfig,
    pub assets: AssetManager,
}

impl Game {
    pub async fn new() -> Self {
        let mut assets = AssetManager::new();
        assets.load_assets().await;
        
        let config = load_config();
        
        Self {
            state: GameState::Menu(MenuState::new()),
            config,
            assets,
        }
    }

    pub fn update(&mut self) {
        let transition = match &mut self.state {
            GameState::Menu(s) => s.update(&self.assets, &self.config),
            GameState::Gameplay(s) => s.update(&self.assets),
            GameState::Results(s) => s.update(&self.assets, &self.config),
        };

        if let Some(t) = transition {
            self.transition(t);
        }
    }

    pub fn draw(&mut self) {
        match &mut self.state {
            GameState::Menu(s) => s.draw(&self.assets),
            GameState::Gameplay(s) => s.draw(&self.assets),
            GameState::Results(s) => s.draw(&self.assets),
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

