# Task 07: Data Definitions

## Priority: 🟢 MEDIUM
## Dependencies: Task 01 (Core Architecture)
## Estimated Effort: 1-2 hours
## Can Parallel With: Task 02, Task 03

---

## Objective
Create JSON data files for game configuration and content. These allow balance tuning without recompilation.

---

## Deliverables

### 1. assets/config.json

Game balance and configuration constants.

```json
{
  "version": "0.1.0",
  
  "starting_conditions": {
    "player_money": 5000,
    "starting_tenants": 1,
    "building_floors": 3,
    "units_per_floor": 2
  },
  
  "economy": {
    "repair_cost_per_point": 10,
    "hallway_repair_cost_per_point": 15,
    "design_upgrade_costs": {
      "bare_to_practical": 500,
      "practical_to_cozy": 1000
    },
    "soundproofing_cost": 300,
    "base_rent": {
      "small": 600,
      "medium": 900
    }
  },
  
  "decay": {
    "apartment_per_tick": 2,
    "hallway_per_tick": 1
  },
  
  "happiness": {
    "base": 50,
    "min_for_victory": 60,
    "leave_threshold": 0,
    "unhappy_threshold": 30,
    "tenure_bonus_max": 12
  },
  
  "win_conditions": {
    "full_occupancy_required": true,
    "min_ticks_for_victory": 6
  },
  
  "applications": {
    "expire_after_ticks": 3,
    "base_per_vacancy": 0.5,
    "appeal_bonus_divisor": 50
  }
}
```

### 2. assets/building_templates.json

Pre-defined building configurations.

```json
{
  "templates": [
    {
      "id": "mvp_default",
      "name": "Sunset Apartments",
      "floors": 3,
      "units_per_floor": 2,
      "hallway_condition": 60,
      "apartments": [
        {
          "unit_number": "1A",
          "floor": 1,
          "size": "small",
          "base_noise": "high",
          "initial_condition": 50,
          "initial_design": "bare",
          "initial_rent": 600
        },
        {
          "unit_number": "1B",
          "floor": 1,
          "size": "medium",
          "base_noise": "low",
          "initial_condition": 55,
          "initial_design": "bare",
          "initial_rent": 900
        },
        {
          "unit_number": "2A",
          "floor": 2,
          "size": "small",
          "base_noise": "high",
          "initial_condition": 45,
          "initial_design": "bare",
          "initial_rent": 650
        },
        {
          "unit_number": "2B",
          "floor": 2,
          "size": "medium",
          "base_noise": "low",
          "initial_condition": 60,
          "initial_design": "practical",
          "initial_rent": 950
        },
        {
          "unit_number": "3A",
          "floor": 3,
          "size": "medium",
          "base_noise": "low",
          "initial_condition": 40,
          "initial_design": "bare",
          "initial_rent": 850
        },
        {
          "unit_number": "3B",
          "floor": 3,
          "size": "small",
          "base_noise": "low",
          "initial_condition": 70,
          "initial_design": "cozy",
          "initial_rent": 800
        }
      ],
      "initial_tenant": {
        "apartment_unit": "2B",
        "archetype": "professional",
        "name": "Sarah M."
      }
    }
  ]
}
```

### 3. assets/tenant_archetypes.json

Tenant archetype definitions and preferences.

