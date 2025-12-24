# Task 06: UI System

## Priority: 🔵 FINAL PHASE
## Dependencies: All previous tasks (01-05)
## Estimated Effort: 4-5 hours
## Cannot Parallel: Requires all systems functional

---

## Objective
Implement the visual interface using immediate-mode UI patterns with Macroquad. Create building overview, apartment panels, tenant management, and notification system.

---

## UI Philosophy

- **Immediate Mode**: Redraw every frame based on state
- **No Hidden State**: UI reads from game state, emits intents
- **Simple Interactions**: Click, hover, scroll
- **Minimal Graphics**: Icons, bars, text - no complex art

---

## Screen Layout

```
┌────────────────────────────────────────────────────────────────┐
│  HEADER: Money | Month | Building Name | End Turn Button       │
├──────────────────────────────────┬─────────────────────────────┤
│                                  │                             │
│     BUILDING VIEW                │    DETAIL PANEL             │
│     (Left 60%)                   │    (Right 40%)              │
│                                  │                             │
│  ┌─────┐  ┌─────┐               │    [Apartment Details]      │
│  │ 3A  │  │ 3B  │  Floor 3      │         OR                  │
│  └─────┘  └─────┘               │    [Tenant Details]         │
│                                  │         OR                  │
│  ┌─────┐  ┌─────┐               │    [Application List]       │
│  │ 2A  │  │ 2B  │  Floor 2      │                             │
│  └─────┘  └─────┘               │                             │
│                                  │                             │
│  ┌─────┐  ┌─────┐               │                             │
│  │ 1A  │  │ 1B  │  Floor 1      │                             │
│  └─────┘  └─────┘               │                             │
│                                  │                             │
│  [======= HALLWAY =======]       │                             │
│                                  │                             │
├──────────────────────────────────┴─────────────────────────────┤
│  NOTIFICATIONS / EVENT LOG (Bottom strip)                      │
└────────────────────────────────────────────────────────────────┘
```

---

## Deliverables

### 1. src/ui/mod.rs

```rust
mod common;
mod building_view;
mod apartment_panel;
mod tenant_list;
mod application_panel;
mod notifications;
mod header;

pub use common::*;
pub use building_view::draw_building_view;
pub use apartment_panel::draw_apartment_panel;
pub use tenant_list::draw_tenant_list;
pub use application_panel::draw_application_panel;
pub use notifications::draw_notifications;
pub use header::draw_header;

/// What's currently selected for the detail panel
#[derive(Clone, Debug, PartialEq)]
pub enum Selection {
    None,
    Apartment(u32),      // Apartment ID
    Tenant(u32),         // Tenant ID
    Applications,        // Show all pending applications
    Hallway,             // Hallway details
}

/// UI action intents (returned to game logic)
#[derive(Clone, Debug)]
pub enum UiAction {
    SelectApartment(u32),
    SelectTenant(u32),
    SelectApplications,
    SelectHallway,
    ClearSelection,
    
    // Upgrade actions
    RepairApartment { apartment_id: u32, amount: i32 },
    UpgradeDesign { apartment_id: u32 },
    AddSoundproofing { apartment_id: u32 },
    RepairHallway { amount: i32 },
    SetRent { apartment_id: u32, new_rent: i32 },
    
    // Tenant actions
    AcceptApplication { application_index: usize },
    RejectApplication { application_index: usize },
    
    // Game flow
    EndTurn,
    ReturnToMenu,
}
```

### 2. src/ui/common.rs

