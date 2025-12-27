
use std::fs;
use std::path::Path;
use serde_json;
use serde::{Deserialize, Serialize};
use crate::state::GameplayState;
use crate::data::config::load_config;

const SAVE_FILE_PATH: &str = "savegame.json";
const PROGRESS_FILE_PATH: &str = "player_progress.json";

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
    let json = serde_json::to_string_pretty(state)?;
    fs::write(SAVE_FILE_PATH, json)?;
    Ok(())
}

/// Load the game state from disk
pub fn load_game() -> std::io::Result<GameplayState> {
    let json = fs::read_to_string(SAVE_FILE_PATH)?;
    let mut state: GameplayState = serde_json::from_str(&json)?;
    
    // Restore non-serialized fields
    state.config = load_config();
    state.sync_building();
    
    Ok(state)
}

/// Check if a save file exists
pub fn has_save_game() -> bool {
    Path::new(SAVE_FILE_PATH).exists()
}

/// Load player progress (persistent unlock state)
pub fn load_player_progress() -> PlayerProgress {
    match fs::read_to_string(PROGRESS_FILE_PATH) {
        Ok(json) => {
            serde_json::from_str(&json).unwrap_or_else(|_| PlayerProgress::new())
        }
        Err(_) => PlayerProgress::new()
    }
}

/// Save player progress (persistent unlock state)
pub fn save_player_progress(progress: &PlayerProgress) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(progress)?;
    fs::write(PROGRESS_FILE_PATH, json)?;
    Ok(())
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
