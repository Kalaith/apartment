
use macroquad::prelude::*;
use std::collections::HashMap;
use super::{StateTransition};
use crate::building::Building;
use crate::data::config::GameConfig;
use crate::tenant::{Tenant, TenantApplication};
use crate::economy::{PlayerFunds, FinancialLedger};
use crate::simulation::{EventLog, GameOutcome, TickResult};
use crate::ui::{Selection, UiAction, FloatingText, Tween, colors};
use crate::assets::AssetManager;
use crate::ui::layout::HEADER_HEIGHT;

// Phase 3 imports
use crate::city::City;
use crate::consequences::{TenantNetwork, ComplianceSystem, GentrificationTracker};
use crate::narrative::{TenantStory, NarrativeEventSystem, Mailbox, TutorialManager, MissionManager, NotificationManager, TenantEventsConfig, load_events_config, RelationshipEventsConfig, load_relationship_config};

use serde::{Deserialize, Serialize};

/// View mode for the gameplay screen
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Building,       // Current single-building view
    CityMap,        // City overview with all neighborhoods
    Market,         // Property acquisition screen
    Mail,           // Mailbox view
    CareerSummary,  // Phase 5: Endgame result
}

#[derive(Serialize, Deserialize)]
pub struct GameplayState {
    // Phase 3: City replaces single building
    pub city: City,
    
    // Legacy field for backwards compatibility - now derived from city
    #[serde(skip)]
    pub building: Building,

    #[serde(skip)]
    pub config: GameConfig,
    
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
    pub dialogue_system: crate::narrative::DialogueSystem,
    #[serde(skip)]
    pub tenant_events_config: TenantEventsConfig,
    #[serde(skip)]
    pub relationship_events_config: RelationshipEventsConfig,
    
    // Phase 4: Tutorial & Missions
    pub tutorial: TutorialManager,
    pub missions: MissionManager,
    
    // Phase 5: Notifications (relationship changes, hints)
    pub notifications: NotificationManager,
    
    // Phase 5: Achievements
    pub achievements: crate::narrative::AchievementSystem,
    
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
    #[serde(skip)]
    pub panel_scroll_offset: f32,
    #[serde(skip)]
    pub show_pause_menu: bool,
    #[serde(skip)]
    pub is_fullscreen: bool,
    #[serde(skip)]
    pub pending_quit_to_menu: bool,
    
    /// Current building template ID (for unlock tracking)
    #[serde(default)]
    pub current_building_id: String,
}

