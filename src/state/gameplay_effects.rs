//! Narrative effect application for gameplay state.

use crate::narrative::events::NarrativeEffect;
use crate::narrative::{MissionGoal, MissionStatus};
use crate::ui::colors;
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
                building_id,
                change,
            } => {
                for tenant in &mut self.tenants {
                    if tenant.building_id == *building_id {
                        tenant.happiness = (tenant.happiness + change).clamp(0, 100);
                    }
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
            NarrativeEffect::TriggerInspection { building_id } => {
                self.execute_inspection_for(
                    *building_id,
                    crate::consequences::InspectionTrigger::TenantComplaint,
                );
                self.bill_outstanding_fines();
            }
            NarrativeEffect::PropertyValue {
                building_id,
                change_percent,
            } => {
                // Property value is expressed through the building's rent ceiling:
                // a value change lets the landlord command proportionally more (or
                // less) rent.
                let factor = 1.0 + change_percent / 100.0;
                if *building_id == self.active_building_id() {
                    self.building.rent_multiplier =
                        (self.building.rent_multiplier * factor).clamp(0.5, 2.0);
                    self.save_building_to_city();
                } else if let Some(building) = self.city.buildings.get_mut(*building_id as usize) {
                    building.rent_multiplier = (building.rent_multiplier * factor).clamp(0.5, 2.0);
                }
            }
        }
    }

    fn sell_building_from_event(&mut self, building_id: u32) {
        let index = building_id as usize;
        if index >= self.city.buildings.len() {
            return;
        }

        self.save_building_to_city();
        let sold_building = self.city.buildings[index].clone();
        let neighborhood_name = self
            .city
            .neighborhood_for_building(index)
            .map(|neighborhood| neighborhood.name.clone())
            .unwrap_or_else(|| "Unknown neighborhood".to_string());
        let removed_tenant_ids: std::collections::HashSet<u32> = self
            .tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
            .map(|tenant| tenant.id)
            .collect();

        for tenant in self
            .tenants
            .iter()
            .filter(|tenant| tenant.building_id == building_id)
        {
            let rent = tenant
                .apartment_id
                .and_then(|apartment_id| sold_building.get_apartment(apartment_id))
                .map(|apartment| apartment.rent_price)
                .unwrap_or(0);
            self.gentrification
                .displacements
                .push(crate::consequences::DisplacementEvent {
                    tenant_name: tenant.name.clone(),
                    archetype: tenant.archetype.clone(),
                    original_rent: rent,
                    final_rent: rent,
                    months_resided: tenant.months_residing,
                    reason: crate::consequences::DisplacementReason::BuildingSold,
                    month: self.current_tick,
                    building_name: sold_building.name.clone(),
                    neighborhood_name: neighborhood_name.clone(),
                });
        }

        self.tenants
            .retain(|tenant| tenant.building_id != building_id);
        self.applications
            .retain(|application| application.building_id != building_id);
        self.tenant_stories
            .retain(|tenant_id, _| !removed_tenant_ids.contains(tenant_id));
        self.tenant_network.relationships.retain(|relationship| {
            !removed_tenant_ids.contains(&relationship.tenant_a_id)
                && !removed_tenant_ids.contains(&relationship.tenant_b_id)
        });
        self.tenant_network
            .long_term_tenants
            .retain(|record| !removed_tenant_ids.contains(&record.tenant_id));
        self.tenant_network
            .dilemma_history
            .retain(|tenant_id, _| !removed_tenant_ids.contains(tenant_id));
        self.tenant_network
            .tensions
            .retain(|tension| tension.building_id != building_id);

        self.city.buildings.remove(index);
        reindex_building_references(self, building_id);

        if self.city.buildings.is_empty() {
            self.game_outcome = Some(crate::simulation::GameOutcome::Victory {
                score: self.funds.balance,
                months: self.current_tick,
                total_income: self.funds.total_income,
            });
            self.view_mode = ViewMode::CareerSummary;
        } else {
            let previous_active = self.city.active_building_index;
            self.city.active_building_index = if previous_active > index {
                previous_active - 1
            } else if previous_active == index {
                index.min(self.city.buildings.len() - 1)
            } else {
                previous_active
            };
            self.sync_building();
            self.selection = crate::ui::Selection::None;
            self.council_formed = false;
            self.floating_texts.spawn(
                "Building Sold!",
                vec2(screen_width() / 2.0, screen_height() / 2.0),
                colors::POSITIVE(),
            );
        }
    }
}

