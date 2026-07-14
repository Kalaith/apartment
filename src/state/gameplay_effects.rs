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
                    self.funds
                        .apply_required_expense(crate::economy::Transaction::expense(
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
                self.tenant_network
                    .apply_relationship_change(*tenant_a_id, *tenant_b_id, *change);
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
            NarrativeEffect::NeighborhoodReputation {
                neighborhood_id,
                change,
            } => {
                if let Some(neighborhood) = self
                    .city
                    .neighborhoods
                    .iter_mut()
                    .find(|n| n.id == *neighborhood_id)
                {
                    neighborhood.reputation = (neighborhood.reputation + change).clamp(0, 100);
                }
            }
            NarrativeEffect::BuildingHappiness {
                building_id: _,
                change,
            } => {
                // Only the active building is simulated at a time, so a
                // building-wide morale swing applies to its current tenants.
                for tenant in &mut self.tenants {
                    tenant.happiness = (tenant.happiness + change).clamp(0, 100);
                }
            }
            NarrativeEffect::EconomyChange {
                economy_health_change,
            } => {
                self.city.economy_health =
                    (self.city.economy_health + economy_health_change).clamp(0.5, 1.5);
            }
            NarrativeEffect::RentDemand {
                neighborhood_id,
                change,
            } => {
                if let Some(neighborhood) = self
                    .city
                    .neighborhoods
                    .iter_mut()
                    .find(|n| n.id == *neighborhood_id)
                {
                    neighborhood.stats.rent_demand =
                        (neighborhood.stats.rent_demand + change).clamp(0.5, 2.0);
                }
            }
            NarrativeEffect::TriggerInspection { building_id: _ } => {
                // A complaint-driven inspection of the active building, billed
                // immediately (outside the monthly billing pass).
                self.execute_inspection(crate::consequences::InspectionTrigger::TenantComplaint);
                self.bill_outstanding_fines();
            }
            NarrativeEffect::PropertyValue {
                building_id: _,
                change_percent,
            } => {
                // Property value is expressed through the building's rent ceiling:
                // a value change lets the landlord command proportionally more (or
                // less) rent.
                let factor = 1.0 + change_percent / 100.0;
                self.building.rent_multiplier =
                    (self.building.rent_multiplier * factor).clamp(0.5, 2.0);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GameplayState;

    #[test]
    fn neighborhood_reputation_effect_moves_reputation() {
        let mut state = GameplayState::new();
        let nid = state.city.neighborhoods[0].id;
        let before = state.city.neighborhoods[0].reputation;
        state.apply_narrative_effect(&NarrativeEffect::NeighborhoodReputation {
            neighborhood_id: nid,
            change: 10,
        });
        assert_eq!(
            state.city.neighborhoods[0].reputation,
            (before + 10).clamp(0, 100)
        );
    }

    #[test]
    fn economy_change_effect_clamps_to_boom_ceiling() {
        let mut state = GameplayState::new();
        state.city.economy_health = 1.4;
        state.apply_narrative_effect(&NarrativeEffect::EconomyChange {
            economy_health_change: 0.5,
        });
        assert!((state.city.economy_health - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn rent_demand_effect_moves_neighborhood_demand() {
        let mut state = GameplayState::new();
        let nid = state.city.neighborhoods[0].id;
        let before = state.city.neighborhoods[0].stats.rent_demand;
        state.apply_narrative_effect(&NarrativeEffect::RentDemand {
            neighborhood_id: nid,
            change: 0.1,
        });
        assert!(state.city.neighborhoods[0].stats.rent_demand > before);
    }

    #[test]
    fn property_value_effect_scales_rent_ceiling() {
        let mut state = GameplayState::new();
        state.building.rent_multiplier = 1.0;
        state.apply_narrative_effect(&NarrativeEffect::PropertyValue {
            building_id: 0,
            change_percent: 10.0,
        });
        assert!((state.building.rent_multiplier - 1.1).abs() < 0.001);
    }

    #[test]
    fn building_happiness_effect_shifts_all_tenants() {
        let mut state = GameplayState::new();
        if state.tenants.is_empty() {
            return; // No tenants to shift; the empty case simply must not panic.
        }
        for tenant in &mut state.tenants {
            tenant.happiness = 50;
        }
        state.apply_narrative_effect(&NarrativeEffect::BuildingHappiness {
            building_id: 0,
            change: -5,
        });
        assert!(state.tenants.iter().all(|t| t.happiness == 45));
    }

    #[test]
    fn reputation_change_moves_active_neighborhood() {
        // Use the non-UI mutation helper: apply_reputation_change also pushes
        // floating text, which needs a macroquad GL context unit tests lack.
        let mut state = GameplayState::new();
        let before = state.active_neighborhood_reputation();
        state.adjust_active_neighborhood_reputation(10);
        assert_eq!(
            state.active_neighborhood_reputation(),
            (before + 10).clamp(0, 100)
        );
    }

    #[test]
    fn same_seed_reproduces_initial_state() {
        use crate::data::config::load_config;
        use crate::data::templates::load_templates;

        let Some(template) = load_templates().and_then(|t| t.templates.into_iter().next()) else {
            return;
        };
        let a = GameplayState::new_with_template_seed(load_config(), template.clone(), 777);
        let b = GameplayState::new_with_template_seed(load_config(), template, 777);

        assert_eq!(a.seed, 777);
        assert_eq!(a.next_tenant_id, b.next_tenant_id);
        let archetypes = |s: &GameplayState| {
            s.applications
                .iter()
                .map(|app| format!("{:?}", app.tenant.archetype))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            archetypes(&a),
            archetypes(&b),
            "same seed must reproduce the same initial applicants"
        );
    }

    #[test]
    fn historic_building_carries_extra_preservation_regulation() {
        use crate::data::config::load_config;
        use crate::data::templates::load_templates;

        let templates = load_templates().map(|t| t.templates).unwrap_or_default();
        let historic = templates.iter().find(|t| t.neighborhood_id == 3).cloned();
        let plain = templates.iter().find(|t| t.neighborhood_id != 3).cloned();
        let (Some(historic), Some(plain)) = (historic, plain) else {
            return;
        };

        let hstate = GameplayState::new_with_template(load_config(), historic);
        let pstate = GameplayState::new_with_template(load_config(), plain);
        let reg_count = |s: &GameplayState| {
            s.compliance
                .building_regulations
                .values()
                .map(|v| v.len())
                .max()
                .unwrap_or(0)
        };
        assert!(
            reg_count(&hstate) > reg_count(&pstate),
            "historic building should carry the extra preservation regulation ({} vs {})",
            reg_count(&hstate),
            reg_count(&pstate)
        );
    }

    #[test]
    fn condo_sale_multiplier_tracks_the_market() {
        let mut state = GameplayState::new();
        state.city.economy_health = 1.5; // boom
        let boom = state.condo_sale_market_multiplier();
        state.city.economy_health = 0.5; // recession
        let bust = state.condo_sale_market_multiplier();
        assert!(
            boom > 1.0,
            "a booming economy should lift condo sale prices"
        );
        assert!(bust < 1.0, "a recession should depress condo sale prices");
        assert!(boom > bust);
    }

    #[test]
    fn application_multiplier_scales_with_reputation() {
        let mut state = GameplayState::new();
        state.adjust_active_neighborhood_reputation(-50); // drive toward 0
        let low = state.application_reputation_multiplier();
        state.adjust_active_neighborhood_reputation(100); // drive toward 100
        let high = state.application_reputation_multiplier();
        assert!(low < 1.0, "poor reputation should suppress applicants");
        assert!(high > 1.0, "strong reputation should draw applicants");
    }
}