```rust
use macroquad::prelude::*;

/// Color palette
pub mod colors {
    use macroquad::prelude::Color;
    
    pub const BACKGROUND: Color = Color::new(0.12, 0.12, 0.14, 1.0);
    pub const PANEL: Color = Color::new(0.18, 0.18, 0.22, 1.0);
    pub const PANEL_HEADER: Color = Color::new(0.22, 0.22, 0.28, 1.0);
    pub const TEXT: Color = Color::new(0.9, 0.9, 0.9, 1.0);
    pub const TEXT_DIM: Color = Color::new(0.6, 0.6, 0.6, 1.0);
    pub const ACCENT: Color = Color::new(0.3, 0.6, 0.9, 1.0);
    pub const POSITIVE: Color = Color::new(0.3, 0.8, 0.4, 1.0);
    pub const WARNING: Color = Color::new(0.9, 0.7, 0.2, 1.0);
    pub const NEGATIVE: Color = Color::new(0.9, 0.3, 0.3, 1.0);
    
    pub const VACANT: Color = Color::new(0.3, 0.3, 0.35, 1.0);
    pub const OCCUPIED: Color = Color::new(0.25, 0.35, 0.45, 1.0);
    pub const SELECTED: Color = Color::new(0.35, 0.5, 0.7, 1.0);
    pub const HOVERED: Color = Color::new(0.3, 0.4, 0.55, 1.0);
}

/// Layout constants
pub mod layout {
    pub const HEADER_HEIGHT: f32 = 60.0;
    pub const FOOTER_HEIGHT: f32 = 100.0;
    pub const PANEL_SPLIT: f32 = 0.6;  // Building view takes 60%
    pub const PADDING: f32 = 10.0;
    pub const UNIT_WIDTH: f32 = 120.0;
    pub const UNIT_HEIGHT: f32 = 80.0;
    pub const UNIT_GAP: f32 = 15.0;
    pub const FLOOR_HEIGHT: f32 = 100.0;
}

/// Draw a simple button, returns true if clicked
pub fn button(x: f32, y: f32, w: f32, h: f32, text: &str, enabled: bool) -> bool {
    let mouse = mouse_position();
    let rect = Rect::new(x, y, w, h);
    let hovered = rect.contains(Vec2::new(mouse.0, mouse.1));
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left) && enabled;
    
    let bg_color = if !enabled {
        Color::new(0.2, 0.2, 0.2, 1.0)
    } else if hovered {
        colors::HOVERED
    } else {
        colors::PANEL
    };
    
    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, 2.0, colors::ACCENT);
    
    let text_color = if enabled { colors::TEXT } else { colors::TEXT_DIM };
    let text_size = 20.0;
    let text_width = measure_text(text, None, text_size as u16, 1.0).width;
    draw_text(text, x + (w - text_width) / 2.0, y + h / 2.0 + 6.0, text_size, text_color);
    
    clicked
}

/// Draw a progress bar
pub fn progress_bar(x: f32, y: f32, w: f32, h: f32, value: f32, max: f32, color: Color) {
    let fill_width = (value / max).clamp(0.0, 1.0) * w;
    
    draw_rectangle(x, y, w, h, Color::new(0.15, 0.15, 0.15, 1.0));
    draw_rectangle(x, y, fill_width, h, color);
    draw_rectangle_lines(x, y, w, h, 1.0, colors::TEXT_DIM);
}

/// Draw labeled progress bar
pub fn labeled_bar(x: f32, y: f32, w: f32, label: &str, value: i32, max: i32, color: Color) {
    let h = 16.0;
    draw_text(label, x, y, 16.0, colors::TEXT_DIM);
    progress_bar(x, y + 4.0, w, h, value as f32, max as f32, color);
    
    let value_text = format!("{}/{}", value, max);
    let text_width = measure_text(&value_text, None, 14, 1.0).width;
    draw_text(&value_text, x + w - text_width, y, 14.0, colors::TEXT);
}

/// Draw a panel with header
pub fn panel(x: f32, y: f32, w: f32, h: f32, title: &str) {
    // Background
    draw_rectangle(x, y, w, h, colors::PANEL);
    
    // Header
    draw_rectangle(x, y, w, 30.0, colors::PANEL_HEADER);
    draw_text(title, x + 10.0, y + 22.0, 20.0, colors::TEXT);
    
    // Border
    draw_rectangle_lines(x, y, w, h, 1.0, colors::TEXT_DIM);
}

/// Check if mouse is over a rectangle
pub fn is_hovered(x: f32, y: f32, w: f32, h: f32) -> bool {
    let mouse = mouse_position();
    let rect = Rect::new(x, y, w, h);
    rect.contains(Vec2::new(mouse.0, mouse.1))
}

/// Check if rectangle was clicked
pub fn was_clicked(x: f32, y: f32, w: f32, h: f32) -> bool {
    is_hovered(x, y, w, h) && is_mouse_button_pressed(MouseButton::Left)
}

/// Get color for condition value
pub fn condition_color(condition: i32) -> Color {
    match condition {
        80..=100 => colors::POSITIVE,
        50..=79 => colors::ACCENT,
        30..=49 => colors::WARNING,
        _ => colors::NEGATIVE,
    }
}

/// Get color for happiness value
pub fn happiness_color(happiness: i32) -> Color {
    match happiness {
        70..=100 => colors::POSITIVE,
        40..=69 => colors::ACCENT,
        20..=39 => colors::WARNING,
        _ => colors::NEGATIVE,
    }
}
```

