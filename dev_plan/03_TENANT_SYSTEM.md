# Task 03: Tenant System

## Priority: 🟡 HIGH
## Dependencies: Task 01 (Core Architecture)
## Estimated Effort: 3-4 hours
## Can Parallel With: Task 02, Task 07

---

## Objective
Implement the tenant archetype system, happiness mechanics, and apartment matching logic that determines who wants to live where.

---

## Deliverables

### 1. src/tenant/mod.rs

```rust
mod tenant;
mod archetype;
mod happiness;
mod matching;
mod application;

pub use tenant::Tenant;
pub use archetype::{TenantArchetype, ArchetypePreferences};
pub use happiness::{calculate_happiness, HappinessFactors};
pub use matching::{calculate_match_score, MatchResult};
pub use application::{TenantApplication, generate_applications};
```

### 2. src/tenant/archetype.rs

**Three MVP Archetypes:**

| Tenant | Cares About | Hates |
|--------|-------------|-------|
| Student | Low rent | Bad condition |
| Professional | Condition, quiet | Noise |
| Artist | Cozy design | Sterile/Bare spaces |

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TenantArchetype {
    Student,
    Professional,
    Artist,
}

impl TenantArchetype {
    pub fn name(&self) -> &'static str {
        match self {
            TenantArchetype::Student => "Student",
            TenantArchetype::Professional => "Professional",
            TenantArchetype::Artist => "Artist",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            TenantArchetype::Student => "Budget-conscious, tolerates some issues",
            TenantArchetype::Professional => "Values quality and quiet",
            TenantArchetype::Artist => "Seeks creative, cozy spaces",
        }
    }
    
    /// Get the preferences for this archetype
    pub fn preferences(&self) -> ArchetypePreferences {
        match self {
            TenantArchetype::Student => ArchetypePreferences {
                rent_sensitivity: 0.9,      // Very price sensitive
                condition_sensitivity: 0.3, // Low - tolerates some wear
                noise_sensitivity: 0.4,     // Low - can deal with noise
                design_sensitivity: 0.2,    // Doesn't care much
                
                ideal_rent_max: 700,
                min_acceptable_condition: 30,
                prefers_quiet: false,
                preferred_design: None,
                hates_design: None,
            },
            TenantArchetype::Professional => ArchetypePreferences {
                rent_sensitivity: 0.4,      // Can afford more
                condition_sensitivity: 0.8, // Values good condition
                noise_sensitivity: 0.9,     // Hates noise
                design_sensitivity: 0.5,    // Moderate
                
                ideal_rent_max: 1200,
                min_acceptable_condition: 60,
                prefers_quiet: true,
                preferred_design: None,
                hates_design: None,
            },
            TenantArchetype::Artist => ArchetypePreferences {
                rent_sensitivity: 0.6,      // Moderate budget
                condition_sensitivity: 0.5, // Moderate
                noise_sensitivity: 0.5,     // Moderate
                design_sensitivity: 0.95,   // Very design focused
                
                ideal_rent_max: 900,
                min_acceptable_condition: 40,
                prefers_quiet: false,
                preferred_design: Some(crate::building::DesignType::Cozy),
                hates_design: Some(crate::building::DesignType::Bare),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArchetypePreferences {
    // Sensitivity weights (0.0 - 1.0, higher = more affected)
    pub rent_sensitivity: f32,
    pub condition_sensitivity: f32,
    pub noise_sensitivity: f32,
    pub design_sensitivity: f32,
    
    // Thresholds
    pub ideal_rent_max: i32,
    pub min_acceptable_condition: i32,
    pub prefers_quiet: bool,
    
    // Design preferences
    pub preferred_design: Option<crate::building::DesignType>,
    pub hates_design: Option<crate::building::DesignType>,
}
```

### 3. src/tenant/tenant.rs

```rust
use serde::{Deserialize, Serialize};
use super::TenantArchetype;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: u32,
    pub name: String,
    pub archetype: TenantArchetype,
    
    // Current state
    pub happiness: i32,         // 0-100
    pub months_residing: u32,   // How long they've lived here
    pub apartment_id: Option<u32>,
    
    // Tolerances (derived from archetype but can vary slightly)
    pub rent_tolerance: i32,    // Max rent they'll accept
    pub noise_tolerance: i32,   // 0-100, higher = more tolerant
}

impl Tenant {
    pub fn new(id: u32, name: &str, archetype: TenantArchetype) -> Self {
        let prefs = archetype.preferences();
        
        Self {
            id,
            name: name.to_string(),
            archetype,
            happiness: 70,  // Start reasonably happy
            months_residing: 0,
            apartment_id: None,
            rent_tolerance: prefs.ideal_rent_max,
            noise_tolerance: if prefs.prefers_quiet { 30 } else { 70 },
        }
    }
    
    /// Create a tenant with some randomization
    pub fn generate(id: u32, archetype: TenantArchetype) -> Self {
        let name = generate_random_name(&archetype);
        let mut tenant = Self::new(id, &name, archetype);
        
        // Add some variance to tolerances (±15%)
        let variance = 0.15;
        let rent_var = (tenant.rent_tolerance as f32 * variance) as i32;
        tenant.rent_tolerance += rand::gen_range(-rent_var, rent_var);
        
        let noise_var = (tenant.noise_tolerance as f32 * variance) as i32;
        tenant.noise_tolerance = (tenant.noise_tolerance + rand::gen_range(-noise_var, noise_var)).clamp(0, 100);
        
        tenant
    }
    
    /// Check if tenant is at risk of leaving
    pub fn is_unhappy(&self) -> bool {
        self.happiness < 30
    }
    
    /// Check if tenant will leave this tick
    pub fn will_leave(&self) -> bool {
        self.happiness == 0
    }
    
    /// Update happiness (called each tick)
    pub fn set_happiness(&mut self, new_happiness: i32) {
        self.happiness = new_happiness.clamp(0, 100);
    }
    
    /// Increment months residing
    pub fn add_month(&mut self) {
        self.months_residing += 1;
    }
    
    /// Move into an apartment
    pub fn move_into(&mut self, apartment_id: u32) {
        self.apartment_id = Some(apartment_id);
        self.months_residing = 0;
    }
    
    /// Move out of current apartment
    pub fn move_out(&mut self) {
        self.apartment_id = None;
    }
}

/// Generate a random name appropriate for the archetype
fn generate_random_name(archetype: &TenantArchetype) -> String {
    use rand::ChooseRandom;
    
    let first_names = match archetype {
        TenantArchetype::Student => vec![
            "Alex", "Jordan", "Casey", "Riley", "Morgan",
            "Sam", "Taylor", "Jamie", "Quinn", "Avery"
        ],
        TenantArchetype::Professional => vec![
            "Michael", "Sarah", "David", "Jennifer", "Robert",
            "Lisa", "James", "Amanda", "William", "Elizabeth"
        ],
        TenantArchetype::Artist => vec![
            "Luna", "River", "Sage", "Phoenix", "Indigo",
            "Willow", "Ash", "Sky", "Ocean", "Storm"
        ],
    };
    
    let last_initials = vec!["A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "R", "S", "T", "W"];
    
    let first = first_names.choose().unwrap_or(&"Pat");
    let last = last_initials.choose().unwrap_or(&"X");
    
    format!("{} {}.", first, last)
}
```

### 4. src/tenant/happiness.rs

```rust
use crate::building::{Apartment, Building, DesignType, NoiseLevel};
use super::{Tenant, ArchetypePreferences};

/// All factors that influence happiness
#[derive(Clone, Debug)]
pub struct HappinessFactors {
    pub base_happiness: i32,
    pub rent_factor: i32,       // Negative if too expensive
    pub condition_factor: i32,  // Based on apartment condition
    pub noise_factor: i32,      // Negative if too noisy
    pub design_factor: i32,     // Based on design preference
    pub hallway_factor: i32,    // Building shared space condition
    pub tenure_bonus: i32,      // Small bonus for long-term residents
}

impl HappinessFactors {
    pub fn total(&self) -> i32 {
        (self.base_happiness 
            + self.rent_factor 
            + self.condition_factor 
            + self.noise_factor 
            + self.design_factor
            + self.hallway_factor
            + self.tenure_bonus)
            .clamp(0, 100)
    }
}

/// Calculate happiness factors for a tenant in their apartment
pub fn calculate_happiness(
    tenant: &Tenant, 
    apartment: &Apartment,
    building: &Building,
) -> HappinessFactors {
    let prefs = tenant.archetype.preferences();
    
    HappinessFactors {
        base_happiness: 50,
        rent_factor: calculate_rent_factor(apartment.rent_price, &prefs),
        condition_factor: calculate_condition_factor(apartment.condition, &prefs),
        noise_factor: calculate_noise_factor(&apartment.effective_noise(), tenant.noise_tolerance, &prefs),
        design_factor: calculate_design_factor(&apartment.design, &prefs),
        hallway_factor: calculate_hallway_factor(building.hallway_condition),
        tenure_bonus: calculate_tenure_bonus(tenant.months_residing),
    }
}

fn calculate_rent_factor(rent: i32, prefs: &ArchetypePreferences) -> i32 {
    let diff = prefs.ideal_rent_max - rent;
    let sensitivity = prefs.rent_sensitivity;
    
    if diff >= 0 {
        // Under budget - small bonus
        ((diff as f32 * 0.02 * sensitivity) as i32).min(15)
    } else {
        // Over budget - penalty
        ((diff as f32 * 0.05 * sensitivity) as i32).max(-30)
    }
}

fn calculate_condition_factor(condition: i32, prefs: &ArchetypePreferences) -> i32 {
    let sensitivity = prefs.condition_sensitivity;
    let min_acceptable = prefs.min_acceptable_condition;
    
    if condition >= min_acceptable {
        // Good condition - bonus based on how much above minimum
        let excess = condition - min_acceptable;
        ((excess as f32 * 0.3 * sensitivity) as i32).min(20)
    } else {
        // Below minimum - significant penalty
        let deficit = min_acceptable - condition;
        -((deficit as f32 * 0.5 * sensitivity) as i32).min(40)
    }
}

fn calculate_noise_factor(noise: &NoiseLevel, tolerance: i32, prefs: &ArchetypePreferences) -> i32 {
    let sensitivity = prefs.noise_sensitivity;
    
    match noise {
        NoiseLevel::Low => {
            // Quiet - small bonus for those who prefer it
            if prefs.prefers_quiet {
                (10.0 * sensitivity) as i32
            } else {
                0
            }
        }
        NoiseLevel::High => {
            // Noisy - penalty based on sensitivity and tolerance
            let base_penalty = -25;
            let tolerance_mod = (tolerance as f32 * 0.3) as i32;
            ((base_penalty + tolerance_mod) as f32 * sensitivity) as i32
        }
    }
}

fn calculate_design_factor(design: &DesignType, prefs: &ArchetypePreferences) -> i32 {
    let sensitivity = prefs.design_sensitivity;
    let mut factor = 0;
    
    // Check if this is their preferred design
    if let Some(ref preferred) = prefs.preferred_design {
        if design == preferred {
            factor += 20;
        }
    }
    
    // Check if this is their hated design
    if let Some(ref hated) = prefs.hates_design {
        if design == hated {
            factor -= 25;
        }
    }
    
    // General design quality bonus
    factor += match design {
        DesignType::Bare => -5,
        DesignType::Practical => 5,
        DesignType::Cozy => 10,
    };
    
    (factor as f32 * sensitivity) as i32
}

fn calculate_hallway_factor(hallway_condition: i32) -> i32 {
    // Shared space affects everyone equally but mildly
    ((hallway_condition - 50) as f32 * 0.1) as i32  // -5 to +5
}

fn calculate_tenure_bonus(months: u32) -> i32 {
    // Long-term residents get a small stability bonus
    (months as i32).min(12)  // Max +12 after a year
}

/// Check if apartment meets minimum requirements for tenant
pub fn apartment_meets_minimum(tenant: &Tenant, apartment: &Apartment) -> bool {
    let prefs = tenant.archetype.preferences();
    
    // Check condition minimum
    if apartment.condition < prefs.min_acceptable_condition {
        return false;
    }
    
    // Check rent tolerance
    if apartment.rent_price > tenant.rent_tolerance {
        return false;
    }
    
    // Check design compatibility (artists won't accept bare)
    if let Some(ref hated) = prefs.hates_design {
        if &apartment.design == hated {
            return false;
        }
    }
    
    // Check noise for noise-sensitive tenants
    if prefs.prefers_quiet {
        if matches!(apartment.effective_noise(), NoiseLevel::High) 
           && tenant.noise_tolerance < 40 
        {
            return false;
        }
    }
    
    true
}
```

### 5. src/tenant/matching.rs

```rust
use crate::building::Apartment;
use super::{Tenant, TenantArchetype, happiness};

/// Result of matching a tenant to an apartment
#[derive(Clone, Debug)]
pub struct MatchResult {
    pub score: i32,              // 0-100, higher = better match
    pub meets_minimum: bool,     // Would tenant even consider this?
    pub reasons: Vec<String>,    // Why this score
}

/// Calculate how well a tenant matches an apartment
pub fn calculate_match_score(tenant: &Tenant, apartment: &Apartment) -> MatchResult {
    let mut score = 50;  // Start at neutral
    let mut reasons = Vec::new();
    
    let prefs = tenant.archetype.preferences();
    
    // Check minimum requirements
    let meets_minimum = happiness::apartment_meets_minimum(tenant, apartment);
    if !meets_minimum {
        return MatchResult {
            score: 0,
            meets_minimum: false,
            reasons: vec!["Does not meet minimum requirements".to_string()],
        };
    }
    
    // Rent scoring
    let rent_diff = prefs.ideal_rent_max - apartment.rent_price;
    if rent_diff > 200 {
        score += 15;
        reasons.push("Great price".to_string());
    } else if rent_diff > 0 {
        score += 8;
        reasons.push("Fair price".to_string());
    } else if rent_diff > -100 {
        score -= 5;
        reasons.push("Slightly expensive".to_string());
    } else {
        score -= 15;
        reasons.push("Expensive".to_string());
    }
    
    // Condition scoring
    if apartment.condition >= 80 {
        let bonus = (15.0 * prefs.condition_sensitivity) as i32;
        score += bonus;
        reasons.push("Excellent condition".to_string());
    } else if apartment.condition >= 60 {
        let bonus = (8.0 * prefs.condition_sensitivity) as i32;
        score += bonus;
        reasons.push("Good condition".to_string());
    } else if apartment.condition < 50 {
        let penalty = (10.0 * prefs.condition_sensitivity) as i32;
        score -= penalty;
        reasons.push("Poor condition".to_string());
    }
    
    // Noise scoring
    match apartment.effective_noise() {
        crate::building::NoiseLevel::Low => {
            if prefs.prefers_quiet {
                let bonus = (12.0 * prefs.noise_sensitivity) as i32;
                score += bonus;
                reasons.push("Nice and quiet".to_string());
            }
        }
        crate::building::NoiseLevel::High => {
            let penalty = (15.0 * prefs.noise_sensitivity) as i32;
            score -= penalty;
            reasons.push("Too noisy".to_string());
        }
    }
    
    // Design scoring
    if let Some(ref preferred) = prefs.preferred_design {
        if &apartment.design == preferred {
            let bonus = (18.0 * prefs.design_sensitivity) as i32;
            score += bonus;
            reasons.push(format!("Loves the {:?} style", apartment.design));
        }
    }
    
    // Size bonus (everyone likes more space)
    match apartment.size {
        crate::building::ApartmentSize::Medium => {
            score += 5;
            reasons.push("Good space".to_string());
        }
        crate::building::ApartmentSize::Small => {}
    }
    
    MatchResult {
        score: score.clamp(0, 100),
        meets_minimum: true,
        reasons,
    }
}

/// Find the best apartment match for a tenant from available options
pub fn find_best_match<'a>(
    tenant: &Tenant, 
    apartments: &'a [&'a Apartment]
) -> Option<(&'a Apartment, MatchResult)> {
    apartments
        .iter()
        .filter(|apt| apt.is_vacant())
        .map(|apt| (*apt, calculate_match_score(tenant, apt)))
        .filter(|(_, result)| result.meets_minimum)
        .max_by_key(|(_, result)| result.score)
}

/// Get all apartments a tenant would consider (meets minimum)
pub fn get_acceptable_apartments<'a>(
    tenant: &Tenant,
    apartments: &'a [&'a Apartment]
) -> Vec<(&'a Apartment, MatchResult)> {
    apartments
        .iter()
        .filter(|apt| apt.is_vacant())
        .map(|apt| (*apt, calculate_match_score(tenant, apt)))
        .filter(|(_, result)| result.meets_minimum)
        .collect()
}
```

### 6. src/tenant/application.rs

```rust
use rand::ChooseRandom;
use super::{Tenant, TenantArchetype, matching::MatchResult};
use crate::building::Building;

