
use macroquad::prelude::*;
use crate::city::{City, Neighborhood, NeighborhoodType, PropertyListing};
use crate::narrative::NarrativeEventSystem;
use crate::ui::colors;
use crate::assets::AssetManager;

/// Draw the city map showing all neighborhoods
pub fn draw_city_map(city: &City, assets: &AssetManager, narrative: &NarrativeEventSystem) -> Option<CityMapAction> {
    let map_x = 20.0;
    let map_y = 80.0;
    let map_width = screen_width() * 0.5 - 40.0;
    let map_height = screen_height() - 140.0;

    // Background
    draw_rectangle(
        map_x, map_y, map_width, map_height,
        Color::from_rgba(25, 25, 30, 255)
    );
    draw_rectangle_lines(
        map_x, map_y, map_width, map_height, 2.0,
        Color::from_rgba(60, 60, 70, 255)
    );

    // Title
    draw_text_ex(
        &city.name,
        map_x + 10.0,
        map_y + 25.0,
        TextParams {
            font_size: 24,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    // Draw neighborhoods as a 2x2 grid
    let grid_x = map_x + 20.0;
    let grid_y = map_y + 50.0;
    let cell_width = (map_width - 60.0) / 2.0;
    let cell_height = (map_height - 100.0) / 2.0;
    let padding = 10.0;

    let mut action = None;

    for (i, neighborhood) in city.neighborhoods.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        
        let x = grid_x + col as f32 * (cell_width + padding);
        let y = grid_y + row as f32 * (cell_height + padding);

        if let Some(a) = draw_neighborhood_cell(neighborhood, x, y, cell_width, cell_height, city, assets, narrative) {
            action = Some(a);
        }
    }

    action
}

/// Draw a single neighborhood cell
fn draw_neighborhood_cell(
    neighborhood: &Neighborhood, 
    x: f32, 
    y: f32, 
    width: f32, 
    height: f32,
    _city: &City,
    assets: &AssetManager,
    narrative: &NarrativeEventSystem,
) -> Option<CityMapAction> {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width 
               && mouse.1 >= y && mouse.1 <= y + height;

    // Background with neighborhood color (fallback or tint)
    let base_color = neighborhood.neighborhood_type.color();
    let bg_color = if hovered {
        Color::from_rgba(
            ((base_color.r * 255.0) + 20.0).min(255.0) as u8,
            ((base_color.g * 255.0) + 20.0).min(255.0) as u8,
            ((base_color.b * 255.0) + 20.0).min(255.0) as u8,
            200
        )
    } else {
        Color::from_rgba(
            (base_color.r * 255.0 * 0.6) as u8,
            (base_color.g * 255.0 * 0.6) as u8,
            (base_color.b * 255.0 * 0.6) as u8,
            180
        )
    };

    draw_rectangle(x, y, width, height, bg_color);
    
    // Draw Neighborhood Texture
    let texture_id = match neighborhood.neighborhood_type {
        NeighborhoodType::Downtown => "neighborhood_downtown",
        NeighborhoodType::Industrial => "neighborhood_industrial",
        NeighborhoodType::Suburbs => "neighborhood_residential", // Suburbs maps to residential graphic
        NeighborhoodType::Historic => "neighborhood_university", // Fallback or maybe we should have a historic one? Let's use university graphic for historic for now or residential
        // _ => "neighborhood_residential", 
    };
    
    if let Some(tex) = assets.get_texture(texture_id) {
         // Draw with some transparency or multiply to blend with selection?
         // Or just draw it fully opaque and draw selection border/overlay.
         draw_texture_ex(tex, x, y, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(width, height)),
            ..Default::default()
        });
        
        // Darken it a bit to make text readable
        draw_rectangle(x, y, width, height, Color::new(0.0, 0.0, 0.0, 0.5));
    }

    draw_rectangle_lines(x, y, width, height, 2.0, base_color);

    // Neighborhood name
    draw_text_ex(
        &neighborhood.name,
        x + 8.0,
        y + 22.0,
        TextParams {
            font_size: 18,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    // Neighborhood type
    draw_text_ex(
        neighborhood.neighborhood_type.name(),
        x + 8.0,
        y + 40.0,
        TextParams {
            font_size: 14,
            color: colors::TEXT_DIM,
            ..Default::default()
        }
    );

    // Building count
    let building_count = neighborhood.building_ids.len();
    let slot_text = format!("Buildings: {}/{}", building_count, neighborhood.available_slots);
    draw_text_ex(
        &slot_text,
        x + 8.0,
        y + 60.0,
        TextParams {
            font_size: 14,
            color: if building_count > 0 { colors::POSITIVE } else { colors::TEXT_DIM },
            ..Default::default()
        }
    );

    // Stats preview
    let stats = &neighborhood.stats;
    draw_text_ex(
        &format!("Crime: {} | Transit: {}", stats.crime_level, stats.transit_access),
        x + 8.0,
        y + 80.0,
        TextParams {
            font_size: 12,
            color: colors::TEXT_DIM,
            ..Default::default()
        }
    );

    // Reputation bar
    let bar_y = y + height - 25.0;
    let bar_width = width - 16.0;
    draw_text_ex(
        &format!("Rep: {}", neighborhood.reputation),
        x + 8.0,
        bar_y - 3.0,
        TextParams {
            font_size: 12,
            color: colors::TEXT_DIM,
            ..Default::default()
        }
    );
    draw_progress_bar(x + 8.0, bar_y, bar_width, 8.0, neighborhood.reputation as f32 / 100.0, colors::POSITIVE);

    // Event indicator
    let has_event = narrative.events.iter().any(|e| 
        !e.read && e.related_neighborhood_id == Some(neighborhood.id)
    );

    if has_event {
        let icon_x = x + width - 30.0;
        let icon_y = y + 30.0;
        draw_circle(icon_x, icon_y, 12.0, colors::ACCENT);
        draw_text("!", icon_x - 3.0, icon_y + 5.0, 20.0, colors::TEXT_BRIGHT);
    }

    // Button area
    if hovered && is_mouse_button_pressed(MouseButton::Left) {
        return Some(CityMapAction::SelectNeighborhood(neighborhood.id));
    }

    None
}

/// Draw the portfolio panel showing all player buildings
pub fn draw_portfolio_panel(city: &City, selected_building: usize, assets: &AssetManager) -> Option<CityMapAction> {
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
        "Your Properties",
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
    let item_height = 80.0;

    for (index, building, neighborhood_name) in city.buildings_with_info() {
        let is_selected = index == selected_building;
        
        let item_width = panel_width - 20.0;
        let item_x = panel_x + 10.0;

        // Background
        let bg_color = if is_selected {
            Color::from_rgba(50, 60, 80, 255)
        } else {
            Color::from_rgba(40, 40, 45, 255)
        };
        draw_rectangle(item_x, y, item_width, item_height - 5.0, bg_color);
        
        // Building Icon/Thumbnail?
        // Maybe just use a generic icon for now or small building exterior
        if let Some(_tex) = assets.get_texture("icon_building") { // Assuming we have one, or reuse building_exterior
             // If we don't have icon_building, we can use building_exterior scaled down?
             // But building_exterior is large. Let's just skip for now or use rectangle.
        }


        if is_selected {
            draw_rectangle_lines(item_x, y, item_width, item_height - 5.0, 2.0, colors::ACCENT);
            
            // Enter Button
            if draw_button_mini("Enter", item_x + item_width - 70.0, y + 25.0, 60.0, 30.0) {
                 action = Some(CityMapAction::EnterBuilding(index));
            }
        }

        // Building name
        draw_text_ex(
            &building.name,
            item_x + 10.0,
            y + 22.0,
            TextParams {
                font_size: 18,
                color: if is_selected { colors::ACCENT } else { colors::TEXT_BRIGHT },
                ..Default::default()
            }
        );

        // Location
        draw_text_ex(
            &neighborhood_name,
            item_x + 10.0,
            y + 40.0,
            TextParams {
                font_size: 14,
                color: colors::TEXT_DIM,
                ..Default::default()
            }
        );

        // Stats
        let occupancy = building.occupancy_count();
        let total = building.apartments.len();
        let appeal = building.building_appeal();
        
        draw_text_ex(
            &format!("Occupancy: {}/{} | Appeal: {}", occupancy, total, appeal),
            item_x + 10.0,
            y + 58.0,
            TextParams {
                font_size: 14,
                color: if occupancy == total { colors::POSITIVE } else { colors::TEXT_DIM },
                ..Default::default()
            }
        );

        // Click to select
        let mouse = mouse_position();
        let hovered = mouse.0 >= item_x && mouse.0 <= item_x + item_width 
                   && mouse.1 >= y && mouse.1 <= y + item_height - 5.0;

        if action.is_none() && hovered && is_mouse_button_pressed(MouseButton::Left) {
            action = Some(CityMapAction::SelectBuilding(index));
        }

        y += item_height;
        
        if y > panel_y + panel_height - item_height {
            break;
        }
    }

    // "Add Building" button if there's space
    if y < panel_y + panel_height - 50.0 {
        let btn_width = panel_width - 40.0;
        let btn_x = panel_x + 20.0;
        
        if draw_button_icon("+ Acquire New Building", btn_x, y + 10.0, btn_width, 35.0) {
            action = Some(CityMapAction::OpenMarket);
        }
    }

    action
}

/// Draw property market listings
pub fn draw_market_panel(
    listings: &[&PropertyListing],
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let panel_x = 20.0;
    let panel_y = 80.0;
    let panel_width = screen_width() - 40.0;
    let panel_height = screen_height() - 140.0;

    // Background
    draw_rectangle(
        panel_x, panel_y, panel_width, panel_height,
        Color::from_rgba(25, 28, 35, 255)
    );
    draw_rectangle_lines(
        panel_x, panel_y, panel_width, panel_height, 2.0,
        Color::from_rgba(70, 80, 100, 255)
    );

    // Title
    draw_text_ex(
        "Property Market",
        panel_x + 15.0,
        panel_y + 28.0,
        TextParams {
            font_size: 22,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    // Budget display
    draw_text_ex(
        &format!("Your Budget: ${}", player_funds),
        panel_x + panel_width - 200.0,
        panel_y + 28.0,
        TextParams {
            font_size: 16,
            color: colors::POSITIVE,
            ..Default::default()
        }
    );

    let mut action = None;
    let start_y = panel_y + 50.0;
    let listing_height = 120.0;
    let listing_width = (panel_width - 60.0) / 2.0;

    for (i, listing) in listings.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        
        let x = panel_x + 20.0 + col as f32 * (listing_width + 20.0);
        let y = start_y + row as f32 * (listing_height + 15.0);

        if y + listing_height > panel_y + panel_height - 20.0 {
            break;
        }

        if let Some(a) = draw_listing_card(listing, x, y, listing_width, listing_height, neighborhoods, player_funds, assets) {
            action = Some(a);
        }
    }

    // Back button
    if draw_button_icon("← Back to Map", panel_x + 15.0, panel_y + panel_height - 60.0, 150.0, 35.0) {
        action = Some(CityMapAction::CloseMarket);
    }

    action
}

fn draw_listing_card(
    listing: &PropertyListing,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width 
               && mouse.1 >= y && mouse.1 <= y + height;

    // Get neighborhood
    let neighborhood = neighborhoods.iter().find(|n| n.id == listing.neighborhood_id);
    let neighborhood_color = neighborhood
        .map(|n| n.neighborhood_type.color())
        .unwrap_or(Color::from_rgba(100, 100, 100, 255));

    // Background
    let bg_color = if hovered {
        Color::from_rgba(50, 55, 65, 255)
    } else {
        Color::from_rgba(35, 38, 45, 255)
    };
    draw_rectangle(x, y, width, height, bg_color);
    
    // Neighborhood texture preview
    if let Some(n) = neighborhood {
         let texture_id = match n.neighborhood_type {
            NeighborhoodType::Downtown => "neighborhood_downtown",
            NeighborhoodType::Industrial => "neighborhood_industrial",
            NeighborhoodType::Suburbs => "neighborhood_residential",
            NeighborhoodType::Historic => "neighborhood_university", // Using university graphic for now
        };
        if let Some(tex) = assets.get_texture(texture_id) {
             draw_texture_ex(tex, x + width - 100.0, y + 10.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(90.0, 60.0)),
                ..Default::default()
            });
        }
    }

    
    // Color accent bar
    draw_rectangle(x, y, 5.0, height, neighborhood_color);

    // Building name
    draw_text_ex(
        &listing.name,
        x + 15.0,
        y + 22.0,
        TextParams {
            font_size: 16,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        }
    );

    // Location
    let location_name = neighborhood.map(|n| n.name.as_str()).unwrap_or("Unknown");
    draw_text_ex(
        location_name,
        x + 15.0,
        y + 40.0,
        TextParams {
            font_size: 13,
            color: colors::TEXT_DIM,
            ..Default::default()
        }
    );

    // Stats
    draw_text_ex(
        &format!("{} floors, {} units | {} condition",
            listing.num_floors,
            listing.total_units(),
            listing.condition.name()
        ),
        x + 15.0,
        y + 58.0,
        TextParams {
            font_size: 12,
            color: colors::TEXT_DIM,
            ..Default::default()
        }
    );

    // Existing tenants
    if listing.existing_tenants > 0 {
        draw_text_ex(
            &format!("{} existing tenants", listing.existing_tenants),
            x + 15.0,
            y + 73.0,
            TextParams {
                font_size: 11,
                color: colors::WARNING,
                ..Default::default()
            }
        );
    }

    // Price and button
    let can_afford = player_funds >= listing.asking_price;
    let price_color = if can_afford { colors::POSITIVE } else { colors::NEGATIVE };
    
    draw_text_ex(
        &format!("${}", listing.asking_price),
        x + 15.0,
        y + height - 12.0,
        TextParams {
            font_size: 18,
            color: price_color,
            ..Default::default()
        }
    );

    // Buy button
    let btn_width = 80.0;
    let btn_x = x + width - btn_width - 10.0;
    let btn_y = y + height - 30.0;

    if can_afford {
        if draw_button_mini("Buy", btn_x, btn_y, btn_width, 22.0) {
            return Some(CityMapAction::PurchaseBuilding(listing.id));
        }
    } else {
        draw_text_ex(
            "Can't afford",
            btn_x,
            btn_y + 15.0,
            TextParams {
                font_size: 11,
                color: colors::TEXT_DIM,
                ..Default::default()
            }
        );
    }

    None
}

/// Actions from the city map UI
#[derive(Clone, Debug)]
pub enum CityMapAction {
    SelectNeighborhood(u32),
    SelectBuilding(usize),
    OpenMarket,
    CloseMarket,
    PurchaseBuilding(u32),
    EnterBuilding(usize),
}


// Helper functions
fn draw_progress_bar(x: f32, y: f32, width: f32, height: f32, progress: f32, color: Color) {
    draw_rectangle(x, y, width, height, Color::from_rgba(40, 40, 45, 255));
    draw_rectangle(x, y, width * progress, height, color);
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
