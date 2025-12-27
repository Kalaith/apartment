# Data-Driven Refactoring Opportunities

This document outlines all hardcoded constants and data structures in the codebase that could be extracted to JSON configuration files for a truly data-driven game design.

---

## Priority Legend
| Priority | Meaning |
|:---:|---|
| 🔴 | **High** – Core gameplay, frequently tweaked |
| 🟡 | **Medium** – Affects balance, occasionally changed |
| 🟢 | **Low** – Rarely changes, cosmetic or structural |

---

## 1. Apartment Properties
**File:** `src/building/apartment.rs`

### DesignType Scores (🔴)
```rust
// Lines 22-28
DesignType::Bare => 0,
DesignType::Practical => 20,
DesignType::Cozy => 40,
```
**Recommendation:** Move to `config.json` under `design_appeal_scores`.

### ApartmentSize Base Rent (🔴)
```rust
// Lines 38-43
ApartmentSize::Small => 600,
ApartmentSize::Medium => 850,
```
**Note:** Partially in `config.json` but duplicated here. Remove hardcoded values.

### ApartmentSize Space Score (🟡)
```rust
// Lines 45-50
ApartmentSize::Small => 0,
ApartmentSize::Medium => 15,
```

### NoiseLevel Penalty (🔴)
```rust
// Lines 60-65
NoiseLevel::Low => 0,
NoiseLevel::High => -20,
```

### Market Value Constants (🟡)
```rust
// Lines 175-216
ApartmentSize::Small => 50_000,
ApartmentSize::Medium => 75_000,
// Condition bonus: +$500 per point above 50, -$300 per point below 50
// Design bonuses: 0, 5_000, 15_000
// Kitchen bonuses: 0, 8_000, 15_000
// Floor bonus: $2,000 per floor
// Soundproofing bonus: 3_000
// Noise penalty: -5_000
```

---

## 2. Building System
**File:** `src/building/building.rs`

### Marketing Costs (🔴)
```rust
// Lines 18-25
MarketingType::None => 0,
MarketingType::SocialMedia => 50,
MarketingType::LocalNewspaper => 150,
MarketingType::PremiumAgency => 500,
```

### Starting Hallway Condition (🟡)
```rust
// Line 89
hallway_condition: 60,
```

### Monthly Decay Rates (🔴)
```rust
// Lines 167-170
apt.decay_condition(2);    // Apartment decay per tick
self.decay_hallway(1);     // Hallway decay per tick
```
**Note:** Already in `config.json` under `decay.*`, but not consumed here.

### Condo HOA Default (🟡)
```rust
// Line 191
board.add_unit(..., 200, ...);  // $200 HOA default
```

### Buyback Multiplier (🟡)
```rust
// Line 250
let buyback_price = (... * 1.1) as i32;  // 110% buyback
```

### Laundry Appeal Bonus (🟡)
```rust
// Lines 148-150
if self.has_laundry { score += 10; }
```

---

## 3. Economy & Costs
**File:** `src/economy/costs.rs`

### Property Tax Rate (🔴)
```rust
// Lines 10-12
(rent_income as f32 * 0.10) as i32  // 10% of rent
```

### Utility Cost Per Unit (🔴)
```rust
// Line 22
occupied * 50  // $50 per occupied unit
```

### Insurance Rates (🟡)
```rust
// Lines 31-35
let base_rate = 150;
let discount = if building.hallway_condition > 80 { 50 } else { 0 };
```

### Default Starting Funds (🟡)
**File:** `src/economy/money.rs`
```rust
// Line 116
Self::new(5000)
```

---

## 4. Tenant Vetting
**File:** `src/tenant/vetting.rs`

### Vetting Costs (🔴)
```rust
// Lines 5-6
pub const COST_CREDIT_CHECK: i32 = 25;
pub const COST_BACKGROUND_CHECK: i32 = 10;
```

