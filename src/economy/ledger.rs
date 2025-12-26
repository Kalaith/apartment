
use super::{Transaction, TransactionType};
use serde::{Deserialize, Serialize};

/// Monthly financial summary
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonthlyReport {
    pub tick: u32,
    pub rent_income: i32,
    pub repair_costs: i32,
    pub upgrade_costs: i32,
    pub net: i32,
    pub ending_balance: i32,
}

/// Financial tracking across the game
#[derive(Clone, Debug, Serialize, Deserialize)]
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
                TransactionType::BuildingPurchase => upgrade_costs += t.amount.abs(), // Count as capital upgrade for now
                TransactionType::AssetSale => rent_income += t.amount.abs(), // Count condo sales as income
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
}

impl Default for FinancialLedger {
    fn default() -> Self {
        Self::new()
    }
}
