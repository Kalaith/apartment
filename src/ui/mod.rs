mod common;
mod building_view;
mod apartment_panel;
mod application_panel;
mod notifications;
mod header;
mod tenant_list;

pub use common::*;
pub use building_view::draw_building_view;
pub use apartment_panel::{draw_apartment_panel, draw_hallway_panel};
pub use tenant_list::draw_tenant_list;
pub use application_panel::draw_application_panel;
pub use notifications::{draw_notifications, draw_toast};
pub use header::draw_header;

/// What's currently selected for the detail panel
#[derive(Clone, Debug, PartialEq)]
pub enum Selection {
    None,
    Apartment(u32),      // Apartment ID
    Tenant(u32),         // Tenant ID
    Applications,        // Show all pending applications
    Hallway,             // Hallway details
}

/// UI action intents (returned to game logic)
#[derive(Clone, Debug)]
pub enum UiAction {
    SelectApartment(u32),
    SelectTenant(u32),
    SelectApplications,
    SelectHallway,
    ClearSelection,
    
    // Upgrade actions
    RepairApartment { apartment_id: u32, amount: i32 },
    UpgradeDesign { apartment_id: u32 },
    AddSoundproofing { apartment_id: u32 },
    RepairHallway { amount: i32 },
    SetRent { apartment_id: u32, new_rent: i32 },
    
    // Tenant actions
    AcceptApplication { application_index: usize },
    RejectApplication { application_index: usize },
    
    // Game flow
    EndTurn,
    ReturnToMenu,
}
