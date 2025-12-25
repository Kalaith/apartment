# Task 09: Persistence & Data
## Priority: 🟠 MEDIUM (Backend)
## Dependencies: Task 01 (Architecture)
## Estimated Effort: 3 hours

## Objective
Move hardcoded values to data files and implement save/load functionality to allow multi-session play.

## Deliverables

### 1. JSON Data Loading (`src/data/`)
- Move constants from `src/simulation/decay.rs` and `src/economy/costs.rs` to `assets/balance.json`.
- Move archetype definitions to `assets/tenants.json`.
- Implement `serde` loading logic in `src/data/mod.rs` to populate these structs on startup.

### 2. Save System (`src/save/`)
- **Serializable State**: Ensure `GameplayState` and all children derive `Serialize` / `Deserialize`.
- **Save File**: `user_data/savegame.json`.
- **Save Logic**:
  - "Save Game" button in Menu.
  - "Continue" button in Menu (greyed out if no save).
  - Auto-save on "End Month".

### 3. Error Handling
- Graceful fallback if JSON files are missing/corrupt (use baked-in defaults).
- Visual feedback if save fails.

## Implementation Notes
- Use `serde_json`.
- Macroquad provides some file system abstraction, but standard `std::fs` is usually fine for desktop builds.
- Ensure strict versioning or handle missing fields gracefully if possible.
