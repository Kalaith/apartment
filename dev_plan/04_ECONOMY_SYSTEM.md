# Task 04: Economy System

## Priority: 🟢 MEDIUM
## Dependencies: Task 02 (Building System), Task 03 (Tenant System)
## Estimated Effort: 1-2 hours
## Can Parallel With: Task 05

---

## Objective
Implement the money system including rent collection, repair/upgrade costs, and financial state tracking.

---

## Deliverables

### 1. src/economy/mod.rs

```rust
mod money;
mod rent;
mod costs;
mod ledger;

pub use money::{PlayerFunds, Transaction, TransactionType};
pub use rent::collect_rent;
pub use costs::UpgradeCosts;
pub use ledger::{FinancialLedger, MonthlyReport};
```

### 2. src/economy/money.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    RentIncome,
    RepairCost,
    UpgradeCost,
    HallwayRepair,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub transaction_type: TransactionType,
    pub amount: i32,  // Positive = income, negative = expense
    pub description: String,
    pub tick: u32,
}

impl Transaction {
    pub fn income(transaction_type: TransactionType, amount: i32, description: &str, tick: u32) -> Self {
        Self {
            transaction_type,
            amount: amount.abs(),  // Ensure positive
            description: description.to_string(),
            tick,
        }
    }
    
