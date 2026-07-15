use super::GameplayState;
use crate::narrative::{ActiveTaxBreak, MissionGoal, MissionReward, MissionStatus};
use crate::simulation::GameEvent;
use crate::ui::colors;
use macroquad::prelude::*;

/// System for handling mission updates and rewards
pub fn update_missions(state: &mut GameplayState) {
    let current_month = state.current_tick;

    // Snapshot this month's building-wide signals up front so per-mission
    // evaluation can read them without borrowing conflicts.
    let avg_happiness = if state.tenants.is_empty() {
        0.0
    } else {
        state
            .tenants
            .iter()
            .map(|t| t.happiness as f32)
            .sum::<f32>()
            / state.tenants.len() as f32
    };
    // "Perfect collection" = at least one tenant and no missed-rent event this
    // month.
    let perfect_collection = !state.tenants.is_empty()
        && state.last_tick_result.as_ref().is_some_and(|r| {
            !r.events
                .iter()
                .any(|e| matches!(e, GameEvent::RentMissed { .. }))
        });
    let building_fully_repaired = !state.building.apartments.is_empty()
        && state.building.apartments.iter().all(|a| a.condition >= 90)
        && state.building.hallway_condition >= 90;

    // Check for expirations (expired missions are marked as such)
    state.missions.check_expirations(current_month);

    // Check for unrecoverable failures (e.g., building sold that was needed)
    for mission in &mut state.missions.missions {
        if mission.status == MissionStatus::Active {
            // Check if "AcquireBuilding" mission should fail if we sold a building
            if matches!(mission.goal, MissionGoal::AcquireBuilding)
                && state.city.buildings.is_empty()
            {
                mission.fail();
                state.floating_texts.spawn(
                    "Mission Failed!",
                    vec2(screen_width() / 2.0, screen_height() / 2.0),
                    colors::NEGATIVE(),
                );
            }
        }
    }

    // Check active missions for completion
    let active_mission_ids: Vec<u32> = state
        .missions
        .active_missions()
        .iter()
        .map(|m| m.id)
        .collect();

    for mission_id in active_mission_ids {
        let mut completed = false;
        let mut reward = None;
        let mut legacy_info: Option<(String, String)> = None;

        if let Some(mission) = state
            .missions
            .missions
            .iter_mut()
            .find(|m| m.id == mission_id)
        {
            match &mut mission.goal {
                MissionGoal::HouseTenants { count, archetype } => {
                    let current_count = state
                        .tenants
                        .iter()
                        .filter(|t| {
                            archetype
                                .as_ref()
                                .is_none_or(|arch| t.archetype.name() == arch)
                        })
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
                    if state.city.buildings.len() > 1 {
                        // Started with 1.
                        completed = true;
                    }
                }
                MissionGoal::MaintainHappiness {
                    threshold,
                    months,
                    current_months,
                } => {
                    // Accrue consecutive months at/above the happiness threshold;
                    // a bad month resets the streak.
                    if !state.tenants.is_empty() && avg_happiness >= *threshold {
                        *current_months += 1;
                    } else {
                        *current_months = 0;
                    }
                    if *current_months >= *months {
                        completed = true;
                    }
                }
                MissionGoal::PerfectCollection {
                    months,
                    current_months,
                } => {
                    if perfect_collection {
                        *current_months += 1;
                    } else {
                        *current_months = 0;
                    }
                    if *current_months >= *months {
                        completed = true;
                    }
                }
                MissionGoal::FullRepair { building_id: _ } => {
                    if building_fully_repaired {
                        completed = true;
                    }
                }
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
                &format!("Completed objective: {}", description),
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
                        current_month,
                    );
                    state.funds.add_income(t);

                    state.floating_texts.spawn(
                        format!("+${}", amount),
                        vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                        colors::POSITIVE(),
                    );
                }
                MissionReward::UnlockBuilding(unlock_order) => {
                    state.unlock_building_by_order(unlock_order);
                    state.floating_texts.spawn(
                        "New property unlocked!",
                        vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                        colors::ACCENT(),
                    );
                }
                MissionReward::Reputation(amount) => {
                    // Reward reputation in the active building's neighborhood.
                    state.apply_reputation_change(amount, None);
                }
                MissionReward::TaxBreak { months, percentage } => {
                    state
                        .active_tax_breaks
                        .push(ActiveTaxBreak::new(months, percentage));
                    state.floating_texts.spawn(
                        format!(
                            "Tax Break! {}% for {} months",
                            (percentage * 100.0) as i32,
                            months
                        ),
                        vec2(screen_width() / 2.0, screen_height() / 2.0 + 30.0),
                        colors::POSITIVE(),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::narrative::missions::Mission;
    use crate::state::GameplayState;

    #[test]
    fn maintain_happiness_goal_accrues_a_month() {
        let mut state = GameplayState::new();
        if state.tenants.is_empty() {
            return;
        }
        for tenant in &mut state.tenants {
            tenant.happiness = 90;
        }
        let id = state.missions.add_mission(Mission::new(
            0,
            "Steady Ship",
            "Keep tenants content.",
            0,
            MissionGoal::MaintainHappiness {
                threshold: 50.0,
                months: 3,
                current_months: 0,
            },
            MissionReward::Money(100),
            None,
        ));
        state.missions.accept_mission(id, 1);

        update_missions(&mut state);

        let mission = state.missions.missions.iter().find(|m| m.id == id).unwrap();
        assert!(matches!(
            mission.goal,
            MissionGoal::MaintainHappiness {
                current_months: 1,
                ..
            }
        ));
        // One month of three: still in progress, no reward granted yet.
        assert_eq!(mission.status, MissionStatus::Active);
    }

    #[test]
    fn full_repair_goal_stays_incomplete_for_a_neglected_building() {
        let mut state = GameplayState::new();
        // Drive the building below the repair bar so completion (and its UI
        // feedback, which needs a GL context) can't fire in the test.
        for apt in &mut state.building.apartments {
            apt.condition = 40;
        }
        state.building.hallway_condition = 40;
        let id = state.missions.add_mission(Mission::new(
            0,
            "Fix It Up",
            "Restore the building.",
            0,
            MissionGoal::FullRepair { building_id: 0 },
            MissionReward::Money(100),
            None,
        ));
        state.missions.accept_mission(id, 1);

        update_missions(&mut state);

        let mission = state.missions.missions.iter().find(|m| m.id == id).unwrap();
        assert_eq!(mission.status, MissionStatus::Active);
    }
}
