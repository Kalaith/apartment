use serde::{Deserialize, Serialize};
use super::GameOutcome;

/// Significant events that happen during simulation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEvent {
    // Economy
    RentPaid { tenant_name: String, amount: i32 },
    RentMissed { tenant_name: String, amount: i32 },
    UpgradeCompleted { description: String, cost: i32 },
    InsufficientFunds { action: String, needed: i32, available: i32 },
    
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
    
    // Time events
    MonthEnd { tick: u32, income: i32, expenses: i32, balance: i32 },
    
    // Game state events
    GameEnded { outcome: GameOutcome },
    
    // Random Events
    Heatwave { tick_duration: u32 },
    PipeBurst { apartment_unit: String, damage: i32 },
    Gentrification { tick_duration: u32, effect_desc: String },
    Inspection { result: String, fine: i32 },
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
                    GameOutcome::Bankruptcy { .. } => "💸 Bankrupt!".to_string(),
                    GameOutcome::AllTenantsLeft => "🚪 All tenants left!".to_string(),
                }
            }
            GameEvent::Heatwave { tick_duration } => {
                format!("☀️ Heatwave! (Duration: {} months)", tick_duration)
            }
            GameEvent::PipeBurst { apartment_unit, damage } => {
                format!("💧 Pipe Burst in Unit {}! (-{} condition)", apartment_unit, damage)
            }
            GameEvent::Gentrification { tick_duration, effect_desc } => {
                format!("📈 Neighborhood improving! {} (Duration: {})", effect_desc, tick_duration)
            }
            GameEvent::Inspection { result, fine } => {
                if *fine > 0 {
                    format!("📋 Inspection Failed: {} (Fine: -${})", result, fine)
                } else {
                    format!("📋 Inspection Passed: {}", result)
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
            GameEvent::Heatwave { .. } => EventSeverity::Warning,
            GameEvent::PipeBurst { .. } => EventSeverity::Negative,
            GameEvent::Gentrification { .. } => EventSeverity::Positive,
            GameEvent::Inspection { fine, .. } => {
                if *fine > 0 {
                    EventSeverity::Negative
                } else {
                    EventSeverity::Positive
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum EventSeverity {
    Positive,
    Info,
    Warning,
    Negative,
}

/// Log of all game events
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