### 3. src/ui/header.rs

```rust
use macroquad::prelude::*;
use super::{common::*, UiAction};

pub fn draw_header(
    money: i32,
    tick: u32,
    building_name: &str,
    occupancy: usize,
    total_units: usize,
) -> Option<UiAction> {
    let mut action = None;
    let w = screen_width();
    
    // Background
    draw_rectangle(0.0, 0.0, w, layout::HEADER_HEIGHT, colors::PANEL_HEADER);
    
    // Building name
    draw_text(building_name, 20.0, 38.0, 28.0, colors::TEXT);
    
    // Money
    let money_text = format!("${}", money);
    let money_color = if money < 500 { colors::WARNING } 
                      else if money < 0 { colors::NEGATIVE }
                      else { colors::POSITIVE };
    draw_text(&money_text, 250.0, 38.0, 24.0, money_color);
    
    // Month
    let month_text = format!("Month {}", tick);
    draw_text(&month_text, 400.0, 38.0, 20.0, colors::TEXT_DIM);
    
    // Occupancy
    let occ_text = format!("Occupancy: {}/{}", occupancy, total_units);
    draw_text(&occ_text, 520.0, 38.0, 20.0, colors::TEXT_DIM);
    
    // End Turn button
    let btn_x = w - 150.0;
    let btn_y = 10.0;
    if button(btn_x, btn_y, 130.0, 40.0, "End Month", true) {
        action = Some(UiAction::EndTurn);
    }
    
    // Bottom border
    draw_line(0.0, layout::HEADER_HEIGHT, w, layout::HEADER_HEIGHT, 2.0, colors::ACCENT);
    
    action
}
```

### 4. src/ui/building_view.rs

