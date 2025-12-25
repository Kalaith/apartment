#![allow(dead_code)]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTemplateFile {
    pub templates: Vec<BuildingTemplate>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingTemplate {
    pub id: String,
    pub name: String,
    pub floors: u32,
    pub units_per_floor: u32,
    pub hallway_condition: i32,
    pub apartments: Vec<ApartmentTemplate>,
    pub initial_tenant: Option<InitialTenant>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApartmentTemplate {
    pub unit_number: String,
    pub floor: u32,
    pub size: String,         // "small" or "medium"
    pub base_noise: String,   // "low" or "high"
    pub initial_condition: i32,
    pub initial_design: String,  // "bare", "practical", "cozy"
    pub initial_rent: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InitialTenant {
    pub apartment_unit: String,
    pub archetype: String,
    pub name: String,
}

pub fn load_building_templates() -> Vec<BuildingTemplate> {
    match std::fs::read_to_string("assets/building_templates.json") {
        Ok(json) => {
            match serde_json::from_str::<BuildingTemplateFile>(&json) {
                Ok(file) => file.templates,
                Err(e) => {
                    eprintln!("Failed to parse building_templates.json: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load building_templates.json: {}", e);
            Vec::new()
        }
    }
}

pub fn get_template(id: &str) -> Option<BuildingTemplate> {
    load_building_templates().into_iter().find(|t| t.id == id)
}
