use crate::data::config::RegulationsConfig;
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

    /// Months between scheduled inspections for this regulation.
    pub fn inspection_interval(&self) -> u32 {
        match self {
            RegulationType::FireSafety => 12,
            RegulationType::Electrical => 24,
            RegulationType::Plumbing => 18,
            RegulationType::Structural => 36,
            RegulationType::HistoricPreservation => 6,
            RegulationType::RentControl => 12,
            RegulationType::Accessibility => 24,
            RegulationType::HealthSanitation => 6,
        }
    }

    /// Human-readable name for UI / notifications.
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
        let months = regulation_type.inspection_interval();

        Self {
            regulation_type,
            active: true,
            warned: false,
            compliant: true,
            violation_count: 0,
            months_until_inspection: months,
        }
    }

    /// Record a violation against this regulation.
    pub fn add_violation(&mut self) {
        self.violation_count += 1;
        self.compliant = false;
        self.warned = true;
    }

    /// Current fine owed for this regulation.
    #[cfg(test)]
    pub fn current_fine(&self) -> i32 {
        if self.compliant || self.violation_count == 0 {
            0
        } else {
            self.regulation_type.base_fine() * self.violation_count as i32
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

    /// Get regulations for a building.
    #[cfg(test)]
    pub fn get_regulations(&self, building_id: u32) -> Option<&Vec<Regulation>> {
        self.building_regulations.get(&building_id)
    }

    /// True if any active regulation for the building is due for a scheduled
    /// inspection this month.
    pub fn has_due_inspection(&self, building_id: u32) -> bool {
        self.building_regulations
            .get(&building_id)
            .is_some_and(|regs| {
                regs.iter()
                    .any(|r| r.active && r.months_until_inspection == 0)
            })
    }

    /// Run an inspection against a building. `inspection_score` is the condition
    /// metric the inspector grades against (typically the min of average unit
    /// condition and hallway condition). A `Scheduled` trigger only grades the
    /// regulations that are actually due; any other trigger grades all of them.
    ///
    /// Mutates regulation state, accrues fines into `unpaid_fines`, records fix
    /// deadlines, adjusts `compliance_reputation`, and returns the `Inspection`
    /// (also pushed to `inspection_history`).
    pub fn run_inspection(
        &mut self,
        building_id: u32,
        inspection_score: i32,
        current_month: u32,
        trigger: InspectionTrigger,
        config: &RegulationsConfig,
    ) -> Inspection {
        let mut results = Vec::new();
        let mut new_pending = Vec::new();
        let mut total_fines = 0;
        let mut citations = 0;

        if let Some(regs) = self.building_regulations.get_mut(&building_id) {
            for reg in regs.iter_mut() {
                if !reg.active {
                    continue;
                }
                let due = reg.months_until_inspection == 0;
                if trigger == InspectionTrigger::Scheduled && !due {
                    continue;
                }

                // Reset the clock for the next scheduled cycle.
                reg.months_until_inspection = reg.regulation_type.inspection_interval();

                if inspection_score >= config.pass_condition_threshold {
                    reg.compliant = true;
                    results.push(InspectionResult {
                        regulation_type: reg.regulation_type.clone(),
                        passed: true,
                        issues_found: Vec::new(),
                        fine_amount: 0,
                        deadline_months: 0,
                        required_fixes: Vec::new(),
                    });
                } else {
                    reg.add_violation();
                    let fine =
                        (reg.regulation_type.base_fine() as f32 * config.fine_multiplier) as i32;
                    total_fines += fine;
                    citations += 1;
                    new_pending.push((
                        building_id,
                        reg.regulation_type.clone(),
                        current_month + config.fix_deadline_months,
                    ));
                    results.push(InspectionResult {
                        regulation_type: reg.regulation_type.clone(),
                        passed: false,
                        issues_found: vec![format!(
                            "{} below code standard",
                            reg.regulation_type.name()
                        )],
                        fine_amount: fine,
                        deadline_months: config.fix_deadline_months,
                        required_fixes: vec![format!(
                            "Raise building condition to clear the {} citation",
                            reg.regulation_type.name()
                        )],
                    });
                }
            }
        }

        // Apply cross-field mutations now that the `regs` borrow has ended.
        self.pending_fixes.extend(new_pending);
        if citations > 0 {
            self.unpaid_fines += total_fines;
            self.compliance_reputation = (self.compliance_reputation
                - citations * config.compliance_penalty_per_violation)
                .max(0);
        } else if !results.is_empty() {
            self.compliance_reputation =
                (self.compliance_reputation + config.compliance_gain_on_pass).min(100);
        }

        let inspection = Inspection {
            building_id,
            month: current_month,
            results,
            total_fines,
            triggered_by: trigger,
        };
        self.inspection_history.push(inspection.clone());
        inspection
    }

    /// Check if a building currently has any regulation violations.
    #[cfg(test)]
    pub fn has_violations(&self, building_id: u32) -> bool {
        self.get_regulations(building_id)
            .is_some_and(|regulations| {
                regulations
                    .iter()
                    .any(|regulation| !regulation.compliant || regulation.violation_count > 0)
            })
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
        self.pending_fixes
            .retain(|(building_id, reg_type, deadline)| {
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

    #[test]
    fn failed_inspection_cites_and_fines_a_neglected_building() {
        let cfg = RegulationsConfig::default();
        let mut system = ComplianceSystem::new();
        system.init_building_regulations(0, false);

        // A condition well below the pass threshold cites every regulation.
        let inspection = system.run_inspection(0, 10, 6, InspectionTrigger::Random, &cfg);

        assert!(inspection.total_fines > 0);
        assert!(inspection.results.iter().all(|r| !r.passed));
        assert_eq!(system.unpaid_fines, inspection.total_fines);
        assert!(system.compliance_reputation < 100);
        assert!(!system.pending_fixes.is_empty());
        assert!(system.has_violations(0));
    }

    #[test]
    fn clean_inspection_passes_a_maintained_building() {
        let cfg = RegulationsConfig::default();
        let mut system = ComplianceSystem::new();
        system.init_building_regulations(0, false);

        let inspection = system.run_inspection(0, 90, 6, InspectionTrigger::Random, &cfg);

        assert_eq!(inspection.total_fines, 0);
        assert!(inspection.results.iter().all(|r| r.passed));
        assert_eq!(system.unpaid_fines, 0);
        assert!(!system.has_violations(0));
    }

    #[test]
    fn scheduled_inspection_only_grades_due_regulations() {
        let cfg = RegulationsConfig::default();
        let mut system = ComplianceSystem::new();
        system.init_building_regulations(0, false);

        // Nothing is due on a freshly initialised building, so a scheduled
        // inspection grades nothing and levies no fine.
        let inspection = system.run_inspection(0, 10, 1, InspectionTrigger::Scheduled, &cfg);

        assert!(inspection.results.is_empty());
        assert_eq!(system.unpaid_fines, 0);
    }
}
