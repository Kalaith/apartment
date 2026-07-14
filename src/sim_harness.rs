//! Headless balance-simulation harness.
//!
//! Drives the game's *pure* simulation logic (no macroquad/rendering) under a
//! set of scripted "landlord" strategies over the full 36-month game, records
//! per-month metrics, and writes a markdown balance report. This lets us
//! measure the economy empirically ("is it too easy to make money?") instead of
//! guessing, and re-run it after every balance tweak.
//!
//! Run it with:
//!   cargo test --lib balance -- --ignored --nocapture      (bin crate: `--bin` not needed)
//!   cargo test balance_report -- --ignored --nocapture
//!
//! The report is written to `balance_report.md` at the repo root.

use crate::building::{Building, DesignType, UpgradeAction};
use crate::consequences::{ComplianceSystem, InspectionTrigger};
use crate::data::config::GameConfig;
use crate::economy::{process_upgrade, FinancialLedger, PlayerFunds, Transaction, TransactionType};
use crate::simulation::{advance_tick, EventLog, GameOutcome};
use crate::tenant::matching::{evaluate_lease_offer, LeaseOffer};
use crate::tenant::{Tenant, TenantApplication, TenantArchetype};
use macroquad_toolkit::rng;

/// A scripted player policy. Every field is a lever the harness pulls each month.
#[derive(Clone, Copy)]
struct Strategy {
    name: &'static str,
    /// Reject applicants whose hidden reliability/behavior is poor (simulates a
    /// player who pays for vetting and turns risky tenants away).
    vet_applicants: bool,
    /// Repair any unit/hallway that falls below this condition.
    repair_threshold: i32,
    /// Spend surplus cash pushing unit designs up to Cozy.
    upgrade_designs: bool,
    /// Hire janitor/manager/security once affordable.
    hire_staff: bool,
    /// Keep at least this much cash before any discretionary spending.
    cash_reserve: i32,
}

/// Metrics captured at the end of each simulated month.
#[derive(Clone, Copy)]
struct MonthMetrics {
    month: u32,
    balance: i32,
    rent: i32,
    expenses: i32,
    occupancy: f32,
    avg_happiness: i32,
    avg_condition: i32,
    tenants: usize,
}

/// Aggregated outcome of a single full playthrough.
struct RunResult {
    months: Vec<MonthMetrics>,
    final_balance: i32,
    min_balance: i32,
    peak_balance: i32,
    month_full_occupancy: Option<u32>,
    end_occupancy: f32,
    total_rent: i64,
    total_expenses: i64,
    upgrades_bought: u32,
    outcome: Option<GameOutcome>,
}

struct Sim {
    building: Building,
    tenants: Vec<Tenant>,
    applications: Vec<TenantApplication>,
    funds: PlayerFunds,
    ledger: FinancialLedger,
    event_log: EventLog,
    compliance: ComplianceSystem,
    current_tick: u32,
    next_tenant_id: u32,
    config: GameConfig,
    upgrades_bought: u32,
}

impl Sim {
    /// Build the same starting position the real game uses: the first building
    /// template, its seeded tenant, and default starting funds.
    fn new(config: GameConfig) -> Self {
        let template =
            crate::data::templates::load_templates().and_then(|t| t.templates.into_iter().next());

        let (mut building, initial_tenant) = match template {
            Some(t) => (Building::from_template(&t), t.initial_tenant.clone()),
            None => (
                Building::new(
                    "Sim Building",
                    config.starting_conditions.building_floors,
                    config.starting_conditions.units_per_floor,
                ),
                None,
            ),
        };

        let mut next_tenant_id = 1u32;
        let mut tenants = Vec::new();

        if let Some(data) = initial_tenant {
            if let Some(archetype) = TenantArchetype::from_id(&data.archetype) {
                if let Some(apt) = building
                    .apartments
                    .iter_mut()
                    .find(|a| a.unit_number == data.apartment_unit)
                {
                    let id = next_tenant_id;
                    next_tenant_id += 1;
                    let mut tenant = Tenant::new(id, &data.name, archetype);
                    tenant.move_into(apt.id);
                    apt.move_in(id);
                    tenants.push(tenant);
                }
            }
        }

        let mut compliance = ComplianceSystem::new();
        compliance.init_building_regulations(0, false);

        Sim {
            building,
            tenants,
            applications: Vec::new(),
            funds: PlayerFunds::default(),
            ledger: FinancialLedger::default(),
            event_log: EventLog::new(),
            compliance,
            current_tick: 0,
            next_tenant_id,
            config,
            upgrades_bought: 0,
        }
    }

