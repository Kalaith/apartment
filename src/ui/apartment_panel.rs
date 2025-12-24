use macroquad::prelude::*;
use crate::building::{Apartment, Building, DesignType, ApartmentSize, NoiseLevel};
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
        DesignType::Cozy => "Cozy",
    };
    draw_text(&format!("Design: {}", design_text), content_x, y, 18.0, colors::TEXT);
    y += 25.0;
    
    // Size
    let size_text = match apt.size {
        ApartmentSize::Small => "Small",
        ApartmentSize::Medium => "Medium",
    };
    draw_text(&format!("Size: {}", size_text), content_x, y, 18.0, colors::TEXT);
    y += 25.0;
    
    // Noise
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
    y += 25.0;
    
    // Soundproofing
    if apt.has_soundproofing {
        draw_text("Soundproofed", content_x, y, 16.0, colors::POSITIVE);
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
    
    // === Upgrade Buttons (mouse-clickable) ===
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
        
        let label = format!("Repair +{} (${})", repair_amount, repair_cost);
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
    
    // Repair button (mouse-clickable)
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
