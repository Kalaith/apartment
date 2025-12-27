use macroquad::rand::gen_range;
use super::{Tenant, TenantArchetype, matching::MatchResult};
use crate::building::Building;
use crate::data::config::MatchingConfig;
use serde::{Deserialize, Serialize};

/// A tenant application for a specific apartment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantApplication {
    pub tenant: Tenant,
    pub apartment_id: u32,
    pub match_result: MatchResult,
    pub tick_created: u32,  // When this application was generated
    
    // Vetting state (hidden stats revealed after checks)
    pub revealed_reliability: bool,  // Credit check done?
    pub revealed_behavior: bool,     // Background check done?
}

impl TenantApplication {
    pub fn new(tenant: Tenant, apartment_id: u32, match_result: MatchResult, tick: u32) -> Self {
        Self {
            tenant,
            apartment_id,
            match_result,
            tick_created: tick,
            revealed_reliability: false,
            revealed_behavior: false,
        }
    }
    
    /// Applications expire after a few ticks
    pub fn is_expired(&self, current_tick: u32) -> bool {
        current_tick > self.tick_created + 3  // Expire after 3 months
    }
}

/// Generate new tenant applications for listed apartments
pub fn generate_applications(
    building: &Building,
    existing_applications: &[TenantApplication],
    current_tick: u32,
    next_tenant_id: &mut u32,
    config: &MatchingConfig,
) -> Vec<TenantApplication> {
    let mut new_applications = Vec::new();
    
    // 1. Identify listed vacancies
    let listed_apartments: Vec<&_> = building.vacant_apartments()
        .into_iter()
        .filter(|a| a.is_listed_for_lease)
        .collect();
        
    if listed_apartments.is_empty() {
        return new_applications;
    }
    
    let building_appeal = building.building_appeal();
    
    // Marketing multipliers (same as before)
    let marketing_multiplier = match building.marketing_strategy {
        crate::building::MarketingType::None => 1.0,
        crate::building::MarketingType::SocialMedia => 2.0,
        crate::building::MarketingType::LocalNewspaper => 1.5,
        crate::building::MarketingType::PremiumAgency => 0.8,
    };
    
    let open_house_multiplier = if building.open_house_remaining > 0 { 2.0 } else { 1.0 };
    
    // 2. Generate applications for EACH listed apartment
    for apt in listed_apartments {
        // Base probability per apartment
        let appeal_factor = (building_appeal as f32 / 50.0).max(0.5);
        let chance = 0.8 * appeal_factor * marketing_multiplier * open_house_multiplier; 
        
        // Debug print
        println!("Gen App Check: Apt {} Chance {:.2}", apt.unit_number, chance);
        
        // Random check to see if we generate an applicant this tick
        if gen_range(0.0, 1.0) < chance {
            // Pick archetype based on preference + marketing
            let archetype = pick_archetype_with_preference(
                &building.marketing_strategy, 
                apt.preferred_archetype.as_ref()
            );
            
            // Generate tenant
            let tenant = Tenant::generate(*next_tenant_id, archetype);
            *next_tenant_id += 1;
            
            // Check match
            let apt_slice = [apt];
            if let Some((_, match_result)) = super::matching::find_best_match(&tenant, &apt_slice, config) {
                // Check dupes
                let already_applied = existing_applications.iter().any(|app| {
                    app.apartment_id == apt.id && app.tenant.archetype == tenant.archetype
                }) || new_applications.iter().any(|app: &TenantApplication| {
                    app.apartment_id == apt.id && app.tenant.archetype == tenant.archetype
                });
                
                if !already_applied {
                    println!("SUCCESS: Added applicant for unit {} (Archetype: {:?})", apt.unit_number, tenant.archetype); // Debug print
                    new_applications.push(TenantApplication::new(
                        tenant,
                        apt.id,
                        match_result,
                        current_tick,
                    ));
                } else {
                    println!("SKIPPED: Duplicate application for unit {}", apt.unit_number);
                }
            } else {
                // Debug: Why did they reject?
                let prefs = tenant.archetype.preferences();
                if apt.rent_price > tenant.rent_tolerance {
                     println!("SKIPPED: Tenant {:?} rejected unit {} due to RENT (Rent: {}, Tolerance: {})", 
                        tenant.archetype, apt.unit_number, apt.rent_price, tenant.rent_tolerance);
                } else if apt.condition < prefs.min_acceptable_condition {
                     println!("SKIPPED: Tenant {:?} rejected unit {} due to CONDITION (Cond: {}, Min: {})",
                        tenant.archetype, apt.unit_number, apt.condition, prefs.min_acceptable_condition);
                } else {
                     println!("SKIPPED: Tenant {:?} rejected unit {} due to other factors (Noise/Design)", tenant.archetype, apt.unit_number);
                }
            }
        }
    }
    
    new_applications
}

fn pick_archetype_with_preference(
    marketing: &crate::building::MarketingType, 
    preference: Option<&TenantArchetype>
) -> TenantArchetype {
    // If preference exists, 80% chance to pick it
    if let Some(pref) = preference {
        if gen_range(0, 100) < 80 {
            return pref.clone();
        }
    }

    let roll = gen_range(0, 100);
    
    // Adjust weights based on marketing
    match marketing {
        crate::building::MarketingType::SocialMedia => {
            if roll < 50 { TenantArchetype::Student }
            else if roll < 80 { TenantArchetype::Artist }
            else if roll < 90 { TenantArchetype::Professional }
            else { TenantArchetype::Family }
        },
        crate::building::MarketingType::LocalNewspaper => {
            if roll < 15 { TenantArchetype::Student }
            else if roll < 30 { TenantArchetype::Professional }
            else if roll < 60 { TenantArchetype::Family }
            else if roll < 90 { TenantArchetype::Elderly }
            else { TenantArchetype::Artist }
        },
        crate::building::MarketingType::PremiumAgency => {
            if roll < 5 { TenantArchetype::Student }
            else if roll < 65 { TenantArchetype::Professional }
            else if roll < 85 { TenantArchetype::Family }
            else if roll < 95 { TenantArchetype::Elderly }
            else { TenantArchetype::Artist }
        },
        crate::building::MarketingType::None => {
            if roll < 35 { TenantArchetype::Student }
            else if roll < 60 { TenantArchetype::Professional }
            else if roll < 75 { TenantArchetype::Family }
            else if roll < 85 { TenantArchetype::Elderly }
            else { TenantArchetype::Artist }
        }
    }
}

/// Process tenant decisions to leave
pub fn process_departures(tenants: &mut Vec<Tenant>, building: &mut Building) -> Vec<String> {
    let mut notifications = Vec::new();
    let mut departing_ids = Vec::new();
    
    for tenant in tenants.iter_mut() {
        // Check for unhappy tenants (early warning)
        if tenant.is_unhappy() && !tenant.will_leave() {
            notifications.push(format!("{} is unhappy and may leave soon!", tenant.name));
        }
        
        if tenant.will_leave() {
            notifications.push(format!("{} has moved out!", tenant.name));
            departing_ids.push(tenant.id);
            
            // Clear apartment
            if let Some(apt_id) = tenant.apartment_id {
                if let Some(apt) = building.get_apartment_mut(apt_id) {
                    apt.move_out();
                }
            }
            
            // Clear tenant's apartment reference
            tenant.move_out();
        }
    }
    
    tenants.retain(|t| !departing_ids.contains(&t.id));
    notifications
}
