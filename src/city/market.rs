use serde::{Deserialize, Serialize};
use super::{Neighborhood, NeighborhoodType};
use crate::building::Building;

/// Condition of a building on the market
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum BuildingCondition {
    /// Needs demolition/major renovation
    Condemned,
    /// Rundown but livable
    Poor,
    /// Average condition
    Fair,
    /// Well-maintained
    Good,
    /// Recently renovated
    Excellent,
}

impl BuildingCondition {
    pub fn price_multiplier(&self) -> f32 {
        match self {
            BuildingCondition::Condemned => 0.3,
            BuildingCondition::Poor => 0.6,
            BuildingCondition::Fair => 1.0,
            BuildingCondition::Good => 1.3,
            BuildingCondition::Excellent => 1.6,
        }
    }

    pub fn starting_apartment_condition(&self) -> i32 {
        match self {
            BuildingCondition::Condemned => 10,
            BuildingCondition::Poor => 30,
            BuildingCondition::Fair => 50,
            BuildingCondition::Good => 70,
            BuildingCondition::Excellent => 90,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            BuildingCondition::Condemned => "Condemned",
            BuildingCondition::Poor => "Poor",
            BuildingCondition::Fair => "Fair",
            BuildingCondition::Good => "Good",
            BuildingCondition::Excellent => "Excellent",
        }
    }
}

/// Financing options for property acquisition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FinancingOption {
    /// Pay full price upfront
    Cash,
    /// Bank mortgage with monthly payments
    Mortgage {
        down_payment_percent: f32,
        interest_rate: f32,
        term_months: u32,
    },
    /// Investor partner (takes % of profits)
    Investor {
        investment_percent: f32,
        profit_share_percent: f32,
    },
}

impl FinancingOption {
    pub fn name(&self) -> &'static str {
        match self {
            FinancingOption::Cash => "Cash Purchase",
            FinancingOption::Mortgage { .. } => "Bank Mortgage",
            FinancingOption::Investor { .. } => "Investor Partner",
        }
    }

    pub fn description(&self) -> String {
        match self {
            FinancingOption::Cash => "Pay the full amount upfront".to_string(),
            FinancingOption::Mortgage { down_payment_percent, interest_rate, term_months } => {
                format!(
                    "{:.0}% down, {:.1}% APR, {} month term",
                    down_payment_percent * 100.0,
                    interest_rate * 100.0,
                    term_months
                )
            }
            FinancingOption::Investor { investment_percent, profit_share_percent } => {
                format!(
                    "Investor covers {:.0}%, takes {:.0}% of profits",
                    investment_percent * 100.0,
                    profit_share_percent * 100.0
                )
            }
        }
    }

    /// Calculate upfront cost for a given property price
    pub fn upfront_cost(&self, price: i32) -> i32 {
        match self {
            FinancingOption::Cash => price,
            FinancingOption::Mortgage { down_payment_percent, .. } => {
                (price as f32 * down_payment_percent) as i32
            }
            FinancingOption::Investor { investment_percent, .. } => {
                (price as f32 * (1.0 - investment_percent)) as i32
            }
        }
    }

    /// Calculate monthly payment for a given property price
    pub fn monthly_payment(&self, price: i32) -> i32 {
        match self {
            FinancingOption::Cash => 0,
            FinancingOption::Mortgage { down_payment_percent, interest_rate, term_months } => {
                let principal = price as f32 * (1.0 - down_payment_percent);
                let monthly_rate = interest_rate / 12.0;
                if monthly_rate == 0.0 {
                    (principal / *term_months as f32) as i32
                } else {
                    let payment = principal * (monthly_rate * (1.0 + monthly_rate).powi(*term_months as i32))
                        / ((1.0 + monthly_rate).powi(*term_months as i32) - 1.0);
                    payment as i32
                }
            }
            FinancingOption::Investor { .. } => 0, // Profit share is handled differently
        }
    }
}

