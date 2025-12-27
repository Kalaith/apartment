
use serde::{Deserialize, Serialize};
use crate::data::config::RelationshipsConfig;

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

impl RelationshipType {
    pub fn happiness_modifier(&self, config: &RelationshipsConfig) -> i32 {
        let key = match self {
            RelationshipType::Friendly => "friendly",
            RelationshipType::Neutral => "neutral",
            RelationshipType::Hostile => "hostile",
            RelationshipType::Romantic => "romantic",
            RelationshipType::Family => "family",
        };
        *config.happiness_modifiers.get(key).unwrap_or(&0)
    }
}

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
    
    // Phase 4C: Landlord opinions
    pub landlord_opinion_a: i32,  // How tenant A views landlord (-100 to 100)
    pub landlord_opinion_b: i32,  // How tenant B views landlord
}

/// Dynamic tension between apartments (e.g., noise complaints)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SocialTension {
    pub apartment_a: u32,
    pub apartment_b: u32,
    pub tension_level: i32,  // 0-100
    pub cause: String,
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
            landlord_opinion_a: 0,
            landlord_opinion_b: 0,
        }
    }

    /// Apply monthly relationship dynamics
    pub fn tick(&mut self, config: &RelationshipsConfig) {
        self.duration_months += 1;
        
        // Long-term relationships tend to strengthen
        if self.duration_months > 6 && !matches!(self.relationship_type, RelationshipType::Hostile) {
            self.strength = (self.strength + 1).min(100);
        }
        
        // Hostile relationships can cool down over time
        if matches!(self.relationship_type, RelationshipType::Hostile) 
            && macroquad::rand::gen_range(0, 100) < config.hostile_cooldown_chance 
        {
            self.strength = (self.strength - config.hostile_strength_decay).max(0);
            if self.strength < config.hostile_transition_threshold {
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
    
    // Phase 4C: Social Tension
    pub tensions: Vec<SocialTension>,
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
            tensions: Vec::new(),
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
    pub fn tick(
        &mut self, 
        tenants: &[crate::tenant::Tenant], 
        building: &crate::building::Building,
        config: &RelationshipsConfig,
    ) {
        // Update existing relationships
        for relationship in &mut self.relationships {
            relationship.tick(config);
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

                // Chance per month for new relationship
                if macroquad::rand::gen_range(0, 100) < config.formation_chance {
                    let rel_type = self.determine_initial_relationship(tenant_a, tenant_b, building, config);
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
        building: &crate::building::Building,
        config: &RelationshipsConfig,
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
                    if macroquad::rand::gen_range(0, 100) < config.adjacent_hostile_chance {
                        return RelationshipType::Hostile;
                    }
                }
            }
        }

        // Same archetype tends to be friendly
        if tenant_a.archetype == tenant_b.archetype {
            if macroquad::rand::gen_range(0, 100) < config.same_archetype_friendly_chance {
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
    
    /// Calculate community cohesion bonus based on matching archetypes
    pub fn calculate_cohesion(&self, tenants: &[crate::tenant::Tenant], config: &crate::data::config::CohesionConfig) -> i32 {
        if tenants.is_empty() { return 0; }
        
        let mut archetype_counts = std::collections::HashMap::new();
        for tenant in tenants {
            *archetype_counts.entry(tenant.archetype.clone()).or_insert(0) += 1;
        }
        
        let mut bonus = 0;
        
        // Bonus for having significant groups of same archetype
        for (_, count) in archetype_counts {
            if count >= config.archetype_group_threshold {
                bonus += config.archetype_group_base_bonus + (count - config.archetype_group_threshold) * config.archetype_group_per_extra;
            }
        }
        
        // Bonus for friendly relationships
        let friendly_count = self.relationships.iter()
            .filter(|r| matches!(r.relationship_type, RelationshipType::Friendly | RelationshipType::Family))
            .count() as i32;
            
        bonus += friendly_count * config.friendly_relationship_bonus;
        
        // Penalty for tensions/hostility
        let hostile_count = self.relationships.iter()
            .filter(|r| matches!(r.relationship_type, RelationshipType::Hostile))
            .count() as i32;
            
        bonus -= hostile_count * config.hostile_relationship_penalty;
        bonus -= (self.tensions.len() as i32) * config.tension_penalty;
        
        bonus.clamp(config.cohesion_min, config.cohesion_max)
    }
    
    /// Check if tenants are unhappy enough to form a council
    pub fn should_form_council(&self, tenants: &[crate::tenant::Tenant], config: &crate::data::config::GentrificationConfig) -> bool {
        if tenants.len() < config.council_min_tenants { return false; }
        
        let unhappy_count = tenants.iter().filter(|t| t.is_unhappy()).count();
        let relative_unhappiness = unhappy_count as f32 / tenants.len() as f32;
        
        // Formation threshold from config
        relative_unhappiness >= config.council_formation_threshold
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
        let config = RelationshipsConfig::default();
        assert!(RelationshipType::Friendly.happiness_modifier(&config) > 0);
        assert!(RelationshipType::Hostile.happiness_modifier(&config) < 0);
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
