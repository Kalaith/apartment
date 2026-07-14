//! Tuning for the social layer between tenants: relationships, the emergent
//! disruptor dilemma, and building-wide cohesion.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipsConfig {
    pub happiness_modifiers: HashMap<String, i32>,
    pub formation_chance: i32,
    pub hostile_cooldown_chance: i32,
    pub hostile_strength_decay: i32,
    pub hostile_transition_threshold: i32,
    pub same_archetype_friendly_chance: i32,
    pub adjacent_hostile_chance: i32,
    #[serde(default)]
    pub dilemma: DilemmaConfig,
}

impl Default for RelationshipsConfig {
    fn default() -> Self {
        let mut happiness_modifiers = HashMap::new();
        happiness_modifiers.insert("friendly".to_string(), 5);
        happiness_modifiers.insert("neutral".to_string(), 0);
        happiness_modifiers.insert("hostile".to_string(), -10);
        happiness_modifiers.insert("romantic".to_string(), 8);
        happiness_modifiers.insert("family".to_string(), 10);

        Self {
            happiness_modifiers,
            formation_chance: 5,
            hostile_cooldown_chance: 5,
            hostile_strength_decay: 5,
            hostile_transition_threshold: 20,
            same_archetype_friendly_chance: 60,
            adjacent_hostile_chance: 30,
            dilemma: DilemmaConfig::default(),
        }
    }
}

/// Thresholds for the emergent "high-rent tenant vs. unhappy neighbors" dilemma
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DilemmaConfig {
    /// Hostile relationships needed to qualify as a disruptor outright
    pub min_hostile_relationships: u32,
    /// With only one hostile relationship, behavior below this still qualifies
    pub single_hostile_max_behavior: i32,
    /// Disruptor's rent must be at least this multiple of the average occupied rent
    pub rent_premium_multiplier: f32,
    /// Months before the same tenant can trigger another dilemma
    pub cooldown_months: u32,
}

impl Default for DilemmaConfig {
    fn default() -> Self {
        Self {
            min_hostile_relationships: 2,
            single_hostile_max_behavior: 40,
            rent_premium_multiplier: 1.2,
            cooldown_months: 6,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohesionConfig {
    pub archetype_group_threshold: i32,
    pub archetype_group_base_bonus: i32,
    pub archetype_group_per_extra: i32,
    pub friendly_relationship_bonus: i32,
    pub hostile_relationship_penalty: i32,
    pub tension_penalty: i32,
    pub cohesion_min: i32,
    pub cohesion_max: i32,
}

impl Default for CohesionConfig {
    fn default() -> Self {
        Self {
            archetype_group_threshold: 3,
            archetype_group_base_bonus: 5,
            archetype_group_per_extra: 2,
            friendly_relationship_bonus: 2,
            hostile_relationship_penalty: 5,
            tension_penalty: 8,
            cohesion_min: -50,
            cohesion_max: 50,
        }
    }
}
