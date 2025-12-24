# Task 05: Simulation System

## Priority: 🟢 MEDIUM
## Dependencies: Task 02 (Building), Task 03 (Tenant), Task 04 (Economy)
## Estimated Effort: 2-3 hours
## Can Parallel With: Task 04 (partially)

---

## Objective
Implement the time progression system that drives the game loop: decay, happiness updates, rent collection, and win/lose conditions.

---

## Deliverables

### 1. src/simulation/mod.rs

```rust
mod tick;
mod decay;
mod events;
mod win_condition;

pub use tick::{GameTick, TickResult};
pub use decay::apply_decay;
pub use events::{GameEvent, EventLog};
pub use win_condition::{check_win_condition, GameOutcome};
```

### 2. src/simulation/tick.rs

```rust
use crate::building::Building;
use crate::tenant::{Tenant, TenantApplication, happiness, application};
use crate::economy::{PlayerFunds, FinancialLedger, rent};
use super::{GameEvent, EventLog, decay, win_condition, GameOutcome};

/// Result of processing a game tick
#[derive(Clone, Debug)]
pub struct TickResult {
    pub events: Vec<GameEvent>,
    pub rent_collected: i32,
    pub tenants_moved_out: Vec<String>,
    pub new_applications: usize,
    pub outcome: Option<GameOutcome>,
}

/// Main tick processor
pub struct GameTick;

impl GameTick {
    /// Process a single game tick (one month)
    pub fn process(
        building: &mut Building,
        tenants: &mut Vec<Tenant>,
        applications: &mut Vec<TenantApplication>,
        funds: &mut PlayerFunds,
        ledger: &mut FinancialLedger,
        event_log: &mut EventLog,
        current_tick: u32,
        next_tenant_id: &mut u32,
    ) -> TickResult {
        let mut result = TickResult {
            events: Vec::new(),
            rent_collected: 0,
            tenants_moved_out: Vec::new(),
            new_applications: 0,
            outcome: None,
        };
        
        // === Phase 1: Collect Rent ===
        let rent_result = rent::collect_rent(tenants, building, funds, current_tick);
        result.rent_collected = rent_result.total_collected;
        
        for payment in &rent_result.payments {
            result.events.push(GameEvent::RentPaid {
                tenant_name: payment.tenant_name.clone(),
                amount: payment.amount,
            });
        }
        
        for missed in &rent_result.missed_payments {
            result.events.push(GameEvent::RentMissed {
                tenant_name: missed.tenant_name.clone(),
                reason: missed.reason.clone(),
            });
        }
        
        // === Phase 2: Apply Decay ===
        let decay_events = decay::apply_decay(building);
        result.events.extend(decay_events);
        
        // === Phase 3: Update Tenant Happiness ===
        for tenant in tenants.iter_mut() {
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apartment) = building.get_apartment(apt_id) {
                    let factors = happiness::calculate_happiness(tenant, apartment, building);
                    let old_happiness = tenant.happiness;
                    let new_happiness = factors.total();
                    tenant.set_happiness(new_happiness);
                    
                    // Generate events for significant changes
                    if new_happiness < 30 && old_happiness >= 30 {
                        result.events.push(GameEvent::TenantUnhappy {
                            tenant_name: tenant.name.clone(),
                            happiness: new_happiness,
                        });
                    }
                    
                    // Check for complaints based on factors
                    if factors.noise_factor < -10 {
                        result.events.push(GameEvent::NoiseComplaint {
                            tenant_name: tenant.name.clone(),
                        });
                    }
                    if factors.condition_factor < -15 {
                        result.events.push(GameEvent::ConditionComplaint {
                            tenant_name: tenant.name.clone(),
                            apartment_unit: apartment.unit_number.clone(),
                        });
                    }
                }
            }
            tenant.add_month();
        }
        
        // === Phase 4: Process Move-outs ===
        let departure_notices = application::process_departures(tenants, building);
        for notice in departure_notices {
            result.events.push(GameEvent::TenantMovedOut {
                message: notice.clone(),
            });
            result.tenants_moved_out.push(notice);
        }
        
        // === Phase 5: Remove Expired Applications ===
        applications.retain(|app| !app.is_expired(current_tick));
        
        // === Phase 6: Generate New Applications ===
        let new_apps = application::generate_applications(
            building,
            applications,
            current_tick,
            next_tenant_id,
        );
        result.new_applications = new_apps.len();
        
        for app in &new_apps {
            result.events.push(GameEvent::NewApplication {
                tenant_name: app.tenant.name.clone(),
                archetype: format!("{:?}", app.tenant.archetype),
                apartment_unit: building.get_apartment(app.apartment_id)
                    .map(|a| a.unit_number.clone())
                    .unwrap_or_default(),
            });
        }
        
        applications.extend(new_apps);
        
        // === Phase 7: Generate Monthly Report ===
        let tick_transactions: Vec<_> = funds.transactions_for_tick(current_tick);
        let report = ledger.generate_report(current_tick, &tick_transactions, funds.balance);
        
        result.events.push(GameEvent::MonthEnd {
            tick: current_tick,
            income: report.rent_income,
            expenses: report.repair_costs + report.upgrade_costs,
            balance: report.ending_balance,
        });
        
        // === Phase 8: Check Win/Lose Conditions ===
        result.outcome = win_condition::check_win_condition(
            building,
            tenants,
            funds,
            current_tick,
        );
        
        if let Some(ref outcome) = result.outcome {
            result.events.push(GameEvent::GameEnded {
                outcome: outcome.clone(),
            });
        }
        
        // Log all events
        for event in &result.events {
            event_log.log(event.clone(), current_tick);
        }
        
        result
    }
}

/// Advance time and return whether game should continue
pub fn advance_tick(
    building: &mut Building,
    tenants: &mut Vec<Tenant>,
    applications: &mut Vec<TenantApplication>,
    funds: &mut PlayerFunds,
    ledger: &mut FinancialLedger,
    event_log: &mut EventLog,
    current_tick: &mut u32,
    next_tenant_id: &mut u32,
) -> TickResult {
    *current_tick += 1;
    
    GameTick::process(
        building,
        tenants,
        applications,
        funds,
        ledger,
        event_log,
        *current_tick,
        next_tenant_id,
    )
}
```

