use macroquad_toolkit::rng;
// Monthly turn advancement for gameplay state. The narrative, inspection,
// neighborhood, and awards halves of the turn live in sibling modules.

use crate::economy::{Transaction, TransactionType};
use crate::simulation::{advance_tick, ActiveWorldEvent, ActiveWorldEventKind, GameEvent};
use crate::ui::colors;
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// End the current turn and advance time.
    pub fn end_turn(&mut self) {
        // Latch once the building has ever been occupied, so the "all tenants left"
        // loss can distinguish real mass-departure from a not-yet-filled building.
        self.has_ever_had_tenant |= !self.tenants.is_empty();

        let reputation_multiplier = self.application_reputation_multiplier();

        let result = advance_tick(
            &mut self.building,
            &mut self.tenants,
            &mut self.applications,
            &mut self.funds,
            &mut self.ledger,
            &mut self.event_log,
            &mut self.current_tick,
            &mut self.next_tenant_id,
            self.has_ever_had_tenant,
            reputation_multiplier,
            &self.config,
        );

        self.game_outcome = result.outcome.clone();
        self.spawn_tick_feedback(&result.events);
        self.register_active_world_events(&result.events);
        self.apply_active_world_events();
        self.apply_active_tax_breaks();
        self.update_city_systems();
        self.collect_portfolio_passive_income();
        self.generate_monthly_narrative(&result);
        self.generate_tenant_life_events();
        self.auto_approve_manager_requests();
        self.expire_narrative_events();
        self.sync_building();
        self.missions.generate_available_missions(self.current_tick);

        if self.current_tick.is_multiple_of(12) && self.current_tick > 0 {
            self.check_annual_awards();
        }

        self.apply_monthly_social_happiness();
        self.log_monthly_status();
        self.update_context_hints();
        self.check_game_completion();
        // Record the tick result before evaluating missions so goals like
        // PerfectCollection can inspect this month's rent outcome.
        self.last_tick_result = Some(result);
        self.update_missions();
        self.autosave_current_game();
    }

    fn spawn_tick_feedback(&mut self, events: &[GameEvent]) {
        for event in events {
            match event {
                GameEvent::RentPaid { amount, .. } => self.spawn_center_text(
                    &format!("+${}", amount),
                    rng::gen_range(-50.0, 50.0),
                    rng::gen_range(-50.0, 50.0),
                    colors::POSITIVE,
                ),
                GameEvent::RentMissed { .. } => {
                    self.spawn_center_text("Missed Rent!", 0.0, 0.0, colors::NEGATIVE);
                }
                GameEvent::TenantUnhappy { .. } => self.spawn_center_text(
                    "Unhappy!",
                    rng::gen_range(-50.0, 50.0),
                    rng::gen_range(-50.0, 50.0),
                    colors::WARNING,
                ),
                _ => {}
            }
        }
    }

    pub(super) fn spawn_center_text(
        &mut self,
        text: &str,
        offset_x: f32,
        offset_y: f32,
        color: Color,
    ) {
        self.floating_texts.spawn(
            text,
            vec2(
                screen_width() / 2.0 + offset_x,
                screen_height() / 2.0 + offset_y,
            ),
            color,
        );
    }

    fn register_active_world_events(&mut self, events: &[GameEvent]) {
        for event in events {
            match event {
                GameEvent::Heatwave { tick_duration } => {
                    self.add_active_world_event(ActiveWorldEventKind::Heatwave, *tick_duration);
                }
                GameEvent::Gentrification { tick_duration, .. } => {
                    self.add_active_world_event(
                        ActiveWorldEventKind::Gentrification,
                        *tick_duration,
                    );
                }
                _ => {}
            }
        }
    }

    fn add_active_world_event(&mut self, kind: ActiveWorldEventKind, duration: u32) {
        if duration == 0 {
            return;
        }

        if let Some(existing) = self
            .active_world_events
            .iter_mut()
            .find(|event| event.kind == kind)
        {
            existing.remaining_ticks = existing.remaining_ticks.max(duration);
            return;
        }

        self.active_world_events
            .push(ActiveWorldEvent::new(kind, duration));
    }

    fn apply_active_world_events(&mut self) {
        let mut heatwave_active = false;
        let mut gentrification_active = false;

        for event in &mut self.active_world_events {
            match &event.kind {
                ActiveWorldEventKind::Heatwave => {
                    heatwave_active = true;
                }
                ActiveWorldEventKind::Gentrification => {
                    gentrification_active = true;
                }
            }
            event.tick();
        }

        if heatwave_active {
            for tenant in &mut self.tenants {
                tenant.happiness = (tenant.happiness - 3).max(0);
            }
        }

        if gentrification_active {
            self.gentrification.gentrification_score = (self.gentrification.gentrification_score
                + 1)
            .min(self.config.gentrification.max_gentrification_score);

            for neighborhood in &mut self.city.neighborhoods {
                neighborhood.stats.gentrification =
                    (neighborhood.stats.gentrification + 1).min(100);
                neighborhood.stats.rent_demand = (neighborhood.stats.rent_demand + 0.02).min(2.0);
            }
        }

        self.active_world_events
            .retain(|event| event.remaining_ticks > 0);
    }

    fn update_city_systems(&mut self) {
        self.save_building_to_city();
        self.city.tick();

        let (rel_changes, rel_events) = self.tenant_network.tick(
            &self.tenants,
            &self.building,
            &self.config.relationships,
            &self.relationship_events_config,
            self.current_tick,
        );
        self.notifications.add_relationship_changes(rel_changes);
        for mut event in rel_events {
            event.month = self.current_tick;
            if event.requires_response {
                event.response_deadline = Some(self.current_tick + 2);
            }
            let immediate_effect = if event.requires_response {
                None
            } else {
                Some(event.default_effect.clone())
            };
            self.narrative_events.add_event(event);
            if let Some(effect) = immediate_effect {
                self.apply_narrative_effect(&effect);
            }
        }

        self.compliance.tick(self.current_tick);
        self.run_due_inspections();
        self.gentrification
            .update_affordable_units(&self.building.apartments, &self.config.gentrification);
    }

    /// Portfolio-lite: buildings you own but aren't actively managing run
    /// themselves at a simplified steady state and contribute passive net income
    /// each month. The active building is fully simulated by `advance_tick` and
    /// excluded here.
    pub(super) fn collect_portfolio_passive_income(&mut self) {
        let active = self.city.active_building_index;
        let cfg = &self.config.portfolio;
        let mut net = 0i32;
        let mut earning = 0u32;
        for (i, building) in self.city.buildings.iter().enumerate() {
            if i == active || building.apartments.is_empty() {
                continue;
            }
            let potential: i32 = building.apartments.iter().map(|a| a.rent_price).sum();
            let income = (potential as f32 * cfg.passive_occupancy) as i32;
            let cost = building.apartments.len() as i32 * cfg.passive_cost_per_unit;
            net += income - cost;
            earning += 1;
        }

        if earning == 0 || net == 0 {
            return;
        }

        if net > 0 {
            self.funds.add_income(Transaction::income(
                TransactionType::RentIncome,
                net,
                "Portfolio passive income",
                self.current_tick,
            ));
        } else {
            self.funds.apply_required_expense(Transaction::expense(
                TransactionType::Mortgage,
                net.abs(),
                "Portfolio upkeep",
                self.current_tick,
            ));
        }

        self.event_log.log(
            GameEvent::Notification {
                message: format!(
                    "Your other {} propert{} netted {:+} this month.",
                    earning,
                    if earning == 1 { "y" } else { "ies" },
                    net
                ),
                level: crate::simulation::NotificationLevel::Info,
            },
            self.current_tick,
        );
    }

    fn autosave_current_game(&mut self) {
        if let Err(error) = crate::save::save_game(self) {
            eprintln!("Failed to save game: {}", error);
            self.spawn_center_text("Save Failed!", 0.0, 0.0, colors::NEGATIVE);
        }
    }

    fn apply_monthly_social_happiness(&mut self) {
        let cohesion = self
            .tenant_network
            .calculate_cohesion(&self.tenants, &self.config.cohesion);
        for tenant in &mut self.tenants {
            let relationship_bonus = crate::tenant::happiness::calculate_relationship_happiness(
                tenant.id,
                &self.tenant_network,
                &self.config.relationships,
            );
            let cohesion_bonus = if cohesion > 20 { 1 } else { 0 };
            tenant.happiness =
                (tenant.happiness + relationship_bonus + cohesion_bonus).clamp(0, 100);
        }
    }

    fn log_monthly_status(&self) {
        let marketing_name = self.building.marketing_strategy.name();
        if marketing_name != "None" {
            println!("Active marketing campaign: {}", marketing_name);
        }

        let pending_count = self.dialogue_system.pending_dialogues().len();
        if pending_count > 0 {
            println!("Pending dialogues: {}", pending_count);
        }
    }

    fn update_context_hints(&mut self) {
        let total_units = self.building.apartments.len();
        let vacancy_count = self
            .building
            .apartments
            .iter()
            .filter(|apartment| apartment.is_vacant())
            .count();
        let avg_condition = self.building.average_condition();
        let any_unhappy = self
            .tenants
            .iter()
            .any(|tenant| tenant.is_unhappy(self.config.happiness.unhappy_threshold));

        self.notifications.check_context_hints(
            self.current_tick,
            vacancy_count,
            total_units,
            avg_condition,
            self.funds.balance,
            any_unhappy,
        );
    }

    fn check_game_completion(&mut self) {
        let duration = self.config.win_conditions.game_duration_ticks.unwrap_or(36);
        if self.current_tick < duration || self.game_outcome.is_some() {
            return;
        }

        self.game_outcome = Some(crate::simulation::GameOutcome::Victory {
            score: 0,
            months: self.current_tick,
            total_income: self.funds.total_income,
        });
        self.view_mode = ViewMode::CareerSummary;
        self.unlock_next_building();
        self.check_final_achievements();
    }

    fn check_final_achievements(&mut self) {
        let new_unlocks = self.achievements.check_new_unlocks(
            &self.city,
            &self.building,
            &self.tenants,
            &self.funds,
            self.current_tick,
            &self.config,
        );
        for id in new_unlocks {
            self.achievements.unlock(&id);
        }
    }
}
