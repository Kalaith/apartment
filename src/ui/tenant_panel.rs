use crate::assets::AssetManager;
use crate::building::Apartment;
use crate::consequences::TenantNetwork;
use crate::narrative::{TenantRequest, TenantStory};
use crate::tenant::Tenant;
use macroquad::prelude::*;
use std::collections::HashMap;

use super::{common::*, UiAction};

pub(super) fn draw_tenant_info(
    apt: &Apartment,
    tenants: &[Tenant],
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    network: &TenantNetwork,
    stories: &HashMap<u32, TenantStory>,
) -> Option<UiAction> {
    if *y > content_top && *y < content_bottom {
        draw_line(
            content_x,
            *y,
            content_x + panel_w - 30.0,
            *y,
            1.0,
            colors::TEXT_DIM,
        );
    }
    *y += 15.0;

    if let Some(tenant_id) = apt.tenant_id {
        return draw_occupied_tenant_info(
            tenant_id,
            tenants,
            assets,
            content_x,
            y,
            panel_w,
            content_top,
            content_bottom,
            network,
            stories,
        );
    }

    draw_vacant_unit_actions(apt, content_x, y, panel_w, content_top, content_bottom)
}

fn draw_occupied_tenant_info(
    tenant_id: u32,
    tenants: &[Tenant],
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
    network: &TenantNetwork,
    stories: &HashMap<u32, TenantStory>,
) -> Option<UiAction> {
    let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) else {
        return None;
    };

    if *y > content_top && *y < content_bottom {
        draw_text("TENANT", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 20.0;

    let portrait_id = format!("tenant_{}", tenant.archetype.name().to_lowercase());
    let has_portrait = if let Some(tex) = assets.get_texture(&portrait_id) {
        if *y + 80.0 > content_top && *y < content_bottom {
            draw_texture_ex(
                tex,
                content_x,
                *y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(80.0, 80.0)),
                    ..Default::default()
                },
            );
        }
        true
    } else {
        false
    };

    let text_x = if has_portrait {
        content_x + 90.0
    } else {
        content_x + 10.0
    };

    if *y + 40.0 > content_top && *y < content_bottom {
        if !has_portrait {
            draw_rectangle(content_x, *y, 4.0, 20.0, archetype_color(&tenant.archetype));
        }

        draw_text(&tenant.name, text_x, *y + 16.0, 20.0, colors::TEXT);
        draw_text(
            tenant.archetype.name(),
            text_x,
            *y + 36.0,
            16.0,
            colors::TEXT_DIM,
        );

        draw_relationship_icons(tenant.id, network, text_x, *y + 45.0);

        if let Some(action) = draw_pending_request(
            tenant,
            stories,
            content_x,
            y,
            content_top,
            content_bottom,
        ) {
            return Some(action);
        }
    }

    if has_portrait {
        *y += 85.0;
    } else {
        *y += 75.0;
    }

    draw_tenant_happiness(tenant, assets, content_x, y, panel_w, content_top, content_bottom);

    if *y > content_top && *y < content_bottom {
        draw_text(
            &format!("Months: {}", tenant.months_residing),
            content_x,
            *y,
            14.0,
            colors::TEXT_DIM,
        );
    }
    *y += 30.0;

    None
}

fn draw_relationship_icons(tenant_id: u32, network: &TenantNetwork, text_x: f32, icon_y: f32) {
    let relationships: Vec<_> = network
        .relationships
        .iter()
        .filter(|rel| rel.tenant_a_id == tenant_id || rel.tenant_b_id == tenant_id)
        .collect();

    if relationships.is_empty() {
        return;
    }

    let mut icon_x = text_x;

    for rel in relationships.iter().take(4) {
        let icon = match rel.relationship_type {
            crate::consequences::RelationshipType::Friendly => "💚",
            crate::consequences::RelationshipType::Hostile => "⚡",
            crate::consequences::RelationshipType::Romantic => "💕",
            crate::consequences::RelationshipType::Family => "👨‍👩‍👧",
            crate::consequences::RelationshipType::Neutral => "⚪",
        };

        draw_text(icon, icon_x, icon_y + 15.0, 16.0, WHITE);
        icon_x += 25.0;
    }

    if relationships.len() > 4 {
        draw_text("+", icon_x, icon_y + 15.0, 14.0, colors::TEXT_DIM);
    }
}

