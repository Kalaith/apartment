use super::theme::{color, scale, space, Tone};
use super::widgets::{button_at, button_width};
use super::{common::*, UiAction};
use crate::assets::AssetManager;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{
    draw_surface, draw_ui_text, measure_ui_text, truncate_text_to_width, SurfaceStyle,
};

/// Draw a stat chip (optional icon + label) at `x`, vertically centered in the
/// header. Returns the chip width so callers can flow chips without overlap.
fn stat_chip(
    x: f32,
    icon: Option<&Texture2D>,
    label: &str,
    text_color: Color,
    header_h: f32,
) -> f32 {
    let chip_h = 34.0;
    let chip_y = (header_h - chip_h) / 2.0;
    let icon_size = 20.0;
    let text_w = measure_ui_text(label, None, scale::BODY as u16, 1.0).width;
    let icon_w = if icon.is_some() {
        icon_size + space::XS
    } else {
        0.0
    };
    let w = space::MD + icon_w + text_w + space::MD;

    let style = SurfaceStyle::new(color::SURFACE_ALT).with_border(1.0, color::BORDER);
    draw_surface(Rect::new(x, chip_y, w, chip_h), &style);

    let mut cx = x + space::MD;
    if let Some(tex) = icon {
        draw_texture_ex(
            tex,
            cx,
            chip_y + (chip_h - icon_size) / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(icon_size, icon_size)),
                ..Default::default()
            },
        );
        cx += icon_size + space::XS;
    }
    draw_ui_text(
        label,
        cx,
        chip_y + chip_h / 2.0 + scale::BODY / 2.0 - 1.0,
        scale::BODY,
        text_color,
    );
    w
}

pub fn draw_header(
    money: i32,
    tick: u32,
    building_name: &str,
    occupancy: usize,
    total_units: usize,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;
    let w = screen_width();
    let h = layout::HEADER_HEIGHT;

    // Background + bottom hairline
    draw_rectangle(0.0, 0.0, w, h, color::SURFACE_HEADER);
    draw_line(0.0, h, w, h, 1.0, color::BORDER_STRONG);

    // End Month button, right-anchored, vertically centered.
    let btn_h = 40.0;
    let btn_w = button_width("End Month", btn_h).max(120.0);
    let btn_x = w - btn_w - space::LG;
    let btn_y = (h - btn_h) / 2.0;
    if button_at(
        Rect::new(btn_x, btn_y, btn_w, btn_h),
        "End Month",
        true,
        Tone::Primary,
    ) {
        action = Some(UiAction::EndTurn);
    }
    // Space hint just left of the button.
    let hint = "Space";
    let hint_w = measure_ui_text(hint, None, scale::CAPTION as u16, 1.0).width;
    let hint_x = btn_x - hint_w - space::MD;
    draw_ui_text(
        hint,
        hint_x,
        h / 2.0 + scale::CAPTION / 2.0,
        scale::CAPTION,
        color::TEXT_DIM,
    );

    // Stat cluster: money / month / occupancy chips, flowed right-to-left so
    // they hug the button and never collide with the building name.
    let money_color = if money < 0 {
        color::NEGATIVE
    } else if money < 500 {
        color::WARNING
    } else {
        color::POSITIVE
    };
    let money_label = macroquad_toolkit::ui::format_money(money as i64);
    let month_label = format!("Month {}", tick);
    let occ_label = format!("{}/{}", occupancy, total_units);

    // Measure chip widths (mirror stat_chip's math) to place them.
    let chip_gap = space::SM;
    let chips: [(Option<&Texture2D>, &str, Color); 3] = [
        (assets.get_texture("icon_money"), &money_label, money_color),
        (
            assets.get_texture("icon_calendar"),
            &month_label,
            color::TEXT,
        ),
        (assets.get_texture("icon_key"), &occ_label, color::TEXT),
    ];
    let widths: Vec<f32> = chips
        .iter()
        .map(|(icon, label, _)| {
            let text_w = measure_ui_text(label, None, scale::BODY as u16, 1.0).width;
            let icon_w = if icon.is_some() {
                20.0 + space::XS
            } else {
                0.0
            };
            space::MD + icon_w + text_w + space::MD
        })
        .collect();
    let cluster_w: f32 = widths.iter().sum::<f32>() + chip_gap * (chips.len() as f32 - 1.0);
    let cluster_right = hint_x - space::MD;
    let mut cx = (cluster_right - cluster_w).max(0.0);
    let cluster_left = cx;
    for (i, (icon, label, text_color)) in chips.iter().enumerate() {
        stat_chip(cx, *icon, label, *text_color, h);
        cx += widths[i] + chip_gap;
    }

    // Building name, left-aligned, ellipsized to the space before the cluster.
    let name_x = space::LG;
    let name_avail = (cluster_left - space::MD - name_x).max(40.0);
    let name = truncate_text_to_width(building_name, name_avail, scale::TITLE);
    draw_ui_text(
        &name,
        name_x,
        h / 2.0 + scale::TITLE / 2.0 - 1.0,
        scale::TITLE,
        color::TEXT_BRIGHT,
    );

    action
}
