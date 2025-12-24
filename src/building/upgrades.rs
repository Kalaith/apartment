use super::{Apartment, Building, DesignType};

/// Costs for various upgrades
pub mod costs {
    pub const REPAIR_PER_POINT: i32 = 10;      // $10 per condition point
    pub const UPGRADE_TO_PRACTICAL: i32 = 500;
    pub const UPGRADE_TO_COZY: i32 = 1000;
    pub const SOUNDPROOFING: i32 = 300;
    pub const HALLWAY_REPAIR_PER_POINT: i32 = 15;
}

#[derive(Clone, Debug)]
pub enum UpgradeAction {
    RepairApartment { apartment_id: u32, amount: i32 },
    UpgradeDesign { apartment_id: u32 },
    AddSoundproofing { apartment_id: u32 },
    RepairHallway { amount: i32 },
}

impl UpgradeAction {
    /// Calculate the cost of this upgrade
    pub fn cost(&self, building: &Building) -> Option<i32> {
        match self {
            UpgradeAction::RepairApartment { amount, .. } => {
                Some(amount * costs::REPAIR_PER_POINT)
            }
            UpgradeAction::UpgradeDesign { apartment_id } => {
                let apt = building.get_apartment(*apartment_id)?;
                match apt.design {
                    DesignType::Bare => Some(costs::UPGRADE_TO_PRACTICAL),
                    DesignType::Practical => Some(costs::UPGRADE_TO_COZY),
                    DesignType::Cozy => None,  // Already max
                }
            }
            UpgradeAction::AddSoundproofing { apartment_id } => {
                let apt = building.get_apartment(*apartment_id)?;
                if apt.has_soundproofing {
                    None  // Already has it
                } else {
                    Some(costs::SOUNDPROOFING)
                }
            }
            UpgradeAction::RepairHallway { amount } => {
                Some(amount * costs::HALLWAY_REPAIR_PER_POINT)
            }
        }
    }
    
    /// Check if this upgrade is valid (can be performed)
    pub fn is_valid(&self, building: &Building) -> bool {
        self.cost(building).is_some()
    }
}

/// Apply an upgrade action to the building
/// Returns the cost if successful, None if failed
pub fn apply_upgrade(building: &mut Building, action: &UpgradeAction) -> Option<i32> {
    let cost = action.cost(building)?;
    
    match action {
        UpgradeAction::RepairApartment { apartment_id, amount } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.repair(*amount);
        }
        UpgradeAction::UpgradeDesign { apartment_id } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            if !apt.upgrade_design() {
                return None;
            }
        }
        UpgradeAction::AddSoundproofing { apartment_id } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.install_soundproofing();
        }
        UpgradeAction::RepairHallway { amount } => {
            building.repair_hallway(*amount);
        }
    }
    
    Some(cost)
}

/// Get available upgrades for an apartment
pub fn available_apartment_upgrades(apt: &Apartment) -> Vec<UpgradeAction> {
    let mut upgrades = Vec::new();
    
    // Repair is always available if condition < 100
    if apt.condition < 100 {
        let repair_amount = (100 - apt.condition).min(25);  // Max 25 at a time
        upgrades.push(UpgradeAction::RepairApartment {
            apartment_id: apt.id,
            amount: repair_amount,
        });
    }
    
    // Design upgrade if not at max
    if apt.design.next_upgrade().is_some() {
        upgrades.push(UpgradeAction::UpgradeDesign {
            apartment_id: apt.id,
        });
    }
    
    // Soundproofing if not installed
    if !apt.has_soundproofing {
        upgrades.push(UpgradeAction::AddSoundproofing {
            apartment_id: apt.id,
        });
    }
    
    upgrades
}

/// Get available building-wide upgrades
pub fn available_building_upgrades(building: &Building) -> Vec<UpgradeAction> {
    let mut upgrades = Vec::new();
    
    if building.hallway_condition < 100 {
        let repair_amount = (100 - building.hallway_condition).min(20);
        upgrades.push(UpgradeAction::RepairHallway {
            amount: repair_amount,
        });
    }
    
    upgrades
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::{ApartmentSize, NoiseLevel};
    
    #[test]
    fn test_upgrade_costs() {
        let building = Building::default_mvp();
        
        let repair = UpgradeAction::RepairApartment { apartment_id: 0, amount: 10 };
        assert_eq!(repair.cost(&building), Some(100));  // 10 * $10
        
        let design = UpgradeAction::UpgradeDesign { apartment_id: 0 };
        assert_eq!(design.cost(&building), Some(500));  // Bare -> Practical
    }
    
    #[test]
    fn test_apply_repair_upgrade() {
        let mut building = Building::default_mvp();
        let initial_condition = building.apartments[0].condition;
        
        let action = UpgradeAction::RepairApartment { apartment_id: 0, amount: 20 };
        let cost = apply_upgrade(&mut building, &action);
        
        assert_eq!(cost, Some(200));
        assert_eq!(building.apartments[0].condition, initial_condition + 20);
    }
    
    #[test]
    fn test_apply_design_upgrade() {
        let mut building = Building::default_mvp();
        assert_eq!(building.apartments[0].design, DesignType::Bare);
        
        let action = UpgradeAction::UpgradeDesign { apartment_id: 0 };
        let cost = apply_upgrade(&mut building, &action);
        
        assert_eq!(cost, Some(500));
        assert_eq!(building.apartments[0].design, DesignType::Practical);
    }
    
    #[test]
    fn test_apply_soundproofing() {
        let mut building = Building::default_mvp();
        assert!(!building.apartments[0].has_soundproofing);
        
        let action = UpgradeAction::AddSoundproofing { apartment_id: 0 };
        let cost = apply_upgrade(&mut building, &action);
        
        assert_eq!(cost, Some(300));
        assert!(building.apartments[0].has_soundproofing);
        
        // Second application should fail
        let cost = apply_upgrade(&mut building, &action);
        assert_eq!(cost, None);
    }
    
    #[test]
    fn test_available_apartment_upgrades() {
        let apt = Apartment::new(0, "1A", 1, ApartmentSize::Small, NoiseLevel::High);
        let upgrades = available_apartment_upgrades(&apt);
        
        // Should have repair, design upgrade, and soundproofing
        assert_eq!(upgrades.len(), 3);
    }
    
    #[test]
    fn test_available_building_upgrades() {
        let building = Building::default_mvp();
        let upgrades = available_building_upgrades(&building);
        
        // Should have hallway repair available (condition is 60, not 100)
        assert_eq!(upgrades.len(), 1);
    }
}
