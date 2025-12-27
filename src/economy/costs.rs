use crate::building::{UpgradeAction, Building, apply_upgrade};
use crate::data::config::OperatingCostsConfig;
use super::{Transaction, TransactionType, PlayerFunds};


/// Calculate operating costs
pub struct OperatingCosts;

impl OperatingCosts {
    /// Calculate monthly property tax based on building value/income
    pub fn calculate_property_tax(_building: &Building, rent_income: i32, config: &OperatingCostsConfig) -> i32 {
        (rent_income as f32 * config.property_tax_rate) as i32
    }
    
    /// Calculate monthly utilities
    pub fn calculate_utilities(building: &Building, config: &OperatingCostsConfig) -> i32 {
        if !building.utilities_included {
            return 0;
        }
        
        // Base cost per occupied unit
        let occupied = building.occupancy_count() as i32;
        occupied * config.utility_cost_per_unit
    }
    
    /// Calculate monthly insurance
    pub fn calculate_insurance(building: &Building, config: &OperatingCostsConfig) -> i32 {
        if !building.insurance_active {
            return 0;
        }
        
        // Discount for good condition
        let discount = if building.hallway_condition > config.insurance_good_condition_threshold { 
            config.insurance_good_condition_discount 
        } else { 
            0 
        };
        
        config.insurance_base_rate - discount
    }
    
    /// Calculate monthly staff salaries
    pub fn calculate_staff_salaries(building: &Building, config: &crate::data::config::EconomyConfig) -> i32 {
        let mut total = 0;
        
        for (staff_type, cost) in &config.staff_costs {
            let flag = format!("staff_{}", staff_type);
            if building.flags.contains(&flag) {
                total += cost;
            }
        }
        
        total
    }
}

/// Process an upgrade and deduct funds
/// Returns Ok(cost) if successful, Err(reason) if failed
pub fn process_upgrade(
    action: &UpgradeAction,
    building: &mut Building,
    funds: &mut PlayerFunds,
    config: &crate::data::config::GameConfig,
    current_tick: u32,
) -> Result<i32, String> {
    // Calculate cost using central logic
    let cost = action.cost(building, &config.economy, &config.upgrades).ok_or("Invalid upgrade")?;
    
    // Additional Validation
    match action {
        UpgradeAction::RepairApartment { apartment_id, .. } => {
             let apt = building.get_apartment(*apartment_id).ok_or("Apartment not found")?;
              if apt.condition >= 100 {
                return Err("Apartment already at max condition".to_string());
            }
        }
        UpgradeAction::UpgradeDesign { apartment_id } => {
             building.get_apartment(*apartment_id).ok_or("Apartment not found")?;
        }
        UpgradeAction::RepairHallway { .. } => {
             if building.hallway_condition >= 100 {
                return Err("Hallway already at max condition".to_string());
            }
        }
        UpgradeAction::Apply { upgrade_id, target_id } => {
             let def = config.upgrades.get(upgrade_id).ok_or("Unknown upgrade")?;
             
             // Validate requirements
             match def.target {
                crate::data::config::UpgradeTarget::Apartment => {
                    let apt_id = target_id.ok_or("Missing apartment ID")?;
                    let apt = building.get_apartment(apt_id).ok_or("Apartment not found")?;
                     
                    // Verify requirements again (safety check)
                    for req in &def.requirements {
                        match req {
                            crate::data::config::UpgradeRequirement::MissingFlag(flag) => {
                                if apt.flags.contains(flag) || 
                                   (flag == "has_soundproofing" && apt.has_soundproofing) ||
                                   (flag == "has_renovated_kitchen" && apt.kitchen_level >= 2) {
                                    return Err(format!("Requirement failed: {}", flag));
                                }
                            }
                            crate::data::config::UpgradeRequirement::HasFlag(flag) => {
                                let has = apt.flags.contains(flag) ||
                                          (flag == "has_soundproofing" && apt.has_soundproofing) ||
                                          (flag == "has_renovated_kitchen" && apt.kitchen_level >= 2);
                                if !has { return Err(format!("Missing requirement: {}", flag)); }
                            }
                             _ => {}
                        }
                    }
                }
                crate::data::config::UpgradeTarget::Building => {
                     for req in &def.requirements {
                        match req {
                             crate::data::config::UpgradeRequirement::MissingFlag(flag) => {
                                if building.flags.contains(flag) || 
                                   (flag == "has_laundry" && building.has_laundry) {
                                     return Err(format!("Requirement failed: {}", flag));
                                }
                             }
                             // ... check other reqs
                             _ => {}
                        }
                     }
                }
             }
        }
    };

    
    // Check funds
    if !funds.can_afford(cost) {
        return Err(format!("Insufficient funds (need ${}, have ${})", cost, funds.balance));
    }
    
    // Create transaction description
    let description = match action {
        UpgradeAction::RepairApartment { apartment_id, amount } => {
            let unit = building.get_apartment(*apartment_id)
                .map(|a| a.unit_number.clone())
                .unwrap_or_default();
            format!("Repair Unit {} (+{} condition)", unit, amount)
        }
        UpgradeAction::UpgradeDesign { apartment_id } => {
            let apt = building.get_apartment(*apartment_id).unwrap();
            let unit = apt.unit_number.clone();
            let to_design = apt.design.next_upgrade().unwrap();
            format!("Upgrade Unit {} to {:?}", unit, to_design)
        }
        UpgradeAction::RepairHallway { amount } => {
            format!("Hallway repair (+{} condition)", amount)
        }
        UpgradeAction::Apply { upgrade_id, target_id } => {
            let name = config.upgrades.get(upgrade_id).map(|u| u.name.clone()).unwrap_or_else(|| "Upgrade".to_string());
            if let Some(apt_id) = target_id {
                 let unit = building.get_apartment(*apt_id)
                    .map(|a| a.unit_number.clone())
                    .unwrap_or_default();
                 format!("{} (Unit {})", name, unit)
            } else {
                 name
            }
        }
    };
    
    let transaction = Transaction::expense(
        match action {
            UpgradeAction::RepairApartment { .. } => TransactionType::RepairCost,
            UpgradeAction::UpgradeDesign { .. } => TransactionType::UpgradeCost,
            UpgradeAction::RepairHallway { .. } => TransactionType::HallwayRepair,
            UpgradeAction::Apply { .. } => TransactionType::UpgradeCost,
        },
        cost,
        &description,
        current_tick,
    );
    
    // Deduct funds
    if !funds.deduct_expense(transaction) {
        return Err("Failed to deduct funds".to_string());
    }
    
    // Apply the upgrade
    apply_upgrade(building, action, &config.upgrades)
        .ok_or("Failed to apply upgrade")?;
    
    Ok(cost)
}
