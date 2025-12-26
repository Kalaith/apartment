//! # Tenant Module
//! 
//! Everything related to the people living in the building:
//! - `Tenant`: Individual stats, name, and state.
//! - `Archetypes`: Defined behaviors and preferences (e.g., Student, Retiree).
//! - `Happiness`: Calculations for tenant satisfaction.
//! - `Applications`: New potential tenants and vetting.

mod tenant;
mod archetype;
pub mod happiness;
pub mod matching;
mod application;
pub mod vetting;

pub use tenant::Tenant;
pub use archetype::{TenantArchetype, ArchetypePreferences};
pub use happiness::calculate_happiness;
// pub use matching::MatchResult;
pub use application::{TenantApplication, generate_applications, process_departures};
