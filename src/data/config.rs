
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
