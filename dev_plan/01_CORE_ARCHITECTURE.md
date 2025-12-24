# Task 01: Core Architecture

## Priority: 🔴 CRITICAL (Must Complete First)
## Dependencies: None
## Estimated Effort: 2-3 hours

---

## Objective
Set up the foundational project structure, state machine, and main game loop following the Macroquad architecture pattern.

---

## Deliverables

### 1. Cargo.toml
```toml
[package]
name = "apartment"
version = "0.1.0"
edition = "2021"

[dependencies]
macroquad = "0.4"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"

[profile.release]
opt-level = 3
lto = true
```

### 2. src/main.rs

**Requirements:**
- Configure window (1280x720, resizable, title "Apartment")
- Initialize Game struct
- Run main loop: clear, update, draw, next_frame

**Template:**
```rust
use macroquad::prelude::*;

mod game;
mod state;
mod building;
mod tenant;
mod economy;
mod simulation;
mod ui;
mod data;
mod save;

use game::Game;

fn window_conf() -> Conf {
    Conf {
        window_title: "Apartment".to_owned(),
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
        clear_background(Color::from_rgba(30, 30, 35, 255));
        game.update();
        game.draw();
        next_frame().await;
    }
}
```

### 3. src/game.rs

**Requirements:**
- Define `Game` struct holding:
  - `state: GameState`
  - `textures: HashMap<String, Texture2D>` (optional for MVP)
- Implement `Game::new()` async constructor
- Implement `Game::update()` - delegates to current state, handles transitions
- Implement `Game::draw()` - delegates to current state
- Implement `Game::transition(StateTransition)` - applies state changes

**Key Logic:**
```rust
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
```

### 4. src/state/mod.rs

**Requirements:**
- Define `GameState` enum with variants: Menu, Gameplay, Results
- Define `StateTransition` enum for explicit state changes
- Re-export state structs

```rust
mod menu;
mod gameplay;
mod results;

pub use menu::MenuState;
pub use gameplay::GameplayState;
pub use results::ResultsState;

pub enum GameState {
    Menu(MenuState),
    Gameplay(GameplayState),
    Results(ResultsState),
}

pub enum StateTransition {
    ToMenu,
    ToGameplay(GameplayState),
    ToResults(ResultsState),
}
```

### 5. src/state/menu.rs

**Requirements:**
- Simple menu with "New Game" button
- Returns `StateTransition::ToGameplay` when clicked

```rust
use macroquad::prelude::*;
use super::{StateTransition, GameplayState};

pub struct MenuState {
    // No fields needed for MVP
}

impl MenuState {
    pub fn new() -> Self {
        Self {}
    }
    
    pub fn update(&mut self) -> Option<StateTransition> {
        // Check for New Game button click
        // Return Some(StateTransition::ToGameplay(GameplayState::new())) if clicked
        None
    }
    
    pub fn draw(&self) {
        // Draw title
        // Draw "New Game" button
    }
}
```

### 6. src/state/gameplay.rs

**Requirements:**
- Main gameplay state struct holding:
  - `building: Building`
  - `tenants: Vec<Tenant>`
  - `money: i32`
  - `current_tick: u32`
  - `applications: Vec<TenantApplication>`
- Stub implementations for update/draw
- Will be expanded by other tasks

```rust
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
            money: 5000,  // Starting funds
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
```

### 7. src/state/results.rs

**Requirements:**
- End-of-game summary screen
- Display: total income, tenant turnover, final building state
- "Return to Menu" button

```rust
use macroquad::prelude::*;
use super::StateTransition;

pub struct ResultsState {
    pub total_income: i32,
    pub tenants_housed: u32,
    pub tenants_left: u32,
    pub final_reputation: f32,
    pub won: bool,
}

impl ResultsState {
    pub fn new(total_income: i32, tenants_housed: u32, tenants_left: u32, won: bool) -> Self {
        Self {
            total_income,
            tenants_housed,
            tenants_left,
            final_reputation: 0.0,
            won,
        }
    }
    
    pub fn update(&mut self) -> Option<StateTransition> {
        // Check for Return to Menu click
        None
    }
    
    pub fn draw(&self) {
        let title = if self.won { "SUCCESS!" } else { "BANKRUPT" };
        // Draw summary stats
    }
}
```

### 8. Module Stubs

Create empty mod.rs files for modules that will be implemented by other tasks:

**src/building/mod.rs**
```rust
// Implemented by Task 02
```

**src/tenant/mod.rs**
```rust
// Implemented by Task 03
```

**src/economy/mod.rs**
```rust
// Implemented by Task 04
```

**src/simulation/mod.rs**
```rust
// Implemented by Task 05
```

**src/ui/mod.rs**
```rust
// Implemented by Task 06
```

**src/data/mod.rs**
```rust
// Implemented by Task 07
```

**src/save/mod.rs**
```rust
// Implemented later (not MVP critical)
```

---

## Acceptance Criteria

- [ ] `cargo build` succeeds with no errors
- [ ] `cargo run` opens a window titled "Apartment"
- [ ] Main menu displays with "New Game" button
- [ ] Clicking "New Game" transitions to gameplay state
- [ ] Gameplay state shows money and month on screen
- [ ] State transitions are explicit (no hidden callbacks)
- [ ] All module stubs exist (even if empty)

---

## Testing Checklist

```bash
# Build test
cargo build

# Run and verify:
# 1. Window opens at 1280x720
# 2. Menu state renders
# 3. Can transition to gameplay
# 4. Gameplay state renders basic info
```

---

## Notes for Agent

- Use `Color::from_rgba(30, 30, 35, 255)` as background color (dark slate)
- Keep state structs minimal - other tasks will expand them
- Don't implement any game logic yet - just the skeleton
- Follow the exact file structure in the master plan