```json
{
  "archetypes": [
    {
      "id": "student",
      "name": "Student",
      "description": "Budget-conscious, tolerates some issues",
      "spawn_weight": 40,
      
      "preferences": {
        "rent_sensitivity": 0.9,
        "condition_sensitivity": 0.3,
        "noise_sensitivity": 0.4,
        "design_sensitivity": 0.2,
        
        "ideal_rent_max": 700,
        "min_acceptable_condition": 30,
        "prefers_quiet": false,
        "preferred_design": null,
        "hates_design": null
      },
      
      "name_pool": {
        "first_names": ["Alex", "Jordan", "Casey", "Riley", "Morgan", "Sam", "Taylor", "Jamie", "Quinn", "Avery"],
        "last_initials": ["A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "R", "S", "T", "W"]
      }
    },
    {
      "id": "professional",
      "name": "Professional",
      "description": "Values quality and quiet",
      "spawn_weight": 35,
      
      "preferences": {
        "rent_sensitivity": 0.4,
        "condition_sensitivity": 0.8,
        "noise_sensitivity": 0.9,
        "design_sensitivity": 0.5,
        
        "ideal_rent_max": 1200,
        "min_acceptable_condition": 60,
        "prefers_quiet": true,
        "preferred_design": null,
        "hates_design": null
      },
      
      "name_pool": {
        "first_names": ["Michael", "Sarah", "David", "Jennifer", "Robert", "Lisa", "James", "Amanda", "William", "Elizabeth"],
        "last_initials": ["A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "R", "S", "T", "W"]
      }
    },
    {
      "id": "artist",
      "name": "Artist",
      "description": "Seeks creative, cozy spaces",
      "spawn_weight": 25,
      
      "preferences": {
        "rent_sensitivity": 0.6,
        "condition_sensitivity": 0.5,
        "noise_sensitivity": 0.5,
        "design_sensitivity": 0.95,
        
        "ideal_rent_max": 900,
        "min_acceptable_condition": 40,
        "prefers_quiet": false,
        "preferred_design": "cozy",
        "hates_design": "bare"
      },
      
      "name_pool": {
        "first_names": ["Luna", "River", "Sage", "Phoenix", "Indigo", "Willow", "Ash", "Sky", "Ocean", "Storm"],
        "last_initials": ["A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "R", "S", "T", "W"]
      }
    }
  ]
}
```

### 4. assets/text_strings.json

UI text and messages for easy localization/editing.

```json
{
  "ui": {
    "header": {
      "end_turn": "End Month",
      "money_label": "Funds",
      "month_label": "Month"
    },
    "building_view": {
      "vacant": "VACANT",
      "hallway": "HALLWAY"
    },
    "apartment_panel": {
      "condition": "CONDITION",
      "design": "Design",
      "size": "Size",
      "noise": "Noise Level",
      "rent": "Rent",
      "quality_score": "Quality Score",
      "tenant": "TENANT",
      "happiness": "Happiness",
      "months_resided": "Months",
      "upgrades": "UPGRADES"
    },
    "applications": {
      "title": "Applications",
      "no_applications": "No pending applications",
      "accept": "Accept",
      "reject": "Reject",
      "match_score": "Match"
    }
  },
  
  "design_types": {
    "bare": "Bare",
    "practical": "Practical",
    "cozy": "Cozy ★"
  },
  
  "noise_levels": {
    "low": "Quiet",
    "high": "Noisy ⚠"
  },
  
  "sizes": {
    "small": "Small",
    "medium": "Medium"
  },
  
  "events": {
    "rent_paid": "Received ${amount} rent from {tenant}",
    "rent_missed": "{tenant} missed rent payment",
    "tenant_unhappy": "{tenant} is unhappy ({happiness}%)",
    "tenant_moved_out": "{tenant} has moved out!",
    "tenant_moved_in": "{tenant} moved into Unit {unit}",
    "new_application": "{tenant} ({archetype}) applied for Unit {unit}",
    "noise_complaint": "Noise complaint from {tenant}",
    "condition_complaint": "{tenant} complained about Unit {unit} condition",
    "poor_condition": "Unit {unit} in poor condition ({condition}%)",
    "critical_condition": "⚠️ Unit {unit} CRITICAL ({condition}%)",
    "hallway_deteriorating": "Hallway deteriorating ({condition}%)",
    "month_end": "Month {tick} ended: +${income} -${expenses} = ${balance}",
    "victory": "🎉 Victory!",
    "bankruptcy": "💸 Bankrupt!",
    "all_left": "🚪 All tenants left!"
  },
  
  "upgrade_buttons": {
    "repair": "Repair +{amount} (${cost})",
    "design_upgrade": "Upgrade to {design} (${cost})",
    "soundproofing": "Add Soundproofing (${cost})",
    "hallway_repair": "Repair Hallway +{amount} (${cost})"
  },
  
  "tooltips": {
    "condition": "Physical state of the apartment. Decays monthly.",
    "design": "Interior quality. Affects tenant preferences.",
    "noise": "Ambient noise level. Can be mitigated with soundproofing.",
    "quality_score": "Combined score of all factors. Higher attracts better tenants.",
    "happiness": "Tenant satisfaction. If this reaches 0, they leave.",
    "hallway": "Shared space condition. Affects all tenant happiness."
  }
}
```

