mod tenant;
mod archetype;
mod happiness;
mod matching;
mod application;

pub use tenant::Tenant;
pub use archetype::{TenantArchetype, ArchetypePreferences};
pub use happiness::{calculate_happiness, HappinessFactors};
pub use matching::{calculate_match_score, MatchResult};
pub use application::{TenantApplication, generate_applications, process_departures};
