use serde::{Deserialize, Serialize};

/// Status of a mission
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionStatus {
    Available,
    Active,
    Completed,
    Failed,
    Expired,
}

/// Type of mission reward
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionReward {
    Money(i32),
    TaxBreak { months: u32, percentage: f32 },
    Reputation(i32),
    UnlockBuilding(u32),
}

/// A mission/quest in the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mission {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub giver_npc_id: u32,
    pub goal: MissionGoal,
    pub reward: MissionReward,
    pub status: MissionStatus,
    pub deadline: Option<u32>, // Month deadline
    pub started_month: Option<u32>,
}

/// Goals for missions
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MissionGoal {
    /// House a specific number of tenants of a type
    HouseTenants { count: u32, archetype: Option<String> },
    /// Reach a certain occupancy rate
    ReachOccupancy { percentage: f32 },
    /// Maintain happiness above a threshold for X months
    MaintainHappiness { threshold: f32, months: u32, current_months: u32 },
    /// Collect rent without issues for X months
    PerfectCollection { months: u32, current_months: u32 },
    /// Repair all issues in a building
    FullRepair { building_id: u32 },
    /// Acquire a new building
    AcquireBuilding,
}

impl Mission {
    pub fn new(
        id: u32,
        title: &str,
        description: &str,
        giver_npc_id: u32,
        goal: MissionGoal,
        reward: MissionReward,
        deadline: Option<u32>,
    ) -> Self {
        Self {
            id,
            title: title.to_string(),
            description: description.to_string(),
            giver_npc_id,
            goal,
            reward,
            status: MissionStatus::Available,
            deadline,
            started_month: None,
        }
    }
    
    /// Start the mission
    pub fn start(&mut self, current_month: u32) {
        self.status = MissionStatus::Active;
        self.started_month = Some(current_month);
    }
    
    /// Complete the mission
    pub fn complete(&mut self) {
        self.status = MissionStatus::Completed;
    }
    
    /// Fail the mission
    pub fn fail(&mut self) {
        self.status = MissionStatus::Failed;
    }
    
    /// Check if mission has expired
    pub fn check_expired(&mut self, current_month: u32) -> bool {
        if let Some(deadline) = self.deadline {
            if current_month > deadline && self.status == MissionStatus::Active {
                self.status = MissionStatus::Expired;
                return true;
            }
        }
        false
    }
}

/// Manages all missions in the game
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissionManager {
    pub missions: Vec<Mission>,
    pub next_mission_id: u32,
    /// Legacy tracking: major events that happened
    pub legacy_events: Vec<LegacyEvent>,
    /// Awards earned
    pub awards: Vec<BuildingAward>,
}

/// A major event recorded in the player's legacy
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LegacyEvent {
    pub month: u32,
    pub year: u32,
    pub title: String,
    pub description: String,
}

/// Building awards for the legacy system
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingAward {
    pub year: u32,
    pub title: String,
    pub building_name: String,
}

impl MissionManager {
    pub fn new() -> Self {
        Self {
            missions: Vec::new(),
            next_mission_id: 0,
            legacy_events: Vec::new(),
            awards: Vec::new(),
        }
    }
    
    /// Add a new mission
    pub fn add_mission(&mut self, mut mission: Mission) -> u32 {
        let id = self.next_mission_id;
        mission.id = id;
        self.next_mission_id += 1;
        self.missions.push(mission);
        id
    }
    
    /// Get available missions
    pub fn available_missions(&self) -> Vec<&Mission> {
        self.missions.iter()
            .filter(|m| m.status == MissionStatus::Available)
            .collect()
    }
    
    /// Get active missions
    pub fn active_missions(&self) -> Vec<&Mission> {
        self.missions.iter()
            .filter(|m| m.status == MissionStatus::Active)
            .collect()
    }
    
    /// Get completed missions
    pub fn completed_missions(&self) -> Vec<&Mission> {
        self.missions.iter()
            .filter(|m| m.status == MissionStatus::Completed)
            .collect()
    }
    
    /// Accept a mission by ID
    pub fn accept_mission(&mut self, mission_id: u32, current_month: u32) -> bool {
        if let Some(mission) = self.missions.iter_mut().find(|m| m.id == mission_id) {
            if mission.status == MissionStatus::Available {
                mission.start(current_month);
                return true;
            }
        }
        false
    }
    
    /// Check all active missions for expiration
    pub fn check_expirations(&mut self, current_month: u32) {
        for mission in &mut self.missions {
            mission.check_expired(current_month);
        }
    }
    
    /// Record a legacy event
    pub fn record_legacy_event(&mut self, month: u32, title: &str, description: &str) {
        let year = 2024 + (month / 12);
        self.legacy_events.push(LegacyEvent {
            month,
            year,
            title: title.to_string(),
            description: description.to_string(),
        });
    }
    
    /// Award a building award
    pub fn grant_award(&mut self, year: u32, title: &str, building_name: &str) {
        self.awards.push(BuildingAward {
            year,
            title: title.to_string(),
            building_name: building_name.to_string(),
        });
    }
    
