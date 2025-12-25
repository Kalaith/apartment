use crate::building::Building;
use crate::economy::PlayerFunds;
use crate::simulation::events::GameEvent;
use macroquad::rand::gen_range;

pub struct EventSystem {
    // We could track cooldowns here if needed
}

impl EventSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_events(
        &mut self, 
        building: &mut Building, 
        funds: &mut PlayerFunds, 
        _current_tick: u32
    ) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // 1. Heatwave (2% chance)
        // Note: Actual gameplay effects (happiness decay) would be handled by checking active events,
        // but for this MVP scope we'll treat it as a flavor event or immediate effect if possible.
        // To properly implement duration effects, we'd need to store "ActiveEvents" in the GameState.
        // For now, let's just emit the event.
        if gen_range(0, 100) < 2 {
            events.push(GameEvent::Heatwave { tick_duration: 3 });
        }

        // 2. Pipe Burst (3% chance per tick to happen in ONE apartment)
        if gen_range(0, 100) < 3 {
            let num_apts = building.apartments.len();
            if num_apts > 0 {
                let idx = gen_range(0, num_apts);
                if let Some(apt) = building.apartments.get_mut(idx) {
                    let damage = 30;
                    let old_condition = apt.condition;
                    apt.condition = (apt.condition - damage).max(0);
                    
                    // Only report if it actually did damage or if it's a significant event
                    if old_condition > 0 {
                        events.push(GameEvent::PipeBurst { 
                            apartment_unit: apt.unit_number.clone(), 
                            damage 
                        });
                    }
                }
            }
        }

        // 3. Gentrification (Very rare, 0.5% chance)
        if gen_range(0, 1000) < 5 {
            events.push(GameEvent::Gentrification { 
                tick_duration: 6, 
                effect_desc: "Rent tolerance +20%".to_string() 
            });
        }

        // 4. Inspection (5% chance if avg condition is low)
        let avg_condition = building.building_appeal(); // This includes hallway, roughly maps to condition
        // If appeal is low (< 40), higher chance of inspection stuff
        let inspection_chance = if avg_condition < 40 { 5 } else { 1 };
        
        if gen_range(0, 100) < inspection_chance {
            let passed = avg_condition >= 40;
            let fine = if passed { 0 } else { 500 };
            
            if fine > 0 {
                // Deduct fine
                funds.spend(fine); 
            }
            
            events.push(GameEvent::Inspection { 
                result: if passed { "Passed".to_string() } else { "Failed (Building Appeal < 40)".to_string() },
                fine 
            });
        }

        events
    }
}
