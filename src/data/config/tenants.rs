//! Tuning for everything tenant-facing: matching applicants to units, lease
//! terms, hidden risk, vetting, marketing reach, staff, and life events.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchingConfig {
    pub base_score: i32,
    pub desperate_penalty: i32,
    pub rent_great_threshold: i32,
    pub rent_great_bonus: i32,
    pub rent_fair_bonus: i32,
    pub rent_slight_penalty: i32,
    pub rent_unaffordable_penalty: i32,
    pub condition_excellent_threshold: i32,
    pub condition_excellent_bonus: i32,
    pub condition_good_threshold: i32,
    pub condition_good_bonus: i32,
    pub condition_poor_threshold: i32,
    pub condition_poor_penalty: i32,
    pub noise_quiet_bonus: i32,
    pub noise_loud_penalty: i32,
    pub design_preferred_bonus: i32,
    pub size_medium_bonus: i32,
    pub lease_defaults: LeaseDefaultsConfig,
    pub lease_acceptance: LeaseAcceptanceConfig,
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            base_score: 50,
            desperate_penalty: -40,
            rent_great_threshold: 200,
            rent_great_bonus: 15,
            rent_fair_bonus: 8,
            rent_slight_penalty: -5,
            rent_unaffordable_penalty: -20,
            condition_excellent_threshold: 80,
            condition_excellent_bonus: 15,
            condition_good_threshold: 60,
            condition_good_bonus: 8,
            condition_poor_threshold: 50,
            condition_poor_penalty: 10,
            noise_quiet_bonus: 12,
            noise_loud_penalty: 15,
            design_preferred_bonus: 18,
            size_medium_bonus: 5,
            lease_defaults: LeaseDefaultsConfig::default(),
            lease_acceptance: LeaseAcceptanceConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseDefaultsConfig {
    pub security_deposit_months: u32,
    pub lease_duration_months: u32,
    pub cleaning_fee: i32,
}