fn draw_pending_request(
    tenant: &Tenant,
    stories: &HashMap<u32, TenantStory>,
    content_x: f32,
    y: &mut f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    let story = stories.get(&tenant.id)?;
    let request = story.pending_request.as_ref()?;

    *y += 40.0;
    if *y <= content_top || *y >= content_bottom {
        return None;
    }

    draw_text("PENDING REQUEST", content_x, *y, 14.0, colors::ACCENT);
    *y += 40.0;

    let req_text = request_text(request);
    draw_text(&req_text, content_x, *y, 16.0, colors::TEXT);
    *y += 25.0;

    let effect_text = approval_effect_text(request);
    if !effect_text.is_empty() {
        draw_text(
            &format!("Effect: {}", effect_text),
            content_x,
            *y,
            14.0,
            colors::ACCENT,
        );
        *y += 25.0;
    }

    if crate::ui::common::colored_button(
        content_x,
        *y,
        100.0,
        30.0,
        "APPROVE",
        true,
        colors::POSITIVE,
        colors::TEXT_BRIGHT,
    ) {
        return Some(UiAction::ApproveRequest {
            tenant_id: tenant.id,
        });
    }

    if crate::ui::common::colored_button(
        content_x + 110.0,
        *y,
        100.0,
        30.0,
        "DENY",
        true,
        colors::NEGATIVE,
        colors::TEXT_BRIGHT,
    ) {
        return Some(UiAction::DenyRequest {
            tenant_id: tenant.id,
        });
    }

    *y += 35.0;
    None
}

fn request_text(request: &TenantRequest) -> String {
    match request {
        TenantRequest::Pet { pet_type } => format!("Can I keep a {}?", pet_type),
        TenantRequest::TemporaryGuest {
            guest_name,
            duration_months,
        } => format!("Can {} stay for {} months?", guest_name, duration_months),
        TenantRequest::HomeBusiness { business_type } => {
            format!("Can I start a {} business?", business_type)
        }
        TenantRequest::Modification { description } => format!("Can I {}?", description),
        TenantRequest::Sublease => "Can I sublease a room?".to_string(),
    }
}

fn approval_effect_text(request: &TenantRequest) -> String {
    let effect = request.approval_effect();
    let mut effect_text = String::new();
    let mut stack = vec![effect];

    while let Some(effect) = stack.pop() {
        match effect {
            crate::narrative::StoryImpact::Happiness(amount) => {
                append_effect_text(&mut effect_text, &format!("Happiness {:+}", amount));
            }
            crate::narrative::StoryImpact::SetApartmentFlag(flag) => {
                if flag == "high_noise" {
                    append_effect_text(&mut effect_text, "Noise Increases");
                } else {
                    append_effect_text(&mut effect_text, &format!("Flag: {}", flag));
                }
            }
            crate::narrative::StoryImpact::Multiple(list) => {
                for item in list.iter().rev() {
                    stack.push(item.clone());
                }
            }
            _ => {}
        }
    }

    effect_text
}

fn append_effect_text(effect_text: &mut String, value: &str) {
    if !effect_text.is_empty() {
        effect_text.push_str(", ");
    }
    effect_text.push_str(value);
}

