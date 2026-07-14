//! Core simulation tuning: how a game starts, what it costs to run, how
//! condition decays, how happy tenants are, and when the game ends.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

fn default_leave_chance_percent() -> i32 {
    35
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HappinessConfig {
    pub base: i32,
    pub min_for_victory: i32,
    /// Happiness at/below which a tenant may decide to move out.
    pub leave_threshold: i32,
    /// Monthly chance (percent) that a tenant at/below `leave_threshold` actually leaves.
    #[serde(default = "default_leave_chance_percent")]
    pub leave_chance_percent: i32,
    pub unhappy_threshold: i32,
    pub tenure_bonus_max: i32,

    // Rent
    pub rent_bonus_multiplier: f32,
    pub rent_bonus_cap: i32,
    pub rent_penalty_multiplier: f32,
    pub rent_penalty_cap: i32,

    // Condition
    pub condition_bonus_multiplier: f32,
    pub condition_bonus_cap: i32,
    pub condition_penalty_multiplier: f32,
    pub condition_penalty_cap: i32,

    // Noise
    pub noise_quiet_bonus: f32,
    pub noise_high_penalty_base: i32,
    pub noise_tolerance_multiplier: f32,

    // Design
    pub design_preferred_bonus: i32,
    pub design_hated_penalty: i32,
    pub design_style_modifiers: HashMap<String, i32>,

    // Hallway
    pub hallway_condition_base: i32,
    pub hallway_condition_multiplier: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinConditions {
    pub full_occupancy_required: bool,
    pub min_ticks_for_victory: u32,
    /// Game duration in ticks (months). After this many ticks, the game ends with a score.
    /// Defaults to 36 (3 years) if not specified.
    #[serde(default)]
    pub game_duration_ticks: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub expire_after_ticks: u32,
    pub base_per_vacancy: f32,
    pub appeal_bonus_divisor: i32,
    /// How strongly neighborhood reputation swings applicant volume. At the
    /// neutral reputation (50) the multiplier is 1.0; at reputation 0 it is
    /// `1 - influence`, and at 100 it is `1 + influence`.
    #[serde(default = "default_reputation_influence")]
    pub reputation_influence: f32,
}

fn default_reputation_influence() -> f32 {
    0.5
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    pub poor_condition: i32,
    pub critical_condition: i32,
    pub all_left_check_tick: u32,
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            poor_condition: 40,
            critical_condition: 20,
            all_left_check_tick: 3,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatingCostsConfig {
    pub property_tax_rate: f32,
    /// Additional property tax rate added per year of ownership (reassessment).
    #[serde(default)]
    pub property_tax_annual_increase: f32,
    /// Fixed monthly overhead charged per unit regardless of occupancy (mortgage/upkeep).
    #[serde(default)]
    pub base_monthly_cost_per_unit: i32,
    pub utility_cost_per_unit: i32,
    pub insurance_base_rate: i32,
    pub insurance_good_condition_discount: i32,
    pub insurance_good_condition_threshold: i32,
}

impl Default for OperatingCostsConfig {
    fn default() -> Self {
        Self {
            property_tax_rate: 0.10,
            property_tax_annual_increase: 0.035,
            base_monthly_cost_per_unit: 140,
            utility_cost_per_unit: 50,
            insurance_base_rate: 150,
            insurance_good_condition_discount: 50,
            insurance_good_condition_threshold: 80,
        }
    }
}
