use macroquad_toolkit::rng;
// Monthly turn advancement for gameplay state.

use crate::economy::{Transaction, TransactionType};
use crate::simulation::{
    advance_tick, ActiveWorldEvent, ActiveWorldEventKind, GameEvent, TickResult,
};
use crate::ui::{colors, FloatingText};
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

    fn spawn_center_text(&mut self, text: &str, offset_x: f32, offset_y: f32, color: Color) {
        self.floating_texts.push(FloatingText::new(
            text,
            screen_width() / 2.0 + offset_x,
            screen_height() / 2.0 + offset_y,
            color,
        ));
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

    /// Run any scheduled or random building inspections for the active building,
    /// bill the resulting fines, and reflect the outcome in reputation. A
    /// well-maintained building (condition at/above the configured threshold)
    /// passes cleanly; a neglected one gets cited and fined — the economic teeth
    /// that make repairs and upgrades matter.
    fn run_due_inspections(&mut self) {
        let building_id = self.city.active_building_index as u32;
        let due = self.compliance.has_due_inspection(building_id);
        let random_check = macroquad_toolkit::rng::gen_range(0, 100)
            < self.config.regulations.random_inspection_chance_percent;

        if due || random_check {
            let trigger = if due {
                crate::consequences::InspectionTrigger::Scheduled
            } else {
                crate::consequences::InspectionTrigger::Random
            };
            self.execute_inspection(trigger);
        }

        self.bill_outstanding_fines();
    }

    /// Run a single inspection of the active building with the given trigger,
    /// grade it against current condition, and surface the outcome (reputation
    /// move, event-log entry, floating text). Fines accrue to
    /// `compliance.unpaid_fines`; call `bill_outstanding_fines` to charge them.
    pub(super) fn execute_inspection(&mut self, trigger: crate::consequences::InspectionTrigger) {
        let building_id = self.city.active_building_index as u32;
        let inspection_score = self
            .building
            .average_condition()
            .min(self.building.hallway_condition);
        let config = self.config.regulations.clone();

        let inspection = self.compliance.run_inspection(
            building_id,
            inspection_score,
            self.current_tick,
            trigger,
            &config,
        );

        let citations = inspection.results.iter().filter(|r| !r.passed).count();
        if citations > 0 {
            self.adjust_active_neighborhood_reputation(-config.neighborhood_reputation_penalty);
            self.event_log.log(
                GameEvent::Notification {
                    message: format!(
                        "Inspection failed: {} citation(s), {} in fines.",
                        citations, inspection.total_fines
                    ),
                    level: crate::simulation::NotificationLevel::Warning,
                },
                self.current_tick,
            );
            self.floating_texts.push(FloatingText::new(
                &format!("Inspection: {} cited!", citations),
                screen_width() / 2.0,
                screen_height() / 2.0,
                colors::NEGATIVE,
            ));
        } else if !inspection.results.is_empty() {
            self.adjust_active_neighborhood_reputation(config.neighborhood_reputation_gain);
            self.floating_texts.push(FloatingText::new(
                "Inspection passed",
                screen_width() / 2.0,
                screen_height() / 2.0,
                colors::POSITIVE,
            ));
        }
    }

    /// Charge any outstanding regulatory fines (from inspections or missed fix
    /// deadlines escalated in `ComplianceSystem::tick`) as a required expense, so
    /// persistent neglect can genuinely push a landlord toward bankruptcy.
    pub(super) fn bill_outstanding_fines(&mut self) {
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

    /// Nudge the visible reputation of the neighborhood the active building sits
    /// in, clamped to [0, 100].
    pub(super) fn adjust_active_neighborhood_reputation(&mut self, delta: i32) {
        let building_id = self.city.active_building_index as u32;
        if let Some(neighborhood) = self
            .city
            .neighborhoods
            .iter_mut()
            .find(|n| n.building_ids.contains(&building_id))
        {
            neighborhood.reputation = (neighborhood.reputation + delta).clamp(0, 100);
        }
    }

    /// Gentrification pressure (0–100) of the neighborhood the active building
    /// sits in, defaulting to 0 when the building isn't placed yet.
    pub(super) fn active_neighborhood_gentrification(&self) -> i32 {
        let building_id = self.city.active_building_index as u32;
        self.city
            .neighborhoods
            .iter()
            .find(|n| n.building_ids.contains(&building_id))
            .map(|n| n.stats.gentrification)
            .unwrap_or(0)
    }

    /// Market multiplier applied to a condo's base value at sale time. A booming
    /// economy and a gentrifying neighborhood raise it well above 1.0; a
    /// recession drags it below — so timing a sale to a hot market is worth real
    /// money, giving selling a purpose beyond emergency liquidity.
    pub(super) fn condo_sale_market_multiplier(&self) -> f32 {
        let economy = self.city.economy_health; // 0.5..1.5
        let gentrification = self.active_neighborhood_gentrification() as f32 / 100.0;
        let boom_bonus = self.config.gentrification.condo_sale_boom_bonus;
        (economy * (1.0 + gentrification * boom_bonus)).clamp(0.4, 2.5)
    }

    /// Reputation of the neighborhood the active building sits in (0–100),
    /// defaulting to the neutral 50 when the building isn't placed yet.
    pub(super) fn active_neighborhood_reputation(&self) -> i32 {
        let building_id = self.city.active_building_index as u32;
        self.city
            .neighborhoods
            .iter()
            .find(|n| n.building_ids.contains(&building_id))
            .map(|n| n.reputation)
            .unwrap_or(50)
    }

    /// Applicant-volume multiplier derived from the active neighborhood's
    /// reputation. Neutral reputation (50) yields 1.0; a strong reputation draws
    /// proportionally more applicants and a poor one drives them away — the
    /// consequence that makes reputation worth cultivating.
    pub(super) fn application_reputation_multiplier(&self) -> f32 {
        let reputation = self.active_neighborhood_reputation();
        let influence = self.config.applications.reputation_influence;
        (1.0 + (reputation - 50) as f32 / 50.0 * influence).clamp(0.25, 2.0)
    }

    /// Apply a reputation change to a specific neighborhood (or the active
    /// building's neighborhood when `neighborhood_id` is `None`) with feedback.
    /// This is the write path that makes reputation a currency the player moves
    /// through event choices and mission rewards.
    pub(super) fn apply_reputation_change(&mut self, delta: i32, neighborhood_id: Option<u32>) {
        if delta == 0 {
            return;
        }
        match neighborhood_id {
            Some(id) => {
                if let Some(neighborhood) = self.city.neighborhoods.iter_mut().find(|n| n.id == id)
                {
                    neighborhood.reputation = (neighborhood.reputation + delta).clamp(0, 100);
                }
            }
            None => self.adjust_active_neighborhood_reputation(delta),
        }

        let color = if delta >= 0 {
            colors::POSITIVE
        } else {
            colors::NEGATIVE
        };
        self.floating_texts.push(FloatingText::new(
            &format!("Rep {:+}", delta),
            screen_width() / 2.0,
            screen_height() / 2.0 + 60.0,
            color,
        ));
    }

    fn generate_monthly_narrative(&mut self, result: &TickResult) {
        self.narrative_events.generate_events(
            self.current_tick,
            &self.city.neighborhoods,
            &self.city.buildings,
            &self.tenants,
        );

        let expenses = self
            .funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| transaction.amount < 0)
            .map(|transaction| transaction.amount.abs())
            .sum();
        self.mailbox.generate_mail(
            self.current_tick,
            result.rent_collected,
            expenses,
            &self.tenants,
            &self.city.buildings,
        );
        self.mailbox.cleanup(self.current_tick, 12);

        self.generate_dialogues();
        self.accept_available_missions();
        self.generate_tenant_requests();
    }

    fn generate_dialogues(&mut self) {
        let tenants = self.tenants.clone();
        let building = self.building.clone();
        let funds = self.funds.clone();
        self.dialogue_system.generate_dialogues(
            self.current_tick,
            &tenants,
            &building,
            &funds,
            &self.tenant_network,
        );
    }

    fn accept_available_missions(&mut self) {
        let available_ids: Vec<u32> = self
            .missions
            .available_missions()
            .iter()
            .map(|mission| mission.id)
            .collect();
        for mission_id in available_ids {
            self.missions.accept_mission(mission_id, self.current_tick);
        }
    }

    /// With a manager employed, routine tenant requests are handled for you
    /// (approved) instead of piling up as manual to-dos — the manager's job.
    fn auto_approve_manager_requests(&mut self) {
        if !self.config.staff_effects.manager_auto_approve_requests
            || !self.building.flags.contains("staff_manager")
        {
            return;
        }

        let tenant_ids: Vec<u32> = self
            .tenant_stories
            .iter()
            .filter(|(_, story)| story.pending_request.is_some())
            .map(|(id, _)| *id)
            .collect();

        for tenant_id in tenant_ids {
            let effect = self.tenant_stories.get_mut(&tenant_id).and_then(|story| {
                story.pending_request.take().map(|request| {
                    let effect = request.approval_effect();
                    story.add_event(
                        self.current_tick,
                        "Request approved by property manager",
                        effect.clone(),
                    );
                    effect
                })
            });

            if let Some(effect) = effect {
                self.apply_story_impact(tenant_id, effect);
            }
        }
    }

    fn generate_tenant_requests(&mut self) {
        for tenant in &self.tenants {
            if let Some(story) = self.tenant_stories.get_mut(&tenant.id) {
                if rng::gen_range(0, 100) < 10 {
                    story.make_request(&tenant.archetype, &self.tenant_events_config);
                }
            }
        }
    }

    fn expire_narrative_events(&mut self) {
        let expired_effects = self.narrative_events.expire_due_events(self.current_tick);
        for effect in expired_effects {
            self.apply_narrative_effect(&effect);
        }
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

    fn apply_active_tax_breaks(&mut self) {
        let refund = self.process_active_tax_breaks();
        if refund > 0 {
            self.spawn_center_text(
                &format!("Tax Break +${}", refund),
                0.0,
                60.0,
                colors::POSITIVE,
            );
        }
    }

    fn process_active_tax_breaks(&mut self) -> i32 {
        if self.active_tax_breaks.is_empty() {
            return 0;
        }

        let percentage = self
            .active_tax_breaks
            .iter()
            .map(|tax_break| tax_break.percentage)
            .sum::<f32>()
            .clamp(0.0, 0.75);
        let tax_paid = self.current_tick_property_tax();
        let refund = (tax_paid as f32 * percentage).round() as i32;

        if refund > 0 {
            self.funds.add_income(Transaction::income(
                TransactionType::Grant,
                refund,
                "Mission Tax Break Refund",
                self.current_tick,
            ));
        }

        for tax_break in &mut self.active_tax_breaks {
            tax_break.remaining_months = tax_break.remaining_months.saturating_sub(1);
        }
        self.active_tax_breaks
            .retain(|tax_break| tax_break.remaining_months > 0 && tax_break.percentage > 0.0);

        refund
    }

    fn current_tick_property_tax(&self) -> i32 {
        self.funds
            .transactions_for_tick(self.current_tick)
            .iter()
            .filter(|transaction| {
                transaction.transaction_type == TransactionType::PropertyTax
                    && transaction.amount < 0
            })
            .map(|transaction| transaction.amount.abs())
            .sum()
    }

    fn check_annual_awards(&mut self) {
        let avg_happiness = if self.tenants.is_empty() {
            0.0
        } else {
            self.tenants
                .iter()
                .map(|tenant| tenant.happiness as f32)
                .sum::<f32>()
                / self.tenants.len() as f32
        };
        let total = self.building.apartments.len();
        let occupied = self.building.occupancy_count();
        let occupancy_rate = if total > 0 {
            occupied as f32 / total as f32
        } else {
            0.0
        };

        self.missions.check_for_awards(
            self.current_tick,
            &self.building.name,
            avg_happiness,
            occupancy_rate,
            self.tenants.len() as u32,
        );

        let forming = self.tenant_network.should_form_council(
            &self.tenants,
            &self.config.gentrification,
            self.config.happiness.unhappy_threshold,
        );

        if forming && !self.council_formed {
            self.council_formed = true;
            self.apply_council_collective_action();
        } else if !forming {
            // Conditions improved; the council disbands and could re-form later.
            self.council_formed = false;
        }
    }

    /// A newly-formed tenant council's one-time collective action: it bargains
    /// the building's rent multiplier down and the solidarity of organizing
    /// lifts tenant morale. This is the mechanical consequence of pushing rent
    /// too hard on an unhappy building — previously the council was cosmetic.
    fn apply_council_collective_action(&mut self) {
        let rollback = self.config.gentrification.council_rent_rollback;
        self.building.rent_multiplier =
            (self.building.rent_multiplier * (1.0 - rollback)).clamp(0.5, 2.0);

        let bump = self.config.gentrification.council_solidarity_happiness;
        for tenant in &mut self.tenants {
            tenant.happiness = (tenant.happiness + bump).clamp(0, 100);
        }

        self.spawn_center_text(
            "Tenants formed a council — rent rolled back!",
            0.0,
            30.0,
            colors::ACCENT,
        );
        self.event_log.log(
            GameEvent::Notification {
                message: "A tenant council organized and bargained rent down.".to_string(),
                level: crate::simulation::NotificationLevel::Warning,
            },
            self.current_tick,
        );
        self.missions.record_legacy_event(
            self.current_tick,
            "Tenant Council Formed",
            "Tenants organized a council and won a rent rollback.",
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::narrative::ActiveTaxBreak;

    #[test]
    fn tax_break_refunds_current_tick_property_tax_and_expires() {
        let mut state = GameplayState::new();
        state.current_tick = 4;
        state.active_tax_breaks = vec![ActiveTaxBreak::new(1, 0.25)];
        state.funds.deduct_expense(Transaction::expense(
            TransactionType::PropertyTax,
            400,
            "Monthly Property Tax",
            state.current_tick,
        ));

        let refund = state.process_active_tax_breaks();

        assert_eq!(refund, 100);
        assert!(state.active_tax_breaks.is_empty());
        assert!(state.funds.transactions.iter().any(|transaction| {
            transaction.transaction_type == TransactionType::Grant
                && transaction.amount == 100
                && transaction.description == "Mission Tax Break Refund"
                && transaction.tick == 4
        }));
    }

    #[test]
    fn monthly_mail_uses_current_tick_rent_income() {
        let mut state = GameplayState::new();
        state.current_tick = 2;
        state.mailbox.items.clear();
        state.last_tick_result = Some(TickResult {
            events: Vec::new(),
            rent_collected: 10,
            tenants_moved_out: Vec::new(),
            new_applications: 0,
            outcome: None,
        });

        let result = TickResult {
            events: Vec::new(),
            rent_collected: 1234,
            tenants_moved_out: Vec::new(),
            new_applications: 0,
            outcome: None,
        };
        state.generate_monthly_narrative(&result);

        let statement = state
            .mailbox
            .items
            .iter()
            .find(|item| item.subject == "Monthly Statement - Month 2");
        assert!(statement.is_some(), "expected month 2 financial statement");
        if let Some(statement) = statement {
            assert!(
                statement.body.contains("Total Income: $1234"),
                "statement should use current tick income"
            );
        }
    }
}