### Credit Score Thresholds (🟡)
```rust
// Lines 37-47
if score >= 90 => "Excellent..."
if score >= 75 => "Good..."
if score >= 60 => "Average..."
// etc.
```

---

## 5. Tenant Matching
**File:** `src/tenant/matching.rs`

### Match Scoring Constants (🔴)
```rust
// Lines 17-102
score = 50;                    // Starting score
score -= 40;                   // Desperate/unqualified penalty
rent_diff > 200 => score += 15
rent_diff > 0   => score += 8
rent_diff > -100 => score -= 5
rent_diff <= -100 => score -= 20
condition >= 80 => bonus = 15
condition >= 60 => bonus = 8
condition < 50  => penalty = 10
// Noise bonuses/penalties: 12, 15
// Design bonus: 18
// Size bonus: 5
```

### Lease Offer Defaults (🟡)
```rust
// Lines 128-136
security_deposit_months: 1,
lease_duration_months: 12,
cleaning_fee: 0,
```

### Lease Acceptance Probabilities (🟡)
```rust
// Lines 139-197
deposit_penalty (2 months): 0.15
deposit_penalty (3+ months): 0.35
short_lease_bonus: 0.1
long_lease_penalty: -0.15
cleaning_fee_impact: fee_ratio * sensitivity
good_deal_bonus: 0.1
```

---

## 6. Tenant Archetypes
**File:** `src/tenant/archetype.rs`

### All Archetype Preferences (🔴)
```rust
// Lines 25-88
// Each archetype defines:
// - rent_sensitivity: 0.0-1.0
// - condition_sensitivity: 0.0-1.0
// - noise_sensitivity: 0.0-1.0
// - design_sensitivity: 0.0-1.0
// - ideal_rent_max: i32
// - min_acceptable_condition: i32
// - prefers_quiet: bool
// - preferred_design: Option<DesignType>
// - hates_design: Option<DesignType>
```
**Note:** Already have `assets/tenant_archetypes.json` – ensure it's being consumed.

---

## 7. Win/Lose Conditions
**File:** `src/simulation/win_condition.rs`

### Victory Thresholds (🔴)
```rust
// Lines 22-30
pub const MIN_HAPPINESS: i32 = 60;
pub const FULL_OCCUPANCY_REQUIRED: bool = true;
pub const MIN_TICKS_FOR_VICTORY: u32 = 6;
```
**Note:** Partially in `config.json` under `win_conditions`, but not consumed.

### Bankruptcy/All-Left Check (🟡)
```rust
// Line 45
if tenants.is_empty() && current_tick > 3
```

---

## 8. Decay & Condition Thresholds
**File:** `src/simulation/decay.rs`

### Condition Thresholds (🔴)
```rust
// Lines 9-13
pub const POOR_CONDITION_THRESHOLD: i32 = 40;
pub const CRITICAL_CONDITION_THRESHOLD: i32 = 20;
```

---

## 9. Critical Failures
**File:** `src/simulation/tick.rs`

### Failure Probabilities & Costs (🔴)
```rust
// Lines 244-276
let mut prob = 0.005;
prob *= 0.5;                    // Security staff reduction
let cost = 1500;                // Boiler failure cost
t.happiness - 30                // Unfixed boiler happiness penalty
let cost = 2500;                // Structural issue cost
hallway_condition - 20          // Unfixed structural penalty
```

### Staff Effects (🔴)
```rust
// Lines 206-231
// Janitor: +1 condition if < 90 and > 50
// Security: +2 happiness per tenant
// Manager: +1 happiness per tenant
```

---

## 10. Relationships
**File:** `src/consequences/relationships.rs`

### Relationship Happiness Modifiers (🔴)
```rust
// Lines 20-28
RelationshipType::Friendly => 5,
RelationshipType::Neutral => 0,
RelationshipType::Hostile => -10,
RelationshipType::Romantic => 8,
RelationshipType::Family => 10,
```

