use macroquad::prelude::*;
use crate::tenant::TenantApplication;
use crate::building::Building;
use super::{common::*, UiAction};
use crate::assets::AssetManager;

pub fn draw_application_panel(
    applications: &[TenantApplication],
    building: &Building,
    filter_apartment_id: Option<u32>,
    offset_x: f32,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING + offset_x;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;

    if panel_x > screen_width() {
        return None;
    }

    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Applications");
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    let filtered_apps: Vec<(usize, &TenantApplication)> = applications.iter()
        .enumerate()
        .filter(|(_, app)| filter_apartment_id.map_or(true, |id| app.apartment_id == id))
        .collect();
    
    if filtered_apps.is_empty() {
        // Debug info
        if !applications.is_empty() {
             println!("DEBUG UI: Has {} total apps, but 0 after filter {:?}", applications.len(), filter_apartment_id);
             for app in applications {
                 println!(" - App for unit with ID: {}", app.apartment_id);
             }
        }
        
        if filter_apartment_id.is_some() {
            draw_text("No applications for this unit", content_x, y, 18.0, colors::TEXT_DIM);
        } else {
            draw_text("No pending applications", content_x, y, 18.0, colors::TEXT_DIM);
            draw_text("List apartments for lease, then End Month!", content_x, y + 25.0, 14.0, colors::TEXT_DIM);
        }
        return None;
    }
    
    draw_text(&format!("{} pending", filtered_apps.len()), content_x, y, 16.0, colors::TEXT_DIM);
    y += 25.0;
    
    for (i, app) in filtered_apps {
        if y > panel_y + panel_h - 100.0 {
            draw_text("... more applications", content_x, y, 14.0, colors::TEXT_DIM);
            break;
        }
        
        // Application card
        let card_h = 95.0;
        draw_rectangle(content_x, y, panel_w - 30.0, card_h, colors::PANEL_HEADER);
        draw_rectangle_lines(content_x, y, panel_w - 30.0, card_h, 1.0, colors::TEXT_DIM);
        
        // Tenant Portrait
        let portrait_id = format!("tenant_{}", format!("{:?}", app.tenant.archetype).to_lowercase());
        let has_portrait = if let Some(tex) = assets.get_texture(&portrait_id) {
            draw_texture_ex(tex, content_x + 5.0, y + 5.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(80.0, 80.0)),
                ..Default::default()
            });
            true
        } else {
            false
        };

        let text_x = if has_portrait { content_x + 95.0 } else { content_x + 10.0 };
        
        // Tenant info
        draw_text(&app.tenant.name, text_x, y + 22.0, 18.0, colors::TEXT);
        draw_text(
            &format!("{:?}", app.tenant.archetype),
            text_x,
            y + 42.0,
            14.0,
            colors::TEXT_DIM,
        );
        
        // Target apartment
        if let Some(apt) = building.get_apartment(app.apartment_id) {
            draw_text(
                &format!("-> Unit {}", apt.unit_number),
                text_x + 140.0,
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
            text_x + 140.0,
            y + 42.0,
            14.0,
            score_color,
        );
        
        // Accept/Reject buttons (mouse-clickable)
        let btn_y = y + 58.0;
        let btn_w = 80.0;
        
        if button(text_x, btn_y, btn_w, 28.0, "Accept", true) {
            action = Some(UiAction::AcceptApplication { application_index: i });
        }
        
        if button(text_x + 90.0, btn_y, btn_w, 28.0, "Reject", true) {
            action = Some(UiAction::RejectApplication { application_index: i });
        }
        
        y += card_h + 10.0;
    }
    
    action
}
