
use serde::{Deserialize, Serialize};

/// Type of relationship between tenants
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    /// Friendly neighbors who help each other
    Friendly,
    /// Neutral - no strong feelings either way
    Neutral,
    /// Conflict - noise complaints, disputes
    Hostile,
    /// Romantic relationship (may combine units)
    Romantic,
    /// Family connection
    Family,
}

impl RelationshipType {}

/// A relationship between two tenants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantRelationship {
    pub tenant_a_id: u32,
    pub tenant_b_id: u32,
    pub relationship_type: RelationshipType,
    /// How strong the relationship is (0-100)
    pub strength: i32,
    /// Months this relationship has existed
    pub duration_months: u32,
    /// Recent interactions that affected the relationship
    pub recent_events: Vec<String>,
}

impl TenantRelationship {
    pub fn new(tenant_a: u32, tenant_b: u32, initial_type: RelationshipType) -> Self {
        Self {
            tenant_a_id: tenant_a,
            tenant_b_id: tenant_b,
            relationship_type: initial_type,
            strength: 50,
            duration_months: 0,
            recent_events: Vec::new(),
        }
    }

    /// Apply monthly relationship dynamics
    pub fn tick(&mut self) {
        self.duration_months += 1;
        
        // Long-term relationships tend to strengthen
        if self.duration_months > 6 && !matches!(self.relationship_type, RelationshipType::Hostile) {
            self.strength = (self.strength + 1).min(100);
        }
        
        // Hostile relationships can cool down over time
        if matches!(self.relationship_type, RelationshipType::Hostile) && macroquad::rand::gen_range(0, 100) < 5 {
            self.strength = (self.strength - 5).max(0);
            if self.strength < 20 {
                self.relationship_type = RelationshipType::Neutral;
                self.recent_events.push("Conflict cooled down".to_string());
            }
        }

        // Clean up old events
        while self.recent_events.len() > 5 {
            self.recent_events.remove(0);
        }
    }

    /// Can these tenants potentially form this relationship?
    pub fn can_form(tenant_a: &crate::tenant::Tenant, tenant_b: &crate::tenant::Tenant) -> bool {
        // Different apartments
        tenant_a.apartment_id != tenant_b.apartment_id
    }
}

/// Manages all tenant relationships in a building
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantNetwork {
    pub relationships: Vec<TenantRelationship>,
    /// Track tenant history for displacement detection
    pub long_term_tenants: Vec<LongTermTenantRecord>,
}

/// Record of a long-term tenant's history
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LongTermTenantRecord {
    pub tenant_id: u32,
    pub tenant_name: String,
    pub archetype: crate::tenant::TenantArchetype,
    pub months_at_move_in: u32,
    pub original_rent: i32,
    pub current_rent: i32,
    pub is_displaced: bool,
    pub displacement_reason: Option<String>,
}

impl TenantNetwork {
    pub fn new() -> Self {
        Self {
            relationships: Vec::new(),
            long_term_tenants: Vec::new(),
        }
    }

    /// Get relationship between two specific tenants
    fn relationship_between(&self, tenant_a: u32, tenant_b: u32) -> Option<&TenantRelationship> {
        self.relationships.iter().find(|r| 
            (r.tenant_a_id == tenant_a && r.tenant_b_id == tenant_b) ||
            (r.tenant_a_id == tenant_b && r.tenant_b_id == tenant_a)
        )
    }

    /// Create a new relationship
    fn add_relationship(&mut self, tenant_a: u32, tenant_b: u32, rel_type: RelationshipType) {
        if self.relationship_between(tenant_a, tenant_b).is_none() {
            self.relationships.push(TenantRelationship::new(tenant_a, tenant_b, rel_type));
        }
    }

    /// Process monthly relationship dynamics
    pub fn tick(&mut self, tenants: &[crate::tenant::Tenant], building: &crate::building::Building) {
        // Update existing relationships
        for relationship in &mut self.relationships {
            relationship.tick();
        }

        // Chance to form new relationships between neighbors
        for tenant_a in tenants {
            for tenant_b in tenants {
                if tenant_a.id >= tenant_b.id {
                    continue; // Avoid duplicates
                }
                
                // Check if they're already related
                if self.relationship_between(tenant_a.id, tenant_b.id).is_some() {
                    continue;
                }

                // Check if they can form a relationship
                if !TenantRelationship::can_form(tenant_a, tenant_b) {
                    continue;
                }

                // 5% chance per month for new relationship
                if macroquad::rand::gen_range(0, 100) < 5 {
                    let rel_type = self.determine_initial_relationship(tenant_a, tenant_b, building);
                    self.add_relationship(tenant_a.id, tenant_b.id, rel_type);
                }
            }
        }
    }

    /// Determine what kind of relationship forms between two tenants
    fn determine_initial_relationship(
        &self, 
        tenant_a: &crate::tenant::Tenant, 
        tenant_b: &crate::tenant::Tenant,
        building: &crate::building::Building
    ) -> RelationshipType {
        use crate::tenant::TenantArchetype;

        // Get their apartments
        let apt_a = tenant_a.apartment_id.and_then(|id| building.get_apartment(id));
        let apt_b = tenant_b.apartment_id.and_then(|id| building.get_apartment(id));

        // Noise conflicts: Artists/Students in noisy units + Professionals/Elderly nearby
        let noisy_type_a = matches!(tenant_a.archetype, TenantArchetype::Artist | TenantArchetype::Student);
        let quiet_type_b = matches!(tenant_b.archetype, TenantArchetype::Professional | TenantArchetype::Elderly | TenantArchetype::Family);
        
        if (noisy_type_a && quiet_type_b) || (!noisy_type_a && !quiet_type_b && macroquad::rand::gen_range(0, 100) < 20) {
            // Check if apartments are adjacent (same floor or floor ±1)
            if let (Some(a), Some(b)) = (apt_a, apt_b) {
                if (a.floor as i32 - b.floor as i32).abs() <= 1 {
                    if macroquad::rand::gen_range(0, 100) < 30 {
                        return RelationshipType::Hostile;
                    }
                }
            }
        }

        // Same archetype tends to be friendly
        if tenant_a.archetype == tenant_b.archetype {
            if macroquad::rand::gen_range(0, 100) < 60 {
                return RelationshipType::Friendly;
            }
        }

        // Families tend to connect
        if matches!(tenant_a.archetype, TenantArchetype::Family) && 
           matches!(tenant_b.archetype, TenantArchetype::Family) {
            return RelationshipType::Friendly;
        }

        // Default to neutral
        RelationshipType::Neutral
    }
}

impl Default for TenantNetwork {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relationship_modifiers() {
        assert!(RelationshipType::Friendly.happiness_modifier() > 0);
        assert!(RelationshipType::Hostile.happiness_modifier() < 0);
    }

    #[test]
    fn test_network_basics() {
        let mut network = TenantNetwork::new();
        network.add_relationship(1, 2, RelationshipType::Friendly);
        
        assert!(network.relationship_between(1, 2).is_some());
        assert!(network.relationship_between(2, 1).is_some());
        assert!(network.relationship_between(1, 3).is_none());
    }
}
