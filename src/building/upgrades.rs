
use super::{Apartment, Building, DesignType};
use serde::{Deserialize, Serialize};
use crate::data::config::{EconomyConfig, UiConfig, UpgradeDefinition, UpgradeTarget, UpgradeRequirement};
use std::collections::HashMap;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UpgradeAction {
    RepairApartment { apartment_id: u32, amount: i32 },
    UpgradeDesign { apartment_id: u32 },
    RepairHallway { amount: i32 },
    // Generic upgrade identified by ID (from config.json)
    Apply { upgrade_id: String, target_id: Option<u32> }, 
}

impl UpgradeAction {
    /// Get a user-friendly label for this action
    pub fn label(&self, building: &Building, config: &UiConfig, upgrades: &HashMap<String, UpgradeDefinition>) -> String {
        match self {
            UpgradeAction::RepairApartment { amount, .. } => {
                let fmt = config.upgrade_labels.get("repair_fmt").map(|s| s.as_str()).unwrap_or("Repair +{}");
                fmt.replace("{}", &amount.to_string())
            }
            UpgradeAction::UpgradeDesign { apartment_id } => {
                if let Some(apt) = building.get_apartment(*apartment_id) {
                    if let Some(next) = apt.design.next_upgrade() {
                         let fmt = config.upgrade_labels.get("upgrade_design_fmt").map(|s| s.as_str()).unwrap_or("Upgrade to {}");
                         return fmt.replace("{}", &format!("{:?}", next));
                    }
                }
                config.upgrade_labels.get("max_design").cloned().unwrap_or_else(|| "Max Design".to_string())
            }
            UpgradeAction::RepairHallway { amount } => {
                 let fmt = config.upgrade_labels.get("repair_hallway_fmt").map(|s| s.as_str()).unwrap_or("Repair Hallway +{}");
                 fmt.replace("{}", &amount.to_string())
            }
            UpgradeAction::Apply { upgrade_id, .. } => {
                upgrades.get(upgrade_id).map(|u| u.name.clone()).unwrap_or_else(|| upgrade_id.clone())
            }
        }
    }

    /// Calculate the cost of this action
    pub fn cost(&self, building: &Building, config: &EconomyConfig, upgrades: &HashMap<String, UpgradeDefinition>) -> Option<i32> {
        match self {
            UpgradeAction::RepairApartment { amount, ..} => {
                Some(amount * config.repair_cost_per_point)
            }
            UpgradeAction::UpgradeDesign { apartment_id } => {
                let apt = building.get_apartment(*apartment_id)?;
                let current_design_key = match apt.design {
                    DesignType::Bare => "bare_to_practical",
                    DesignType::Practical => "practical_to_cozy",
                    DesignType::Cozy => return None,
                };
                config.design_upgrade_costs.get(current_design_key).cloned()
            }
            UpgradeAction::RepairHallway { amount } => {
                Some(amount * config.hallway_repair_cost_per_point)
            }
            UpgradeAction::Apply { upgrade_id, .. } => {
                upgrades.get(upgrade_id).map(|u| u.cost)
            }
        }
    }
}

/// Apply an upgrade action to the building
/// Returns the cost if successful, None if failed
pub fn apply_upgrade(building: &mut Building, action: &UpgradeAction, upgrades: &HashMap<String, UpgradeDefinition>) -> Option<()> {
    match action {
        UpgradeAction::RepairApartment { apartment_id, amount } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.repair(*amount);
            Some(())
        }
        UpgradeAction::UpgradeDesign { apartment_id } => {
            let apt = building.get_apartment_mut(*apartment_id)?;
            apt.upgrade_design();
            Some(())
        }
        UpgradeAction::RepairHallway { amount } => {
            building.repair_hallway(*amount);
            Some(())
        }
        UpgradeAction::Apply { upgrade_id, target_id } => {
            let def = upgrades.get(upgrade_id)?;
            match def.target {
                UpgradeTarget::Apartment => {
                    let apt_id = (*target_id)?;
                    let apt = building.get_apartment_mut(apt_id)?;
                    
                    // Apply generic effects
                    for effect in &def.effects {
                        match effect {
                            crate::data::config::UpgradeEffect::SetFlag(flag) => {
                                apt.flags.insert(flag.clone());
                                if flag == "has_soundproofing" { apt.has_soundproofing = true; }
                                if flag == "has_renovated_kitchen" && apt.kitchen_level < 1 { apt.kitchen_level = 1; }
                            }
                            crate::data::config::UpgradeEffect::RemoveFlag(flag) => {
                                apt.flags.remove(flag);
                            }
                            _ => {}
                        }
                    }
                    Some(())
                }
                UpgradeTarget::Building => {
                     for effect in &def.effects {
                        match effect {
                            crate::data::config::UpgradeEffect::SetFlag(flag) => {
                                building.flags.insert(flag.clone());
                            }
                            crate::data::config::UpgradeEffect::RemoveFlag(flag) => {
                                building.flags.remove(flag);
                            }
                            _ => {}
                        }
                    }
                    Some(())
                }
            }
        }
    }
}

