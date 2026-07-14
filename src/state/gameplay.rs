use super::StateTransition;
use crate::assets::AssetManager;
use crate::building::Building;
use crate::data::config::GameConfig;
use crate::economy::{FinancialLedger, PlayerFunds};
use crate::simulation::{ActiveWorldEvent, EventLog, GameOutcome, TickResult};
use crate::tenant::{Tenant, TenantApplication};
use crate::ui::layout::HEADER_HEIGHT;
use crate::ui::{colors, FloatingText, Selection, Tween, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;
use std::collections::HashMap;

// Phase 3 imports
use crate::city::City;
use crate::consequences::{ComplianceSystem, GentrificationTracker, TenantNetwork};
use crate::narrative::{
    load_events_config, load_relationship_config, Mailbox, MissionManager, NarrativeEventSystem,
    NotificationManager, RelationshipEventsConfig, TenantEventsConfig, TenantStory,
    TutorialManager,
};

use serde::{Deserialize, Serialize};

/// View mode for the gameplay screen
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ViewMode {
    #[default]
    Building, // Current single-building view
    CityMap,       // City overview with all neighborhoods
    Market,        // Property acquisition screen
    Mail,          // Mailbox view
    CareerSummary, // Phase 5: Endgame result
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
    #[serde(default)]
    pub active_world_events: Vec<ActiveWorldEvent>,

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
    #[serde(default)]
    pub active_tax_breaks: Vec<crate::narrative::ActiveTaxBreak>,

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

    /// Latches true once the building has ever housed a tenant. The "all tenants
    /// left" loss condition keys off this so it can't fire on a building that was
    /// simply never occupied yet.
    #[serde(default)]
    pub has_ever_had_tenant: bool,

    /// True while a tenant council is organized. Latches when one forms (so its
    /// collective action applies once) and clears when conditions improve, so a
    /// council can re-form if the landlord backslides.
    #[serde(default)]
    pub council_formed: bool,

    /// The run's RNG seed, recorded so a run can be reproduced (bug reports,
    /// daily challenges) and re-applied on load so reloading doesn't reroll
    /// outcomes.
    #[serde(default)]
    pub seed: u64,
}

/// Pick a fresh run seed from wall-clock time. Uses macroquad's date source so
/// it works on both native and wasm (unlike `std::time`, which panics on wasm).
fn generate_run_seed() -> u64 {
    let now = macroquad::miniquad::date::now();
    ((now * 1_000_000.0) as u64) ^ 0x9E37_79B9_7F4A_7C15
}

impl GameplayState {
    /// Create a new game using the first configured building template.
    #[cfg(test)]
    pub fn new() -> Self {
        let config = crate::data::config::load_config();
        let template = crate::data::templates::load_templates()
            .and_then(|templates| templates.templates.into_iter().next())
            .unwrap_or_else(default_starter_template);

        Self::new_with_template(config, template)
    }

    /// Create a new game with a specific building template, choosing a fresh
    /// run seed from wall-clock entropy. Every game therefore differs (the RNG
    /// was previously never seeded, so all playthroughs were identical), and the
    /// chosen seed is recorded for reproducibility / bug reports.
    pub fn new_with_template(
        config: GameConfig,
        template: crate::data::templates::BuildingTemplate,
    ) -> Self {
        Self::new_with_template_seed(config, template, generate_run_seed())
    }

    /// Create a new game with a specific building template and an explicit run
    /// seed. Two games created from the same (config, template, seed) produce
    /// the same randomness — the basis for reproducible runs and daily
    /// challenges.
    pub fn new_with_template_seed(
        mut config: GameConfig,
        template: crate::data::templates::BuildingTemplate,
        seed: u64,
    ) -> Self {
        use crate::building::Building;

        // Seed the shared RNG before any generation so the run is reproducible
        // from `seed`.
        macroquad_toolkit::rng::srand(seed);

        // Apply the tier's rule modifiers (fines, inspections, problem tenants,
        // overhead) and derive its starting funds — this is what makes the three
        // property tiers genuinely different games, not just different sizes.
        let starting_funds = config.apply_difficulty(&template.difficulty);

        // Create building from template
        let building = Building::from_template(&template);
        let building_id = template.id.clone();

        // Place the building in its campaign neighborhood (falls back to a bare
        // slot if that neighborhood is full/missing).
        let mut city = City::new("Metropolis");
        let neighborhood_id = template.neighborhood_id;
        let starter_building_index = city
            .add_building(building.clone(), neighborhood_id)
            .unwrap_or_else(|_| {
                let index = city.buildings.len() as u32;
                city.buildings.push(building.clone());
                city.total_buildings_managed += 1;
                index
            });
        city.active_building_index = starter_building_index as usize;

        // Historic-quarter buildings carry preservation regulations.
        let is_historic = city
            .neighborhoods
            .iter()
            .find(|n| n.id == neighborhood_id)
            .map(|n| n.is_historic())
            .unwrap_or(false);

        // Initialize compliance
        let mut compliance = ComplianceSystem::new();
        compliance.init_building_regulations(starter_building_index, is_historic);

        let mut state = Self {
            city,
            building,
            config,
            tenants: Vec::new(),
            applications: Vec::new(),
            next_tenant_id: 1,
            funds: PlayerFunds::new(starting_funds),
            ledger: FinancialLedger::default(),
            event_log: EventLog::new(),
            current_tick: 0,
            game_outcome: None,
            last_tick_result: None,
            active_world_events: Vec::new(),

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
            active_tax_breaks: Vec::new(),
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
            has_ever_had_tenant: false,
            council_formed: false,
            seed,
        };

        // Handle initial tenant if present in template
        if let Some(data) = &template.initial_tenant {
            if let Some(archetype) = crate::tenant::TenantArchetype::from_id(&data.archetype) {
                if let Some(apt) = state
                    .building
                    .apartments
                    .iter_mut()
                    .find(|a| a.unit_number == data.apartment_unit)
                {
                    let tenant_id = state.next_tenant_id;
                    state.next_tenant_id += 1;

                    let mut tenant = Tenant::new(tenant_id, &data.name, archetype);
                    tenant.move_into(apt.id);
                    apt.move_in(tenant_id);

                    let story = TenantStory::generate(tenant_id, &tenant.archetype);
                    state.tenant_stories.insert(tenant_id, story);
                    state.tenants.push(tenant);

                    if let Some(city_building) = state.city.active_building_mut() {
                        if let Some(city_apt) =
                            city_building.apartments.iter_mut().find(|a| a.id == apt.id)
                        {
                            city_apt.move_in(tenant_id);
                        }
                    }
                }
            }
        }

        // Generate initial applications (neutral reputation at game start).
        state.applications = crate::tenant::generate_applications(
            &state.building,
            &[],
            0,
            &mut state.next_tenant_id,
            1.0,
            &state.config,
        );

        state.missions.generate_available_missions(0);

        state
    }

    /// Restore fields that are intentionally skipped from save data.
    pub fn post_load(&mut self) {
        self.config = crate::data::config::load_config();
        // config isn't serialized, so re-apply the building's difficulty
        // modifiers that were baked in at new-game time.
        if let Some(templates) = crate::data::templates::load_templates() {
            if let Some(template) = templates
                .templates
                .iter()
                .find(|t| t.id == self.current_building_id)
            {
                self.config.apply_difficulty(&template.difficulty);
            }
        }
        // Re-seed the shared RNG from the saved run seed so reloading a save
        // doesn't let the player reroll future random outcomes.
        macroquad_toolkit::rng::srand(self.seed);
        self.tenant_events_config = load_events_config();
        self.relationship_events_config = load_relationship_config();
        self.view_mode = ViewMode::Building;
        self.selection = Selection::None;
        self.pending_actions.clear();
        self.floating_texts.clear();
        self.panel_tween = Tween::new(0.0);
        self.panel_scroll_offset = 0.0;
        self.show_pause_menu = false;
        self.pending_quit_to_menu = false;
        self.active_world_events
            .retain(|event| event.remaining_ticks > 0);

        self.ensure_city_integrity();
        self.sync_building();
        self.ensure_compliance_for_buildings();
        self.ensure_tenant_stories();

        if self.current_building_id.is_empty() {
            self.current_building_id = crate::data::templates::load_templates()
                .and_then(|templates| templates.templates.into_iter().next())
                .map(|template| template.id)
                .unwrap_or_else(|| "mvp_default".to_string());
        }
    }

    fn ensure_city_integrity(&mut self) {
        if self.city.buildings.is_empty() {
            self.city.buildings.push(self.building.clone());
            self.city.active_building_index = 0;
        }

        if self.city.active_building_index >= self.city.buildings.len() {
            self.city.active_building_index = 0;
        }

        for building_id in 0..self.city.buildings.len() as u32 {
            let already_linked = self
                .city
                .neighborhoods
                .iter()
                .any(|neighborhood| neighborhood.building_ids.contains(&building_id));

            if already_linked {
                continue;
            }

            if let Some(neighborhood) = self
                .city
                .neighborhoods
                .iter_mut()
                .find(|neighborhood| neighborhood.can_add_building())
            {
                neighborhood.add_building(building_id);
            }
        }

        self.city.total_buildings_managed = self
            .city
            .total_buildings_managed
            .max(self.city.buildings.len() as u32);
    }

    fn ensure_compliance_for_buildings(&mut self) {
        let missing: Vec<(u32, bool)> = (0..self.city.buildings.len() as u32)
            .filter(|building_id| {
                !self
                    .compliance
                    .building_regulations
                    .contains_key(building_id)
            })
            .map(|building_id| {
                let is_historic = self
                    .city
                    .neighborhood_for_building(building_id as usize)
                    .is_some_and(|neighborhood| {
                        matches!(
                            neighborhood.neighborhood_type,
                            crate::city::NeighborhoodType::Historic
                        )
                    });
                (building_id, is_historic)
            })
            .collect();

        for (building_id, is_historic) in missing {
            self.compliance
                .init_building_regulations(building_id, is_historic);
        }
    }

    fn ensure_tenant_stories(&mut self) {
        for tenant in &self.tenants {
            self.tenant_stories
                .entry(tenant.id)
                .or_insert_with(|| TenantStory::generate(tenant.id, &tenant.archetype));
        }
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

    /// Unlock a specific building (by its template `unlock_order`) in the
    /// persistent player progress — used by `MissionReward::UnlockBuilding`.
    pub(super) fn unlock_building_by_order(&self, unlock_order: u32) {
        use crate::data::templates::load_templates;
        use crate::save::{load_player_progress, save_player_progress};

        let mut progress = load_player_progress();
        if let Some(templates) = load_templates() {
            if let Some(template) = templates
                .templates
                .iter()
                .find(|t| t.unlock_order == unlock_order)
            {
                progress.unlock_building(&template.id);
            }
        }
        let _ = save_player_progress(&progress);
    }

    /// Unlock the next building after completing the current one
    pub fn unlock_next_building(&self) {
        use crate::data::templates::load_templates;
        use crate::save::{load_player_progress, save_player_progress};

        let mut progress = load_player_progress();

        // Mark current building as completed
        progress.mark_completed(&self.current_building_id);

        // Find the next building to unlock based on unlock_order
        if let Some(templates) = load_templates() {
            // Find current building's unlock_order
            let current_order = templates
                .templates
                .iter()
                .find(|t| t.id == self.current_building_id)
                .map(|t| t.unlock_order)
                .unwrap_or(0);

            // Find the next building in sequence
            if let Some(next_template) = templates
                .templates
                .iter()
                .find(|t| t.unlock_order == current_order + 1)
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

        // Update tutorial
        self.update_tutorial();

        // Tutorial/notification toasts handle their own dismissal in draw().

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
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            HEADER_HEIGHT,
            colors::PANEL_HEADER,
        );

        // Title
        draw_ui_text_ex(
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
        draw_ui_text_ex(
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
        draw_ui_text_ex(
            &format!(
                "{} Buildings | Month {}",
                self.city.buildings.len(),
                self.current_tick
            ),
            screen_width() - 400.0,
            35.0,
            TextParams {
                font_size: 16,
                color: colors::TEXT_DIM,
                ..Default::default()
            },
        );

        // Gentrification score
        draw_ui_text_ex(
            &format!(
                "Gentrification Score: {} | Affordable Units: {}",
                self.gentrification.gentrification_score, self.gentrification.affordable_units
            ),
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

        draw_ui_text_ex(
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

#[cfg(test)]
fn default_starter_template() -> crate::data::templates::BuildingTemplate {
    use crate::data::templates::{ApartmentTemplate, BuildingTemplate};

    BuildingTemplate {
        id: "starter".to_string(),
        name: "Starter Building".to_string(),
        unlock_order: 0,
        difficulty: "easy".to_string(),
        neighborhood_id: 1,
        description: "A small starter property.".to_string(),
        floors: 2,
        units_per_floor: 2,
        hallway_condition: 60,
        apartments: vec![
            ApartmentTemplate {
                unit_number: "1A".to_string(),
                floor: 1,
                size_str: "small".to_string(),
                base_noise_str: "high".to_string(),
                initial_condition: 55,
                initial_design: "bare".to_string(),
                initial_rent: 600,
            },
            ApartmentTemplate {
                unit_number: "1B".to_string(),
                floor: 1,
                size_str: "medium".to_string(),
                base_noise_str: "low".to_string(),
                initial_condition: 60,
                initial_design: "bare".to_string(),
                initial_rent: 800,
            },
            ApartmentTemplate {
                unit_number: "2A".to_string(),
                floor: 2,
                size_str: "small".to_string(),
                base_noise_str: "low".to_string(),
                initial_condition: 65,
                initial_design: "practical".to_string(),
                initial_rent: 650,
            },
            ApartmentTemplate {
                unit_number: "2B".to_string(),
                floor: 2,
                size_str: "medium".to_string(),
                base_noise_str: "low".to_string(),
                initial_condition: 65,
                initial_design: "practical".to_string(),
                initial_rent: 850,
            },
        ],
        initial_tenant: None,
    }
}