```rust
use macroquad::prelude::*;
use crate::building::{Building, Apartment, DesignType, NoiseLevel};
use crate::tenant::Tenant;
use super::{common::*, Selection, UiAction};

pub fn draw_building_view(
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
) -> Option<UiAction> {
    let mut action = None;
    
    let view_width = screen_width() * layout::PANEL_SPLIT;
    let view_height = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT;
    let view_x = 0.0;
    let view_y = layout::HEADER_HEIGHT;
    
    // Background
    draw_rectangle(view_x, view_y, view_width, view_height, colors::BACKGROUND);
    
    // Calculate layout
    let max_floor = building.apartments.iter().map(|a| a.floor).max().unwrap_or(1);
    let units_per_floor = building.apartments.iter()
        .filter(|a| a.floor == 1)
        .count();
    
    let total_width = units_per_floor as f32 * (layout::UNIT_WIDTH + layout::UNIT_GAP);
    let total_height = max_floor as f32 * layout::FLOOR_HEIGHT;
    
    let start_x = view_x + (view_width - total_width) / 2.0;
    let start_y = view_y + view_height - 80.0;  // Start from bottom
    
    // Draw floors (bottom to top)
    for floor in 1..=max_floor {
        let floor_y = start_y - (floor as f32 * layout::FLOOR_HEIGHT);
        
        // Floor label
        draw_text(
            &format!("Floor {}", floor),
            start_x - 80.0,
            floor_y + layout::UNIT_HEIGHT / 2.0,
            18.0,
            colors::TEXT_DIM,
        );
        
        // Draw units on this floor
        let floor_apartments: Vec<_> = building.apartments.iter()
            .filter(|a| a.floor == floor)
            .collect();
        
        for (i, apt) in floor_apartments.iter().enumerate() {
            let apt_x = start_x + i as f32 * (layout::UNIT_WIDTH + layout::UNIT_GAP);
            let apt_y = floor_y;
            
            if let Some(apt_action) = draw_apartment_unit(
                apt,
                tenants,
                apt_x,
                apt_y,
                selection,
            ) {
                action = Some(apt_action);
            }
        }
    }
    
    // Draw hallway at bottom
    let hallway_y = start_y + 20.0;
    let hallway_width = total_width - layout::UNIT_GAP;
    
    let hallway_selected = matches!(selection, Selection::Hallway);
    let hallway_hovered = is_hovered(start_x, hallway_y, hallway_width, 40.0);
    
    let hallway_color = if hallway_selected {
        colors::SELECTED
    } else if hallway_hovered {
        colors::HOVERED
    } else {
        colors::PANEL
    };
    
    draw_rectangle(start_x, hallway_y, hallway_width, 40.0, hallway_color);
    draw_rectangle_lines(start_x, hallway_y, hallway_width, 40.0, 2.0, colors::ACCENT);
    
    // Hallway label and condition
    draw_text("HALLWAY", start_x + 10.0, hallway_y + 25.0, 18.0, colors::TEXT);
    
    let cond_color = condition_color(building.hallway_condition);
    progress_bar(
        start_x + hallway_width - 110.0,
        hallway_y + 12.0,
        100.0,
        16.0,
        building.hallway_condition as f32,
        100.0,
        cond_color,
    );
    
    if was_clicked(start_x, hallway_y, hallway_width, 40.0) {
        action = Some(UiAction::SelectHallway);
    }
    
    action
}

fn draw_apartment_unit(
    apt: &Apartment,
    tenants: &[Tenant],
    x: f32,
    y: f32,
    selection: &Selection,
) -> Option<UiAction> {
    let w = layout::UNIT_WIDTH;
    let h = layout::UNIT_HEIGHT;
    
    let is_selected = matches!(selection, Selection::Apartment(id) if *id == apt.id);
    let is_hovered = is_hovered(x, y, w, h);
    
    // Background color
    let bg_color = if is_selected {
        colors::SELECTED
    } else if is_hovered {
        colors::HOVERED
    } else if apt.is_vacant() {
        colors::VACANT
    } else {
        colors::OCCUPIED
    };
    
    draw_rectangle(x, y, w, h, bg_color);
    
    // Border (thicker if selected)
    let border_width = if is_selected { 3.0 } else { 1.0 };
    let border_color = if is_selected { colors::ACCENT } else { colors::TEXT_DIM };
    draw_rectangle_lines(x, y, w, h, border_width, border_color);
    
    // Unit number
    draw_text(&apt.unit_number, x + 5.0, y + 18.0, 20.0, colors::TEXT);
    
    // Size indicator
    let size_text = match apt.size {
        crate::building::ApartmentSize::Small => "S",
        crate::building::ApartmentSize::Medium => "M",
    };
    draw_text(size_text, x + w - 20.0, y + 18.0, 16.0, colors::TEXT_DIM);
    
    // Condition bar
    let cond_color = condition_color(apt.condition);
    progress_bar(x + 5.0, y + 25.0, w - 10.0, 8.0, apt.condition as f32, 100.0, cond_color);
    
    // Design indicator
    let design_char = match apt.design {
        DesignType::Bare => "○",       // Empty circle
        DesignType::Practical => "◐",  // Half circle
        DesignType::Cozy => "●",       // Full circle
    };
    draw_text(design_char, x + 5.0, y + 50.0, 16.0, colors::TEXT_DIM);
    
    // Noise indicator (if high)
    if matches!(apt.effective_noise(), NoiseLevel::High) {
        draw_text("🔊", x + 25.0, y + 50.0, 14.0, colors::WARNING);
    }
    
    // Soundproofing indicator
    if apt.has_soundproofing {
        draw_text("🔇", x + 45.0, y + 50.0, 14.0, colors::POSITIVE);
    }
    
    // Tenant name or VACANT
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            // Truncate name to fit
            let name = if tenant.name.len() > 12 {
                format!("{}...", &tenant.name[..10])
            } else {
                tenant.name.clone()
            };
            draw_text(&name, x + 5.0, y + 68.0, 14.0, colors::TEXT);
            
            // Happiness indicator
            let happy_color = happiness_color(tenant.happiness);
            draw_circle(x + w - 12.0, y + h - 12.0, 6.0, happy_color);
        }
    } else {
        draw_text("VACANT", x + 5.0, y + 68.0, 14.0, colors::TEXT_DIM);
        
        // Rent
        draw_text(
            &format!("${}", apt.rent_price),
            x + w - 50.0,
            y + 68.0,
            14.0,
            colors::ACCENT,
        );
    }
    
    // Handle click
    if was_clicked(x, y, w, h) {
        return Some(UiAction::SelectApartment(apt.id));
    }
    
    None
}
```

