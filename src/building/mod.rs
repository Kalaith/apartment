mod apartment;
mod building;
pub mod upgrades;
pub mod ownership;

pub use apartment::{Apartment, DesignType, ApartmentSize, NoiseLevel};
pub use building::{Building, MarketingType};
pub use upgrades::{UpgradeAction, apply_upgrade};

