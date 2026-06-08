use crate::assets::AssetManager;
use crate::city::{Neighborhood, NeighborhoodType, PropertyListing};
use crate::ui::colors;
use macroquad::prelude::*;

use super::city_view::CityMapAction;

pub(super) fn draw_listing_card(
    listing: &PropertyListing,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    neighborhoods: &[Neighborhood],
    player_funds: i32,
    assets: &AssetManager,
) -> Option<CityMapAction> {
    let mouse = mouse_position();
    let hovered = mouse.0 >= x && mouse.0 <= x + width && mouse.1 >= y && mouse.1 <= y + height;
    let neighborhood = neighborhoods
        .iter()
        .find(|n| n.id == listing.neighborhood_id);

    draw_listing_background(x, y, width, height, hovered, neighborhood);
    draw_neighborhood_preview(neighborhood, x, y, width, assets);
    draw_listing_text(listing, neighborhood, x, y);
    draw_listing_purchase(listing, x, y, width, height, player_funds)
}

pub(super) fn draw_progress_bar(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    progress: f32,
    color: Color,
) {
    macroquad_toolkit::ui::progress_bar(x, y, width, height, progress, 1.0, color);
}

pub(super) fn draw_button_icon(label: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
    let style = macroquad_toolkit::ui::ButtonStyle {
        normal: Color::from_rgba(50, 55, 65, 255),
        hovered: Color::from_rgba(70, 80, 100, 255),
        pressed: Color::from_rgba(42, 46, 56, 255),
        border: Color::from_rgba(80, 90, 110, 255),
        text_color: colors::TEXT_BRIGHT,
        disabled: Color::from_rgba(35, 35, 40, 255),
    };
    macroquad_toolkit::ui::button_rect_enabled_styled_ex(
        Rect::new(x, y, width, height),
        label,
        true,
        &style,
        macroquad_toolkit::ui::TextStyle::new(14.0, colors::TEXT_BRIGHT),
        macroquad_toolkit::ui::ButtonTrigger::Press,
    )
}

pub(super) fn draw_button_mini(label: &str, x: f32, y: f32, width: f32, height: f32) -> bool {
    let style = macroquad_toolkit::ui::ButtonStyle {
        normal: Color::from_rgba(60, 90, 120, 255),
        hovered: colors::ACCENT,
        pressed: Color::from_rgba(45, 70, 100, 255),
        border: colors::ACCENT,
        text_color: colors::TEXT_BRIGHT,
        disabled: Color::from_rgba(35, 35, 40, 255),
    };
    macroquad_toolkit::ui::button_rect_enabled_styled_ex(
        Rect::new(x, y, width, height),
        label,
        true,
        &style,
        macroquad_toolkit::ui::TextStyle::new(12.0, colors::TEXT_BRIGHT),
        macroquad_toolkit::ui::ButtonTrigger::Press,
    )
}

fn draw_listing_background(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    hovered: bool,
    neighborhood: Option<&Neighborhood>,
) {
    let bg_color = if hovered {
        Color::from_rgba(50, 55, 65, 255)
    } else {
        Color::from_rgba(35, 38, 45, 255)
    };
    let neighborhood_color = neighborhood
        .map(|n| n.neighborhood_type.color())
        .unwrap_or(Color::from_rgba(100, 100, 100, 255));

    draw_rectangle(x, y, width, height, bg_color);
    draw_rectangle(x, y, 5.0, height, neighborhood_color);
}

fn draw_neighborhood_preview(
    neighborhood: Option<&Neighborhood>,
    x: f32,
    y: f32,
    width: f32,
    assets: &AssetManager,
) {
    let Some(neighborhood) = neighborhood else {
        return;
    };

    let texture_id = match neighborhood.neighborhood_type {
        NeighborhoodType::Downtown => "neighborhood_downtown",
        NeighborhoodType::Industrial => "neighborhood_industrial",
        NeighborhoodType::Suburbs => "neighborhood_residential",
        NeighborhoodType::Historic => "neighborhood_university",
    };

    if let Some(texture) = assets.get_texture(texture_id) {
        draw_texture_ex(
            texture,
            x + width - 100.0,
            y + 10.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(90.0, 60.0)),
                ..Default::default()
            },
        );
    }
}

fn draw_listing_text(
    listing: &PropertyListing,
    neighborhood: Option<&Neighborhood>,
    x: f32,
    y: f32,
) {
    draw_text_ex(
        &listing.name,
        x + 15.0,
        y + 22.0,
        TextParams {
            font_size: 16,
            color: colors::TEXT_BRIGHT,
            ..Default::default()
        },
    );

    let location_name = neighborhood.map(|n| n.name.as_str()).unwrap_or("Unknown");
    draw_text_ex(
        location_name,
        x + 15.0,
        y + 40.0,
        text_params(13, colors::TEXT_DIM),
    );
    draw_text_ex(
        &format!(
            "{} floors, {} units | {} condition",
            listing.num_floors,
            listing.total_units(),
            listing.condition.name()
        ),
        x + 15.0,
        y + 58.0,
        text_params(12, colors::TEXT_DIM),
    );

    if listing.existing_tenants > 0 {
        draw_text_ex(
            &format!("{} existing tenants", listing.existing_tenants),
            x + 15.0,
            y + 73.0,
            text_params(11, colors::WARNING),
        );
    }
}

fn draw_listing_purchase(
    listing: &PropertyListing,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    player_funds: i32,
) -> Option<CityMapAction> {
    let can_afford = player_funds >= listing.asking_price;
    let btn_width = 80.0;
    let btn_x = x + width - btn_width - 10.0;
    let btn_y = y + height - 30.0;

    draw_text_ex(
        &format!("${}", listing.asking_price),
        x + 15.0,
        y + height - 12.0,
        text_params(18, price_color(listing, player_funds)),
    );

    if can_afford && draw_button_mini("Buy", btn_x, btn_y, btn_width, 22.0) {
        return Some(CityMapAction::PurchaseBuilding(listing.id));
    }

    if !can_afford {
        draw_text_ex(
            "Can't afford",
            btn_x,
            btn_y + 15.0,
            text_params(11, colors::TEXT_DIM),
        );
    }

    None
}

fn price_color(listing: &PropertyListing, player_funds: i32) -> Color {
    if player_funds >= listing.asking_price {
        colors::POSITIVE
    } else {
        colors::NEGATIVE
    }
}

fn text_params(font_size: u16, color: Color) -> TextParams<'static> {
    TextParams {
        font_size,
        color,
        ..Default::default()
    }
}
