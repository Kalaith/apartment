
use serde::{Deserialize, Serialize};

/// Types of building regulations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RegulationType {
    /// Fire safety requirements
    FireSafety,
    /// Electrical safety standards
    Electrical,
    /// Plumbing and water safety
    Plumbing,
    /// Structural integrity
    Structural,
    /// Historic preservation (stricter in historic neighborhoods)
    HistoricPreservation,
    /// Rent control regulations
    RentControl,
    /// Accessibility requirements
    Accessibility,
    /// Health and sanitation
    HealthSanitation,
}

impl RegulationType {
    /// Base fine for violation
    pub fn base_fine(&self) -> i32 {
        match self {
            RegulationType::FireSafety => 2000,
            RegulationType::Electrical => 1500,
            RegulationType::Plumbing => 1000,
            RegulationType::Structural => 3000,
            RegulationType::HistoricPreservation => 5000,
            RegulationType::RentControl => 2500,
            RegulationType::Accessibility => 1500,
            RegulationType::HealthSanitation => 1000,
        }
    }
}

/// A specific regulation affecting a building
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Regulation {
    pub regulation_type: RegulationType,
    /// Is this regulation currently being enforced?
    pub active: bool,
    /// Has the player been warned about this?
    pub warned: bool,
    /// Current compliance status
    pub compliant: bool,
    /// Number of violations
    pub violation_count: u32,
    /// Months until next mandatory inspection
    pub months_until_inspection: u32,
}

impl Regulation {
    pub fn new(regulation_type: RegulationType) -> Self {
        let months = match regulation_type {
            RegulationType::FireSafety => 12,
            RegulationType::Electrical => 24,
            RegulationType::Plumbing => 18,
            RegulationType::Structural => 36,
            RegulationType::HistoricPreservation => 6,
            RegulationType::RentControl => 12,
            RegulationType::Accessibility => 24,
            RegulationType::HealthSanitation => 6,
        };

        Self {
            regulation_type,
            active: true,
            warned: false,
            compliant: true,
            violation_count: 0,
            months_until_inspection: months,
        }
    }
}

/// Result of an inspection
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InspectionResult {
    pub regulation_type: RegulationType,
    pub passed: bool,
    pub issues_found: Vec<String>,
    pub fine_amount: i32,
    pub deadline_months: u32,
    pub required_fixes: Vec<String>,
}

/// An inspection event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inspection {
    pub building_id: u32,
    pub month: u32,
    pub results: Vec<InspectionResult>,
    pub total_fines: i32,
    pub triggered_by: InspectionTrigger,
}

/// What triggered the inspection
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InspectionTrigger {
    /// Scheduled regular inspection
    Scheduled,
    /// Tenant filed a complaint
    TenantComplaint,
    /// Random spot check
    Random,
    /// Following up on previous violation
    FollowUp,
}

impl InspectionTrigger {}

/// Manages all compliance and inspection logic for a player
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComplianceSystem {
    /// Regulations by building ID
    pub building_regulations: std::collections::HashMap<u32, Vec<Regulation>>,
    /// History of inspections
    pub inspection_history: Vec<Inspection>,
    /// Pending fixes with deadlines (building_id, regulation_type, deadline_month)
    pub pending_fixes: Vec<(u32, RegulationType, u32)>,
    /// Total fines unpaid
    pub unpaid_fines: i32,
    /// Player's overall compliance reputation (affects inspection frequency)
    pub compliance_reputation: i32,
}

impl ComplianceSystem {
    pub fn new() -> Self {
        Self {
            building_regulations: std::collections::HashMap::new(),
            inspection_history: Vec::new(),
            pending_fixes: Vec::new(),
            unpaid_fines: 0,
            compliance_reputation: 100,
        }
    }

    /// Initialize regulations for a new building
    pub fn init_building_regulations(&mut self, building_id: u32, is_historic: bool) {
        let mut regulations = vec![
            Regulation::new(RegulationType::FireSafety),
            Regulation::new(RegulationType::Electrical),
            Regulation::new(RegulationType::Plumbing),
            Regulation::new(RegulationType::Structural),
            Regulation::new(RegulationType::HealthSanitation),
        ];

        if is_historic {
            regulations.push(Regulation::new(RegulationType::HistoricPreservation));
        }

        self.building_regulations.insert(building_id, regulations);
    }

    /// Monthly tick - decrement inspection timers, check deadlines
    pub fn tick(&mut self, current_month: u32) {
        // Decrement inspection timers
        for regulations in self.building_regulations.values_mut() {
            for reg in regulations.iter_mut() {
                if reg.months_until_inspection > 0 {
                    reg.months_until_inspection -= 1;
                }
            }
        }

        // Check for missed deadlines
        let mut escalations = Vec::new();
        self.pending_fixes.retain(|(building_id, reg_type, deadline)| {
            if current_month >= *deadline {
                escalations.push((*building_id, reg_type.clone()));
                false
            } else {
                true
            }
        });

        // Apply penalties for missed deadlines
        for (_building_id, reg_type) in escalations {
            self.unpaid_fines += reg_type.base_fine();
            self.compliance_reputation = (self.compliance_reputation - 15).max(0);
        }
    }
}

impl Default for ComplianceSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regulation_fines() {
        let mut reg = Regulation::new(RegulationType::FireSafety);
        assert!(reg.compliant);
        assert_eq!(reg.current_fine(), 0);
        
        reg.add_violation();
        assert!(!reg.compliant);
        assert!(reg.current_fine() > 0);
    }

    #[test]
    fn test_compliance_system() {
        let mut system = ComplianceSystem::new();
        system.init_building_regulations(0, false);
        
        assert!(system.get_regulations(0).is_some());
        assert!(!system.has_violations(0));
    }
}
