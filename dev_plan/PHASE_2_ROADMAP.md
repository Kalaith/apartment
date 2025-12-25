# Phase 2: Polish & Expansion Roadmap

## Overview
With the MVP functionality complete, Phase 2 focuses on "Juice", Content Depth, and Persistence. These tracks are designed to be run **concurrently** by different agents.

## Concurrent Tracks

### Track A: Visuals & UX (The "Feel")
**Goal**: Make the game feel responsive and satisfying, moving away from "developer UI".
- **A1_VISUAL_POLISH**:
    - Animated transitions for panels (slide in/out).
    - Floating text effects for money changes (`+$500`, `-$100`).
    - Color coding refinement (archetypes get distinct colors).
    - Mouse cursor feedback (hover states, active states).
- **A2_FEEDBACK_JUICE**:
    - Screen shake on bad events (critical failure).
    - Particles/Confetti on positive events (full occupancy, perfect happiness).
    - Sound effect triggers (stubbed out for now, but logical hooks needed).

### Track B: Content Expansion (The "Variety")
**Goal**: Add replayability and strategic depth.
- **B1_ADVANCED_TENANTS**:
    - Implement *Families* (need larger units, sensitive to noise).
    - Implement *Elderly* (need lower floors, sensitive to condition).
    - Add "Tenant Traits" (e.g., "Hoarder", "Musician") for micro-variations.
- **B2_EVENTS_SYSTEM**:
    - Random monthly events (Heatwave,Pipe Burst, Inspection).
    - Choices with trade-offs (e.g., "Cheap fix vs Expensive fix").
- **B3_ADVANCED_ECONOMY**:
    - Variable Rent: UI to set rent per unit.
    - Market fluctuations (rent demand goes up/down).

### Track C: Systems Deepening (The "Bones")
**Goal**: Technical robustness and persistence.
- **C1_DATA_DRIVEN**:
    - Fully implement `src/data` loading from JSON.
    - Move hardcoded balance values (costs, happiness weights) to `assets/balance.json`.
- **C2_PERSISTENCE**:
    - Implement `src/save` module.
    - Serialize/Deserialize `GameState`.
    - Auto-save functionality.

## Task Files Structure

New task files created for these tracks:

| File | Description | Track |
|------|-------------|-------|
| `dev_plan/07_VISUAL_POLISH.md` | Animations, floating text, better layout | A |
| `dev_plan/08_CONTENT_EXPANSION.md` | New archetypes, random events | B |
| `dev_plan/09_PERSISTENCE_AND_DATA.md` | Save systems, JSON loading | C |

## Suggested Next Step
Pick **Track A (Visual Polish)** or **Track C (Persistence)** as they are least dependent on gameplay balance changes.
