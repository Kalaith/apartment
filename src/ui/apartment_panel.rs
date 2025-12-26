use macroquad::prelude::*;
use crate::building::{Apartment, Building, DesignType, ApartmentSize, NoiseLevel};
use crate::tenant::Tenant;

use super::{common::*, UiAction};
use crate::assets::AssetManager;

pub fn draw_apartment_panel(
    apt: &Apartment,
    _building: &Building,
    tenants: &[Tenant],
    money: i32,
    offset_x: f32,
    scroll_offset: f32,
    assets: &AssetManager,
) -> (Option<UiAction>, f32) {
    let mut action = None;
    let mut new_scroll = scroll_offset;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING + offset_x;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;

    if panel_x > screen_width() {
        return (None, scroll_offset);
    }
    
    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, &format!("Unit {}", apt.unit_number));
    
    // Handle mouse wheel scrolling when hovering over the panel
    let mouse = mouse_position();
    let is_hovering = mouse.0 >= panel_x && mouse.0 <= panel_x + panel_w
        && mouse.1 >= panel_y && mouse.1 <= panel_y + panel_h;
    
    if is_hovering {
        let wheel = mouse_wheel();
        new_scroll -= wheel.1 * 30.0; // Scroll speed
        new_scroll = new_scroll.max(0.0); // Clamp minimum
    }
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0 - new_scroll;
    
    // Calculate content start for clipping
    let content_top = panel_y + 35.0; // Below header
    let content_bottom = panel_y + panel_h - 10.0;
    
    // === Stats Section ===
    if y + 20.0 > content_top && y < content_bottom {
        draw_text("CONDITION", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 5.0;
    if y + 20.0 > content_top && y < content_bottom {
        let cond_color = condition_color(apt.condition);
        progress_bar(content_x, y, panel_w - 30.0, 20.0, apt.condition as f32, 100.0, cond_color);
        // Condition Icon
        if let Some(icon) = if apt.condition > 50 { assets.get_texture("icon_condition_good") } else { assets.get_texture("icon_condition_poor") } {
            draw_texture_ex(icon, content_x + panel_w - 60.0, y - 2.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
        draw_text(&format!("{}%", apt.condition), content_x + panel_w - 110.0, y + 15.0, 16.0, colors::TEXT);
    }
    y += 35.0;
    
    // Design
    if y > content_top && y < content_bottom {
        let design_text = match apt.design {
            DesignType::Bare => "Bare",
            DesignType::Practical => "Practical",
            DesignType::Cozy => "Cozy",
        };
        draw_text(&format!("Design: {}", design_text), content_x, y, 18.0, colors::TEXT);
    }
    y += 25.0;
    
    // Size
    if y > content_top && y < content_bottom {
        let size_text = match apt.size {
            ApartmentSize::Small => "Small",
            ApartmentSize::Medium => "Medium",
        };
        draw_text(&format!("Size: {}", size_text), content_x, y, 18.0, colors::TEXT);
    }
    y += 25.0;
    
    // Noise
    if y > content_top && y < content_bottom {
        let noise_text = match apt.effective_noise() {
            NoiseLevel::Low => "Quiet",
            NoiseLevel::High => "Noisy",
        };
        let noise_color = if matches!(apt.effective_noise(), NoiseLevel::High) {
            colors::WARNING
        } else {
            colors::POSITIVE
        };
        draw_text(&format!("Noise: {}", noise_text), content_x, y, 18.0, noise_color);
        if let Some(icon) = assets.get_texture("icon_noise") {
            draw_texture_ex(icon, content_x + 120.0, y - 15.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
    }
    y += 25.0;
    
    // Soundproofing
    if apt.has_soundproofing {
        if y > content_top && y < content_bottom {
            draw_text("Soundproofed", content_x, y, 16.0, colors::POSITIVE);
            if let Some(icon) = assets.get_texture("icon_soundproofing") {
                 draw_texture_ex(icon, content_x + 110.0, y - 15.0, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                });
            }
        }
        y += 25.0;
    }
    
    // Rent
    if y > content_top && y < content_bottom {
        draw_text(&format!("Rent: ${}/mo", apt.rent_price), content_x, y, 20.0, colors::ACCENT);
        if let Some(icon) = assets.get_texture("icon_rent") {
             draw_texture_ex(icon, content_x + 150.0, y - 18.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
    }
    y += 35.0;
    
    // Quality score
    if y > content_top && y < content_bottom {
        let quality = apt.quality_score();
        draw_text(&format!("Quality Score: {}", quality), content_x, y, 16.0, colors::TEXT_DIM);
    }
    y += 40.0;
    
    // === Tenant Info ===
    if y > content_top && y < content_bottom {
        draw_line(content_x, y, content_x + panel_w - 30.0, y, 1.0, colors::TEXT_DIM);
    }
    y += 15.0;
    
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            if y > content_top && y < content_bottom {
                draw_text("TENANT", content_x, y, 14.0, colors::TEXT_DIM);
            }
            y += 20.0;
            
            // Portrait
            let portrait_id = format!("tenant_{}", tenant.archetype.name().to_lowercase());
            let has_portrait = if let Some(tex) = assets.get_texture(&portrait_id) {
                if y + 80.0 > content_top && y < content_bottom {
                    draw_texture_ex(tex, content_x, y, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2::new(80.0, 80.0)),
                        ..Default::default()
                    });
                }
                true
            } else {
                false
            };
            
            let text_x = if has_portrait { content_x + 90.0 } else { content_x + 10.0 };

            if y + 40.0 > content_top && y < content_bottom {
                // Colored strip for archetype (keep it even with portrait as a backup or accent)
                if !has_portrait {
                     draw_rectangle(content_x, y, 4.0, 20.0, archetype_color(&tenant.archetype));
                }
                
                draw_text(&tenant.name, text_x, y + 16.0, 20.0, colors::TEXT);
                draw_text(tenant.archetype.name(), text_x, y + 36.0, 16.0, colors::TEXT_DIM);
            }
            
            let _months_y = if has_portrait { y + 85.0 } else { y + 60.0 };
             // Use original y increment if no portrait
             if has_portrait {
                 y += 85.0; 
             } else {
                 y += 60.0;
             }

            if y > content_top && y < content_bottom {
                draw_text("Happiness", content_x, y, 14.0, colors::TEXT_DIM);
            }
            y += 5.0;
            if y + 16.0 > content_top && y < content_bottom {
                let happy_color = happiness_color(tenant.happiness);
                progress_bar(content_x, y, panel_w - 60.0, 16.0, tenant.happiness as f32, 100.0, happy_color);
                
                // Icon next to bar
                let happiness_level = if tenant.happiness >= 90 { "happiness_ecstatic" }
                else if tenant.happiness >= 70 { "happiness_happy" }
                else if tenant.happiness >= 40 { "happiness_neutral" }
                else if tenant.happiness >= 20 { "happiness_unhappy" }
                else { "happiness_miserable" };
                
                if let Some(icon) = assets.get_texture(happiness_level) {
                    draw_texture_ex(icon, content_x + panel_w - 55.0, y - 4.0, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2::new(24.0, 24.0)),
                        ..Default::default()
                    });
                } else {
                     let icon_char = happiness_icon(tenant.happiness);
                     draw_text(icon_char, content_x + panel_w - 50.0, y + 14.0, 20.0, colors::TEXT);
                }
            }

            y += 25.0;
            
            if y > content_top && y < content_bottom {
                draw_text(&format!("Months: {}", tenant.months_residing), content_x, y, 14.0, colors::TEXT_DIM);
            }
            y += 30.0;
        }
    } else {
        if y > content_top && y < content_bottom {
            draw_text("VACANT", content_x, y, 18.0, colors::WARNING);
        }
        y += 25.0;
        
        // Button to jump to applications
        if y + 30.0 > content_top && y < content_bottom {
            if button(content_x, y, panel_w - 30.0, 30.0, "View Applications", true) {
                action = Some(UiAction::SelectApplications);
            }
        }
        y += 40.0;
    }
    
    // === Upgrade Buttons (mouse-clickable) ===
    if y > content_top && y < content_bottom {
        draw_line(content_x, y, content_x + panel_w - 30.0, y, 1.0, colors::TEXT_DIM);
    }
    y += 15.0;
    
    if y > content_top && y < content_bottom {
        draw_text("UPGRADES", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 25.0;
    
    let btn_w = panel_w - 30.0;
    let btn_h = 36.0;
    
    // Dynamic upgrades from building/upgrades.rs
    let available = crate::building::upgrades::available_apartment_upgrades(apt);
    
    // Track total content height for scroll clamping
    let upgrades_start_y = y;
    let mut total_upgrade_height = 0.0;
    
    for upgrade in &available {
        if upgrade.cost(_building).is_some() {
            total_upgrade_height += btn_h + 8.0;
        }
    }
    
    // Clamp scroll to prevent scrolling past content
    let max_scroll = (upgrades_start_y + total_upgrade_height - content_bottom + new_scroll).max(0.0);
    new_scroll = new_scroll.min(max_scroll);
    
    for upgrade in available {
        if let Some(cost) = upgrade.cost(_building) {
            let can_afford = money >= cost;
            let label = format!("{} (${})", upgrade.label(_building), cost);
            
            // Only draw and handle clicks if visible
            if y + btn_h > content_top && y < content_bottom {
                if button(content_x, y, btn_w, btn_h, &label, can_afford) {
                    action = Some(UiAction::UpgradeAction(upgrade));
                }
            }
            y += btn_h + 8.0;
        }
    }
    
    // Draw scroll indicator if there's more content
    if max_scroll > 0.0 {
        let scroll_hint_y = content_bottom - 20.0;
        if new_scroll < max_scroll - 5.0 {
            draw_text("▼ Scroll for more", content_x, scroll_hint_y, 12.0, colors::TEXT_DIM);
        }
    }
    
    (action, new_scroll)
}

pub fn draw_hallway_panel(building: &Building, money: i32, offset_x: f32, assets: &AssetManager) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING + offset_x;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;

    if panel_x > screen_width() {
        return None;
    }

    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Hallway");
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    draw_text("CONDITION", content_x, y, 14.0, colors::TEXT_DIM);
    y += 5.0;
    let cond_color = condition_color(building.hallway_condition);
    progress_bar(content_x, y, panel_w - 30.0, 20.0, building.hallway_condition as f32, 100.0, cond_color);
     if let Some(icon) = if building.hallway_condition > 50 { assets.get_texture("icon_condition_good") } else { assets.get_texture("icon_condition_poor") } {
        draw_texture_ex(icon, content_x + panel_w - 60.0, y - 2.0, WHITE, DrawTextureParams {
            dest_size: Some(Vec2::new(24.0, 24.0)),
            ..Default::default()
        });
    }

    draw_text(&format!("{}%", building.hallway_condition), content_x + panel_w - 110.0, y + 15.0, 16.0, colors::TEXT);
    y += 45.0;
    
    draw_text("Affects overall building appeal", content_x, y, 14.0, colors::TEXT_DIM);
    y += 20.0;
    
    let appeal = building.building_appeal();
    draw_text(&format!("Building Appeal: {}", appeal), content_x, y, 18.0, colors::ACCENT);
    y += 50.0;
    
    // Dynamic upgrades
    let available = crate::building::upgrades::available_building_upgrades(building);
    
    let btn_w = panel_w - 30.0;
    
    for upgrade in available {
        if let Some(cost) = upgrade.cost(building) {
            let can_afford = money >= cost;
            let label = format!("{} (${})", upgrade.label(building), cost);
            
            if button(content_x, y, btn_w, 36.0, &label, can_afford) {
                action = Some(UiAction::UpgradeAction(upgrade));
            }
            y += 44.0;
        }
    }
    
    action
}
