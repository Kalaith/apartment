
use std::fs;
use std::path::Path;
use serde_json;
use crate::state::GameplayState;
use crate::data::config::load_config;

const SAVE_FILE_PATH: &str = "savegame.json";

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
