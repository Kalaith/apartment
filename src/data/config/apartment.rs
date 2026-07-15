//! Tuning for per-apartment scoring: how `DesignType`, `ApartmentSize`, and
//! `NoiseLevel` feed into design appeal, starting rent, quality score, and
//! resale market value (`crate::building::apartment`).

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApartmentPropertiesConfig {
    // DesignType::appeal_score
    pub design_appeal_bare: i32,
    pub design_appeal_practical: i32,
    pub design_appeal_cozy: i32,
    pub design_appeal_luxury: i32,
    pub design_appeal_opulent: i32,

    // ApartmentSize::base_rent
    pub base_rent_small: i32,
    pub base_rent_medium: i32,
    pub base_rent_large: i32,
    pub base_rent_penthouse: i32,

    // ApartmentSize::space_score
    pub space_score_small: i32,
    pub space_score_medium: i32,
    pub space_score_large: i32,
    pub space_score_penthouse: i32,

    // NoiseLevel::noise_penalty (Low is always 0)
    pub noise_penalty_high: i32,

    // Apartment::market_value
    pub market_base_small: i32,
    pub market_base_medium: i32,
    pub market_base_large: i32,
    pub market_base_penthouse: i32,
    pub market_condition_bonus_above_50_per_point: i32,
    pub market_condition_bonus_below_50_per_point: i32,
    pub market_design_bonus_bare: i32,
    pub market_design_bonus_practical: i32,
    pub market_design_bonus_cozy: i32,
    pub market_design_bonus_luxury: i32,
    pub market_design_bonus_opulent: i32,
    pub market_kitchen_bonus_level1: i32,
    pub market_kitchen_bonus_level2_plus: i32,
    pub market_floor_bonus_per_floor: i32,
    pub market_soundproofing_bonus: i32,
    pub market_high_noise_penalty: i32,
    pub market_value_floor: i32,
}

impl Default for ApartmentPropertiesConfig {
    fn default() -> Self {
        Self {
            design_appeal_bare: 0,
            design_appeal_practical: 20,
            design_appeal_cozy: 40,
            design_appeal_luxury: 65,
            design_appeal_opulent: 90,

            base_rent_small: 600,
            base_rent_medium: 850,
            base_rent_large: 1200,
            base_rent_penthouse: 2500,

            space_score_small: 0,
            space_score_medium: 15,
            space_score_large: 30,
            space_score_penthouse: 50,

            noise_penalty_high: -20,

            market_base_small: 50_000,
            market_base_medium: 75_000,
            market_base_large: 120_000,
            market_base_penthouse: 250_000,
            market_condition_bonus_above_50_per_point: 500,
            market_condition_bonus_below_50_per_point: 300,
            market_design_bonus_bare: 0,
            market_design_bonus_practical: 5_000,
            market_design_bonus_cozy: 15_000,
            market_design_bonus_luxury: 30_000,
            market_design_bonus_opulent: 60_000,
            market_kitchen_bonus_level1: 8_000,
            market_kitchen_bonus_level2_plus: 15_000,
            market_floor_bonus_per_floor: 2_000,
            market_soundproofing_bonus: 3_000,
            market_high_noise_penalty: -5_000,
            market_value_floor: 10_000,
        }
    }
}
