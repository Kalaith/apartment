use macroquad::prelude::*;
use crate::narrative::{MissionGoal, MissionReward, MissionStatus};
use crate::ui::{FloatingText, colors};
use super::GameplayState;

/// System for handling mission updates and rewards
pub fn update_missions(state: &mut GameplayState) {
    let current_month = state.current_tick;
    
    // Check for expirations (expired missions are marked as such)
    state.missions.check_expirations(current_month);
    
    // Check for unrecoverable failures (e.g., building sold that was needed)
    for mission in &mut state.missions.missions {
        if mission.status == MissionStatus::Active {
            // Check if "AcquireBuilding" mission should fail if we sold a building
            if matches!(mission.goal, MissionGoal::AcquireBuilding) && state.city.buildings.is_empty() {
                mission.fail();
                state.floating_texts.push(FloatingText::new(
                    "Mission Failed!",
                    screen_width() / 2.0,
                    screen_height() / 2.0,
                    colors::NEGATIVE,
                ));
            }
        }
    }
    
    // Check active missions for completion
    let active_mission_ids: Vec<u32> = state.missions.active_missions().iter().map(|m| m.id).collect();
    
    for mission_id in active_mission_ids {
        let mut completed = false;
        let mut reward = None;
        let mut legacy_info: Option<(String, String)> = None;
        
        if let Some(mission) = state.missions.missions.iter_mut().find(|m| m.id == mission_id) {
            match &mission.goal {
                MissionGoal::HouseTenants { count, archetype } => {
                    let current_count = state.tenants.iter()
                        .filter(|t| archetype.as_ref().map_or(true, |arch| t.archetype.name() == arch))
                        .count();
                    if current_count as u32 >= *count {
                        completed = true;
                    }
                }
                MissionGoal::ReachOccupancy { percentage } => {
                    let total = state.building.apartments.len();
                    let occupied = state.building.occupancy_count();
                    if total > 0 && (occupied as f32 / total as f32) >= *percentage {
                        completed = true;
                    }
                }
                MissionGoal::AcquireBuilding => {
                    if state.city.buildings.len() > 1 { // Started with 1
                        completed = true;
                    }
                }
                // Implement other goals...
                _ => {}
            }
            
            if completed {
                mission.complete();
                reward = Some(mission.reward.clone());
                legacy_info = Some((mission.title.clone(), mission.description.clone()));
            }
        }
        
        // Record legacy (outside the mutable borrow)
        if let Some((title, description)) = legacy_info {
            state.missions.record_legacy_event(
                current_month, 
                &format!("Mission Complete: {}", title), 
                &format!("Completed objective: {}", description)
            );
        }
        
        // Grant reward
        if let Some(r) = reward {
            match r {
                MissionReward::Money(amount) => {
                    let t = crate::economy::Transaction::income(
                        crate::economy::TransactionType::Grant,
                        amount,
                        "Mission Reward",
                        current_month
                    );
                    state.funds.add_income(t);
                    
                    state.floating_texts.push(FloatingText::new(
                        &format!("+${}", amount),
                        screen_width() / 2.0,
                        screen_height() / 2.0 + 30.0,
                        colors::POSITIVE,
                    ));
                }
                MissionReward::UnlockBuilding(_) => {
                    // Logic to unlock building (handled by city/economy generally)
                }
                MissionReward::Reputation(amount) => {
                    // Logic for reputation
                    state.floating_texts.push(FloatingText::new(
                        &format!("+{} Rep", amount),
                        screen_width() / 2.0,
                        screen_height() / 2.0 + 30.0,
                        colors::ACCENT,
                    ));
                }
                MissionReward::TaxBreak { months, percentage } => {
                     state.floating_texts.push(FloatingText::new(
                        &format!("Tax Break! {}% for {} months", (percentage * 100.0) as i32, months),
                        screen_width() / 2.0,
                        screen_height() / 2.0 + 30.0,
                        colors::POSITIVE,
                    ));
                    // TODO: Implement actual tax reduction logic storage
                }
            }
        }
    }
}