### 5. src/ui/apartment_panel.rs

```rust
use macroquad::prelude::*;
use crate::building::{Apartment, Building, DesignType, UpgradeAction};
use crate::tenant::Tenant;
use crate::economy::UpgradeCosts;
use super::{common::*, UiAction};

pub fn draw_apartment_panel(
    apt: &Apartment,
    building: &Building,
    tenants: &[Tenant],
    money: i32,
) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;
    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, &format!("Unit {}", apt.unit_number));
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    // === Stats Section ===
    draw_text("CONDITION", content_x, y, 14.0, colors::TEXT_DIM);
    y += 5.0;
    let cond_color = condition_color(apt.condition);
    progress_bar(content_x, y, panel_w - 30.0, 20.0, apt.condition as f32, 100.0, cond_color);
    draw_text(&format!("{}%", apt.condition), content_x + panel_w - 70.0, y + 15.0, 16.0, colors::TEXT);
    y += 35.0;
    
    // Design
    let design_text = match apt.design {
        DesignType::Bare => "Bare",
        DesignType::Practical => "Practical",
        DesignType::Cozy => "Cozy ★",
    };
    draw_text(&format!("Design: {}", design_text), content_x, y, 18.0, colors::TEXT);
    y += 25.0;
    
    // Size
    let size_text = match apt.size {
        crate::building::ApartmentSize::Small => "Small",
        crate::building::ApartmentSize::Medium => "Medium",
    };
    draw_text(&format!("Size: {}", size_text), content_x, y, 18.0, colors::TEXT);
    y += 25.0;
    
    // Noise
    let noise_text = match apt.effective_noise() {
        crate::building::NoiseLevel::Low => "Quiet",
        crate::building::NoiseLevel::High => "Noisy ⚠",
    };
    let noise_color = if matches!(apt.effective_noise(), crate::building::NoiseLevel::High) {
        colors::WARNING
    } else {
        colors::POSITIVE
    };
    draw_text(&format!("Noise: {}", noise_text), content_x, y, 18.0, noise_color);
    y += 25.0;
    
    // Soundproofing
    if apt.has_soundproofing {
        draw_text("✓ Soundproofed", content_x, y, 16.0, colors::POSITIVE);
        y += 25.0;
    }
    
    // Rent
    draw_text(&format!("Rent: ${}/mo", apt.rent_price), content_x, y, 20.0, colors::ACCENT);
    y += 35.0;
    
    // Quality score
    let quality = apt.quality_score();
    draw_text(&format!("Quality Score: {}", quality), content_x, y, 16.0, colors::TEXT_DIM);
    y += 40.0;
    
    // === Tenant Info ===
    draw_line(content_x, y, content_x + panel_w - 30.0, y, 1.0, colors::TEXT_DIM);
    y += 15.0;
    
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            draw_text("TENANT", content_x, y, 14.0, colors::TEXT_DIM);
            y += 20.0;
            
            draw_text(&tenant.name, content_x, y, 20.0, colors::TEXT);
            y += 25.0;
            
            draw_text(&format!("{:?}", tenant.archetype), content_x, y, 16.0, colors::TEXT_DIM);
            y += 25.0;
            
            draw_text("Happiness", content_x, y, 14.0, colors::TEXT_DIM);
            y += 5.0;
            let happy_color = happiness_color(tenant.happiness);
            progress_bar(content_x, y, panel_w - 30.0, 16.0, tenant.happiness as f32, 100.0, happy_color);
            y += 25.0;
            
            draw_text(&format!("Months: {}", tenant.months_residing), content_x, y, 14.0, colors::TEXT_DIM);
            y += 30.0;
        }
    } else {
        draw_text("VACANT", content_x, y, 18.0, colors::WARNING);
        y += 30.0;
    }
    
    // === Upgrade Buttons ===
    draw_line(content_x, y, content_x + panel_w - 30.0, y, 1.0, colors::TEXT_DIM);
    y += 15.0;
    
    draw_text("UPGRADES", content_x, y, 14.0, colors::TEXT_DIM);
    y += 25.0;
    
    let btn_w = panel_w - 30.0;
    let btn_h = 36.0;
    
    // Repair button
    if apt.condition < 100 {
        let repair_amount = (100 - apt.condition).min(25);
        let repair_cost = UpgradeCosts::repair_cost(repair_amount);
        let can_afford = money >= repair_cost;
        
        let label = format!("Repair +{} (${}))", repair_amount, repair_cost);
        if button(content_x, y, btn_w, btn_h, &label, can_afford) {
            action = Some(UiAction::RepairApartment {
                apartment_id: apt.id,
                amount: repair_amount,
            });
        }
        y += btn_h + 8.0;
    }
    
    // Design upgrade button
    if let Some(next_design) = apt.design.next_upgrade() {
        if let Some(cost) = UpgradeCosts::design_upgrade_cost(&apt.design) {
            let can_afford = money >= cost;
            let label = format!("Upgrade to {:?} (${})", next_design, cost);
            
            if button(content_x, y, btn_w, btn_h, &label, can_afford) {
                action = Some(UiAction::UpgradeDesign { apartment_id: apt.id });
            }
            y += btn_h + 8.0;
        }
    }
    
    // Soundproofing button
    if !apt.has_soundproofing {
        let cost = UpgradeCosts::SOUNDPROOFING;
        let can_afford = money >= cost;
        let label = format!("Add Soundproofing (${})", cost);
        
        if button(content_x, y, btn_w, btn_h, &label, can_afford) {
            action = Some(UiAction::AddSoundproofing { apartment_id: apt.id });
        }
    }
    
    action
}

pub fn draw_hallway_panel(building: &Building, money: i32) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;
    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Hallway");
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    draw_text("CONDITION", content_x, y, 14.0, colors::TEXT_DIM);
    y += 5.0;
    let cond_color = condition_color(building.hallway_condition);
    progress_bar(content_x, y, panel_w - 30.0, 20.0, building.hallway_condition as f32, 100.0, cond_color);
    draw_text(&format!("{}%", building.hallway_condition), content_x + panel_w - 70.0, y + 15.0, 16.0, colors::TEXT);
    y += 45.0;
    
    draw_text("Affects overall building appeal", content_x, y, 14.0, colors::TEXT_DIM);
    y += 20.0;
    
    let appeal = building.building_appeal();
    draw_text(&format!("Building Appeal: {}", appeal), content_x, y, 18.0, colors::ACCENT);
    y += 50.0;
    
    // Repair button
    if building.hallway_condition < 100 {
        let repair_amount = (100 - building.hallway_condition).min(20);
        let repair_cost = UpgradeCosts::hallway_repair_cost(repair_amount);
        let can_afford = money >= repair_cost;
        
        let label = format!("Repair +{} (${})", repair_amount, repair_cost);
        let btn_w = panel_w - 30.0;
        
        if button(content_x, y, btn_w, 36.0, &label, can_afford) {
            action = Some(UiAction::RepairHallway { amount: repair_amount });
        }
    }
    
    action
}
```