/// A tenant application for a specific apartment
#[derive(Clone, Debug)]
pub struct TenantApplication {
    pub tenant: Tenant,
    pub apartment_id: u32,
    pub match_result: MatchResult,
    pub tick_created: u32,  // When this application was generated
}

impl TenantApplication {
    pub fn new(tenant: Tenant, apartment_id: u32, match_result: MatchResult, tick: u32) -> Self {
        Self {
            tenant,
            apartment_id,
            match_result,
            tick_created: tick,
        }
    }
    
    /// Applications expire after a few ticks
    pub fn is_expired(&self, current_tick: u32) -> bool {
        current_tick > self.tick_created + 3  // Expire after 3 months
    }
}

/// Generate new tenant applications based on building state
pub fn generate_applications(
    building: &Building,
    existing_applications: &[TenantApplication],
    current_tick: u32,
    next_tenant_id: &mut u32,
) -> Vec<TenantApplication> {
    let mut new_applications = Vec::new();
    
    let vacant = building.vacant_apartments();
    if vacant.is_empty() {
        return new_applications;
    }
    
    let building_appeal = building.building_appeal();
    
    // Number of applications based on building appeal and vacancies
    let base_apps = (vacant.len() as f32 * 0.5).ceil() as usize;
    let appeal_bonus = (building_appeal as f32 / 50.0) as usize;
    let num_applications = (base_apps + appeal_bonus).min(vacant.len()).max(1);
    
    for _ in 0..num_applications {
        // Pick a random archetype (weighted)
        let archetype = pick_random_archetype();
        
        // Generate a tenant
        let tenant = Tenant::generate(*next_tenant_id, archetype);
        *next_tenant_id += 1;
        
        // Find an apartment they'd apply to
        let apartment_refs: Vec<&_> = vacant.iter().map(|a| *a).collect();
        
        if let Some((apt, match_result)) = super::matching::find_best_match(&tenant, &apartment_refs) {
            // Check if there's already an application for this apartment from this archetype
            let already_applied = existing_applications.iter().any(|app| {
                app.apartment_id == apt.id && app.tenant.archetype == tenant.archetype
            }) || new_applications.iter().any(|app: &TenantApplication| {
                app.apartment_id == apt.id && app.tenant.archetype == tenant.archetype
            });
            
            if !already_applied {
                new_applications.push(TenantApplication::new(
                    tenant,
                    apt.id,
                    match_result,
                    current_tick,
                ));
            }
        }
    }
    
    new_applications
}

