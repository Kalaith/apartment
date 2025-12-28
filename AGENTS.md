# Agent Instructions (AGENTS.md)

**Project**: Apartment Manager  
**Engine**: Macroquad (Rust)  
**Platform**: Windows + WebGL (itch.io)

This document provides instructions for AI agents working on this project.

---

## 1. Critical Rules

### 1.1 No Cargo Commands
**Never run cargo commands** (cargo run, cargo build, cargo check, cargo test). The user will run these manually.

### 1.2 Follow CODE_STANDARDS.md
All code must align with the project's Rust coding standards. Key highlights:
- Readability over cleverness
- Module responsibilities are strict (see Section 2.1 of CODE_STANDARDS.md)
- Target 200-400 lines per file, max 800
- Functions target 20-50 lines, max 100
- UI code is "dumb" - reads state, emits actions, no business logic

---

## 2. Project Structure

```
src/
├── building/     # Apartments, upgrades, ownership
├── city/         # Neighborhoods, market
├── consequences/ # Relationships, compliance, gentrification
├── data/         # Config loaders, templates
├── economy/      # Funds, rent, transactions
├── narrative/    # Events, mail, stories, missions
├── save/         # Save/load, player progress
├── simulation/   # Game tick, win conditions
├── state/        # Game state, views
├── tenant/       # Tenants, applications, archetypes
├── ui/           # All rendering (read-only)
└── util/         # Cross-platform utilities
assets/
├── *.json        # Configuration files (data-driven)
└── textures/     # PNG images
```

---

## 3. WebGL/WASM Builds

### 3.1 File Loading for WASM
**WASM cannot use `std::fs`**. Use `include_str!` for JSON configs:

```rust
#[cfg(target_arch = "wasm32")]
let json = include_str!("../../assets/config.json");

#[cfg(not(target_arch = "wasm32"))]
let json = std::fs::read_to_string("assets/config.json")
    .unwrap_or_else(|_| include_str!("../../assets/config.json").to_string());
```

### 3.2 Random Numbers
Use `macroquad::rand` (not the `rand` crate) for WASM compatibility:
```rust
use macroquad::rand::gen_range;
let value = gen_range(0, 100);
```

### 3.3 Asset Paths
Use **relative paths** (no leading `/`):
```rust
// Correct
let path = format!("assets/textures/{}.png", id);

// Wrong - absolute path breaks itch.io
let path = format!("/assets/textures/{}.png", id);
```

---

## 4. Publishing

### 4.1 Build Script
Use `publish.ps1` to create distributable packages:
```powershell
.\publish.ps1              # Windows + WebGL
.\publish.ps1 -WindowsOnly # Windows only
.\publish.ps1 -WebGLOnly   # WebGL only
```

### 4.2 Itch.io Settings
For WebGL uploads on itch.io:
- Enable "This file will be played in the browser"
- Set viewport dimensions: **1280 x 720**
- SharedArrayBuffer: OFF

---

## 5. Graphics & Assets

### 5.1 Requesting Graphics
When graphics are needed, create a prompt request using these guidelines:

**Prompt Template:**
```
Create a [SIZE] pixel art image for [SUBJECT].
Style: [STYLE DESCRIPTION]
Background: [transparent/solid color]
Purpose: [in-game icon/portrait/background/etc]
```

**Example Prompts:**
```
Create a 64x64 pixel art icon of a golden key.
Style: Warm tones, slight glow, clean edges
Background: Transparent
Purpose: UI icon for apartment access

Create a 256x256 pixel art portrait of a professional businessperson.
Style: Neutral expression, office attire, diverse
Background: Transparent
Purpose: Tenant archetype portrait
```

### 5.2 Asset Naming Convention
```
tenant_[archetype].png     # tenant_student.png
icon_[function].png        # icon_money.png
design_[style].png         # design_cozy.png
event_[type].png           # event_pipe_burst.png
neighborhood_[type].png    # neighborhood_downtown.png
```

### 5.3 Asset Sizes
- **Icons**: 32x32 or 64x64
- **Portraits**: 256x256
- **Backgrounds**: 1280x720 or larger
- **Design previews**: 256x256

---

## 6. Data-Driven Design

Configuration lives in JSON files under `assets/`:
- `config.json` - Core game balance
- `upgrades.json` - Upgrade definitions
- `building_templates.json` - Building layouts
- `tenant_archetypes.json` - Tenant types
- `neighborhoods.json` - Neighborhood stats

**Prefer adding to JSON over modifying Rust code** when possible.

---

## 7. Common Patterns

### 7.1 UI Actions
UI returns actions, game state handles them:
```rust
pub enum UiAction {
    SelectApartment(u32),
    UpgradeApartment { apartment_id: u32, upgrade_id: String },
    EndTurn,
}
```

### 7.2 Game Events
Events are logged and displayed:
```rust
pub enum GameEvent {
    RentPaid { tenant_name: String, amount: i32 },
    TenantMovedOut { message: String },
    // ...
}
```

### 7.3 Flags System
Buildings and apartments use string flags:
```rust
building.flags.insert("staff_janitor".to_string());
if building.flags.contains("has_laundry") { ... }
```

---

## 8. Testing

Focus tests on:
- Economy calculations
- Simulation rules
- Tenant behavior

Do NOT write tests for UI rendering.

---

## 9. Debugging Tips

- Check the browser console (F12) for WASM errors
- JSON parse errors usually mean malformed config files
- 403 errors on itch.io = assets not being served (check zip structure)
- Black screen in WebGL = likely WASM loading failure

---

## 10. Quick Reference

| Task | Command/Action |
|------|----------------|
| Build Windows | User runs `cargo build --release` |
| Build WebGL | User runs `cargo build --release --target wasm32-unknown-unknown` |
| Package for release | `.\publish.ps1` |
| Test locally (WebGL) | `python -m http.server 8080` in dist/webgl |
| Add new config value | Edit assets/*.json |
| Add new upgrade | Edit assets/upgrades.json |
| Add new building | Edit assets/building_templates.json |