impl GameplayState {
    /// Create a new game with a specific building template
    pub fn new_with_template(config: GameConfig, template: crate::data::templates::BuildingTemplate) -> Self {
        use crate::building::Building;
        
        // Create building from template
        let building = Building::from_template(&template);
        let building_id = template.id.clone();
        
        // Create minimal city with just this building
        let mut city = City::new("Metropolis");
        city.buildings.push(building.clone());
        city.total_buildings_managed = 1;
        
        // Initialize compliance
        let compliance = ComplianceSystem::new();
        
        let mut state = Self {
            city,
            building,
            config,
            tenants: Vec::new(),
            applications: Vec::new(),
            next_tenant_id: 1,
            funds: PlayerFunds::default(),
            ledger: FinancialLedger::default(),
            event_log: EventLog::new(),
            current_tick: 0,
            game_outcome: None,
            last_tick_result: None,
            
            tenant_network: TenantNetwork::new(),
            compliance,
            gentrification: GentrificationTracker::new(),
            narrative_events: NarrativeEventSystem::new(),
            mailbox: Mailbox::new(),
            tenant_stories: HashMap::new(),
            dialogue_system: crate::narrative::DialogueSystem::new(),
            tenant_events_config: load_events_config(),
            relationship_events_config: load_relationship_config(),
            
            tutorial: TutorialManager::new(),
            missions: MissionManager::new(),
            notifications: NotificationManager::new(),
            achievements: crate::narrative::AchievementSystem::new(),
            
            view_mode: ViewMode::Building,
            selection: Selection::None,
            pending_actions: Vec::new(),
            floating_texts: Vec::new(),
            panel_tween: Tween::new(0.0),
            panel_scroll_offset: 0.0,
            show_pause_menu: false,
            is_fullscreen: false,
            pending_quit_to_menu: false,
            current_building_id: building_id,
        };
        
        // Handle initial tenant if present in template
        if let Some(data) = &template.initial_tenant {
            if let Some(archetype) = crate::tenant::TenantArchetype::from_id(&data.archetype) {
                if let Some(apt) = state.building.apartments.iter_mut().find(|a| a.unit_number == data.apartment_unit) {
                    let tenant_id = state.next_tenant_id;
                    state.next_tenant_id += 1;
                    
                    let mut tenant = Tenant::new(tenant_id, &data.name, archetype);
                    tenant.move_into(apt.id);
                    apt.move_in(tenant_id);
                    
                    state.tenants.push(tenant);
                    
                    if let Some(city_building) = state.city.active_building_mut() {
                        if let Some(city_apt) = city_building.apartments.iter_mut().find(|a| a.id == apt.id) {
                            city_apt.move_in(tenant_id);
                        }
                    }
                }
            }
        }
        
        // Generate initial applications
        state.applications = crate::tenant::generate_applications(
            &state.building, 
            &[], 
            0, 
            &mut state.next_tenant_id,
            &state.config.matching,
        );
        
        state.missions.generate_starter_missions();
        
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
    
    /// Unlock the next building after completing the current one
    pub fn unlock_next_building(&self) {
        use crate::save::{load_player_progress, save_player_progress};
        use crate::data::templates::load_templates;
        
        let mut progress = load_player_progress();
        
        // Mark current building as completed
        progress.mark_completed(&self.current_building_id);
        
        // Find the next building to unlock based on unlock_order
        if let Some(templates) = load_templates() {
            // Find current building's unlock_order
            let current_order = templates.templates.iter()
                .find(|t| t.id == self.current_building_id)
                .map(|t| t.unlock_order)
                .unwrap_or(0);
            
            // Find the next building in sequence
            if let Some(next_template) = templates.templates.iter()
                .filter(|t| t.unlock_order == current_order + 1)
                .next()
            {
                progress.unlock_building(&next_template.id);
            }
        }
        
        // Save progress
        let _ = save_player_progress(&progress);
    }

    /// Main update function - handles game logic and input
    pub fn update(&mut self, assets: &AssetManager) -> Option<StateTransition> {
        // Ensure assets are loaded before processing
        if !assets.loaded {
            return None;
        }
        
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
        
        // Dialogue generation happens in end_turn() via gameplay_actions.rs
        // Update Dialogue System timeouts
        self.dialogue_system.tick(self.current_tick);
        
        // Update panel animation
        if matches!(self.selection, Selection::None) {
            self.panel_tween.target(0.0);
        } else {
            self.panel_tween.target(1.0);
        }
        self.panel_tween.update(dt);
        
        // Check if game has ended
        // Phase 5: Use CareerSummary view instead of StateTransition
        if self.game_outcome.is_some() && self.view_mode != ViewMode::CareerSummary {
             self.view_mode = ViewMode::CareerSummary;
             // Check final achievements immediately
             let new_unlocks = self.achievements.check_new_unlocks(
                 &self.city, &self.building, &self.tenants, &self.funds, self.current_tick, &self.config
             );
             for id in new_unlocks {
                 self.achievements.unlock(&id);
             }
        }
        
        
        // Update tutorial
        self.update_tutorial();
        
        // Handle tutorial "Next" button click
        if self.tutorial.active && !self.tutorial.pending_messages.is_empty() {
            // Button layout must match draw_tutorial_overlay
            let panel_w = 650.0;
            let panel_h = 180.0;
            let panel_x = (screen_width() - panel_w) / 2.0;
            let panel_y = screen_height() - panel_h - 20.0;
            let btn_w = 120.0;
            let btn_h = 35.0;
            let btn_x = panel_x + panel_w - btn_w - 20.0;
            let btn_y = panel_y + panel_h - btn_h - 15.0;
            
            let mouse = mouse_position();
            if mouse.0 >= btn_x && mouse.0 <= btn_x + btn_w && 
               mouse.1 >= btn_y && mouse.1 <= btn_y + btn_h &&
               is_mouse_button_pressed(MouseButton::Left) {
                self.tutorial.pending_messages.remove(0);
            }
        }
        // Handle notification "OK" button click (when tutorial is not blocking)
        else if self.notifications.has_pending() {
            // Button layout must match draw_notification_overlay
            let panel_w = 550.0;
            let panel_h = 150.0;
            let panel_x = (screen_width() - panel_w) / 2.0;
            let panel_y = screen_height() - panel_h - 20.0;
            let btn_w = 100.0;
            let btn_h = 32.0;
            let btn_x = panel_x + panel_w - btn_w - 15.0;
            let btn_y = panel_y + panel_h - btn_h - 12.0;
            
            let mouse = mouse_position();
            if mouse.0 >= btn_x && mouse.0 <= btn_x + btn_w && 
               mouse.1 >= btn_y && mouse.1 <= btn_y + btn_h &&
               is_mouse_button_pressed(MouseButton::Left) {
                self.notifications.pop();
            }
        }

        
        // Handle keyboard input for ending turn (Space)
        if is_key_pressed(KeyCode::Space) && matches!(self.view_mode, ViewMode::Building) {
            self.end_turn();
        }
        
        // ESC key toggles pause menu
        if is_key_pressed(KeyCode::Escape) {
            self.show_pause_menu = !self.show_pause_menu;
        }
        
        // If pause menu is showing, skip regular game input processing but check for quit
        if self.show_pause_menu {
            if self.pending_quit_to_menu {
                self.pending_quit_to_menu = false;
                return Some(StateTransition::ToMenu);
            }
            return None;
        }

        // Global check for quit (e.g. from Career Summary)
        if self.pending_quit_to_menu {
             self.pending_quit_to_menu = false;
             return Some(StateTransition::ToMenu);
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
        
        // Gentrification score
        draw_text_ex(
            &format!("Gentrification Score: {} | Affordable Units: {}", 
                self.gentrification.gentrification_score,
                self.gentrification.affordable_units),
            20.0,
            55.0,
            TextParams {
                font_size: 12,
                color: colors::TEXT_DIM,
                ..Default::default()
            },
        );
        
        // Navigation hint
        let nav_hint = match self.view_mode {
            ViewMode::Building => "[Tab] City Map | [M] Mail",
            ViewMode::CityMap => "[Tab] Building View | [M] Mail",
            ViewMode::Market => "[Tab] City Map | [M] Mail",
            ViewMode::Mail => "[Tab] Return | [Esc] Return",
            ViewMode::CareerSummary => "",
        };

        draw_text_ex(
            nav_hint,
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
}