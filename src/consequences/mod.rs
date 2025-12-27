//! # Consequences Module
//! 
//! Handles the long-term effects of player actions:
//! - `Relationships`: Tenant-tenant and tenant-landlord social networks.
//! - `Regulations`: City ordinances and compliance checks.
//! - `Gentrification`: Tracking neighborhood change over time.

mod relationships;
mod regulations;
mod gentrification;

pub use relationships::{TenantNetwork, RelationshipType};
pub use regulations::ComplianceSystem;
pub use gentrification::GentrificationTracker;
