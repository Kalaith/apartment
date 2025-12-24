# Rust Game Development Reference
## Tech Stack & Project Structure Based on Frontier Kingdom

Use this document when prompting an AI to create a new Rust game with the same architecture and patterns.

---

## Technology Stack

### Language: Rust
- **Edition**: 2021
- **Toolchain**: Stable Rust (1.70+)
- **Package Manager**: Cargo

### Core Dependencies

```toml
[dependencies]
macroquad = "0.4"              # Game framework (rendering, input, audio)
serde = { version = "1.0", features = ["derive"] }  # Serialization
serde_json = "1.0"             # JSON data loading
rand = "0.8"                   # Random number generation

[profile.release]
opt-level = 3
lto = true
```

### Framework: Macroquad
**Use for:**
- Rendering (shapes, textures, text)
- Input handling (keyboard, mouse)
- Audio playback
- Main loop timing

**DO NOT use for:**
- Scene management
- Game state authority
- UI framework (use immediate-mode UI instead)

> Macroquad should remain a *thin* rendering/input layer.

---

## Project Structure

```
game_name/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point, window config
│   ├── game.rs              # Game loop & state machine
│   ├── state/               # Game states (one per screen/mode)
│   │   ├── mod.rs           # State exports & StateTransition enum
│   │   ├── menu.rs          # Main menu state
│   │   ├── gameplay.rs      # Core gameplay state
│   │   └── results.rs       # End/results state
│   ├── combat/              # Combat system (if applicable)
│   │   ├── mod.rs
│   │   ├── unit.rs          # Character/unit structs
│   │   ├── card.rs          # Cards/abilities
│   │   ├── effects.rs       # Effect enums
│   │   └── resolver.rs      # Effect resolution logic
│   ├── kingdom/             # Management/progression (if applicable)
│   │   ├── mod.rs
│   │   ├── buildings.rs     # Building definitions
│   │   ├── roster.rs        # Character roster
│   │   └── progression.rs   # Unlock logic
│   ├── data/                # Data loading modules
│   │   ├── mod.rs
│   │   ├── cards.rs         # Card data loader
│   │   └── enemies.rs       # Enemy data loader
│   ├── ui/                  # UI helpers (optional)
│   │   └── mod.rs
│   └── save/                # Save/load system
│       └── mod.rs
├── assets/
│   ├── cards.json           # Card definitions (data-driven)
│   ├── enemies.json         # Enemy definitions
│   ├── missions.json        # Level/mission definitions
│   └── images/              # Sprite assets
└── README.md
```

---

## Architectural Principles

### 1. Explicit State Machine
```rust
pub enum GameState {
    Menu(MenuState),
    Gameplay(GameplayState),
    Combat(CombatState),
    Results(ResultState),
}

// Explicit transitions via enum
pub enum StateTransition {
    ToMenu,
    ToGameplay(GameplayState),
    ToCombat(CombatState),
    ToResults(ResultState),
}
```

**Rules:**
- Only ONE state active at a time
- Transitions are explicit (no magic callbacks)
- No shared mutable global state

### 2. Separation of Concerns
- **Rendering** does not contain logic
- **UI** does not mutate game state directly
- **Cards/Abilities** emit effects, **systems** resolve them

### 3. Data-Driven Content
- Game rules live in **code**
- Content lives in **JSON/RON files**
- Balance changes don't require recompilation

---

## Core Patterns

### Entry Point (main.rs)
```rust
use macroquad::prelude::*;

mod game;
mod state;
mod combat;
mod data;
mod save;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Game Name".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;
    
    loop {
        clear_background(Color::from_rgba(20, 20, 25, 255));
        game.update();
        game.draw();
        next_frame().await;
    }
}
```

### Game Struct (game.rs)
```rust
pub struct Game {
    pub state: GameState,
    pub textures: HashMap<String, Texture2D>,
    // Shared resources go here
}

impl Game {
    pub async fn new() -> Self {
        // Load textures, initialize state
    }
    
    pub fn update(&mut self) {
        // Match on current state, call state.update()
        // Handle StateTransition return values
    }
    
    pub fn draw(&self) {
        // Match on current state, call state.draw()
    }
    
    pub fn transition(&mut self, transition: StateTransition) {
        // Apply explicit state change
    }
}
```

### State Module Pattern (state/mod.rs)
```rust
mod menu;
mod gameplay;
mod combat;

pub use menu::MenuState;
pub use gameplay::GameplayState;
pub use combat::CombatState;

pub enum StateTransition {
    ToMenu,
    ToGameplay(GameplayState),
    ToCombat(CombatState),
}
```

