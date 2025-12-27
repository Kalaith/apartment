use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use crate::building::Building;
use crate::city::City;
use crate::tenant::Tenant;
use crate::economy::PlayerFunds;
use crate::data::config::GameConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AchievementCondition {
    TotalTenants { min: usize },
    Funds { min: i32 },
    AvgHappiness { max: i32 }, // "below" certain value
    MaxReputation { min: i32 },
    FullOccupancy,
    GameComplete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AchievementCondition,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AchievementSystem {
    pub list: Vec<Achievement>,
    pub unlocked: HashSet<String>,
}

impl AchievementSystem {
    pub fn new() -> Self {
        Self {
            list: load_achievements_config(),
            unlocked: HashSet::new(),
        }
    }

    pub fn unlock(&mut self, id: &str) {
        self.unlocked.insert(id.to_string());
    }

    pub fn is_unlocked(&self, id: &str) -> bool {
        self.unlocked.contains(id)
    }

    pub fn check_new_unlocks(&self, 
        city: &City, 
        building: &Building, 
        tenants: &[Tenant], 
        funds: &PlayerFunds, 
        current_tick: u32,
        config: &GameConfig
    ) -> Vec<String> {
        let mut new_ids = Vec::new();

        for achievement in &self.list {
            if self.is_unlocked(&achievement.id) {
                continue;
            }

            let condition_met = match &achievement.condition {
                AchievementCondition::TotalTenants { min } => {
                    tenants.len() >= *min
                },
                AchievementCondition::Funds { min } => {
                    funds.balance >= *min
                },
                AchievementCondition::AvgHappiness { max } => {
                    // Only check if we have tenants to avoid instant slumlord on empty building
                    if tenants.is_empty() {
                        false
                    } else {
                        let avg = tenants.iter().map(|t| t.happiness).sum::<i32>() / tenants.len() as i32;
                        avg <= *max
                    }
                },
                AchievementCondition::MaxReputation { min } => {
                    // Check all neighborhoods
                    city.neighborhoods.iter().any(|n| n.reputation >= *min)
                },
                AchievementCondition::FullOccupancy => {
                    // Check if all apartments have a tenant
                    // Assuming building has apartments list
                    // And we check vacancy.
                    // Need to verify how to check occupancy from state.
                    // state.building.apartments is Vec<Apartment>.
                    // Apartment.is_vacant().
                    !building.apartments.is_empty() && building.apartments.iter().all(|a| !a.is_vacant())
                },
                AchievementCondition::GameComplete => {
                    current_tick >= config.win_conditions.game_duration_ticks.unwrap_or(36)
                }
            };

            if condition_met {
                new_ids.push(achievement.id.clone());
            }
        }
        new_ids
    }
}

fn load_achievements_config() -> Vec<Achievement> {
    let path = "assets/achievements.json";
    match fs::read_to_string(path) {
        Ok(data) => serde_json::from_str(&data).unwrap_or_else(|e| {
            println!("Failed to parse achievements.json: {}", e);
            Vec::new()
        }),
        Err(e) => {
            println!("Failed to read achievements.json: {}", e);
            Vec::new()
        }
    }
}
