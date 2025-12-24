use serde::{Deserialize, Serialize};
use super::GameOutcome;

/// All possible game events
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    // Rent events
    RentPaid { tenant_name: String, amount: i32 },
    RentMissed { tenant_name: String, reason: String },
    
    // Tenant events
    TenantUnhappy { tenant_name: String, happiness: i32 },
    TenantMovedOut { message: String },
    NewApplication { tenant_name: String, archetype: String, apartment_unit: String },
    TenantMovedIn { tenant_name: String, apartment_unit: String },
    
    // Complaint events
    NoiseComplaint { tenant_name: String },
    ConditionComplaint { tenant_name: String, apartment_unit: String },
    
    // Building events
    PoorCondition { apartment_unit: String, condition: i32 },
    CriticalCondition { apartment_unit: String, condition: i32 },
    HallwayDeteriorating { condition: i32 },
    
    // Economy events
    UpgradeCompleted { description: String, cost: i32 },
    InsufficientFunds { action: String, needed: i32, available: i32 },
    
    // Time events
    MonthEnd { tick: u32, income: i32, expenses: i32, balance: i32 },
    
    // Game state events
    GameEnded { outcome: GameOutcome },
}

impl GameEvent {
    /// Get a short message for display
    pub fn message(&self) -> String {
        match self {
            GameEvent::RentPaid { tenant_name, amount } => {
                format!("Received ${} rent from {}", amount, tenant_name)
            }
            GameEvent::RentMissed { tenant_name, .. } => {
                format!("{} missed rent payment", tenant_name)
            }
            GameEvent::TenantUnhappy { tenant_name, happiness } => {
                format!("{} is unhappy ({}%)", tenant_name, happiness)
            }
            GameEvent::TenantMovedOut { message } => message.clone(),
            GameEvent::NewApplication { tenant_name, archetype, apartment_unit } => {
                format!("{} ({}) applied for Unit {}", tenant_name, archetype, apartment_unit)
            }
            GameEvent::TenantMovedIn { tenant_name, apartment_unit } => {
                format!("{} moved into Unit {}", tenant_name, apartment_unit)
            }
            GameEvent::NoiseComplaint { tenant_name } => {
                format!("Noise complaint from {}", tenant_name)
            }
            GameEvent::ConditionComplaint { tenant_name, apartment_unit } => {
                format!("{} complained about Unit {} condition", tenant_name, apartment_unit)
            }
            GameEvent::PoorCondition { apartment_unit, condition } => {
                format!("Unit {} in poor condition ({}%)", apartment_unit, condition)
            }
            GameEvent::CriticalCondition { apartment_unit, condition } => {
                format!("⚠️ Unit {} CRITICAL ({}%)", apartment_unit, condition)
            }
            GameEvent::HallwayDeteriorating { condition } => {
                format!("Hallway deteriorating ({}%)", condition)
            }
            GameEvent::UpgradeCompleted { description, cost } => {
                format!("{} (-${})", description, cost)
            }
            GameEvent::InsufficientFunds { action, needed, available } => {
                format!("Cannot afford {} (need ${}, have ${})", action, needed, available)
            }
            GameEvent::MonthEnd { tick, income, expenses, balance } => {
                format!("Month {} ended: +${} -${} = ${}", tick, income, expenses, balance)
            }
            GameEvent::GameEnded { outcome } => {
                match outcome {
                    GameOutcome::Victory { .. } => "🎉 Victory!".to_string(),
                    GameOutcome::Bankruptcy => "💸 Bankrupt!".to_string(),
                    GameOutcome::AllTenantsLeft => "🚪 All tenants left!".to_string(),
                }
            }
        }
    }
    
    /// Get event severity for UI coloring
    pub fn severity(&self) -> EventSeverity {
        match self {
            GameEvent::RentPaid { .. } => EventSeverity::Positive,
            GameEvent::TenantMovedIn { .. } => EventSeverity::Positive,
            GameEvent::UpgradeCompleted { .. } => EventSeverity::Positive,
            GameEvent::NewApplication { .. } => EventSeverity::Info,
            GameEvent::MonthEnd { .. } => EventSeverity::Info,
            GameEvent::RentMissed { .. } => EventSeverity::Warning,
            GameEvent::TenantUnhappy { .. } => EventSeverity::Warning,
            GameEvent::NoiseComplaint { .. } => EventSeverity::Warning,
            GameEvent::ConditionComplaint { .. } => EventSeverity::Warning,
            GameEvent::PoorCondition { .. } => EventSeverity::Warning,
            GameEvent::HallwayDeteriorating { .. } => EventSeverity::Warning,
            GameEvent::InsufficientFunds { .. } => EventSeverity::Negative,
            GameEvent::TenantMovedOut { .. } => EventSeverity::Negative,
            GameEvent::CriticalCondition { .. } => EventSeverity::Negative,
            GameEvent::GameEnded { outcome } => match outcome {
                GameOutcome::Victory { .. } => EventSeverity::Positive,
                _ => EventSeverity::Negative,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EventSeverity {
    Positive,
    Info,
    Warning,
    Negative,
}

/// Log of all game events
#[derive(Clone, Debug, Default)]
pub struct EventLog {
    events: Vec<(u32, GameEvent)>,  // (tick, event)
}

impl EventLog {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }
    
    pub fn log(&mut self, event: GameEvent, tick: u32) {
        self.events.push((tick, event));
    }
    
    pub fn events_for_tick(&self, tick: u32) -> Vec<&GameEvent> {
        self.events.iter()
            .filter(|(t, _)| *t == tick)
            .map(|(_, e)| e)
            .collect()
    }
    
    pub fn recent_events(&self, count: usize) -> Vec<&GameEvent> {
        self.events.iter()
            .rev()
            .take(count)
            .map(|(_, e)| e)
            .collect()
    }
    
    pub fn all_events(&self) -> &[(u32, GameEvent)] {
        &self.events
    }
}
