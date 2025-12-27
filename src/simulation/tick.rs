use crate::building::Building;
use crate::tenant::{Tenant, TenantApplication, calculate_happiness, generate_applications, process_departures};
use crate::economy::{PlayerFunds, FinancialLedger, collect_rent, OperatingCosts, Transaction, TransactionType};
use super::{GameEvent, EventLog, decay, win_condition, GameOutcome, EventSystem};


use serde::{Deserialize, Serialize};

/// Result of processing a game tick
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        config: &crate::data::config::GameConfig,
    ) -> TickResult {
        let mut result = TickResult {
            events: Vec::new(),
            rent_collected: 0,
            tenants_moved_out: Vec::new(),
            new_applications: 0,
            outcome: None,
        };
        
        // 1. Collect Rent
        Self::collect_rent(building, tenants, funds, current_tick, &mut result);
        
        // 2. Operating Costs & Staff
        Self::process_operating_costs(building, funds, current_tick, &mut result, config);
        Self::process_staff_effects(building, tenants);
        Self::process_critical_failures(building, tenants, funds, current_tick, &mut result);

        // 3. Random Events
        let mut event_system = EventSystem::new();
        let random_events = event_system.check_events(building, funds, current_tick);
        result.events.extend(random_events);
        
        // 4. Decay & Ownership
        if building.update_ownership(current_tick) {
            // Logic for handling ownership updates could go here
        }
        let decay_events = decay::apply_decay(building, &config.thresholds);
        result.events.extend(decay_events);
        
        // 5. Tenant Happiness & Updates
        Self::update_tenants(building, tenants, &mut result, &config.happiness);
        
        // 6. Move-outs
        let departure_notices = process_departures(tenants, building);
        for notice in departure_notices {
            result.events.push(GameEvent::TenantMovedOut {
                message: notice.clone(),
            });
            result.tenants_moved_out.push(notice);
        }
        
        // 7. Applications
        applications.retain(|app| !app.is_expired(current_tick));
        let new_apps = generate_applications(
            building,
            applications,
            current_tick,
            next_tenant_id,
            &config.matching,
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
        
        // 8. Monthly Report
        let tick_transactions: Vec<_> = funds.transactions_for_tick(current_tick);
        let report = ledger.generate_report(current_tick, &tick_transactions, funds.balance);
        
        result.events.push(GameEvent::MonthEnd {
            tick: current_tick,
            income: report.rent_income,
            expenses: report.repair_costs + report.upgrade_costs,
            balance: report.ending_balance,
        });
        
        // 9. Win/Lose check
        result.outcome = win_condition::check_win_condition(
            building,
            tenants,
            funds,
            current_tick,
            &config.win_conditions,
            &config.happiness,
            &config.thresholds,
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

    fn collect_rent(
        building: &mut Building,
        tenants: &mut Vec<Tenant>,
        funds: &mut PlayerFunds,
        current_tick: u32,
        result: &mut TickResult,
    ) {
        let rent_result = collect_rent(tenants, building, funds, current_tick);
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
                amount: missed.amount,
            });
        }
    }

    fn process_operating_costs(
        building: &mut Building,
        funds: &mut PlayerFunds,
        current_tick: u32,
        result: &mut TickResult,
        config: &crate::data::config::GameConfig,
    ) {
        // Marketing
        let marketing_cost = building.marketing_strategy.monthly_cost(&config.marketing);
        if marketing_cost > 0 {
            if !funds.spend(marketing_cost) {
                building.marketing_strategy = crate::building::MarketingType::None;
                result.events.push(GameEvent::Notification { 
                    message: "Marketing campaign cancelled due to lack of funds.".to_string(),
                    level: crate::simulation::NotificationLevel::Warning 
                });
            }
        }
        
        if building.open_house_remaining > 0 {
            building.open_house_remaining -= 1;
            if building.open_house_remaining == 0 {
                result.events.push(GameEvent::Notification {
                    message: "Open House event has ended.".to_string(),
                    level: crate::simulation::NotificationLevel::Info,
                });
            }
        }
        
        // Taxes & Expenses
        let tax = OperatingCosts::calculate_property_tax(building, result.rent_collected, &config.operating_costs);
        if tax > 0 {
            funds.deduct_expense(Transaction::expense(TransactionType::PropertyTax, tax, "Monthly Property Tax", current_tick));
        }
        
        let utilities = OperatingCosts::calculate_utilities(building, &config.operating_costs);
        if utilities > 0 {
            funds.deduct_expense(Transaction::expense(TransactionType::Utilities, utilities, "Utility Bills", current_tick));
        }
        
        let insurance = OperatingCosts::calculate_insurance(building, &config.operating_costs);
        if insurance > 0 {
            funds.deduct_expense(Transaction::expense(TransactionType::Insurance, insurance, "Property Insurance", current_tick));
        }
        
        // Staff Salaries - Data Driven
        let salaries = OperatingCosts::calculate_staff_salaries(building, &config.economy);
        if salaries > 0 {
            funds.deduct_expense(Transaction::expense(TransactionType::StaffSalary, salaries, "Staff Salaries", current_tick));
        }
    }

    fn process_staff_effects(building: &mut Building, tenants: &mut Vec<Tenant>) {
        // Janitor: Auto-repair small decay
        if building.flags.contains("staff_janitor") {
             for apt in &mut building.apartments {
                 if apt.condition < 90 && apt.condition > 50 {
                     apt.condition += 1;
                 }
             }
             if building.hallway_condition < 90 && building.hallway_condition > 50 {
                 building.hallway_condition += 1;
             }
        }

        // Security: Boost happiness
        if building.flags.contains("staff_security") {
            for t in tenants.iter_mut() {
                t.happiness = (t.happiness + 2).min(100);
            }
        }
        
        // Manager: Bonus happiness (simplification of "handles issues")
        if building.flags.contains("staff_manager") {
             for t in tenants.iter_mut() {
                t.happiness = (t.happiness + 1).min(100);
            }
        }
    }

    fn process_critical_failures(
        building: &mut Building,
        tenants: &mut Vec<Tenant>,
        funds: &mut PlayerFunds,
        current_tick: u32,
        result: &mut TickResult
    ) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let mut prob = 0.005;
        // Security reduces failure probability
        if building.flags.contains("staff_security") {
            prob *= 0.5;
        }
        
        // Boiler Failure
        if rng.gen_bool(prob) {
            let cost = 1500;
            if funds.can_afford(cost) {
                funds.deduct_expense(Transaction::expense(TransactionType::CriticalFailure, cost, "Boiler Emergency Repair", current_tick));
                result.events.push(GameEvent::BoilerFailure { cost });
            } else {
                 result.events.push(GameEvent::TenantUnhappy { tenant_name: "ALL TENANTS".to_string(), happiness: 0 }); 
                 for t in tenants.iter_mut() {
                     t.happiness = (t.happiness - 30).max(0);
                 }
                 result.events.push(GameEvent::InsufficientFunds { action: "Fix Boiler".to_string(), needed: cost, available: funds.balance });
             }
        }
        
        // Structural Issue
        if rng.gen_bool(prob) {
             let cost = 2500;
             let tx = Transaction::expense(TransactionType::CriticalFailure, cost, "Structural Reinforcement", current_tick);
             if funds.deduct_expense(tx) {
                  result.events.push(GameEvent::StructuralIssue { cost, description: "Foundation Crack".to_string() });
             } else {
                  building.hallway_condition = (building.hallway_condition - 20).max(0);
                  result.events.push(GameEvent::HallwayDeteriorating { condition: building.hallway_condition });
                  result.events.push(GameEvent::InsufficientFunds { action: "Fix Foundation".to_string(), needed: cost, available: funds.balance });
             }
        }
    }

    fn update_tenants(
        building: &Building,
        tenants: &mut Vec<Tenant>,
        result: &mut TickResult,
        config: &crate::data::config::HappinessConfig,
    ) {
        for tenant in tenants.iter_mut() {
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apartment) = building.get_apartment(apt_id) {
                    let factors = calculate_happiness(tenant, apartment, building, config);
                    let old_happiness = tenant.happiness;
                    let new_happiness = factors.total();
                    tenant.set_happiness(new_happiness);
                    
                    if new_happiness < 30 && old_happiness >= 30 {
                        result.events.push(GameEvent::TenantUnhappy {
                            tenant_name: tenant.name.clone(),
                            happiness: new_happiness,
                        });
                    }
                    
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
    config: &crate::data::config::GameConfig,
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
        config,
    )
}
