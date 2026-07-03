use super::{common::*, UiAction};
use crate::assets::AssetManager;
use crate::building::Building;
use crate::tenant::TenantApplication;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

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
        .filter(|(_, app)| filter_apartment_id.is_none_or(|id| app.apartment_id == id))
        .collect();

    if filtered_apps.is_empty() {
        draw_empty_applications(content_x, y, filter_apartment_id);
        return None;
    }

    draw_ui_text(
        &format!("{} pending", filtered_apps.len()),
        content_x,
        y,
        16.0,
        colors::TEXT_DIM,
    );
    y += 25.0;

    let mut action = None;
    for (index, application) in filtered_apps {
        if y > panel_rect.y + panel_rect.h - 60.0 {
            draw_ui_text(
                "... more applications",
                content_x,
                y,
                14.0,
                colors::TEXT_DIM,
            );
            break;
        }

        let (card_action, card_h) = draw_application_card(
            index,
            application,
            building,
            content_x,
            y,
            panel_rect.w - 30.0,
            assets,
        );
        if card_action.is_some() {
            action = card_action;
        }
        y += card_h + 12.0;
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
        draw_ui_text(
            "No applications for this unit",
            content_x,
            y,
            18.0,
            colors::TEXT_DIM,
        );
        return;
    }

    draw_ui_text(
        "No pending applications",
        content_x,
        y,
        18.0,
        colors::TEXT_DIM,
    );
    draw_ui_text(
        "List apartments for lease, then End Month!",
        content_x,
        y + 25.0,
        14.0,
        colors::TEXT_DIM,
    );
}

/// Draw one application card. Returns the chosen action (if any) and the card
/// height, which grows when the action buttons wrap to a second row on narrow
/// panels — so cards never overlap.
fn draw_application_card(
    index: usize,
    application: &TenantApplication,
    building: &Building,
    x: f32,
    y: f32,
    width: f32,
    assets: &AssetManager,
) -> (Option<UiAction>, f32) {
    use crate::ui::theme::Tone;
    use crate::ui::widgets::button_at;

    // Does a portrait exist? (Cheap check so we can lay out before drawing.)
    let portrait_id = format!(
        "tenant_{}",
        format!("{:?}", application.tenant.archetype).to_lowercase()
    );
    let has_portrait = assets.get_texture(&portrait_id).is_some();
    let text_x = if has_portrait { x + 95.0 } else { x + 12.0 };

    let btn_y = y + 88.0;
    let bh = 28.0;
    let gap = 6.0;
    let right = x + width - 8.0;

    // Adaptive grid: 4 across when there's room, otherwise 2x2.
    let cols = if right - text_x >= 4.0 * 74.0 + 3.0 * gap {
        4
    } else {
        2
    };
    let rows = 4_usize.div_ceil(cols);
    let bw = ((right - text_x) - (cols - 1) as f32 * gap) / cols as f32;
    let card_h = 88.0 + rows as f32 * (bh + gap) + 4.0;

    // Card frame (sized to fit the buttons), then portrait + content on top.
    crate::ui::widgets::draw_card(Rect::new(x, y, width, card_h), false);
    if let Some(texture) = assets.get_texture(&portrait_id) {
        draw_texture_ex(
            texture,
            x + 8.0,
            y + 8.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(78.0, 78.0)),
                ..Default::default()
            },
        );
    }
    draw_application_text(application, building, text_x, y);

    let specs: [(&str, bool, Tone, UiAction); 4] = [
        (
            "Accept",
            true,
            Tone::Positive,
            UiAction::AcceptApplication {
                application_index: index,
            },
        ),
        (
            "Reject",
            true,
            Tone::Danger,
            UiAction::RejectApplication {
                application_index: index,
            },
        ),
        (
            "Credit",
            !application.revealed_reliability,
            Tone::Secondary,
            UiAction::CreditCheck {
                application_index: index,
            },
        ),
        (
            "BG Check",
            !application.revealed_behavior,
            Tone::Secondary,
            UiAction::BackgroundCheck {
                application_index: index,
            },
        ),
    ];

    let mut action = None;
    for (i, (label, enabled, tone, act)) in specs.into_iter().enumerate() {
        let col = i % cols;
        let row = i / cols;
        let bx = text_x + col as f32 * (bw + gap);
        let by = btn_y + row as f32 * (bh + gap);
        if button_at(Rect::new(bx, by, bw, bh), label, enabled, tone) {
            action = Some(act);
        }
    }

    (action, card_h)
}

fn draw_application_text(
    application: &TenantApplication,
    building: &Building,
    text_x: f32,
    y: f32,
) {
    draw_ui_text(
        &application.tenant.name,
        text_x,
        y + 22.0,
        18.0,
        colors::TEXT,
    );
    draw_ui_text(
        &format!("{:?}", application.tenant.archetype),
        text_x,
        y + 42.0,
        14.0,
        colors::TEXT_DIM,
    );

    if let Some(apartment) = building.get_apartment(application.apartment_id) {
        draw_ui_text(
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
    draw_ui_text(
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
    draw_ui_text(fit_text, text_x + 240.0, y + 42.0, 14.0, colors::TEXT_DIM);

    let credit_text = if application.revealed_reliability {
        format!("Credit: {}", application.tenant.rent_reliability)
    } else {
        "Credit: ?".to_string()
    };
    draw_ui_text(&credit_text, text_x, y + 67.0, 14.0, colors::TEXT_DIM);

    let background_text = if application.revealed_behavior {
        format!("Behavior: {}", application.tenant.behavior_score)
    } else {
        "Behavior: ?".to_string()
    };
    draw_ui_text(
        &background_text,
        text_x + 140.0,
        y + 67.0,
        14.0,
        colors::TEXT_DIM,
    );
}