impl Default for LeaseDefaultsConfig {
    fn default() -> Self {
        Self {
            security_deposit_months: 1,
            lease_duration_months: 12,
            cleaning_fee: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseAcceptanceConfig {
    pub deposit_2_month_penalty: f32,
    pub deposit_3_month_penalty: f32,
    pub short_lease_bonus: f32,
    pub long_lease_penalty: f32,
    pub good_deal_bonus: f32,
    pub expensive_penalty: f32,
}

impl Default for LeaseAcceptanceConfig {
    fn default() -> Self {
        Self {
            deposit_2_month_penalty: 0.15,
            deposit_3_month_penalty: 0.35,
            short_lease_bonus: 0.1,
            long_lease_penalty: 0.15,
            good_deal_bonus: 0.1,
            expensive_penalty: 0.1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StaffEffectsConfig {
    /// Number of units the janitor fully maintains (offsets their monthly decay).
    pub janitor_units_maintained: usize,
    /// Happiness bonus applied to every tenant while security is employed.
    pub security_happiness_bonus: i32,
    /// Percent reduction to critical-failure odds while security is employed.
    pub security_failure_reduction_percent: i32,
    /// Happiness bonus applied to every tenant while a manager is employed.
    pub manager_happiness_bonus: i32,
    /// Whether the manager automatically approves pending tenant requests.
    pub manager_auto_approve_requests: bool,
}

impl Default for StaffEffectsConfig {
    fn default() -> Self {
        Self {
            janitor_units_maintained: 5,
            security_happiness_bonus: 6,
            security_failure_reduction_percent: 50,
            manager_happiness_bonus: 4,
            manager_auto_approve_requests: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantRiskConfig {
    /// Tenants with rent_reliability below this may skip rent even when content.
    pub unreliable_threshold: i32,
    /// Monthly chance (percent) an unreliable tenant skips rent.
    pub skip_rent_chance_percent: i32,
    /// Tenants with behavior_score below this may damage the property.
    pub low_behavior_threshold: i32,
    /// Monthly chance (percent) a low-behavior tenant causes damage.
    pub damage_chance_percent: i32,
    /// Condition points removed from the unit when damage occurs.
    pub damage_amount: i32,
    /// Hallway condition points removed when a disruptive tenant acts up.
    pub hallway_disturbance_amount: i32,
    /// Max rent-tolerance premium (percent) a maximally-risky applicant carries.
    /// Risky applicants are more desperate: they tolerate (and will pay) higher
    /// rent, which makes accepting them tempting despite the skipped-rent and
    /// property-damage risk. Scales to zero as an applicant approaches the
    /// unreliable threshold.
    #[serde(default = "default_risky_rent_premium_percent")]
    pub risky_rent_premium_percent: i32,
    /// Percent of applicants who are "problem tenants" regardless of archetype —
    /// their hidden reliability/behavior are forced into the risky range. Without
    /// this the applicant pool is almost all reliable archetypes, so screening
    /// has nothing to catch and tenant selection doesn't matter.
    #[serde(default = "default_problem_applicant_chance_percent")]
    pub problem_applicant_chance_percent: i32,
}

fn default_risky_rent_premium_percent() -> i32 {
    30
}

fn default_problem_applicant_chance_percent() -> i32 {
    18
}

impl Default for TenantRiskConfig {
    fn default() -> Self {
        Self {
            unreliable_threshold: 50,
            skip_rent_chance_percent: 20,
            low_behavior_threshold: 50,
            damage_chance_percent: 25,
            damage_amount: 6,
            hallway_disturbance_amount: 3,
            risky_rent_premium_percent: default_risky_rent_premium_percent(),
            problem_applicant_chance_percent: default_problem_applicant_chance_percent(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VettingConfig {
    pub credit_check_cost: i32,
    pub background_check_cost: i32,
    pub credit_thresholds: VettingThresholds,
    pub behavior_thresholds: VettingThresholds,
}

impl Default for VettingConfig {
    fn default() -> Self {
        Self {
            credit_check_cost: 25,
            background_check_cost: 10,
            credit_thresholds: VettingThresholds::default(),
            behavior_thresholds: VettingThresholds::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VettingThresholds {
    pub excellent: i32,
    pub good: i32,
    pub average: i32,
    pub below_average: i32,
}

impl Default for VettingThresholds {
    fn default() -> Self {
        Self {
            excellent: 90,
            good: 75,
            average: 60,
            below_average: 40,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketingConfig {
    pub none_cost: i32,
    pub social_media_cost: i32,
    pub local_newspaper_cost: i32,
    pub premium_agency_cost: i32,
}

impl Default for MarketingConfig {
    fn default() -> Self {
        Self {
            none_cost: 0,
            social_media_cost: 50,
            local_newspaper_cost: 150,
            premium_agency_cost: 500,
        }
    }
}

/// Tuning for emergent tenant life events (new job, job loss, new baby, …). The
/// per-type consequences are composed from these reusable magnitudes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifeEventsConfig {
    /// Per-tenant monthly chance (percent) that a life event occurs.
    pub monthly_chance_percent: i32,
    /// Happiness gained on a positive life change / lost on a negative one.
    pub positive_happiness: i32,
    pub negative_happiness: i32,
    /// Rent-tolerance shift when a tenant's income rises / falls.
    pub rent_tolerance_boost: i32,
    pub rent_tolerance_drop: i32,
    /// Move-out risk (0–100) for a major / minor life disruption.
    pub major_move_out_risk: i32,
    pub minor_move_out_risk: i32,
}

impl Default for LifeEventsConfig {
    fn default() -> Self {
        Self {
            monthly_chance_percent: 6,
            positive_happiness: 12,
            negative_happiness: 12,
            rent_tolerance_boost: 150,
            rent_tolerance_drop: 150,
            major_move_out_risk: 40,
            minor_move_out_risk: 15,
        }
    }
}