### 3. src/simulation/decay.rs

```rust
use crate::building::Building;
use super::GameEvent;

/// Decay rates per tick
pub mod rates {
    /// Condition points lost per apartment per tick
    pub const APARTMENT_DECAY: i32 = 2;
    
    /// Hallway condition decay per tick
    pub const HALLWAY_DECAY: i32 = 1;
    
    /// Threshold for "poor condition" warning
    pub const POOR_CONDITION_THRESHOLD: i32 = 40;
    
    /// Threshold for "critical condition" warning
    pub const CRITICAL_CONDITION_THRESHOLD: i32 = 20;
}

/// Apply monthly decay to all building elements
/// Returns events for significant decay milestones
pub fn apply_decay(building: &mut Building) -> Vec<GameEvent> {
    let mut events = Vec::new();
    
    // Track conditions before decay for event generation
    let conditions_before: Vec<_> = building.apartments
        .iter()
        .map(|a| (a.id, a.unit_number.clone(), a.condition))
        .collect();
    
    let hallway_before = building.hallway_condition;
    
    // Apply decay
    building.apply_monthly_decay();
    
    // Check for significant condition changes in apartments
    for (id, unit, old_condition) in conditions_before {
        if let Some(apt) = building.get_apartment(id) {
            let new_condition = apt.condition;
            
            // Check for crossing thresholds
            if old_condition >= rates::CRITICAL_CONDITION_THRESHOLD 
               && new_condition < rates::CRITICAL_CONDITION_THRESHOLD 
            {
                events.push(GameEvent::CriticalCondition {
                    apartment_unit: unit,
                    condition: new_condition,
                });
            } else if old_condition >= rates::POOR_CONDITION_THRESHOLD 
               && new_condition < rates::POOR_CONDITION_THRESHOLD 
            {
                events.push(GameEvent::PoorCondition {
                    apartment_unit: unit,
                    condition: new_condition,
                });
            }
        }
    }
    
    // Check hallway
    let hallway_after = building.hallway_condition;
    if hallway_before >= rates::POOR_CONDITION_THRESHOLD 
       && hallway_after < rates::POOR_CONDITION_THRESHOLD 
    {
        events.push(GameEvent::HallwayDeteriorating {
            condition: hallway_after,
        });
    }
    
    events
}

/// Calculate turns until apartment reaches critical condition
pub fn turns_until_critical(current_condition: i32) -> i32 {
    let gap = current_condition - rates::CRITICAL_CONDITION_THRESHOLD;
    if gap <= 0 {
        return 0;
    }
    (gap as f32 / rates::APARTMENT_DECAY as f32).ceil() as i32
}

/// Get overall building health status
#[derive(Clone, Debug, PartialEq)]
pub enum BuildingHealth {
    Excellent,  // Average condition >= 80
    Good,       // Average condition >= 60
    Fair,       // Average condition >= 40
    Poor,       // Average condition >= 20
    Critical,   // Average condition < 20
}

pub fn assess_building_health(building: &Building) -> BuildingHealth {
    let avg_condition: i32 = if building.apartments.is_empty() {
        building.hallway_condition
    } else {
        let apt_total: i32 = building.apartments.iter().map(|a| a.condition).sum();
        let apt_avg = apt_total / building.apartments.len() as i32;
        (apt_avg + building.hallway_condition) / 2
    };
    
    match avg_condition {
        80..=100 => BuildingHealth::Excellent,
        60..=79 => BuildingHealth::Good,
        40..=59 => BuildingHealth::Fair,
        20..=39 => BuildingHealth::Poor,
        _ => BuildingHealth::Critical,
    }
}
```

