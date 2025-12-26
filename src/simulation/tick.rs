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
    ) -> TickResult {
        let mut result = TickResult {
            events: Vec::new(),
            rent_collected: 0,
            tenants_moved_out: Vec::new(),
            new_applications: 0,
            outcome: None,
        };
        
        // === Phase 1: Collect Rent ===
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
        
        // === Phase 1.1: Operating & Marketing Costs ===
        let marketing_cost = building.marketing_strategy.monthly_cost();
        if marketing_cost > 0 {
            if funds.spend(marketing_cost) {
                // Paid for marketing
            } else {
                // Ran out of money for marketing -> auto cancel
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
        
        // 1. Property Tax
        // Estimate rent income (simple heuristic: previous rent collected or current potential)
        // Using actual corrected rent this month is safer.
        let tax = OperatingCosts::calculate_property_tax(building, result.rent_collected);
        if tax > 0 {
            let tx = Transaction::expense(TransactionType::PropertyTax, tax, "Monthly Property Tax", current_tick);
            funds.deduct_expense(tx);
        }
        
        // 2. Utilities
        let utilities = OperatingCosts::calculate_utilities(building);
        if utilities > 0 {
            let tx = Transaction::expense(TransactionType::Utilities, utilities, "Utility Bills", current_tick);
            funds.deduct_expense(tx);
        }
        
        // 3. Insurance
        let insurance = OperatingCosts::calculate_insurance(building);
        if insurance > 0 {
            let tx = Transaction::expense(TransactionType::Insurance, insurance, "Property Insurance", current_tick);
            funds.deduct_expense(tx);
        }
        
        // 4. Staff Salaries
        let salaries = OperatingCosts::calculate_staff_salaries(building);
        if salaries > 0 {
            let tx = Transaction::expense(TransactionType::StaffSalary, salaries, "Staff Salaries", current_tick);
            funds.deduct_expense(tx);
        }

        // === Phase 1.2: Staff Effects & Critical Failures ===
        
        // Staff Effects
        if building.staff_janitor {
             // Auto-repair small decay (Condition < 90 but > 50, fix 1-2 points)
             for apt in &mut building.apartments {
                 if apt.condition < 90 && apt.condition > 50 {
                     // Free repair by janitor (already paid salary)
                     apt.condition += 1;
                 }
             }
             if building.hallway_condition < 90 && building.hallway_condition > 50 {
                 building.hallway_condition += 1;
             }
        }
        
        // Critical Failures (Low chance, high impact)
        // 1% chance per tick of Boiler Failure (if no insurance/maintenance?)
        use rand::Rng; // Need to import or use crate::rand if available, but let's use a simple deterministic check or rand via crate
        // actually rand is likely available as dependency.
        // For now, let's use a simple modulo or pseudo-random if rand isn't easily imported in this context without checking Cargo.toml
        // But the previous prompt context said `Rand` is in the tech stack!
        let mut rng = rand::thread_rng();
        
        if rng.gen_bool(0.005) { // 0.5% chance
            let cost = 1500;
            if funds.can_afford(cost) {
                let tx = Transaction::expense(TransactionType::CriticalFailure, cost, "Boiler Emergency Repair", current_tick);
                funds.deduct_expense(tx);
                result.events.push(GameEvent::BoilerFailure { cost });
            } else {
                 // Severe penalty if can't afford
                 result.events.push(GameEvent::TenantUnhappy { tenant_name: "ALL TENANTS".to_string(), happiness: 0 }); 
                 for t in tenants.iter_mut() {
                     t.happiness = (t.happiness - 30).max(0);
                 }
                 result.events.push(GameEvent::InsufficientFunds { action: "Fix Boiler".to_string(), needed: cost, available: funds.balance });
            }
        }
        
        // 0.5% chance of Structural Issue
        if rng.gen_bool(0.005) {
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

        // === Phase 1.5: Random Events ===
        let mut event_system = EventSystem::new();
        let random_events = event_system.check_events(building, funds, current_tick);
        result.events.extend(random_events);
        
        // === Phase 2: Apply Decay ===
        // Update ownership (HOA fees, votes)
        if building.update_ownership(current_tick) {
            // Logic for handling ownership updates could go here
        }

        let decay_events = decay::apply_decay(building);
        result.events.extend(decay_events);
        
        // === Phase 3: Update Tenant Happiness ===
        for tenant in tenants.iter_mut() {
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apartment) = building.get_apartment(apt_id) {
                    let factors = calculate_happiness(tenant, apartment, building);
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
        let departure_notices = process_departures(tenants, building);
        for notice in departure_notices {
            result.events.push(GameEvent::TenantMovedOut {
                message: notice.clone(),
            });
            result.tenants_moved_out.push(notice);
        }
        
        // === Phase 5: Remove Expired Applications ===
        applications.retain(|app| !app.is_expired(current_tick));
        
        // === Phase 6: Generate New Applications ===
        let new_apps = generate_applications(
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
