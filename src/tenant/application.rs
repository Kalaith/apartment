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
}

impl TenantApplication {
    pub fn new(tenant: Tenant, apartment_id: u32, match_result: MatchResult, tick: u32) -> Self {
        Self {
            tenant,
            apartment_id,
            match_result,
            tick_created: tick,
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
    let num_applications = (base_apps + appeal_bonus).min(vacant.len()).max(1);
    
    for _ in 0..num_applications {
        // Pick a random archetype (weighted)
        let archetype = pick_random_archetype();
        
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

fn pick_random_archetype() -> TenantArchetype {
    let roll = gen_range(0, 100);
    
    // Weighted distribution
    if roll < 35 {
        TenantArchetype::Student      // 35% - Budget option
    } else if roll < 60 {
        TenantArchetype::Professional // 25% - Standard
    } else if roll < 75 {
        TenantArchetype::Family       // 15% - Needs space
    } else if roll < 85 {
        TenantArchetype::Elderly      // 10% - Needs quiet
    } else {
        TenantArchetype::Artist       // 15% - Niche
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