    /// Check for annual awards (call at end of each year / month 12, 24, etc.)
    pub fn check_for_awards(
        &mut self,
        current_month: u32,
        building_name: &str,
        avg_happiness: f32,
        occupancy_rate: f32,
        total_tenants_housed: u32,
    ) {
        // Only check at year boundaries
        if current_month % 12 != 0 || current_month == 0 {
            return;
        }
        
        let year = 2024 + (current_month / 12);
        
        // Check if we already have an award for this year
        if self.awards.iter().any(|a| a.year == year) {
            return;
        }
        
        // Best Managed Property - high happiness
        if avg_happiness >= 80.0 {
            self.grant_award(year, "Best Managed Property", building_name);
            self.record_legacy_event(current_month, "Award Won!", 
                &format!("Won 'Best Managed Property {}' for {}", year, building_name));
        }
        // Full Occupancy Achievement
        else if occupancy_rate >= 1.0 {
            self.grant_award(year, "Perfect Occupancy", building_name);
            self.record_legacy_event(current_month, "Award Won!", 
                &format!("Achieved 100% occupancy at {} in {}", building_name, year));
        }
        // Community Builder - housed many tenants
        else if total_tenants_housed >= 20 {
            self.grant_award(year, "Community Builder", building_name);
            self.record_legacy_event(current_month, "Award Won!", 
                &format!("Housed {} tenants at {} by {}", total_tenants_housed, building_name, year));
        }
    }
    
    /// Generate initial missions from the council member
    pub fn generate_starter_missions(&mut self) {
        // Mission from Councilwoman Reyes
        let student_housing = Mission::new(
            0,
            "Student Housing Initiative",
            "The city needs affordable student housing. House 3 students in your buildings.",
            2, // Councilwoman Reyes NPC ID
            MissionGoal::HouseTenants { count: 3, archetype: Some("Student".to_string()) },
            MissionReward::TaxBreak { months: 6, percentage: 0.15 },
            Some(12), // 12 month deadline
        );
        self.add_mission(student_housing);
        
        let full_occupancy = Mission::new(
            0,
            "Full House",
            "Achieve 100% occupancy in your first building.",
            2,
            MissionGoal::ReachOccupancy { percentage: 1.0 },
            MissionReward::Money(5000),
            None,
        );
        self.add_mission(full_occupancy);
        
        // Senior Housing - for diverse community
        let senior_housing = Mission::new(
            0,
            "Senior Care Housing",
            "House 2 elderly tenants to support our senior community.",
            2, // Councilwoman Reyes
            MissionGoal::HouseTenants { count: 2, archetype: Some("Elderly".to_string()) },
            MissionReward::Reputation(25),
            Some(18),
        );
        self.add_mission(senior_housing);
    }
    
    /// Generate missions that appear later in the game
    pub fn generate_late_game_missions(&mut self, current_month: u32) {
        // Magnuson Corp rivalry mission - appears after month 6
        if current_month >= 6 && !self.missions.iter().any(|m| m.title == "Expansion Race") {
            let expansion = Mission::new(
                0,
                "Expansion Race",
                "Magnuson Corp is expanding! Acquire a second building before they dominate the market.",
                1, // Magnuson Corp NPC ID (as challenger)
                MissionGoal::AcquireBuilding,
                MissionReward::Money(10000),
                Some(current_month + 24), // 2 year deadline
            );
            self.add_mission(expansion);
        }
        
        // Family housing - appears after month 12
        if current_month >= 12 && !self.missions.iter().any(|m| m.title == "Family Friendly") {
            let family = Mission::new(
                0,
                "Family Friendly",
                "Create a family-friendly environment. House 3 family tenants.",
                2, // Councilwoman Reyes
                MissionGoal::HouseTenants { count: 3, archetype: Some("Family".to_string()) },
                MissionReward::TaxBreak { months: 12, percentage: 0.10 },
                Some(current_month + 18),
            );
            self.add_mission(family);
        }
    }
}

impl Default for MissionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mission_lifecycle() {
        let mut manager = MissionManager::new();
        
        let mission = Mission::new(
            0,
            "Test Mission",
            "A test",
            0,
            MissionGoal::AcquireBuilding,
            MissionReward::Money(1000),
            Some(10),
        );
        
        let id = manager.add_mission(mission);
        assert_eq!(manager.available_missions().len(), 1);
        
        manager.accept_mission(id, 1);
        assert_eq!(manager.active_missions().len(), 1);
        assert_eq!(manager.available_missions().len(), 0);
    }
    
    #[test]
    fn test_mission_expiration() {
        let mut manager = MissionManager::new();
        
        let mission = Mission::new(
            0,
            "Expiring Mission",
            "Will expire",
            0,
            MissionGoal::AcquireBuilding,
            MissionReward::Money(1000),
            Some(5),
        );
        
        let id = manager.add_mission(mission);
        manager.accept_mission(id, 1);
        
        manager.check_expirations(10);
        
        let mission = manager.missions.iter().find(|m| m.id == id).unwrap();
        assert_eq!(mission.status, MissionStatus::Expired);
    }
    
    #[test]
    fn test_legacy_system() {
        let mut manager = MissionManager::new();
        
        manager.record_legacy_event(15, "The Great Fire", "A fire broke out in Building A.");
        manager.grant_award(2025, "Best Managed Property", "Sunset Apartments");
        
        assert_eq!(manager.legacy_events.len(), 1);
        assert_eq!(manager.awards.len(), 1);
        assert_eq!(manager.awards[0].year, 2025);
    }
}