pub fn available_apartment_upgrades(apt: &Apartment, upgrades: &HashMap<String, UpgradeDefinition>) -> Vec<UpgradeAction> {
    let mut actions = Vec::new();

    // 1. Repair (hardcoded logic for now as it's variable amount)
    if apt.condition < 100 {
        let amount = (100 - apt.condition).min(10); 
        actions.push(UpgradeAction::RepairApartment { 
            apartment_id: apt.id, 
            amount 
        });
    }

    // 2. Design (hardcoded logic)
    if apt.design.next_upgrade().is_some() {
        actions.push(UpgradeAction::UpgradeDesign { apartment_id: apt.id });
    }

    // 3. Generic Upgrades
    for (id, def) in upgrades {
        if def.target == UpgradeTarget::Apartment {
            if check_requirements(&def.requirements, apt, None) {
                actions.push(UpgradeAction::Apply { 
                    upgrade_id: id.clone(), 
                    target_id: Some(apt.id) 
                });
            }
        }
    }

    actions
}

pub fn available_building_upgrades(building: &Building, upgrades: &HashMap<String, UpgradeDefinition>) -> Vec<UpgradeAction> {
    let mut actions = Vec::new();

    // 1. Repair Hallway
    if building.hallway_condition < 100 {
         let amount = (100 - building.hallway_condition).min(10);
         actions.push(UpgradeAction::RepairHallway { amount });
    }

    // 2. Generic Upgrades
    for (id, def) in upgrades {
        if def.target == UpgradeTarget::Building {
            if check_requirements_building(&def.requirements, building) {
                 actions.push(UpgradeAction::Apply { 
                    upgrade_id: id.clone(), 
                    target_id: None 
                });
            }
        }
    }

    actions
}

fn check_requirements(reqs: &[UpgradeRequirement], apt: &Apartment, _building: Option<&Building>) -> bool {
    for req in reqs {
        match req {
            UpgradeRequirement::MissingFlag(flag) => {
                // Check new generic flags AND backwards compatibility hardcoded fields
                if apt.flags.contains(flag) { return false; }
                if flag == "has_soundproofing" && apt.has_soundproofing { return false; }
                if flag == "has_renovated_kitchen" && apt.kitchen_level >= 2 { return false; }
            }
            UpgradeRequirement::HasFlag(flag) => {
                let has = apt.flags.contains(flag)
                    || (flag == "has_soundproofing" && apt.has_soundproofing)
                    || (flag == "has_renovated_kitchen" && apt.kitchen_level >= 2);
                if !has { return false; }
            }
             _ => {} // Implement generic stat checks later if needed
        }
    }
    true
}

fn check_requirements_building(reqs: &[UpgradeRequirement], building: &Building) -> bool {
    for req in reqs {
        match req {
            UpgradeRequirement::MissingFlag(flag) => {
                 if building.flags.contains(flag) { return false; }
                 if flag == "has_laundry" && building.has_laundry { return false; }
            }
            UpgradeRequirement::HasFlag(flag) => {
                let has = building.flags.contains(flag)
                    || (flag == "has_laundry" && building.has_laundry);
                if !has { return false; }
            }
            _ => {}
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::{ApartmentSize, NoiseLevel};
    use crate::data::config::GameConfig;
    
    #[test]
    fn test_upgrade_costs() {
        let building = Building::default_mvp();
        let config = GameConfig::default().economy;
        let upgrades = GameConfig::default().upgrades;
        
        let repair = UpgradeAction::RepairApartment { apartment_id: 0, amount: 10 };
        assert_eq!(repair.cost(&building, &config, &upgrades), Some(100));  // 10 * $10
        
        let design = UpgradeAction::UpgradeDesign { apartment_id: 0 };
        assert_eq!(design.cost(&building, &config, &upgrades), Some(500));  // Bare -> Practical
    }
    
    #[test]
    fn test_apply_repair_upgrade() {
        let mut building = Building::default_mvp();
        let config = GameConfig::default().economy;
        let upgrades = GameConfig::default().upgrades;
        let initial_condition = building.apartments[0].condition;
        
        let action = UpgradeAction::RepairApartment { apartment_id: 0, amount: 20 };
        let cost = apply_upgrade(&mut building, &action, &upgrades);
        
        assert_eq!(cost, Some(())); // Returns Option<()>
        assert_eq!(building.apartments[0].condition, initial_condition + 20);
    }
}
