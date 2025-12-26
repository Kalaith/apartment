//! # Building Module
//! 
//! Handles the physical structure of the game world:
//! - `Apartment`: Individual units, their condition, and properties.
//! - `Building`: The container for apartments and shared spaces (hallways).
//! - `Upgrades`: Systems for improving building and apartment quality.
//! - `Ownership`: Logic for selling units as condos.

mod apartment;
mod building;
pub mod upgrades;
pub mod ownership;

pub use apartment::{Apartment, DesignType, ApartmentSize, NoiseLevel};
pub use building::{Building, MarketingType};
pub use upgrades::{UpgradeAction, apply_upgrade};