### 6. src/ui/application_panel.rs

```rust
use macroquad::prelude::*;
use crate::tenant::TenantApplication;
use crate::building::Building;
use super::{common::*, UiAction};

pub fn draw_application_panel(
    applications: &[TenantApplication],
    building: &Building,
) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;
    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Applications");
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    if applications.is_empty() {
        draw_text("No pending applications", content_x, y, 18.0, colors::TEXT_DIM);
        return None;
    }
    
    for (i, app) in applications.iter().enumerate() {
        if y > panel_y + panel_h - 100.0 {
            draw_text("... more applications", content_x, y, 14.0, colors::TEXT_DIM);
            break;
        }
        
        // Application card
        let card_h = 90.0;
        draw_rectangle(content_x, y, panel_w - 30.0, card_h, colors::PANEL_HEADER);
        
        // Tenant info
        draw_text(&app.tenant.name, content_x + 10.0, y + 22.0, 18.0, colors::TEXT);
        draw_text(
            &format!("{:?}", app.tenant.archetype),
            content_x + 10.0,
            y + 42.0,
            14.0,
            colors::TEXT_DIM,
        );
        
        // Target apartment
        if let Some(apt) = building.get_apartment(app.apartment_id) {
            draw_text(
                &format!("→ Unit {}", apt.unit_number),
                content_x + 150.0,
                y + 22.0,
                16.0,
                colors::ACCENT,
            );
        }
        
        // Match score
        let score_color = if app.match_result.score >= 70 {
            colors::POSITIVE
        } else if app.match_result.score >= 50 {
            colors::ACCENT
        } else {
            colors::WARNING
        };
        draw_text(
            &format!("Match: {}%", app.match_result.score),
            content_x + 150.0,
            y + 42.0,
            14.0,
            score_color,
        );
        
        // Accept/Reject buttons
        let btn_y = y + 55.0;
        let btn_w = 80.0;
        
        if button(content_x + 10.0, btn_y, btn_w, 28.0, "Accept", true) {
            action = Some(UiAction::AcceptApplication { application_index: i });
        }
        
        if button(content_x + 100.0, btn_y, btn_w, 28.0, "Reject", true) {
            action = Some(UiAction::RejectApplication { application_index: i });
        }
        
        y += card_h + 10.0;
    }
    
    action
}
```

