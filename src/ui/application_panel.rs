use macroquad::prelude::*;
use crate::tenant::TenantApplication;
use crate::building::Building;
use super::{common::*, UiAction};

pub fn draw_application_panel(
    applications: &[TenantApplication],
    building: &Building,
) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;
    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Applications");
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    if applications.is_empty() {
        draw_text("No pending applications", content_x, y, 18.0, colors::TEXT_DIM);
        draw_text("Improve your building to attract tenants!", content_x, y + 25.0, 14.0, colors::TEXT_DIM);
        return None;
    }
    
    draw_text(&format!("{} pending", applications.len()), content_x, y, 16.0, colors::TEXT_DIM);
    y += 25.0;
    
    for (i, app) in applications.iter().enumerate() {
        if y > panel_y + panel_h - 100.0 {
            draw_text("... more applications", content_x, y, 14.0, colors::TEXT_DIM);
            break;
        }
        
        // Application card
        let card_h = 95.0;
        draw_rectangle(content_x, y, panel_w - 30.0, card_h, colors::PANEL_HEADER);
        draw_rectangle_lines(content_x, y, panel_w - 30.0, card_h, 1.0, colors::TEXT_DIM);
        
        // Tenant info
        draw_text(&app.tenant.name, content_x + 10.0, y + 22.0, 18.0, colors::TEXT);
        draw_text(
            &format!("{:?}", app.tenant.archetype),
            content_x + 10.0,
            y + 42.0,
            14.0,
            colors::TEXT_DIM,
        );
        
        // Target apartment
        if let Some(apt) = building.get_apartment(app.apartment_id) {
            draw_text(
                &format!("-> Unit {}", apt.unit_number),
                content_x + 150.0,
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
            content_x + 150.0,
            y + 42.0,
            14.0,
            score_color,
        );
        
        // Accept/Reject buttons (mouse-clickable)
        let btn_y = y + 58.0;
        let btn_w = 80.0;
        
        if button(content_x + 10.0, btn_y, btn_w, 28.0, "Accept", true) {
            action = Some(UiAction::AcceptApplication { application_index: i });
        }
        
        if button(content_x + 100.0, btn_y, btn_w, 28.0, "Reject", true) {
            action = Some(UiAction::RejectApplication { application_index: i });
        }
        
        y += card_h + 10.0;
    }
    
    action
}
