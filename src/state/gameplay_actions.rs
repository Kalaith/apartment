//! Game action processing - split from gameplay.rs for maintainability

use macroquad::prelude::*;
use crate::city::NeighborhoodType;
use crate::economy::process_upgrade;
use crate::narrative::{TenantStory, StoryImpact};
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
}
