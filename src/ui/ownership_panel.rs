use crate::building::ownership::OwnershipType;
use crate::building::Building;
use crate::ui::{colors, UiAction};
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text_ex;

pub fn draw_ownership_panel(building: &Building, market_multiplier: f32) -> Option<UiAction> {
    let panel_x = screen_width() * 0.5 + 10.0;
    let panel_y = 80.0;
    let panel_width = screen_width() * 0.5 - 30.0;
    let panel_height = screen_height() - 140.0;

    // Themed panel frame + header.
    crate::ui::common::panel(
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        "Building Ownership",
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

    draw_ui_text_ex(
        model_name,
        panel_x + 10.0,
        y,
        TextParams {
            font_size: 16,
            color: colors::ACCENT(),
            ..Default::default()
        },
    );
    y += 30.0;

    // Handle different models
    match &building.ownership_model {
        OwnershipType::FullRental => {
            draw_ui_text_ex(
                "You own 100% of this building and collect all rent.",
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT(),
                    ..Default::default()
                },
            );
            y += 20.0;
            draw_ui_text_ex(
                "You can convert individual units to Condos to raise quick capital.",
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT_DIM(),
                    ..Default::default()
                },
            );
            y += 30.0;

            // Show conversion options for vacant units?
            // For now, let's just list unit counts
            let owned_count = building.apartments.len();
            draw_ui_text_ex(
                &format!("Units Owned: {}", owned_count),
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT(),
                    ..Default::default()
                },
            );
            y += 30.0;

            // Allow converting a unit?
            // It's a bit complex to select WHICH unit here without a selector.
            // Maybe just a note: "Select a unit in the main view to Sell as Condo"
            // Or listed items.

            draw_ui_text_ex(
                "Available Units for Conversion:",
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT(),
                    ..Default::default()
                },
            );
            y += 20.0;

            for apt in &building.apartments {
                // Background strip
                draw_rectangle(
                    panel_x + 10.0,
                    y,
                    panel_width - 20.0,
                    30.0,
                    colors::SURFACE(),
                );

                // Unit Name
                draw_ui_text_ex(
                    &format!("Unit {}", apt.unit_number),
                    panel_x + 20.0,
                    y + 20.0,
                    TextParams {
                        font_size: 14,
                        color: colors::TEXT(),
                        ..Default::default()
                    },
                );

                // Status
                let status = if apt.is_vacant() {
                    "Vacant"
                } else {
                    "Occupied"
                };
                draw_ui_text_ex(
                    status,
                    panel_x + 100.0,
                    y + 20.0,
                    TextParams {
                        font_size: 14,
                        color: if apt.is_vacant() {
                            colors::POSITIVE()
                        } else {
                            colors::WARNING()
                        },
                        ..Default::default()
                    },
                );

                // Sell Button - use calculated market value
                let sale_price = (apt.market_value() as f32 * market_multiplier) as i32;

                if crate::ui::widgets::button_at(
                    Rect::new(panel_x + panel_width - 160.0, y + 4.0, 148.0, 24.0),
                    &format!("Sell Condo (${})", sale_price),
                    true,
                    crate::ui::theme::Tone::Positive,
                ) {
                    action = Some(UiAction::SellUnitAsCondo {
                        apartment_id: apt.id,
                    });
                }

                y += 35.0;
                if y > panel_y + panel_height - 50.0 {
                    break;
                }
            }
        }
        OwnershipType::MixedOwnership(board) | OwnershipType::FullCondo(board) => {
            // Show condo board stats
            draw_ui_text_ex(
                &format!("Reserve Fund: ${}", board.reserve_fund),
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 16,
                    color: colors::POSITIVE(),
                    ..Default::default()
                },
            );
            y += 25.0;

            draw_ui_text_ex(
                &format!(
                    "Sold Units: {} | Remaining: {}",
                    board.units.len(),
                    building.apartments.len() - board.units.len()
                ),
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT(),
                    ..Default::default()
                },
            );
            y += 30.0;

            // Show unsold units that can still be converted
            let sold_ids: std::collections::HashSet<u32> =
                board.units.iter().map(|u| u.apartment_id).collect();

            let unsold: Vec<_> = building
                .apartments
                .iter()
                .filter(|apt| !sold_ids.contains(&apt.id))
                .collect();

            if !unsold.is_empty() {
                draw_ui_text_ex(
                    "Remaining Units for Sale:",
                    panel_x + 10.0,
                    y,
                    TextParams {
                        font_size: 14,
                        color: colors::ACCENT(),
                        ..Default::default()
                    },
                );
                y += 20.0;

                for apt in unsold {
                    // Background strip
                    draw_rectangle(
                        panel_x + 10.0,
                        y,
                        panel_width - 20.0,
                        30.0,
                        colors::SURFACE(),
                    );

                    // Unit Name
                    draw_ui_text_ex(
                        &format!("Unit {}", apt.unit_number),
                        panel_x + 20.0,
                        y + 20.0,
                        TextParams {
                            font_size: 14,
                            color: colors::TEXT(),
                            ..Default::default()
                        },
                    );

                    // Status
                    let status = if apt.is_vacant() {
                        "Vacant"
                    } else {
                        "Occupied"
                    };
                    draw_ui_text_ex(
                        status,
                        panel_x + 100.0,
                        y + 20.0,
                        TextParams {
                            font_size: 14,
                            color: if apt.is_vacant() {
                                colors::POSITIVE()
                            } else {
                                colors::WARNING()
                            },
                            ..Default::default()
                        },
                    );

                    // Sell Button
                    let sale_price = (apt.market_value() as f32 * market_multiplier) as i32;

                    if crate::ui::widgets::button_at(
                        Rect::new(panel_x + panel_width - 140.0, y + 4.0, 128.0, 24.0),
                        &format!("Sell (${})", sale_price),
                        true,
                        crate::ui::theme::Tone::Positive,
                    ) {
                        action = Some(UiAction::SellUnitAsCondo {
                            apartment_id: apt.id,
                        });
                    }

                    y += 35.0;
                    if y > panel_y + panel_height - 80.0 {
                        break;
                    }
                }
            } else {
                draw_ui_text_ex(
                    "All units have been sold as condos.",
                    panel_x + 10.0,
                    y,
                    TextParams {
                        font_size: 14,
                        color: colors::TEXT_DIM(),
                        ..Default::default()
                    },
                );
            }
        }
        _ => {
            draw_ui_text_ex(
                "Management options not yet implemented for this ownership type.",
                panel_x + 10.0,
                y,
                TextParams {
                    font_size: 14,
                    color: colors::TEXT_DIM(),
                    ..Default::default()
                },
            );
        }
    }

    // Close / Back button
    if crate::ui::common::button(
        panel_x + 10.0,
        panel_y + panel_height - 40.0,
        120.0,
        30.0,
        "Close Panel",
        true,
    ) {
        action = Some(UiAction::ClearSelection);
    }

    action
}
