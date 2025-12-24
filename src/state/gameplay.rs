use macroquad::prelude::*;
use super::StateTransition;

pub struct GameplayState {
    // Will be populated by building/tenant/economy tasks
    pub money: i32,
    pub current_tick: u32,
}

impl GameplayState {
    pub fn new() -> Self {
        Self {
            money: 5000, // Starting funds
            current_tick: 0,
        }
    }

    pub fn update(&mut self) -> Option<StateTransition> {
        // Stub - will be expanded
        None
    }

    pub fn draw(&self) {
        // Stub - will be expanded
        draw_text(&format!("Money: ${}", self.money), 20.0, 40.0, 30.0, WHITE);
        draw_text(&format!("Month: {}", self.current_tick), 20.0, 80.0, 30.0, WHITE);
    }
}
