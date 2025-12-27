
use serde::{Deserialize, Serialize};
use crate::narrative::events::NarrativeEffect;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEventsConfig {
    #[serde(default)]
    pub hostile: Vec<RelationshipEventTemplate>,
    #[serde(default)]
    pub friendly: Vec<RelationshipEventTemplate>,
    #[serde(default)]
    pub romance: Vec<RelationshipEventTemplate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipEventTemplate {
    pub id: String,
    pub trigger_strength_min: Option<i32>,
    pub trigger_strength_max: Option<i32>,
    pub probability: u32,
    pub headline: String,
    pub description: String,
    #[serde(default)]
    pub choices: Vec<RelationshipChoiceTemplate>,
    #[serde(default)]
    pub default_effect: Option<NarrativeEffect>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipChoiceTemplate {
    pub label: String,
    pub description: String,
    pub effect: NarrativeEffect,
    #[serde(default)]
    pub reputation_change: i32,
}

impl Default for RelationshipEventsConfig {
    fn default() -> Self {
        Self {
            hostile: Vec::new(),
            friendly: Vec::new(),
            romance: Vec::new(),
        }
    }
}

pub fn load_relationship_config() -> RelationshipEventsConfig {
    match std::fs::read_to_string("assets/relationship_events.json") {
        Ok(json) => {
            serde_json::from_str(&json).unwrap_or_else(|e| {
                eprintln!("Failed to parse relationship_events.json: {}", e);
                RelationshipEventsConfig::default()
            })
        }
        Err(e) => {
            eprintln!("Failed to load relationship_events.json: {}", e);
            RelationshipEventsConfig::default()
        }
    }
}
