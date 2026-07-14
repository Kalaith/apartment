//! # Consequences Module
//!
//! Handles the long-term effects of player actions:
//! - `Relationships`: Tenant-tenant and tenant-landlord social networks.
//! - `Regulations`: City ordinances and compliance checks.
//! - `Gentrification`: Tracking neighborhood change over time.

mod gentrification;
mod regulations;
mod relationship_dilemma;
mod relationships;

pub use gentrification::GentrificationTracker;
pub use regulations::{ComplianceSystem, InspectionTrigger};
pub use relationships::{RelationshipType, TenantNetwork};