### 5. src/data/mod.rs

Data loading module.

```rust
mod config;
mod building_templates;
mod tenant_archetypes;

pub use config::{GameConfig, load_config};
pub use building_templates::{BuildingTemplate, ApartmentTemplate, load_building_templates};
pub use tenant_archetypes::{ArchetypeData, load_archetypes};
```

### 6. src/data/config.rs

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub version: String,
    pub starting_conditions: StartingConditions,
    pub economy: EconomyConfig,
    pub decay: DecayConfig,
    pub happiness: HappinessConfig,
    pub win_conditions: WinConditions,
    pub applications: ApplicationConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartingConditions {
    pub player_money: i32,
    pub starting_tenants: i32,
    pub building_floors: u32,
    pub units_per_floor: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub repair_cost_per_point: i32,
    pub hallway_repair_cost_per_point: i32,
    pub design_upgrade_costs: HashMap<String, i32>,
    pub soundproofing_cost: i32,
    pub base_rent: HashMap<String, i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecayConfig {
    pub apartment_per_tick: i32,
    pub hallway_per_tick: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HappinessConfig {
    pub base: i32,
    pub min_for_victory: i32,
    pub leave_threshold: i32,
    pub unhappy_threshold: i32,
    pub tenure_bonus_max: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinConditions {
    pub full_occupancy_required: bool,
    pub min_ticks_for_victory: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub expire_after_ticks: u32,
    pub base_per_vacancy: f32,
    pub appeal_bonus_divisor: i32,
}

impl Default for GameConfig {
    fn default() -> Self {
        // Hardcoded fallback if JSON fails to load
        Self {
            version: "0.1.0".to_string(),
            starting_conditions: StartingConditions {
                player_money: 5000,
                starting_tenants: 1,
                building_floors: 3,
                units_per_floor: 2,
            },
            economy: EconomyConfig {
                repair_cost_per_point: 10,
                hallway_repair_cost_per_point: 15,
                design_upgrade_costs: {
                    let mut m = HashMap::new();
                    m.insert("bare_to_practical".to_string(), 500);
                    m.insert("practical_to_cozy".to_string(), 1000);
                    m
                },
                soundproofing_cost: 300,
                base_rent: {
                    let mut m = HashMap::new();
                    m.insert("small".to_string(), 600);
                    m.insert("medium".to_string(), 900);
                    m
                },
            },
            decay: DecayConfig {
                apartment_per_tick: 2,
                hallway_per_tick: 1,
            },
            happiness: HappinessConfig {
                base: 50,
                min_for_victory: 60,
                leave_threshold: 0,
                unhappy_threshold: 30,
                tenure_bonus_max: 12,
            },
            win_conditions: WinConditions {
                full_occupancy_required: true,
                min_ticks_for_victory: 6,
            },
            applications: ApplicationConfig {
                expire_after_ticks: 3,
                base_per_vacancy: 0.5,
                appeal_bonus_divisor: 50,
            },
        }
    }
}

pub fn load_config() -> GameConfig {
    match std::fs::read_to_string("assets/config.json") {
        Ok(json) => {
            serde_json::from_str(&json).unwrap_or_else(|e| {
                eprintln!("Failed to parse config.json: {}", e);
                GameConfig::default()
            })
        }
        Err(e) => {
            eprintln!("Failed to load config.json: {}", e);
            GameConfig::default()
        }
    }
}
```

### 7. src/data/building_templates.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTemplateFile {
    pub templates: Vec<BuildingTemplate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTemplate {
    pub id: String,
    pub name: String,
    pub floors: u32,
    pub units_per_floor: u32,
    pub hallway_condition: i32,
    pub apartments: Vec<ApartmentTemplate>,
    pub initial_tenant: Option<InitialTenant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApartmentTemplate {
    pub unit_number: String,
    pub floor: u32,
    pub size: String,         // "small" or "medium"
    pub base_noise: String,   // "low" or "high"
    pub initial_condition: i32,
    pub initial_design: String,  // "bare", "practical", "cozy"
    pub initial_rent: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialTenant {
    pub apartment_unit: String,
    pub archetype: String,
    pub name: String,
}

pub fn load_building_templates() -> Vec<BuildingTemplate> {
    match std::fs::read_to_string("assets/building_templates.json") {
        Ok(json) => {
            match serde_json::from_str::<BuildingTemplateFile>(&json) {
                Ok(file) => file.templates,
                Err(e) => {
                    eprintln!("Failed to parse building_templates.json: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load building_templates.json: {}", e);
            Vec::new()
        }
    }
}

pub fn get_template(id: &str) -> Option<BuildingTemplate> {
    load_building_templates().into_iter().find(|t| t.id == id)
}
```

### 8. src/data/tenant_archetypes.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeFile {
    pub archetypes: Vec<ArchetypeData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub spawn_weight: i32,
    pub preferences: PreferencesData,
    pub name_pool: NamePool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreferencesData {
    pub rent_sensitivity: f32,
    pub condition_sensitivity: f32,
    pub noise_sensitivity: f32,
    pub design_sensitivity: f32,
    pub ideal_rent_max: i32,
    pub min_acceptable_condition: i32,
    pub prefers_quiet: bool,
    pub preferred_design: Option<String>,
    pub hates_design: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamePool {
    pub first_names: Vec<String>,
    pub last_initials: Vec<String>,
}

pub fn load_archetypes() -> Vec<ArchetypeData> {
    match std::fs::read_to_string("assets/tenant_archetypes.json") {
        Ok(json) => {
            match serde_json::from_str::<ArchetypeFile>(&json) {
                Ok(file) => file.archetypes,
                Err(e) => {
                    eprintln!("Failed to parse tenant_archetypes.json: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load tenant_archetypes.json: {}", e);
            Vec::new()
        }
    }
}

pub fn get_archetype(id: &str) -> Option<ArchetypeData> {
    load_archetypes().into_iter().find(|a| a.id == id)
}
```

---

## Folder Structure

```
assets/
├── config.json              # Game balance constants
├── building_templates.json  # Pre-defined building layouts
├── tenant_archetypes.json   # Tenant type definitions  
└── text_strings.json        # UI text for localization
```

---

## Usage Example

```rust
// In game initialization:
use crate::data::{load_config, get_template};

let config = load_config();

// Use config values
let starting_money = config.starting_conditions.player_money;
let decay_rate = config.decay.apartment_per_tick;

// Load building from template
if let Some(template) = get_template("mvp_default") {
    let building = Building::from_template(&template);
}
```

---

## Acceptance Criteria

- [ ] All JSON files are valid and parseable
- [ ] Config values are used throughout the game
- [ ] Building can be loaded from template
- [ ] Tenant archetypes loaded from JSON
- [ ] Fallback defaults if files missing
- [ ] No hardcoded values in game logic (use config)

---

## Balance Tuning Guide

| Parameter | Effect | Suggested Range |
|-----------|--------|-----------------|
| `player_money` | Starting difficulty | 3000-7000 |
| `repair_cost_per_point` | Maintenance pressure | 5-20 |
| `apartment_per_tick` | Decay speed | 1-5 |
| `min_for_victory` | Win difficulty | 50-80 |
| `spawn_weight` | Tenant type frequency | Adjust ratios |

---

## Notes for Agent

- JSON must be valid - test with online validators
- Provide sensible defaults in code for resilience
- Text strings enable future localization
- Keep all magic numbers in config.json
- Building templates allow varied starting scenarios
