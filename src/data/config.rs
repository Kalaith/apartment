
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
    pub ui: UiConfig,
    #[serde(default)]
    pub upgrades: HashMap<String, UpgradeDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeDefinition {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub target: UpgradeTarget,
    pub effects: Vec<UpgradeEffect>,
    pub requirements: Vec<UpgradeRequirement>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpgradeTarget {
    Apartment,
    Building,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum UpgradeEffect {
    SetFlag(String),
    RemoveFlag(String),
    ModifyStat { stat: String, amount: i32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum UpgradeRequirement {
    MissingFlag(String),
    HasFlag(String),
    MinStat { stat: String, value: i32 },
    MaxStat { stat: String, value: i32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiConfig {
    pub upgrade_labels: HashMap<String, String>,
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
    pub kitchen_renovation_cost: i32,
    pub laundry_installation_cost: i32,
    pub soundproofing_cost: i32,
    pub base_rent: HashMap<String, i32>,
    #[serde(default)]
    pub staff_costs: HashMap<String, i32>,
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
                    m.insert("practical_to_cozy".to_string(), 1000);
                    m
                },
                kitchen_renovation_cost: 800,
                laundry_installation_cost: 2000,
                soundproofing_cost: 300,
                base_rent: {
                    let mut m = HashMap::new();
                    m.insert("small".to_string(), 600);
                    m.insert("medium".to_string(), 900);
                    m
                },
                staff_costs: {
                    let mut m = HashMap::new();
                    m.insert("janitor".to_string(), 200);
                    m.insert("security".to_string(), 400);
                    m.insert("manager".to_string(), 600);
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
            ui: UiConfig {
                upgrade_labels: {
                    let mut m = HashMap::new();
                    m.insert("repair_fmt".to_string(), "Repair +{}".to_string());
                    m.insert("repair_hallway_fmt".to_string(), "Repair Hallway +{}".to_string());
                    m.insert("upgrade_design_fmt".to_string(), "Upgrade to {}".to_string());
                    m.insert("max_design".to_string(), "Max Design".to_string());
                    m.insert("soundproofing".to_string(), "Add Soundproofing".to_string());
                    m.insert("kitchen_renovation".to_string(), "Renovate Kitchen".to_string());
                    m.insert("install_laundry".to_string(), "Install Laundry".to_string());
                    m
                }
            },
            upgrades: HashMap::new(),
        }
    }
}

pub fn load_config() -> GameConfig {
    let mut config = match std::fs::read_to_string("assets/config.json") {
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
    };
    
    // Load upgrades from separate file (allows adding upgrades without touching Rust code)
    match std::fs::read_to_string("assets/upgrades.json") {
        Ok(json) => {
            match serde_json::from_str::<HashMap<String, UpgradeDefinition>>(&json) {
                Ok(upgrades) => {
                    config.upgrades = upgrades;
                }
                Err(e) => {
                    eprintln!("Failed to parse upgrades.json: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load upgrades.json: {}", e);
        }
    }
    
    config
}
