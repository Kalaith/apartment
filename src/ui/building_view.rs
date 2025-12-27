
use macroquad::prelude::*;
use crate::building::{Building, Apartment, DesignType, NoiseLevel, ApartmentSize};
use crate::tenant::Tenant;
use super::{common::*, Selection, UiAction};
use crate::assets::AssetManager;

pub fn draw_building_view(
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;
    
    let view_width = screen_width() * layout::PANEL_SPLIT;
    let view_height = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT;
    let view_x = 0.0;
    let view_y = layout::HEADER_HEIGHT;
    
    // Background - Building Exterior
    if let Some(tex) = assets.get_texture("building_exterior") {
         draw_texture_ex(tex, view_x, view_y, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(view_width, view_height)),
            ..Default::default()
        });
    } else {
        draw_rectangle(view_x, view_y, view_width, view_height, colors::BACKGROUND);
    }

    
    // Calculate layout - use max units per floor for total width
    let max_floor = building.apartments.iter().map(|a| a.floor).max().unwrap_or(1);
    let max_units_per_floor = (1..=max_floor)
        .map(|f| building.apartments.iter().filter(|a| a.floor == f).count())
        .max()
        .unwrap_or(1);
    
    let total_width = max_units_per_floor as f32 * (layout::UNIT_WIDTH + layout::UNIT_GAP);
    
    let center_x = view_x + view_width / 2.0;
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
        
        
        
        // Calculate total floor width (accounting for penthouse double-width)
        let mut floor_total_width = 0.0;
        for apt in &floor_apartments {
            let unit_w = if matches!(apt.size, ApartmentSize::Penthouse) {
                (layout::UNIT_WIDTH * 2.0) + layout::UNIT_GAP // Double width
            } else {
                layout::UNIT_WIDTH
            };
            floor_total_width += unit_w + layout::UNIT_GAP;
        }
        floor_total_width -= layout::UNIT_GAP; // Remove trailing gap
        
        // Center this floor's units
        let floor_start_x = center_x - floor_total_width / 2.0;
        
        let mut current_x = floor_start_x;
        for apt in floor_apartments.iter() {
            let unit_w = if matches!(apt.size, ApartmentSize::Penthouse) {
                (layout::UNIT_WIDTH * 2.0) + layout::UNIT_GAP
            } else {
                layout::UNIT_WIDTH
            };
            
            if let Some(apt_action) = draw_apartment_unit_sized(
                apt,
                tenants,
                current_x,
                floor_y,
                unit_w,
                selection,
                assets,
            ) {
                action = Some(apt_action);
            }
            
            current_x += unit_w + layout::UNIT_GAP;
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
    
    // Use texture for hallway if available
    let drawn_texture = if let Some(tex) = assets.get_texture("hallway") {
        draw_texture_ex(tex, start_x, hallway_y, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(hallway_width, 40.0)),
            ..Default::default()
        });
        true
    } else {
        draw_rectangle(start_x, hallway_y, hallway_width, 40.0, hallway_color);
        false
    };
    
    if !drawn_texture || hallway_selected {
        draw_rectangle_lines(start_x, hallway_y, hallway_width, 40.0, 2.0, colors::ACCENT);
    }

    
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
     // Condition Icon for Hallway
    if let Some(icon) = if building.hallway_condition > 50 { assets.get_texture("icon_condition_good") } else { assets.get_texture("icon_condition_poor") } {
        draw_texture_ex(icon, start_x + hallway_width - 130.0, hallway_y + 8.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(24.0, 24.0)),
            ..Default::default()
        });
    }

    
    if was_clicked(start_x, hallway_y, hallway_width, 40.0) {
        action = Some(UiAction::SelectHallway);
    }
    
    // Applications button (mouse-clickable) - Move to top
    let apps_btn_x = start_x;
    let apps_btn_y = view_y + 10.0; // Top of the panel
    if button(apps_btn_x, apps_btn_y, 150.0, 35.0, "Applications", true) {
        action = Some(UiAction::SelectApplications(None));
    }
    
    // Ownership button
    let own_btn_x = apps_btn_x + 160.0;
    if button(own_btn_x, apps_btn_y, 150.0, 35.0, "Ownership", true) {
        action = Some(UiAction::SelectOwnership);
    }
    
    action
}

