# Phase 4D: Tutorial & Narrative

## Goal
Provide a "Soul" to the game through a guided narrative and a clear path for new players.

## Features

### 1. The Mentor (Uncle Artie)
A scripted advisor who guides the player through the first building.
- **Tutorial Milestones**:
  - "The Inherited Mess": Clear the first building of trash.
  - "The First Resident": Step-by-step through Marketing and Vetting.
  - "The Leak": Forces a repair decision to teach the maintenance UI.

### 2. Rivals and Allies
- **The Developer (Magnuson Corp)**: A corporate rival who buys nearby buildings and drives up property taxes through gentrification.
- **Local Council Member**: Gives missions (e.g., "House 3 Students") in exchange for tax breaks.

### 3. The "Legacy" System
- **Building Awards**: "Best Managed Property 2026."
- **Story Archive**: A log of major events (The Fire of '24, The 10-Year Tenant).

## Technical Tasks
- [ ] Implement a `TutorialManager` that hooks into game actions.
- [ ] Add `NarrativeNPC` definitions.
- [ ] Create a "Mission" system to track active narrative goals.
- [ ] Expand the "Win/Loss" screen into a full "Career Summary."
