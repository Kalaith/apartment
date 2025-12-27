use macroquad::prelude::*;
use crate::building::{Apartment, Building, DesignType, ApartmentSize, NoiseLevel};
use crate::tenant::Tenant;

use super::{common::*, UiAction};
use crate::assets::AssetManager;
use crate::consequences::TenantNetwork;
use crate::data::config::RelationshipsConfig;
use crate::narrative::{TenantStory, TenantRequest};
use std::collections::HashMap;

pub fn draw_apartment_panel(
    apt: &Apartment,
    building: &Building,
    tenants: &[Tenant],
    money: i32,
    offset_x: f32,
    scroll_offset: f32,
    assets: &AssetManager,
    config: &crate::data::config::GameConfig,
    tenant_network: &TenantNetwork,
    stories: &HashMap<u32, TenantStory>,
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
    
    // Check if this unit has been sold as a condo
    if building.is_unit_sold(apt.id) {
        if let Some(act) = draw_sold_condo_panel(apt, building, money, panel_x, panel_y, panel_w, panel_h) {
             action = Some(act);
        }
        return (action, new_scroll);
    }
    
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
    draw_apartment_stats(apt, assets, content_x, &mut y, panel_w, content_top, content_bottom);
    
    // === Tenant Info ===
    if let Some(act) = draw_tenant_info(apt, tenants, assets, content_x, &mut y, panel_w, content_top, content_bottom, tenant_network, &config.relationships, stories) {
        action = Some(act);
    }
    
    // === Upgrade Buttons ===
    let (upgrade_action, scroll_result) = draw_upgrades(
        apt, building, money, 
        content_x, &mut y, panel_w, 
        content_top, content_bottom, new_scroll,
        config
    );
     if let Some(act) = upgrade_action {
        action = Some(act);
    }
    new_scroll = scroll_result;
    
    (action, new_scroll)
}

