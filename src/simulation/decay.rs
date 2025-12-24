use crate::building::Building;
use super::GameEvent;

/// Decay rates per tick
pub mod rates {
    /// Condition points lost per apartment per tick
    pub const APARTMENT_DECAY: i32 = 2;
    
    /// Hallway condition decay per tick
    pub const HALLWAY_DECAY: i32 = 1;
    
    /// Threshold for "poor condition" warning
    pub const POOR_CONDITION_THRESHOLD: i32 = 40;
    
    /// Threshold for "critical condition" warning
    pub const CRITICAL_CONDITION_THRESHOLD: i32 = 20;
}

/// Apply monthly decay to all building elements
/// Returns events for significant decay milestones
pub fn apply_decay(building: &mut Building) -> Vec<GameEvent> {
    let mut events = Vec::new();
    
    // Track conditions before decay for event generation
    let conditions_before: Vec<_> = building.apartments
        .iter()
        .map(|a| (a.id, a.unit_number.clone(), a.condition))
        .collect();
    
    let hallway_before = building.hallway_condition;
    
    // Apply decay
    building.apply_monthly_decay();
    
    // Check for significant condition changes in apartments
    for (id, unit, old_condition) in conditions_before {
        if let Some(apt) = building.get_apartment(id) {
            let new_condition = apt.condition;
            
            // Check for crossing thresholds
            if old_condition >= rates::CRITICAL_CONDITION_THRESHOLD 
               && new_condition < rates::CRITICAL_CONDITION_THRESHOLD 
            {
                events.push(GameEvent::CriticalCondition {
                    apartment_unit: unit,
                    condition: new_condition,
                });
            } else if old_condition >= rates::POOR_CONDITION_THRESHOLD 
               && new_condition < rates::POOR_CONDITION_THRESHOLD 
            {
                events.push(GameEvent::PoorCondition {
                    apartment_unit: unit,
                    condition: new_condition,
                });
            }
        }
    }
    
    // Check hallway
    let hallway_after = building.hallway_condition;
    if hallway_before >= rates::POOR_CONDITION_THRESHOLD 
       && hallway_after < rates::POOR_CONDITION_THRESHOLD 
    {
        events.push(GameEvent::HallwayDeteriorating {
            condition: hallway_after,
        });
    }
    
    events
}

/// Calculate turns until apartment reaches critical condition
pub fn turns_until_critical(current_condition: i32) -> i32 {
    let gap = current_condition - rates::CRITICAL_CONDITION_THRESHOLD;
    if gap <= 0 {
        return 0;
    }
    (gap as f32 / rates::APARTMENT_DECAY as f32).ceil() as i32
}

/// Get overall building health status
#[derive(Clone, Debug, PartialEq)]
pub enum BuildingHealth {
    Excellent,  // Average condition >= 80
    Good,       // Average condition >= 60
    Fair,       // Average condition >= 40
    Poor,       // Average condition >= 20
    Critical,   // Average condition < 20
}

pub fn assess_building_health(building: &Building) -> BuildingHealth {
    let avg_condition: i32 = if building.apartments.is_empty() {
        building.hallway_condition
    } else {
        let apt_total: i32 = building.apartments.iter().map(|a| a.condition).sum();
        let apt_avg = apt_total / building.apartments.len() as i32;
        (apt_avg + building.hallway_condition) / 2
    };
    
    match avg_condition {
        80..=100 => BuildingHealth::Excellent,
        60..=79 => BuildingHealth::Good,
        40..=59 => BuildingHealth::Fair,
        20..=39 => BuildingHealth::Poor,
        _ => BuildingHealth::Critical,
    }
}
