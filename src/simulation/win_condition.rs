use crate::building::Building;
use crate::data::config::{HappinessConfig, ThresholdsConfig, WinConditions};
use crate::economy::PlayerFunds;
use crate::tenant::Tenant;
use serde::{Deserialize, Serialize};

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
    has_ever_had_tenant: bool,
    win_conditions: &WinConditions,
    happiness_config: &HappinessConfig,
    thresholds: &ThresholdsConfig,
) -> Option<GameOutcome> {
    // Check for bankruptcy
    if funds.is_bankrupt() {
        return Some(GameOutcome::Bankruptcy {
            debt: funds.balance.abs(),
        });
    }

    // Check if all tenants left (only after the building was actually occupied at
    // some point — otherwise a brand-new empty building would instantly "lose").
    if has_ever_had_tenant && tenants.is_empty() && current_tick > thresholds.all_left_check_tick {
        return Some(GameOutcome::AllTenantsLeft);
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

        // Full occupancy is only required to earn the bonus when the config
        // says so; otherwise it's awarded unconditionally.
        let occupancy_bonus =
            if !win_conditions.full_occupancy_required || building.vacancy_count() == 0 {
                100
            } else {
                0
            };
        // Reward clearing the configured happiness bar, on top of the
        // continuous happiness contribution below.
        let happiness_bonus = if avg_happiness >= happiness_config.min_for_victory {
            50
        } else {
            0
        };
        // A defensive floor: a run that (via a very low game_duration_ticks)
        // ends before min_ticks_for_victory still gets an outcome, just
        // without this small "played it out" bonus.
        let maturity_bonus = if current_tick >= win_conditions.min_ticks_for_victory {
            20
        } else {
            0
        };
        let tenant_count_bonus = (tenants.len() as i32) * 10;

        let score = (avg_happiness * 5)  // Happiness contribution
            + (funds.total_income / 100)  // Income contribution
            + occupancy_bonus             // Full building bonus
            + happiness_bonus             // Cleared the victory happiness bar
            + maturity_bonus              // Played out at least the minimum duration
            + tenant_count_bonus; // Tenant retention bonus

        return Some(GameOutcome::Victory {
            total_income: funds.total_income,
            months: current_tick,
            score,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::Building;
    use crate::data::config::GameConfig;

    fn check(tenants: &[Tenant], tick: u32, ever_occupied: bool) -> Option<GameOutcome> {
        let building = Building::new("Test", 3, 2);
        let funds = PlayerFunds::default(); // 5000, solvent
        let cfg = GameConfig::default(); // all_left_check_tick = 3, duration = 36
        check_win_condition(
            &building,
            tenants,
            &funds,
            tick,
            ever_occupied,
            &cfg.win_conditions,
            &cfg.happiness,
            &cfg.thresholds,
        )
    }

    #[test]
    fn never_occupied_building_does_not_trigger_all_tenants_left() {
        // Empty and past the check tick, but no tenant ever moved in: not a loss.
        assert!(check(&[], 5, false).is_none());
    }

    #[test]
    fn previously_occupied_then_empty_triggers_all_tenants_left() {
        assert!(matches!(
            check(&[], 5, true),
            Some(GameOutcome::AllTenantsLeft)
        ));
    }

    #[test]
    fn empty_before_check_tick_is_not_yet_a_loss() {
        // tick 2 <= all_left_check_tick (3): a temporary early vacancy is tolerated.
        assert!(check(&[], 2, true).is_none());
    }
}
