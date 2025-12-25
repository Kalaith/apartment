mod apartment;
mod building;
mod upgrades;

pub use apartment::{Apartment, DesignType, ApartmentSize, NoiseLevel};
pub use building::Building;
pub use upgrades::{UpgradeAction, apply_upgrade};