    /// Mirror the real game's monthly inspection + fine billing so the harness
    /// measures the regulatory teeth that punish neglect (the game runs these in
    /// `end_turn`, outside `advance_tick`).
    fn run_inspections_and_fines(&mut self) {
        self.compliance.tick(self.current_tick);

        let score = self
            .building
            .average_condition()
            .min(self.building.hallway_condition);
        let cfg = self.config.regulations.clone();
        let due = self.compliance.has_due_inspection(0);
        let random_check = rng::gen_range(0, 100) < cfg.random_inspection_chance_percent;
        if due || random_check {
            let trigger = if due {
                InspectionTrigger::Scheduled
            } else {
                InspectionTrigger::Random
            };
            self.compliance
                .run_inspection(0, score, self.current_tick, trigger, &cfg);
        }

        if self.compliance.unpaid_fines > 0 {
            let amount = self.compliance.unpaid_fines;
            self.funds.apply_required_expense(Transaction::expense(
                TransactionType::InspectionFine,
                amount,
                "Regulatory fines",
                self.current_tick,
            ));
            self.compliance.unpaid_fines = 0;
        }
    }

    fn total_units(&self) -> usize {
        self.building.apartments.len()
    }

    fn occupancy(&self) -> f32 {
        let total = self.total_units();
        if total == 0 {
            return 0.0;
        }
        self.building.occupancy_count() as f32 / total as f32
    }

    fn avg_happiness(&self) -> i32 {
        if self.tenants.is_empty() {
            return 0;
        }
        self.tenants.iter().map(|t| t.happiness).sum::<i32>() / self.tenants.len() as i32
    }

    /// List every vacant unit so it can draw applicants next tick.
    fn list_vacancies(&mut self) {
        for apt in &mut self.building.apartments {
            if apt.is_vacant() {
                apt.is_listed_for_lease = true;
            }
        }
    }

