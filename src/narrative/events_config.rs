
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TenantEventsConfig {
    pub requests: HashMap<String, Vec<RequestTemplate>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RequestTemplate {
    Pet { options: Vec<String>, weight: u32 },
    Sublease { weight: u32 },
    HomeBusiness { options: Vec<String>, weight: u32 },
    Modification { options: Vec<String>, weight: u32 },
    TemporaryGuest { options: Vec<String>, duration_min: u32, duration_max: u32, weight: u32 },
    None { weight: u32 },
}

impl Default for TenantEventsConfig {
    fn default() -> Self {
        Self {
            requests: HashMap::new(),
        }
    }
}

pub fn load_events_config() -> TenantEventsConfig {
    match std::fs::read_to_string("assets/tenant_events.json") {
        Ok(json) => {
            serde_json::from_str(&json).unwrap_or_else(|e| {
                eprintln!("Failed to parse tenant_events.json: {}", e);
                TenantEventsConfig::default()
            })
        }
        Err(e) => {
            eprintln!("Failed to load tenant_events.json: {}", e);
            TenantEventsConfig::default()
        }
    }
}
