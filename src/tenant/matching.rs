use crate::building::Apartment;
use super::{Tenant, happiness};

/// Result of matching a tenant to an apartment
#[derive(Clone, Debug)]
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

/// Get all apartments a tenant would consider (meets minimum)
pub fn get_acceptable_apartments<'a>(
    tenant: &Tenant,
    apartments: &'a [&'a Apartment]
) -> Vec<(&'a Apartment, MatchResult)> {
    apartments
        .iter()
        .filter(|apt| apt.is_vacant())
        .map(|apt| (*apt, calculate_match_score(tenant, apt)))
        .filter(|(_, result)| result.meets_minimum)
        .collect()
}