    /// Decide on each pending application. Accepting rolls the same lease-decline
    /// dice the real UI does, and a decline consumes the applicant (as in-game).
    fn handle_applications(&mut self, strat: &Strategy) {
        // The bot acts on every pending application each month, so we drain the
        // whole queue; a declined offer simply consumes the applicant.
        let applications = std::mem::take(&mut self.applications);
        for app in applications {
            if strat.vet_applicants
                && (app.tenant.rent_reliability < self.config.tenant_risk.unreliable_threshold
                    || app.tenant.behavior_score < self.config.tenant_risk.low_behavior_threshold)
            {
                continue;
            }

            let Some(apt) = self.building.get_apartment(app.apartment_id) else {
                continue;
            };
            if !apt.is_vacant() {
                continue;
            }

            let offer =
                LeaseOffer::from_config(apt.rent_price, &self.config.matching.lease_defaults);
            let accept_prob =
                evaluate_lease_offer(&app.tenant, &offer, &self.config.matching.lease_acceptance);
            let leverage_penalty = app.tenant.negotiation_leverage() as f32 * 0.002;
            let adjusted = (accept_prob - leverage_penalty).clamp(0.0, 1.0);

            if rng::gen_range(0.0, 1.0) > adjusted {
                // Tenant declined the offer — applicant is gone.
                continue;
            }

            let apartment_id = app.apartment_id;
            let mut tenant = app.tenant;
            tenant.move_into(apartment_id);
            if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                apt.move_in(tenant.id);
            }
            self.tenants.push(tenant);
        }
    }

    fn affordable(&self, cost: i32, reserve: i32) -> bool {
        self.funds.balance - cost >= reserve
    }

    /// Repairs, optional design upgrades, and optional staff hires.
    fn maintain(&mut self, strat: &Strategy) {
        let tick = self.current_tick;

        // Repair worn units (cheapest-first is irrelevant; just cap by reserve).
        let ids: Vec<u32> = self.building.apartments.iter().map(|a| a.id).collect();
        for id in ids {
            let (cond, cost_per) = {
                let apt = self.building.get_apartment(id).unwrap();
                (apt.condition, self.config.economy.repair_cost_per_point)
            };
            if cond < strat.repair_threshold {
                let amount = 100 - cond;
                let cost = amount * cost_per;
                if self.affordable(cost, strat.cash_reserve) {
                    let _ = process_upgrade(
                        &UpgradeAction::RepairApartment {
                            apartment_id: id,
                            amount,
                        },
                        &mut self.building,
                        &mut self.funds,
                        &self.config,
                        tick,
                    );
                }
            }
        }

        // Hallway.
        if self.building.hallway_condition < strat.repair_threshold {
            let amount = 100 - self.building.hallway_condition;
            let cost = amount * self.config.economy.hallway_repair_cost_per_point;
            if self.affordable(cost, strat.cash_reserve) {
                let _ = process_upgrade(
                    &UpgradeAction::RepairHallway { amount },
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    tick,
                );
            }
        }

        if strat.hire_staff {
            self.hire_staff();
        }

        if strat.upgrade_designs && self.affordable(12_000, strat.cash_reserve) {
            // Push one occupied unit up a design tier per month while flush.
            let target = self.building.apartments.iter().find_map(|a| {
                if a.tenant_id.is_some()
                    && matches!(a.design, DesignType::Bare | DesignType::Practical)
                {
                    Some(a.id)
                } else {
                    None
                }
            });
            if let Some(id) = target {
                if process_upgrade(
                    &UpgradeAction::UpgradeDesign { apartment_id: id },
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    tick,
                )
                .is_ok()
                {
                    self.upgrades_bought += 1;
                    // A real landlord reprices after investing in the unit —
                    // capture some of the upgrade's value as higher rent
                    // instead of leaving it purely as a sunk cosmetic cost.
                    if let Some(apt) = self.building.get_apartment_mut(id) {
                        apt.rent_price += (apt.rent_price as f32 * 0.15) as i32;
                    }
                }
            }
        }
    }

    /// Hire staff in priority order once there's comfortable headroom. Staff are
    /// modelled as building flags (salaries are charged from these in the tick).
    fn hire_staff(&mut self) {
        let headroom = self.funds.balance > 4_000;
        if !headroom {
            return;
        }
        for role in ["staff_janitor", "staff_manager", "staff_security"] {
            if !self.building.flags.contains(role) {
                self.building.flags.insert(role.to_string());
                break;
            }
        }
    }

    fn tick_expenses(&self) -> i32 {
        self.funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|t| t.amount < 0)
            .map(|t| t.amount.abs())
            .sum()
    }

    /// Play the full game under `strat` and return the aggregated result.
    fn run(mut self, strat: &Strategy, duration: u32) -> RunResult {
        let mut months = Vec::with_capacity(duration as usize);
        let mut min_balance = self.funds.balance;
        let mut peak_balance = self.funds.balance;
        let mut month_full_occupancy = None;
        let mut total_rent = 0i64;
        let mut total_expenses = 0i64;
        let mut outcome = None;
        let mut has_ever_had_tenant = false;

        for _ in 0..duration {
            self.list_vacancies();
            self.handle_applications(strat);
            self.maintain(strat);

            has_ever_had_tenant |= !self.tenants.is_empty();

            let result = advance_tick(
                &mut self.building,
                &mut self.tenants,
                &mut self.applications,
                &mut self.funds,
                &mut self.ledger,
                &mut self.event_log,
                &mut self.current_tick,
                &mut self.next_tenant_id,
                has_ever_had_tenant,
                1.0, // neutral reputation multiplier: the harness has no city layer
                &self.config,
            );

            // Apply the regulatory teeth that live outside advance_tick so the
            // report reflects the real cost of neglect.
            self.run_inspections_and_fines();

            let expenses = self.tick_expenses();
            let occupancy = self.occupancy();

            total_rent += result.rent_collected as i64;
            total_expenses += expenses as i64;
            min_balance = min_balance.min(self.funds.balance);
            peak_balance = peak_balance.max(self.funds.balance);
            if occupancy >= 1.0 && month_full_occupancy.is_none() {
                month_full_occupancy = Some(self.current_tick);
            }
            if outcome.is_none() {
                outcome = result.outcome.clone();
            }

            months.push(MonthMetrics {
                month: self.current_tick,
                balance: self.funds.balance,
                rent: result.rent_collected,
                expenses,
                occupancy,
                avg_happiness: self.avg_happiness(),
                avg_condition: self.building.average_condition(),
                tenants: self.tenants.len(),
            });
        }

        RunResult {
            months,
            final_balance: self.funds.balance,
            min_balance,
            peak_balance,
            month_full_occupancy,
            end_occupancy: self.occupancy(),
            total_rent,
            total_expenses,
            upgrades_bought: self.upgrades_bought,
            outcome,
        }
    }
}

