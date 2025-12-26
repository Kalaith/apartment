use macroquad::rand::gen_range;
use super::{Tenant, TenantArchetype, matching::MatchResult};
use crate::building::Building;
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

/// Generate new tenant applications based on building state
pub fn generate_applications(
    building: &Building,
    existing_applications: &[TenantApplication],
    current_tick: u32,
    next_tenant_id: &mut u32,
) -> Vec<TenantApplication> {
    let mut new_applications = Vec::new();
    
    let vacant = building.vacant_apartments();
    if vacant.is_empty() {
        return new_applications;
    }
    
    let building_appeal = building.building_appeal();
    
    // Number of applications based on building appeal and vacancies
    let base_apps = (vacant.len() as f32 * 0.5).ceil() as usize;
    let appeal_bonus = (building_appeal as f32 / 50.0) as usize;
    
    // Apply marketing strategy multipliers
    let marketing_multiplier = match building.marketing_strategy {
        crate::building::MarketingType::None => 1.0,
        crate::building::MarketingType::SocialMedia => 2.0, // High volume
        crate::building::MarketingType::LocalNewspaper => 1.5, // Moderate volume
        crate::building::MarketingType::PremiumAgency => 0.8, // Low volume, high quality
    };
    
    // Open house bonus (doubles volume)
    let open_house_multiplier = if building.open_house_remaining > 0 { 2.0 } else { 1.0 };
    
    let raw_num_applications = (base_apps + appeal_bonus) as f32 * marketing_multiplier * open_house_multiplier;
    let num_applications = (raw_num_applications as usize).min(vacant.len() * 2).max(1);
    
    for _ in 0..num_applications {
        // Pick a random archetype (weighted by marketing)
        let archetype = pick_random_archetype(&building.marketing_strategy);
        
        // Generate a tenant
        let tenant = Tenant::generate(*next_tenant_id, archetype);
        *next_tenant_id += 1;
        
        // Find an apartment they'd apply to
        let apartment_refs: Vec<&_> = vacant.iter().map(|a| *a).collect();
        
        if let Some((apt, match_result)) = super::matching::find_best_match(&tenant, &apartment_refs) {
            // Check if there's already an application for this apartment from this archetype
            let already_applied = existing_applications.iter().any(|app| {
                app.apartment_id == apt.id && app.tenant.archetype == tenant.archetype
            }) || new_applications.iter().any(|app: &TenantApplication| {
                app.apartment_id == apt.id && app.tenant.archetype == tenant.archetype
            });
            
            if !already_applied {
                new_applications.push(TenantApplication::new(
                    tenant,
                    apt.id,
                    match_result,
                    current_tick,
                ));
            }
        }
    }
    
    new_applications
}

fn pick_random_archetype(marketing: &crate::building::MarketingType) -> TenantArchetype {
    let roll = gen_range(0, 100);
    
    // Adjust weights based on marketing
    match marketing {
        crate::building::MarketingType::SocialMedia => {
            // Heavily favors Students and Artists
            if roll < 50 { TenantArchetype::Student }
            else if roll < 80 { TenantArchetype::Artist }
            else if roll < 90 { TenantArchetype::Professional }
            else { TenantArchetype::Family }
        },
        crate::building::MarketingType::LocalNewspaper => {
            // Favors Elderly and Families
            if roll < 15 { TenantArchetype::Student }
            else if roll < 30 { TenantArchetype::Professional }
            else if roll < 60 { TenantArchetype::Family }
            else if roll < 90 { TenantArchetype::Elderly }
            else { TenantArchetype::Artist }
        },
        crate::building::MarketingType::PremiumAgency => {
            // Heavily favors Professionals, filters out Students
            if roll < 5 { TenantArchetype::Student }
            else if roll < 65 { TenantArchetype::Professional }
            else if roll < 85 { TenantArchetype::Family }
            else if roll < 95 { TenantArchetype::Elderly }
            else { TenantArchetype::Artist }
        },
        crate::building::MarketingType::None => {
            // Default distribution
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