fn draw_tenant_happiness(
    tenant: &Tenant,
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) {
    if *y > content_top && *y < content_bottom {
        draw_text("Happiness", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 5.0;

    if *y + 16.0 > content_top && *y < content_bottom {
        let happy_color = happiness_color(tenant.happiness);
        progress_bar(
            content_x,
            *y,
            panel_w - 60.0,
            16.0,
            tenant.happiness as f32,
            100.0,
            happy_color,
        );

        let happiness_level = if tenant.happiness >= 90 {
            "happiness_ecstatic"
        } else if tenant.happiness >= 70 {
            "happiness_happy"
        } else if tenant.happiness >= 40 {
            "happiness_neutral"
        } else if tenant.happiness >= 20 {
            "happiness_unhappy"
        } else {
            "happiness_miserable"
        };

        if let Some(icon) = assets.get_texture(happiness_level) {
            draw_texture_ex(
                icon,
                content_x + panel_w - 55.0,
                *y - 4.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                },
            );
        } else {
            let icon_char = happiness_icon(tenant.happiness);
            draw_text(
                icon_char,
                content_x + panel_w - 50.0,
                *y + 14.0,
                20.0,
                colors::TEXT,
            );
        }
    }

    *y += 25.0;
}

fn draw_vacant_unit_actions(
    apt: &Apartment,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    if *y > content_top && *y < content_bottom {
        draw_text("VACANT", content_x, *y, 18.0, colors::WARNING);
    }
    *y += 25.0;

    let btn_w = panel_w - 30.0;

    if apt.is_listed_for_lease {
        return draw_listed_vacancy_actions(apt, content_x, y, btn_w, content_top, content_bottom);
    }

    draw_unlisted_vacancy_actions(apt, content_x, y, btn_w, content_top, content_bottom)
}

fn draw_listed_vacancy_actions(
    apt: &Apartment,
    content_x: f32,
    y: &mut f32,
    btn_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
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
            return Some(UiAction::UnlistApartment {
                apartment_id: apt.id,
            });
        }
    }
    *y += 40.0;

    None
}

fn draw_unlisted_vacancy_actions(
    apt: &Apartment,
    content_x: f32,
    y: &mut f32,
    btn_w: f32,
    content_top: f32,
    content_bottom: f32,
) -> Option<UiAction> {
    if *y > content_top && *y < content_bottom {
        draw_text("Status: OFF MARKET", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 30.0;

    if *y > content_top && *y < content_bottom {
        draw_text(
            &format!("Rent: ${}", apt.rent_price),
            content_x,
            *y,
            20.0,
            colors::TEXT,
        );

        let btn_size = 25.0;
        if button(content_x + 120.0, *y - 18.0, btn_size, btn_size, "-", true) {
            return Some(UiAction::AdjustRent {
                apartment_id: apt.id,
                amount: -50,
            });
        }
        if button(content_x + 150.0, *y - 18.0, btn_size, btn_size, "+", true) {
            return Some(UiAction::AdjustRent {
                apartment_id: apt.id,
                amount: 50,
            });
        }
    }
    *y += 40.0;

    if *y > content_top && *y < content_bottom {
        draw_text("List for Lease:", content_x, *y, 14.0, colors::ACCENT);
    }
    *y += 20.0;

    if *y + 30.0 > content_top && *y < content_bottom {
        if button(content_x, *y, btn_w, 30.0, "Any Tenant", true) {
            return Some(UiAction::ListApartment {
                apartment_id: apt.id,
                preference: None,
            });
        }
    }
    *y += 35.0;

    let tenant_types = [
        (crate::tenant::TenantArchetype::Student, "Student"),
        (crate::tenant::TenantArchetype::Professional, "Pro"),
        (crate::tenant::TenantArchetype::Artist, "Artist"),
        (crate::tenant::TenantArchetype::Family, "Family"),
        (crate::tenant::TenantArchetype::Elderly, "Elderly"),
    ];
    let small_btn_w = (btn_w - 10.0) / 2.0;

    for (index, (archetype, label)) in tenant_types.iter().enumerate() {
        let col = index % 2;
        let x = content_x + col as f32 * (small_btn_w + 10.0);

        if *y + 25.0 > content_top && *y < content_bottom {
            if button(x, *y, small_btn_w, 25.0, label, true) {
                return Some(UiAction::ListApartment {
                    apartment_id: apt.id,
                    preference: Some(archetype.clone()),
                });
            }
        }

        if col == 1 || index == tenant_types.len() - 1 {
            *y += 30.0;
        }
    }
    *y += 10.0;

    None
}
