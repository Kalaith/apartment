use serde::{Deserialize, Serialize};

/// The type of neighborhood - each has distinct characteristics affecting gameplay
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NeighborhoodType {
    /// High rent, high turnover, noisy, good for professionals
    Downtown,
    /// Families, stability, lower margins, quiet
    Suburbs,
    /// Students, artists, gentrification pressure, affordable
    Industrial,
    /// Elderly, regulations, preservation requirements, historic charm
    Historic,
}

impl NeighborhoodType {
    pub fn name(&self) -> &'static str {
        match self {
            NeighborhoodType::Downtown => "Downtown",
            NeighborhoodType::Suburbs => "Suburbs",
            NeighborhoodType::Industrial => "Industrial District",
            NeighborhoodType::Historic => "Historic Quarter",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NeighborhoodType::Downtown => "High-energy urban core. Premium rents but demanding tenants and constant noise.",
            NeighborhoodType::Suburbs => "Family-friendly residential area. Stable tenants, lower turnover, moderate returns.",
            NeighborhoodType::Industrial => "Up-and-coming area popular with artists and students. Affordable but gentrifying.",
            NeighborhoodType::Historic => "Character-rich area with strict preservation rules. Appeals to elderly and history buffs.",
        }
    }

    /// Base rent multiplier for this neighborhood
    pub fn rent_multiplier(&self) -> f32 {
        match self {
            NeighborhoodType::Downtown => 1.5,
            NeighborhoodType::Suburbs => 1.0,
            NeighborhoodType::Industrial => 0.75,
            NeighborhoodType::Historic => 1.2,
        }
    }

    /// Base noise level (0-100)
    pub fn base_noise(&self) -> i32 {
        match self {
            NeighborhoodType::Downtown => 70,
            NeighborhoodType::Suburbs => 20,
            NeighborhoodType::Industrial => 50,
            NeighborhoodType::Historic => 30,
        }
    }

    /// How strict are building regulations? (1.0 = normal, 2.0 = very strict)
    pub fn regulation_strictness(&self) -> f32 {
        match self {
            NeighborhoodType::Downtown => 1.0,
            NeighborhoodType::Suburbs => 0.8,
            NeighborhoodType::Industrial => 0.6,
            NeighborhoodType::Historic => 2.0,
        }
    }

    /// Which tenant archetypes are common here?
    pub fn common_archetypes(&self) -> Vec<crate::tenant::TenantArchetype> {
        use crate::tenant::TenantArchetype;
        match self {
            NeighborhoodType::Downtown => vec![TenantArchetype::Professional, TenantArchetype::Student],
            NeighborhoodType::Suburbs => vec![TenantArchetype::Family, TenantArchetype::Elderly],
            NeighborhoodType::Industrial => vec![TenantArchetype::Artist, TenantArchetype::Student],
            NeighborhoodType::Historic => vec![TenantArchetype::Elderly, TenantArchetype::Professional, TenantArchetype::Artist],
        }
    }

    /// Color for UI display
    pub fn color(&self) -> macroquad::color::Color {
        use macroquad::color::Color;
        match self {
            NeighborhoodType::Downtown => Color::from_rgba(100, 149, 237, 255), // Cornflower blue
            NeighborhoodType::Suburbs => Color::from_rgba(144, 238, 144, 255),  // Light green
            NeighborhoodType::Industrial => Color::from_rgba(255, 165, 79, 255), // Orange
            NeighborhoodType::Historic => Color::from_rgba(221, 160, 221, 255), // Plum
        }
    }
}

/// Dynamic stats for a neighborhood that change over time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeighborhoodStats {
    /// Crime level (0-100, higher = worse)
    pub crime_level: i32,
    /// Transit access score (0-100)
    pub transit_access: i32,
    /// Walkability score (0-100)
    pub walkability: i32,
    /// School quality (0-100) - matters for families
    pub school_quality: i32,
    /// Local services (shops, cafes, etc.) (0-100)
    pub services: i32,
    /// Current rent demand (affects application rate)
    pub rent_demand: f32,
    /// Gentrification pressure (0-100)
    pub gentrification: i32,
}

