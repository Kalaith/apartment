//! # City Module
//! 
//! Manages the broader context outside the player's building:
//! - `City`: The container for all neighborhoods and buildings.
//! - `Neighborhood`: Specific districts with unique modifiers and demographics.
//! - `Market`: The real estate market for buying new properties.

mod neighborhood;
mod city;
mod market;

pub use neighborhood::{Neighborhood, NeighborhoodType};
pub use city::City;
pub use market::{PropertyListing, PropertyMarket};
