#![allow(dead_code)]
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
