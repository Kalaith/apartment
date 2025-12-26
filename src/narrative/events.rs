
use serde::{Deserialize, Serialize};
use macroquad::rand::{ChooseRandom, gen_range};

/// Types of narrative events
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum NarrativeEventType {
    /// News about the neighborhood
    NeighborhoodNews,
    /// City-wide event affecting economy
    CityEvent,
    /// Tenant-specific story beat
    TenantStory { tenant_id: u32 },
    /// Building milestone
    BuildingMilestone,
    /// Random character encounter
    CharacterEncounter,
    /// External offer (developer wants to buy, investor interest)
    ExternalOffer,
    /// Seasonal event
    SeasonalEvent,
}

/// A narrative event with context and choices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeEvent {
    pub id: u32,
    pub event_type: NarrativeEventType,
    pub month: u32,
    pub headline: String,
    pub description: String,
    /// Optional choices the player can make
    pub choices: Vec<NarrativeChoice>,
    /// Effect if no choice is made (or no choices available)
    pub default_effect: NarrativeEffect,
    /// Has this been seen by the player?
    pub read: bool,
    /// Does this require a response?
    pub requires_response: bool,
    /// Deadline month for response (if applicable)
    pub response_deadline: Option<u32>,
}

/// A choice for a narrative event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeChoice {
    pub label: String,
    pub description: String,
    pub effect: NarrativeEffect,
    pub reputation_change: i32,
}

/// Effects of narrative events/choices
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NarrativeEffect {
    /// No gameplay effect
    None,
    /// Money gained or lost
    Money(i32),
    /// Reputation in neighborhood
    NeighborhoodReputation { neighborhood_id: u32, change: i32 },
    /// Building-wide happiness change
    BuildingHappiness { building_id: u32, change: i32 },
    /// Specific tenant happiness
    TenantHappiness { tenant_id: u32, change: i32 },
    /// Economic change
    EconomyChange { economy_health_change: f32 },
    /// Rent demand change
    RentDemand { neighborhood_id: u32, change: f32 },
    /// Trigger an inspection
    TriggerInspection { building_id: u32 },
    /// Property value change
    PropertyValue { building_id: u32, change_percent: f32 },
    /// Multiple effects
    Multiple(Vec<NarrativeEffect>),
}

impl NarrativeEvent {
    /// Create a simple news event
    pub fn news(id: u32, month: u32, headline: &str, description: &str) -> Self {
        Self {
            id,
            event_type: NarrativeEventType::NeighborhoodNews,
            month,
            headline: headline.to_string(),
            description: description.to_string(),
            choices: Vec::new(),
            default_effect: NarrativeEffect::None,
            read: false,
            requires_response: false,
            response_deadline: None,
        }
    }

    /// Create an event with choices
    pub fn with_choices(
        id: u32,
        event_type: NarrativeEventType,
        month: u32,
        headline: &str,
        description: &str,
        choices: Vec<NarrativeChoice>,
    ) -> Self {
        Self {
            id,
            event_type,
            month,
            headline: headline.to_string(),
            description: description.to_string(),
            choices,
            default_effect: NarrativeEffect::None,
            read: false,
            requires_response: true,
            response_deadline: Some(month + 2), // 2 months to respond
        }
    }



    /// Check if deadline has passed
    pub fn is_expired(&self, current_month: u32) -> bool {
        self.response_deadline.map(|d| current_month > d).unwrap_or(false)
    }
}

/// Manages narrative events
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NarrativeEventSystem {
    pub events: Vec<NarrativeEvent>,
    pub next_event_id: u32,
    /// Events awaiting player response
    pub pending_events: Vec<u32>,
    /// Processed event IDs
    pub processed_events: Vec<u32>,
}

