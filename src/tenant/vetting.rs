use super::application::TenantApplication;
use crate::economy::PlayerFunds;

/// Costs for vetting actions
pub const COST_CREDIT_CHECK: i32 = 25;
pub const COST_BACKGROUND_CHECK: i32 = 10;

/// Results of a credit check
pub struct CreditCheckResult {
    pub reliability_score: i32,
    pub recommendation: String,
}

/// Results of a background check
pub struct BackgroundCheckResult {
    pub behavior_score: i32,
    pub history_notes: String,
}

/// Perform a credit check on a tenant applicant
pub fn perform_credit_check(
    application: &mut TenantApplication,
    funds: &mut PlayerFunds
) -> Option<CreditCheckResult> {
    if application.revealed_reliability {
        return None; // Already checked
    }
    
    if !funds.spend(COST_CREDIT_CHECK) {
        return None; // Cannot afford
    }
    
    // Reveal stats
    application.revealed_reliability = true;
    let score = application.tenant.rent_reliability;
    
    // Generate simple recommendation string
    let recommendation = if score >= 90 {
        "Excellent credit history. Highly recommended.".to_string()
    } else if score >= 75 {
        "Good credit standing. No major concerns.".to_string()
    } else if score >= 60 {
        "Average credit. Has some missed payments.".to_string()
    } else if score >= 40 {
        "Below average. High risk of late rent.".to_string()
    } else {
        "Poor credit history. Default risk high.".to_string()
    };
    
    Some(CreditCheckResult {
        reliability_score: score,
        recommendation,
    })
}

/// Perform a background check (previous landlord reference)
pub fn perform_background_check(
    application: &mut TenantApplication,
    funds: &mut PlayerFunds
) -> Option<BackgroundCheckResult> {
    if application.revealed_behavior {
        return None; // Already checked
    }
    
    if !funds.spend(COST_BACKGROUND_CHECK) {
        return None; // Cannot afford
    }
    
    // Reveal stats
    application.revealed_behavior = true;
    let score = application.tenant.behavior_score;
    
    // Generate notes
    let history_notes = if score >= 90 {
        "Quiet, respectful, keeps unit in perfect condition.".to_string()
    } else if score >= 75 {
        "Generally good tenant. No noise complaints.".to_string()
    } else if score >= 60 {
        "Occasional minor complaints but pays for damages.".to_string()
    } else if score >= 40 {
        "History of noise complaints and minor damage.".to_string()
    } else {
        "Evicted from previous apartment for disturbance.".to_string()
    };
    
    Some(BackgroundCheckResult {
        behavior_score: score,
        history_notes,
    })
}