### 7. src/ui/notifications.rs

```rust
use macroquad::prelude::*;
use crate::simulation::{GameEvent, EventSeverity, EventLog};
use super::common::*;

pub fn draw_notifications(event_log: &EventLog, current_tick: u32) {
    let y = screen_height() - layout::FOOTER_HEIGHT;
    let w = screen_width();
    let h = layout::FOOTER_HEIGHT;
    
    // Background
    draw_rectangle(0.0, y, w, h, colors::PANEL);
    draw_line(0.0, y, w, y, 2.0, colors::TEXT_DIM);
    
    // Title
    draw_text("EVENTS", 15.0, y + 22.0, 16.0, colors::TEXT_DIM);
    
    // Recent events
    let recent = event_log.recent_events(5);
    let mut event_y = y + 45.0;
    
    for event in recent {
        let color = match event.severity() {
            EventSeverity::Positive => colors::POSITIVE,
            EventSeverity::Info => colors::TEXT_DIM,
            EventSeverity::Warning => colors::WARNING,
            EventSeverity::Negative => colors::NEGATIVE,
        };
        
        let msg = event.message();
        let display_msg = if msg.len() > 80 {
            format!("{}...", &msg[..77])
        } else {
            msg
        };
        
        draw_text(&display_msg, 15.0, event_y, 14.0, color);
        event_y += 18.0;
        
        if event_y > y + h - 10.0 {
            break;
        }
    }
}

/// Draw a floating notification for important events
pub fn draw_toast(message: &str, severity: EventSeverity, progress: f32) {
    if progress <= 0.0 {
        return;
    }
    
    let alpha = progress.min(1.0);
    let w = 400.0;
    let h = 50.0;
    let x = (screen_width() - w) / 2.0;
    let y = layout::HEADER_HEIGHT + 20.0;
    
    let bg_color = match severity {
        EventSeverity::Positive => Color::new(0.2, 0.5, 0.3, alpha),
        EventSeverity::Info => Color::new(0.2, 0.3, 0.5, alpha),
        EventSeverity::Warning => Color::new(0.5, 0.4, 0.2, alpha),
        EventSeverity::Negative => Color::new(0.5, 0.2, 0.2, alpha),
    };
    
    draw_rectangle(x, y, w, h, bg_color);
    draw_rectangle_lines(x, y, w, h, 2.0, Color::new(1.0, 1.0, 1.0, alpha * 0.5));
    
    let text_color = Color::new(1.0, 1.0, 1.0, alpha);
    let text_width = measure_text(message, None, 20, 1.0).width;
    draw_text(message, x + (w - text_width) / 2.0, y + 32.0, 20.0, text_color);
}
```