### Individual State Pattern
```rust
pub struct GameplayState {
    // State-specific data
}

impl GameplayState {
    pub fn new() -> Self { ... }
    
    pub fn update(&mut self) -> Option<StateTransition> {
        // Return None to stay, Some(transition) to change
    }
    
    pub fn draw(&self, textures: &HashMap<String, Texture2D>) {
        // Render this state
    }
}
```

---

## Combat System Pattern

### Card Effects (Emit, Don't Mutate)
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CardEffect {
    Damage(i32),
    Block(i32),
    Heal(i32),
    ApplyStatus { status: StatusType, stacks: i32 },
    DrawCards(i32),
    GainEnergy(i32),
}

pub struct Card {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub description: String,
    pub effects: Vec<CardEffect>,
    pub image_path: Option<String>,
}
```

### Resolution Flow
```
Player selects card
 → Card emits effects
 → Resolver validates legality
 → Effects applied in order
 → State updated
```

This enables: replays, logging, AI simulation, balance changes.

---

## Status Effects Pattern
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatusType {
    Vulnerable,    // Take more damage
    Weak,          // Deal less damage
    Strengthened,  // Deal more damage
    Guarded,       // Take less damage
    Stunned,       // Skip next turn
}

pub struct StatusEffect {
    pub status_type: StatusType,
    pub stacks: i32,
    pub duration: Option<i32>,
}
```

---

## Data Loading Pattern

### JSON Definition (assets/cards.json)
```json
[
  {
    "id": "strike",
    "name": "Strike",
    "cost": 1,
    "description": "Deal 6 damage",
    "effects": [{ "Damage": 6 }],
    "image_path": "assets/images/cards/strike.png"
  }
]
```

### Loader (data/cards.rs)
```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardData {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub description: String,
    pub effects: Vec<CardEffect>,
    pub image_path: Option<String>,
}

impl CardData {
    pub fn load_all() -> Result<Vec<CardData>, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string("assets/cards.json")?;
        let cards: Vec<CardData> = serde_json::from_str(&json)?;
        Ok(cards)
    }
}
```

---

## Save/Load Pattern

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub roster: Vec<Adventurer>,
    pub buildings: Vec<Building>,
    pub progress: ProgressData,
}

impl SaveData {
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write("save.json", json)?;
        Ok(())
    }
    
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string("save.json")?;
        let data: SaveData = serde_json::from_str(&json)?;
        Ok(data)
    }
}
```

---

## UI Philosophy: Immediate Mode

```rust
// Every frame, redraw everything
fn draw_button(x: f32, y: f32, text: &str) -> bool {
    let rect = Rect::new(x, y, 200.0, 40.0);
    let hovered = rect.contains(mouse_position().into());
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);
    
    let color = if hovered { LIGHTGRAY } else { GRAY };
    draw_rectangle(x, y, 200.0, 40.0, color);
    draw_text(text, x + 10.0, y + 28.0, 24.0, WHITE);
    
    clicked
}
```

**Rules:**
- UI reads state
- UI emits intents (returns booleans/enums)
- Game logic applies changes
- No logic hidden in UI callbacks

---

## Texture Loading Pattern

```rust
use std::collections::HashMap;
use macroquad::prelude::*;

async fn load_textures() -> HashMap<String, Texture2D> {
    let mut textures = HashMap::new();
    
    // Load from JSON paths
    if let Ok(cards) = CardData::load_all() {
        for card in cards {
            if let Some(path) = &card.image_path {
                if let Ok(tex) = load_texture(path).await {
                    tex.set_filter(FilterMode::Nearest);
                    textures.insert(card.id.clone(), tex);
                }
            }
        }
    }
    
    textures
}
```

---

## Key Commands

```bash
# Run development
cargo run

# Build release
cargo build --release

# Run release
./target/release/game_name
```

---

## Non-Goals (Keep It Simple)

- ❌ No ECS overengineering
- ❌ No custom editor tooling (initially)
- ❌ No real-time combat (keep turn-based)
- ❌ No procedural generation until core systems stable

> **Simplicity is a feature.**

---

## Checklist for New Game

1. [ ] Initialize with `cargo new game_name`
2. [ ] Add dependencies to `Cargo.toml`
3. [ ] Create `src/` folder structure
4. [ ] Implement `GameState` enum
5. [ ] Implement `StateTransition` enum
6. [ ] Create `Game` struct with update/draw loop
7. [ ] Create initial state (Menu or Gameplay)
8. [ ] Set up `assets/` folder for JSON data
9. [ ] Implement data loaders (`data/` module)
10. [ ] Implement save/load system
11. [ ] Add textures and assets

---

*Based on Frontier Kingdom architecture - a card-based expedition RPG built with Rust and Macroquad.*
