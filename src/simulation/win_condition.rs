#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use crate::building::Building;
use crate::tenant::Tenant;
use crate::economy::PlayerFunds;

/// Game outcome
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameOutcome {
    Victory {
        score: i32,
        months: u32,
        total_income: i32,
    },
    Bankruptcy {
        debt: i32,
    },
    AllTenantsLeft,
}

/// Win condition thresholds
pub mod thresholds {
    /// Minimum average happiness for victory
    pub const MIN_HAPPINESS: i32 = 60;
    
    /// All units must be occupied for victory
    pub const FULL_OCCUPANCY_REQUIRED: bool = true;
    
    /// Minimum ticks before victory can trigger (prevent instant win)
    pub const MIN_TICKS_FOR_VICTORY: u32 = 6;
}

/// Check current game state for win/lose conditions
pub fn check_win_condition(
    building: &Building,
    tenants: &[Tenant],
    funds: &PlayerFunds,
    current_tick: u32,
) -> Option<GameOutcome> {
    // Check for bankruptcy
    if funds.is_bankrupt() {
        return Some(GameOutcome::Bankruptcy { debt: funds.balance.abs() });
    }
    
    // Check if all tenants left (after having some)
    if tenants.is_empty() && current_tick > 3 {
        // Check if we ever had tenants (building was lived in)
        let was_occupied = building.apartments.iter().any(|a| a.tenant_id.is_some());
        if !was_occupied {
            return Some(GameOutcome::AllTenantsLeft);
        }
    }
    
    // Check for victory conditions
    if current_tick < thresholds::MIN_TICKS_FOR_VICTORY {
        return None;  // Too early for victory
    }
    
    // All units must be occupied
    if thresholds::FULL_OCCUPANCY_REQUIRED {
        if building.vacancy_count() > 0 {
            return None;
        }
    }
    
    // Calculate average happiness
    if tenants.is_empty() {
        return None;
    }
    
    let avg_happiness: i32 = tenants.iter()
        .map(|t| t.happiness)
        .sum::<i32>() / tenants.len() as i32;
    
    if avg_happiness >= thresholds::MIN_HAPPINESS {
        return Some(GameOutcome::Victory {
            total_income: funds.total_income,
            months: current_tick,
            score: ((avg_happiness as f32) * 10.0 + (funds.total_income as f32 / 100.0)) as i32,
        });
    }
    
    None
}

/// Get progress towards victory
#[derive(Clone, Debug)]
pub struct VictoryProgress {
    pub occupancy_percent: f32,
    pub avg_happiness: i32,
    pub happiness_target: i32,
    pub months_played: u32,
    pub months_required: u32,
    pub is_profitable: bool,
}

pub fn get_victory_progress(
    building: &Building,
    tenants: &[Tenant],
    funds: &PlayerFunds,
    current_tick: u32,
) -> VictoryProgress {
    let total_units = building.apartments.len() as f32;
    let occupied = building.occupancy_count() as f32;
    
    let avg_happiness = if tenants.is_empty() {
        0
    } else {
        tenants.iter().map(|t| t.happiness).sum::<i32>() / tenants.len() as i32
    };
    
    VictoryProgress {
        occupancy_percent: if total_units > 0.0 { (occupied / total_units) * 100.0 } else { 0.0 },
        avg_happiness,
        happiness_target: thresholds::MIN_HAPPINESS,
        months_played: current_tick,
        months_required: thresholds::MIN_TICKS_FOR_VICTORY,
        is_profitable: funds.balance > 0 && funds.net_profit() >= 0,
    }
}
