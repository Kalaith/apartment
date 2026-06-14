use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, DesignType, NoiseLevel};
use macroquad::prelude::*;

use super::{common::*, UiAction};
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw_sold_condo_panel(
    apt: &Apartment,
    building: &Building,
    money: i32,
    panel_x: f32,
    panel_y: f32,
    panel_w: f32,
    panel_h: f32,
) -> Option<UiAction> {
    panel(
        panel_x,
        panel_y,
        panel_w,
        panel_h,
        &format!("Unit {} - SOLD", apt.unit_number),
    );

    let content_x = panel_x + 15.0;
    let mut y = panel_y + 60.0;

    draw_ui_text(
        "CONDO - PRIVATELY OWNED",
        content_x,
        y,
        20.0,
        colors::WARNING,
    );
    y += 35.0;

    if let Some((owner_name, purchase_price)) = building.get_condo_info(apt.id) {
        draw_ui_text(
            &format!("Owner: {}", owner_name),
            content_x,
            y,
            18.0,
            colors::TEXT,
        );
        y += 25.0;
        draw_ui_text(
            &format!("Purchased for: ${}", purchase_price),
            content_x,
            y,
            16.0,
            colors::TEXT_DIM,
        );
        y += 35.0;

        let buyback_price = (purchase_price as f32 * 1.1) as i32;
        draw_ui_text("Buyback Option:", content_x, y, 16.0, colors::ACCENT);
        y += 25.0;

        let can_afford = money >= buyback_price;
        let btn_label = format!("Buy Back (${})", buyback_price);

        if button(content_x, y, panel_w - 30.0, 35.0, &btn_label, can_afford) {
            return Some(UiAction::BuybackCondo {
                apartment_id: apt.id,
            });
        }
        y += 45.0;

        if !can_afford {
            draw_ui_text("Insufficient funds", content_x, y, 14.0, colors::NEGATIVE);
        }
    }

    y += 30.0;
    draw_ui_text(
        "You cannot modify or rent this unit",
        content_x,
        y,
        14.0,
        colors::TEXT_DIM,
    );
    y += 20.0;
    draw_ui_text(
        "while it is privately owned.",
        content_x,
        y,
        14.0,
        colors::TEXT_DIM,
    );

    None
}

pub(super) fn draw_apartment_stats(
    apt: &Apartment,
    assets: &AssetManager,
    content_x: f32,
    y: &mut f32,
    panel_w: f32,
    content_top: f32,
    content_bottom: f32,
) {
    if *y + 20.0 > content_top && *y < content_bottom {
        draw_ui_text("CONDITION", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 5.0;

    if *y + 20.0 > content_top && *y < content_bottom {
        let cond_color = condition_color(apt.condition);
        progress_bar(
            content_x,
            *y,
            panel_w - 30.0,
            20.0,
            apt.condition as f32,
            100.0,
            cond_color,
        );
        if let Some(icon) = if apt.condition > 50 {
            assets.get_texture("icon_condition_good")
        } else {
            assets.get_texture("icon_condition_poor")
        } {
            draw_texture_ex(
                icon,
                content_x + panel_w - 60.0,
                *y - 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                },
            );
        }
        draw_ui_text(
            &format!("{}%", apt.condition),
            content_x + panel_w - 110.0,
            *y + 15.0,
            16.0,
            colors::TEXT,
        );
    }
    *y += 35.0;

    if *y > content_top && *y < content_bottom {
        let design_text = match apt.design {
            DesignType::Bare => "Bare",
            DesignType::Practical => "Practical",
            DesignType::Cozy => "Cozy",
            DesignType::Luxury => "Luxury",
            DesignType::Opulent => "Opulent",
        };
        draw_ui_text(
            &format!("Design: {}", design_text),
            content_x,
            *y,
            18.0,
            colors::TEXT,
        );
    }
    *y += 25.0;

    if *y > content_top && *y < content_bottom {
        let size_text = match apt.size {
            ApartmentSize::Small => "Small",
            ApartmentSize::Medium => "Medium",
            ApartmentSize::Large => "Large",
            ApartmentSize::Penthouse => "Penthouse",
        };
        draw_ui_text(
            &format!("Size: {}", size_text),
            content_x,
            *y,
            18.0,
            colors::TEXT,
        );
    }
    *y += 25.0;

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
        draw_ui_text(
            &format!("Noise: {}", noise_text),
            content_x,
            *y,
            18.0,
            noise_color,
        );
        if let Some(icon) = assets.get_texture("icon_noise") {
            draw_texture_ex(
                icon,
                content_x + 120.0,
                *y - 15.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                },
            );
        }
    }
    *y += 25.0;

    if apt.has_soundproofing {
        if *y > content_top && *y < content_bottom {
            draw_ui_text("Soundproofed", content_x, *y, 16.0, colors::POSITIVE);
            if let Some(icon) = assets.get_texture("icon_soundproofing") {
                draw_texture_ex(
                    icon,
                    content_x + 110.0,
                    *y - 15.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(24.0, 24.0)),
                        ..Default::default()
                    },
                );
            }
        }
        *y += 25.0;
    }

    if *y > content_top && *y < content_bottom {
        draw_ui_text(
            &format!("Rent: ${}/mo", apt.rent_price),
            content_x,
            *y,
            20.0,
            colors::ACCENT,
        );
        if let Some(icon) = assets.get_texture("icon_rent") {
            draw_texture_ex(
                icon,
                content_x + 150.0,
                *y - 18.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                },
            );
        }
    }
    *y += 35.0;

    if *y > content_top && *y < content_bottom {
        let quality = apt.quality_score();
        draw_ui_text(
            &format!("Quality Score: {}", quality),
            content_x,
            *y,
            16.0,
            colors::TEXT_DIM,
        );
    }
    *y += 40.0;
}

pub(super) fn draw_upgrades(
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

    if *y > content_top && *y < content_bottom {
        draw_ui_text("UPGRADES", content_x, *y, 14.0, colors::TEXT_DIM);
    }
    *y += 25.0;

    let btn_w = panel_w - 30.0;
    let btn_h = 36.0;
    let available = crate::building::upgrades::available_apartment_upgrades(apt, &config.upgrades);

    let upgrades_start_y = *y;
    let mut total_upgrade_height = 0.0;

    for upgrade in &available {
        if upgrade
            .cost(building, &config.economy, &config.upgrades)
            .is_some()
        {
            total_upgrade_height += btn_h + 8.0;
        }
    }

    let max_scroll =
        (upgrades_start_y + total_upgrade_height - content_bottom + current_scroll).max(0.0);
    let final_scroll = current_scroll.min(max_scroll);
    let mut action = None;

    for upgrade in available {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!(
                "{} (${})",
                upgrade.label(building, &config.ui, &config.upgrades),
                cost
            );

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
            draw_ui_text(
                "▼ Scroll for more",
                content_x,
                scroll_hint_y,
                12.0,
                colors::TEXT_DIM,
            );
        }
    }

    (action, final_scroll)
}