/// Mean of the (rent - expenses) net over the final `window` months — the
/// "steady-state" monthly profit once the building is established.
fn steady_state_net(months: &[MonthMetrics], window: usize) -> i32 {
    if months.is_empty() {
        return 0;
    }
    let slice = &months[months.len().saturating_sub(window)..];
    let sum: i64 = slice.iter().map(|m| (m.rent - m.expenses) as i64).sum();
    (sum / slice.len() as i64) as i32
}

/// Averaged summary of many seeded runs of one strategy.
struct StrategySummary {
    name: &'static str,
    runs: usize,
    mean_final_balance: i64,
    mean_min_balance: i64,
    mean_peak_balance: i64,
    mean_end_occupancy: f32,
    mean_month_full: f32,
    never_full_count: usize,
    bankruptcy_count: usize,
    mean_steady_net: i64,
    mean_total_rent: i64,
    mean_total_expenses: i64,
    mean_upgrades: f32,
    sample_months: Vec<MonthMetrics>,
}

fn summarize(
    name: &'static str,
    config: &GameConfig,
    strat: &Strategy,
    seeds: u64,
) -> StrategySummary {
    let duration = config.win_conditions.game_duration_ticks.unwrap_or(36);

    let mut sum_final = 0i64;
    let mut sum_min = 0i64;
    let mut sum_peak = 0i64;
    let mut sum_end_occ = 0f32;
    let mut sum_month_full = 0u32;
    let mut full_runs = 0usize;
    let mut never_full = 0usize;
    let mut bankruptcies = 0usize;
    let mut sum_steady = 0i64;
    let mut sum_rent = 0i64;
    let mut sum_exp = 0i64;
    let mut sum_upgrades = 0u32;
    let mut sample_months = Vec::new();

    for seed in 0..seeds {
        rng::srand(0xA11CE ^ seed);
        let sim = Sim::new(config.clone());
        let result = sim.run(strat, duration);

        sum_final += result.final_balance as i64;
        sum_min += result.min_balance as i64;
        sum_peak += result.peak_balance as i64;
        sum_end_occ += result.end_occupancy;
        sum_rent += result.total_rent;
        sum_exp += result.total_expenses;
        sum_upgrades += result.upgrades_bought;
        sum_steady += steady_state_net(&result.months, 12) as i64;

        match result.month_full_occupancy {
            Some(m) => {
                sum_month_full += m;
                full_runs += 1;
            }
            None => never_full += 1,
        }
        if matches!(result.outcome, Some(GameOutcome::Bankruptcy { .. })) {
            bankruptcies += 1;
        }
        if seed == 0 {
            sample_months = result.months.clone();
        }
    }

    let runs = seeds as i64;
    StrategySummary {
        name,
        runs: seeds as usize,
        mean_final_balance: sum_final / runs,
        mean_min_balance: sum_min / runs,
        mean_peak_balance: sum_peak / runs,
        mean_end_occupancy: sum_end_occ / seeds as f32,
        mean_month_full: if full_runs > 0 {
            sum_month_full as f32 / full_runs as f32
        } else {
            f32::NAN
        },
        never_full_count: never_full,
        bankruptcy_count: bankruptcies,
        mean_steady_net: sum_steady / runs,
        mean_total_rent: sum_rent / runs,
        mean_total_expenses: sum_exp / runs,
        mean_upgrades: sum_upgrades as f32 / seeds as f32,
        sample_months,
    }
}

fn strategies() -> Vec<Strategy> {
    vec![
        Strategy {
            name: "Greedy (accept-all, minimal upkeep)",
            vet_applicants: false,
            repair_threshold: 55,
            upgrade_designs: false,
            hire_staff: false,
            cash_reserve: 500,
        },
        Strategy {
            name: "Investor (vet + repairs + upgrades + staff)",
            vet_applicants: true,
            repair_threshold: 75,
            upgrade_designs: true,
            hire_staff: true,
            cash_reserve: 1_500,
        },
        Strategy {
            name: "Slumlord (accept-all, never repair)",
            vet_applicants: false,
            repair_threshold: 0,
            upgrade_designs: false,
            hire_staff: false,
            cash_reserve: 0,
        },
    ]
}

