//! Game action processing - split from gameplay.rs for maintainability

use crate::city::NeighborhoodType;
use crate::economy::process_upgrade;
use crate::narrative::{StoryImpact, TenantStory};
use crate::simulation::GameEvent;
use crate::ui::{colors, FloatingText, Selection, UiAction};
use macroquad::prelude::*;

use super::gameplay::{GameplayState, ViewMode};
use super::mission_system;
use super::tutorial_system;

impl GameplayState {
    /// Process a UI action
    pub(super) fn process_action(&mut self, action: UiAction) {
        match action {
            UiAction::SelectApartment(id) => {
                self.selection = Selection::Apartment(id);
                self.panel_scroll_offset = 0.0;
            }
            UiAction::SelectTenant(id) => {
                self.selection = Selection::Tenant(id);
            }
            UiAction::SelectApplications(filter) => {
                self.selection = Selection::Applications(filter);
                self.panel_scroll_offset = 0.0;
            }
            UiAction::SelectHallway => {
                self.selection = Selection::Hallway;
            }
            UiAction::ClearSelection => {
                self.selection = Selection::None;
            }

            UiAction::ListApartment {
                apartment_id,
                preference,
            } => {
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    apt.is_listed_for_lease = true;
                    apt.preferred_archetype = preference;

                    self.floating_texts.push(FloatingText::new(
                        "Listed for Lease",
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        colors::POSITIVE,
                    ));
                }
            }

            UiAction::UnlistApartment { apartment_id } => {
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    apt.is_listed_for_lease = false;
                    apt.preferred_archetype = None;

                    self.floating_texts.push(FloatingText::new(
                        "Property Unlisted",
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        colors::TEXT,
                    ));
                }
            }

