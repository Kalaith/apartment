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
