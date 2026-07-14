use super::{decay, win_condition, EventLog, EventSystem, GameEvent, GameOutcome};
use crate::building::Building;
use crate::economy::{
    collect_rent, FinancialLedger, OperatingCosts, PlayerFunds, Transaction, TransactionType,
};
use crate::tenant::{
    calculate_happiness, generate_applications, process_departures, Tenant, TenantApplication,
};

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
    #[allow(clippy::too_many_arguments)]
    pub fn process(
        building: &mut Building,
        tenants: &mut Vec<Tenant>,
        applications: &mut Vec<TenantApplication>,
        funds: &mut PlayerFunds,
        ledger: &mut FinancialLedger,
        event_log: &mut EventLog,
        current_tick: u32,
        next_tenant_id: &mut u32,
        has_ever_had_tenant: bool,
        reputation_multiplier: f32,
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
        Self::collect_rent(building, tenants, funds, current_tick, config, &mut result);

        // 2. Operating Costs & Staff
        Self::process_operating_costs(building, funds, current_tick, &mut result, config);
        Self::process_critical_failures(
            building,
            tenants,
            funds,
            current_tick,
            &mut result,
            config,
        );

        // 3. Random Events
        let mut event_system = EventSystem::new();
        let random_events = event_system.check_events(building, funds, current_tick);
        result.events.extend(random_events);

        // 4. Decay & Ownership
        if building.update_ownership(current_tick) {
            // Logic for handling ownership updates could go here
        }
        let decay_events = decay::apply_decay(building, &config.decay, &config.thresholds);
        result.events.extend(decay_events);

        // 4b. Staff maintenance offsets decay; disruptive tenants add damage.
        Self::process_janitor_maintenance(building, &mut result, config);
        Self::process_tenant_risk(building, tenants, config, &mut result);

        // 5. Tenant Happiness & Updates
        Self::update_tenants(
            building,
            tenants,
            &mut result,
            &config.happiness,
            &config.staff_effects,
        );

        // 6. Move-outs
        let departure_notices = process_departures(tenants, building, &config.happiness);
        for notice in departure_notices {
            result.events.push(GameEvent::TenantMovedOut {
                message: notice.clone(),
            });
            result.tenants_moved_out.push(notice);
        }

        // 7. Applications
        applications.retain(|app| {
            !app.is_expired_after(current_tick, config.applications.expire_after_ticks)
        });
        let new_apps = generate_applications(
            building,
            applications,
            current_tick,
            next_tenant_id,
            reputation_multiplier,
            config,
        );
        result.new_applications = new_apps.len();

        for app in &new_apps {
            result.events.push(GameEvent::NewApplication {
                tenant_name: app.tenant.name.clone(),
                archetype: format!("{:?}", app.tenant.archetype),
                apartment_unit: building
                    .get_apartment(app.apartment_id)
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
            has_ever_had_tenant,
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
        tenants: &[Tenant],
        funds: &mut PlayerFunds,
        current_tick: u32,
        config: &crate::data::config::GameConfig,
        result: &mut TickResult,
    ) {
        let rent_result = collect_rent(tenants, building, funds, current_tick, &config.tenant_risk);
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
            let transaction = Transaction::expense(
                TransactionType::Marketing,
                marketing_cost,
                &format!("{} Marketing Campaign", building.marketing_strategy.name()),
                current_tick,
            );
            if !funds.deduct_expense(transaction) {
                building.marketing_strategy = crate::building::MarketingType::None;
                result.events.push(GameEvent::Notification {
                    message: "Marketing campaign cancelled due to lack of funds.".to_string(),
                    level: crate::simulation::NotificationLevel::Warning,
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

        // Fixed monthly overhead (mortgage/upkeep) — always-on structural cost.
        let overhead = OperatingCosts::calculate_base_overhead(building, &config.operating_costs);
        if overhead > 0 {
            funds.apply_required_expense(Transaction::expense(
                TransactionType::Mortgage,
                overhead,
                "Mortgage & Upkeep",
                current_tick,
            ));
        }

        // Taxes & Expenses
        let tax = OperatingCosts::calculate_property_tax(
            building,
            result.rent_collected,
            &config.operating_costs,
            current_tick,
        );
        if tax > 0 {
            funds.apply_required_expense(Transaction::expense(
                TransactionType::PropertyTax,
                tax,
                "Monthly Property Tax",
                current_tick,
            ));
        }

        let utilities = OperatingCosts::calculate_utilities(building, &config.operating_costs);
        if utilities > 0 {
            funds.apply_required_expense(Transaction::expense(
                TransactionType::Utilities,
                utilities,
                "Utility Bills",
                current_tick,
            ));
        }

        let insurance = OperatingCosts::calculate_insurance(building, &config.operating_costs);
        if insurance > 0 {
            funds.apply_required_expense(Transaction::expense(
                TransactionType::Insurance,
                insurance,
                "Property Insurance",
                current_tick,
            ));
        }

        // Staff Salaries - Data Driven
        let salaries = OperatingCosts::calculate_staff_salaries(building, &config.economy);
        if salaries > 0 {
            funds.apply_required_expense(Transaction::expense(
                TransactionType::StaffSalary,
                salaries,
                "Staff Salaries",
                current_tick,
            ));
        }
    }

    /// Janitor maintenance runs *after* decay so it genuinely offsets it:
    /// the most-worn `janitor_units_maintained` units (and the hallway) are
    /// repaired by exactly one month of decay, so the player only maintains
    /// the units the janitor can't cover.
    fn process_janitor_maintenance(
        building: &mut Building,
        result: &mut TickResult,
        config: &crate::data::config::GameConfig,
    ) {
        if !building.flags.contains("staff_janitor") {
            return;
        }

        let apt_decay = config.decay.apartment_per_tick;
        let hallway_decay = config.decay.hallway_per_tick;
        let units_maintained = config.staff_effects.janitor_units_maintained;
        let apt_cost = config.economy.repair_cost_per_point;
        let hallway_cost = config.economy.hallway_repair_cost_per_point;

        // Repair the lowest-condition units first (most in need of upkeep), and
        // tally the repair cost the janitor's upkeep spared us.
        let mut indices: Vec<usize> = (0..building.apartments.len()).collect();
        indices.sort_by_key(|&i| building.apartments[i].condition);
        let mut value_saved = 0;
        for &i in indices.iter().take(units_maintained) {
            let before = building.apartments[i].condition;
            building.apartments[i].repair(apt_decay);
            value_saved += (building.apartments[i].condition - before) * apt_cost;
        }

        let hall_before = building.hallway_condition;
        building.repair_hallway(hallway_decay);
        value_saved += (building.hallway_condition - hall_before) * hallway_cost;

        // Make the janitor's contribution legible — this is upkeep the landlord
        // would otherwise have to pay for (or lose to decay-driven fines).
        if value_saved > 0 {
            result.events.push(GameEvent::Notification {
                message: format!(
                    "Janitor upkeep offset ~${} of wear this month.",
                    value_saved
                ),
                level: crate::simulation::NotificationLevel::Info,
            });
        }
    }

    fn process_critical_failures(
        building: &mut Building,
        tenants: &mut [Tenant],
        funds: &mut PlayerFunds,
        current_tick: u32,
        result: &mut TickResult,
        config: &crate::data::config::GameConfig,
    ) {
        use macroquad_toolkit::rng;

        let base_prob = 5; // 0.5% as integer (out of 1000)
        let mut prob = base_prob;
        // Security reduces failure probability
        if building.flags.contains("staff_security") {
            let reduction = config
                .staff_effects
                .security_failure_reduction_percent
                .clamp(0, 100);
            prob = prob * (100 - reduction) / 100;
        }

        // Boiler Failure (prob out of 1000)
        if rng::gen_range(0, 1000) < prob {
            let cost = 1500;
            if funds.can_afford(cost) {
                funds.deduct_expense(Transaction::expense(
                    TransactionType::CriticalFailure,
                    cost,
                    "Boiler Emergency Repair",
                    current_tick,
                ));
                result.events.push(GameEvent::BoilerFailure { cost });
            } else {
                result.events.push(GameEvent::TenantUnhappy {
                    tenant_name: "ALL TENANTS".to_string(),
                    happiness: 0,
                });
                for t in tenants.iter_mut() {
                    t.happiness = (t.happiness - 30).max(0);
                }
                result.events.push(GameEvent::InsufficientFunds {
                    action: "Fix Boiler".to_string(),
                    needed: cost,
                    available: funds.balance,
                });
            }
        }

        // Structural Issue
        if rng::gen_range(0, 1000) < prob {
            let cost = 2500;
            let tx = Transaction::expense(
                TransactionType::CriticalFailure,
                cost,
                "Structural Reinforcement",
                current_tick,
            );
            if funds.deduct_expense(tx) {
                result.events.push(GameEvent::StructuralIssue {
                    cost,
                    description: "Foundation Crack".to_string(),
                });
            } else {
                building.hallway_condition = (building.hallway_condition - 20).max(0);
                result.events.push(GameEvent::HallwayDeteriorating {
                    condition: building.hallway_condition,
                });
                result.events.push(GameEvent::InsufficientFunds {
                    action: "Fix Foundation".to_string(),
                    needed: cost,
                    available: funds.balance,
                });
            }
        }
    }

    /// Low-quality tenants create real, visible losses so that vetting and
    /// rejecting risky applicants actually matters. Disruptive (low behavior)
    /// tenants damage their own unit and the shared hallway; unreliable rent
    /// payers are handled in `collect_rent`.
    fn process_tenant_risk(
        building: &mut Building,
        tenants: &[Tenant],
        config: &crate::data::config::GameConfig,
        result: &mut TickResult,
    ) {
        use macroquad_toolkit::rng;

        let risk = &config.tenant_risk;

        for tenant in tenants {
            let Some(apt_id) = tenant.apartment_id else {
                continue;
            };
            if tenant.behavior_score >= risk.low_behavior_threshold {
                continue;
            }
            if rng::gen_range(0, 100) >= risk.damage_chance_percent {
                continue;
            }

            let unit_number = building
                .get_apartment(apt_id)
                .map(|a| a.unit_number.clone())
                .unwrap_or_default();

            if let Some(apt) = building.get_apartment_mut(apt_id) {
                apt.decay_condition(risk.damage_amount);
            }
            building.decay_hallway(risk.hallway_disturbance_amount);

            result.events.push(GameEvent::TenantDamage {
                tenant_name: tenant.name.clone(),
                apartment_unit: unit_number,
                damage: risk.damage_amount,
            });
        }
    }

    fn update_tenants(
        building: &Building,
        tenants: &mut [Tenant],
        result: &mut TickResult,
        config: &crate::data::config::HappinessConfig,
        staff: &crate::data::config::StaffEffectsConfig,
    ) {
        for tenant in tenants.iter_mut() {
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apartment) = building.get_apartment(apt_id) {
                    let factors = calculate_happiness(tenant, apartment, building, config, staff);
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
#[allow(clippy::too_many_arguments)]
pub fn advance_tick(
    building: &mut Building,
    tenants: &mut Vec<Tenant>,
    applications: &mut Vec<TenantApplication>,
    funds: &mut PlayerFunds,
    ledger: &mut FinancialLedger,
    event_log: &mut EventLog,
    current_tick: &mut u32,
    next_tenant_id: &mut u32,
    has_ever_had_tenant: bool,
    reputation_multiplier: f32,
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
        has_ever_had_tenant,
        reputation_multiplier,
        config,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::Building;
    use crate::data::config::GameConfig;
    use crate::tenant::{Tenant, TenantArchetype};

    fn empty_result() -> TickResult {
        TickResult {
            events: Vec::new(),
            rent_collected: 0,
            tenants_moved_out: Vec::new(),
            new_applications: 0,
            outcome: None,
        }
    }

    #[test]
    fn janitor_offsets_decay_on_maintained_units() {
        let mut config = GameConfig::default();
        config.decay.apartment_per_tick = 3;
        config.staff_effects.janitor_units_maintained = 5;

        let mut building = Building::new("Test", 3, 2); // 6 units at condition 50
        building.flags.insert("staff_janitor".to_string());

        building.apply_monthly_decay(3, 1); // every unit -> 47
        let mut result = empty_result();
        GameTick::process_janitor_maintenance(&mut building, &mut result, &config);

        // 5 of 6 units are restored to their pre-decay condition; one is not.
        let restored = building
            .apartments
            .iter()
            .filter(|a| a.condition == 50)
            .count();
        let unmaintained = building
            .apartments
            .iter()
            .filter(|a| a.condition == 47)
            .count();
        assert_eq!(restored, 5);
        assert_eq!(unmaintained, 1);
    }

    #[test]
    fn low_behavior_tenant_damages_property() {
        let mut config = GameConfig::default();
        config.tenant_risk.low_behavior_threshold = 100;
        config.tenant_risk.damage_chance_percent = 100;
        config.tenant_risk.damage_amount = 6;

        let mut building = Building::new("Test", 1, 1);
        let apt_id = building.apartments[0].id;
        let before = building.apartments[0].condition;

        let mut tenant = Tenant::new(1, "Risky", TenantArchetype::Student);
        tenant.behavior_score = 10;
        tenant.apartment_id = Some(apt_id);
        let tenants = vec![tenant];

        let mut result = empty_result();
        GameTick::process_tenant_risk(&mut building, &tenants, &config, &mut result);

        assert_eq!(building.apartments[0].condition, before - 6);
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::TenantDamage { .. })));
    }
}
