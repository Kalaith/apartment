# Task 02: Building System

## Priority: 🟡 HIGH
## Dependencies: Task 01 (Core Architecture)
## Estimated Effort: 2-3 hours
## Can Parallel With: Task 03, Task 07

---

## Objective
Implement the building and apartment data structures, including all properties needed for the MVP simulation.

---

## Deliverables

### 1. src/building/mod.rs

```rust
mod apartment;
mod building;
mod upgrades;

pub use apartment::{Apartment, DesignType, ApartmentSize, NoiseLevel};
pub use building::Building;
pub use upgrades::{UpgradeAction, apply_upgrade};
```

### 2. src/building/apartment.rs

**Apartment Properties (from MVP):**
- Condition: 0-100
- Design Type: Bare, Practical, Cozy
- Size: Small, Medium
- Noise Level: Low, High
- Rent Price: i32
- Has Soundproofing: bool
- Tenant ID: Option<u32>

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DesignType {
    Bare,
    Practical,
    Cozy,
}

impl DesignType {
    /// Returns the next design upgrade level, if available
    pub fn next_upgrade(&self) -> Option<DesignType> {
        match self {
            DesignType::Bare => Some(DesignType::Practical),
            DesignType::Practical => Some(DesignType::Cozy),
            DesignType::Cozy => None, // Max level
        }
    }
    
