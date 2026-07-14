//! Presentation tuning: the colour theme, screen layout metrics, and the
//! thresholds the UI uses to label happiness and condition.

use serde::{Deserialize, Serialize};

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
