use crate::assets::AssetManager;
use crate::building::Building;
use macroquad::prelude::*;

use super::{common::*, UiAction};
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_hallway_panel(
    building: &Building,
    money: i32,
    offset_x: f32,
    scroll_offset: f32,
    assets: &AssetManager,
    config: &crate::data::config::GameConfig,
) -> (Option<UiAction>, f32) {
    let mut action = None;
    let mut new_scroll = scroll_offset;

    let panel_x = screen_width() * layout::PANEL_SPLIT + layout::PADDING + offset_x;
    let panel_y = layout::HEADER_HEIGHT + layout::PADDING;
    let panel_w = screen_width() * (1.0 - layout::PANEL_SPLIT) - layout::PADDING * 2.0;

    if panel_x > screen_width() {
        return (None, scroll_offset);
    }

    let panel_h =
        screen_height() - layout::HEADER_HEIGHT - layout::FOOTER_HEIGHT - layout::PADDING * 2.0;

    panel(panel_x, panel_y, panel_w, panel_h, "Hallway");

    let mouse = mouse_position();
    let is_hovering = mouse.0 >= panel_x
        && mouse.0 <= panel_x + panel_w
        && mouse.1 >= panel_y
        && mouse.1 <= panel_y + panel_h;

    if is_hovering {
        let wheel = mouse_wheel();
        new_scroll -= wheel.1 * 30.0;
        new_scroll = new_scroll.max(0.0);
    }

    let content_x = panel_x + 15.0;
    let content_top = panel_y + 40.0;
    let content_bottom = panel_y + panel_h - 10.0;
    let mut y = panel_y + 50.0 - new_scroll;

    if y + 20.0 > content_top && y < content_bottom {
        draw_ui_text("CONDITION", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 5.0;

    if y + 20.0 > content_top && y < content_bottom {
        let cond_color = condition_color(building.hallway_condition);
        progress_bar(
            content_x,
            y,
            panel_w - 30.0,
            20.0,
            building.hallway_condition as f32,
            100.0,
            cond_color,
        );
        if let Some(icon) = if building.hallway_condition > 50 {
            assets.get_texture("icon_condition_good")
        } else {
            assets.get_texture("icon_condition_poor")
        } {
            draw_texture_ex(
                icon,
                content_x + panel_w - 60.0,
                y - 2.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(24.0, 24.0)),
                    ..Default::default()
                },
            );
        }
        draw_ui_text(
            &format!("{}%", building.hallway_condition),
            content_x + panel_w - 110.0,
            y + 15.0,
            16.0,
            colors::TEXT,
        );
    }
    y += 45.0;

    if y + 14.0 > content_top && y < content_bottom {
        draw_ui_text(
            "Affects overall building appeal",
            content_x,
            y,
            14.0,
            colors::TEXT_DIM,
        );
    }
    y += 20.0;

    if y + 18.0 > content_top && y < content_bottom {
        let appeal = building.building_appeal();
        draw_ui_text(
            &format!("Building Appeal: {}", appeal),
            content_x,
            y,
            18.0,
            colors::ACCENT,
        );
    }
    y += 50.0;

    if y + 14.0 > content_top && y < content_bottom {
        draw_ui_text("STAFF", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 25.0;

    let mut staff_count = 0;
    let mut staff_types: Vec<_> = config.economy.staff_costs.keys().collect();
    staff_types.sort();

    for staff_type in staff_types {
        let Some(cost) = config.economy.staff_costs.get(staff_type) else {
            continue;
        };
        let flag = format!("staff_{}", staff_type);

        if building.flags.contains(&flag) {
            let mut chars = staff_type.chars();
            let label = match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            };

            if y + 16.0 > content_top && y < content_bottom {
                draw_ui_text(
                    &format!("{} (${}/mo)", label, cost),
                    content_x,
                    y,
                    16.0,
                    colors::TEXT,
                );
            }
            y += 25.0;
            staff_count += 1;
        }
    }

    if staff_count == 0 {
        if y + 16.0 > content_top && y < content_bottom {
            draw_ui_text("None hired", content_x, y, 16.0, colors::TEXT_DIM);
        }
        y += 25.0;
    }

    y += 25.0;

    let available =
        crate::building::upgrades::available_building_upgrades(building, &config.upgrades);

    let mut staff_actions = Vec::new();
    let mut other_actions = Vec::new();

    for upgrade in available {
        let is_staff = match &upgrade {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => {
                if let Some(def) = config.upgrades.get(upgrade_id) {
                    def.effects.iter().any(|effect| match effect {
                        crate::data::config::UpgradeEffect::SetFlag(flag)
                        | crate::data::config::UpgradeEffect::RemoveFlag(flag) => {
                            flag.starts_with("staff_")
                        }
                        _ => false,
                    })
                } else {
                    false
                }
            }
            _ => false,
        };

        if is_staff {
            staff_actions.push(upgrade);
        } else {
            other_actions.push(upgrade);
        }
    }

    staff_actions.sort_by(|a, b| {
        let a_id = match a {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id,
            _ => "",
        };
        let b_id = match b {
            crate::building::upgrades::UpgradeAction::Apply { upgrade_id, .. } => upgrade_id,
            _ => "",
        };

        let a_is_fire = a_id.contains("fire");
        let b_is_fire = b_id.contains("fire");

        if a_is_fire && !b_is_fire {
            std::cmp::Ordering::Greater
        } else if !a_is_fire && b_is_fire {
            std::cmp::Ordering::Less
        } else {
            a_id.cmp(b_id)
        }
    });

    let btn_w = panel_w - 30.0;

    for upgrade in staff_actions {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!(
                "{} (${})",
                upgrade.label(building, &config.ui, &config.upgrades),
                cost
            );

            if y + 36.0 > content_top && y < content_bottom {
                if button(content_x, y, btn_w, 36.0, &label, can_afford) {
                    action = Some(UiAction::UpgradeAction(upgrade));
                }
            }
            y += 44.0;
        }
    }

    if y + 14.0 > content_top && y < content_bottom {
        draw_ui_text("UPGRADES", content_x, y, 14.0, colors::TEXT_DIM);
    }
    y += 25.0;

    for upgrade in other_actions {
        if let Some(cost) = upgrade.cost(building, &config.economy, &config.upgrades) {
            let can_afford = money >= cost;
            let label = format!(
                "{} (${})",
                upgrade.label(building, &config.ui, &config.upgrades),
                cost
            );

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