fn format_report(config: &GameConfig, summaries: &[StrategySummary], seeds: u64) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let start = config.starting_conditions.player_money;
    let duration = config.win_conditions.game_duration_ticks.unwrap_or(36);

    writeln!(out, "# Apartment Manager — Balance Report").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "Headless simulation of {} strategies × {} seeds over {} months. \
         Starting cash **${}**.",
        summaries.len(),
        seeds,
        duration,
        start
    )
    .unwrap();
    writeln!(out).unwrap();

    writeln!(out, "## Summary (means across seeds)").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "| Strategy | Final $ | Min $ | Peak $ | Steady net $/mo | End occ. | Month→full | Never full | Bankrupt |"
    )
    .unwrap();
    writeln!(out, "|---|--:|--:|--:|--:|--:|--:|--:|--:|").unwrap();
    for s in summaries {
        let month_full = if s.mean_month_full.is_nan() {
            "—".to_string()
        } else {
            format!("{:.1}", s.mean_month_full)
        };
        writeln!(
            out,
            "| {} | {} | {} | {} | {} | {:.0}% | {} | {}/{} | {}/{} |",
            s.name,
            s.mean_final_balance,
            s.mean_min_balance,
            s.mean_peak_balance,
            s.mean_steady_net,
            s.mean_end_occupancy * 100.0,
            month_full,
            s.never_full_count,
            s.runs,
            s.bankruptcy_count,
            s.runs,
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    writeln!(out, "## Totals (means across seeds)").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "| Strategy | Total rent | Total expenses | Net | Design upgrades |"
    )
    .unwrap();
    writeln!(out, "|---|--:|--:|--:|--:|").unwrap();
    for s in summaries {
        writeln!(
            out,
            "| {} | {} | {} | {} | {:.1} |",
            s.name,
            s.mean_total_rent,
            s.mean_total_expenses,
            s.mean_total_rent - s.mean_total_expenses,
            s.mean_upgrades,
        )
        .unwrap();
    }
    writeln!(out).unwrap();

    // Month-by-month trajectory for the greedy strategy (seed 0) — the case the
    // player reported as "too easy".
    if let Some(greedy) = summaries.first() {
        writeln!(out, "## Month-by-month — {} (seed 0)", greedy.name).unwrap();
        writeln!(out).unwrap();
        writeln!(
            out,
            "| Mo | Balance | Rent | Expenses | Net | Occ | Tenants | Avg happy | Avg cond |"
        )
        .unwrap();
        writeln!(out, "|--:|--:|--:|--:|--:|--:|--:|--:|--:|").unwrap();
        for m in &greedy.sample_months {
            writeln!(
                out,
                "| {} | {} | {} | {} | {} | {:.0}% | {} | {} | {} |",
                m.month,
                m.balance,
                m.rent,
                m.expenses,
                m.rent - m.expenses,
                m.occupancy * 100.0,
                m.tenants,
                m.avg_happiness,
                m.avg_condition,
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }

    writeln!(out, "## Read-out").unwrap();
    writeln!(out).unwrap();
    for s in summaries {
        let growth = if start != 0 {
            s.mean_final_balance as f32 / start as f32
        } else {
            0.0
        };
        writeln!(
            out,
            "- **{}**: ends with ~${} ({:.1}× start), steady net ~${}/mo, {:.0}% occupied. \
             Bankruptcies {}/{}.",
            s.name,
            s.mean_final_balance,
            growth,
            s.mean_steady_net,
            s.mean_end_occupancy * 100.0,
            s.bankruptcy_count,
            s.runs,
        )
        .unwrap();
    }

    out
}

/// Run the full harness and produce the balance report string.
fn generate_report(seeds: u64) -> String {
    let config = crate::data::config::load_config();
    let strategies = strategies();
    let summaries: Vec<StrategySummary> = strategies
        .iter()
        .map(|strat| summarize(strat.name, &config, strat, seeds))
        .collect();
    format_report(&config, &summaries, seeds)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fast smoke test (always runs in CI): a single greedy playthrough must
    /// complete 36 months without panicking and stay internally consistent.
    #[test]
    fn balance_harness_runs_without_panic() {
        rng::srand(1);
        let config = crate::data::config::load_config();
        let strat = strategies()[0];
        let duration = config.win_conditions.game_duration_ticks.unwrap_or(36);
        let result = Sim::new(config).run(&strat, duration);

        assert_eq!(result.months.len() as u32, duration);
        assert!(result.end_occupancy >= 0.0 && result.end_occupancy <= 1.0);
    }

    /// Full balance report. Ignored by default (writes a file, runs many seeds).
    /// Run with: `cargo test balance_report -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn balance_report() {
        let seeds = 60;
        let report = generate_report(seeds);
        println!("\n{}\n", report);

        std::fs::write("balance_report.md", &report).expect("write balance_report.md");
        println!("Wrote balance_report.md");
    }
}
