use crate::data::config::load_config;
use crate::state::GameplayState;
use macroquad_toolkit::persistence::{json_key_exists, load_json_key, save_json_key};
use serde::{Deserialize, Serialize};

const GAME_NAME: &str = "apartment_manager";
const SAVE_FILE_NAME: &str = "savegame.json";
const PROGRESS_FILE_NAME: &str = "player_progress.json";

/// Player progress - persists across game sessions
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct PlayerProgress {
    pub unlocked_buildings: Vec<String>,
    pub completed_buildings: Vec<String>,
}

impl PlayerProgress {
    pub fn new() -> Self {
        Self {
            unlocked_buildings: vec!["mvp_default".to_string()], // First building unlocked by default
            completed_buildings: Vec::new(),
        }
    }

    pub fn is_unlocked(&self, building_id: &str) -> bool {
        self.unlocked_buildings.contains(&building_id.to_string())
    }

    pub fn unlock_building(&mut self, building_id: &str) {
        if !self.unlocked_buildings.contains(&building_id.to_string()) {
            self.unlocked_buildings.push(building_id.to_string());
        }
    }

    pub fn mark_completed(&mut self, building_id: &str) {
        if !self.completed_buildings.contains(&building_id.to_string()) {
            self.completed_buildings.push(building_id.to_string());
        }
    }
}

/// Save the current game state to disk
pub fn save_game(state: &GameplayState) -> std::io::Result<()> {
    save_json_key(GAME_NAME, SAVE_FILE_NAME, state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Load the game state from disk
pub fn load_game() -> std::io::Result<GameplayState> {
    let mut state: GameplayState = load_json_key(GAME_NAME, SAVE_FILE_NAME)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Restore non-serialized fields
    state.config = load_config();
    state.sync_building();

    Ok(state)
}

/// Check if a save file exists
pub fn has_save_game() -> bool {
    json_key_exists(GAME_NAME, SAVE_FILE_NAME)
}

/// Load player progress (persistent unlock state)
pub fn load_player_progress() -> PlayerProgress {
    load_json_key(GAME_NAME, PROGRESS_FILE_NAME).unwrap_or_else(|_| PlayerProgress::new())
}

/// Save player progress (persistent unlock state)
pub fn save_player_progress(progress: &PlayerProgress) -> std::io::Result<()> {
    save_json_key(GAME_NAME, PROGRESS_FILE_NAME, progress)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

#[cfg(test)]
mod tests {

    use crate::state::GameplayState;

    #[test]
    fn test_save_load_serialization() {
        // 1. Create a dummy state
        let mut state = GameplayState::new();
        state.funds.balance = 9999;
        state.current_tick = 5;
        state.next_tenant_id = 10;

        // 2. Serialize
        let serialized = serde_json::to_string(&state);
        assert!(
            serialized.is_ok(),
            "Failed to serialize: {:?}",
            serialized.as_ref().err()
        );
        let Ok(json) = serialized else {
            return;
        };

        // 3. Deserialize
        let deserialized: Result<GameplayState, _> = serde_json::from_str(&json);
        assert!(
            deserialized.is_ok(),
            "Failed to deserialize: {:?}",
            deserialized.as_ref().err()
        );
        let Ok(loaded) = deserialized else {
            return;
        };

        // 4. Verify fields
        assert_eq!(loaded.funds.balance, 9999);
        assert_eq!(loaded.current_tick, 5);
        assert_eq!(loaded.next_tenant_id, 10);
        // Default values for skipped fields
        assert_eq!(loaded.pending_actions.len(), 0);
    }
}