/// A property listing on the market
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyListing {
    pub id: u32,
    pub name: String,
    pub neighborhood_id: u32,
    pub condition: BuildingCondition,
    pub num_floors: u32,
    pub units_per_floor: u32,
    pub asking_price: i32,
    pub existing_tenants: u32,
    pub months_on_market: u32,
    pub available_financing: Vec<FinancingOption>,
    /// Special features or issues
    pub notes: Vec<String>,
}

impl PropertyListing {
    /// Create a random listing for a neighborhood
    pub fn generate(id: u32, neighborhood: &Neighborhood) -> Self {
        use macroquad::rand::gen_range;

        // Random building size
        let num_floors = gen_range(2, 5);
        let units_per_floor = gen_range(2, 4);
        let total_units = num_floors * units_per_floor;

        // Random condition (biased by neighborhood)
        let condition = match gen_range(0, 100) {
            0..=20 => BuildingCondition::Condemned,
            21..=40 => BuildingCondition::Poor,
            41..=70 => BuildingCondition::Fair,
            71..=90 => BuildingCondition::Good,
            _ => BuildingCondition::Excellent,
        };

        // Calculate base price
        let base_unit_price = match neighborhood.neighborhood_type {
            NeighborhoodType::Downtown => 80000,
            NeighborhoodType::Suburbs => 60000,
            NeighborhoodType::Industrial => 40000,
            NeighborhoodType::Historic => 70000,
        };

        let asking_price = (base_unit_price as f32 
            * total_units as f32 
            * condition.price_multiplier()
            * neighborhood.stats.rent_demand) as i32;

        // Existing tenants (condemned buildings are empty)
        let existing_tenants = if matches!(condition, BuildingCondition::Condemned) {
            0
        } else {
            gen_range(0, total_units / 2 + 1)
        };

        // Generate name
        let name = generate_building_name(&neighborhood.neighborhood_type);

        // Available financing based on price
        let mut financing = vec![FinancingOption::Cash];
        if asking_price > 50000 {
            financing.push(FinancingOption::Mortgage {
                down_payment_percent: 0.2,
                interest_rate: 0.06,
                term_months: 120,
            });
        }
        if asking_price > 100000 {
            financing.push(FinancingOption::Investor {
                investment_percent: 0.5,
                profit_share_percent: 0.3,
            });
        }

        // Notes based on condition and features
        let mut notes = Vec::new();
        if matches!(condition, BuildingCondition::Condemned) {
            notes.push("⚠️ Major renovation required".to_string());
        }
        if existing_tenants > 0 {
            notes.push(format!("📋 {} existing tenants with leases", existing_tenants));
        }
        if matches!(neighborhood.neighborhood_type, NeighborhoodType::Historic) {
            notes.push("🏛️ Historic preservation restrictions apply".to_string());
        }
        if neighborhood.stats.gentrification > 70 {
            notes.push("📈 Area rapidly gentrifying".to_string());
        }

        Self {
            id,
            name,
            neighborhood_id: neighborhood.id,
            condition,
            num_floors,
            units_per_floor,
            asking_price,
            existing_tenants,
            months_on_market: 0,
            available_financing: financing,
            notes,
        }
    }

    /// Convert this listing to an actual Building
    pub fn to_building(&self) -> Building {
        let mut building = Building::new(&self.name, self.num_floors, self.units_per_floor);
        
        // Set condition based on listing
        let target_condition = self.condition.starting_apartment_condition();
        for apt in &mut building.apartments {
            apt.condition = target_condition + macroquad::rand::gen_range(-10, 11);
            apt.condition = apt.condition.clamp(0, 100);
        }
        building.hallway_condition = target_condition;

        building
    }

    /// Total number of units
    pub fn total_units(&self) -> u32 {
        self.num_floors * self.units_per_floor
    }

