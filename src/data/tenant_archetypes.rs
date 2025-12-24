use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeFile {
    pub archetypes: Vec<ArchetypeData>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArchetypeData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub spawn_weight: i32,
    pub preferences: PreferencesData,
    pub name_pool: NamePool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreferencesData {
    pub rent_sensitivity: f32,
    pub condition_sensitivity: f32,
    pub noise_sensitivity: f32,
    pub design_sensitivity: f32,
    pub ideal_rent_max: i32,
    pub min_acceptable_condition: i32,
    pub prefers_quiet: bool,
    pub preferred_design: Option<String>,
    pub hates_design: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NamePool {
    pub first_names: Vec<String>,
    pub last_initials: Vec<String>,
}

pub fn load_archetypes() -> Vec<ArchetypeData> {
    match std::fs::read_to_string("assets/tenant_archetypes.json") {
        Ok(json) => {
            match serde_json::from_str::<ArchetypeFile>(&json) {
                Ok(file) => file.archetypes,
                Err(e) => {
                    eprintln!("Failed to parse tenant_archetypes.json: {}", e);
                    Vec::new()
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load tenant_archetypes.json: {}", e);
            Vec::new()
        }
    }
}

pub fn get_archetype(id: &str) -> Option<ArchetypeData> {
    load_archetypes().into_iter().find(|a| a.id == id)
}
