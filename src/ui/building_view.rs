use macroquad::prelude::*;
use crate::building::{Building, Apartment, DesignType, NoiseLevel, ApartmentSize};
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
    
    // Applications button (mouse-clickable)
    let apps_btn_x = start_x;
    let apps_btn_y = start_y + 70.0;
    if button(apps_btn_x, apps_btn_y, 150.0, 35.0, "Applications", true) {
        action = Some(UiAction::SelectApplications);
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
    let unit_hovered = is_hovered(x, y, w, h);
    
    // Background color
    let bg_color = if is_selected {
        colors::SELECTED
    } else if unit_hovered {
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
        ApartmentSize::Small => "S",
        ApartmentSize::Medium => "M",
    };
    draw_text(size_text, x + w - 20.0, y + 18.0, 16.0, colors::TEXT_DIM);
    
    // Condition bar
    let cond_color = condition_color(apt.condition);
    progress_bar(x + 5.0, y + 25.0, w - 10.0, 8.0, apt.condition as f32, 100.0, cond_color);
    
    // Design indicator
    let design_char = match apt.design {
        DesignType::Bare => "B",
        DesignType::Practical => "P",
        DesignType::Cozy => "C",
    };
    draw_text(design_char, x + 5.0, y + 50.0, 16.0, colors::TEXT_DIM);
    
    // Noise indicator (if high)
    if matches!(apt.effective_noise(), NoiseLevel::High) {
        draw_text("!", x + 25.0, y + 50.0, 14.0, colors::WARNING);
    }
    
    // Soundproofing indicator
    if apt.has_soundproofing {
        draw_text("S", x + 45.0, y + 50.0, 14.0, colors::POSITIVE);
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
