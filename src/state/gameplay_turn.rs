//! Monthly turn advancement for gameplay state.

use crate::economy::{Transaction, TransactionType};
use crate::simulation::{advance_tick, GameEvent, TickResult};
use crate::ui::{colors, FloatingText};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// End the current turn and advance time.
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
            &self.config,
        );

        self.game_outcome = result.outcome.clone();
        self.spawn_tick_feedback(&result.events);
        self.apply_active_tax_breaks();
        self.update_city_systems();
        self.generate_monthly_narrative(&result);
        self.expire_narrative_events();
        self.sync_building();
        self.autosave_current_game();
        self.missions.generate_late_game_missions(self.current_tick);

        if self.current_tick % 12 == 0 && self.current_tick > 0 {
            self.check_annual_awards();
        }

        self.apply_monthly_social_happiness();
        self.log_monthly_status();
        self.update_context_hints();
        self.check_game_completion();
        self.update_missions();
        self.last_tick_result = Some(result);
    }

    fn spawn_tick_feedback(&mut self, events: &[GameEvent]) {
        for event in events {
            match event {
                GameEvent::RentPaid { amount, .. } => self.spawn_center_text(
                    &format!("+${}", amount),
                    macroquad::rand::gen_range(-50.0, 50.0),
                    macroquad::rand::gen_range(-50.0, 50.0),
                    colors::POSITIVE,
                ),
                GameEvent::RentMissed { .. } => {
                    self.spawn_center_text("Missed Rent!", 0.0, 0.0, colors::NEGATIVE);
                }
                GameEvent::TenantUnhappy { .. } => self.spawn_center_text(
                    "Unhappy!",
                    macroquad::rand::gen_range(-50.0, 50.0),
                    macroquad::rand::gen_range(-50.0, 50.0),
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

    fn update_city_systems(&mut self) {
        self.save_building_to_city();
        self.city.tick();

        let (rel_changes, rel_events) = self.tenant_network.tick(
            &self.tenants,
            &self.building,
            &self.config.relationships,
            &self.relationship_events_config,
        );
        self.notifications.add_relationship_changes(rel_changes);
        for mut event in rel_events {
            event.month = self.current_tick;
            self.narrative_events.events.push(event);
        }

        self.compliance.tick(self.current_tick);
        self.gentrification
            .update_affordable_units(&self.building.apartments, &self.config.gentrification);
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
        self.dialogue_system
            .generate_dialogues(self.current_tick, &tenants, &building, &funds);
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

    fn generate_tenant_requests(&mut self) {
        for tenant in &self.tenants {
            if let Some(story) = self.tenant_stories.get_mut(&tenant.id) {
                if macroquad::rand::gen_range(0, 100) < 10 {
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
        let any_unhappy = self.tenants.iter().any(|tenant| tenant.is_unhappy());

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
        if self.active_tax_breaks.is_empty() {
            return;
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
            self.spawn_center_text(
                &format!("Tax Break +${}", refund),
                0.0,
                60.0,
                colors::POSITIVE,
            );
        }

        for tax_break in &mut self.active_tax_breaks {
            tax_break.remaining_months = tax_break.remaining_months.saturating_sub(1);
        }
        self.active_tax_breaks
            .retain(|tax_break| tax_break.remaining_months > 0 && tax_break.percentage > 0.0);
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

        if self
            .tenant_network
            .should_form_council(&self.tenants, &self.config.gentrification)
        {
            self.spawn_center_text("Tenants forming a council!", 0.0, 30.0, colors::ACCENT);
            self.missions.record_legacy_event(
                self.current_tick,
                "Tenant Council Formed",
                "Tenants organized to form a tenant council.",
            );
        }
    }
}
