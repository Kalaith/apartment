use crate::building::{UpgradeAction, Building, DesignType};
use super::{Transaction, TransactionType, PlayerFunds};

/// Centralized cost configuration
pub struct UpgradeCosts;

impl UpgradeCosts {
    /// Base cost per condition point for repairs
    pub const REPAIR_PER_POINT: i32 = 10;
    
    /// Cost to upgrade from Bare to Practical
    pub const DESIGN_BARE_TO_PRACTICAL: i32 = 500;
    
    /// Cost to upgrade from Practical to Cozy  
    pub const DESIGN_PRACTICAL_TO_COZY: i32 = 1000;
    
    /// Cost to install soundproofing
    pub const SOUNDPROOFING: i32 = 300;
    
    /// Cost per condition point for hallway repair
    pub const HALLWAY_PER_POINT: i32 = 15;
    
    /// Get cost for a repair action
    pub fn repair_cost(points: i32) -> i32 {
        points * Self::REPAIR_PER_POINT
    }
    
    /// Get cost for design upgrade
    pub fn design_upgrade_cost(from: &DesignType) -> Option<i32> {
        match from {
            DesignType::Bare => Some(Self::DESIGN_BARE_TO_PRACTICAL),
            DesignType::Practical => Some(Self::DESIGN_PRACTICAL_TO_COZY),
            DesignType::Cozy => None,
        }
    }
    
    /// Get cost for hallway repair
    pub fn hallway_repair_cost(points: i32) -> i32 {
        points * Self::HALLWAY_PER_POINT
    }
}

/// Process an upgrade and deduct funds
/// Returns Ok(cost) if successful, Err(reason) if failed
pub fn process_upgrade(
    action: &UpgradeAction,
    building: &mut Building,
    funds: &mut PlayerFunds,
    current_tick: u32,
) -> Result<i32, String> {
    // Calculate cost
    let cost = match action {
        UpgradeAction::RepairApartment { apartment_id, amount } => {
            let apt = building.get_apartment(*apartment_id)
                .ok_or("Apartment not found")?;
            if apt.condition >= 100 {
                return Err("Apartment already at max condition".to_string());
            }
            UpgradeCosts::repair_cost(*amount)
        }
        UpgradeAction::UpgradeDesign { apartment_id } => {
            let apt = building.get_apartment(*apartment_id)
                .ok_or("Apartment not found")?;
            UpgradeCosts::design_upgrade_cost(&apt.design)
                .ok_or("Already at max design level")?
        }
        UpgradeAction::AddSoundproofing { apartment_id } => {
            let apt = building.get_apartment(*apartment_id)
                .ok_or("Apartment not found")?;
            if apt.has_soundproofing {
                return Err("Already has soundproofing".to_string());
            }
            UpgradeCosts::SOUNDPROOFING
        }
        UpgradeAction::RepairHallway { amount } => {
            if building.hallway_condition >= 100 {
                return Err("Hallway already at max condition".to_string());
            }
            UpgradeCosts::hallway_repair_cost(*amount)
        }
    };
    
    // Check funds
    if !funds.can_afford(cost) {
        return Err(format!("Insufficient funds (need ${}, have ${})", cost, funds.balance));
    }
    
    // Create transaction
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
        UpgradeAction::AddSoundproofing { apartment_id } => {
            let unit = building.get_apartment(*apartment_id)
                .map(|a| a.unit_number.clone())
                .unwrap_or_default();
            format!("Soundproofing Unit {}", unit)
        }
        UpgradeAction::RepairHallway { amount } => {
            format!("Hallway repair (+{} condition)", amount)
        }
    };
    
    let transaction = Transaction::expense(
        match action {
            UpgradeAction::RepairApartment { .. } => TransactionType::RepairCost,
            UpgradeAction::UpgradeDesign { .. } => TransactionType::UpgradeCost,
            UpgradeAction::AddSoundproofing { .. } => TransactionType::UpgradeCost,
            UpgradeAction::RepairHallway { .. } => TransactionType::HallwayRepair,
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
    crate::building::apply_upgrade(building, action)
        .ok_or("Failed to apply upgrade")?;
    
    Ok(cost)
}
