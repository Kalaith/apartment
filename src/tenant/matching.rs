
use crate::building::Apartment;
use super::{Tenant, happiness};

/// Result of matching a tenant to an apartment
use serde::{Deserialize, Serialize};

/// Result of matching a tenant to an apartment
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchResult {
    pub score: i32,              // 0-100, higher = better match
    pub meets_minimum: bool,     // Would tenant even consider this?
    pub reasons: Vec<String>,    // Why this score
}

/// Calculate how well a tenant matches an apartment
pub fn calculate_match_score(tenant: &Tenant, apartment: &Apartment) -> MatchResult {
    let mut score = 50;  // Start at neutral
    let mut reasons = Vec::new();
    
    let prefs = tenant.archetype.preferences();
    
    // Check minimum requirements
    let meets_minimum = happiness::apartment_meets_minimum(tenant, apartment);
    if !meets_minimum {
        return MatchResult {
            score: 0,
            meets_minimum: false,
            reasons: vec!["Does not meet minimum requirements".to_string()],
        };
    }
    
    // Rent scoring
    let rent_diff = prefs.ideal_rent_max - apartment.rent_price;
    if rent_diff > 200 {
        score += 15;
        reasons.push("Great price".to_string());
    } else if rent_diff > 0 {
        score += 8;
        reasons.push("Fair price".to_string());
    } else if rent_diff > -100 {
        score -= 5;
        reasons.push("Slightly expensive".to_string());
    } else {
        score -= 15;
        reasons.push("Expensive".to_string());
    }
    
    // Condition scoring
    if apartment.condition >= 80 {
        let bonus = (15.0 * prefs.condition_sensitivity) as i32;
        score += bonus;
        reasons.push("Excellent condition".to_string());
    } else if apartment.condition >= 60 {
        let bonus = (8.0 * prefs.condition_sensitivity) as i32;
        score += bonus;
        reasons.push("Good condition".to_string());
    } else if apartment.condition < 50 {
        let penalty = (10.0 * prefs.condition_sensitivity) as i32;
        score -= penalty;
        reasons.push("Poor condition".to_string());
    }
    
    // Noise scoring
    match apartment.effective_noise() {
        crate::building::NoiseLevel::Low => {
            if prefs.prefers_quiet {
                let bonus = (12.0 * prefs.noise_sensitivity) as i32;
                score += bonus;
                reasons.push("Nice and quiet".to_string());
            }
        }
        crate::building::NoiseLevel::High => {
            let penalty = (15.0 * prefs.noise_sensitivity) as i32;
            score -= penalty;
            reasons.push("Too noisy".to_string());
        }
    }
    
    // Design scoring
    if let Some(ref preferred) = prefs.preferred_design {
        if &apartment.design == preferred {
            let bonus = (18.0 * prefs.design_sensitivity) as i32;
            score += bonus;
            reasons.push(format!("Loves the {:?} style", apartment.design));
        }
    }
    
    // Size bonus (everyone likes more space)
    match apartment.size {
        crate::building::ApartmentSize::Medium => {
            score += 5;
            reasons.push("Good space".to_string());
        }
        crate::building::ApartmentSize::Small => {}
    }
    
    MatchResult {
        score: score.clamp(0, 100),
        meets_minimum: true,
        reasons,
    }
}

/// Find the best apartment match for a tenant from available options
pub fn find_best_match<'a>(
    tenant: &Tenant, 
    apartments: &'a [&'a Apartment]
) -> Option<(&'a Apartment, MatchResult)> {
    apartments
        .iter()
        .filter(|apt| apt.is_vacant())
        .map(|apt| (*apt, calculate_match_score(tenant, apt)))
        .filter(|(_, result)| result.meets_minimum)
        .max_by_key(|(_, result)| result.score)
}



/// Parameters for a lease offer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseOffer {
    pub rent_price: i32,
    pub security_deposit_months: u32,  // 1, 2, or 3
    pub lease_duration_months: u32,    // Usually 6 or 12
    pub cleaning_fee: i32,             // 0 or more
}

impl LeaseOffer {
    /// Default standard lease
    pub fn standard(rent: i32) -> Self {
        Self {
            rent_price: rent,
            security_deposit_months: 1,
            lease_duration_months: 12,
            cleaning_fee: 0,
        }
    }
}

/// Calculate probability of tenant accepting a lease offer (0.0 to 1.0)
pub fn evaluate_lease_offer(tenant: &Tenant, offer: &LeaseOffer) -> f32 {
    let prefs = tenant.archetype.preferences();
    let mut probability = 1.0;
    
    // 1. Rent affordability (Hard limit)
    if offer.rent_price > tenant.rent_tolerance {
        return 0.0;
    }
    
    // 2. Security Deposit Impact
    // High deposit hurts budget-conscious tenants (Students, Artists)
    // Rent sensitivity serves as a proxy for "budget consciousness"
    if offer.security_deposit_months > 1 {
        let deposit_penalty = match offer.security_deposit_months {
            2 => 0.15 * prefs.rent_sensitivity,
            _ => 0.35 * prefs.rent_sensitivity, // 3+ months
        };
        probability -= deposit_penalty;
    }
    
    // 3. Lease Duration
    // Students/Artists might prefer shorter leases (6 months)
    // Families/Elderly/Professionals prefer stability (12 months)
    match tenant.archetype {
        crate::tenant::TenantArchetype::Student | crate::tenant::TenantArchetype::Artist => {
            if offer.lease_duration_months == 6 {
                probability += 0.1; // Bonus for flexibility
            } else if offer.lease_duration_months > 12 {
                probability -= 0.1; // Too long
            }
        },
        _ => {
            // Stability seekers
            if offer.lease_duration_months < 12 {
                probability -= 0.15; // Too unstable
            }
        }
    }
    
    // 4. Cleaning Fee
    if offer.cleaning_fee > 0 {
        // Flat penalty relative to rent
        let fee_ratio = offer.cleaning_fee as f32 / offer.rent_price as f32;
        probability -= fee_ratio * prefs.rent_sensitivity;
    }
    
    // 5. Rent Value (Deal vs Rip-off)
    let rent_diff = prefs.ideal_rent_max - offer.rent_price;
    if rent_diff < 0 {
        // Slightly above comfortable max (but below absolute tolerance)
        probability -= 0.1; 
    } else if rent_diff > 100 {
        // Good deal
        probability += 0.1; 
    }
    
    probability.clamp(0.0, 1.0)
}