    /// Apply time passing (price drops, etc.)
    pub fn tick(&mut self) {
        self.months_on_market += 1;
        
        // Price drops after 3 months on market
        if self.months_on_market > 3 && self.months_on_market % 2 == 0 {
            self.asking_price = (self.asking_price as f32 * 0.98) as i32;
        }
    }
}

/// Property market managing available listings
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyMarket {
    pub listings: Vec<PropertyListing>,
    next_listing_id: u32,
}

impl PropertyMarket {
    pub fn new() -> Self {
        Self {
            listings: Vec::new(),
            next_listing_id: 0,
        }
    }

    /// Generate new listings based on neighborhoods
    pub fn refresh_listings(&mut self, neighborhoods: &[Neighborhood]) {
        // Add 1-2 new listings per refresh
        let new_listings = macroquad::rand::gen_range(1, 3);
        
        for _ in 0..new_listings {
            // Pick a random neighborhood with available slots
            let available: Vec<_> = neighborhoods.iter()
                .filter(|n| n.can_add_building())
                .collect();
            
            if let Some(neighborhood) = available.first() {
                let listing = PropertyListing::generate(self.next_listing_id, neighborhood);
                self.next_listing_id += 1;
                self.listings.push(listing);
            }
        }

        // Cap total listings
        while self.listings.len() > 8 {
            self.listings.remove(0);
        }
    }

    /// Tick all listings
    pub fn tick(&mut self) {
        for listing in &mut self.listings {
            listing.tick();
        }
        
        // Remove listings that have been on market too long
        self.listings.retain(|l| l.months_on_market < 12);
    }

    /// Get listings for a specific neighborhood
    pub fn listings_for_neighborhood(&self, neighborhood_id: u32) -> Vec<&PropertyListing> {
        self.listings.iter()
            .filter(|l| l.neighborhood_id == neighborhood_id)
            .collect()
    }

    /// Remove a listing (when purchased)
    pub fn remove_listing(&mut self, listing_id: u32) {
        self.listings.retain(|l| l.id != listing_id);
    }

    /// Get a specific listing
    pub fn get_listing(&self, listing_id: u32) -> Option<&PropertyListing> {
        self.listings.iter().find(|l| l.id == listing_id)
    }
}

impl Default for PropertyMarket {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate a random building name based on neighborhood type
fn generate_building_name(neighborhood_type: &NeighborhoodType) -> String {
    use macroquad::rand::ChooseRandom;

    let prefixes: Vec<&str> = match neighborhood_type {
        NeighborhoodType::Downtown => vec!["Metro", "City", "Central", "Tower", "Urban", "Sky"],
        NeighborhoodType::Suburbs => vec!["Green", "Oak", "Maple", "Willow", "Pine", "Garden"],
        NeighborhoodType::Industrial => vec!["Brick", "Steel", "Dock", "Foundry", "Mill", "Factory"],
        NeighborhoodType::Historic => vec!["Heritage", "Colonial", "Victorian", "Classic", "Grand", "Royal"],
    };

    let suffixes: Vec<&str> = vec![
        "Apartments", "Place", "Court", "Terrace", "Manor", "House", "Arms", "Lodge",
    ];

    let prefix = prefixes.choose().unwrap_or(&"The");
    let suffix = suffixes.choose().unwrap_or(&"Building");

    format!("{} {}", prefix, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_listing_generation() {
        let neighborhood = Neighborhood::new(0, NeighborhoodType::Downtown, "Test");
        let listing = PropertyListing::generate(0, &neighborhood);
        
        assert!(listing.asking_price > 0);
        assert!(listing.num_floors >= 2);
    }

    #[test]
    fn test_financing_calculations() {
        let mortgage = FinancingOption::Mortgage {
            down_payment_percent: 0.2,
            interest_rate: 0.06,
            term_months: 120,
        };
        
        let upfront = mortgage.upfront_cost(100000);
        assert_eq!(upfront, 20000); // 20% down
        
        let monthly = mortgage.monthly_payment(100000);
        assert!(monthly > 0 && monthly < 2000); // Reasonable range
    }
}
