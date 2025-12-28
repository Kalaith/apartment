
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    pub version: String,
    pub starting_conditions: StartingConditions,
    pub economy: EconomyConfig,
    pub decay: DecayConfig,
    pub happiness: HappinessConfig,
    pub win_conditions: WinConditions,
    pub applications: ApplicationConfig,
    pub ui: UiConfig,
    #[serde(default)]
    pub upgrades: HashMap<String, UpgradeDefinition>,
    #[serde(default)]
    pub matching: MatchingConfig,
    #[serde(default)]
    pub thresholds: ThresholdsConfig,
    #[serde(default)]
    pub operating_costs: OperatingCostsConfig,
    #[serde(default)]
    pub vetting: VettingConfig,
    #[serde(default)]
    pub marketing: MarketingConfig,
    #[serde(default)]
    pub relationships: RelationshipsConfig,
    #[serde(default)]
    pub cohesion: CohesionConfig,
    #[serde(default)]
    pub gentrification: GentrificationConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub layout: LayoutConfig,
    #[serde(default)]
    pub ui_thresholds: UiThresholdsConfig,
}

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StartingConditions {
    pub player_money: i32,
    pub starting_tenants: i32,
    pub building_floors: u32,
    pub units_per_floor: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EconomyConfig {
    pub repair_cost_per_point: i32,
    pub hallway_repair_cost_per_point: i32,
    pub design_upgrade_costs: HashMap<String, i32>,
    pub kitchen_renovation_cost: i32,
    pub laundry_installation_cost: i32,
    pub soundproofing_cost: i32,
    pub base_rent: HashMap<String, i32>,
    #[serde(default)]
    pub staff_costs: HashMap<String, i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DecayConfig {
    pub apartment_per_tick: i32,
    pub hallway_per_tick: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HappinessConfig {
    pub base: i32,
    pub min_for_victory: i32,
    pub leave_threshold: i32,
    pub unhappy_threshold: i32,
    pub tenure_bonus_max: i32,

    // Rent
    pub rent_bonus_multiplier: f32,
    pub rent_bonus_cap: i32,
    pub rent_penalty_multiplier: f32,
    pub rent_penalty_cap: i32,

    // Condition
    pub condition_bonus_multiplier: f32,
    pub condition_bonus_cap: i32,
    pub condition_penalty_multiplier: f32,
    pub condition_penalty_cap: i32,

    // Noise
    pub noise_quiet_bonus: f32,
    pub noise_high_penalty_base: i32,
    pub noise_tolerance_multiplier: f32,

    // Design
    pub design_preferred_bonus: i32,
    pub design_hated_penalty: i32,
    pub design_style_modifiers: HashMap<String, i32>,
    
    // Hallway
    pub hallway_condition_base: i32,
    pub hallway_condition_multiplier: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WinConditions {
    pub full_occupancy_required: bool,
    pub min_ticks_for_victory: u32,
    /// Game duration in ticks (months). After this many ticks, the game ends with a score.
    /// Defaults to 36 (3 years) if not specified.
    #[serde(default)]
    pub game_duration_ticks: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub expire_after_ticks: u32,
    pub base_per_vacancy: f32,
    pub appeal_bonus_divisor: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MatchingConfig {
    pub base_score: i32,
    pub desperate_penalty: i32,
    pub rent_great_threshold: i32,
    pub rent_great_bonus: i32,
    pub rent_fair_bonus: i32,
    pub rent_slight_penalty: i32,
    pub rent_unaffordable_penalty: i32,
    pub condition_excellent_threshold: i32,
    pub condition_excellent_bonus: i32,
    pub condition_good_threshold: i32,
    pub condition_good_bonus: i32,
    pub condition_poor_threshold: i32,
    pub condition_poor_penalty: i32,
    pub noise_quiet_bonus: i32,
    pub noise_loud_penalty: i32,
    pub design_preferred_bonus: i32,
    pub size_medium_bonus: i32,
    pub lease_defaults: LeaseDefaultsConfig,
    pub lease_acceptance: LeaseAcceptanceConfig,
}

impl Default for MatchingConfig {
    fn default() -> Self {
        Self {
            base_score: 50,
            desperate_penalty: -40,
            rent_great_threshold: 200,
            rent_great_bonus: 15,
            rent_fair_bonus: 8,
            rent_slight_penalty: -5,
            rent_unaffordable_penalty: -20,
            condition_excellent_threshold: 80,
            condition_excellent_bonus: 15,
            condition_good_threshold: 60,
            condition_good_bonus: 8,
            condition_poor_threshold: 50,
            condition_poor_penalty: 10,
            noise_quiet_bonus: 12,
            noise_loud_penalty: 15,
            design_preferred_bonus: 18,
            size_medium_bonus: 5,
            lease_defaults: LeaseDefaultsConfig::default(),
            lease_acceptance: LeaseAcceptanceConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseDefaultsConfig {
    pub security_deposit_months: u32,
    pub lease_duration_months: u32,
    pub cleaning_fee: i32,
}

impl Default for LeaseDefaultsConfig {
    fn default() -> Self {
        Self {
            security_deposit_months: 1,
            lease_duration_months: 12,
            cleaning_fee: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeaseAcceptanceConfig {
    pub deposit_2_month_penalty: f32,
    pub deposit_3_month_penalty: f32,
    pub short_lease_bonus: f32,
    pub long_lease_penalty: f32,
    pub good_deal_bonus: f32,
    pub expensive_penalty: f32,
}

impl Default for LeaseAcceptanceConfig {
    fn default() -> Self {
        Self {
            deposit_2_month_penalty: 0.15,
            deposit_3_month_penalty: 0.35,
            short_lease_bonus: 0.1,
            long_lease_penalty: 0.15,
            good_deal_bonus: 0.1,
            expensive_penalty: 0.1,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThresholdsConfig {
    pub poor_condition: i32,
    pub critical_condition: i32,
    pub all_left_check_tick: u32,
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            poor_condition: 40,
            critical_condition: 20,
            all_left_check_tick: 3,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperatingCostsConfig {
    pub property_tax_rate: f32,
    pub utility_cost_per_unit: i32,
    pub insurance_base_rate: i32,
    pub insurance_good_condition_discount: i32,
    pub insurance_good_condition_threshold: i32,
}

impl Default for OperatingCostsConfig {
    fn default() -> Self {
        Self {
            property_tax_rate: 0.10,
            utility_cost_per_unit: 50,
            insurance_base_rate: 150,
            insurance_good_condition_discount: 50,
            insurance_good_condition_threshold: 80,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VettingConfig {
    pub credit_check_cost: i32,
    pub background_check_cost: i32,
    pub credit_thresholds: VettingThresholds,
    pub behavior_thresholds: VettingThresholds,
}

impl Default for VettingConfig {
    fn default() -> Self {
        Self {
            credit_check_cost: 25,
            background_check_cost: 10,
            credit_thresholds: VettingThresholds::default(),
            behavior_thresholds: VettingThresholds::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VettingThresholds {
    pub excellent: i32,
    pub good: i32,
    pub average: i32,
    pub below_average: i32,
}

impl Default for VettingThresholds {
    fn default() -> Self {
        Self {
            excellent: 90,
            good: 75,
            average: 60,
            below_average: 40,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketingConfig {
    pub none_cost: i32,
    pub social_media_cost: i32,
    pub local_newspaper_cost: i32,
    pub premium_agency_cost: i32,
}

impl Default for MarketingConfig {
    fn default() -> Self {
        Self {
            none_cost: 0,
            social_media_cost: 50,
            local_newspaper_cost: 150,
            premium_agency_cost: 500,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RelationshipsConfig {
    pub happiness_modifiers: HashMap<String, i32>,
    pub formation_chance: i32,
    pub hostile_cooldown_chance: i32,
    pub hostile_strength_decay: i32,
    pub hostile_transition_threshold: i32,
    pub same_archetype_friendly_chance: i32,
    pub adjacent_hostile_chance: i32,
}

impl Default for RelationshipsConfig {
    fn default() -> Self {
        let mut happiness_modifiers = HashMap::new();
        happiness_modifiers.insert("friendly".to_string(), 5);
        happiness_modifiers.insert("neutral".to_string(), 0);
        happiness_modifiers.insert("hostile".to_string(), -10);
        happiness_modifiers.insert("romantic".to_string(), 8);
        happiness_modifiers.insert("family".to_string(), 10);
        
        Self {
            happiness_modifiers,
            formation_chance: 5,
            hostile_cooldown_chance: 5,
            hostile_strength_decay: 5,
            hostile_transition_threshold: 20,
            same_archetype_friendly_chance: 60,
            adjacent_hostile_chance: 30,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CohesionConfig {
    pub archetype_group_threshold: i32,
    pub archetype_group_base_bonus: i32,
    pub archetype_group_per_extra: i32,
    pub friendly_relationship_bonus: i32,
    pub hostile_relationship_penalty: i32,
    pub tension_penalty: i32,
    pub cohesion_min: i32,
    pub cohesion_max: i32,
}

impl Default for CohesionConfig {
    fn default() -> Self {
        Self {
            archetype_group_threshold: 3,
            archetype_group_base_bonus: 5,
            archetype_group_per_extra: 2,
            friendly_relationship_bonus: 2,
            hostile_relationship_penalty: 5,
            tension_penalty: 8,
            cohesion_min: -50,
            cohesion_max: 50,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GentrificationConfig {
    pub affordable_threshold: i32,
    pub rent_increase_threshold_percent: i32,
    pub rent_increase_score_divisor: i32,
    pub max_gentrification_score: i32,
    pub council_formation_threshold: f32,
    pub council_min_tenants: usize,
}

impl Default for GentrificationConfig {
    fn default() -> Self {
        Self {
            affordable_threshold: 700,
            rent_increase_threshold_percent: 10,
            rent_increase_score_divisor: 5,
            max_gentrification_score: 100,
            council_formation_threshold: 0.4,
            council_min_tenants: 4,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background: [f32; 4],
    pub panel: [f32; 4],
    pub panel_header: [f32; 4],
    pub text: [f32; 4],
    pub text_bright: [f32; 4],
    pub text_dim: [f32; 4],
    pub accent: [f32; 4],
    pub positive: [f32; 4],
    pub warning: [f32; 4],
    pub negative: [f32; 4],
    pub vacant: [f32; 4],
    pub occupied: [f32; 4],
    pub selected: [f32; 4],
    pub hovered: [f32; 4],
    pub archetype_student: [f32; 4],
    pub archetype_professional: [f32; 4],
    pub archetype_artist: [f32; 4],
    pub archetype_family: [f32; 4],
    pub archetype_elderly: [f32; 4],
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: [0.12, 0.12, 0.14, 1.0],
            panel: [0.18, 0.18, 0.22, 1.0],
            panel_header: [0.22, 0.22, 0.28, 1.0],
            text: [0.9, 0.9, 0.9, 1.0],
            text_bright: [1.0, 1.0, 1.0, 1.0],
            text_dim: [0.6, 0.6, 0.6, 1.0],
            accent: [0.3, 0.6, 0.9, 1.0],
            positive: [0.3, 0.8, 0.4, 1.0],
            warning: [0.9, 0.7, 0.2, 1.0],
            negative: [0.9, 0.3, 0.3, 1.0],
            vacant: [0.3, 0.3, 0.35, 1.0],
            occupied: [0.25, 0.35, 0.45, 1.0],
            selected: [0.35, 0.5, 0.7, 1.0],
            hovered: [0.3, 0.4, 0.55, 1.0],
            archetype_student: [0.8, 0.5, 0.3, 1.0],
            archetype_professional: [0.3, 0.5, 0.8, 1.0],
            archetype_artist: [0.8, 0.3, 0.7, 1.0],
            archetype_family: [0.4, 0.8, 0.4, 1.0],
            archetype_elderly: [0.7, 0.7, 0.7, 1.0],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutConfig {
    pub header_height: f32,
    pub footer_height: f32,
    pub panel_split: f32,
    pub padding: f32,
    pub unit_width: f32,
    pub unit_height: f32,
    pub unit_gap: f32,
    pub floor_height: f32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            header_height: 60.0,
            footer_height: 100.0,
            panel_split: 0.6,
            padding: 10.0,
            unit_width: 120.0,
            unit_height: 80.0,
            unit_gap: 15.0,
            floor_height: 100.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiThresholdsConfig {
    pub happiness_ecstatic: i32,
    pub happiness_happy: i32,
    pub happiness_neutral: i32,
    pub happiness_unhappy: i32,
    pub condition_good: i32,
    pub condition_fair: i32,
    pub condition_poor: i32,
}

impl Default for UiThresholdsConfig {
    fn default() -> Self {
        Self {
            happiness_ecstatic: 85,
            happiness_happy: 70,
            happiness_neutral: 50,
            happiness_unhappy: 30,
            condition_good: 80,
            condition_fair: 50,
            condition_poor: 30,
        }
    }
}

impl Default for GameConfig {
    fn default() -> Self {
        // Hardcoded fallback if JSON fails to load
        Self {
            version: "0.1.0".to_string(),
            starting_conditions: StartingConditions {
                player_money: 5000,
                starting_tenants: 1,
                building_floors: 3,
                units_per_floor: 2,
            },
            economy: EconomyConfig {
                repair_cost_per_point: 10,
                hallway_repair_cost_per_point: 15,
                design_upgrade_costs: {
                    let mut m = HashMap::new();
                    m.insert("bare_to_practical".to_string(), 500);
                    m.insert("practical_to_cozy".to_string(), 1000);
                    m.insert("practical_to_cozy".to_string(), 1000);
                    m
                },
                kitchen_renovation_cost: 800,
                laundry_installation_cost: 2000,
                soundproofing_cost: 300,
                base_rent: {
                    let mut m = HashMap::new();
                    m.insert("small".to_string(), 600);
                    m.insert("medium".to_string(), 900);
                    m
                },
                staff_costs: {
                    let mut m = HashMap::new();
                    m.insert("janitor".to_string(), 200);
                    m.insert("security".to_string(), 400);
                    m.insert("manager".to_string(), 600);
                    m
                },
            },
            decay: DecayConfig {
                apartment_per_tick: 2,
                hallway_per_tick: 1,
            },
            happiness: HappinessConfig {
                base: 50,
                min_for_victory: 60,
                leave_threshold: 0,
                unhappy_threshold: 30,
                tenure_bonus_max: 12,
                rent_bonus_multiplier: 0.02,
                rent_bonus_cap: 15,
                rent_penalty_multiplier: 0.05,
                rent_penalty_cap: -30,
                condition_bonus_multiplier: 0.3,
                condition_bonus_cap: 20,
                condition_penalty_multiplier: 0.5,
                condition_penalty_cap: 40,
                noise_quiet_bonus: 10.0,
                noise_high_penalty_base: -25,
                noise_tolerance_multiplier: 0.3,
                design_preferred_bonus: 20,
                design_hated_penalty: -25,
                design_style_modifiers: {
                    let mut m = HashMap::new();
                    m.insert("Bare".to_string(), -5);
                    m.insert("Practical".to_string(), 5);
                    m.insert("Cozy".to_string(), 10);
                    m
                },
                hallway_condition_base: 50,
                hallway_condition_multiplier: 0.1,
            },
            win_conditions: WinConditions {
                full_occupancy_required: true,
                min_ticks_for_victory: 6,
                game_duration_ticks: Some(36), // 3 years
            },
            applications: ApplicationConfig {
                expire_after_ticks: 3,
                base_per_vacancy: 0.5,
                appeal_bonus_divisor: 50,
            },
            ui: UiConfig {
                upgrade_labels: {
                    let mut m = HashMap::new();
                    m.insert("repair_fmt".to_string(), "Repair +{}".to_string());
                    m.insert("repair_hallway_fmt".to_string(), "Repair Hallway +{}".to_string());
                    m.insert("upgrade_design_fmt".to_string(), "Upgrade to {}".to_string());
                    m.insert("max_design".to_string(), "Max Design".to_string());
                    m.insert("soundproofing".to_string(), "Add Soundproofing".to_string());
                    m.insert("kitchen_renovation".to_string(), "Renovate Kitchen".to_string());
                    m.insert("install_laundry".to_string(), "Install Laundry".to_string());
                    m
                }
            },
            upgrades: HashMap::new(),
            matching: MatchingConfig::default(),
            thresholds: ThresholdsConfig::default(),
            operating_costs: OperatingCostsConfig::default(),
            vetting: VettingConfig::default(),
            marketing: MarketingConfig::default(),
            relationships: RelationshipsConfig::default(),
            cohesion: CohesionConfig::default(),
            gentrification: GentrificationConfig::default(),
            theme: ThemeConfig::default(),
            layout: LayoutConfig::default(),
            ui_thresholds: UiThresholdsConfig::default(),
        }
    }
}

pub fn load_config() -> GameConfig {
    // For WASM, embed configs at compile time
    #[cfg(target_arch = "wasm32")]
    let config_json = include_str!("../../assets/config.json");
    
    #[cfg(not(target_arch = "wasm32"))]
    let config_json = std::fs::read_to_string("assets/config.json")
        .unwrap_or_else(|_| include_str!("../../assets/config.json").to_string());
    
    let mut config: GameConfig = serde_json::from_str(&config_json).unwrap_or_else(|e| {
        eprintln!("Failed to parse config.json: {}", e);
        GameConfig::default()
    });
    
    // Load upgrades from separate file
    #[cfg(target_arch = "wasm32")]
    let upgrades_json = include_str!("../../assets/upgrades.json");
    
    #[cfg(not(target_arch = "wasm32"))]
    let upgrades_json = std::fs::read_to_string("assets/upgrades.json")
        .unwrap_or_else(|_| include_str!("../../assets/upgrades.json").to_string());
    
    if let Ok(upgrades) = serde_json::from_str::<HashMap<String, UpgradeDefinition>>(&upgrades_json) {
        config.upgrades = upgrades;
    }
    
    config
}
