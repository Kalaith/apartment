# Apartment Game - Master Development Plan

## Overview
This document outlines the complete development plan for the Apartment MVP. Tasks are organized into independent modules that can be developed in parallel by multiple agents.

## Dependency Graph

```
                    ┌─────────────────────┐
                    │  01_CORE_ARCHITECTURE │
                    │  (Must be first)     │
                    └──────────┬──────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
         ▼                     ▼                     ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ 02_BUILDING     │  │ 03_TENANT       │  │ 07_DATA         │
│ SYSTEM          │  │ SYSTEM          │  │ DEFINITIONS     │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                     │                     │
         └──────────┬──────────┘                     │
                    │                                │
                    ▼                                │
         ┌─────────────────────┐                     │
         │ 04_ECONOMY_SYSTEM   │◄────────────────────┘
         └──────────┬──────────┘
                    │
                    ▼
         ┌─────────────────────┐
         │ 05_SIMULATION       │
         │ (Time & Game Loop)  │
         └──────────┬──────────┘
                    │
                    ▼
         ┌─────────────────────┐
         │ 06_UI_SYSTEM        │
         │ (Needs all above)   │
         └─────────────────────┘
```

## Parallelization Strategy

### Phase 1 - Foundation (Sequential)
- **01_CORE_ARCHITECTURE** - Must complete first, establishes project structure

### Phase 2 - Domain Systems (Parallel)
These can all be worked on simultaneously:
- **02_BUILDING_SYSTEM** - Apartment and building data structures
- **03_TENANT_SYSTEM** - Tenant archetypes and behavior
- **07_DATA_DEFINITIONS** - JSON data files

### Phase 3 - Logic Integration (Parallel)
- **04_ECONOMY_SYSTEM** - Money, rent, costs
- **05_SIMULATION** - Time progression and state updates

### Phase 4 - User Interface (Sequential)
- **06_UI_SYSTEM** - Requires all systems to be functional

## File Structure (Target)

```
apartment/
├── Cargo.toml
├── src/
│   ├── main.rs                 # Entry point
│   ├── game.rs                 # Main game struct & loop
│   ├── state/
│   │   ├── mod.rs              # State exports
│   │   ├── menu.rs             # Main menu
│   │   ├── gameplay.rs         # Core gameplay state
│   │   └── results.rs          # End game summary
│   ├── building/
│   │   ├── mod.rs              # Building exports
│   │   ├── apartment.rs        # Apartment struct
│   │   ├── building.rs         # Building struct
│   │   └── upgrades.rs         # Repair/upgrade logic
│   ├── tenant/
│   │   ├── mod.rs              # Tenant exports
│   │   ├── tenant.rs           # Tenant struct
│   │   ├── archetype.rs        # Student/Professional/Artist
│   │   ├── happiness.rs        # Happiness calculation
│   │   └── matching.rs         # Tenant-apartment matching
│   ├── economy/
│   │   ├── mod.rs              # Economy exports
│   │   ├── money.rs            # Player funds
│   │   ├── rent.rs             # Rent calculation
│   │   └── costs.rs            # Repair/upgrade costs
│   ├── simulation/
│   │   ├── mod.rs              # Simulation exports
│   │   ├── tick.rs             # Time tick processing
│   │   └── decay.rs            # Condition decay
│   ├── ui/
│   │   ├── mod.rs              # UI exports
│   │   ├── building_view.rs    # Building overview screen
│   │   ├── apartment_panel.rs  # Apartment detail panel
│   │   ├── tenant_list.rs      # Tenant management
│   │   └── notifications.rs    # Event notifications
│   ├── data/
│   │   ├── mod.rs              # Data loader exports
│   │   ├── apartments.rs       # Apartment data loader
│   │   └── tenants.rs          # Tenant data loader
│   └── save/
│       └── mod.rs              # Save/load system
├── assets/
│   ├── apartments.json         # Initial apartment configs
│   ├── tenants.json            # Tenant archetype definitions
│   └── config.json             # Game balance constants
└── dev_plan/
    └── (these task files)
```

## Task Files

| File | Description | Dependencies | Est. Complexity |
|------|-------------|--------------|-----------------|
| 01_CORE_ARCHITECTURE.md | Project setup, state machine | None | Medium |
| 02_BUILDING_SYSTEM.md | Apartment & building structs | 01 | Medium |
| 03_TENANT_SYSTEM.md | Tenant logic & matching | 01 | Medium |
| 04_ECONOMY_SYSTEM.md | Money, rent, costs | 02, 03 | Low |
| 05_SIMULATION.md | Time ticks, decay, updates | 02, 03, 04 | Medium |
| 06_UI_SYSTEM.md | All UI screens | All above | High |
| 07_DATA_DEFINITIONS.md | JSON data files | 01 | Low |

## MVP Completion Criteria

- [ ] Player can view building with 6-8 apartments
- [ ] Player can inspect individual apartment stats
- [ ] Player can repair apartments (costs money, raises condition)
- [ ] Player can upgrade apartment design (Bare → Practical → Cozy)
- [ ] Player can add soundproofing to apartments
- [ ] Player can repair shared hallway
- [ ] Tenants apply to vacant apartments
- [ ] Player can accept/reject tenant applications
- [ ] Tenant happiness calculated each tick based on preferences
- [ ] Unhappy tenants leave
- [ ] Rent collected monthly
- [ ] Condition decays over time
- [ ] Game ends when all units filled with happy tenants OR bankruptcy
- [ ] End summary shows stats

## Development Notes

### Coding Standards
- Follow Rust 2021 edition idioms
- Use `#[derive(Clone, Debug, Serialize, Deserialize)]` for data structs
- Keep systems decoupled - emit effects, resolve separately
- No global mutable state

### Testing Strategy
- Unit tests for calculation functions (happiness, rent, decay)
- Integration tests for tenant matching logic
- Manual playtesting for balance

### Iteration Plan
1. Get buildings and apartments rendering
2. Add tenant movement (apply, accept, leave)
3. Add money flow (rent in, repairs out)
4. Add time progression (ticks, decay)
5. Add win/lose conditions
6. Polish UI and notifications