impl NarrativeEventSystem {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_event_id: 0,
            pending_events: Vec::new(),
            processed_events: Vec::new(),
        }
    }

    /// Add a new event
    pub fn add_event(&mut self, mut event: NarrativeEvent) -> u32 {
        let id = self.next_event_id;
        event.id = id;
        self.next_event_id += 1;
        
        if event.requires_response {
            self.pending_events.push(id);
        }
        
        self.events.push(event);
        id
    }



    /// Get pending events requiring response
    pub fn events_requiring_response(&self) -> Vec<&NarrativeEvent> {
        self.pending_events.iter()
            .filter_map(|id| self.events.iter().find(|e| e.id == *id))
            .collect()
    }

    /// Process a choice for an event
    pub fn process_choice(&mut self, event_id: u32, choice_index: usize) -> Option<NarrativeEffect> {
        let event = self.events.iter_mut().find(|e| e.id == event_id)?;
        let effect = event.choices.get(choice_index).map(|c| c.effect.clone())?;
        
        event.read = true;
        self.pending_events.retain(|&id| id != event_id);
        self.processed_events.push(event_id);
        
        Some(effect)
    }

    /// Generate random events based on game state
    pub fn generate_events(
        &mut self,
        month: u32,
        neighborhoods: &[crate::city::Neighborhood],
        buildings: &[crate::building::Building],
        _tenants: &[crate::tenant::Tenant],
    ) {
        // Chance for neighborhood news
        if gen_range(0, 100) < 20 {
            if let Some(neighborhood) = neighborhoods.choose() {
                let event = self.generate_neighborhood_event(month, neighborhood);
                self.add_event(event);
            }
        }

        // Chance for city-wide event
        if gen_range(0, 100) < 10 {
            let event = self.generate_city_event(month);
            self.add_event(event);
        }

        // Seasonal events
        let season = (month % 12) / 3; // 0=spring, 1=summer, 2=fall, 3=winter
        if gen_range(0, 100) < 15 {
            let event = self.generate_seasonal_event(month, season);
            self.add_event(event);
        }

        // Developer/investor offers (rare)
        if gen_range(0, 100) < 5 && !buildings.is_empty() {
            let building = buildings.choose().unwrap();
            let building_id = buildings.iter().position(|b| std::ptr::eq(b, building)).unwrap_or(0) as u32;
            let event = self.generate_offer_event(month, building_id, building);
            self.add_event(event);
        }

        // Building milestones
        for (_i, building) in buildings.iter().enumerate() {
            if building.occupancy_count() == building.apartments.len() && gen_range(0, 100) < 30 {
                let event = NarrativeEvent::news(
                    0, month,
                    &format!("{} Achieves Full Occupancy!", building.name),
                    "All units are now occupied. Your reputation is growing.",
                );
                self.add_event(event);
            }
        }

        // Process expired events
        self.handle_expired_events(month);
    }

    fn generate_neighborhood_event(&self, month: u32, neighborhood: &crate::city::Neighborhood) -> NarrativeEvent {
        let templates: Vec<(&str, &str, NarrativeEffect)> = vec![
            (
                "New Business Opens",
                "A new café has opened in the neighborhood, adding to local charm.",
                NarrativeEffect::NeighborhoodReputation { 
                    neighborhood_id: neighborhood.id, 
                    change: 5 
                },
            ),
            (
                "Street Improvements",
                "The city has announced road improvements for the area.",
                NarrativeEffect::RentDemand { 
                    neighborhood_id: neighborhood.id, 
                    change: 0.05 
                },
            ),
            (
                "Crime Report",
                "Local news reports a slight uptick in property crime.",
                NarrativeEffect::NeighborhoodReputation { 
                    neighborhood_id: neighborhood.id, 
                    change: -3 
                },
            ),
            (
                "Community Festival",
                "The annual neighborhood festival brought residents together.",
                NarrativeEffect::NeighborhoodReputation { 
                    neighborhood_id: neighborhood.id, 
                    change: 3 
                },
            ),
        ];

        let (headline, description, effect) = templates.choose().cloned().unwrap();
        let mut event = NarrativeEvent::news(0, month, headline, description);
        event.default_effect = effect;
        event
    }

    fn generate_city_event(&self, month: u32) -> NarrativeEvent {
        let templates: Vec<(&str, &str, NarrativeEffect)> = vec![
            (
                "Housing Market Heats Up",
                "Analysts report increased demand for rental properties citywide.",
                NarrativeEffect::EconomyChange { economy_health_change: 0.05 },
            ),
            (
                "Economic Concerns",
                "Business leaders express worry about the local economy.",
                NarrativeEffect::EconomyChange { economy_health_change: -0.05 },
            ),
            (
                "New Transit Line Announced",
                "The city will expand public transit, improving access across neighborhoods.",
                NarrativeEffect::None,
            ),
            (
                "Property Tax Review",
                "City council is reviewing property tax rates.",
                NarrativeEffect::None,
            ),
        ];

        let (headline, description, effect) = templates.choose().cloned().unwrap();
        let mut event = NarrativeEvent::news(0, month, headline, description);
        event.event_type = NarrativeEventType::CityEvent;
        event.default_effect = effect;
        event
    }

    fn generate_seasonal_event(&self, month: u32, season: u32) -> NarrativeEvent {
        let (headline, description) = match season {
            0 => ("Spring Cleaning Season", "Tenants report increased satisfaction with well-maintained properties."),
            1 => ("Summer Heat Wave Warning", "Hot weather expected. Tenants are asking about AC."),
            2 => ("Back to School Rush", "Students are actively hunting for housing near universities."),
            _ => ("Winter Preparedness", "Cold weather approaching. Heating systems should be checked."),
        };

        let mut event = NarrativeEvent::news(0, month, headline, description);
        event.event_type = NarrativeEventType::SeasonalEvent;
        event
    }

    fn generate_offer_event(&self, month: u32, _building_id: u32, building: &crate::building::Building) -> NarrativeEvent {
        let base_value = 50000 * building.apartments.len() as i32;
        let offer = (base_value as f32 * gen_range(0.9, 1.3)) as i32;

        NarrativeEvent::with_choices(
            0,
            NarrativeEventType::ExternalOffer,
            month,
            "Developer Makes Offer",
            &format!(
                "A developer has expressed interest in purchasing {} for ${}.",
                building.name, offer
            ),
            vec![
                NarrativeChoice {
                    label: "Accept Offer".to_string(),
                    description: format!("Sell the building for ${}", offer),
                    effect: NarrativeEffect::Money(offer),
                    reputation_change: -20,
                },
                NarrativeChoice {
                    label: "Counter Offer".to_string(),
                    description: "Negotiate for a better price".to_string(),
                    effect: NarrativeEffect::None, // Would trigger follow-up event
                    reputation_change: 0,
                },
                NarrativeChoice {
                    label: "Decline".to_string(),
                    description: "This building is not for sale".to_string(),
                    effect: NarrativeEffect::None,
                    reputation_change: 5,
                },
            ],
        )
    }

    fn handle_expired_events(&mut self, current_month: u32) {
        let expired: Vec<u32> = self.events.iter()
            .filter(|e| e.is_expired(current_month) && !self.processed_events.contains(&e.id))
            .map(|e| e.id)
            .collect();

        for id in expired {
            if let Some(event) = self.events.iter_mut().find(|e| e.id == id) {
                event.read = true;
            }
            self.pending_events.retain(|&pid| pid != id);
            self.processed_events.push(id);
        }
    }


}

impl Default for NarrativeEventSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let event = NarrativeEvent::news(0, 1, "Test", "Description");
        assert!(!event.read);
        assert!(!event.requires_response);
    }

    #[test]
    fn test_event_system() {
        let mut system = NarrativeEventSystem::new();
        let _id = system.add_event(NarrativeEvent::news(0, 1, "Test", "Desc"));
        assert_eq!(system.events.len(), 1);
    }
}