fn pick_random_archetype() -> TenantArchetype {
    let roll = rand::gen_range(0, 100);
    
    // Weighted distribution
    if roll < 40 {
        TenantArchetype::Student      // 40% - most common
    } else if roll < 75 {
        TenantArchetype::Professional // 35% - common
    } else {
        TenantArchetype::Artist       // 25% - less common
    }
}

/// Process tenant decisions to leave
pub fn process_departures(tenants: &mut Vec<Tenant>, building: &mut Building) -> Vec<String> {
    let mut notifications = Vec::new();
    let mut departing_ids = Vec::new();
    
    for tenant in tenants.iter() {
        if tenant.will_leave() {
            notifications.push(format!("{} has moved out!", tenant.name));
            departing_ids.push(tenant.id);
            
            // Clear apartment
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apt) = building.get_apartment_mut(apt_id) {
                    apt.move_out();
                }
            }
        }
    }
    
    tenants.retain(|t| !departing_ids.contains(&t.id));
    notifications
}
```

---

## Integration Points

### With Building System (Task 02)
- Uses `Apartment`, `DesignType`, `NoiseLevel` for matching
- Calls `apartment.effective_noise()` for noise calculations

### With GameplayState (Task 01)
```rust
// In src/state/gameplay.rs, add:
use crate::tenant::{Tenant, TenantApplication};

