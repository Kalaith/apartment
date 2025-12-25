# Task 14: Advanced Economy

## Priority: 🟡 HIGH (Complexity)
## Dependencies: Task 11 (Multi-Building), Task 04 (Economy)
## Estimated Effort: 4-5 hours

## Objective
Expand the economic simulation beyond simple rent/repairs to include operating costs, multiple revenue streams, and economic cycles that create natural pressure and decision points.

## Implementation Status: 📋 PLANNED

---

## Planned Features

### 1. Operating Costs (Expanding Beyond Repairs)

**Staff Wages:**
- Maintenance worker (reduces repair time/cost)
- Security guard (reduces crime impact)
- Building manager (allows portfolio expansion)
- Cleaning service (improves hallway condition)

**Utilities:**
- Base utility costs per building
- Option to include in rent or bill separately
- Energy-efficient upgrades reduce costs

**Insurance:**
- Required for all buildings
- Rates vary by condition and neighborhood
- Claims after disasters
- Premium vs basic coverage

**Property Taxes:**
- Annual or monthly payments
- Can increase dramatically (assessments)
- Varies by neighborhood value

### 2. Revenue Streams

**Commercial Ground Floor:**
- Shops generate rental income
- Cafes increase building appeal
- Services (laundromat, etc.)

**Parking Spaces:**
- Monthly parking rental
- More valuable in Downtown

**Laundry Room Income:**
- Per-use revenue
- Requires laundry upgrade

**Billboard / Phone Tower Leases:**
- Passive income
- May reduce building appeal

### 3. Economic Cycles

**Market Conditions:**
- Boom: High rent demand, expensive acquisitions
- Normal: Balanced market
- Recession: Reduced demand, cheaper properties

**Rent Demand Fluctuation:**
- Seasonal patterns (fall = students)
- Economic cycle effects
- Neighborhood-specific trends

**Interest Rate Changes:**
- Affects mortgage costs
- Influences property values
- Creates refinancing opportunities

### 4. Loans & Debt

**Bank Loans:**
- Property-backed mortgages
- Renovation loans
- Monthly payment obligations
- Default consequences

**Credit Rating:**
- Affects loan availability
- Impacts interest rates
- Built through consistent payments

**Emergency Funds:**
- Recommended reserve amount
- Warning when depleted

---

## Data Structures Needed

```rust
pub struct OperatingCosts {
    pub staff: Vec<StaffMember>,
    pub utility_base: i32,
    pub insurance_premium: i32,
    pub property_tax_annual: i32,
}

pub struct StaffMember {
    pub role: StaffRole,
    pub monthly_wage: i32,
    pub effectiveness: f32, // 0-1
}

pub enum StaffRole {
    Maintenance,
    Security,
    Manager,
    Cleaner,
}

pub struct CommercialUnit {
    pub business_type: CommercialType,
    pub monthly_rent: i32,
    pub appeal_bonus: i32,
    pub is_occupied: bool,
}

pub struct Loan {
    pub principal: i32,
    pub interest_rate: f32,
    pub monthly_payment: i32,
    pub remaining_balance: i32,
    pub term_months: u32,
    pub months_remaining: u32,
}

pub struct EconomicCycle {
    pub current_phase: MarketPhase,
    pub phase_duration: u32,
    pub rent_modifier: f32,
    pub acquisition_modifier: f32,
}

pub enum MarketPhase {
    Boom,
    Normal,
    Recession,
}
```

---

## Economic Pressure Design

The goal is to create **interesting decisions**, not frustration:

1. **Positive Pressure**: Opportunities to expand, upgrade, hire
2. **Negative Pressure**: Costs that require attention but don't cripple
3. **Choices**: Trade-offs between short-term cash and long-term value
4. **Recovery**: Ability to bounce back from setbacks

---

## Balance Considerations

- Operating costs should be ~20-30% of gross rent
- Staff should provide clear ROI
- Economic cycles visible 1-2 months before changing
- Loans available but with meaningful interest
- Emergency reserve = 3 months operating costs

---

## Integration Points
- City struct already has economy_health, interest_rate, inflation_rate
- FinancialLedger needs expense categories
- PlayerFunds needs loan tracking
- UI needs financial report panel

---

## Future Implementation
This task is deferred until multi-building is fully integrated into gameplay state.