---

## Integration in GameplayState

```rust
// In src/state/gameplay.rs:
use crate::ui::{Selection, UiAction, draw_header, draw_building_view, 
                draw_apartment_panel, draw_application_panel, draw_notifications};

impl GameplayState {
    pub fn draw(&self) {
        // Header
        if let Some(action) = draw_header(
            self.funds.balance,
            self.current_tick,
            &self.building.name,
            self.building.occupancy_count(),
            self.building.apartments.len(),
        ) {
            // Store action for processing
        }
        
        // Building view
        if let Some(action) = draw_building_view(
            &self.building,
            &self.tenants,
            &self.selection,
        ) {
            // Store action
        }
        
        // Detail panel based on selection
        match &self.selection {
            Selection::Apartment(id) => {
                if let Some(apt) = self.building.get_apartment(*id) {
                    if let Some(action) = draw_apartment_panel(
                        apt,
                        &self.building,
                        &self.tenants,
                        self.funds.balance,
                    ) {
                        // Store action
                    }
                }
            }
            Selection::Applications => {
                if let Some(action) = draw_application_panel(
                    &self.applications,
                    &self.building,
                ) {
                    // Store action
                }
            }
            // ... other selections
        }
        
        // Notifications
        draw_notifications(&self.event_log, self.current_tick);
    }
}
```

---

## Acceptance Criteria

- [ ] Building overview shows all apartments visually
- [ ] Apartments show condition, design, occupancy at a glance
- [ ] Clicking apartment selects it and shows detail panel
- [ ] Detail panel shows all apartment stats
- [ ] Upgrade buttons work and show costs
- [ ] Disabled buttons for unaffordable upgrades
- [ ] Applications panel shows pending applications
- [ ] Accept/Reject buttons work
- [ ] Event log shows recent events
- [ ] End Turn button advances game
- [ ] Header shows money, month, occupancy

---

## Notes for Agent

- Use immediate mode patterns - no retained state in UI
- All UI functions return `Option<UiAction>` for intent
- Game logic processes actions, not UI
- Colors are centralized in `colors` module
- Layout constants prevent magic numbers
- Test with various window sizes
