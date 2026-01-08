

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::state::GameplayState;
use crate::data::config::load_config;
use macroquad_toolkit::persistence::{save_json, load_json, get_app_data_path, file_exists};

const SAVE_FILE_NAME: &str = "savegame.json";
const PROGRESS_FILE_NAME: &str = "player_progress.json";

fn get_save_path() -> PathBuf {
    get_app_data_path("apartment_manager", SAVE_FILE_NAME)
        .unwrap_or_else(|| PathBuf::from(SAVE_FILE_NAME))
}

fn get_progress_path() -> PathBuf {
    get_app_data_path("apartment_manager", PROGRESS_FILE_NAME)
        .unwrap_or_else(|| PathBuf::from(PROGRESS_FILE_NAME))
}

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
    save_json(get_save_path(), state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Load the game state from disk
pub fn load_game() -> std::io::Result<GameplayState> {
    let mut state: GameplayState = load_json(get_save_path())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    
    // Restore non-serialized fields
    state.config = load_config();
    state.sync_building();
    
    Ok(state)
}

/// Check if a save file exists
pub fn has_save_game() -> bool {
    file_exists(get_save_path())
}

/// Load player progress (persistent unlock state)
pub fn load_player_progress() -> PlayerProgress {
    load_json(get_progress_path()).unwrap_or_else(|_| PlayerProgress::new())
}

/// Save player progress (persistent unlock state)
pub fn save_player_progress(progress: &PlayerProgress) -> std::io::Result<()> {
    save_json(get_progress_path(), progress)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameplayState;

    #[test]
    fn test_save_load_serialization() {
        // 1. Create a dummy state
        let mut state = GameplayState::new();
        state.funds.balance = 9999;
        state.current_tick = 5;
        state.next_tenant_id = 10;
        
        // 2. Serialize
        let json = serde_json::to_string(&state).expect("Failed to serialize");
        
        // 3. Deserialize
        let loaded: GameplayState = serde_json::from_str(&json).expect("Failed to deserialize");
        
        // 4. Verify fields
        assert_eq!(loaded.funds.balance, 9999);
        assert_eq!(loaded.current_tick, 5);
        assert_eq!(loaded.next_tenant_id, 10);
        // Default values for skipped fields
        assert_eq!(loaded.pending_actions.len(), 0);
    }
}
