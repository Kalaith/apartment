# Rust Coding Standards

**Engine**: Macroquad  
**Language**: Rust  

This document defines the coding standards for the Apartment Manager project. Its goal is not academic purity, but long-term sanity. The building may be falling apart, but the code should not be.

These standards prioritize:  
- Readability over cleverness  
- Explicitness over magic  
- Stability over premature optimization  
- A clear mental model for future-you  

## 1. Core Philosophy

### 1.1 Write for Maintenance
This is a simulation-heavy game that will evolve over time. Code should be easy to return to after weeks away.  
- Prefer boring, obvious code  
- Avoid overly abstract designs unless they earn their keep  
- Optimize only when profiling demands it  
- If a junior Rust developer can understand it, you are doing it right.

### 1.2 Consistency Beats Preference
If a pattern already exists in the codebase, follow it even if you dislike it. A consistent codebase is more valuable than a perfect one.

## 2. Project Structure Rules

### 2.1 Module Responsibilities
Each top-level module owns a single conceptual domain:  
- `building/` – Physical structure, apartments, upgrades  
- `tenant/` – Tenants, preferences, applications  
- `economy/` – Money flow, rent, transactions  
- `simulation/` – Time progression, turns, rules  
- `state/` – Global game state and transitions  
- `ui/` – Rendering, input, layout only  
- `narrative/` – Events, mail, story beats  

Avoid cross-domain leakage. UI should never mutate economy directly. Simulation should not render.

### 2.2 File Size Guideline
- Target: 200–400 lines per file  
- Hard limit: 800 lines  
- If a file grows beyond this, split by responsibility.

## 3. Naming Conventions

### 3.1 General Rules
- Types: PascalCase  
- Functions & variables: snake_case  
- Constants: SCREAMING_SNAKE_CASE  
- Modules: snake_case  

Names should describe what the thing is, not how it works.

Good examples:  
```rust
TenantHappiness  
calculate_rent  
apply_building_upgrade  
```

Bad examples:  
```rust
do_thing  
temp2  
handle_stuff  
```

### 3.2 Boolean Naming
Booleans should read like facts:  
```rust
is_occupied  
can_afford_rent  
has_soundproofing  
```  
Avoid `flag`, `value`, or `state` in names.

## 4. Functions & Methods

### 4.1 Function Size
- Target: 20–50 lines  
- Absolute max: 100 lines  
- If a function needs scrolling, it probably needs refactoring.

### 4.2 Single Responsibility
Each function should answer one question or perform one action.

Bad:  
```rust
// Calculates rent, applies happiness change, writes to save state  
fn process_tenant_update() { ... }  
```

Good:  
```rust
fn calculate_rent() { ... }  
fn apply_happiness_delta() { ... }  
fn persist_game_state() { ... }  
```

### 4.3 Argument Count
- Prefer ≤ 3 parameters  
- If more are needed, use a struct  
- This improves readability and future extensibility.

## 5. Data & State Management

### 5.1 Game State Ownership
- There should be a single authoritative game state  
- Mutation happens through well-defined systems  
- Avoid passing mutable references everywhere like loose wires.

### 5.2 Prefer Plain Data
Use structs with clear fields. Avoid overly clever enums with embedded logic unless they model a real-world state machine.  

Game data should be:  
- Serializable (Serde-friendly)  
- Inspectable  
- Easy to debug  

## 6. Error Handling

### 6.1 Prefer Result Over Panics
- `panic!` is acceptable only for truly unrecoverable states  
- Gameplay logic should never panic in normal play  
- Use:  
  - `Result<T, E>` for fallible operations  
  - Graceful degradation for save/load errors  

### 6.2 Custom Error Types
For domain errors (economy, simulation), define small error enums instead of using strings.

## 7. UI Code (Macroquad)

### 7.1 UI Is Dumb
UI code:  
- Reads state  
- Sends intent (commands)  
- It should never contain business logic.  

Bad:  
```rust
// Calculating rent inside a button handler  
fn on_button_click() { calculate_rent(); }  
```

Good:  
```rust
// Button emits UiAction::UpgradeApartment  
fn on_button_click() { emit(UiAction::UpgradeApartment); }  
// Simulation handles consequences  
```

### 7.2 Deterministic Rendering
Rendering must be deterministic and free of side effects. No mutation during draw calls.

## 8. Simulation & Time

### 8.1 Explicit Ticks
All time progression must be explicit:  
- No hidden updates in getters  
- No background mutation  
- End-of-turn logic should live in one clearly named function.

### 8.2 Determinism First
Randomness must be:  
- Seeded  
- Centralized  
- This ensures save/load consistency and reproducibility.

## 9. Comments & Documentation

### 9.1 Comment Why, Not What
Code already explains what it does. Comments should explain why it exists.

Good:  
```rust
// Rent increases are capped to avoid soft-locking low-income tenants  
fn cap_rent_increase(amount: f32) -> f32 { ... }  
```

Bad:  
```rust
// Increase rent by 10%  
fn increase_rent() { ... }  
```

### 9.2 Module-Level Docs
Each module should contain a short `//!` comment explaining its purpose and boundaries.

## 10. Formatting & Tooling

### 10.1 rustfmt
- Always use `cargo fmt`  
- Never fight the formatter  

### 10.2 Clippy
- Run `cargo clippy` regularly  
- Fix warnings unless intentionally ignored  
- Document any `#[allow]` with a comment.

### 10.3 Variable Shadowing
- Avoid variable shadowing (hiding). Do not declare a new variable with the same name as an existing one in the same scope.  
- Unused variables must trigger a warning. Never suppress unused variable warnings with `_` prefixes or `#[allow(unused_variables)]`.

## 11. Testing Guidelines

### 11.1 What to Test
Focus tests on:  
- Economy calculations  
- Simulation rules  
- Tenant behavior logic  
- UI and rendering generally do not need unit tests.

### 11.2 Test Style
- Tests should read like rules  
- Avoid complex setups  
- If a test is hard to write, the code is probably too tangled.

## 12. Final Rule
If a piece of code feels fragile, confusing, or brittle, it probably is. Refactor early. Leave the building better than you found it.