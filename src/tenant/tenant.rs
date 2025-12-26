
use serde::{Deserialize, Serialize};
use macroquad::rand::{ChooseRandom, gen_range};
use super::TenantArchetype;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tenant {
    pub id: u32,
    pub name: String,
    pub archetype: TenantArchetype,
    
    // Current state
    pub happiness: i32,         // 0-100
    pub months_residing: u32,   // How long they've lived here
    pub apartment_id: Option<u32>,
    
    // Tolerances (derived from archetype but can vary slightly)
    pub rent_tolerance: i32,    // Max rent they'll accept
    pub noise_tolerance: i32,   // 0-100, higher = more tolerant
}

impl Tenant {
    pub fn new(id: u32, name: &str, archetype: TenantArchetype) -> Self {
        let prefs = archetype.preferences();
        
        Self {
            id,
            name: name.to_string(),
            archetype,
            happiness: 70,  // Start reasonably happy
            months_residing: 0,
            apartment_id: None,
            rent_tolerance: prefs.ideal_rent_max,
            noise_tolerance: if prefs.prefers_quiet { 30 } else { 70 },
        }
    }
    
    /// Create a tenant with some randomization
    pub fn generate(id: u32, archetype: TenantArchetype) -> Self {
        let name = generate_random_name(&archetype);
        let mut tenant = Self::new(id, &name, archetype);
        
        // Add some variance to tolerances (±15%)
        let variance = 0.15;
        let rent_var = (tenant.rent_tolerance as f32 * variance) as i32;
        tenant.rent_tolerance += gen_range(-rent_var, rent_var);
        
        let noise_var = (tenant.noise_tolerance as f32 * variance) as i32;
        tenant.noise_tolerance = (tenant.noise_tolerance + gen_range(-noise_var, noise_var)).clamp(0, 100);
        
        tenant
    }
    
    /// Check if tenant is at risk of leaving
    pub fn is_unhappy(&self) -> bool {
        self.happiness < 30
    }
    
    /// Check if tenant will leave this tick
    pub fn will_leave(&self) -> bool {
        self.happiness == 0
    }
    
    /// Update happiness (called each tick)
    pub fn set_happiness(&mut self, new_happiness: i32) {
        self.happiness = new_happiness.clamp(0, 100);
    }
    
    /// Increment months residing
    pub fn add_month(&mut self) {
        self.months_residing += 1;
    }
    
    /// Move into an apartment
    pub fn move_into(&mut self, apartment_id: u32) {
        self.apartment_id = Some(apartment_id);
        self.months_residing = 0;
    }
    
    /// Move out of current apartment
    pub fn move_out(&mut self) {
        self.apartment_id = None;
    }
}

/// Generate a random name appropriate for the archetype
fn generate_random_name(archetype: &TenantArchetype) -> String {
    let first_names = match archetype {
        TenantArchetype::Student => vec![
            "Alex", "Jordan", "Casey", "Riley", "Morgan",
            "Sam", "Taylor", "Jamie", "Quinn", "Avery"
        ],
        TenantArchetype::Professional => vec![
            "Michael", "Sarah", "David", "Jennifer", "Robert",
            "Lisa", "James", "Amanda", "William", "Elizabeth"
        ],
        TenantArchetype::Artist => vec![
            "Luna", "River", "Sky", "Echo", "Rain", "Ash",
            "Sage", "Raven", "Nova", "Orion", "Willow", "Jasper"
        ],
        TenantArchetype::Family => vec![
            "The Smiths", "The Garcias", "The Kims", "The Patels",
            "The Joneses", "The Wangs", "The Johnsons", "The Millers"
        ],
        TenantArchetype::Elderly => vec![
            "Mrs. Higgins", "Mr. Abernathy", "Betty", "Harold",
            "Martha", "Walter", "Ethel", "Arthur"
        ]
    };
    
    let last_initials = vec!["A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "L", "M", "N", "P", "R", "S", "T", "W"];
    
    let first = first_names.choose().unwrap_or(&"Pat");
    let last = last_initials.choose().unwrap_or(&"X");
    
    format!("{} {}.", first, last)
}
