//! Presentation tuning: the colour theme, screen layout metrics, and the
//! thresholds the UI uses to label happiness and condition.

use serde::{Deserialize, Serialize};

/// The palette consumed by `crate::ui::theme::color` — this struct (and the
/// `theme` block in `assets/config.json`) is the actual source of truth;
/// `color::NAME()` reads through the active config with these as the
/// compile-time fallback. Keep in sync with the "Bolder restyle" palette.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub background: [f32; 4],
    pub surface: [f32; 4],
    pub surface_alt: [f32; 4],
    pub surface_header: [f32; 4],
    pub border: [f32; 4],
    pub border_strong: [f32; 4],
    pub text: [f32; 4],
    pub text_bright: [f32; 4],
    pub text_dim: [f32; 4],
    pub primary: [f32; 4],
    pub primary_hover: [f32; 4],
    pub primary_pressed: [f32; 4],
    pub accent: [f32; 4],
    pub positive: [f32; 4],
    pub warning: [f32; 4],
    pub negative: [f32; 4],
    pub vacant: [f32; 4],
    pub occupied: [f32; 4],
    pub selected: [f32; 4],
    pub hovered: [f32; 4],
    pub student: [f32; 4],
    pub professional: [f32; 4],
    pub artist: [f32; 4],
    pub family: [f32; 4],
    pub elderly: [f32; 4],
    pub shadow: [f32; 4],
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background: [0.07, 0.08, 0.11, 1.0],
            surface: [0.12, 0.13, 0.17, 1.0],
            surface_alt: [0.16, 0.17, 0.22, 1.0],
            surface_header: [0.10, 0.11, 0.15, 1.0],
            border: [0.24, 0.26, 0.33, 1.0],
            border_strong: [0.38, 0.41, 0.50, 1.0],
            text: [0.90, 0.91, 0.94, 1.0],
            text_bright: [1.0, 1.0, 1.0, 1.0],
            text_dim: [0.58, 0.61, 0.69, 1.0],
            primary: [0.96, 0.70, 0.30, 1.0],
            primary_hover: [1.0, 0.78, 0.42, 1.0],
            primary_pressed: [0.78, 0.55, 0.20, 1.0],
            accent: [0.35, 0.78, 0.82, 1.0],
            positive: [0.42, 0.80, 0.48, 1.0],
            warning: [0.95, 0.68, 0.25, 1.0],
            negative: [0.92, 0.36, 0.38, 1.0],
            vacant: [0.20, 0.21, 0.26, 1.0],
            occupied: [0.18, 0.28, 0.34, 1.0],
            selected: [0.30, 0.44, 0.52, 1.0],
            hovered: [0.24, 0.30, 0.40, 1.0],
            student: [0.90, 0.60, 0.35, 1.0],
            professional: [0.40, 0.60, 0.92, 1.0],
            artist: [0.82, 0.42, 0.78, 1.0],
            family: [0.45, 0.82, 0.50, 1.0],
            elderly: [0.72, 0.74, 0.80, 1.0],
            shadow: [0.0, 0.0, 0.0, 0.35],
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
