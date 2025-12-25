#![allow(dead_code)]
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
    pub fn name(&self) -> &'static str {
        match self {
            RegulationType::FireSafety => "Fire Safety",
            RegulationType::Electrical => "Electrical",
            RegulationType::Plumbing => "Plumbing",
            RegulationType::Structural => "Structural",
            RegulationType::HistoricPreservation => "Historic Preservation",
            RegulationType::RentControl => "Rent Control",
            RegulationType::Accessibility => "Accessibility",
            RegulationType::HealthSanitation => "Health & Sanitation",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RegulationType::FireSafety => "Smoke detectors, fire exits, extinguishers",
            RegulationType::Electrical => "Wiring, outlets, circuit breakers up to code",
            RegulationType::Plumbing => "Pipes, water heaters, sewage systems",
            RegulationType::Structural => "Foundation, walls, roof integrity",
            RegulationType::HistoricPreservation => "Maintain original character and materials",
            RegulationType::RentControl => "Rent increase limits and tenant protections",
            RegulationType::Accessibility => "Ramps, elevators, accessible units",
            RegulationType::HealthSanitation => "Pest control, waste disposal, air quality",
        }
    }

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

    /// Cost to bring into compliance
    pub fn compliance_cost(&self) -> i32 {
        match self {
            RegulationType::FireSafety => 500,
            RegulationType::Electrical => 800,
            RegulationType::Plumbing => 600,
            RegulationType::Structural => 2000,
            RegulationType::HistoricPreservation => 3000,
            RegulationType::RentControl => 0, // Just follow the rules
            RegulationType::Accessibility => 4000,
            RegulationType::HealthSanitation => 400,
        }
    }

    /// How condition affects this regulation (threshold below which violations occur)
    pub fn condition_threshold(&self) -> i32 {
        match self {
            RegulationType::FireSafety => 40,
            RegulationType::Electrical => 35,
            RegulationType::Plumbing => 30,
            RegulationType::Structural => 25,
            RegulationType::HistoricPreservation => 50, // Stricter
            RegulationType::RentControl => 0, // Not condition-based
            RegulationType::Accessibility => 0, // Requires specific upgrade
            RegulationType::HealthSanitation => 35,
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

    /// Record a violation
    pub fn add_violation(&mut self) {
        self.violation_count += 1;
        self.compliant = false;
    }

    /// Bring into compliance
    pub fn bring_compliant(&mut self) {
        self.compliant = true;
        self.warned = false;
    }

    /// Calculate current fine
    pub fn current_fine(&self) -> i32 {
        if self.compliant {
            0
        } else {
            let base = self.regulation_type.base_fine();
            // Escalating fines for repeat violations
            base * (1 + self.violation_count as i32 / 2)
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

impl InspectionTrigger {
    pub fn description(&self) -> &'static str {
        match self {
            InspectionTrigger::Scheduled => "Routine scheduled inspection",
            InspectionTrigger::TenantComplaint => "Inspection following tenant complaint",
            InspectionTrigger::Random => "Random spot inspection",
            InspectionTrigger::FollowUp => "Follow-up on previous violations",
        }
    }
}

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

    /// Get regulations for a building
    pub fn get_regulations(&self, building_id: u32) -> Option<&Vec<Regulation>> {
        self.building_regulations.get(&building_id)
    }

    /// Get mutable regulations for a building
    pub fn get_regulations_mut(&mut self, building_id: u32) -> Option<&mut Vec<Regulation>> {
        self.building_regulations.get_mut(&building_id)
    }

    /// Check if a building has any violations
    pub fn has_violations(&self, building_id: u32) -> bool {
        self.building_regulations.get(&building_id)
            .map(|regs| regs.iter().any(|r| !r.compliant))
            .unwrap_or(false)
    }

    /// Count violations for a building
    pub fn violation_count(&self, building_id: u32) -> usize {
        self.building_regulations.get(&building_id)
            .map(|regs| regs.iter().filter(|r| !r.compliant).count())
            .unwrap_or(0)
    }

    /// Perform an inspection on a building
    pub fn inspect_building(
        &mut self,
        building_id: u32,
        building: &crate::building::Building,
        current_month: u32,
        trigger: InspectionTrigger,
    ) -> Option<Inspection> {
        let regulations = self.building_regulations.get_mut(&building_id)?;
        let mut results = Vec::new();
        let mut total_fines = 0;

        for reg in regulations.iter_mut() {
            if !reg.active {
                continue;
            }

            let threshold = reg.regulation_type.condition_threshold();
            let avg_condition = building.apartments.iter()
                .map(|a| a.condition)
                .sum::<i32>() / building.apartments.len().max(1) as i32;

            let passed = if threshold > 0 {
                avg_condition >= threshold
            } else {
                // Non-condition-based regulations (rent control, accessibility)
                reg.compliant
            };

            let mut issues = Vec::new();
            let mut required_fixes = Vec::new();

            if !passed {
                reg.add_violation();
                
                let fine = reg.current_fine();
                total_fines += fine;

                issues.push(format!(
                    "Building condition ({}) below {} standard ({})",
                    avg_condition, reg.regulation_type.name(), threshold
                ));

                required_fixes.push(format!(
                    "Repair building to reach condition level {}",
                    threshold
                ));

                // Add to pending fixes
                self.pending_fixes.push((building_id, reg.regulation_type.clone(), current_month + 3));
            } else {
                reg.bring_compliant();
            }

            results.push(InspectionResult {
                regulation_type: reg.regulation_type.clone(),
                passed,
                issues_found: issues,
                fine_amount: if passed { 0 } else { reg.current_fine() },
                deadline_months: if passed { 0 } else { 3 },
                required_fixes,
            });

            // Reset inspection timer
            reg.months_until_inspection = match reg.regulation_type {
                RegulationType::FireSafety => 12,
                RegulationType::Electrical => 24,
                RegulationType::HistoricPreservation => 6,
                _ => 18,
            };
        }

        self.unpaid_fines += total_fines;
        
        // Reduce reputation for violations
        if total_fines > 0 {
            self.compliance_reputation = (self.compliance_reputation - 10).max(0);
        } else {
            self.compliance_reputation = (self.compliance_reputation + 5).min(100);
        }

        let inspection = Inspection {
            building_id,
            month: current_month,
            results,
            total_fines,
            triggered_by: trigger,
        };

        self.inspection_history.push(inspection.clone());

        Some(inspection)
    }

    /// Check if any inspections are due for a building
    pub fn check_inspections_due(&self, building_id: u32) -> Vec<RegulationType> {
        self.building_regulations.get(&building_id)
            .map(|regs| {
                regs.iter()
                    .filter(|r| r.active && r.months_until_inspection == 0)
                    .map(|r| r.regulation_type.clone())
                    .collect()
            })
            .unwrap_or_default()
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

        // Random spot inspections based on reputation
        // (low reputation = more inspections)
        // This would be triggered from game state, not here
    }

    /// Pay off some of the fines
    pub fn pay_fines(&mut self, amount: i32) -> i32 {
        let paid = amount.min(self.unpaid_fines);
        self.unpaid_fines -= paid;
        paid
    }

    /// Get upcoming inspection deadlines
    pub fn upcoming_deadlines(&self) -> Vec<(u32, RegulationType, u32)> {
        self.pending_fixes.clone()
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
