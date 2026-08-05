//! Monthly operating metrics for headless runs.

use super::Sim;
use crate::economy::TransactionType;

impl Sim {
    pub(super) fn tick_expenses(&self) -> i32 {
        self.funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| transaction.amount < 0)
            .map(|transaction| transaction.amount.abs())
            .sum()
    }

    pub(super) fn tick_earned_income(&self) -> i32 {
        self.funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| {
                transaction.transaction_type == TransactionType::RentIncome
                    && transaction.amount > 0
            })
            .map(|transaction| transaction.amount)
            .sum()
    }
}
