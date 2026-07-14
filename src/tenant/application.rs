use super::{matching::MatchResult, Tenant, TenantArchetype};
use crate::building::Building;
use crate::data::config::GameConfig;
use macroquad_toolkit::rng;
use serde::{Deserialize, Serialize};

/// A tenant application for a specific apartment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantApplication {
    pub tenant: Tenant,
    pub apartment_id: u32,
    pub match_result: MatchResult,
    pub tick_created: u32, // When this application was generated

    // Vetting state (hidden stats revealed after checks)
    pub revealed_reliability: bool, // Credit check done?
    pub revealed_behavior: bool,    // Background check done?
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

    pub fn is_expired_after(&self, current_tick: u32, expire_after_ticks: u32) -> bool {
        current_tick > self.tick_created + expire_after_ticks
    }
}

/// Generate new tenant applications for listed apartments
pub fn generate_applications(
    building: &Building,
    existing_applications: &[TenantApplication],
    current_tick: u32,
    next_tenant_id: &mut u32,
    reputation_multiplier: f32,
    config: &GameConfig,
) -> Vec<TenantApplication> {
    let mut new_applications = Vec::new();

    // 1. Identify listed vacancies
    let listed_apartments: Vec<&_> = building
        .vacant_apartments()
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

    let open_house_multiplier = if building.open_house_remaining > 0 {
        2.0
    } else {
        1.0
    };

    // 2. Generate applications for EACH listed apartment
    for apt in listed_apartments {
        // Base probability per apartment
        let appeal_divisor = config.applications.appeal_bonus_divisor.max(1) as f32;
        let appeal_factor = (building_appeal as f32 / appeal_divisor).max(0.5);
        let chance = config.applications.base_per_vacancy
            * appeal_factor
            * marketing_multiplier
            * open_house_multiplier
            * reputation_multiplier;

        // Random check to see if we generate an applicant this tick
        if rng::gen_range(0.0, 1.0) < chance {
            // Pick archetype based on preference + marketing
            let archetype = pick_archetype_with_preference(
                &building.marketing_strategy,
                apt.preferred_archetype.as_ref(),
            );

            // Generate tenant
            let tenant = Tenant::generate(*next_tenant_id, archetype);
            *next_tenant_id += 1;

            // Check match
            let apt_slice = [apt];
            if let Some((_, match_result)) =
                super::matching::find_best_match(&tenant, &apt_slice, &config.matching)
            {
                // Check dupes
                let already_applied =
                    existing_applications.iter().any(|app| {
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
    }

    new_applications
}

fn pick_archetype_with_preference(
    marketing: &crate::building::MarketingType,
    preference: Option<&TenantArchetype>,
) -> TenantArchetype {
    // If preference exists, 80% chance to pick it
    if let Some(pref) = preference {
        if rng::gen_range(0, 100) < 80 {
            return pref.clone();
        }
    }

    let registry = crate::data::archetypes::archetypes();
    let mut weighted_archetypes: Vec<(TenantArchetype, u32)> = registry
        .definitions
        .values()
        .filter_map(|definition| {
            TenantArchetype::from_id(&definition.id)
                .map(|archetype| (archetype, definition.spawn_weight.max(1)))
        })
        .collect();

    if weighted_archetypes.is_empty() {
        weighted_archetypes = vec![
            (TenantArchetype::Student, 35),
            (TenantArchetype::Professional, 25),
            (TenantArchetype::Family, 15),
            (TenantArchetype::Elderly, 10),
            (TenantArchetype::Artist, 15),
        ];
    }

    for (archetype, weight) in &mut weighted_archetypes {
        let multiplier = match marketing {
            crate::building::MarketingType::SocialMedia => match archetype {
                TenantArchetype::Student | TenantArchetype::Artist => 2,
                _ => 1,
            },
            crate::building::MarketingType::LocalNewspaper => match archetype {
                TenantArchetype::Family | TenantArchetype::Elderly => 2,
                _ => 1,
            },
            crate::building::MarketingType::PremiumAgency => match archetype {
                TenantArchetype::Professional => 3,
                TenantArchetype::Family => 2,
                _ => 1,
            },
            crate::building::MarketingType::None => 1,
        };
        *weight *= multiplier;
    }

    let total_weight: u32 = weighted_archetypes.iter().map(|(_, weight)| *weight).sum();
    let mut roll = rng::gen_range(0, total_weight.max(1));
    for (archetype, weight) in weighted_archetypes {
        if roll < weight {
            return archetype;
        }
        roll -= weight;
    }

    TenantArchetype::Student
}

/// Process tenant decisions to leave
pub fn process_departures(
    tenants: &mut Vec<Tenant>,
    building: &mut Building,
    config: &crate::data::config::HappinessConfig,
) -> Vec<String> {
    let mut notifications = Vec::new();
    let mut departing_ids = Vec::new();

    for tenant in tenants.iter_mut() {
        // Roll once — will_leave is probabilistic, so reuse the result rather
        // than re-rolling it for the early-warning check below.
        let leaving = tenant.will_leave(config.leave_threshold, config.leave_chance_percent);

        if tenant.is_unhappy(config.unhappy_threshold) && !leaving {
            notifications.push(format!("{} is unhappy and may leave soon!", tenant.name));
        }

        if leaving {
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