fn draw_apartment_unit_sized(
    apt: &Apartment,
    tenants: &[Tenant],
    x: f32,
    y: f32,
    w: f32,
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let h = layout::UNIT_HEIGHT;
    
    let is_selected = matches!(selection, Selection::Apartment(id) if *id == apt.id);
    let unit_hovered = is_hovered(x, y, w, h);
    
    // Background color (fallback)
    let bg_color = if is_selected {
        colors::SELECTED
    } else if unit_hovered {
        colors::HOVERED
    } else if apt.is_vacant() {
        colors::VACANT
    } else {
        colors::OCCUPIED
    };
    
    // Draw Design Texture as background
    let design_id = match apt.design {
        DesignType::Bare => "design_bare",
        DesignType::Practical => "design_practical",
        DesignType::Cozy => "design_cozy",
        DesignType::Luxury => "design_luxury",
        DesignType::Opulent => "design_opulent",
    };
    
    if let Some(tex) = assets.get_texture(design_id) {
         draw_texture_ex(tex, x, y, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(w, h)),
            ..Default::default()
        });
        
        // Overlay selection/hover tint
        if is_selected {
            draw_rectangle(x, y, w, h, Color::new(1.0, 1.0, 0.0, 0.2));
        } else if unit_hovered {
            draw_rectangle(x, y, w, h, Color::new(1.0, 1.0, 1.0, 0.1));
        }
    } else {
        draw_rectangle(x, y, w, h, bg_color);
    }

    
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
        ApartmentSize::Large => "L",
        ApartmentSize::Penthouse => "PH",
    };
    draw_text(size_text, x + w - 20.0, y + 18.0, 16.0, colors::TEXT_DIM);
    
    // Condition bar
    let cond_color = condition_color(apt.condition);
    progress_bar(x + 5.0, y + 25.0, w - 10.0, 8.0, apt.condition as f32, 100.0, cond_color);
    
    // Design indicator (text fallback if texture fails, or always showing)
    // Maybe hide design text since we have visual? Or keep small?
    // KEEP for now.
    // let design_char = match apt.design { ... };
    // draw_text(design_char, x + 5.0, y + 50.0, 16.0, colors::TEXT_DIM);
    
    // Noise indicator (if high)
    if matches!(apt.effective_noise(), NoiseLevel::High) {
        if let Some(icon) = assets.get_texture("icon_noise") {
            draw_texture_ex(icon, x + 25.0, y + 35.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(20.0, 20.0)),
                ..Default::default()
            });
        } else {
            draw_text("!", x + 25.0, y + 50.0, 14.0, colors::WARNING);
        }
    }
    
    // Low Condition Warning (Leak/Damage)
    if apt.condition < 40 {
         draw_text("⚠", x + w - 25.0, y + 50.0, 24.0, colors::NEGATIVE);
    }
    
    // Soundproofing indicator
    if apt.has_soundproofing {
        if let Some(icon) = assets.get_texture("icon_soundproofing") {
            draw_texture_ex(icon, x + 50.0, y + 35.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(20.0, 20.0)),
                ..Default::default()
            });
        } else {
            draw_text("S", x + 45.0, y + 50.0, 14.0, colors::POSITIVE);
        }
    }
    
    // Tenant name or VACANT
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            
            // Draw Tenant Sprite in the room
             let portrait_id = format!("tenant_{}", tenant.archetype.name().to_lowercase());
            if let Some(tex) = assets.get_texture(&portrait_id) {
                // Draw sprite scaled down
                draw_texture_ex(tex, x + 35.0, y + 40.0, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(40.0, 40.0)),
                     // Maybe flip? No.
                    ..Default::default()
                });
            } else {
                 // Colored strip for archetype
                draw_rectangle(x + 5.0, y + 68.0, 3.0, 14.0, archetype_color(&tenant.archetype));
            }

            // Truncate name to fit (maybe smaller now?)
            // draw_text(&name, x + 12.0, y + 80.0, 14.0, colors::TEXT);
            
            // Happiness icon
             let happiness_level = if tenant.happiness >= 90 { "happiness_ecstatic" }
            else if tenant.happiness >= 70 { "happiness_happy" }
            else if tenant.happiness >= 40 { "happiness_neutral" }
            else if tenant.happiness >= 20 { "happiness_unhappy" }
            else { "happiness_miserable" };
            
            if let Some(icon) = assets.get_texture(happiness_level) {
                draw_texture_ex(icon, x + w - 24.0, y + h - 24.0, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(20.0, 20.0)),
                    ..Default::default()
                });
            } else {
                 let icon_char = happiness_icon(tenant.happiness);
                 draw_text(icon_char, x + w - 24.0, y + 80.0, 16.0, colors::TEXT);
            }

        }
    } else {
        // Draw window texture if vacant (street or quiet based on noise)
        let window_tex = if matches!(apt.effective_noise(), NoiseLevel::High) { "window_street" } else { "window_quiet" };
         if let Some(tex) = assets.get_texture(window_tex) {
             // Draw window in the middle
            draw_texture_ex(tex, x + 35.0, y + 40.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(40.0, 40.0)),
                ..Default::default()
            });
        }

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
