use crate::building::{Apartment, Building, DesignType, NoiseLevel};
use super::{Tenant, ArchetypePreferences};

/// All factors that influence happiness
#[derive(Clone, Debug)]
pub struct HappinessFactors {
    pub base_happiness: i32,
    pub rent_factor: i32,       // Negative if too expensive
    pub condition_factor: i32,  // Based on apartment condition
    pub noise_factor: i32,      // Negative if too noisy
    pub design_factor: i32,     // Based on design preference
    pub hallway_factor: i32,    // Building shared space condition
    pub tenure_bonus: i32,      // Small bonus for long-term residents
}

impl HappinessFactors {
    pub fn total(&self) -> i32 {
        (self.base_happiness 
            + self.rent_factor 
            + self.condition_factor 
            + self.noise_factor 
            + self.design_factor
            + self.hallway_factor
            + self.tenure_bonus)
            .clamp(0, 100)
    }
}

/// Calculate happiness factors for a tenant in their apartment
pub fn calculate_happiness(
    tenant: &Tenant, 
    apartment: &Apartment,
    building: &Building,
) -> HappinessFactors {
    let prefs = tenant.archetype.preferences();
    
    HappinessFactors {
        base_happiness: 50,
        rent_factor: calculate_rent_factor(apartment.rent_price, &prefs),
        condition_factor: calculate_condition_factor(apartment.condition, &prefs),
        noise_factor: calculate_noise_factor(&apartment.effective_noise(), tenant.noise_tolerance, &prefs),
        design_factor: calculate_design_factor(&apartment.design, &prefs),
        hallway_factor: calculate_hallway_factor(building.hallway_condition),
        tenure_bonus: calculate_tenure_bonus(tenant.months_residing),
    }
}

fn calculate_rent_factor(rent: i32, prefs: &ArchetypePreferences) -> i32 {
    let diff = prefs.ideal_rent_max - rent;
    let sensitivity = prefs.rent_sensitivity;
    
    if diff >= 0 {
        // Under budget - small bonus
        ((diff as f32 * 0.02 * sensitivity) as i32).min(15)
    } else {
        // Over budget - penalty
        ((diff as f32 * 0.05 * sensitivity) as i32).max(-30)
    }
}

fn calculate_condition_factor(condition: i32, prefs: &ArchetypePreferences) -> i32 {
    let sensitivity = prefs.condition_sensitivity;
    let min_acceptable = prefs.min_acceptable_condition;
    
    if condition >= min_acceptable {
        // Good condition - bonus based on how much above minimum
        let excess = condition - min_acceptable;
        ((excess as f32 * 0.3 * sensitivity) as i32).min(20)
    } else {
        // Below minimum - significant penalty
        let deficit = min_acceptable - condition;
        -((deficit as f32 * 0.5 * sensitivity) as i32).min(40)
    }
}

fn calculate_noise_factor(noise: &NoiseLevel, tolerance: i32, prefs: &ArchetypePreferences) -> i32 {
    let sensitivity = prefs.noise_sensitivity;
    
    match noise {
        NoiseLevel::Low => {
            // Quiet - small bonus for those who prefer it
            if prefs.prefers_quiet {
                (10.0 * sensitivity) as i32
            } else {
                0
            }
        }
        NoiseLevel::High => {
            // Noisy - penalty based on sensitivity and tolerance
            let base_penalty = -25;
            let tolerance_mod = (tolerance as f32 * 0.3) as i32;
            ((base_penalty + tolerance_mod) as f32 * sensitivity) as i32
        }
    }
}

fn calculate_design_factor(design: &DesignType, prefs: &ArchetypePreferences) -> i32 {
    let sensitivity = prefs.design_sensitivity;
    let mut factor = 0;
    
    // Check if this is their preferred design
    if let Some(ref preferred) = prefs.preferred_design {
        if design == preferred {
            factor += 20;
        }
    }
    
    // Check if this is their hated design
    if let Some(ref hated) = prefs.hates_design {
        if design == hated {
            factor -= 25;
        }
    }
    
    // General design quality bonus
    factor += match design {
        DesignType::Bare => -5,
        DesignType::Practical => 5,
        DesignType::Cozy => 10,
    };
    
    (factor as f32 * sensitivity) as i32
}

fn calculate_hallway_factor(hallway_condition: i32) -> i32 {
    // Shared space affects everyone equally but mildly
    ((hallway_condition - 50) as f32 * 0.1) as i32  // -5 to +5
}

fn calculate_tenure_bonus(months: u32) -> i32 {
    // Long-term residents get a small stability bonus
    (months as i32).min(12)  // Max +12 after a year
}

/// Check if apartment meets minimum requirements for tenant
pub fn apartment_meets_minimum(tenant: &Tenant, apartment: &Apartment) -> bool {
    let prefs = tenant.archetype.preferences();
    
    // Check condition minimum
    if apartment.condition < prefs.min_acceptable_condition {
        return false;
    }
    
    // Check rent tolerance
    if apartment.rent_price > tenant.rent_tolerance {
        return false;
    }
    
    // Check design compatibility (artists won't accept bare)
    if let Some(ref hated) = prefs.hates_design {
        if &apartment.design == hated {
            return false;
        }
    }
    
    // Check noise for noise-sensitive tenants
    if prefs.prefers_quiet {
        if matches!(apartment.effective_noise(), NoiseLevel::High) 
           && tenant.noise_tolerance < 40 
        {
            return false;
        }
    }
    
    true
}

/// Calculate happiness modifier from tenant relationships
pub fn calculate_relationship_happiness(
    tenant_id: u32,
    network: &crate::consequences::TenantNetwork,
) -> i32 {
    let mut bonus = 0;
    
    for relationship in &network.relationships {
        if relationship.tenant_a_id == tenant_id || relationship.tenant_b_id == tenant_id {
            bonus += relationship.relationship_type.happiness_modifier();
        }
    }
    
    // Cap the relationship bonus
    bonus.clamp(-20, 20)
}
