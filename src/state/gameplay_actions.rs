//! Game action processing - split from gameplay.rs for maintainability

use macroquad::prelude::*;
use crate::city::NeighborhoodType;
use crate::economy::process_upgrade;
use crate::narrative::{TenantStory, StoryImpact, TutorialMilestone, MissionGoal, MissionReward};
use crate::simulation::{advance_tick, GameEvent};
use crate::ui::{UiAction, Selection, FloatingText, colors};

use super::gameplay::{GameplayState, ViewMode};

impl GameplayState {
    /// End the current turn and advance time
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
        );
        
        self.game_outcome = result.outcome.clone();
        
        // Spawn floating texts for events
        for event in &result.events {
            match event {
                GameEvent::RentPaid { amount, .. } => {
                    let x = screen_width() / 2.0;
                    let y = screen_height() / 2.0;
                    self.floating_texts.push(FloatingText::new(
                        &format!("+${}", amount),
                        x + macroquad::rand::gen_range(-50.0, 50.0),
                        y + macroquad::rand::gen_range(-50.0, 50.0),
                        colors::POSITIVE,
                    ));
                }
                GameEvent::RentMissed { .. } => {
                    let x = screen_width() / 2.0;
                    let y = screen_height() / 2.0;
                    self.floating_texts.push(FloatingText::new(
                        "Missed Rent!",
                        x,
                        y,
                        colors::NEGATIVE,
                    ));
                }
                GameEvent::TenantUnhappy { .. } => {
                    let x = screen_width() / 2.0;
                    let y = screen_height() / 2.0;
                    self.floating_texts.push(FloatingText::new(
                        "Unhappy!",
                        x + macroquad::rand::gen_range(-50.0, 50.0),
                        y + macroquad::rand::gen_range(-50.0, 50.0),
                        colors::WARNING,
                    ));
                }
                _ => {}
            }
        }
        
        // === Phase 3: Update consequence and narrative systems ===
        
        // Save local building changes to city before processing city updates
        self.save_building_to_city();
        
        // Update city (neighborhoods, market)
        self.city.tick();
        
        // Update tenant relationships
        self.tenant_network.tick(&self.tenants, &self.building);
        
        // Update compliance system (check inspection timers)
        self.compliance.tick(self.current_tick);
        
        // Update gentrification tracking
        self.gentrification.update_affordable_units(&self.building.apartments);
        
        // Generate narrative events
        self.narrative_events.generate_events(
            self.current_tick,
            &self.city.neighborhoods,
            &self.city.buildings,
            &self.tenants,
        );
        
        // Generate and cleanup mail
        let income = self.last_tick_result.as_ref().map(|r| r.rent_collected).unwrap_or(0);
        let expenses = self.funds.transactions_for_tick(self.current_tick)
            .iter()
            .filter(|t| t.amount < 0)
            .map(|t| t.amount.abs())
            .sum();
        self.mailbox.generate_mail(
            self.current_tick,
            income,
            expenses,
            &self.tenants,
            &self.city.buildings,
        );
        self.mailbox.cleanup(self.current_tick, 12);
        
        // Generate dialogues based on tenant state
        // Clone needed data to avoid borrow issues
        let tenants_clone = self.tenants.clone();
        let building_clone = self.building.clone();
        let funds_clone = self.funds.clone();
        self.dialogue_system.generate_dialogues(
            self.current_tick,
            &tenants_clone,
            &building_clone,
            &funds_clone,
        );
        
        // Auto-accept available missions (player can view in mission log)
        let available_ids: Vec<u32> = self.missions.available_missions()
            .iter()
            .map(|m| m.id)
            .collect();
        for mission_id in available_ids {
            self.missions.accept_mission(mission_id, self.current_tick);
        }
        
        // Generate tenant requests periodically
        for tenant in &self.tenants {
            if let Some(story) = self.tenant_stories.get_mut(&tenant.id) {
                if macroquad::rand::gen_range(0, 100) < 10 {
                    story.make_request(&tenant.archetype);
                }
            }
        }
        
        // Auto-expire old narrative events
        let expired_event_ids: Vec<u32> = self.narrative_events.events_requiring_response()
            .iter()
            .filter(|e| e.is_expired(self.current_tick))
            .map(|e| e.id)
            .collect();
        for event_id in expired_event_ids {
            self.narrative_events.process_choice(event_id, 0);
        }
        
        // Sync building field with city
        self.sync_building();
        
        // Auto-save
        if let Err(e) = crate::save::save_game(self) {
            eprintln!("Failed to save game: {}", e);
            self.floating_texts.push(FloatingText::new(
                "Save Failed!",
                screen_width() / 2.0,
                screen_height() / 2.0,
                colors::NEGATIVE,
            ));
        }
        
        // Generate new missions based on progression
        self.missions.generate_late_game_missions(self.current_tick);
        
        // Check for annual awards (at year boundaries)
        if self.current_tick % 12 == 0 && self.current_tick > 0 {
            let avg_happiness = if self.tenants.is_empty() {
                0.0
            } else {
                self.tenants.iter().map(|t| t.happiness as f32).sum::<f32>() / self.tenants.len() as f32
            };
            let total = self.building.apartments.len();
            let occupied = self.building.occupancy_count();
            let occupancy_rate = if total > 0 { occupied as f32 / total as f32 } else { 0.0 };
            
            self.missions.check_for_awards(
                self.current_tick,
                &self.building.name,
                avg_happiness,
                occupancy_rate,
                self.tenants.len() as u32,
            );
            
            // Check for tenant council formation
            if self.tenant_network.should_form_council(&self.tenants) {
                self.floating_texts.push(FloatingText::new(
                    "Tenants forming a council!",
                    screen_width() / 2.0,
                    screen_height() / 2.0 + 30.0,
                    colors::ACCENT,
                ));
                self.missions.record_legacy_event(
                    self.current_tick,
                    "Tenant Council Formed",
                    "Tenants organized to form a tenant council."
                );
            }
        }
        
        // Calculate and apply community cohesion bonus to happiness
        let cohesion = self.tenant_network.calculate_cohesion(&self.tenants);
        for tenant in &mut self.tenants {
            // Apply relationship happiness modifier (uses RelationshipType::happiness_modifier)
            let relationship_bonus = crate::tenant::happiness::calculate_relationship_happiness(
                tenant.id,
                &self.tenant_network,
            );
            
            // Apply cohesion bonus if threshold met
            let cohesion_bonus = if cohesion > 20 { 1 } else { 0 };
            
            tenant.happiness = (tenant.happiness + relationship_bonus + cohesion_bonus).clamp(0, 100);
        }
        
        // Log marketing campaign status
        let marketing_name = self.building.marketing_strategy.name();
        if marketing_name != "None" {
            println!("Active marketing campaign: {}", marketing_name);
        }
        
        // Check for pending dialogues
        let pending_count = self.dialogue_system.pending_dialogues().len();
        if pending_count > 0 {
            println!("Pending dialogues: {}", pending_count);
        }
        
        self.update_missions();
        self.last_tick_result = Some(result);
    }
    
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
            UiAction::SelectApplications => {
                self.selection = Selection::Applications;
            }
            UiAction::SelectHallway => {
                self.selection = Selection::Hallway;
            }
            UiAction::ClearSelection => {
                self.selection = Selection::None;
            }

            UiAction::UpgradeAction(upgrade) => {
                let description = upgrade.label(&self.building);
                if let Ok(cost) = process_upgrade(&upgrade, &mut self.building, &mut self.funds, self.current_tick) {
                    self.event_log.log(GameEvent::UpgradeCompleted {
                        description,
                        cost,
                    }, self.current_tick);
                    
                    let mouse = mouse_position();
                    self.floating_texts.push(FloatingText::new(
                        &format!("-${}", cost),
                        mouse.0,
                        mouse.1 - 20.0,
                        colors::NEGATIVE,
                    ));
                }
            }
            UiAction::SetRent { apartment_id, new_rent } => {
                if let Some(apt) = self.building.get_apartment_mut(apartment_id) {
                    let old_rent = apt.rent_price;
                    apt.rent_price = new_rent;
                    
                    if old_rent != new_rent {
                        self.gentrification.record_rent_change(0, self.current_tick, old_rent, new_rent);
                    }
                }
            }
            UiAction::AcceptApplication { application_index } => {
                if application_index < self.applications.len() {
                    let app = self.applications.remove(application_index);
                    let mut tenant = app.tenant;
                    
                    // Evaluate lease using the standard offer
                    if let Some(apt) = self.building.get_apartment(app.apartment_id) {
                        let offer = crate::tenant::matching::LeaseOffer::standard(apt.rent_price);
                        let accept_probability = crate::tenant::matching::evaluate_lease_offer(&tenant, &offer);
                        
                        // Log the negotiation outcome
                        let leverage = tenant.negotiation_leverage();
                        println!("Tenant {} has negotiation leverage: {}, accept probability: {:.2}", 
                            tenant.name, leverage, accept_probability);
                    }
                    
                    tenant.move_into(app.apartment_id);
                    
                    if let Some(apt) = self.building.get_apartment_mut(app.apartment_id) {
                        apt.move_in(tenant.id);
                    }
                    
                    self.event_log.log(GameEvent::TenantMovedIn {
                        tenant_name: tenant.name.clone(),
                        apartment_unit: self.building.get_apartment(app.apartment_id)
                            .map(|a| a.unit_number.clone())
                            .unwrap_or_default(),
                    }, self.current_tick);
                    
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
                    if let Some(result) = crate::tenant::vetting::perform_credit_check(app, &mut self.funds) {
                        self.floating_texts.push(FloatingText::new(
                            &format!("Credit: {} - {}", result.reliability_score, result.recommendation),
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            if result.reliability_score >= 75 { colors::POSITIVE } 
                            else if result.reliability_score >= 50 { colors::WARNING } 
                            else { colors::NEGATIVE },
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
                    if let Some(result) = crate::tenant::vetting::perform_background_check(app, &mut self.funds) {
                        self.floating_texts.push(FloatingText::new(
                            &format!("Background: {} - {}", result.behavior_score, result.history_notes),
                            screen_width() / 2.0,
                            screen_height() / 2.0,
                            if result.behavior_score >= 75 { colors::POSITIVE }
                            else if result.behavior_score >= 50 { colors::WARNING }
                            else { colors::NEGATIVE },
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
                // Handled in update() return value
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
                if let Some(listing) = self.city.market.listings.iter().find(|l| l.id == listing_id).cloned() {
                    if self.funds.balance >= listing.asking_price {
                        let building = listing.to_building();
                        let neighborhood_id = listing.neighborhood_id;
                        
                        if let Ok(building_id) = self.city.add_building(building, neighborhood_id) {
                            let transaction = crate::economy::Transaction::expense(
                                crate::economy::TransactionType::BuildingPurchase,
                                listing.asking_price,
                                "Building Purchase",
                                self.current_tick
                            );
                            self.funds.deduct_expense(transaction);
                            
                            let is_historic = self.city.neighborhoods.iter()
                                .any(|n| n.id == neighborhood_id && 
                                    matches!(n.neighborhood_type, NeighborhoodType::Historic));
                            self.compliance.init_building_regulations(building_id, is_historic);
                            
                            self.city.market.listings.retain(|l| l.id != listing_id);
                            
                            self.floating_texts.push(FloatingText::new(
                                "Building Purchased!",
                                screen_width() / 2.0,
                                screen_height() / 2.0,
                                colors::POSITIVE,
                            ));
                            
                            self.event_log.log(GameEvent::UpgradeCompleted {
                                description: "Purchased new building".to_string(),
                                cost: listing.asking_price,
                            }, self.current_tick);
                        }
                    }
                }
            }
            
            // Phase 3: Tenant requests
            UiAction::ApproveRequest { tenant_id } => {
                if let Some(story) = self.tenant_stories.get_mut(&tenant_id) {
                    if story.pending_request.take().is_some() {
                        story.add_event(self.current_tick, "Request approved by landlord", 
                            StoryImpact::Happiness(15));
                        
                        if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                            tenant.happiness = (tenant.happiness + 15).min(100);
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
            UiAction::VoteOnProposal { proposal_index: _index, vote_yes: _vote } => {
                self.floating_texts.push(FloatingText::new(
                    "Vote Cast",
                    screen_width() / 2.0,
                    screen_height() / 2.0,
                    colors::ACCENT,
                ));
            }
            UiAction::SellUnitAsCondo { apartment_id } => {
                let sale_price = self.building.get_apartment(apartment_id)
                    .map(|apt| apt.market_value())
                    .unwrap_or(10_000);
                
                if let Some(apt) = self.building.get_apartment(apartment_id) {
                    if let Some(tenant_id) = apt.tenant_id {
                        self.tenants.retain(|t| t.id != tenant_id);
                        self.tenant_stories.remove(&tenant_id);
                    }
                }
                
                if self.building.convert_unit_to_condo(apartment_id, "New Owner", sale_price) {
                     let transaction = crate::economy::Transaction::income(
                        crate::economy::TransactionType::AssetSale,
                        sale_price,
                        "Condo Sale",
                        self.current_tick
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
                            self.current_tick
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
            UiAction::ResolveDialogue { dialogue_id, choice_index } => {
                if let Some(effects) = self.dialogue_system.resolve_dialogue(dialogue_id, choice_index) {
                    // Apply dialogue effects
                    for effect in effects {
                        match effect {
                            crate::narrative::dialogue::DialogueEffect::HappinessChange { tenant_id, amount } => {
                                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                                    tenant.happiness = (tenant.happiness + amount).clamp(0, 100);
                                }
                            }
                            crate::narrative::dialogue::DialogueEffect::MoneyChange(amount) => {
                                if amount > 0 {
                                    self.funds.balance += amount;
                                } else {
                                    self.funds.spend(amount.abs());
                                }
                            }
                            crate::narrative::dialogue::DialogueEffect::TensionChange { .. } => {
                                // TODO: Apply to tenant network
                            }
                            crate::narrative::dialogue::DialogueEffect::RelationshipChange { .. } => {
                                // TODO: Apply to tenant relationships
                            }
                            crate::narrative::dialogue::DialogueEffect::OpinionChange { tenant_id, amount } => {
                                if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                                    tenant.landlord_opinion = (tenant.landlord_opinion + amount).clamp(-100, 100);
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
                self.view_mode = ViewMode::Building;
            }
            CityMapAction::OpenMarket => {
                self.view_mode = ViewMode::Market;
            }
            CityMapAction::CloseMarket => {
                self.view_mode = ViewMode::CityMap;
            }
            CityMapAction::PurchaseBuilding(listing_id) => {
                self.pending_actions.push(UiAction::PurchaseBuilding { listing_id });
            }
        }
    }

    /// Update tutorial state based on game conditions (called every frame)
    pub fn update_tutorial(&mut self) {
        // Skip if tutorial is complete
        if self.tutorial.is_complete() {
            return;
        }
        
        if !self.tutorial.active {
            return;
        }
        
        // Check if we should introduce the rival (Magnuson Corp)
        if self.tutorial.should_introduce_rival(self.current_tick) && !self.tutorial.rival_introduced {
            // Get rival NPC info and add an introduction message
            if let Some(rival) = self.tutorial.get_npc(1) { // Magnuson Corp ID = 1
                let rival_name = rival.name.clone();
                self.tutorial.pending_messages.push(
                    format!("I hear {} has been buying up properties nearby. Watch out for them!", rival_name)
                );
                self.tutorial.rival_introduced = true;
            }
        }
        
        // Display hint for current milestone if stuck for a while
        if let Some(hint) = self.tutorial.get_hint() {
            // Show hint as floating text occasionally (every 5 ticks if no progress)
            if self.current_tick % 5 == 0 && self.current_tick > 0 {
                self.floating_texts.push(FloatingText::new(
                    hint,
                    screen_width() / 2.0,
                    screen_height() - 100.0,
                    colors::TEXT_DIM,
                ));
            }
        }

        if let Some(milestone) = &self.tutorial.current_milestone {
            match milestone {
                TutorialMilestone::InheritedMess => {
                    // Completes when building is sufficiently clean/repaired
                    // For MVP: Hallway condition > 80
                    if self.building.hallway_condition >= 80 {
                        // Improve relationship with mentor for completing milestone
                        self.tutorial.modify_relationship(0, 10); // Uncle Artie ID = 0
                        
                        self.tutorial.complete_milestone(TutorialMilestone::InheritedMess);
                        self.floating_texts.push(FloatingText::new(
                             "Tutorial: Cleaned Up!",
                             screen_width() / 2.0,
                             screen_height() / 2.0,
                             colors::POSITIVE,
                        ));
                    }
                }
                TutorialMilestone::FirstResident => {
                    // Completes when at least one tenant exists
                    if !self.tenants.is_empty() {
                        self.tutorial.modify_relationship(0, 15); // Mentor happy
                        
                         self.tutorial.complete_milestone(TutorialMilestone::FirstResident);
                         self.floating_texts.push(FloatingText::new(
                             "Tutorial: First Resident!",
                             screen_width() / 2.0,
                             screen_height() / 2.0 + 30.0,
                             colors::POSITIVE,
                        ));
                        
                        // Trigger The Leak event immediately implies a problem
                        // We can sabotage a unit or just let narrative flow
                         if let Some(apt) = self.building.apartments.first_mut() {
                             apt.condition = 40; // Force damage
                         }
                    }
                }
                TutorialMilestone::TheLeak => {
                    // Check if repairs are done (condition > 60 for all units?)
                    let all_good = self.building.apartments.iter().all(|a| a.condition > 60);
                    if all_good {
                        self.tutorial.modify_relationship(0, 20); // Mentor very happy
                        
                        self.tutorial.complete_milestone(TutorialMilestone::TheLeak);
                         self.floating_texts.push(FloatingText::new(
                             "Tutorial Complete!",
                             screen_width() / 2.0,
                             screen_height() / 2.0,
                             colors::POSITIVE,
                        ));
                        
                        // Pop final congratulations message
                        while let Some(msg) = self.tutorial.pop_message() {
                            self.floating_texts.push(FloatingText::new(
                                &msg,
                                screen_width() / 2.0,
                                screen_height() / 2.0 + 60.0,
                                colors::ACCENT,
                            ));
                        }
                    }
                }
                TutorialMilestone::Complete => {}
            }
        }
    }

    /// Update missions states (called on turn end)
    pub fn update_missions(&mut self) {
        let current_month = self.current_tick;
        
        // Check for expirations (expired missions are marked as such)
        self.missions.check_expirations(current_month);
        
        // Check for unrecoverable failures (e.g., building sold that was needed)
        for mission in &mut self.missions.missions {
            if mission.status == crate::narrative::MissionStatus::Active {
                // Check if "AcquireBuilding" mission should fail if we sold a building
                if matches!(mission.goal, MissionGoal::AcquireBuilding) && self.city.buildings.is_empty() {
                    mission.fail();
                    self.floating_texts.push(FloatingText::new(
                        "Mission Failed!",
                        screen_width() / 2.0,
                        screen_height() / 2.0,
                        colors::NEGATIVE,
                    ));
                }
            }
        }
        
        // Check active missions for completion
        let active_mission_ids: Vec<u32> = self.missions.active_missions().iter().map(|m| m.id).collect();
        
        for mission_id in active_mission_ids {
            let mut completed = false;
            let mut reward = None;
            let mut legacy_info: Option<(String, String)> = None;
            
            if let Some(mission) = self.missions.missions.iter_mut().find(|m| m.id == mission_id) {
                match &mission.goal {
                    MissionGoal::HouseTenants { count, archetype } => {
                        let current_count = self.tenants.iter()
                            .filter(|t| archetype.as_ref().map_or(true, |arch| t.archetype.name() == arch))
                            .count();
                        if current_count as u32 >= *count {
                            completed = true;
                        }
                    }
                    MissionGoal::ReachOccupancy { percentage } => {
                        let total = self.building.apartments.len();
                        let occupied = self.building.occupancy_count();
                        if total > 0 && (occupied as f32 / total as f32) >= *percentage {
                            completed = true;
                        }
                    }
                    MissionGoal::AcquireBuilding => {
                        if self.city.buildings.len() > 1 { // Started with 1
                            completed = true;
                        }
                    }
                    // Implement other goals...
                    _ => {}
                }
                
                if completed {
                    mission.complete();
                    reward = Some(mission.reward.clone());
                    legacy_info = Some((mission.title.clone(), mission.description.clone()));
                }
            }
            
            // Record legacy (outside the mutable borrow)
            if let Some((title, description)) = legacy_info {
                self.missions.record_legacy_event(
                    current_month, 
                    &format!("Mission Complete: {}", title), 
                    &format!("Completed objective: {}", description)
                );
            }
            
            // Grant reward
            if let Some(r) = reward {
                match r {
                    MissionReward::Money(amount) => {
                        let t = crate::economy::Transaction::income(
                            crate::economy::TransactionType::Grant,
                            amount,
                            "Mission Reward",
                            current_month
                        );
                        self.funds.add_income(t);
                        self.floating_texts.push(FloatingText::new(
                             &format!("+${} Reward", amount),
                             screen_width() / 2.0,
                             screen_height() / 2.0,
                             colors::POSITIVE,
                        ));
                    }
                    MissionReward::TaxBreak { .. } => {
                         // Implement tax break logic... placeholder
                    }
                    MissionReward::Reputation(amount) => {
                         // TODO: Implement reputation system
                         println!("Earned {} reputation points (system not yet implemented)", amount);
                    }
                     MissionReward::UnlockBuilding(_) => {}
                }
            }
        }
    }
}
