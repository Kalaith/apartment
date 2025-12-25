use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DesignType {
    Bare,
    Practical,
    Cozy,
}

impl DesignType {
    /// Returns the next design upgrade level, if available
    pub fn next_upgrade(&self) -> Option<DesignType> {
        match self {
            DesignType::Bare => Some(DesignType::Practical),
            DesignType::Practical => Some(DesignType::Cozy),
            DesignType::Cozy => None, // Max level
        }
    }
    
    /// Design appeal score (affects tenant happiness)
    pub fn appeal_score(&self) -> i32 {
        match self {
            DesignType::Bare => 0,
            DesignType::Practical => 20,
            DesignType::Cozy => 40,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ApartmentSize {
    Small,
    Medium,
}

impl ApartmentSize {
    pub fn base_rent(&self) -> i32 {
        match self {
            ApartmentSize::Small => 600,
            ApartmentSize::Medium => 900,
        }
    }
    
    pub fn space_score(&self) -> i32 {
        match self {
            ApartmentSize::Small => 0,
            ApartmentSize::Medium => 15,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NoiseLevel {
    Low,
    High,
}

impl NoiseLevel {
    pub fn noise_penalty(&self) -> i32 {
        match self {
            NoiseLevel::Low => 0,
            NoiseLevel::High => -20,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Apartment {
    pub id: u32,
    pub unit_number: String,        // e.g., "1A", "2B"
    pub floor: u32,
    
    // Core stats
    pub condition: i32,             // 0-100
    pub design: DesignType,
    pub size: ApartmentSize,
    pub base_noise: NoiseLevel,     // Inherent noise (street-facing, etc.)
    pub has_soundproofing: bool,
    pub kitchen_level: i32,         // 0=Basic, 1=Renovated, 2=Luxury
    pub rent_price: i32,
    
    // Occupancy
    pub tenant_id: Option<u32>,
}

impl Apartment {
    pub fn new(id: u32, unit_number: &str, floor: u32, size: ApartmentSize, base_noise: NoiseLevel) -> Self {
        let rent_price = size.base_rent();
        Self {
            id,
            unit_number: unit_number.to_string(),
            floor,
            condition: 50,  // Start at half condition
            design: DesignType::Bare,
            size,
            base_noise,
            has_soundproofing: false,
            kitchen_level: 0,
            rent_price,
            tenant_id: None,
        }
    }
    
    /// Current effective noise level (considers soundproofing)
    pub fn effective_noise(&self) -> NoiseLevel {
        if self.has_soundproofing {
            NoiseLevel::Low
        } else {
            self.base_noise.clone()
        }
    }
    
    /// Is the apartment currently vacant?
    pub fn is_vacant(&self) -> bool {
        self.tenant_id.is_none()
    }
    
    /// Calculate overall apartment quality score (0-100)
    pub fn quality_score(&self) -> i32 {
        let base = self.condition;
        let design_bonus = self.design.appeal_score();
        let noise_mod = self.effective_noise().noise_penalty();
        let space_bonus = self.size.space_score();
        let kitchen_bonus = self.kitchen_level * 15;
        
        (base + design_bonus + noise_mod + space_bonus + kitchen_bonus).clamp(0, 100)
    }
    
    /// Apply condition decay (called each tick)
    pub fn decay_condition(&mut self, amount: i32) {
        self.condition = (self.condition - amount).max(0);
    }
    
    /// Repair the apartment
    pub fn repair(&mut self, amount: i32) {
        self.condition = (self.condition + amount).min(100);
    }
    
    /// Upgrade design to next level
    pub fn upgrade_design(&mut self) -> bool {
        if let Some(next) = self.design.next_upgrade() {
            self.design = next;
            true
        } else {
            false
        }
    }
    
    /// Upgrade kitchen
    pub fn upgrade_kitchen(&mut self) -> bool {
        if self.kitchen_level < 2 {
            self.kitchen_level += 1;
            true
        } else {
            false
        }
    }
    
    /// Install soundproofing
    pub fn install_soundproofing(&mut self) {
        self.has_soundproofing = true;
    }
    
    /// Move a tenant in
    pub fn move_in(&mut self, tenant_id: u32) {
        self.tenant_id = Some(tenant_id);
    }
    
    /// Move tenant out
    pub fn move_out(&mut self) {
        self.tenant_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_apartment_quality_score() {
        let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        apt.condition = 50;
        apt.design = DesignType::Bare;
        assert_eq!(apt.quality_score(), 50);  // 50 condition, no bonuses
        
        apt.design = DesignType::Cozy;
        assert_eq!(apt.quality_score(), 90);  // 50 + 40 design
    }
    
    #[test]
    fn test_soundproofing_effect() {
        let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::High);
        assert_eq!(apt.effective_noise(), NoiseLevel::High);
        
        apt.install_soundproofing();
        assert_eq!(apt.effective_noise(), NoiseLevel::Low);
    }
    
    #[test]
    fn test_design_upgrade() {
        let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        assert_eq!(apt.design, DesignType::Bare);
        
        assert!(apt.upgrade_design());
        assert_eq!(apt.design, DesignType::Practical);
        
        assert!(apt.upgrade_design());
        assert_eq!(apt.design, DesignType::Cozy);
        
        assert!(!apt.upgrade_design());  // Already at max
        assert_eq!(apt.design, DesignType::Cozy);
    }
    
    #[test]
    fn test_condition_decay_and_repair() {
        let mut apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::Low);
        apt.condition = 50;
        
        apt.decay_condition(10);
        assert_eq!(apt.condition, 40);
        
        apt.repair(25);
        assert_eq!(apt.condition, 65);
        
        apt.repair(100);  // Should clamp to 100
        assert_eq!(apt.condition, 100);
        
        apt.condition = 5;
        apt.decay_condition(10);  // Should clamp to 0
        assert_eq!(apt.condition, 0);
    }
}
