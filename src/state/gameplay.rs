use macroquad::prelude::*;
use super::{StateTransition, ResultsState};
use crate::building::{Building, UpgradeAction};
use crate::tenant::{Tenant, TenantApplication};
use crate::economy::{PlayerFunds, FinancialLedger, process_upgrade};
use crate::simulation::{EventLog, GameOutcome, TickResult, advance_tick, GameEvent};
use crate::ui::{
    Selection, UiAction, 
    draw_header, draw_building_view, draw_apartment_panel, draw_hallway_panel,
    draw_application_panel, draw_notifications,
};

pub struct GameplayState {
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
    
    // UI state
    pub selection: Selection,
    pending_actions: Vec<UiAction>,
}

impl GameplayState {
    pub fn new() -> Self {
        Self {
            building: Building::default_mvp(),
            tenants: Vec::new(),
            applications: Vec::new(),
            next_tenant_id: 1,
            funds: PlayerFunds::default(),
            ledger: FinancialLedger::default(),
            event_log: EventLog::new(),
            current_tick: 0,
            game_outcome: None,
            last_tick_result: None,
            selection: Selection::None,
            pending_actions: Vec::new(),
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
            UiAction::RepairApartment { apartment_id, amount } => {
                let upgrade = UpgradeAction::RepairApartment { apartment_id, amount };
                if let Ok(cost) = process_upgrade(&upgrade, &mut self.building, &mut self.funds, self.current_tick) {
                    self.event_log.log(GameEvent::UpgradeCompleted {
                        description: format!("Repaired apartment +{}", amount),
                        cost,
                    }, self.current_tick);
                }
            }
            UiAction::UpgradeDesign { apartment_id } => {
                let upgrade = UpgradeAction::UpgradeDesign { apartment_id };
                if let Ok(cost) = process_upgrade(&upgrade, &mut self.building, &mut self.funds, self.current_tick) {
                    self.event_log.log(GameEvent::UpgradeCompleted {
                        description: "Upgraded design".to_string(),
                        cost,
                    }, self.current_tick);
                }
            }
            UiAction::AddSoundproofing { apartment_id } => {
                let upgrade = UpgradeAction::AddSoundproofing { apartment_id };
                if let Ok(cost) = process_upgrade(&upgrade, &mut self.building, &mut self.funds, self.current_tick) {
                    self.event_log.log(GameEvent::UpgradeCompleted {
                        description: "Added soundproofing".to_string(),
                        cost,
                    }, self.current_tick);
                }
            }
            UiAction::RepairHallway { amount } => {
                let upgrade = UpgradeAction::RepairHallway { amount };
                if let Ok(cost) = process_upgrade(&upgrade, &mut self.building, &mut self.funds, self.current_tick) {
                    self.event_log.log(GameEvent::UpgradeCompleted {
                        description: format!("Repaired hallway +{}", amount),
                        cost,
                    }, self.current_tick);
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
        }
    }

    pub fn update(&mut self) -> Option<StateTransition> {
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
        if is_key_pressed(KeyCode::Space) {
            self.end_turn();
        }
        
        // Handle Escape to clear selection
        if is_key_pressed(KeyCode::Escape) {
            self.selection = Selection::None;
        }
        
        // Process pending actions from UI
        let actions = std::mem::take(&mut self.pending_actions);
        for action in actions {
            self.process_action(action);
        }
        
        None
    }

    pub fn draw(&mut self) {
        // Header with End Month button
        if let Some(action) = draw_header(
            self.funds.balance,
            self.current_tick,
            &self.building.name,
            self.building.occupancy_count(),
            self.building.apartments.len(),
        ) {
            self.pending_actions.push(action);
        }
        
        // Building view (left side)
        if let Some(action) = draw_building_view(
            &self.building,
            &self.tenants,
            &self.selection,
        ) {
            self.pending_actions.push(action);
        }
        
        // Detail panel based on selection (right side)
        match &self.selection {
            Selection::Apartment(id) => {
                if let Some(apt) = self.building.get_apartment(*id) {
                    if let Some(action) = draw_apartment_panel(
                        apt,
                        &self.building,
                        &self.tenants,
                        self.funds.balance,
                    ) {
                        self.pending_actions.push(action);
                    }
                }
            }
            Selection::Hallway => {
                if let Some(action) = draw_hallway_panel(&self.building, self.funds.balance) {
                    self.pending_actions.push(action);
                }
            }
            Selection::Applications => {
                if let Some(action) = draw_application_panel(&self.applications, &self.building) {
                    self.pending_actions.push(action);
                }
            }
            Selection::Tenant(_id) => {
                // Could show tenant details here
            }
            Selection::None => {
                // Show applications by default if there are any
                if !self.applications.is_empty() {
                    if let Some(action) = draw_application_panel(&self.applications, &self.building) {
                        self.pending_actions.push(action);
                    }
                }
            }
        }
        
        // Notifications at bottom
        draw_notifications(&self.event_log, self.current_tick);
    }
}