### Relationship Formation Chances (🟡)
```rust
// Line 176
macroquad::rand::gen_range(0, 100) < 5  // 5% monthly chance

// Lines 82-87
gen_range(0, 100) < 5  // 5% hostile cool-down chance
self.strength -5
self.strength < 20  // Transition to neutral
```

### Cohesion Calculation (🟡)
```rust
// Lines 241-262
if count >= 3 { bonus += 5 + (count - 3) * 2; }
bonus += friendly_count * 2;
bonus -= hostile_count * 5;
bonus -= (tensions.len() as i32) * 8;
```

### Council Formation Threshold (🟡)
```rust
// Line 273
relative_unhappiness >= 0.4  // 40% unhappy
```

---

## 11. Gentrification
**File:** `src/consequences/gentrification.rs`

### Affordable Threshold (🔴)
```rust
// Line 81
const AFFORDABLE_THRESHOLD: i32 = 700;
```

### Gentrification Score Calculation (🟡)
```rust
// Lines 72-75
if increase_percent > 10 {
    self.gentrification_score + increase_percent / 5
}
```

---

## 12. Tutorial & NPCs
**File:** `src/narrative/tutorial.rs`

### NPC Starting Relationships (🟡)
```rust
// Lines 23-28
NpcRole::Mentor => 50,
NpcRole::Ally => 30,
NpcRole::Rival => -30,
NpcRole::Neutral => 0,
```

### Tutorial Messages (🟢)
```rust
// Lines 88-92
// All tutorial strings are hardcoded
```
**Recommendation:** Move to `assets/text_strings.json` or a dedicated `tutorial.json`.

### Rival Introduction Timing (🟡)
```rust
// Line 188
month >= 6  // Introduce rival after 6 months
```

---

## 13. UI Constants
**File:** `src/ui/common.rs`

### Color Palette (🟢)
```rust
// Lines 5-28
// All colors are hardcoded:
BACKGROUND, PANEL, PANEL_HEADER, TEXT, TEXT_BRIGHT, TEXT_DIM,
ACCENT, POSITIVE, WARNING, NEGATIVE, VACANT, OCCUPIED, SELECTED,
HOVERED, STUDENT, PROFESSIONAL, ARTIST
```
**Recommendation:** Consider a `theme.json` for customizable UI themes.

### Layout Constants (🟢)
```rust
// Lines 56-65
HEADER_HEIGHT: 60.0,
FOOTER_HEIGHT: 100.0,
PANEL_SPLIT: 0.6,
PADDING: 10.0,
UNIT_WIDTH: 120.0,
UNIT_HEIGHT: 80.0,
UNIT_GAP: 15.0,
FLOOR_HEIGHT: 100.0,
```

### Happiness/Condition Thresholds for Colors (🟡)
```rust
// Lines 44-53, 136-153
// happiness_icon: 85, 70, 50, 30
// condition_color: 80, 50, 30
// happiness_color: 70, 40, 20
```

---

## Proposed New JSON Files

| File | Contents |
|------|----------|
| `assets/balance.json` | Match scoring, lease probabilities, relationship modifiers |
| `assets/economy.json` | Costs, taxes, insurance, failure probabilities |
| `assets/archetypes.json` | *(exists)* – ensure it's consumed |
| `assets/theme.json` | Colors, layout constants |
| `assets/tutorial.json` | NPC data, tutorial messages, milestone triggers |
| `assets/events.json` | Critical failure definitions, random event pools |

---

## Migration Strategy

1. **Phase 1 (High Impact):** Happiness config ✅, Match scoring ✅, Decay/Thresholds ✅, Archetypes JSON ✅
2. **Phase 2 (Economy):** Tax rates ✅, Insurance ✅, Vetting costs ✅, Marketing costs ✅
3. **Phase 3 (Balance):** Relationship modifiers ✅, Cohesion calculation ✅, Gentrification thresholds ✅
4. **Phase 4 (Polish):** Theme colors ✅, Layout constants ✅, UI thresholds ✅




