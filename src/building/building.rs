use serde::{Deserialize, Serialize};
use super::{Apartment, ApartmentSize, NoiseLevel};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Building {
    pub name: String,
    pub apartments: Vec<Apartment>,
    pub hallway_condition: i32,  // 0-100, affects building appeal
}

impl Building {
    /// Create a new building with generated apartments
    pub fn new(name: &str, num_floors: u32, units_per_floor: u32) -> Self {
        let mut apartments = Vec::new();
        let mut id = 0;
        
        for floor in 1..=num_floors {
            for unit in 0..units_per_floor {
                let unit_letter = (b'A' + unit as u8) as char;
                let unit_number = format!("{}{}", floor, unit_letter);
                
                // Alternate sizes and noise levels for variety
                let size = if (floor + unit) % 2 == 0 {
                    ApartmentSize::Small
                } else {
                    ApartmentSize::Medium
                };
                
                // Ground floor and street-facing (A) units are noisier
                let noise = if floor == 1 || unit == 0 {
                    NoiseLevel::High
                } else {
                    NoiseLevel::Low
                };
                
                apartments.push(Apartment::new(id, &unit_number, floor, size, noise));
                id += 1;
            }
        }
        
        Self {
            name: name.to_string(),
            apartments,
            hallway_condition: 60,  // Start slightly worn
        }
    }
    
    /// Create the default MVP building (6 units, 3 floors, 2 per floor)
    pub fn default_mvp() -> Self {
        Self::new("Sunset Apartments", 3, 2)
    }
    
    /// Get apartment by ID
    pub fn get_apartment(&self, id: u32) -> Option<&Apartment> {
        self.apartments.iter().find(|a| a.id == id)
    }
    
    /// Get mutable apartment by ID
    pub fn get_apartment_mut(&mut self, id: u32) -> Option<&mut Apartment> {
        self.apartments.iter_mut().find(|a| a.id == id)
    }
    
    /// Get all vacant apartments
    pub fn vacant_apartments(&self) -> Vec<&Apartment> {
        self.apartments.iter().filter(|a| a.is_vacant()).collect()
    }
    
    /// Get all occupied apartments
    pub fn occupied_apartments(&self) -> Vec<&Apartment> {
        self.apartments.iter().filter(|a| !a.is_vacant()).collect()
    }
    
    /// Count vacant units
    pub fn vacancy_count(&self) -> usize {
        self.apartments.iter().filter(|a| a.is_vacant()).count()
    }
    
    /// Count occupied units
    pub fn occupancy_count(&self) -> usize {
        self.apartments.iter().filter(|a| !a.is_vacant()).count()
    }
    
    /// Calculate overall building appeal (affects tenant applications)
    pub fn building_appeal(&self) -> i32 {
        let hallway_factor = self.hallway_condition / 2;  // 0-50
        let avg_condition: i32 = if self.apartments.is_empty() {
            0
        } else {
            self.apartments.iter().map(|a| a.condition).sum::<i32>() 
                / self.apartments.len() as i32
        };
        let avg_factor = avg_condition / 2;  // 0-50
        
        hallway_factor + avg_factor
    }
    
    /// Repair hallway
    pub fn repair_hallway(&mut self, amount: i32) {
        self.hallway_condition = (self.hallway_condition + amount).min(100);
    }
    
    /// Decay hallway condition
    pub fn decay_hallway(&mut self, amount: i32) {
        self.hallway_condition = (self.hallway_condition - amount).max(0);
    }
    
    /// Apply decay to all apartments and hallway
    pub fn apply_monthly_decay(&mut self) {
        for apt in &mut self.apartments {
            apt.decay_condition(2);  // Slow decay
        }
        self.decay_hallway(1);  // Even slower for shared space
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_building_generation() {
        let building = Building::new("Test", 3, 2);
        assert_eq!(building.apartments.len(), 6);
    }
    
    #[test]
    fn test_default_mvp_building() {
        let building = Building::default_mvp();
        assert_eq!(building.name, "Sunset Apartments");
        assert_eq!(building.apartments.len(), 6);
        assert_eq!(building.hallway_condition, 60);
    }
    
    #[test]
    fn test_vacancy_tracking() {
        let mut building = Building::default_mvp();
        assert_eq!(building.vacancy_count(), 6);
        assert_eq!(building.occupancy_count(), 0);
        
        building.get_apartment_mut(0).unwrap().move_in(1);
        building.get_apartment_mut(1).unwrap().move_in(2);
        
        assert_eq!(building.vacancy_count(), 4);
        assert_eq!(building.occupancy_count(), 2);
    }
    
    #[test]
    fn test_building_appeal() {
        let building = Building::default_mvp();
        // hallway_condition = 60, each apartment condition = 50
        // hallway_factor = 60 / 2 = 30
        // avg_factor = 50 / 2 = 25
        // total = 55
        assert_eq!(building.building_appeal(), 55);
    }
    
    #[test]
    fn test_monthly_decay() {
        let mut building = Building::default_mvp();
        let initial_hallway = building.hallway_condition;
        let initial_apt_condition = building.apartments[0].condition;
        
        building.apply_monthly_decay();
        
        assert_eq!(building.hallway_condition, initial_hallway - 1);
        assert_eq!(building.apartments[0].condition, initial_apt_condition - 2);
    }
}