            UiAction::AdjustRent {
                apartment_id,
                amount,
            } => {
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    apt.rent_price = (apt.rent_price + amount).max(100); // Minimum rent $100
                }
            }

            UiAction::UpgradeAction(upgrade) => {
                let description =
                    upgrade.label(&self.building, &self.config.ui, &self.config.upgrades);
                if let Ok(cost) = process_upgrade(
                    &upgrade,
                    &mut self.building,
                    &mut self.funds,
                    &self.config,
                    self.current_tick,
                ) {
                    self.event_log.log(
                        GameEvent::UpgradeCompleted { description, cost },
                        self.current_tick,
                    );

                    let mouse = mouse_position();
                    self.floating_texts.push(FloatingText::new(
                        &format!("-${}", cost),
                        mouse.0,
                        mouse.1 - 20.0,
                        colors::NEGATIVE,
                    ));
                }
            }
            UiAction::SetRent {
                apartment_id,
                new_rent,
            } => {
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    let old_rent = apt.rent_price;
                    apt.rent_price = new_rent;

                    if old_rent != new_rent {
                        self.gentrification.record_rent_change(
                            0,
                            self.current_tick,
                            old_rent,
                            new_rent,
                            &self.config.gentrification,
                        );
                    }
                }
            }
            UiAction::AcceptApplication { application_index } => {
                if application_index < self.applications.len() {
                    let app = self.applications.remove(application_index);
                    let mut tenant = app.tenant;

                    // Evaluate lease using the configured offer
                    if let Some(apt) = self.building.get_apartment(app.apartment_id) {
                        let offer = crate::tenant::matching::LeaseOffer::from_config(
                            apt.rent_price,
                            &self.config.matching.lease_defaults,
                        );
                        let accept_probability = crate::tenant::matching::evaluate_lease_offer(
                            &tenant,
                            &offer,
                            &self.config.matching.lease_acceptance,
                        );

                        // Log the negotiation outcome
                        let leverage = tenant.negotiation_leverage();
                        println!(
                            "Tenant {} has negotiation leverage: {}, accept probability: {:.2}",
                            tenant.name, leverage, accept_probability
                        );
                    }

                    tenant.move_into(app.apartment_id);

                    if let Some(apt) = self.building.get_apartment_mut(app.apartment_id) {
                        apt.move_in(tenant.id);
                    }

                    self.event_log.log(
                        GameEvent::TenantMovedIn {
                            tenant_name: tenant.name.clone(),
                            apartment_unit: self
                                .building
                                .get_apartment(app.apartment_id)
                                .map(|a| a.unit_number.clone())
                                .unwrap_or_default(),
                        },
                        self.current_tick,
                    );

                    let mouse = mouse_position();
                    self.floating_texts.push(FloatingText::new(
                        "Welcome!",
                        mouse.0,
                        mouse.1 - 20.0,
                        colors::POSITIVE,
                    ));

                    let story = TenantStory::generate(tenant.id, &tenant.archetype);
                    self.tenant_stories.insert(tenant.id, story);

                    self.tenants.push(tenant);
                }
            }
            UiAction::RejectApplication { application_index } => {
                if application_index < self.applications.len() {
                    self.applications.remove(application_index);
                }
            }
            UiAction::CreditCheck { application_index } => {
                if application_index < self.applications.len() {
                    let app = &mut self.applications[application_index];
                    if let Some(result) = crate::tenant::vetting::perform_credit_check(
                        app,
                        &mut self.funds,
                        &self.config.vetting,
                    ) {
                        self.floating_texts.push(FloatingText::new(
                            &format!(
                                "Credit: {} - {}",
                                result.reliability_score, result.recommendation
                            ),
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            if result.reliability_score >= 75 {
                                colors::POSITIVE
                            } else if result.reliability_score >= 50 {
                                colors::WARNING
                            } else {
                                colors::NEGATIVE
                            },
                        ));
                    } else {
                        self.floating_texts.push(FloatingText::new(
                            "Cannot perform credit check",
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            colors::NEGATIVE,
                        ));
                    }
                }
            }
            UiAction::BackgroundCheck { application_index } => {
                if application_index < self.applications.len() {
                    let app = &mut self.applications[application_index];
                    if let Some(result) = crate::tenant::vetting::perform_background_check(
                        app,
                        &mut self.funds,
                        &self.config.vetting,
                    ) {
                        self.floating_texts.push(FloatingText::new(
                            &format!(
                                "Background: {} - {}",
                                result.behavior_score, result.history_notes
                            ),
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            if result.behavior_score >= 75 {
                                colors::POSITIVE
                            } else if result.behavior_score >= 50 {
                                colors::WARNING
                            } else {
                                colors::NEGATIVE
                            },
                        ));
                    } else {
                        self.floating_texts.push(FloatingText::new(
                            "Cannot perform background check",
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            colors::NEGATIVE,
                        ));
                    }
                }
            }
            UiAction::EndTurn => {
                self.end_turn();
            }
            UiAction::ReturnToMenu => {
                self.pending_quit_to_menu = true;
            }

            // Phase 3: City navigation
            UiAction::OpenCityMap => {
                self.view_mode = ViewMode::CityMap;
                self.selection = Selection::None;
            }
            UiAction::CloseCityView => {
                self.view_mode = ViewMode::Building;
            }
            UiAction::OpenMarket => {
                self.view_mode = ViewMode::Market;
            }
            UiAction::CloseMarket => {
                self.view_mode = ViewMode::CityMap;
            }

            UiAction::OpenMail => {
                self.view_mode = ViewMode::Mail;
            }
            UiAction::CloseMail => {
                self.view_mode = ViewMode::Building;
            }

            // Phase 3: Multi-building
            UiAction::SwitchBuilding { index } => {
                self.save_building_to_city();
                self.city.switch_building(index);
                self.sync_building();
                self.selection = Selection::None;

                self.floating_texts.push(FloatingText::new(
                    "Building Changed",
                    screen_width() / 2.0,
                    screen_height() / 2.0,
                    colors::ACCENT,
                ));
            }
            UiAction::PurchaseBuilding { listing_id } => {
                if let Some(listing) = self
                    .city
                    .market
                    .listings
                    .iter()
                    .find(|l| l.id == listing_id)
                    .cloned()
                {
                    if self.funds.balance >= listing.asking_price {
                        let building = listing.to_building();
                        let neighborhood_id = listing.neighborhood_id;

                        if let Ok(building_id) = self.city.add_building(building, neighborhood_id) {
                            let transaction = crate::economy::Transaction::expense(
                                crate::economy::TransactionType::BuildingPurchase,
                                listing.asking_price,
                                "Building Purchase",
                                self.current_tick,
                            );
                            self.funds.deduct_expense(transaction);

                            let is_historic = self.city.neighborhoods.iter().any(|n| {
                                n.id == neighborhood_id
                                    && matches!(n.neighborhood_type, NeighborhoodType::Historic)
                            });
                            self.compliance
                                .init_building_regulations(building_id, is_historic);

                            self.city.market.listings.retain(|l| l.id != listing_id);

                            self.floating_texts.push(FloatingText::new(
                                "Building Purchased!",
                                screen_width() / 2.0,
                                screen_height() / 2.0,
                                colors::POSITIVE,
                            ));

                            self.event_log.log(
                                GameEvent::UpgradeCompleted {
                                    description: "Purchased new building".to_string(),
                                    cost: listing.asking_price,
                                },
                                self.current_tick,
                            );
                        }
                    }
                }
            }

            // Phase 3: Tenant requests
            UiAction::ApproveRequest { tenant_id } => {
                if let Some(story) = self.tenant_stories.get_mut(&tenant_id) {
                    if let Some(request) = story.pending_request.take() {
                        let effect = request.approval_effect();
                        story.add_event(
                            self.current_tick,
                            "Request approved by landlord",
                            effect.clone(),
                        );

                        let mut stack = vec![effect];
                        while let Some(e) = stack.pop() {
                            match e {
                                StoryImpact::Happiness(amount) => {
                                    if let Some(tenant) =
                                        self.tenants.iter_mut().find(|t| t.id == tenant_id)
                                    {
                                        tenant.happiness =
                                            (tenant.happiness + amount).clamp(0, 100);
                                    }
                                }
                                StoryImpact::SetApartmentFlag(flag) => {
                                    if let Some(apt) = self
                                        .building
                                        .apartments
                                        .iter_mut()
                                        .find(|a| a.tenant_id == Some(tenant_id))
                                    {
                                        apt.flags.insert(flag);
                                    }
                                }
                                StoryImpact::Multiple(sub_effects) => {
                                    stack.extend(sub_effects);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            UiAction::DenyRequest { tenant_id } => {
                if let Some(story) = self.tenant_stories.get_mut(&tenant_id) {
                    if let Some(request) = story.pending_request.take() {
                        let effect = request.denial_effect();
                        story.add_event(self.current_tick, "Request denied by landlord", effect);

                        if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                            tenant.happiness = (tenant.happiness - 10).max(0);
                        }
                    }
                }
            }

            // Phase 3: Ownership
            UiAction::SelectOwnership => {
                self.selection = Selection::Ownership;
            }
            UiAction::VoteOnProposal {
                proposal_index: _index,
                vote_yes: _vote,
            } => {
                self.floating_texts.push(FloatingText::new(
                    "Vote Cast",
                    screen_width() / 2.0,
                    screen_height() / 2.0,
                    colors::ACCENT,
                ));
            }
            UiAction::SellUnitAsCondo { apartment_id } => {
                let sale_price = self
                    .building
                    .get_apartment(apartment_id)
                    .map(|apt| apt.market_value())
                    .unwrap_or(10_000);

                if let Some(apt) = self.building.get_apartment(apartment_id) {
                    if let Some(tenant_id) = apt.tenant_id {
                        self.tenants.retain(|t| t.id != tenant_id);
                        self.tenant_stories.remove(&tenant_id);
                    }
                }

                if self
                    .building
                    .convert_unit_to_condo(apartment_id, "New Owner", sale_price)
                {
                    let transaction = crate::economy::Transaction::income(
                        crate::economy::TransactionType::AssetSale,
                        sale_price,
                        "Condo Sale",
                        self.current_tick,
                    );
                    self.funds.add_income(transaction);

                    self.floating_texts.push(FloatingText::new(
                        &format!("+${}", sale_price),
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        colors::POSITIVE,
                    ));

                    self.save_building_to_city();
                }
            }
            UiAction::BuybackCondo { apartment_id } => {
                if let Some(buyback_cost) = self.building.buyback_condo(apartment_id) {
                    if self.funds.balance >= buyback_cost {
                        let transaction = crate::economy::Transaction::expense(
                            crate::economy::TransactionType::BuildingPurchase,
                            buyback_cost,
                            "Condo Buyback",
                            self.current_tick,
                        );
                        self.funds.deduct_expense(transaction);

                        self.floating_texts.push(FloatingText::new(
                            &format!("-${}", buyback_cost),
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            colors::NEGATIVE,
                        ));

                        self.floating_texts.push(FloatingText::new(
                            "Unit Repurchased!",
                            screen_width() / 2.0,
                            screen_height() / 2.0 + 30.0,
                            colors::POSITIVE,
                        ));

                        self.save_building_to_city();
                    }
                }
            }
            UiAction::ResolveDialogue {
                dialogue_id,
                choice_index,
            } => {
                if let Some(effects) = self
                    .dialogue_system
                    .resolve_dialogue(dialogue_id, choice_index)
                {
                    // Apply dialogue effects
                    for effect in effects {
                        match effect {
                            crate::narrative::dialogue::DialogueEffect::HappinessChange {
                                tenant_id,
                                amount,
                            } => {
                                if let Some(tenant) =
                                    self.tenants.iter_mut().find(|t| t.id == tenant_id)
                                {
                                    tenant.happiness = (tenant.happiness + amount).clamp(0, 100);
                                }
                            }
                            crate::narrative::dialogue::DialogueEffect::MoneyChange(amount) => {
                                if amount > 0 {
                                    self.funds.add_income(crate::economy::Transaction::income(
                                        crate::economy::TransactionType::Grant,
                                        amount,
                                        "Dialogue Reward",
                                        self.current_tick,
                                    ));
                                } else {
                                    self.funds.deduct_expense(crate::economy::Transaction::expense(
                                        crate::economy::TransactionType::CriticalFailure,
                                        amount.abs(),
                                        "Dialogue Cost",
                                        self.current_tick,
                                    ));
                                }
                            }
                            crate::narrative::dialogue::DialogueEffect::TensionChange {
                                apt_a,
                                apt_b,
                                amount,
                            } => {
                                self.tenant_network.apply_tension_change(
                                    apt_a,
                                    apt_b,
                                    amount,
                                    "Dialogue choice",
                                );
                            }
                            crate::narrative::dialogue::DialogueEffect::RelationshipChange {
                                tenant_a,
                                tenant_b,
                                change,
                            } => {
                                self.tenant_network.apply_relationship_change(
                                    tenant_a,
                                    tenant_b,
                                    change,
                                );
                            }
                            crate::narrative::dialogue::DialogueEffect::OpinionChange {
                                tenant_id,
                                amount,
                            } => {
                                if let Some(tenant) =
                                    self.tenants.iter_mut().find(|t| t.id == tenant_id)
                                {
                                    tenant.landlord_opinion =
                                        (tenant.landlord_opinion + amount).clamp(-100, 100);
                                }
                            }
                        }
                    }

                    self.floating_texts.push(FloatingText::new(
                        "Dialogue Resolved",
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        colors::ACCENT,
                    ));
                }
            }
            UiAction::ResolveEventChoice {
                event_id,
                choice_index,
            } => {
                if let Some(effect) = self.narrative_events.process_choice(event_id, choice_index) {
                    self.apply_narrative_effect(&effect);
                }
            }
        }
    }

    pub(super) fn handle_city_action(&mut self, action: crate::ui::city_view::CityMapAction) {
        use crate::ui::city_view::CityMapAction;

        match action {
            CityMapAction::SelectNeighborhood(_id) => {
                // Could show neighborhood details
            }
            CityMapAction::SelectBuilding(index) => {
                self.city.switch_building(index);
                self.sync_building();
                // Stay in map view, just update selection
            }
            CityMapAction::EnterBuilding(index) => {
                self.city.switch_building(index);
                self.sync_building();
                self.view_mode = ViewMode::Building;
            }
            CityMapAction::OpenMarket => {
                self.view_mode = ViewMode::Market;
            }
            CityMapAction::CloseMarket => {
                self.view_mode = ViewMode::CityMap;
            }
            CityMapAction::PurchaseBuilding(listing_id) => {
                self.pending_actions
                    .push(UiAction::PurchaseBuilding { listing_id });
            }
        }
    }

    /// Update tutorial state based on game conditions (called every frame)
    pub fn update_tutorial(&mut self) {
        tutorial_system::update_tutorial(self);
    }

    /// Update missions states (called on turn end)
    pub fn update_missions(&mut self) {
        mission_system::update_missions(self);
    }
}