fn reindex_building_references(state: &mut GameplayState, removed_id: u32) {
    for neighborhood in &mut state.city.neighborhoods {
        neighborhood.building_ids.retain(|id| *id != removed_id);
        for id in &mut neighborhood.building_ids {
            if *id > removed_id {
                *id -= 1;
            }
        }
    }
    for tenant in &mut state.tenants {
        if tenant.building_id > removed_id {
            tenant.building_id -= 1;
        }
    }
    for application in &mut state.applications {
        if application.building_id > removed_id {
            application.building_id -= 1;
        }
    }
    for tension in &mut state.tenant_network.tensions {
        if tension.building_id > removed_id {
            tension.building_id -= 1;
        }
    }

    state.ever_occupied_buildings = state
        .ever_occupied_buildings
        .iter()
        .filter_map(|id| reindexed_id(*id, removed_id))
        .collect();
    state.compliance.building_regulations = reindex_map(
        std::mem::take(&mut state.compliance.building_regulations),
        removed_id,
    );
    state
        .compliance
        .inspection_history
        .retain_mut(|inspection| reindex_id_mut(&mut inspection.building_id, removed_id));
    state
        .compliance
        .pending_fixes
        .retain_mut(|(id, _, _)| reindex_id_mut(id, removed_id));
    state.gentrification.rent_history = reindex_map(
        std::mem::take(&mut state.gentrification.rent_history),
        removed_id,
    );
    state.gentrification.demographic_shifts = reindex_map(
        std::mem::take(&mut state.gentrification.demographic_shifts),
        removed_id,
    );

    for mission in &mut state.missions.missions {
        if let MissionGoal::FullRepair { building_id } = &mut mission.goal {
            if *building_id == removed_id {
                if matches!(
                    mission.status,
                    MissionStatus::Available | MissionStatus::Active
                ) {
                    mission.fail();
                }
            } else if *building_id > removed_id {
                *building_id -= 1;
            }
        }
    }
    for event in &mut state.narrative_events.events {
        adjust_effect_building_id(&mut event.default_effect, removed_id);
        for choice in &mut event.choices {
            adjust_effect_building_id(&mut choice.effect, removed_id);
        }
    }
}

fn reindexed_id(id: u32, removed_id: u32) -> Option<u32> {
    match id.cmp(&removed_id) {
        std::cmp::Ordering::Less => Some(id),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(id - 1),
    }
}

fn reindex_id_mut(id: &mut u32, removed_id: u32) -> bool {
    if let Some(new_id) = reindexed_id(*id, removed_id) {
        *id = new_id;
        true
    } else {
        false
    }
}

fn reindex_map<T>(
    values: std::collections::HashMap<u32, T>,
    removed_id: u32,
) -> std::collections::HashMap<u32, T> {
    values
        .into_iter()
        .filter_map(|(id, value)| reindexed_id(id, removed_id).map(|id| (id, value)))
        .collect()
}

fn adjust_effect_building_id(effect: &mut NarrativeEffect, removed_id: u32) {
    let id = match effect {
        NarrativeEffect::BuildingHappiness { building_id, .. }
        | NarrativeEffect::TriggerInspection { building_id }
        | NarrativeEffect::PropertyValue { building_id, .. }
        | NarrativeEffect::SellBuilding { building_id } => Some(building_id),
        NarrativeEffect::Multiple { effects } => {
            for effect in effects {
                adjust_effect_building_id(effect, removed_id);
            }
            None
        }
        _ => None,
    };
    if let Some(id) = id {
        if *id == removed_id {
            *effect = NarrativeEffect::None;
        } else if *id > removed_id {
            *id -= 1;
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
    fn property_value_effect_updates_the_named_non_active_building() {
        let mut state = GameplayState::new();
        let second = crate::building::Building::new("Second", 1, 1);
        state.city.add_building(second, 0).unwrap();

        state.apply_narrative_effect(&NarrativeEffect::PropertyValue {
            building_id: 1,
            change_percent: 20.0,
        });

        assert!((state.city.buildings[1].rent_multiplier - 1.2).abs() < 0.001);
        assert!((state.building.rent_multiplier - 1.0).abs() < 0.001);
    }

    #[test]
    fn inspection_effect_records_the_named_building() {
        let mut state = GameplayState::new();
        state
            .city
            .add_building(crate::building::Building::new("Second", 1, 1), 0)
            .unwrap();

        state.apply_narrative_effect(&NarrativeEffect::TriggerInspection { building_id: 1 });

        assert_eq!(
            state
                .compliance
                .inspection_history
                .last()
                .map(|inspection| inspection.building_id),
            Some(1)
        );
    }

    #[test]
    fn portfolio_reindex_keeps_cross_system_building_ids_aligned() {
        let mut state = GameplayState::new();
        state
            .city
            .add_building(crate::building::Building::new("Second", 1, 1), 0)
            .unwrap();
        let mut tenant = crate::tenant::Tenant::new(
            99,
            "Second Resident",
            crate::tenant::TenantArchetype::Student,
        );
        tenant.building_id = 1;
        state.tenants.push(tenant);
        state.compliance.init_building_regulations(1, false);
        state.ever_occupied_buildings.insert(1);
        state.gentrification.rent_history.insert(1, Vec::new());
        state
            .tenant_network
            .apply_tension_change(1, 0, 1, 20, "test");

        state.city.buildings.remove(0);
        reindex_building_references(&mut state, 0);

        assert_eq!(
            state.tenants.last().map(|tenant| tenant.building_id),
            Some(0)
        );
        assert!(state.compliance.building_regulations.contains_key(&0));
        assert_eq!(
            state.ever_occupied_buildings,
            std::collections::HashSet::from([0])
        );
        assert!(state.gentrification.rent_history.contains_key(&0));
        assert_eq!(state.tenant_network.tensions[0].building_id, 0);
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
    fn non_active_buildings_contribute_passive_income() {
        use crate::building::Building;

        let mut state = GameplayState::new();
        // Add a second building (index 1, non-active) with known rents.
        let mut passive = Building::new("Passive Block", 2, 2); // 4 units
        for apt in &mut passive.apartments {
            apt.rent_price = 1000;
        }
        let _ = state.city.add_building(passive, 0);

        let before = state.funds.balance;
        state.collect_portfolio_passive_income();
        // 4 units × $1000 × 0.8 − 4 × $190 = 3200 − 760 = 2440 (positive).
        assert!(
            state.funds.balance > before,
            "an owned non-active building should earn passive income"
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
