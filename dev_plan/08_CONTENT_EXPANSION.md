# Task 08: Content Expansion
## Priority: 🟢 MEDIUM (Replayability)
## Dependencies: Task 05 (Simulation)
## Estimated Effort: 4-5 hours

## Objective
Add depth to the simulation with new tenant types, random events, and more strategic choices.

## Deliverables

### 1. New Tenant Archetypes
- **Family**: 
  - Needs: Large Units (requires unit merging or specific large unit types if supported, otherwise just Medium).
  - Hates: Noise (Critical).
  - Tolerance: Low for checking condition.
- **Elderly**:
  - Needs: Low Floors (Floor 1-2).
  - Hates: Parties/Noise.
  - Bonus: Very stable (rarely moves out).

### 2. Events System (`src/simulation/random_events.rs`)
- **Trigger Logic**: % chance per tick to spawn an event.
- **Event Types**:
  - *Heatwave*: Happiness decays faster without "AC" upgrade (new upgrade?).
  - *Pipe Burst*: Random apartment takes 30% condition damage.
  - *Gentrification*: Rent tolerance increases for 6 months.
  - *Inspection*: Fine if average condition < 40%.
- **UI Integration**: specific modal or toast notification for events.

### 3. Advanced Economy
- **Variable Rent**: 
  - Add UI slider to set rent per unit.
  - Higher rent = significantly fewer applications + faster happiness decay.
  - Lower rent = flood of applications + happiness boost.
- **Upgrades**:
  - *Kitchen Renovation* (Expensive, huge appeal boost).
  - *Amenities* (Laundry room in shared space).

## Implementation Notes
- `TenantArchetype` enum expansion.
- `EventSystem` needs to hook into `tick.rs`.
- `Rent` calculation needs to change from static to dynamic based on player setting.
