use crate::assets::AssetManager;
use crate::data::config::{load_config, GameConfig};
use crate::state::{GameState, MenuState, StateTransition};

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
        };

        if let Some(t) = transition {
            self.transition(t);
        }
    }

    pub fn draw(&mut self) {
        match &mut self.state {
            GameState::Menu(s) => s.draw(&self.assets),
            GameState::Gameplay(s) => s.draw(&self.assets),
        }
    }

    fn transition(&mut self, transition: StateTransition) {
        self.state = match transition {
            StateTransition::ToMenu => GameState::Menu(MenuState::new()),
            StateTransition::ToGameplay(s) => GameState::Gameplay(s),
        };
    }

    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        match scene {
            "menu" => self.state = GameState::Menu(MenuState::new()),
            _ => {
                // Default: jump straight into gameplay using the first
                // configured building template, which is always available on
                // a fresh save.
                let template = crate::data::templates::load_templates()
                    .and_then(|templates| templates.templates.into_iter().next());
                match template {
                    Some(template) => {
                        let state = crate::state::GameplayState::new_with_template(
                            self.config.clone(),
                            template,
                        );
                        self.state = GameState::Gameplay(state);
                    }
                    None => self.state = GameState::Menu(MenuState::new()),
                }
            }
        }
    }
}
