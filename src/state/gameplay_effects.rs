//! Narrative effect application for gameplay state.

use crate::narrative::events::NarrativeEffect;
use crate::ui::{colors, FloatingText};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// Apply a narrative effect to the current gameplay state.
    pub(super) fn apply_narrative_effect(&mut self, effect: &NarrativeEffect) {
        match effect {
            NarrativeEffect::None => {}
            NarrativeEffect::Money { amount } => {
                if *amount < 0 {
                    self.funds.deduct_expense(crate::economy::Transaction::expense(
                        crate::economy::TransactionType::CriticalFailure,
                        amount.abs(),
                        "Event Consequence",
                        self.current_tick,
                    ));
                } else {
                    self.funds.add_income(crate::economy::Transaction::income(
                        crate::economy::TransactionType::Grant,
                        *amount,
                        "Event Reward",
                        self.current_tick,
                    ));
                }
            }
            NarrativeEffect::TenantHappiness { tenant_id, change } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == *tenant_id) {
                    tenant.happiness = (tenant.happiness + change).clamp(0, 100);
                }
            }
            NarrativeEffect::OpinionChange { tenant_id, amount } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == *tenant_id) {
                    tenant.landlord_opinion = (tenant.landlord_opinion + amount).clamp(-100, 100);
                }
            }
            NarrativeEffect::RelationshipStrength {
                tenant_a_id,
                tenant_b_id,
                change,
            } => {
                self.tenant_network.apply_relationship_change(
                    *tenant_a_id,
                    *tenant_b_id,
                    *change,
                );
            }
            NarrativeEffect::MoveOut { tenant_id } => {
                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == *tenant_id) {
                    tenant.happiness = 0;
                }
            }
            NarrativeEffect::SellBuilding { building_id } => {
                self.sell_building_from_event(*building_id);
            }
            NarrativeEffect::Multiple { effects } => {
                for effect in effects {
                    self.apply_narrative_effect(effect);
                }
            }
            _ => {}
        }
    }

    fn sell_building_from_event(&mut self, building_id: u32) {
        let index = building_id as usize;

        if index < self.city.buildings.len() {
            self.city.buildings.remove(index);

            for neighborhood in &mut self.city.neighborhoods {
                neighborhood.building_ids.retain(|&id| id != building_id);
                for id in &mut neighborhood.building_ids {
                    if *id > building_id {
                        *id -= 1;
                    }
                }
            }
        }

        if self.city.buildings.is_empty() {
            self.game_outcome = Some(crate::simulation::GameOutcome::Victory {
                score: self.funds.balance,
                months: self.current_tick,
                total_income: self.funds.total_income,
            });
            self.view_mode = ViewMode::CareerSummary;
        } else {
            self.city.active_building_index = 0;
            self.sync_building();
            self.floating_texts.push(FloatingText::new(
                "Building Sold!",
                screen_width() / 2.0,
                screen_height() / 2.0,
                colors::POSITIVE,
            ));
        }
    }
}
