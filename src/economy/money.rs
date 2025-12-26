
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    RentIncome,
    RepairCost,
    UpgradeCost,
    HallwayRepair,
    BuildingPurchase,
    AssetSale,
    PropertyTax,
    Utilities,
    Insurance,
    StaffSalary,
    CriticalFailure,
    Grant,  // Mission rewards, grants, bonuses
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
    
    /// Spend money without a specific transaction type (used by random events)
    /// Returns true if successful, false if insufficient funds
    pub fn spend(&mut self, amount: i32) -> bool {
        if self.balance < amount {
            return false;
        }
        self.balance -= amount;
        self.total_expenses += amount;
        true
    }
    
    /// Check if player is bankrupt
    pub fn is_bankrupt(&self) -> bool {
        self.balance < 0
    }
    
    /// Get transactions for a specific tick
    pub fn transactions_for_tick(&self, tick: u32) -> Vec<&Transaction> {
        self.transactions.iter().filter(|t| t.tick == tick).collect()
    }
}

impl Default for PlayerFunds {
    fn default() -> Self {
        Self::new(5000)  // Default starting funds
    }
}
