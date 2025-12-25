# Task 12: Consequence Systems

## Priority: 🟡 HIGH (Depth)
## Dependencies: Task 03 (Tenant), Task 05 (Simulation)
## Estimated Effort: 5-6 hours

## Objective
Add consequence systems that make player decisions feel meaningful with lasting ramifications, including tenant relationships, regulations/compliance, and gentrification tracking.

## Implementation Status: ✅ COMPLETE

### Files Created
- `src/consequences/mod.rs` - Module exports
- `src/consequences/relationships.rs` - Tenant social network
- `src/consequences/regulations.rs` - Compliance and inspections
- `src/consequences/gentrification.rs` - Displacement tracking

---

## Features Implemented

### 1. Tenant Relationships (`relationships.rs`)

**RelationshipType enum:**
- `Friendly` - Neighbors who help each other (+5 happiness)
- `Neutral` - No strong feelings
- `Hostile` - Noise complaints, disputes (-10 happiness)
- `Romantic` - May combine units (+10 happiness)
- `Family` - Family connections (+8 happiness)

**TenantNetwork struct:**
- Manages all relationships in a building
- `relationships_for()` - Get all relationships for a tenant
- `relationship_between()` - Check specific pair
- `happiness_modifier_for()` - Total happiness impact
- `stability_modifier_for()` - Affects move-out likelihood

**Displacement Tracking:**
- `LongTermTenantRecord` - Tracks tenant history
- `record_move_in()` - Log when tenant arrives
- `mark_displaced()` - Record displacement with reason
- `displaced_count()` / `long_term_count()` - Statistics

---

### 2. Regulations & Compliance (`regulations.rs`)

**RegulationType enum:**
- Fire Safety
- Electrical
- Plumbing
- Structural
- Historic Preservation
- Rent Control
- Accessibility
- Health & Sanitation

**Regulation struct:**
- Active/inactive status
- Compliance tracking
- Violation count
- Inspection timer

**Inspection System:**
- `Inspection` - Records inspection event
- `InspectionResult` - Pass/fail with issues and fines
- `InspectionTrigger` - Scheduled, complaint, random, follow-up

**ComplianceSystem struct:**
- Track regulations per building
- `init_building_regulations()` - Setup for new building
- `inspect_building()` - Perform inspection
- `has_violations()` / `violation_count()` - Quick checks
- `pending_fixes` - Deadlined repairs
- `unpaid_fines` - Outstanding penalties
- `compliance_reputation` - Affects inspection frequency

---

### 3. Gentrification Tracker (`gentrification.rs`)

**DisplacementEvent:**
- Records when tenants are priced out
- Tenant name, archetype, original/final rent
- Displacement reason with reputation impact

**DisplacementReason enum:**
- Rent increase
- Unit conversion
- Renovation
- Eviction
- Neighborhood gentrification
- Building sold

**DemographicSnapshot:**
- Tracks tenant type distribution
- Average rent at a point in time
- `diversity_score()` - How mixed the population is

**GentrificationTracker struct:**
- `displacements` - All displacement events
- `rent_history` - Track rent changes over time
- `demographic_shifts` - Population changes
- `gentrification_score` - Player's impact (0-100)
- `tenants_preserved` - Long-term tenants kept
- `affordable_units` - Below threshold rent

**Key Methods:**
- `record_displacement()` - Log an event
- `record_rent_change()` - Track rent increases
- `would_cause_displacement()` - Check before raising rent
- `displacement_risk()` - Calculate building risk level
- `summary()` - Get consolidated statistics

---

## Integration Points

### Into Happiness Calculation
```rust
let relationship_bonus = network.happiness_modifier_for(tenant_id);
let base_happiness = calculate_base_happiness(...);
let total = base_happiness + relationship_bonus;
```

### Into Tick System
```rust
// Monthly tick
tenant_network.tick(&tenants, &building);
compliance_system.tick(current_month);
gentrification_tracker.update_affordable_units(&building.apartments);
```

### For Rent Changes
```rust
if tracker.would_cause_displacement(&tenant, current_rent, new_rent) {
    // Show warning to player
    // If they proceed, record displacement
}
```

---

## No Morality Meter
These systems intentionally track consequences without judging the player:
- High gentrification score shows impact, not "evil"
- Displacement events are recorded, not punished
- Player sees the human cost of decisions
- Consequences manifest through game mechanics (reputation, etc.)

---

## Testing
```rust
cargo test relationships -- --nocapture
cargo test regulations -- --nocapture
cargo test gentrification -- --nocapture
```