pub struct GameplayState {
    pub building: Building,
    pub tenants: Vec<Tenant>,
    pub applications: Vec<TenantApplication>,
    pub next_tenant_id: u32,
    // ...
}
```

### With Simulation (Task 05)
- `calculate_happiness()` called each tick
- `generate_applications()` called each tick
- `process_departures()` called each tick

---

## Acceptance Criteria

- [ ] All three archetypes implemented with distinct preferences
- [ ] Happiness calculation considers all factors
- [ ] Match scoring works for tenant-apartment pairing
- [ ] Applications generated based on building appeal
- [ ] Tenants leave when happiness hits 0
- [ ] Unit tests pass for happiness calculations

---

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::*;
    
    #[test]
    fn test_student_prefers_cheap() {
        let student = Tenant::new(0, "Test", TenantArchetype::Student);
        
        let mut cheap = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        cheap.rent_price = 500;
        
        let mut expensive = Apartment::new(1, "1B", 1, ApartmentSize::Small, NoiseLevel::Low);
        expensive.rent_price = 1000;
        
        let cheap_match = matching::calculate_match_score(&student, &cheap);
        let expensive_match = matching::calculate_match_score(&student, &expensive);
        
        assert!(cheap_match.score > expensive_match.score);
    }
    
    #[test]
    fn test_professional_hates_noise() {
        let pro = Tenant::new(0, "Test", TenantArchetype::Professional);
        
        let quiet = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        let noisy = Apartment::new(1, "1B", 1, ApartmentSize::Small, NoiseLevel::High);
        
        let quiet_match = matching::calculate_match_score(&pro, &quiet);
        let noisy_match = matching::calculate_match_score(&pro, &noisy);
        
        assert!(quiet_match.score > noisy_match.score);
    }
    
    #[test]
    fn test_artist_loves_cozy() {
        let artist = Tenant::new(0, "Test", TenantArchetype::Artist);
        
        let mut cozy = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        cozy.design = DesignType::Cozy;
        
        let bare = Apartment::new(1, "1B", 1, ApartmentSize::Small, NoiseLevel::Low);
        // bare.design is already Bare
        
        let cozy_match = matching::calculate_match_score(&artist, &cozy);
        let bare_match = matching::calculate_match_score(&artist, &bare);
        
        assert!(cozy_match.score > bare_match.score);
        assert!(!bare_match.meets_minimum);  // Artists reject bare apartments
    }
}
```

---

## Notes for Agent

- Happiness recalculates every tick based on current apartment state
- A tenant's `noise_tolerance` varies slightly from archetype default
- Applications expire after 3 ticks if not accepted
- Artists will **not** consider bare apartments (fails minimum check)
- Professionals strongly prefer quiet apartments
- Students are the most flexible archetype
