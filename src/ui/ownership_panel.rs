use macroquad::prelude::*;
use crate::building::Building;
use crate::building::ownership::CondoBoard;
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
                
                // Sell Button
                // Mock price calculation: size base * 100? or something
                let sale_price = apt.size.base_rent() * 120; // 10 years rent roughly? No, 120 months = 10 years.
                
                if draw_button_mini(&format!("Sell Condo (${})", sale_price), panel_x + panel_width - 160.0, y + 5.0, 140.0, 20.0) {
                     // We need a specific action for this
                     // Need to add UiAction::SellUnitAsCondo
                     // But for now let's reuse generic or create new
                     // Assuming we have a way to trigger logic
                }
                
                y += 35.0;
                if y > panel_y + panel_height - 50.0 { break; }
            }
        },
        OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
             draw_condo_board_interface(board, panel_x, y, panel_width, &mut action);
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

fn draw_condo_board_interface(
    board: &CondoBoard,
    x: f32,
    y: f32,
    width: f32,
    action: &mut Option<UiAction>
) {
    let mut current_y = y;
    
    // Board Stats
    draw_text_ex(
        &format!("Reserve Fund: ${}", board.reserve_fund),
        x + 10.0,
        current_y,
        TextParams { font_size: 16, color: colors::POSITIVE, ..Default::default() }
    );
    current_y += 25.0;
    
    draw_text_ex(
        &format!("Active Board Members: {}", board.units.len()),
        x + 10.0,
        current_y,
        TextParams { font_size: 14, color: colors::TEXT, ..Default::default() }
    );
    current_y += 30.0;
    
    // Pending Votes
    draw_text_ex(
        "Pending Votes:",
        x + 10.0,
        current_y,
        TextParams { font_size: 14, color: colors::ACCENT, ..Default::default() }
    );
    current_y += 20.0;
    
    if board.pending_votes.is_empty() {
        draw_text_ex(
            "No active votes.",
            x + 10.0,
            current_y,
            TextParams { font_size: 14, color: colors::TEXT_DIM, ..Default::default() }
        );
         // current_y += 20.0; // Unused
    } else {
        for vote in &board.pending_votes {
            draw_rectangle(x + 10.0, current_y, width - 20.0, 60.0, colors::PANEL);
            draw_rectangle_lines(x + 10.0, current_y, width - 20.0, 60.0, 1.0, colors::TEXT_DIM);
            
            draw_text_ex(
                &vote.proposal,
                x + 20.0,
                current_y + 20.0,
                TextParams { font_size: 14, color: colors::TEXT_BRIGHT, ..Default::default() }
            );
            
            draw_text_ex(
                &format!("Cost: ${} | Deadline: Month {}", vote.cost, vote.deadline_month),
                x + 20.0,
                current_y + 40.0,
                TextParams { font_size: 12, color: colors::TEXT_DIM, ..Default::default() }
            );
            
            
             // Vote buttons
             if draw_button_mini("Vote YES", x + width - 150.0, current_y + 10.0, 60.0, 40.0) {
                 *action = Some(UiAction::VoteOnProposal { proposal_index: 0, vote_yes: true }); // Index 0 placeholder
             }
             if draw_button_mini("Vote NO", x + width - 80.0, current_y + 10.0, 60.0, 40.0) {
                 *action = Some(UiAction::VoteOnProposal { proposal_index: 0, vote_yes: false });
             }

            // current_y += 70.0; // Unused
        }
    }
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
