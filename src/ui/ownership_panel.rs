use macroquad::prelude::*;
use crate::building::Building;
use crate::building::ownership::OwnershipType;
use crate::ui::{UiAction, colors};

pub fn draw_ownership_panel(
    building: &Building,
) -> Option<UiAction> {
    let panel_x = screen_width() * 0.5 + 10.0;
    let panel_y = 80.0;
    let panel_width = screen_width() * 0.5 - 30.0;
    let panel_height = screen_height() - 140.0;

    // Background
    draw_rectangle(
        panel_x, panel_y, panel_width, panel_height,
        Color::from_rgba(30, 30, 35, 255)
    );
    draw_rectangle_lines(
        panel_x, panel_y, panel_width, panel_height, 2.0,
        Color::from_rgba(60, 60, 70, 255)
    );

    // Title
    draw_text_ex(
        "Building Ownership",
        panel_x + 10.0,
        panel_y + 25.0,
        TextParams {
            font_size: 20,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    let mut action = None;
    let mut y = panel_y + 50.0;

    // Display current ownership model
    let model_name = match building.ownership_model {
        OwnershipType::FullRental => "Full Rental (Sole Proprietorship)",
        OwnershipType::MixedOwnership(_) => "Mixed Ownership (Partial Condo)",
        OwnershipType::FullCondo(_) => "Full Condo Association",
        OwnershipType::CooperativeHousing => "Tenant Cooperative",
        OwnershipType::SocialHousing => "Social Housing / Subsidized",
    };

    draw_text_ex(
        model_name,
        panel_x + 10.0,
        y,
        TextParams {
            font_size: 16,
            color: colors::ACCENT,
            ..Default::default()
        }
    );
    y += 30.0;

    // Handle different models
    match &building.ownership_model {
        OwnershipType::FullRental => {
            draw_text_ex(
                "You own 100% of this building and collect all rent.",
                panel_x + 10.0,
                y,
                TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
            );
            y += 20.0;
            draw_text_ex(
                "You can convert individual units to Condos to raise quick capital.",
                panel_x + 10.0,
                y,
                TextParams { font_size: 14, color: colors::TEXT_DIM, ..Default::default() }
            );
            y += 30.0;
            
            // Show conversion options for vacant units?
            // For now, let's just list unit counts
            let owned_count = building.apartments.len();
            draw_text_ex(
                &format!("Units Owned: {}", owned_count),
                panel_x + 10.0,
                y,
                TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
            );
            y += 30.0;
            
            // Allow converting a unit?
            // It's a bit complex to select WHICH unit here without a selector.
            // Maybe just a note: "Select a unit in the main view to Sell as Condo"
            // Or listed items.
            
            draw_text_ex(
                "Available Units for Conversion:",
                panel_x + 10.0,
                y,
                TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
            );
            y += 20.0;
            
            for apt in &building.apartments {
                // Background strip
                draw_rectangle(panel_x + 10.0, y, panel_width - 20.0, 30.0, colors::PANEL);
                
                // Unit Name
                draw_text_ex(
                    &format!("Unit {}", apt.unit_number),
                    panel_x + 20.0,
                    y + 20.0,
                    TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
                );
                
                // Status
                let status = if apt.is_vacant() { "Vacant" } else { "Occupied" };
                draw_text_ex(
                    status,
                    panel_x + 100.0,
                    y + 20.0,
                    TextParams { 
                        font_size: 14, 
                        color: if apt.is_vacant() { colors::POSITIVE } else { colors::WARNING },
                        ..Default::default() 
                    }
                );
                
                // Sell Button - use calculated market value
                let sale_price = apt.market_value();
                
                if draw_button_mini(&format!("Sell Condo (${})", sale_price), panel_x + panel_width - 160.0, y + 5.0, 140.0, 20.0) {
                    action = Some(UiAction::SellUnitAsCondo { apartment_id: apt.id });
                }
                
                y += 35.0;
                if y > panel_y + panel_height - 50.0 { break; }
            }
        },
        OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
            // Show condo board stats
            draw_text_ex(
                &format!("Reserve Fund: ${}", board.reserve_fund),
                panel_x + 10.0,
                y,
                TextParams { font_size: 16, color: colors::POSITIVE, ..Default::default() }
            );
            y += 25.0;
            
            draw_text_ex(
                &format!("Sold Units: {} | Remaining: {}", 
                    board.units.len(),
                    building.apartments.len() - board.units.len()),
                panel_x + 10.0,
                y,
                TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
            );
            y += 30.0;
            
            // Show unsold units that can still be converted
            let sold_ids: std::collections::HashSet<u32> = board.units.iter()
                .map(|u| u.apartment_id)
                .collect();
            
            let unsold: Vec<_> = building.apartments.iter()
                .filter(|apt| !sold_ids.contains(&apt.id))
                .collect();
            
            if !unsold.is_empty() {
                draw_text_ex(
                    "Remaining Units for Sale:",
                    panel_x + 10.0,
                    y,
                    TextParams { font_size: 14, color: colors::ACCENT, ..Default::default() }
                );
                y += 20.0;
                
                for apt in unsold {
                    // Background strip
                    draw_rectangle(panel_x + 10.0, y, panel_width - 20.0, 30.0, colors::PANEL);
                    
                    // Unit Name
                    draw_text_ex(
                        &format!("Unit {}", apt.unit_number),
                        panel_x + 20.0,
                        y + 20.0,
                        TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
                    );
                    
                    // Status
                    let status = if apt.is_vacant() { "Vacant" } else { "Occupied" };
                    draw_text_ex(
                        status,
                        panel_x + 100.0,
                        y + 20.0,
                        TextParams { 
                            font_size: 14, 
                            color: if apt.is_vacant() { colors::POSITIVE } else { colors::WARNING },
                            ..Default::default() 
                        }
                    );
                    
                    // Sell Button
                    let sale_price = apt.market_value();
                    
                    if draw_button_mini(&format!("Sell (${})", sale_price), panel_x + panel_width - 140.0, y + 5.0, 120.0, 20.0) {
                        action = Some(UiAction::SellUnitAsCondo { apartment_id: apt.id });
                    }
                    
                    y += 35.0;
                    if y > panel_y + panel_height - 80.0 { break; }
                }
            } else {
                draw_text_ex(
                    "All units have been sold as condos.",
                    panel_x + 10.0,
                    y,
                    TextParams { font_size: 14, color: colors::TEXT_DIM, ..Default::default() }
                );
            }
        },
        _ => {
             draw_text_ex(
                "Management options not yet implemented for this ownership type.",
                panel_x + 10.0,
                y,
                TextParams { font_size: 14, color: colors::TEXT_DIM, ..Default::default() }
            );
        }
    }
    
    // Close / Back button
    if draw_button_icon("Close Panel", panel_x + 10.0, panel_y + panel_height - 40.0, 120.0, 30.0) {
        action = Some(UiAction::ClearSelection);
    }

    action
}

// Helper duplicates (should be in common really, but for speed)
fn draw_button_mini(label: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width 
               && mouse.1 >= y && mouse.1 <= y + height;

    let bg_color = if hovered {
        colors::ACCENT
    } else {
        Color::from_rgba(60, 90, 120, 255)
    };

    draw_rectangle(x, y, width, height, bg_color);
    
    let text_width = measure_text(label, None, 12, 1.0).width;
    draw_text_ex(
        label,
        x + (width - text_width) / 2.0,
        y + height / 2.0 + 4.0,
        TextParams {
            font_size: 12,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

fn draw_button_icon(label: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width 
               && mouse.1 >= y && mouse.1 <= y + height;

    let bg_color = if hovered {
        Color::from_rgba(70, 80, 100, 255)
    } else {
        Color::from_rgba(50, 55, 65, 255)
    };

    draw_rectangle(x, y, width, height, bg_color);
    draw_rectangle_lines(x, y, width, height, 1.0, Color::from_rgba(80, 90, 110, 255));

    let text_width = measure_text(label, None, 14, 1.0).width;
    draw_text_ex(
        label,
        x + (width - text_width) / 2.0,
        y + height / 2.0 + 5.0,
        TextParams {
            font_size: 14,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    hovered && is_mouse_button_pressed(MouseButton::Left)
}
