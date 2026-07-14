//! Per-difficulty rule modifiers and the one-time application of a tier's
//! rules when a game is created from a building template.

use serde::{Deserialize, Serialize};

use super::GameConfig;

/// Rule adjustments a difficulty tier applies at game start. These turn the
/// three property tiers from "same game, more units" into genuinely different
/// experiences — harder tiers start you with less cash and face stricter
/// inspections, more problem tenants, and higher overhead.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DifficultyModifiers {
    /// Starting cash for a game on this tier.
    pub starting_funds: i32,
    /// Multiplier applied to `regulations.fine_multiplier`.
    pub inspection_fine_multiplier: f32,
    /// Overrides `regulations.random_inspection_chance_percent`.
    pub random_inspection_chance_percent: i32,
    /// Overrides `tenant_risk.problem_applicant_chance_percent`.
    pub problem_applicant_chance_percent: i32,
    /// Multiplier applied to `operating_costs.base_monthly_cost_per_unit`.
    pub operating_cost_multiplier: f32,
}

impl GameConfig {
    /// Apply the modifiers for `difficulty` (case-insensitive) in place and
    /// return the tier's starting funds (falling back to 5000 if the tier is
    /// not configured). Called once when a game is created from a template.
    pub fn apply_difficulty(&mut self, difficulty: &str) -> i32 {
        let key = difficulty.to_lowercase();
        let Some(modifiers) = self
            .difficulty
            .iter()
            .find(|(name, _)| name.to_lowercase() == key)
            .map(|(_, m)| m.clone())
        else {
            return 5000;
        };

        self.regulations.fine_multiplier *= modifiers.inspection_fine_multiplier;
        self.regulations.random_inspection_chance_percent =
            modifiers.random_inspection_chance_percent;
        self.tenant_risk.problem_applicant_chance_percent =
            modifiers.problem_applicant_chance_percent;
        self.operating_costs.base_monthly_cost_per_unit =
            (self.operating_costs.base_monthly_cost_per_unit as f32
                * modifiers.operating_cost_multiplier) as i32;

        modifiers.starting_funds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_difficulty_tightens_the_rules() {
        let mut config = GameConfig::default();
        let base_fine = config.regulations.fine_multiplier;
        let funds = config.apply_difficulty("Hard");
        assert_eq!(funds, 3500);
        assert!(config.regulations.fine_multiplier > base_fine);
        assert_eq!(config.regulations.random_inspection_chance_percent, 12);
        assert_eq!(config.tenant_risk.problem_applicant_chance_percent, 28);
    }

    #[test]
    fn easy_difficulty_is_gentler_than_medium() {
        let mut easy = GameConfig::default();
        let easy_funds = easy.apply_difficulty("Easy");
        let mut medium = GameConfig::default();
        let medium_funds = medium.apply_difficulty("Medium");
        assert!(easy_funds > medium_funds);
        assert!(easy.regulations.fine_multiplier < medium.regulations.fine_multiplier);
    }

    #[test]
    fn unknown_difficulty_defaults_to_5000_without_changes() {
        let mut config = GameConfig::default();
        let base_chance = config.regulations.random_inspection_chance_percent;
        let funds = config.apply_difficulty("Nonexistent");
        assert_eq!(funds, 5000);
        assert_eq!(
            config.regulations.random_inspection_chance_percent,
            base_chance
        );
    }
}
