#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use crate::tenant::TenantArchetype;

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
    /// Rent increase beyond tolerance
    RentIncrease,
    /// Unit sold or converted
    UnitConversion,
    /// Building renovations made it unaffordable
    Renovation,
    /// Eviction (legal process)
    Eviction,
    /// Neighborhood became unaffordable
    NeighborhoodGentrification,
    /// Building sold to new owner
    BuildingSold,
}

impl DisplacementReason {
    pub fn description(&self) -> &'static str {
        match self {
            DisplacementReason::RentIncrease => "Priced out by rent increases",
            DisplacementReason::UnitConversion => "Unit sold or converted",
            DisplacementReason::Renovation => "Renovations made unit unaffordable",
            DisplacementReason::Eviction => "Evicted from unit",
            DisplacementReason::NeighborhoodGentrification => "Neighborhood became unaffordable",
            DisplacementReason::BuildingSold => "Building sold to new owner",
        }
    }

    /// Does this affect player reputation?
    pub fn reputation_impact(&self) -> i32 {
        match self {
            DisplacementReason::RentIncrease => -5,
            DisplacementReason::UnitConversion => -10,
            DisplacementReason::Renovation => -3,
            DisplacementReason::Eviction => -15,
            DisplacementReason::NeighborhoodGentrification => -2,
            DisplacementReason::BuildingSold => -5,
        }
    }
}

/// Tracks gentrification across the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GentrificationTracker {
    /// All displacement events
    pub displacements: Vec<DisplacementEvent>,
    
    /// Rent increases by building (building_id -> Vec<(month, old_avg, new_avg)>)
    pub rent_history: std::collections::HashMap<u32, Vec<(u32, i32, i32)>>,
    
    /// Archetype distribution changes by neighborhood
    pub demographic_shifts: std::collections::HashMap<u32, DemographicSnapshot>,
    
    /// Player's gentrification score (higher = more displacement)
    pub gentrification_score: i32,
    
    /// Number of original/long-term tenants preserved
    pub tenants_preserved: u32,
    
    /// Number of affordable units maintained
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

impl DemographicSnapshot {
    pub fn new(month: u32) -> Self {
        Self {
            month,
            student_count: 0,
            professional_count: 0,
            artist_count: 0,
            family_count: 0,
            elderly_count: 0,
            average_rent: 0,
        }
    }

    pub fn from_tenants(tenants: &[crate::tenant::Tenant], apartments: &[crate::building::Apartment], month: u32) -> Self {
        let mut snapshot = Self::new(month);
        
        for tenant in tenants {
            match tenant.archetype {
                TenantArchetype::Student => snapshot.student_count += 1,
                TenantArchetype::Professional => snapshot.professional_count += 1,
                TenantArchetype::Artist => snapshot.artist_count += 1,
                TenantArchetype::Family => snapshot.family_count += 1,
                TenantArchetype::Elderly => snapshot.elderly_count += 1,
            }
        }

        // Calculate average rent
        let occupied_apts: Vec<_> = apartments.iter()
            .filter(|a| a.tenant_id.is_some())
            .collect();
        
        if !occupied_apts.is_empty() {
            let total_rent: i32 = occupied_apts.iter().map(|a| a.rent_price).sum();
            snapshot.average_rent = total_rent / occupied_apts.len() as i32;
        }

        snapshot
    }

    /// Calculate diversity score (0-100, higher = more diverse)
    pub fn diversity_score(&self) -> i32 {
        let total = self.student_count + self.professional_count + 
                    self.artist_count + self.family_count + self.elderly_count;
        
        if total == 0 {
            return 0;
        }

        // Calculate how evenly distributed the population is
        let counts = [
            self.student_count, 
            self.professional_count, 
            self.artist_count, 
            self.family_count, 
            self.elderly_count
        ];
        
        let non_zero = counts.iter().filter(|&&c| c > 0).count() as i32;
        
        // More different types = more diverse
        (non_zero * 20).min(100)
    }
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

    /// Record a displacement event
    pub fn record_displacement(&mut self, event: DisplacementEvent) {
        let impact = event.reason.reputation_impact();
        self.gentrification_score = (self.gentrification_score - impact).max(0);
        self.displacements.push(event);
    }

