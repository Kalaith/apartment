
use serde::{Deserialize, Serialize};
use crate::building::Building;
use crate::tenant::Tenant;
use crate::economy::PlayerFunds;
use crate::data::config::{WinConditions, HappinessConfig, ThresholdsConfig};

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

/// Check current game state for win/lose conditions
pub fn check_win_condition(
    building: &Building,
    tenants: &[Tenant],
    funds: &PlayerFunds,
    current_tick: u32,
    win_conditions: &WinConditions,
    _happiness_config: &HappinessConfig,
    thresholds: &ThresholdsConfig,
) -> Option<GameOutcome> {
    // Check for bankruptcy
    if funds.is_bankrupt() {
        return Some(GameOutcome::Bankruptcy { debt: funds.balance.abs() });
    }
    
    // Check if all tenants left (after having some)
    if tenants.is_empty() && current_tick > thresholds.all_left_check_tick {
        // Check if we ever had tenants (building was lived in)
        let was_occupied = building.apartments.iter().any(|a| a.tenant_id.is_some());
        if !was_occupied {
            return Some(GameOutcome::AllTenantsLeft);
        }
    }
    
    // Check for game end (3 years = 36 months)
    let game_duration = win_conditions.game_duration_ticks.unwrap_or(36);
    if current_tick >= game_duration {
        // Calculate final score based on performance
        let avg_happiness: i32 = if tenants.is_empty() {
            0
        } else {
            tenants.iter().map(|t| t.happiness).sum::<i32>() / tenants.len() as i32
        };
        
        let occupancy_bonus = if building.vacancy_count() == 0 { 100 } else { 0 };
        let tenant_count_bonus = (tenants.len() as i32) * 10;
        
        let score = (avg_happiness * 5)  // Happiness contribution
            + (funds.total_income / 100)  // Income contribution
            + occupancy_bonus             // Full building bonus
            + tenant_count_bonus;         // Tenant retention bonus
        
        return Some(GameOutcome::Victory {
            total_income: funds.total_income,
            months: current_tick,
            score,
        });
    }
    
    None
}

