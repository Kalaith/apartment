
use macroquad::prelude::*;
use std::collections::HashMap;
use super::{StateTransition, ResultsState};
use crate::building::Building;
use crate::tenant::{Tenant, TenantApplication};
use crate::economy::{PlayerFunds, FinancialLedger, process_upgrade};
use crate::simulation::{EventLog, GameOutcome, TickResult, advance_tick, GameEvent};
use crate::ui::{
    Selection, UiAction, FloatingText, Tween,
    draw_header, draw_building_view, draw_apartment_panel, draw_hallway_panel,
    draw_application_panel, draw_notifications,
};
use crate::ui::colors;
use crate::assets::AssetManager;
use crate::ui::layout::HEADER_HEIGHT; // Fix import

// Phase 3 imports
use crate::city::{City, NeighborhoodType};
use crate::consequences::{TenantNetwork, ComplianceSystem, GentrificationTracker};
use crate::narrative::{TenantStory, NarrativeEventSystem, Mailbox};

use serde::{Deserialize, Serialize};

/// View mode for the gameplay screen
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Building,       // Current single-building view
    CityMap,        // City overview with all neighborhoods
    Market,         // Property acquisition screen
    Mail,           // Mailbox view
}

#[derive(Serialize, Deserialize)]
pub struct GameplayState {
    // Phase 3: City replaces single building
    pub city: City,
    
    // Legacy field for backwards compatibility - now derived from city
    #[serde(skip)]
    pub building: Building,
    
    // Tenants
    pub tenants: Vec<Tenant>,
    pub applications: Vec<TenantApplication>,
    pub next_tenant_id: u32,
    
    // Economy
    pub funds: PlayerFunds,
    pub ledger: FinancialLedger,
    
    // Simulation
    pub event_log: EventLog,
    pub current_tick: u32,
    pub game_outcome: Option<GameOutcome>,
    pub last_tick_result: Option<TickResult>,
    
    // Phase 3: Consequence systems
    pub tenant_network: TenantNetwork,
    pub compliance: ComplianceSystem,
    pub gentrification: GentrificationTracker,
    
    // Phase 3: Narrative systems
    pub narrative_events: NarrativeEventSystem,
    pub mailbox: Mailbox,
    pub tenant_stories: HashMap<u32, TenantStory>,
    
    // UI state - skipped from serialization
    #[serde(skip)]
    pub view_mode: ViewMode,
    #[serde(skip)]
    pub selection: Selection,
    #[serde(skip)]
    pub pending_actions: Vec<UiAction>,
    #[serde(skip)]
    pub floating_texts: Vec<FloatingText>,
    #[serde(skip)]
    pub panel_tween: Tween,
}

impl GameplayState {
    /// Create a new game with a starter building in the default neighborhood (Suburbs)
    pub fn new() -> Self {
        Self::new_in_neighborhood(1) // Default to Suburbs (index 1)
    }
    
    /// Create a new game with a starter building in a specific neighborhood
    pub fn new_in_neighborhood(neighborhood_index: usize) -> Self {
        // Create city with starter building
        let city = City::with_starter_building("Metropolis", neighborhood_index);
        
        // Clone the first building for backwards-compatible `building` field
        let building = city.buildings.first().cloned().unwrap_or_else(Building::default_mvp);
        
        // Initialize compliance for the starter building
        let mut compliance = ComplianceSystem::new();
        let is_historic = city.neighborhoods.get(neighborhood_index)
            .map(|n| matches!(n.neighborhood_type, NeighborhoodType::Historic))
            .unwrap_or(false);
        compliance.init_building_regulations(0, is_historic);
        
        let mut state = Self {
            city,
            building,
            tenants: Vec::new(),
            applications: Vec::new(),
            next_tenant_id: 1,
            funds: PlayerFunds::default(),
            ledger: FinancialLedger::default(),
            event_log: EventLog::new(),
            current_tick: 0,
            game_outcome: None,
            last_tick_result: None,
            
            // Phase 3 systems
            tenant_network: TenantNetwork::new(),
            compliance,
            gentrification: GentrificationTracker::new(),
            narrative_events: NarrativeEventSystem::new(),
            mailbox: Mailbox::new(),
            tenant_stories: HashMap::new(),
            
            // UI state
            view_mode: ViewMode::Building,
            selection: Selection::None,
            pending_actions: Vec::new(),
            floating_texts: Vec::new(),
            panel_tween: Tween::new(0.0),
        };
        
        // Generate initial applications for the starter building
        state.applications = crate::tenant::generate_applications(
            &state.building, 
            &[], 
            0, 
            &mut state.next_tenant_id
        );
        
        state
    }
    

    
    /// Save the current `building` state back to the city
    pub fn save_building_to_city(&mut self) {
        if let Some(city_building) = self.city.active_building_mut() {
            *city_building = self.building.clone();
        }
    }
    
    /// Sync the `building` field with the active city building
    pub fn sync_building(&mut self) {
        if let Some(b) = self.city.active_building() {
            self.building = b.clone();
        }
    }

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
                    // Spawn near center for now, could be improved to spawn over specific units
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
        
