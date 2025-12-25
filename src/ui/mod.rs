mod common;
mod building_view;
mod apartment_panel;
mod application_panel;
mod notifications;
mod header;
mod tenant_list;
pub mod visuals; // Make public so we can use FloatingText
pub mod city_view; // Phase 3 city map

pub use common::*;
pub use building_view::draw_building_view;
pub use apartment_panel::{draw_apartment_panel, draw_hallway_panel};

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
    Applications,        // Show all pending applications
    Hallway,             // Hallway details
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
    SelectApplications,
    SelectHallway,
    ClearSelection,
    
    // Generic Upgrade Action
    UpgradeAction(UpgradeAction),

    SetRent { apartment_id: u32, new_rent: i32 },
    
    // Tenant actions
    AcceptApplication { application_index: usize },
    RejectApplication { application_index: usize },
    
    // Game flow
    EndTurn,
    ReturnToMenu,
    
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
}
