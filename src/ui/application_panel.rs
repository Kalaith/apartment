use super::{common::*, UiAction};
use crate::assets::AssetManager;
use crate::building::Building;
use crate::tenant::TenantApplication;
use macroquad::prelude::*;

pub fn draw_application_panel(
    applications: &[TenantApplication],
    building: &Building,
    filter_apartment_id: Option<u32>,
    offset_x: f32,
    assets: &AssetManager,
) -> Option<UiAction> {
    let panel_rect = application_panel_rect(offset_x)?;
    panel(
        panel_rect.x,
        panel_rect.y,
        panel_rect.w,
        panel_rect.h,
        "Applications",
    );

    let content_x = panel_rect.x + 15.0;
    let mut y = panel_rect.y + 50.0;
    let filtered_apps: Vec<(usize, &TenantApplication)> = applications
        .iter()
        .enumerate()
        .filter(|(_, app)| filter_apartment_id.map_or(true, |id| app.apartment_id == id))
        .collect();

    if filtered_apps.is_empty() {
        draw_empty_applications(content_x, y, filter_apartment_id);
        return None;
    }

    draw_text(
        &format!("{} pending", filtered_apps.len()),
        content_x,
        y,
        16.0,
        colors::TEXT_DIM,
    );
    y += 25.0;

    let mut action = None;
    for (index, application) in filtered_apps {
        if y > panel_rect.y + panel_rect.h - 100.0 {
            draw_text(
                "... more applications",
                content_x,
                y,
                14.0,
                colors::TEXT_DIM,
            );
            break;
        }

        if let Some(card_action) = draw_application_card(
            index,
            application,
            building,
            content_x,
            y,
            panel_rect.w - 30.0,
            assets,
        ) {
            action = Some(card_action);
        }
        y += 135.0;
    }

    action
}

fn application_panel_rect(offset_x: f32) -> Option<Rect> {
    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING + offset_x;
    if panel_x > screen_width() {
        return None;
    }

    Some(Rect::new(
        panel_x,
        layout::HEADER_HEIGHT + layout::PADDING,
        screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0,
        screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0,
    ))
}

fn draw_empty_applications(content_x: f32, y: f32, filter_apartment_id: Option<u32>) {
    if filter_apartment_id.is_some() {
        draw_text(
            "No applications for this unit",
            content_x,
            y,
            18.0,
            colors::TEXT_DIM,
        );
        return;
    }

    draw_text(
        "No pending applications",
        content_x,
        y,
        18.0,
        colors::TEXT_DIM,
    );
    draw_text(
        "List apartments for lease, then End Month!",
        content_x,
        y + 25.0,
        14.0,
        colors::TEXT_DIM,
    );
}

fn draw_application_card(
    index: usize,
    application: &TenantApplication,
    building: &Building,
    x: f32,
    y: f32,
    width: f32,
    assets: &AssetManager,
) -> Option<UiAction> {
    let card_h = 125.0;
    draw_rectangle(x, y, width, card_h, colors::PANEL_HEADER);
    draw_rectangle_lines(x, y, width, card_h, 1.0, colors::TEXT_DIM);

    let text_x = draw_application_portrait(application, assets, x, y);
    draw_application_text(application, building, text_x, y);

    let btn_y = y + 88.0;
    let btn_w = 70.0;
    if button(text_x, btn_y, btn_w, 28.0, "Accept", true) {
        return Some(UiAction::AcceptApplication {
            application_index: index,
        });
    }

    if button(text_x + 78.0, btn_y, btn_w, 28.0, "Reject", true) {
        return Some(UiAction::RejectApplication {
            application_index: index,
        });
    }

    if button(
        text_x + 156.0,
        btn_y,
        btn_w,
        28.0,
        "Credit",
        !application.revealed_reliability,
    ) {
        return Some(UiAction::CreditCheck {
            application_index: index,
        });
    }

    if button(
        text_x + 234.0,
        btn_y,
        95.0,
        28.0,
        "BG Check",
        !application.revealed_behavior,
    ) {
        return Some(UiAction::BackgroundCheck {
            application_index: index,
        });
    }

    None
}

fn draw_application_portrait(
    application: &TenantApplication,
    assets: &AssetManager,
    x: f32,
    y: f32,
) -> f32 {
    let portrait_id = format!(
        "tenant_{}",
        format!("{:?}", application.tenant.archetype).to_lowercase()
    );

    if let Some(texture) = assets.get_texture(&portrait_id) {
        draw_texture_ex(
            texture,
            x + 5.0,
            y + 5.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(80.0, 80.0)),
                ..Default::default()
            },
        );
        x + 95.0
    } else {
        x + 10.0
    }
}

fn draw_application_text(
    application: &TenantApplication,
    building: &Building,
    text_x: f32,
    y: f32,
) {
    draw_text(
        &application.tenant.name,
        text_x,
        y + 22.0,
        18.0,
        colors::TEXT,
    );
    draw_text(
        &format!("{:?}", application.tenant.archetype),
        text_x,
        y + 42.0,
        14.0,
        colors::TEXT_DIM,
    );

    if let Some(apartment) = building.get_apartment(application.apartment_id) {
        draw_text(
            &format!("-> Unit {}", apartment.unit_number),
            text_x + 140.0,
            y + 22.0,
            16.0,
            colors::ACCENT,
        );
    }

    let score_color = if application.match_result.score >= 70 {
        colors::POSITIVE
    } else if application.match_result.score >= 50 {
        colors::ACCENT
    } else {
        colors::WARNING
    };
    draw_text(
        &format!("Match: {}%", application.match_result.score),
        text_x + 140.0,
        y + 42.0,
        14.0,
        score_color,
    );

    let fit_text = if application.match_result.meets_minimum {
        "Fit: Qualified"
    } else {
        "Fit: Stretch"
    };
    draw_text(fit_text, text_x + 240.0, y + 42.0, 14.0, colors::TEXT_DIM);

    let credit_text = if application.revealed_reliability {
        format!("Credit: {}", application.tenant.rent_reliability)
    } else {
        "Credit: ?".to_string()
    };
    draw_text(&credit_text, text_x, y + 67.0, 14.0, colors::TEXT_DIM);

    let background_text = if application.revealed_behavior {
        format!("Behavior: {}", application.tenant.behavior_score)
    } else {
        "Behavior: ?".to_string()
    };
    draw_text(
        &background_text,
        text_x + 140.0,
        y + 67.0,
        14.0,
        colors::TEXT_DIM,
    );
}
