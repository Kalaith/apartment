# Task 10: Neighborhood System

## Priority: 🔴 CRITICAL (Foundation)
## Dependencies: Task 06 (UI), Task 04 (Economy)
## Estimated Effort: 4-5 hours

## Objective
Implement the neighborhood system that provides the foundation for multi-building gameplay. Each neighborhood has distinct characteristics affecting tenant attraction, rent levels, and building reputation.

## Implementation Status: ✅ COMPLETE

### Files Created
- `src/city/mod.rs` - Module exports
- `src/city/neighborhood.rs` - Neighborhood types and stats

### Features Implemented

1. **Neighborhood Types** (`NeighborhoodType` enum)
   - `Downtown` - High rent, high turnover, attracts professionals/students
   - `Suburbs` - Family-friendly, stable, moderate returns
   - `Industrial` - Affordable, gentrifying, attracts artists/students
   - `Historic` - Strict regulations, appeals to elderly

2. **Neighborhood Stats** (`NeighborhoodStats` struct)
   - Crime level (0-100)
   - Transit access (0-100)
   - Walkability (0-100)
   - School quality (0-100) - important for families
   - Services (shops, cafes) (0-100)
   - Rent demand (affects applications)
   - Gentrification pressure (0-100)

3. **Dynamic Changes**
   - Stats change over time via `tick()` method
   - Gentrification increases in Industrial areas
   - Crime fluctuates slightly
   - Rent demand responds to market conditions

4. **Player Reputation**
   - Per-neighborhood reputation (0-100)
   - Affects tenant attraction
   - Updated based on building performance

## Key Methods
- `Neighborhood::new()` - Create with default stats for type
- `NeighborhoodStats::for_type()` - Initialize stats based on type
- `NeighborhoodStats::appeal_score()` - Calculate overall appeal
- `neighborhood.tick()` - Monthly updates
- `common_archetypes()` - Which tenants prefer this area

## Integration Points
- `City` struct manages neighborhoods
- Buildings reference neighborhood by ID
- Tenant generation uses neighborhood preferences
- Rent calculations multiply by neighborhood factors

## Testing
```rust
cargo test neighborhood -- --nocapture
```