    pub fn expense(transaction_type: TransactionType, amount: i32, description: &str, tick: u32) -> Self {
        Self {
            transaction_type,
            amount: -amount.abs(),  // Ensure negative
            description: description.to_string(),
            tick,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerFunds {
    pub balance: i32,
    pub total_income: i32,
    pub total_expenses: i32,
    pub transactions: Vec<Transaction>,
}

impl PlayerFunds {
    pub fn new(starting_balance: i32) -> Self {
        Self {
            balance: starting_balance,
            total_income: 0,
            total_expenses: 0,
            transactions: Vec::new(),
        }
    }
    
    /// Check if player can afford an expense
    pub fn can_afford(&self, cost: i32) -> bool {
        self.balance >= cost
    }
    
    /// Add income to balance
    pub fn add_income(&mut self, transaction: Transaction) {
        let amount = transaction.amount.abs();
        self.balance += amount;
        self.total_income += amount;
        self.transactions.push(transaction);
    }
    
    /// Deduct expense from balance (returns false if insufficient funds)
    pub fn deduct_expense(&mut self, transaction: Transaction) -> bool {
        let cost = transaction.amount.abs();
        if self.balance < cost {
            return false;
        }
        
        self.balance -= cost;
        self.total_expenses += cost;
        self.transactions.push(transaction);
        true
    }
    
    /// Check if player is bankrupt
    pub fn is_bankrupt(&self) -> bool {
        self.balance < 0
    }
    
    /// Get net profit/loss
    pub fn net_profit(&self) -> i32 {
        self.total_income - self.total_expenses
    }
    
    /// Get transactions for a specific tick
    pub fn transactions_for_tick(&self, tick: u32) -> Vec<&Transaction> {
        self.transactions.iter().filter(|t| t.tick == tick).collect()
    }
    
    /// Get recent transactions (last N)
    pub fn recent_transactions(&self, count: usize) -> Vec<&Transaction> {
        self.transactions.iter().rev().take(count).collect()
    }
}

impl Default for PlayerFunds {
    fn default() -> Self {
        Self::new(5000)  // Default starting funds
    }
}
```

### 3. src/economy/rent.rs

```rust
use crate::building::Building;
use crate::tenant::Tenant;
use super::{Transaction, TransactionType, PlayerFunds};

/// Result of rent collection for one tick
#[derive(Clone, Debug)]
pub struct RentCollection {
    pub total_collected: i32,
    pub payments: Vec<RentPayment>,
    pub missed_payments: Vec<MissedPayment>,
}

#[derive(Clone, Debug)]
pub struct RentPayment {
    pub tenant_name: String,
    pub apartment_unit: String,
    pub amount: i32,
}

#[derive(Clone, Debug)]
pub struct MissedPayment {
    pub tenant_name: String,
    pub apartment_unit: String,
    pub reason: String,
}

/// Collect rent from all tenants
pub fn collect_rent(
    tenants: &[Tenant],
    building: &Building,
    funds: &mut PlayerFunds,
    current_tick: u32,
) -> RentCollection {
    let mut collection = RentCollection {
        total_collected: 0,
        payments: Vec::new(),
        missed_payments: Vec::new(),
    };
    
    for tenant in tenants {
        if let Some(apt_id) = tenant.apartment_id {
            if let Some(apartment) = building.get_apartment(apt_id) {
                // Very unhappy tenants might miss payment
                if tenant.happiness < 20 && rand::gen_range(0, 100) < 30 {
                    collection.missed_payments.push(MissedPayment {
                        tenant_name: tenant.name.clone(),
                        apartment_unit: apartment.unit_number.clone(),
                        reason: "Tenant too unhappy".to_string(),
                    });
                    continue;
                }
                
                let rent = apartment.rent_price;
                
                funds.add_income(Transaction::income(
                    TransactionType::RentIncome,
                    rent,
                    &format!("Rent from {} (Unit {})", tenant.name, apartment.unit_number),
                    current_tick,
                ));
                
                collection.payments.push(RentPayment {
                    tenant_name: tenant.name.clone(),
                    apartment_unit: apartment.unit_number.clone(),
                    amount: rent,
                });
                
                collection.total_collected += rent;
            }
        }
    }
    
    collection
}

/// Calculate expected monthly rent (for projections)
pub fn calculate_expected_rent(tenants: &[Tenant], building: &Building) -> i32 {
    tenants.iter()
        .filter_map(|t| t.apartment_id)
        .filter_map(|apt_id| building.get_apartment(apt_id))
        .map(|apt| apt.rent_price)
        .sum()
}

/// Calculate potential rent if fully occupied
pub fn calculate_max_potential_rent(building: &Building) -> i32 {
    building.apartments.iter().map(|apt| apt.rent_price).sum()
}

/// Calculate occupancy rate as percentage
pub fn calculate_occupancy_rate(building: &Building) -> f32 {
    let total = building.apartments.len() as f32;
    if total == 0.0 {
        return 0.0;
    }
    let occupied = building.occupancy_count() as f32;
    (occupied / total) * 100.0
}
```

### 4. src/economy/costs.rs

```rust
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
```

### 5. src/economy/ledger.rs

```rust
use super::{Transaction, TransactionType};

/// Monthly financial summary
#[derive(Clone, Debug)]
pub struct MonthlyReport {
    pub tick: u32,
    pub rent_income: i32,
    pub repair_costs: i32,
    pub upgrade_costs: i32,
    pub net: i32,
    pub ending_balance: i32,
}

/// Financial tracking across the game
#[derive(Clone, Debug)]
pub struct FinancialLedger {
    pub reports: Vec<MonthlyReport>,
}

impl FinancialLedger {
    pub fn new() -> Self {
        Self {
            reports: Vec::new(),
        }
    }
    
    /// Generate a monthly report from transactions
    pub fn generate_report(
        &mut self,
        tick: u32,
        transactions: &[&Transaction],
        ending_balance: i32,
    ) -> MonthlyReport {
        let mut rent_income = 0;
        let mut repair_costs = 0;
        let mut upgrade_costs = 0;
        
        for t in transactions {
            match t.transaction_type {
                TransactionType::RentIncome => rent_income += t.amount.abs(),
                TransactionType::RepairCost | TransactionType::HallwayRepair => {
                    repair_costs += t.amount.abs();
                }
                TransactionType::UpgradeCost => upgrade_costs += t.amount.abs(),
            }
        }
        
        let report = MonthlyReport {
            tick,
            rent_income,
            repair_costs,
            upgrade_costs,
            net: rent_income - repair_costs - upgrade_costs,
            ending_balance,
        };
        
        self.reports.push(report.clone());
        report
    }
    
    /// Get average monthly income
    pub fn average_monthly_income(&self) -> f32 {
        if self.reports.is_empty() {
            return 0.0;
        }
        let total: i32 = self.reports.iter().map(|r| r.rent_income).sum();
        total as f32 / self.reports.len() as f32
    }
    
    /// Get average monthly expenses
    pub fn average_monthly_expenses(&self) -> f32 {
        if self.reports.is_empty() {
            return 0.0;
        }
        let total: i32 = self.reports.iter()
            .map(|r| r.repair_costs + r.upgrade_costs)
            .sum();
        total as f32 / self.reports.len() as f32
    }
    
    /// Get total profit across all months
    pub fn total_profit(&self) -> i32 {
        self.reports.iter().map(|r| r.net).sum()
    }
    
    /// Check if finances are trending positive
    pub fn is_profitable(&self) -> bool {
        if self.reports.len() < 3 {
            return true;  // Not enough data
        }
        
        // Check last 3 months
        let recent: Vec<_> = self.reports.iter().rev().take(3).collect();
        let avg_net: i32 = recent.iter().map(|r| r.net).sum::<i32>() / 3;
        avg_net > 0
    }
}

impl Default for FinancialLedger {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Integration Points

### With GameplayState (Task 01)
```rust
// In src/state/gameplay.rs:
use crate::economy::{PlayerFunds, FinancialLedger};

pub struct GameplayState {
    pub building: Building,
    pub tenants: Vec<Tenant>,
    pub funds: PlayerFunds,
    pub ledger: FinancialLedger,
    // ...
}
```

### With Building System (Task 02)
- Uses `UpgradeAction` for cost calculation
- Calls `apply_upgrade()` after payment

### With Simulation (Task 05)
- `collect_rent()` called each tick
- `generate_report()` called at end of each tick

---

## Acceptance Criteria

- [ ] Starting funds of $5000
- [ ] Rent collected from all tenants each tick
- [ ] Unhappy tenants occasionally miss rent
- [ ] Upgrade costs deducted when applied
- [ ] Insufficient funds blocks upgrades
- [ ] Transaction history tracked
- [ ] Monthly reports generated
- [ ] Bankruptcy detected at negative balance

---

## Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_starting_funds() {
        let funds = PlayerFunds::default();
        assert_eq!(funds.balance, 5000);
    }
    
    #[test]
    fn test_can_afford() {
        let funds = PlayerFunds::new(1000);
        assert!(funds.can_afford(500));
        assert!(funds.can_afford(1000));
        assert!(!funds.can_afford(1001));
    }
    
    #[test]
    fn test_transaction_tracking() {
        let mut funds = PlayerFunds::new(1000);
        
        funds.add_income(Transaction::income(
            TransactionType::RentIncome, 
            500, 
            "Test rent", 
            1
        ));
        
        assert_eq!(funds.balance, 1500);
        assert_eq!(funds.total_income, 500);
        
        funds.deduct_expense(Transaction::expense(
            TransactionType::RepairCost,
            200,
            "Test repair",
            1
        ));
        
        assert_eq!(funds.balance, 1300);
        assert_eq!(funds.total_expenses, 200);
    }
    
    #[test]
    fn test_upgrade_costs() {
        use crate::building::DesignType;
        
        assert_eq!(
            UpgradeCosts::design_upgrade_cost(&DesignType::Bare),
            Some(500)
        );
        assert_eq!(
            UpgradeCosts::design_upgrade_cost(&DesignType::Practical),
            Some(1000)
        );
        assert_eq!(
            UpgradeCosts::design_upgrade_cost(&DesignType::Cozy),
            None
        );
    }
}
```

---

## Balance Notes

| Item | Cost | Notes |
|------|------|-------|
| Repair (per point) | $10 | Full repair (50 pts) = $500 |
| Bare → Practical | $500 | One-time investment |
| Practical → Cozy | $1000 | Premium upgrade |
| Soundproofing | $300 | Per apartment |
| Hallway repair (per point) | $15 | More expensive than individual |

| Income Source | Amount | Notes |
|---------------|--------|-------|
| Small apartment base rent | $600 | Typical student unit |
| Medium apartment base rent | $900 | Professional/family |

Starting with $5000 allows:
- Full repair of 2-3 apartments, OR
- Design upgrade on 3-4 apartments, OR
- Mix of improvements

This creates early resource pressure without immediate bankruptcy risk.
