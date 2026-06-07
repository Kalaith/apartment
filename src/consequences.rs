//! # Consequences Module
//!
//! Handles the long-term effects of player actions:
//! - `Relationships`: Tenant-tenant and tenant-landlord social networks.
//! - `Regulations`: City ordinances and compliance checks.
//! - `Gentrification`: Tracking neighborhood change over time.

mod gentrification;
mod regulations;
mod relationships;

pub use gentrification::GentrificationTracker;
pub use regulations::ComplianceSystem;
pub use relationships::{RelationshipType, TenantNetwork};