pub fn draw_hallway_panel(building: &Building, money: i32, offset_x: f32, scroll_offset: f32, assets: &AssetManager, config: &crate::data::config::GameConfig) -> (Option<UiAction>, f32) {
    let mut action = None;
    let mut new_scroll = scroll_offset;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING + offset_x;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;

    if panel_x > screen_width() {
        return (None, scroll_offset);
    }

    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Hallway");
    
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
    
    // Clipping bounds
    let content_top = panel_y + 40.0;
    let content_bottom = panel_y + panel_h - 10.0;
    
    let mut y = panel_y + 50.0 - new_scroll;
    
    if y + 20.0 > content_top && y < content_bottom {
        draw_text("CONDITION", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 5.0;
    
    if y + 20.0 > content_top && y < content_bottom {
        let cond_color = condition_color(building.hallway_condition);
        progress_bar(content_x, y, panel_w - 30.0, 20.0, building.hallway_condition as f32, 100.0, cond_color);
         if let Some(icon) = if building.hallway_condition > 50 { assets.get_texture("icon_condition_good") } else { assets.get_texture("icon_condition_poor") } {
            draw_texture_ex(icon, content_x + panel_w - 60.0, y - 2.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
        draw_text(&format!("{}%", building.hallway_condition), content_x + panel_w - 110.0, y + 15.0, 16.0, colors::TEXT);
    }
    y += 45.0;
    
    if y + 14.0 > content_top && y < content_bottom {
        draw_text("Affects overall building appeal", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 20.0;
    
    if y + 18.0 > content_top && y < content_bottom {
        let appeal = building.building_appeal();
        draw_text(&format!("Building Appeal: {}", appeal), content_x, y, 18.0, colors::ACCENT);
    }
    y += 50.0;
    
    // Staff Display
    if y + 14.0 > content_top && y < content_bottom {
        draw_text("STAFF", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 25.0;

    let mut staff_count = 0;
    // Sort keys for consistent display order
    let mut staff_types: Vec<_> = config.economy.staff_costs.keys().collect();
    staff_types.sort();

    for staff_type in staff_types {
        let cost = config.economy.staff_costs.get(staff_type).unwrap();
        let flag = format!("staff_{}", staff_type);
        
        if building.flags.contains(&flag) {
            // Capitalize first letter for display
            let mut chars = staff_type.chars();
            let label = match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            };
            
            if y + 16.0 > content_top && y < content_bottom {
                draw_text(&format!("{} (${}/mo)", label, cost), content_x, y, 16.0, colors::TEXT);
            }
            y += 25.0;
            staff_count += 1;
        }
    }
    
    if staff_count == 0 {
        if y + 16.0 > content_top && y < content_bottom {
            draw_text("None hired", content_x, y, 16.0, colors::TEXT_DIM);
        }
        y += 25.0;
    }
    
    y += 25.0;
    
    // Dynamic upgrades
    let available = crate::building::upgrades::available_building_upgrades(building, &config.upgrades);
    
    let mut staff_actions = Vec::new();
    let mut other_actions = Vec::new();
    
    for upgrade in available {
        let is_staff = match &upgrade {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => {
                if let Some(def) = config.upgrades.get(upgrade_id) {
                     def.effects.iter().any(|e| match e {
                        crate::data::config::UpgradeEffect::SetFlag(f) | crate::data::config::UpgradeEffect::RemoveFlag(f) => f.starts_with("staff_"),
                        _ => false
                     })
                } else { false }
            },
            _ => false
        };
        
        if is_staff {
            staff_actions.push(upgrade);
        } else {
            other_actions.push(upgrade);
        }
    }
    
    // Sort staff actions: Hires first, Fires last
    staff_actions.sort_by(|a, b| {
        let a_id = match a { crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id, _ => "" };
        let b_id = match b { crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id, _ => "" };
        
        let a_is_fire = a_id.contains("fire");
        let b_is_fire = b_id.contains("fire");
        
        if a_is_fire && !b_is_fire { std::cmp::Ordering::Greater }
        else if !a_is_fire && b_is_fire { std::cmp::Ordering::Less }
        else { a_id.cmp(b_id) }
    });
    
    let btn_w = panel_w - 30.0;

    // Draw Staff Actions
    if !staff_actions.is_empty() {
        for upgrade in staff_actions {
            if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
                let can_afford = money >= cost;
                let label = format!("{} (${})", upgrade.label(building, &config.ui, &config.upgrades), cost);
                
                if y + 36.0 > content_top && y < content_bottom {
                    if button(content_x, y, btn_w, 36.0, &label, can_afford) {
                        action = Some(UiAction::UpgradeAction(upgrade));
                    }
                }
                y += 44.0;
            }
        }
    }

    if y + 14.0 > content_top && y < content_bottom {
        draw_text("UPGRADES", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 25.0;
    
    // Draw Other Upgrades
    for upgrade in other_actions {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!("{} (${})", upgrade.label(building, &config.ui, &config.upgrades), cost);
            
            if y + 36.0 > content_top && y < content_bottom {
                if button(content_x, y, btn_w, 36.0, &label, can_afford) {
                    action = Some(UiAction::UpgradeAction(upgrade));
                }
            }
            y += 44.0;
        }
    }
    
    (action, new_scroll)
}

fn draw_sold_condo_panel(
    apt: &Apartment,
    building: &Building,
    money: i32,
    panel_x: f32, 
    panel_y: f32, 
    panel_w: f32, 
    panel_h: f32
) -> Option<UiAction> {
    panel(panel_x, panel_y, panel_w, panel_h, &format!("Unit {} - SOLD", apt.unit_number));
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 60.0;
    
    draw_text("CONDO - PRIVATELY OWNED", content_x, y, 20.0, colors::WARNING);
    y += 35.0;
    
    if let Some((owner_name, purchase_price)) = building.get_condo_info(apt.id) {
        draw_text(&format!("Owner: {}", owner_name), content_x, y, 18.0, colors::TEXT);
        y += 25.0;
        draw_text(&format!("Purchased for: ${}", purchase_price), content_x, y, 16.0, colors::TEXT_DIM);
        y += 35.0;
        
        let buyback_price = (purchase_price as f32 * 1.1) as i32;
        draw_text("Buyback Option:", content_x, y, 16.0, colors::ACCENT);
        y += 25.0;
        
        let can_afford = money >= buyback_price;
        let btn_label = format!("Buy Back (${}) ", buyback_price);
        
        if button(content_x, y, panel_w - 30.0, 35.0, &btn_label, can_afford) {
            return Some(UiAction::BuybackCondo { apartment_id: apt.id });
        }
        y += 45.0;
        
        if !can_afford {
            draw_text("Insufficient funds", content_x, y, 14.0, colors::NEGATIVE);
        }
    }
    
    y += 30.0;
    draw_text("You cannot modify or rent this unit", content_x, y, 14.0, colors::TEXT_DIM);
    y += 20.0;
    draw_text("while it is privately owned.", content_x, y, 14.0, colors::TEXT_DIM);
    
    None
}

fn draw_apartment_stats(
    apt: &Apartment,
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32
) {
    if *y + 20.0 > content_top && *y < content_bottom {
        draw_text("CONDITION", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 5.0;
    if *y + 20.0 > content_top && *y < content_bottom {
        let cond_color = condition_color(apt.condition);
        progress_bar(content_x, *y, panel_w - 30.0, 20.0, apt.condition as f32, 100.0, cond_color);
        if let Some(icon) = if apt.condition > 50 { assets.get_texture("icon_condition_good") } else { assets.get_texture("icon_condition_poor") } {
            draw_texture_ex(icon, content_x + panel_w - 60.0, *y - 2.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
        draw_text(&format!("{}%", apt.condition), content_x + panel_w - 110.0, *y + 15.0, 16.0, colors::TEXT);
    }
    *y += 35.0;
    
    // Design
    if *y > content_top && *y < content_bottom {
        let design_text = match apt.design {
            DesignType::Bare => "Bare",
            DesignType::Practical => "Practical",
            DesignType::Cozy => "Cozy",
        };
        draw_text(&format!("Design: {}", design_text), content_x, *y, 18.0, colors::TEXT);
    }
    *y += 25.0;
    
    // Size
    if *y > content_top && *y < content_bottom {
        let size_text = match apt.size {
            ApartmentSize::Small => "Small",
            ApartmentSize::Medium => "Medium",
        };
        draw_text(&format!("Size: {}", size_text), content_x, *y, 18.0, colors::TEXT);
    }
    *y += 25.0;
    
    // Noise
    if *y > content_top && *y < content_bottom {
        let noise_text = match apt.effective_noise() {
            NoiseLevel::Low => "Quiet",
            NoiseLevel::High => "Noisy",
        };
        let noise_color = if matches!(apt.effective_noise(), NoiseLevel::High) {
            colors::WARNING
        } else {
            colors::POSITIVE
        };
        draw_text(&format!("Noise: {}", noise_text), content_x, *y, 18.0, noise_color);
        if let Some(icon) = assets.get_texture("icon_noise") {
            draw_texture_ex(icon, content_x + 120.0, *y - 15.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
    }
    *y += 25.0;
    
    // Soundproofing
    if apt.has_soundproofing {
        if *y > content_top && *y < content_bottom {
            draw_text("Soundproofed", content_x, *y, 16.0, colors::POSITIVE);
            if let Some(icon) = assets.get_texture("icon_soundproofing") {
                 draw_texture_ex(icon, content_x + 110.0, *y - 15.0, WHITE, DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                });
            }
        }
        *y += 25.0;
    }
    
    // Rent
    if *y > content_top && *y < content_bottom {
        draw_text(&format!("Rent: ${}/mo", apt.rent_price), content_x, *y, 20.0, colors::ACCENT);
        if let Some(icon) = assets.get_texture("icon_rent") {
             draw_texture_ex(icon, content_x + 150.0, *y - 18.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(24.0, 24.0)),
                ..Default::default()
            });
        }
    }
    *y += 35.0;
    
    // Quality score
    if *y > content_top && *y < content_bottom {
        let quality = apt.quality_score();
        draw_text(&format!("Quality Score: {}", quality), content_x, *y, 16.0, colors::TEXT_DIM);
    }
    *y += 40.0;
}

fn draw_tenant_info(
    apt: &Apartment,
    tenants: &[Tenant],
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    network: &TenantNetwork,
    _config: &RelationshipsConfig,
    stories: &HashMap<u32, TenantStory>,
) -> Option<UiAction> {
    if *y > content_top && *y < content_bottom {
        draw_line(content_x, *y, content_x + panel_w - 30.0, *y, 1.0, colors::TEXT_DIM);
    }
    *y += 15.0;
    
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            if *y > content_top && *y < content_bottom {
                draw_text("TENANT", content_x, *y, 14.0, colors::TEXT_DIM);
            }
            *y += 20.0;
            
            // Portrait logic
            let portrait_id = format!("tenant_{}", tenant.archetype.name().to_lowercase());
            let has_portrait = if let Some(tex) = assets.get_texture(&portrait_id) {
                if *y + 80.0 > content_top && *y < content_bottom {
                    draw_texture_ex(tex, content_x, *y, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2::new(80.0, 80.0)),
                        ..Default::default()
                    });
                }
                true
            } else {
                false
            };
            
            let text_x = if has_portrait { content_x + 90.0 } else { content_x + 10.0 };

            if *y + 40.0 > content_top && *y < content_bottom {
                if !has_portrait {
                     draw_rectangle(content_x, *y, 4.0, 20.0, archetype_color(&tenant.archetype));
                }
                
                draw_text(&tenant.name, text_x, *y + 16.0, 20.0, colors::TEXT);
                draw_text(tenant.archetype.name(), text_x, *y + 36.0, 16.0, colors::TEXT_DIM);
                
                // Show relationships
                let relationships: Vec<_> = network.relationships.iter()
                    .filter(|r| r.tenant_a_id == tenant.id || r.tenant_b_id == tenant.id)
                    .collect();
                
                if !relationships.is_empty() {
                    let mut icon_x = text_x;
                    let icon_y = *y + 45.0;
                    
                    for rel in relationships.iter().take(4) {
                        let _other_id = if rel.tenant_a_id == tenant.id { rel.tenant_b_id } else { rel.tenant_a_id };
                        let icon = match rel.relationship_type {
                            crate::consequences::RelationshipType::Friendly => "💚",
                            crate::consequences::RelationshipType::Hostile => "⚡",
                            crate::consequences::RelationshipType::Romantic => "💕",
                            crate::consequences::RelationshipType::Family => "👨‍👩‍👧",
                            crate::consequences::RelationshipType::Neutral => "⚪",
                        };
                        
                        draw_text(&format!("{}", icon), icon_x, icon_y + 15.0, 16.0, WHITE);
                        icon_x += 25.0;
                    }
                    if relationships.len() > 4 {
                        draw_text("+", icon_x, icon_y + 15.0, 14.0, colors::TEXT_DIM);
                    }
                }
                
                // Show pending request
                if let Some(story) = stories.get(&tenant.id) {
                    if let Some(request) = &story.pending_request {
                         *y += 40.0;
                         if *y > content_top && *y < content_bottom {
                             draw_text("PENDING REQUEST", content_x, *y, 14.0, colors::ACCENT);
                             *y += 40.0;
                             
                             let req_text = match request {
                                 TenantRequest::Pet { pet_type } => format!("Can I keep a {}?", pet_type),
                                 TenantRequest::TemporaryGuest { guest_name, duration_months } => format!("Can {} stay for {} months?", guest_name, duration_months),
                                 TenantRequest::HomeBusiness { business_type } => format!("Can I start a {} business?", business_type),
                                 TenantRequest::Modification { description } => format!("Can I {}?", description),
                                 TenantRequest::Sublease => "Can I sublease a room?".to_string(),
                             };
                             
                             draw_text(&req_text, content_x, *y, 16.0, colors::TEXT);
                             *y += 25.0;
                             
                             // Show effects
                             let effect = request.approval_effect();
                             let mut effect_text = String::new();
                             
                             // Simple recursive helper to format text
                             let mut stack = vec![effect];
                             while let Some(e) = stack.pop() {
                                 match e {
                                     crate::narrative::StoryImpact::Happiness(amount) => {
                                         if !effect_text.is_empty() { effect_text.push_str(", "); }
                                         effect_text.push_str(&format!("Happiness {:+}", amount));
                                     },
                                     crate::narrative::StoryImpact::SetApartmentFlag(flag) => {
                                         if !effect_text.is_empty() { effect_text.push_str(", "); }
                                         if flag == "high_noise" {
                                             effect_text.push_str("Noise Increases");
                                         } else {
                                             effect_text.push_str(&format!("Flag: {}", flag));
                                         }
                                     },
                                     crate::narrative::StoryImpact::Multiple(list) => {
                                         for item in list.iter().rev() {
                                             stack.push(item.clone());
                                         }
                                     },
                                     _ => {}
                                 }
                             }
                             
                             if !effect_text.is_empty() {
                                 draw_text(&format!("Effect: {}", effect_text), content_x, *y, 14.0, colors::ACCENT);
                                 *y += 25.0;
                             }

                             // Approve/Deny Buttons
                             if crate::ui::common::colored_button(content_x, *y, 100.0, 30.0, "APPROVE", true, colors::POSITIVE, colors::TEXT_BRIGHT) {
                                 return Some(UiAction::ApproveRequest { tenant_id: tenant.id });
                             }
                             
                             if crate::ui::common::colored_button(content_x + 110.0, *y, 100.0, 30.0, "DENY", true, colors::NEGATIVE, colors::TEXT_BRIGHT) {
                                 return Some(UiAction::DenyRequest { tenant_id: tenant.id });
                             }
                             
                             *y += 35.0;
                         }
                    }
                }
            }
            
            if has_portrait {
                 *y += 85.0; 
            } else {
                 *y += 75.0; // Increased to make room for relationship icons
            }

            if *y > content_top && *y < content_bottom {
                draw_text("Happiness", content_x, *y, 14.0, colors::TEXT_DIM);
            }
            *y += 5.0;
            if *y + 16.0 > content_top && *y < content_bottom {
                let happy_color = happiness_color(tenant.happiness);
                progress_bar(content_x, *y, panel_w - 60.0, 16.0, tenant.happiness as f32, 100.0, happy_color);
                
                // Icon next to bar
                let happiness_level = if tenant.happiness >= 90 { "happiness_ecstatic" }
                else if tenant.happiness >= 70 { "happiness_happy" }
                else if tenant.happiness >= 40 { "happiness_neutral" }
                else if tenant.happiness >= 20 { "happiness_unhappy" }
                else { "happiness_miserable" };
                
                if let Some(icon) = assets.get_texture(happiness_level) {
                    draw_texture_ex(icon, content_x + panel_w - 55.0, *y - 4.0, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2::new(24.0, 24.0)),
                        ..Default::default()
                    });
                } else {
                     let icon_char = happiness_icon(tenant.happiness);
                     draw_text(icon_char, content_x + panel_w - 50.0, *y + 14.0, 20.0, colors::TEXT);
                }
            }

            *y += 25.0;
            
            if *y > content_top && *y < content_bottom {
                draw_text(&format!("Months: {}", tenant.months_residing), content_x, *y, 14.0, colors::TEXT_DIM);
            }
            *y += 30.0;
        }
    } else {
        if *y > content_top && *y < content_bottom {
            draw_text("VACANT", content_x, *y, 18.0, colors::WARNING);
        }
        *y += 25.0;
        
        let btn_w = panel_w - 30.0;
        
        // Listing Status
        if apt.is_listed_for_lease {
             if *y > content_top && *y < content_bottom {
                draw_text("Status: LISTED", content_x, *y, 16.0, colors::POSITIVE);
            }
             *y += 20.0;
             
            if *y > content_top && *y < content_bottom {
                let target_text = if let Some(pref) = &apt.preferred_archetype {
                    format!("Target: {}", pref.name())
                } else {
                    "Target: Open (Any)".to_string()
                };
                draw_text(&target_text, content_x, *y, 14.0, colors::TEXT);
            }
            *y += 30.0;
             
            if *y + 30.0 > content_top && *y < content_bottom {
                if button(content_x, *y, btn_w, 30.0, "View Applications", true) {
                    return Some(UiAction::SelectApplications(Some(apt.id)));
                }
            }
            *y += 35.0;
            
            if *y + 30.0 > content_top && *y < content_bottom {
                if button(content_x, *y, btn_w, 30.0, "Unlist Property", true) {
                    return Some(UiAction::UnlistApartment { apartment_id: apt.id });
                }
            }
            *y += 40.0;
            
        } else {
             if *y > content_top && *y < content_bottom {
                draw_text("Status: OFF MARKET", content_x, *y, 14.0, colors::TEXT_DIM);
            }
             *y += 30.0;
             
             // Rent Control
             if *y > content_top && *y < content_bottom {
                 draw_text(&format!("Rent: ${}", apt.rent_price), content_x, *y, 20.0, colors::TEXT);
                 
                 let btn_size = 25.0;
                 if button(content_x + 120.0, *y - 18.0, btn_size, btn_size, "-", true) {
                     return Some(UiAction::AdjustRent { apartment_id: apt.id, amount: -50 });
                 }
                 if button(content_x + 150.0, *y - 18.0, btn_size, btn_size, "+", true) {
                     return Some(UiAction::AdjustRent { apartment_id: apt.id, amount: 50 });
                 }
             }
             *y += 40.0;
             
             if *y > content_top && *y < content_bottom {
                 draw_text("List for Lease:", content_x, *y, 14.0, colors::ACCENT);
             }
             *y += 20.0;
             
             // List Any
             if *y + 30.0 > content_top && *y < content_bottom {
                 if button(content_x, *y, btn_w, 30.0, "Any Tenant", true) {
                     return Some(UiAction::ListApartment { apartment_id: apt.id, preference: None });
                 }
             }
             *y += 35.0;
             
             // Targeted Listing
             let types = [
                 (crate::tenant::TenantArchetype::Student, "Student"),
                 (crate::tenant::TenantArchetype::Professional, "Pro"),
                 (crate::tenant::TenantArchetype::Artist, "Artist"),
                 (crate::tenant::TenantArchetype::Family, "Family"),
                 (crate::tenant::TenantArchetype::Elderly, "Elderly"),
             ];
             
             let small_btn_w = (btn_w - 10.0) / 2.0;
             
             for (i, (arch, label)) in types.iter().enumerate() {
                 let col = i % 2;
                 let _row = i / 2;
                 
                 // If odd number of items, last one takes full width? Or just left align.
                 // let x = content_x + col as f32 * (small_btn_w + 10.0);
                 // let this_y = *y; 
                 // We need to increment y only after completing a row
                 
                 let x = content_x + col as f32 * (small_btn_w + 10.0);
                 
                 // Only verify drawing if within bounds
                 if *y + 25.0 > content_top && *y < content_bottom {
                     if button(x, *y, small_btn_w, 25.0, label, true) {
                         return Some(UiAction::ListApartment { apartment_id: apt.id, preference: Some(arch.clone()) });
                     }
                 }
                 
                 if col == 1 || i == types.len() - 1 {
                     *y += 30.0;
                 }
             }
             *y += 10.0;
        }
    }
    None
}

fn draw_upgrades(
    apt: &Apartment,
    building: &Building,
    money: i32,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    current_scroll: f32,
    config: &crate::data::config::GameConfig,
) -> (Option<UiAction>, f32) {
    if *y > content_top && *y < content_bottom {
        draw_line(content_x, *y, content_x + panel_w - 30.0, *y, 1.0, colors::TEXT_DIM);
    }
    *y += 15.0;
    
    if *y > content_top && *y < content_bottom {
        draw_text("UPGRADES", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 25.0;
    
    let btn_w = panel_w - 30.0;
    let btn_h = 36.0;
    
    let available = crate::building::upgrades::available_apartment_upgrades(apt, &config.upgrades);
    
    let upgrades_start_y = *y;
    let mut total_upgrade_height = 0.0;
    
    for upgrade in &available {
        if upgrade.cost(building, &config.economy, &config.upgrades).is_some() {
            total_upgrade_height += btn_h + 8.0;
        }
    }
    
    let max_scroll = (upgrades_start_y + total_upgrade_height - content_bottom + current_scroll).max(0.0);
    let final_scroll = current_scroll.min(max_scroll);
    
    let mut action = None;

    for upgrade in available {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!("{} (${})", upgrade.label(building, &config.ui, &config.upgrades), cost);
            
            if *y + btn_h > content_top && *y < content_bottom {
                if button(content_x, *y, btn_w, btn_h, &label, can_afford) {
                    action = Some(UiAction::UpgradeAction(upgrade));
                }
            }
            *y += btn_h + 8.0;
        }
    }
    
    if max_scroll > 0.0 {
        let scroll_hint_y = content_bottom - 20.0;
        if final_scroll < max_scroll - 5.0 {
            draw_text("▼ Scroll for more", content_x, scroll_hint_y, 12.0, colors::TEXT_DIM);
        }
    }
    
    (action, final_scroll)
}
