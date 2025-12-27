
use serde::{Deserialize, Serialize};
use crate::tenant::TenantArchetype;
use crate::data::config::GentrificationConfig;

/// Tracks a displacement event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisplacementEvent {
    pub tenant_name: String,
    pub archetype: TenantArchetype,
    pub original_rent: i32,
    pub final_rent: i32,
    pub months_resided: u32,
    pub reason: DisplacementReason,
    pub month: u32,
    pub building_name: String,
    pub neighborhood_name: String,
}

/// Why a tenant was displaced
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DisplacementReason {
    RentIncrease,
    UnitConversion,
    Renovation,
    Eviction,
    NeighborhoodGentrification,
    BuildingSold,
}

/// Tracks gentrification across the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GentrificationTracker {
    pub displacements: Vec<DisplacementEvent>,
    pub rent_history: std::collections::HashMap<u32, Vec<(u32, i32, i32)>>,
    pub demographic_shifts: std::collections::HashMap<u32, DemographicSnapshot>,
    pub gentrification_score: i32,
    pub tenants_preserved: u32,
    pub affordable_units: u32,
}

/// Snapshot of tenant demographics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DemographicSnapshot {
    pub month: u32,
    pub student_count: u32,
    pub professional_count: u32,
    pub artist_count: u32,
    pub family_count: u32,
    pub elderly_count: u32,
    pub average_rent: i32,
}

impl GentrificationTracker {
    pub fn new() -> Self {
        Self {
            displacements: Vec::new(),
            rent_history: std::collections::HashMap::new(),
            demographic_shifts: std::collections::HashMap::new(),
            gentrification_score: 0,
            tenants_preserved: 0,
            affordable_units: 0,
        }
    }

    /// Record a rent change
    pub fn record_rent_change(&mut self, building_id: u32, month: u32, old_avg: i32, new_avg: i32, config: &GentrificationConfig) {
        let history = self.rent_history.entry(building_id).or_insert_with(Vec::new);
        history.push((month, old_avg, new_avg));
        
        // If significant rent increase, add to gentrification score
        if old_avg > 0 {
            let increase_percent = ((new_avg - old_avg) as f32 / old_avg as f32 * 100.0) as i32;
            if increase_percent > config.rent_increase_threshold_percent {
                self.gentrification_score = (self.gentrification_score + increase_percent / config.rent_increase_score_divisor)
                    .min(config.max_gentrification_score);
            }
        }
    }

    /// Update affordable unit count
    pub fn update_affordable_units(&mut self, apartments: &[crate::building::Apartment], config: &GentrificationConfig) {
        self.affordable_units = apartments.iter()
            .filter(|a| a.rent_price <= config.affordable_threshold)
            .count() as u32;
    }
}

impl Default for GentrificationTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gentrification_score() {
        let mut tracker = GentrificationTracker::new();
        let config = GentrificationConfig::default();
        assert_eq!(tracker.gentrification_score, 0);
        
        tracker.record_rent_change(0, 1, 500, 700, &config);
        assert!(tracker.gentrification_score > 0);
    }

    #[test]
    fn test_demographic_diversity() {
        let snapshot = DemographicSnapshot {
            month: 0,
            student_count: 2,
            professional_count: 2,
            artist_count: 2,
            family_count: 0,
            elderly_count: 0,
            average_rent: 600,
        };
        
        assert!(snapshot.diversity_score() >= 40); // 3 different types
    }
}