### 4. src/simulation/events.rs

```rust
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
```

### 5. src/simulation/win_condition.rs

```rust
use serde::{Deserialize, Serialize};
use crate::building::Building;
use crate::tenant::Tenant;
use crate::economy::PlayerFunds;

/// Game outcome
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameOutcome {
    Victory {
        total_income: i32,
        months_played: u32,
        final_happiness_avg: i32,
    },
    Bankruptcy,
    AllTenantsLeft,
}

/// Win condition thresholds
pub mod thresholds {
    /// Minimum average happiness for victory
    pub const MIN_HAPPINESS: i32 = 60;
    
    /// All units must be occupied for victory
    pub const FULL_OCCUPANCY_REQUIRED: bool = true;
    
    /// Minimum ticks before victory can trigger (prevent instant win)
    pub const MIN_TICKS_FOR_VICTORY: u32 = 6;
}

/// Check current game state for win/lose conditions
pub fn check_win_condition(
    building: &Building,
    tenants: &[Tenant],
    funds: &PlayerFunds,
    current_tick: u32,
) -> Option<GameOutcome> {
    // Check for bankruptcy
    if funds.is_bankrupt() {
        return Some(GameOutcome::Bankruptcy);
    }
    
    // Check if all tenants left (after having some)
    if tenants.is_empty() && current_tick > 3 {
        // Check if we ever had tenants (building was lived in)
        let was_occupied = building.apartments.iter().any(|a| a.tenant_id.is_some());
        if !was_occupied {
            return Some(GameOutcome::AllTenantsLeft);
        }
    }
    
    // Check for victory conditions
    if current_tick < thresholds::MIN_TICKS_FOR_VICTORY {
        return None;  // Too early for victory
    }
    
    // All units must be occupied
    if thresholds::FULL_OCCUPANCY_REQUIRED {
        if building.vacancy_count() > 0 {
            return None;
        }
    }
    
    // Calculate average happiness
    if tenants.is_empty() {
        return None;
    }
    
    let avg_happiness: i32 = tenants.iter()
        .map(|t| t.happiness)
        .sum::<i32>() / tenants.len() as i32;
    
    if avg_happiness >= thresholds::MIN_HAPPINESS {
        return Some(GameOutcome::Victory {
            total_income: funds.total_income,
            months_played: current_tick,
            final_happiness_avg: avg_happiness,
        });
    }
    
    None
}

/// Get progress towards victory
#[derive(Clone, Debug)]
pub struct VictoryProgress {
    pub occupancy_percent: f32,
    pub avg_happiness: i32,
    pub happiness_target: i32,
    pub months_played: u32,
    pub months_required: u32,
    pub is_profitable: bool,
}

pub fn get_victory_progress(
    building: &Building,
    tenants: &[Tenant],
    funds: &PlayerFunds,
    current_tick: u32,
) -> VictoryProgress {
    let total_units = building.apartments.len() as f32;
    let occupied = building.occupancy_count() as f32;
    
    let avg_happiness = if tenants.is_empty() {
        0
    } else {
        tenants.iter().map(|t| t.happiness).sum::<i32>() / tenants.len() as i32
    };
    
    VictoryProgress {
        occupancy_percent: if total_units > 0.0 { (occupied / total_units) * 100.0 } else { 0.0 },
        avg_happiness,
        happiness_target: thresholds::MIN_HAPPINESS,
        months_played: current_tick,
        months_required: thresholds::MIN_TICKS_FOR_VICTORY,
        is_profitable: funds.balance > 0 && funds.net_profit() >= 0,
    }
}
```

