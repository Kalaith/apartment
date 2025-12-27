
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use super::{Apartment, ApartmentSize, NoiseLevel};
use super::ownership::OwnershipType;
use crate::data::config::MarketingConfig;

/// Marketing campaign types with different costs and target demographics
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum MarketingType {
    #[default]
    None,              // No active marketing
    SocialMedia,       // Attracts Students/Artists
    LocalNewspaper,    // Attracts Elderly/Families
    PremiumAgency,     // Attracts Professionals
}

impl MarketingType {
    pub fn monthly_cost(&self, config: &MarketingConfig) -> i32 {
        match self {
            MarketingType::None => config.none_cost,
            MarketingType::SocialMedia => config.social_media_cost,
            MarketingType::LocalNewspaper => config.local_newspaper_cost,
            MarketingType::PremiumAgency => config.premium_agency_cost,
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            MarketingType::None => "None",
            MarketingType::SocialMedia => "Social Media",
            MarketingType::LocalNewspaper => "Local Newspaper",
            MarketingType::PremiumAgency => "Premium Agency",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Building {
    pub name: String,
    pub apartments: Vec<Apartment>,
    pub hallway_condition: i32,  // 0-100, affects building appeal
    pub rent_multiplier: f32,    // 0.5 - 2.0 default 1.0
    pub has_laundry: bool,       // Amenity
    pub ownership_model: OwnershipType,
    
    // Operating flags
    pub utilities_included: bool,
    pub insurance_active: bool,
    
    // Marketing & Tenant Acquisition
    pub marketing_strategy: MarketingType,  // Current marketing approach
    pub open_house_remaining: u32,          // Months of open house bonus remaining
    pub flags: HashSet<String>,
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
            rent_multiplier: 1.0,
            has_laundry: false,
            ownership_model: OwnershipType::FullRental,
            
            // Defaults
            utilities_included: false,
            insurance_active: false,
            marketing_strategy: MarketingType::None,
            open_house_remaining: 0,
            flags: HashSet::new(),
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
        
        let mut score = hallway_factor + avg_factor;
        
        if self.has_laundry {
            score += 10;
        }
        
        score.min(100)
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

    /// Convert a rental unit to a condo (sell it)
    pub fn convert_unit_to_condo(&mut self, apartment_id: u32, owner_name: &str, sale_price: i32) -> bool {
        // Ensure apartment exists and is handled correctly ??
        // Actually, we're just updating the ownership model state here.
        // We probably need to verify it's not already owned?
        

        use super::ownership::CondoBoard;
        
        // Check if apartment exists
        if !self.apartments.iter().any(|a| a.id == apartment_id) {
            return false;
        }

        // Initialize board if rental
        match &mut self.ownership_model {
            OwnershipType::FullRental => {
                let mut board = CondoBoard::new();
                board.add_unit(apartment_id, owner_name, 200, sale_price); // $200 HOA default
                self.ownership_model = OwnershipType::MixedOwnership(board);
                true
            },
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                // Check if already in board
                if board.units.iter().any(|u| u.apartment_id == apartment_id) {
                    return false; // Already owned
                }
                board.add_unit(apartment_id, owner_name, 200, sale_price);
                
                // If all units sold, switch to FullCondo ??
                // Logic for "all units" check might be expensive here?
                // Let's just keep Mixed for now unless strict transition needed.
                true
            },
            _ => false, // Can't convert from Coop/Social easily yet
        }
    }
    pub fn update_ownership(&mut self, current_month: u32) -> bool {

        match &mut self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                board.collect_fees();
                board.resolve_votes(current_month);
                true
            },
            _ => false,
        }
    }
    
    /// Check if a specific apartment has been sold as a condo
    pub fn is_unit_sold(&self, apartment_id: u32) -> bool {
        match &self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                board.units.iter().any(|u| u.apartment_id == apartment_id)
            },
            _ => false,
        }
    }
    
    /// Get the condo info for a sold unit (owner name, HOA, purchase price)
    pub fn get_condo_info(&self, apartment_id: u32) -> Option<(String, i32)> {
        match &self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                board.units.iter()
                    .find(|u| u.apartment_id == apartment_id)
                    .map(|u| (u.owner_name.clone(), u.purchase_price))
            },
            _ => None,
        }
    }
    
    /// Buy back a condo unit (returns cost if successful)
    pub fn buyback_condo(&mut self, apartment_id: u32) -> Option<i32> {
        match &mut self.ownership_model {
            OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
                if let Some(idx) = board.units.iter().position(|u| u.apartment_id == apartment_id) {
                    // Buyback costs 110% of original purchase price
                    let buyback_price = (board.units[idx].purchase_price as f32 * 1.1) as i32;
                    board.units.remove(idx);
                    
                    // If no more sold units, revert to FullRental
                    if board.units.is_empty() {
                        self.ownership_model = OwnershipType::FullRental;
                    }
                    
                    Some(buyback_price)
                } else {
                    None
                }
            },
            _ => None,
        }
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
