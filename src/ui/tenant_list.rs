use macroquad::prelude::*;
use crate::tenant::Tenant;
use super::{common::*, UiAction};
use crate::assets::AssetManager;

pub fn draw_tenant_list(tenants: &[Tenant], assets: &AssetManager) -> Option<UiAction> {
    let mut action = None;
    
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;
    let panel_h = screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;
    
    panel(panel_x, panel_y, panel_w, panel_h, "Tenants");
    
    let content_x = panel_x + 15.0;
    let mut y = panel_y + 50.0;
    
    if tenants.is_empty() {
        draw_text("No tenants yet", content_x, y, 18.0, colors::TEXT_DIM);
        return None;
    }
    
    for tenant in tenants {
        if y > panel_y + panel_h - 60.0 {
            draw_text("... more tenants", content_x, y, 14.0, colors::TEXT_DIM);
            break;
        }
        
        // Tenant row (clickable)
        let row_h = 50.0;
        let row_hovered = is_hovered(content_x, y, panel_w - 30.0, row_h);
        let row_color = if row_hovered { colors::HOVERED } else { colors::PANEL_HEADER };
        
        draw_rectangle(content_x, y, panel_w - 30.0, row_h, row_color);
        draw_rectangle_lines(content_x, y, panel_w - 30.0, row_h, 1.0, colors::TEXT_DIM);
        
        // Tenant Portrait (Little icon)
        let portrait_id = format!("tenant_{}", format!("{:?}", tenant.archetype).to_lowercase());
        let has_portrait = if let Some(tex) = assets.get_texture(&portrait_id) {
            draw_texture_ex(tex, content_x + 5.0, y + 5.0, WHITE, DrawTextureParams {
                dest_size: Some(Vec2::new(40.0, 40.0)),
                ..Default::default()
            });
            true
        } else {
            false
        };
        
        let text_x = if has_portrait { content_x + 50.0 } else { content_x + 10.0 };
        
        // Name
        draw_text(&tenant.name, text_x, y + 22.0, 18.0, colors::TEXT);
        
        // Archetype
        draw_text(
            &format!("{:?}", tenant.archetype),
            text_x,
            y + 40.0,
            12.0,
            colors::TEXT_DIM,
        );
        
        // Happiness bar
        let happy_color = happiness_color(tenant.happiness);
        progress_bar(content_x + 160.0, y + 18.0, 80.0, 12.0, tenant.happiness as f32, 100.0, happy_color);
        
        // Click handler
        if was_clicked(content_x, y, panel_w - 30.0, row_h) {
            action = Some(UiAction::SelectTenant(tenant.id));
        }
        
        y += row_h + 5.0;
    }
    
    action
}
