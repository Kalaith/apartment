//! Upgrade definitions and the UI labels that describe them.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpgradeDefinition {
    pub id: String,
    pub name: String,
    pub cost: i32,
    pub target: UpgradeTarget,
    pub effects: Vec<UpgradeEffect>,
    pub requirements: Vec<UpgradeRequirement>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum UpgradeTarget {
    Apartment,
    Building,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum UpgradeEffect {
    SetFlag(String),
    RemoveFlag(String),
    ModifyStat { stat: String, amount: i32 },
    SetDesign(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum UpgradeRequirement {
    MissingFlag(String),
    HasFlag(String),
    MinStat { stat: String, value: i32 },
    MaxStat { stat: String, value: i32 },
    HasDesign(String),
    MissingDesign(String),
    MinSize(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiConfig {
    pub upgrade_labels: HashMap<String, String>,
}
