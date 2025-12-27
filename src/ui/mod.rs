//! # UI Module
//! 
//! Pure view layer using Macroquad immediate mode UI:
//! - Uses `UiAction` pattern to send intents back to the game logic.
//! - Contains sub-panels for different views (Building, Apartment, City).
//! - `Visuals`: Floating text and animations.
//! - Strictly separation of concerns: No game state mutation happens here.

mod common;
mod building_view;
mod apartment_panel;
mod application_panel;
mod notifications;
mod header;
pub mod visuals; // Make public so we can use FloatingText
pub mod city_view; // Phase 3 city map
pub mod ownership_panel; // Phase 3 ownership
pub mod event_modal; // Phase 4 event modal
pub mod career_summary; // Phase 5 career summary

pub use common::*;
pub use building_view::draw_building_view;
pub use apartment_panel::{draw_apartment_panel, draw_hallway_panel};
pub use ownership_panel::draw_ownership_panel;

pub use application_panel::draw_application_panel;
pub use notifications::draw_notifications;
pub use header::draw_header;
pub use visuals::{FloatingText, Tween};


use serde::{Deserialize, Serialize};

/// What's currently selected for the detail panel
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Selection {
    None,
    Apartment(u32),      // Apartment ID
    Tenant(u32),         // Tenant ID
    Applications(Option<u32>), // Show pending applications (Optionally filtered by apartment)
    Hallway,             // Hallway details
    Ownership,           // Ownership View
}

impl Default for Selection {
    fn default() -> Self {
        Self::None
    }
}

use crate::building::UpgradeAction;

/// UI action intents (returned to game logic)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum UiAction {
    SelectApartment(u32),
    SelectTenant(u32),
    SelectApplications(Option<u32>),
    SelectHallway,
    SelectOwnership,
    ClearSelection,
    
    // Generic Upgrade Action
    UpgradeAction(UpgradeAction),

    SetRent { apartment_id: u32, new_rent: i32 },
    
    // Tenant actions
    AcceptApplication { application_index: usize },
    RejectApplication { application_index: usize },
    
    // Game flow
    EndTurn,
    ReturnToMenu, // Used by Career Summary
    
    // Phase 3: City navigation
    OpenCityMap,
    CloseCityView,
    OpenMarket,
    CloseMarket,
    OpenMail,
    CloseMail,
    
    // Phase 3: Multi-building
    SwitchBuilding { index: usize },
    PurchaseBuilding { listing_id: u32 },
    
    // Phase 3: Tenant requests
    ApproveRequest { tenant_id: u32 },
    DenyRequest { tenant_id: u32 },
    
    // Phase 3: Ownership
    SellUnitAsCondo { apartment_id: u32 },
    BuybackCondo { apartment_id: u32 },
    VoteOnProposal { proposal_index: usize, vote_yes: bool },
    
    // Phase 4: Dialogue system
    ResolveDialogue { dialogue_id: u32, choice_index: usize },
    ResolveEventChoice { event_id: u32, choice_index: usize },
    
    // Phase 4: Tenant vetting
    CreditCheck { application_index: usize },
    BackgroundCheck { application_index: usize },
    
    // Leasing
    ListApartment { apartment_id: u32, preference: Option<crate::tenant::TenantArchetype> },
    UnlistApartment { apartment_id: u32 },
    AdjustRent { apartment_id: u32, amount: i32 },
}