    /// Record a rent change
    pub fn record_rent_change(&mut self, building_id: u32, month: u32, old_avg: i32, new_avg: i32) {
        let history = self.rent_history.entry(building_id).or_insert_with(Vec::new);
        history.push((month, old_avg, new_avg));
        
        // If significant rent increase, add to gentrification score
        let increase_percent = ((new_avg - old_avg) as f32 / old_avg as f32 * 100.0) as i32;
        if increase_percent > 10 {
            self.gentrification_score = (self.gentrification_score + increase_percent / 5).min(100);
        }
    }

    /// Update demographic snapshot for a neighborhood
    pub fn update_demographics(
        &mut self, 
        neighborhood_id: u32, 
        tenants: &[crate::tenant::Tenant],
        apartments: &[crate::building::Apartment],
        month: u32
    ) {
        let snapshot = DemographicSnapshot::from_tenants(tenants, apartments, month);
        self.demographic_shifts.insert(neighborhood_id, snapshot);
    }

    /// Record a preserved long-term tenant
    pub fn record_preservation(&mut self) {
        self.tenants_preserved += 1;
        // Reduce gentrification score for preservation
        self.gentrification_score = (self.gentrification_score - 2).max(0);
    }

    /// Update affordable unit count
    pub fn update_affordable_units(&mut self, apartments: &[crate::building::Apartment]) {
        // Define affordable as <= $700/month
        const AFFORDABLE_THRESHOLD: i32 = 700;
        self.affordable_units = apartments.iter()
            .filter(|a| a.rent_price <= AFFORDABLE_THRESHOLD)
            .count() as u32;
    }

    /// Get gentrification level description
    pub fn gentrification_level(&self) -> &'static str {
        match self.gentrification_score {
            0..=20 => "Minimal",
            21..=40 => "Modest",
            41..=60 => "Moderate",
            61..=80 => "Significant",
            _ => "Severe",
        }
    }

    /// Get total displacement count
    pub fn total_displacements(&self) -> usize {
        self.displacements.len()
    }

    /// Get recent displacements (last N)
    pub fn recent_displacements(&self, count: usize) -> Vec<&DisplacementEvent> {
        self.displacements.iter().rev().take(count).collect()
    }

    /// Check if a rent increase would cause displacement
    pub fn would_cause_displacement(
        &self,
        tenant: &crate::tenant::Tenant,
        current_rent: i32,
        new_rent: i32,
    ) -> bool {
        let increase = new_rent - current_rent;
        let tolerance = tenant.rent_tolerance;
        
        new_rent > tolerance || (increase > 0 && increase as f32 / current_rent as f32 > 0.2)
    }

    /// Calculate displacement risk for a building
    pub fn displacement_risk(
        &self,
        tenants: &[crate::tenant::Tenant],
        apartments: &[crate::building::Apartment],
    ) -> i32 {
        let mut at_risk = 0;
        
        for tenant in tenants {
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apt) = apartments.iter().find(|a| a.id == apt_id) {
                    // Tenant is at risk if paying close to their tolerance
                    if apt.rent_price as f32 / tenant.rent_tolerance as f32 > 0.9 {
                        at_risk += 1;
                    }
                }
            }
        }
        
        if tenants.is_empty() {
            0
        } else {
            (at_risk * 100 / tenants.len() as i32).min(100)
        }
    }

    /// Generate summary statistics
    pub fn summary(&self) -> GentrificationSummary {
        GentrificationSummary {
            total_displacements: self.displacements.len(),
            gentrification_score: self.gentrification_score,
            gentrification_level: self.gentrification_level().to_string(),
            tenants_preserved: self.tenants_preserved,
            affordable_units: self.affordable_units,
            by_reason: self.displacements_by_reason(),
        }
    }

    fn displacements_by_reason(&self) -> std::collections::HashMap<String, u32> {
        let mut counts = std::collections::HashMap::new();
        
        for event in &self.displacements {
            let key = event.reason.description().to_string();
            *counts.entry(key).or_insert(0) += 1;
        }
        
        counts
    }
}

/// Summary of gentrification impacts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GentrificationSummary {
    pub total_displacements: usize,
    pub gentrification_score: i32,
    pub gentrification_level: String,
    pub tenants_preserved: u32,
    pub affordable_units: u32,
    pub by_reason: std::collections::HashMap<String, u32>,
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
        assert_eq!(tracker.gentrification_score, 0);
        
        tracker.record_rent_change(0, 1, 500, 700);
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