impl NeighborhoodStats {
    pub fn for_type(neighborhood_type: &NeighborhoodType) -> Self {
        match neighborhood_type {
            NeighborhoodType::Downtown => Self {
                crime_level: 40,
                transit_access: 95,
                walkability: 90,
                school_quality: 50,
                services: 95,
                rent_demand: 1.2,
                gentrification: 80,
            },
            NeighborhoodType::Suburbs => Self {
                crime_level: 15,
                transit_access: 40,
                walkability: 30,
                school_quality: 85,
                services: 60,
                rent_demand: 1.0,
                gentrification: 20,
            },
            NeighborhoodType::Industrial => Self {
                crime_level: 50,
                transit_access: 60,
                walkability: 50,
                school_quality: 35,
                services: 45,
                rent_demand: 0.9,
                gentrification: 60,
            },
            NeighborhoodType::Historic => Self {
                crime_level: 25,
                transit_access: 70,
                walkability: 75,
                school_quality: 65,
                services: 80,
                rent_demand: 1.1,
                gentrification: 40,
            },
        }
    }

    /// Calculate overall neighborhood appeal (affects building reputation)
    pub fn appeal_score(&self) -> i32 {
        let base = 100 - self.crime_level;
        let services_bonus = self.services / 5;
        let transit_bonus = self.transit_access / 10;
        (base + services_bonus + transit_bonus).min(100)
    }

    /// Apply monthly changes to neighborhood (gentrification, crime changes, etc.)
    pub fn tick(&mut self, neighborhood_type: &NeighborhoodType) {
        // Gentrification slowly increases in industrial areas
        if matches!(neighborhood_type, NeighborhoodType::Industrial) && self.gentrification < 100 {
            if macroquad::rand::gen_range(0, 100) < 10 {
                self.gentrification = (self.gentrification + 1).min(100);
                // Gentrification increases rent demand but pushes out long-term residents
                self.rent_demand = (self.rent_demand + 0.01).min(1.5);
            }
        }

        // Crime fluctuates slightly
        let crime_change = macroquad::rand::gen_range(-2, 3);
        self.crime_level = (self.crime_level + crime_change).clamp(5, 95);

        // Rent demand fluctuates
        let demand_change = macroquad::rand::gen_range(-5, 6) as f32 / 100.0;
        self.rent_demand = (self.rent_demand + demand_change).clamp(0.5, 2.0);
    }
}

/// A neighborhood in the city containing multiple building slots
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Neighborhood {
    pub id: u32,
    pub neighborhood_type: NeighborhoodType,
    pub name: String,
    pub stats: NeighborhoodStats,
    /// IDs of buildings in this neighborhood (player-owned)
    pub building_ids: Vec<u32>,
    /// Number of available property slots
    pub available_slots: u32,
    /// Player's reputation in this neighborhood (0-100)
    pub reputation: i32,
}

impl Neighborhood {
    pub fn new(id: u32, neighborhood_type: NeighborhoodType, name: &str) -> Self {
        let stats = NeighborhoodStats::for_type(&neighborhood_type);
        Self {
            id,
            neighborhood_type,
            name: name.to_string(),
            stats,
            building_ids: Vec::new(),
            available_slots: 3, // Can acquire up to 3 buildings per neighborhood
            reputation: 50,
        }
    }

    /// Add a building to this neighborhood
    pub fn add_building(&mut self, building_id: u32) {
        if !self.building_ids.contains(&building_id) {
            self.building_ids.push(building_id);
        }
    }

    /// Remove a building from this neighborhood
    pub fn remove_building(&mut self, building_id: u32) {
        self.building_ids.retain(|&id| id != building_id);
    }

    /// Check if we can add more buildings
    pub fn can_add_building(&self) -> bool {
        (self.building_ids.len() as u32) < self.available_slots
    }

    /// Update reputation based on building performance
    pub fn update_reputation(&mut self, average_building_appeal: i32, tenant_satisfaction: i32) {
        let target = (average_building_appeal + tenant_satisfaction) / 2;
        // Slowly move towards target
        if self.reputation < target {
            self.reputation = (self.reputation + 1).min(100);
        } else if self.reputation > target {
            self.reputation = (self.reputation - 1).max(0);
        }
    }

    /// Apply monthly tick
    pub fn tick(&mut self) {
        self.stats.tick(&self.neighborhood_type);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neighborhood_creation() {
        let neighborhood = Neighborhood::new(0, NeighborhoodType::Downtown, "Central District");
        assert_eq!(neighborhood.name, "Central District");
        assert!(neighborhood.can_add_building());
    }

    #[test]
    fn test_neighborhood_stats() {
        let stats = NeighborhoodStats::for_type(&NeighborhoodType::Suburbs);
        assert!(stats.crime_level < 30); // Suburbs are safe
        assert!(stats.school_quality > 70); // Good schools
    }
}