    /// Design appeal score (affects tenant happiness)
    pub fn appeal_score(&self) -> i32 {
        match self {
            DesignType::Bare => 0,
            DesignType::Practical => 20,
            DesignType::Cozy => 40,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ApartmentSize {
    Small,
    Medium,
}

impl ApartmentSize {
    pub fn base_rent(&self) -> i32 {
        match self {
            ApartmentSize::Small => 600,
            ApartmentSize::Medium => 900,
        }
    }
    
    pub fn space_score(&self) -> i32 {
        match self {
            ApartmentSize::Small => 0,
            ApartmentSize::Medium => 15,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NoiseLevel {
    Low,
    High,
}

impl NoiseLevel {
    pub fn noise_penalty(&self) -> i32 {
        match self {
            NoiseLevel::Low => 0,
            NoiseLevel::High => -20,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Apartment {
    pub id: u32,
    pub unit_number: String,        // e.g., "1A", "2B"
    pub floor: u32,
    
    // Core stats
    pub condition: i32,             // 0-100
    pub design: DesignType,
    pub size: ApartmentSize,
    pub base_noise: NoiseLevel,     // Inherent noise (street-facing, etc.)
    pub has_soundproofing: bool,
    pub rent_price: i32,
    
    // Occupancy
    pub tenant_id: Option<u32>,
}

impl Apartment {
    pub fn new(id: u32, unit_number: &str, floor: u32, size: ApartmentSize, base_noise: NoiseLevel) -> Self {
        let rent_price = size.base_rent();
        Self {
            id,
            unit_number: unit_number.to_string(),
            floor,
            condition: 50,  // Start at half condition
            design: DesignType::Bare,
            size,
            base_noise,
            has_soundproofing: false,
            rent_price,
        }
    }
    
    /// Current effective noise level (considers soundproofing)
    pub fn effective_noise(&self) -> NoiseLevel {
        if self.has_soundproofing {
            NoiseLevel::Low
        } else {
            self.base_noise.clone()
        }
    }
    
    /// Is the apartment currently vacant?
    pub fn is_vacant(&self) -> bool {
        self.tenant_id.is_none()
    }
    
    /// Calculate overall apartment quality score (0-100)
    pub fn quality_score(&self) -> i32 {
        let base = self.condition;
        let design_bonus = self.design.appeal_score();
        let noise_mod = self.effective_noise().noise_penalty();
        let space_bonus = self.size.space_score();
        
        (base + design_bonus + noise_mod + space_bonus).clamp(0, 100)
    }
    
    /// Apply condition decay (called each tick)
    pub fn decay_condition(&mut self, amount: i32) {
        self.condition = (self.condition - amount).max(0);
    }
    
    /// Repair the apartment
    pub fn repair(&mut self, amount: i32) {
        self.condition = (self.condition + amount).min(100);
    }
    
    /// Upgrade design to next level
    pub fn upgrade_design(&mut self) -> bool {
        if let Some(next) = self.design.next_upgrade() {
            self.design = next;
            true
        } else {
            false
        }
    }
    
    /// Install soundproofing
    pub fn install_soundproofing(&mut self) {
        self.has_soundproofing = true;
    }
    
    /// Move a tenant in
    pub fn move_in(&mut self, tenant_id: u32) {
        self.tenant_id = Some(tenant_id);
    }
    
    /// Move tenant out
    pub fn move_out(&mut self) {
        self.tenant_id = None;
    }
}
```

### 3. src/building/building.rs

**Building Properties:**
- Name
- Collection of Apartments
- Hallway Condition: 0-100 (shared space)

```rust
use serde::{Deserialize, Serialize};
use super::{Apartment, ApartmentSize, NoiseLevel};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub name: String,
    pub apartments: Vec<Apartment>,
    pub hallway_condition: i32,  // 0-100, affects building appeal
}

impl Building {
    /// Create a new building with generated apartments
    pub fn new(name: &str, num_floors: u32, units_per_floor: u32) -> Self {
        let mut apartments = Vec::new();
        let mut id = 0;
        
        for floor in 1..=num_floors {
            for unit in 0..units_per_floor {
                let unit_letter = (b'A' + unit as u8) as char;
                let unit_number = format!("{}{}", floor, unit_letter);
                
                // Alternate sizes and noise levels for variety
                let size = if (floor + unit) % 2 == 0 {
                    ApartmentSize::Small
                } else {
                    ApartmentSize::Medium
                };
                
                // Ground floor and street-facing (A) units are noisier
                let noise = if floor == 1 || unit == 0 {
                    NoiseLevel::High
                } else {
                    NoiseLevel::Low
                };
                
                apartments.push(Apartment::new(id, &unit_number, floor, size, noise));
                id += 1;
            }
        }
        
        Self {
            name: name.to_string(),
            apartments,
            hallway_condition: 60,  // Start slightly worn
        }
    }
    
    /// Create the default MVP building (6 units, 3 floors, 2 per floor)
    pub fn default_mvp() -> Self {
        Self::new("Sunset Apartments", 3, 2)
    }
    
    /// Get apartment by ID
    pub fn get_apartment(&self, id: u32) -> Option<&Apartment> {
        self.apartments.iter().find(|a| a.id == id)
    }
    
    /// Get mutable apartment by ID
    pub fn get_apartment_mut(&mut self, id: u32) -> Option<&mut Apartment> {
        self.apartments.iter_mut().find(|a| a.id == id)
    }
    
    /// Get all vacant apartments
    pub fn vacant_apartments(&self) -> Vec<&Apartment> {
        self.apartments.iter().filter(|a| a.is_vacant()).collect()
    }
    
    /// Get all occupied apartments
    pub fn occupied_apartments(&self) -> Vec<&Apartment> {
        self.apartments.iter().filter(|a| !a.is_vacant()).collect()
    }
    
    /// Count vacant units
    pub fn vacancy_count(&self) -> usize {
        self.apartments.iter().filter(|a| a.is_vacant()).count()
    }
    
    /// Count occupied units
    pub fn occupancy_count(&self) -> usize {
        self.apartments.iter().filter(|a| !a.is_vacant()).count()
    }
    
    /// Calculate overall building appeal (affects tenant applications)
    pub fn building_appeal(&self) -> i32 {
        let hallway_factor = self.hallway_condition / 2;  // 0-50
        let avg_condition: i32 = if self.apartments.is_empty() {
            0
        } else {
            self.apartments.iter().map(|a| a.condition).sum::<i32>() 
                / self.apartments.len() as i32
        };
        let avg_factor = avg_condition / 2;  // 0-50
        
        hallway_factor + avg_factor
    }
    
    /// Repair hallway
    pub fn repair_hallway(&mut self, amount: i32) {
        self.hallway_condition = (self.hallway_condition + amount).min(100);
    }
    
    /// Decay hallway condition
    pub fn decay_hallway(&mut self, amount: i32) {
        self.hallway_condition = (self.hallway_condition - amount).max(0);
    }
    
    /// Apply decay to all apartments and hallway
    pub fn apply_monthly_decay(&mut self) {
        for apt in &mut self.apartments {
            apt.decay_condition(2);  // Slow decay
        }
        self.decay_hallway(1);  // Even slower for shared space
    }
}
```

### 4. src/building/upgrades.rs

**Upgrade Actions:**
- Repair apartment (costs money, raises condition)
- Upgrade design (Bare → Practical → Cozy)
- Add soundproofing
- Repair hallway

```rust
use super::{Apartment, Building, DesignType};

/// Costs for various upgrades
pub mod costs {
    pub const REPAIR_PER_POINT: i32 = 10;      // $10 per condition point
    pub const UPGRADE_TO_PRACTICAL: i32 = 500;
    pub const UPGRADE_TO_COZY: i32 = 1000;
    pub const SOUNDPROOFING: i32 = 300;
    pub const HALLWAY_REPAIR_PER_POINT: i32 = 15;
}

#[derive(Clone, Debug)]
pub enum UpgradeAction {
    RepairApartment { apartment_id: u32, amount: i32 },
    UpgradeDesign { apartment_id: u32 },
    AddSoundproofing { apartment_id: u32 },
    RepairHallway { amount: i32 },
}

impl UpgradeAction {
    /// Calculate the cost of this upgrade
    pub fn cost(&self, building: &Building) -> Option<i32> {
        match self {
            UpgradeAction::RepairApartment { amount, .. } => {
                Some(amount * costs::REPAIR_PER_POINT)
            }
            UpgradeAction::UpgradeDesign { apartment_id } => {
                let apt = building.get_apartment(*apartment_id)?;
                match apt.design {
                    DesignType::Bare => Some(costs::UPGRADE_TO_PRACTICAL),
                    DesignType::Practical => Some(costs::UPGRADE_TO_COZY),
                    DesignType::Cozy => None,  // Already max
                }
            }
            UpgradeAction::AddSoundproofing { apartment_id } => {
                let apt = building.get_apartment(*apartment_id)?;
                if apt.has_soundproofing {
                    None  // Already has it
                } else {
                    Some(costs::SOUNDPROOFING)
                }
            }
            UpgradeAction::RepairHallway { amount } => {
                Some(amount * costs::HALLWAY_REPAIR_PER_POINT)
            }
        }
    }
    
    /// Check if this upgrade is valid (can be performed)
    pub fn is_valid(&self, building: &Building) -> bool {
        self.cost(building).is_some()
    }
}

/// Apply an upgrade action to the building
/// Returns the cost if successful, None if failed
pub fn apply_upgrade(building: &mut Building, action: &UpgradeAction) -> Option<i32> {
    let cost = action.cost(building)?;
    
    match action {
        UpgradeAction::RepairApartment { apartment_id, amount } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.repair(*amount);
        }
        UpgradeAction::UpgradeDesign { apartment_id } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            if !apt.upgrade_design() {
                return None;
            }
        }
        UpgradeAction::AddSoundproofing { apartment_id } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.install_soundproofing();
        }
        UpgradeAction::RepairHallway { amount } => {
            building.repair_hallway(*amount);
        }
    }
    
    Some(cost)
}

/// Get available upgrades for an apartment
pub fn available_apartment_upgrades(apt: &Apartment) -> Vec<UpgradeAction> {
    let mut upgrades = Vec::new();
    
    // Repair is always available if condition < 100
    if apt.condition < 100 {
        let repair_amount = (100 - apt.condition).min(25);  // Max 25 at a time
        upgrades.push(UpgradeAction::RepairApartment {
            apartment_id: apt.id,
            amount: repair_amount,
        });
    }
    
    // Design upgrade if not at max
    if apt.design.next_upgrade().is_some() {
        upgrades.push(UpgradeAction::UpgradeDesign {
            apartment_id: apt.id,
        });
    }
    
    // Soundproofing if not installed
    if !apt.has_soundproofing {
        upgrades.push(UpgradeAction::AddSoundproofing {
            apartment_id: apt.id,
        });
    }
    
    upgrades
}

/// Get available building-wide upgrades
pub fn available_building_upgrades(building: &Building) -> Vec<UpgradeAction> {
    let mut upgrades = Vec::new();
    
    if building.hallway_condition < 100 {
        let repair_amount = (100 - building.hallway_condition).min(20);
        upgrades.push(UpgradeAction::RepairHallway {
            amount: repair_amount,
        });
    }
    
    upgrades
}
```

---

## Integration Points

### With GameplayState (Task 01)
```rust
// In src/state/gameplay.rs, add:
use crate::building::Building;

pub struct GameplayState {
    pub building: Building,
    pub money: i32,
    // ... other fields
}

impl GameplayState {
    pub fn new() -> Self {
        Self {
            building: Building::default_mvp(),
            money: 5000,
            // ...
        }
    }
}
```

### With Economy (Task 04)
The `UpgradeAction::cost()` method provides costs that the economy system will use.

### With Simulation (Task 05)
The `Building::apply_monthly_decay()` method will be called each tick.

---

## Acceptance Criteria

- [ ] All enums have `#[derive(Clone, Debug, Serialize, Deserialize)]`
- [ ] Apartment can be created with all 5 MVP properties
- [ ] Building generates correct number of apartments
- [ ] Quality score calculation works correctly
- [ ] Upgrade costs are calculated properly
- [ ] Decay functions work correctly
- [ ] Unit tests pass for key calculations

---

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apartment_quality_score() {
        let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        apt.condition = 50;
        apt.design = DesignType::Bare;
        assert_eq!(apt.quality_score(), 50);  // 50 condition, no bonuses
        
        apt.design = DesignType::Cozy;
        assert_eq!(apt.quality_score(), 90);  // 50 + 40 design
    }
    
    #[test]
    fn test_building_generation() {
        let building = Building::new("Test", 3, 2);
        assert_eq!(building.apartments.len(), 6);
    }
    
    #[test]
    fn test_soundproofing_effect() {
        let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::High);
        assert_eq!(apt.effective_noise(), NoiseLevel::High);
        
        apt.install_soundproofing();
        assert_eq!(apt.effective_noise(), NoiseLevel::Low);
    }
    
    #[test]
    fn test_upgrade_costs() {
        let building = Building::default_mvp();
        
        let repair = UpgradeAction::RepairApartment { apartment_id: 0, amount: 10 };
        assert_eq!(repair.cost(&building), Some(100));  // 10 * $10
        
        let design = UpgradeAction::UpgradeDesign { apartment_id: 0 };
        assert_eq!(design.cost(&building), Some(500));  // Bare -> Practical
    }
}
```

---

## Notes for Agent

- Keep all costs in the `costs` module for easy balance tuning
- The `quality_score()` method is crucial for tenant happiness calculations
- Floor 1 is ground floor (noisier)
- Unit A is always street-facing (noisier)
- Soundproofing completely negates high noise (simple for MVP)
