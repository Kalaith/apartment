
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TenantArchetype {
    Student,
    Professional,
    Artist,
    Family,
    Elderly,
}

impl TenantArchetype {
    pub fn name(&self) -> &'static str {
        match self {
            TenantArchetype::Student => "Student",
            TenantArchetype::Professional => "Professional",
            TenantArchetype::Artist => "Artist",
            TenantArchetype::Family => "Family",
            TenantArchetype::Elderly => "Elderly",
        }
    }
    
    /// Get the preferences for this archetype
    pub fn preferences(&self) -> ArchetypePreferences {
        match self {
            TenantArchetype::Student => ArchetypePreferences {
                rent_sensitivity: 0.9,      // Very price sensitive
                condition_sensitivity: 0.3, // Low - tolerates some wear
                noise_sensitivity: 0.4,     // Low - can deal with noise
                design_sensitivity: 0.2,    // Doesn't care much
                
                ideal_rent_max: 700,
                min_acceptable_condition: 30,
                prefers_quiet: false,
                preferred_design: None,
                hates_design: None,
            },
            TenantArchetype::Professional => ArchetypePreferences {
                rent_sensitivity: 0.4,      // Can afford more
                condition_sensitivity: 0.8, // Values good condition
                noise_sensitivity: 0.9,     // Hates noise
                design_sensitivity: 0.5,    // Moderate
                
                ideal_rent_max: 1200,
                min_acceptable_condition: 60,
                prefers_quiet: true,
                preferred_design: None,
                hates_design: None,
            },
            TenantArchetype::Artist => ArchetypePreferences {
                rent_sensitivity: 0.6,      // Moderate budget
                condition_sensitivity: 0.5, // Moderate
                noise_sensitivity: 0.5,     // Moderate
                design_sensitivity: 0.95,   // Very design focused
                
                ideal_rent_max: 900,
                min_acceptable_condition: 40,
                prefers_quiet: false,
                preferred_design: Some(crate::building::DesignType::Cozy),
                hates_design: Some(crate::building::DesignType::Bare),
            },
            TenantArchetype::Family => ArchetypePreferences {
                rent_sensitivity: 0.7,      // Moderate-High (kids are expensive)
                condition_sensitivity: 0.7, // Needs decent condition
                noise_sensitivity: 1.0,     // Hates noise (kids sleeping)
                design_sensitivity: 0.4,    // Moderate
                
                ideal_rent_max: 1100,
                min_acceptable_condition: 50,
                prefers_quiet: true,
                preferred_design: Some(crate::building::DesignType::Practical),
                hates_design: None,
            },
            TenantArchetype::Elderly => ArchetypePreferences {
                rent_sensitivity: 0.8,      // Fixed income
                condition_sensitivity: 0.6, // Moderate
                noise_sensitivity: 0.9,     // Hates noise
                design_sensitivity: 0.3,    // Low
                
                ideal_rent_max: 800,
                min_acceptable_condition: 45,
                prefers_quiet: true,
                preferred_design: None,
                hates_design: Some(crate::building::DesignType::Bare), // Wants some comfort
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct ArchetypePreferences {
    // Sensitivity weights (0.0 - 1.0, higher = more affected)
    pub rent_sensitivity: f32,
    pub condition_sensitivity: f32,
    pub noise_sensitivity: f32,
    pub design_sensitivity: f32,
    
    // Thresholds
    pub ideal_rent_max: i32,
    pub min_acceptable_condition: i32,
    pub prefers_quiet: bool,
    
    // Design preferences
    pub preferred_design: Option<crate::building::DesignType>,
    pub hates_design: Option<crate::building::DesignType>,
}
