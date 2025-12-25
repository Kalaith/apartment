#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use super::{Neighborhood, NeighborhoodType, PropertyMarket};
use crate::building::Building;
use crate::tenant::Tenant;

/// The city contains all neighborhoods and provides the top-level game world
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct City {
    pub name: String,
    pub neighborhoods: Vec<Neighborhood>,
    pub buildings: Vec<Building>,
    pub market: PropertyMarket,
    
    /// Currently selected building index
    pub active_building_index: usize,
    
    /// Global economic factors
    pub economy_health: f32,        // 0.5 = recession, 1.0 = normal, 1.5 = boom
    pub interest_rate: f32,         // Affects loan costs
    pub inflation_rate: f32,        // Affects rent expectations
    
    /// City statistics
    pub total_months: u32,
    pub total_buildings_managed: u32,
}

impl City {
    /// Create a new city with default neighborhoods
    pub fn new(name: &str) -> Self {
        let neighborhoods = vec![
            Neighborhood::new(0, NeighborhoodType::Downtown, "Central District"),
            Neighborhood::new(1, NeighborhoodType::Suburbs, "Greenfield Heights"),
            Neighborhood::new(2, NeighborhoodType::Industrial, "Old Docks"),
            Neighborhood::new(3, NeighborhoodType::Historic, "Heritage Row"),
        ];

        Self {
            name: name.to_string(),
            neighborhoods,
            buildings: Vec::new(),
            market: PropertyMarket::new(),
            active_building_index: 0,
            economy_health: 1.0,
            interest_rate: 0.05,
            inflation_rate: 0.02,
            total_months: 0,
            total_buildings_managed: 0,
        }
    }

    /// Create a city with a starting building in a given neighborhood
    pub fn with_starter_building(name: &str, neighborhood_index: usize) -> Self {
        let mut city = Self::new(name);
        
        // Create starter building
        let building = Building::default_mvp();
        let building_id = 0;
        
        city.buildings.push(building);
        city.total_buildings_managed = 1;
        
        // Add to neighborhood
        if let Some(neighborhood) = city.neighborhoods.get_mut(neighborhood_index) {
            neighborhood.add_building(building_id);
        }
        
        city
    }

    /// Get the currently active building
    pub fn active_building(&self) -> Option<&Building> {
        self.buildings.get(self.active_building_index)
    }

    /// Get mutable reference to the currently active building
    pub fn active_building_mut(&mut self) -> Option<&mut Building> {
        self.buildings.get_mut(self.active_building_index)
    }

    /// Get building by ID
    pub fn get_building(&self, id: u32) -> Option<&Building> {
        self.buildings.iter().find(|b| self.building_index_to_id(self.buildings.iter().position(|x| std::ptr::eq(x, *b)).unwrap_or(0)) == id)
    }

    /// Get mutable building by ID
    pub fn get_building_mut(&mut self, id: u32) -> Option<&mut Building> {
        self.buildings.get_mut(id as usize)
    }

    /// Helper to convert building index to ID (for now, they're the same)
    fn building_index_to_id(&self, index: usize) -> u32 {
        index as u32
    }

    /// Switch to a different building
    pub fn switch_building(&mut self, index: usize) {
        if index < self.buildings.len() {
            self.active_building_index = index;
        }
    }

    /// Get neighborhood for a building
    pub fn neighborhood_for_building(&self, building_index: usize) -> Option<&Neighborhood> {
        let building_id = building_index as u32;
        self.neighborhoods.iter().find(|n| n.building_ids.contains(&building_id))
    }

    /// Get mutable neighborhood for a building
    pub fn neighborhood_for_building_mut(&mut self, building_index: usize) -> Option<&mut Neighborhood> {
        let building_id = building_index as u32;
        self.neighborhoods.iter_mut().find(|n| n.building_ids.contains(&building_id))
    }

    /// Add a new building to a neighborhood
    pub fn add_building(&mut self, building: Building, neighborhood_id: u32) -> Result<u32, String> {
        // Check if neighborhood can accept more buildings
        let neighborhood = self.neighborhoods.iter_mut()
            .find(|n| n.id == neighborhood_id)
            .ok_or("Neighborhood not found")?;

        if !neighborhood.can_add_building() {
            return Err("Neighborhood is at capacity".to_string());
        }

        let building_id = self.buildings.len() as u32;
        self.buildings.push(building);
        neighborhood.add_building(building_id);
        self.total_buildings_managed += 1;

        Ok(building_id)
    }

    /// Calculate total monthly income across all buildings
    pub fn total_monthly_income(&self, tenants: &[Tenant]) -> i32 {
        let mut total = 0;
        for building in &self.buildings {
            for apt in &building.apartments {
                if let Some(tenant_id) = apt.tenant_id {
                    if tenants.iter().any(|t| t.id == tenant_id) {
                        total += apt.rent_price;
                    }
                }
            }
        }
        total
    }

    /// Calculate total property value
    pub fn total_property_value(&self) -> i32 {
        self.buildings.iter()
            .map(|b| self.estimate_building_value(b))
            .sum()
    }

    /// Estimate value of a single building
    fn estimate_building_value(&self, building: &Building) -> i32 {
        let base_value = 50000 * building.apartments.len() as i32;
        let condition_factor = building.building_appeal() as f32 / 100.0;
        let upgrades_value: i32 = building.apartments.iter()
            .map(|a| a.design.appeal_score() * 500 + if a.has_soundproofing { 2000 } else { 0 })
            .sum();
        
        (base_value as f32 * condition_factor) as i32 + upgrades_value
    }

    /// Get all buildings as a vector of (index, building, neighborhood_name)
    pub fn buildings_with_info(&self) -> Vec<(usize, &Building, String)> {
        self.buildings.iter().enumerate().map(|(i, b)| {
            let neighborhood_name = self.neighborhood_for_building(i)
                .map(|n| n.name.clone())
                .unwrap_or_else(|| "Unknown".to_string());
            (i, b, neighborhood_name)
        }).collect()
    }

    /// Monthly tick for all city systems
    pub fn tick(&mut self) {
        self.total_months += 1;
        
        // Update neighborhoods
        for neighborhood in &mut self.neighborhoods {
            neighborhood.tick();
        }
        
        // Refresh market listings periodically
        if self.total_months % 3 == 0 {
            self.market.refresh_listings(&self.neighborhoods);
        }
        
        // Random economic events
        self.update_economy();
    }

    /// Update economic conditions
    fn update_economy(&mut self) {
        // Small random fluctuations
        let change = macroquad::rand::gen_range(-5, 6) as f32 / 100.0;
        self.economy_health = (self.economy_health + change).clamp(0.5, 1.5);
        
        // Interest rates inversely track economy health
        let target_rate = 0.08 - (self.economy_health - 1.0) * 0.05;
        self.interest_rate = (self.interest_rate + (target_rate - self.interest_rate) * 0.1).clamp(0.02, 0.15);
        
        // Inflation tracks economy health
        self.inflation_rate = (self.economy_health - 0.7) * 0.05;
    }
}

impl Default for City {
    fn default() -> Self {
        Self::new("Metropolis")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_city_creation() {
        let city = City::new("Test City");
        assert_eq!(city.neighborhoods.len(), 4);
        assert_eq!(city.buildings.len(), 0);
    }

    #[test]
    fn test_starter_building() {
        let city = City::with_starter_building("Test City", 0);
        assert_eq!(city.buildings.len(), 1);
        assert!(city.neighborhoods[0].building_ids.contains(&0));
    }
}