---

## Integration with GameplayState

```rust
// In src/state/gameplay.rs:
use crate::simulation::{EventLog, GameOutcome, advance_tick, TickResult};

pub struct GameplayState {
    pub building: Building,
    pub tenants: Vec<Tenant>,
    pub applications: Vec<TenantApplication>,
    pub funds: PlayerFunds,
    pub ledger: FinancialLedger,
    pub event_log: EventLog,
    pub current_tick: u32,
    pub next_tenant_id: u32,
    pub game_outcome: Option<GameOutcome>,
    pub last_tick_result: Option<TickResult>,
}

impl GameplayState {
    pub fn end_turn(&mut self) {
        let result = advance_tick(
            &mut self.building,
            &mut self.tenants,
            &mut self.applications,
            &mut self.funds,
            &mut self.ledger,
            &mut self.event_log,
            &mut self.current_tick,
            &mut self.next_tenant_id,
        );
        
        self.game_outcome = result.outcome.clone();
        self.last_tick_result = Some(result);
    }
}
```

---

## Acceptance Criteria

- [ ] Tick advances all game systems
- [ ] Rent collected from all occupied apartments
- [ ] Condition decays each tick
- [ ] Happiness updates based on current state
- [ ] Unhappy tenants leave
- [ ] New applications generated based on appeal
- [ ] Victory triggered at full occupancy + high happiness
- [ ] Bankruptcy triggered at negative balance
- [ ] All events logged and displayable

---

## Tick Processing Order

```
1. Collect Rent
   └── Generate rent events

2. Apply Decay
   └── Generate condition warnings

3. Update Happiness
   └── Generate complaints/warnings

4. Process Departures
   └── Generate move-out events

5. Expire Old Applications

6. Generate New Applications
   └── Generate application events

7. Generate Monthly Report

8. Check Win/Lose Conditions
```

This order ensures:
- Rent is collected before expenses might cause bankruptcy
- Decay happens before happiness is calculated
- Departures clear apartments for new applications
