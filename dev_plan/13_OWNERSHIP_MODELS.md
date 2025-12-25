# Task 13: Ownership Models

## Priority: 🟢 MEDIUM (Strategy)
## Dependencies: Task 11 (Multi-Building), Task 04 (Economy)
## Estimated Effort: 4 hours

## Objective
Expand building ownership models to create strategic variety. Allow for condominium conversions, different tenant arrangements, and building specialization.

## Implementation Status: 📋 PLANNED

---

## Planned Features

### 1. Condo Conversion
- Convert rental units to condominiums
- Sell individual units for cash
- Condo owners pay monthly HOA fees
- Owners can vote on building decisions
- Mixed ownership creates management complexity

### 2. Owner-Occupied Units
- Sold units remove rental income
- Owners may block certain upgrades
- Condo board voting system
- Owner complaints have more weight

### 3. Social Housing / Section 8
- Guaranteed government income
- Strict maintenance requirements
- Must accept qualifying tenants
- Reputation benefits in community
- Lower per-unit income but guaranteed

### 4. Building Specialization
- Luxury apartments
  - High margins, demanding tenants
  - Require premium finishes
  
- Artist lofts
  - Lower rent, creative community
  - May qualify for arts grants
  
- Family housing
  - Stable, long leases
  - Need larger units, good schools nearby
  
- Student dormitory
  - Seasonal turnover
  - Bulk contracts with universities

### 5. Cooperative Buildings
- Tenants collectively own building
- Different management dynamics
- Player becomes hired manager
- Focus on resident satisfaction

---

## Data Structures Needed

```rust
pub enum OwnershipType {
    FullRental,           // Player owns all, rents all
    MixedOwnership,       // Some units sold as condos
    FullCondo,            // All units sold, player manages
    CooperativeHousing,   // Tenant-owned
    SocialHousing,        // Section 8 / subsidized
}

pub struct CondoUnit {
    pub apartment_id: u32,
    pub owner_name: String,
    pub monthly_hoa: i32,
    pub owner_satisfaction: i32,
    pub voting_power: i32,
}

pub struct CondoBoard {
    pub units: Vec<CondoUnit>,
    pub pending_votes: Vec<BoardVote>,
    pub reserve_fund: i32,
}

pub struct BoardVote {
    pub proposal: String,
    pub cost: i32,
    pub votes_for: u32,
    pub votes_against: u32,
    pub deadline_month: u32,
}
```

---

## Integration Points
- Building struct needs ownership type
- Economy needs HOA fee calculations
- Upgrades may require board approval
- Tenant flow changes for social housing
- Specialization affects which archetypes apply

---

## Future Implementation
This task is deferred until multi-building is fully integrated into gameplay state.