        // Generate mail
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
    fn process_action(&mut self, action: UiAction) {
        match action {
            UiAction::SelectApartment(id) => {
                self.selection = Selection::Apartment(id);
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
                        description: description,
                        cost,
                    }, self.current_tick);
                    
                    // Floating text
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
                    apt.rent_price = new_rent;
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
                    
                    // Floating text for new tenant
                    let mouse = mouse_position();
                    self.floating_texts.push(FloatingText::new(
                        "Welcome!",
                        mouse.0,
                        mouse.1 - 20.0,
                        colors::POSITIVE,
                    ));
                    
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
                // Find the listing
                if let Some(listing) = self.city.market.listings.iter().find(|l| l.id == listing_id).cloned() {
                    // Check if player can afford it
                    if self.funds.balance >= listing.asking_price {
                        // Purchase the building
                        let building = listing.to_building();
                        let neighborhood_id = listing.neighborhood_id;
                        
                        if let Ok(building_id) = self.city.add_building(building, neighborhood_id) {
                            // Deduct funds
                            let transaction = crate::economy::Transaction::expense(
                                crate::economy::TransactionType::BuildingPurchase,
                                listing.asking_price,
                                "Building Purchase",
                                self.current_tick
                            );
                            self.funds.deduct_expense(transaction);
                            
                            // Initialize compliance for new building
                            let is_historic = self.city.neighborhoods.iter()
                                .any(|n| n.id == neighborhood_id && 
                                    matches!(n.neighborhood_type, NeighborhoodType::Historic));
                            self.compliance.init_building_regulations(building_id, is_historic);
                            
                            // Remove listing from market
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
                    if story.pending_request.is_some() {
                        story.pending_request = None;
                        story.add_event(self.current_tick, "Request approved by landlord", 
                            crate::narrative::StoryImpact::Happiness(15));
                        
                        // Apply happiness bonus to tenant
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
                        
                        // Apply happiness penalty to tenant
                        if let Some(tenant) = self.tenants.iter_mut().find(|t| t.id == tenant_id) {
                            tenant.happiness = (tenant.happiness - 10).max(0);
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, _assets: &AssetManager) -> Option<StateTransition> {
        // Process pending UI actions from previous frame
        let actions: Vec<UiAction> = self.pending_actions.drain(..).collect();
        for action in actions {
            self.process_action(action);
        }
        
        let dt = get_frame_time();
        
        // Update floating texts
        for text in &mut self.floating_texts {
            text.update(dt);
        }
        self.floating_texts.retain(|t| !t.is_dead());
        
        // Update panel animation
        if matches!(self.selection, Selection::None) {
            self.panel_tween.target(0.0);
        } else {
            self.panel_tween.target(1.0);
        }
        self.panel_tween.update(dt);
        
        // Check if game has ended
        if let Some(ref outcome) = self.game_outcome {
            let won = matches!(outcome, GameOutcome::Victory { .. });
            return Some(StateTransition::ToResults(ResultsState::new(
                self.funds.total_income,
                self.tenants.len() as u32,
                0,
                won,
            )));
        }
        
        // Handle keyboard input for ending turn (Space)
        if is_key_pressed(KeyCode::Space) && matches!(self.view_mode, ViewMode::Building) {
            self.end_turn();
        }
        
        // Tab key toggles between Building and CityMap views
        if is_key_pressed(KeyCode::Tab) {
            self.view_mode = match self.view_mode {
                ViewMode::Building => ViewMode::CityMap,
                ViewMode::CityMap => ViewMode::Building,
                ViewMode::Market => ViewMode::CityMap,
                ViewMode::Mail => ViewMode::Building,
            };
            self.selection = Selection::None;
        }
        
        
        // Background
        draw_rectangle(0.0, 0.0, screen_width(), HEADER_HEIGHT, colors::PANEL_HEADER);
        
        // Title
        draw_text_ex(
            &format!("{} - City Overview", self.city.name),
            20.0,
            35.0,
            TextParams {
                font_size: 28,
                color: colors::TEXT,
                ..Default::default()
            },
        );
        
        // Funds
        draw_text_ex(
            &format!("${}", self.funds.balance),
            screen_width() - 200.0,
            35.0,
            TextParams {
                font_size: 24,
                color: colors::POSITIVE,
                ..Default::default()
            },
        );
        
        // Buildings count
        draw_text_ex(
            &format!("{} Buildings | Month {}", self.city.buildings.len(), self.current_tick),
            screen_width() - 400.0,
            35.0,
            TextParams {
                font_size: 16,
                color: colors::TEXT_DIM,
                ..Default::default()
            },
        );
        
        // Navigation hint
        draw_text_ex(
            "[Tab] Building View | [M] Mail",
            20.0,
            55.0,
            TextParams {
                font_size: 14,
                color: colors::TEXT_DIM,
                ..Default::default()
            },
        );
        
        None
    }
    
    /// Handle city map actions
    fn handle_city_action(&mut self, action: crate::ui::city_view::CityMapAction) {
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
            CityMapAction::ManageBuilding(index) => {
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
            CityMapAction::BackToBuilding => {
                self.view_mode = ViewMode::Building;
            }
        }
    }
    
    /// Draw mail view
    fn draw_mail_view(&self, _assets: &AssetManager) {
        use crate::ui::layout::HEADER_HEIGHT;
        
        // Header
        draw_rectangle(0.0, 0.0, screen_width(), HEADER_HEIGHT, colors::PANEL_HEADER);
        draw_text_ex(
            "Mailbox",
            20.0,
            35.0,
            TextParams {
                font_size: 28,
                color: colors::TEXT,
                ..Default::default()
            },
        );
        
        // Unread count
        let unread = self.mailbox.unread_count();
        if unread > 0 {
            draw_text_ex(
                &format!("{} unread", unread),
                150.0,
                35.0,
                TextParams {
                    font_size: 16,
                    color: colors::WARNING,
                    ..Default::default()
                },
            );
        }
        
        // Mail list
        let start_y = HEADER_HEIGHT + 20.0;
        let mail_height = 80.0;
        
        for (i, mail) in self.mailbox.recent(10).iter().enumerate() {
            let y = start_y + i as f32 * (mail_height + 10.0);
            
            // Background
            let bg_color = if mail.read {
                Color::from_rgba(40, 40, 45, 255)
            } else {
                Color::from_rgba(50, 55, 70, 255)
            };
            draw_rectangle(20.0, y, screen_width() - 40.0, mail_height, bg_color);
            
            // Icon
            draw_text_ex(
                mail.mail_type.icon(),
                30.0,
                y + 30.0,
                TextParams {
                    font_size: 24,
                    color: colors::TEXT,
                    ..Default::default()
                },
            );
            
            // Subject
            draw_text_ex(
                &mail.subject,
                60.0,
                y + 25.0,
                TextParams {
                    font_size: 18,
                    color: if mail.read { colors::TEXT_DIM } else { colors::TEXT },
                    ..Default::default()
                },
            );
            
            // Sender
            draw_text_ex(
                &format!("From: {}", mail.sender),
                60.0,
                y + 45.0,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT_DIM,
                    ..Default::default()
                },
            );
            
            // Month
            draw_text_ex(
                &format!("Month {}", mail.month_received),
                screen_width() - 120.0,
                y + 25.0,
                TextParams {
                    font_size: 12,
                    color: colors::TEXT_DIM,
                    ..Default::default()
                },
            );
        }
        
        // Back hint
        draw_text_ex(
            "[Esc] Back to Building",
            20.0,
            screen_height() - 30.0,
            TextParams {
                font_size: 14,
                color: colors::TEXT_DIM,
                ..Default::default()
            },
        );
    }

    pub fn draw(&mut self, assets: &AssetManager) {
        match self.view_mode {
            ViewMode::Building => {
                self.draw_building_mode(assets);
            }
            ViewMode::CityMap => {
                if let Some(action) = crate::ui::city_view::draw_city_map(&self.city, assets) {
                    self.handle_city_action(action);
                }
                
                 if let Some(action) = crate::ui::city_view::draw_portfolio_panel(&self.city, self.city.active_building_index, assets) {
                     self.handle_city_action(action);
                 }
            }
            ViewMode::Market => {
                 // Convert listings to vector of references
                 let listings: Vec<&crate::city::PropertyListing> = self.city.market.listings.iter().collect();
                 if let Some(action) = crate::ui::city_view::draw_market_panel(&listings, &self.city.neighborhoods, self.funds.balance, assets) {
                     self.handle_city_action(action);
                 }
            }
            ViewMode::Mail => {
                self.draw_mail_view(assets);
            }
        }
        
        draw_notifications(&self.event_log, self.current_tick, assets);
        
        // Floating text
        for text in &self.floating_texts {
            text.draw();
        }
    }

    fn draw_building_mode(&mut self, assets: &AssetManager) {
        // Draw Header
        if let Some(action) = draw_header(
            self.funds.balance, 
            self.current_tick, 
            &self.building.name, 
            self.building.occupancy_count(), 
            self.building.apartments.len(), 
            assets
        ) {
            self.pending_actions.push(action);
        }
        
        // Draw Building View
        if let Some(action) = draw_building_view(&self.building, &self.tenants, &self.selection, assets) {
            self.pending_actions.push(action);
        }
        
        match self.selection {
            Selection::Apartment(id) => {
                 if let Some(apt) = self.building.get_apartment(id) {
                     if let Some(action) = draw_apartment_panel(apt, &self.building, &self.tenants, self.funds.balance, 0.0, assets) {
                         self.pending_actions.push(action);
                     }
                 }
            }
            Selection::Hallway => {
                if let Some(action) = draw_hallway_panel(&self.building, self.funds.balance, 0.0, assets) {
                    self.pending_actions.push(action);
                }
            }
            Selection::Applications => {
                if let Some(action) = draw_application_panel(&self.applications, &self.building, 0.0, assets) {
                    self.pending_actions.push(action);
                }
            }
            _ => {}
        }
    }
}
