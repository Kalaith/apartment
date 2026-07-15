use super::theme::{color, scale, space, Tone};
use super::widgets::button_at;
use super::{common::*, Selection, UiAction};
use crate::assets::AssetManager;
use crate::building::{Apartment, ApartmentSize, Building, DesignType, NoiseLevel};
use crate::tenant::Tenant;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

pub fn draw_building_view(
    building: &Building,
    tenants: &[Tenant],
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let mut action = None;

    let view_width = screen_width() * layout::PANEL_SPLIT();
    let view_height = screen_height() - layout::HEADER_HEIGHT() - layout::FOOTER_HEIGHT();
    let view_x = 0.0;
    let view_y = layout::HEADER_HEIGHT();

    // Background - Building Exterior
    if let Some(tex) = assets.get_texture("building_exterior") {
        draw_texture_ex(
            tex,
            view_x,
            view_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(view_width, view_height)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(view_x, view_y, view_width, view_height, color::BACKGROUND());
    }

    // Calculate layout - use max units per floor for total width
    let max_floor = building
        .apartments
        .iter()
        .map(|a| a.floor)
        .max()
        .unwrap_or(1);
    let max_units_per_floor = (1..=max_floor)
        .map(|f| building.apartments.iter().filter(|a| a.floor == f).count())
        .max()
        .unwrap_or(1);

    let total_width = max_units_per_floor as f32 * (layout::UNIT_WIDTH() + layout::UNIT_GAP());

    let center_x = view_x + view_width / 2.0;
    let start_x = view_x + (view_width - total_width) / 2.0;
    let start_y = view_y + view_height - 80.0; // Start from bottom

    // Draw floors (bottom to top)
    for floor in 1..=max_floor {
        let floor_y = start_y - (floor as f32 * layout::FLOOR_HEIGHT());

        // Floor label
        draw_ui_text(
            &format!("Floor {}", floor),
            start_x - 80.0,
            floor_y + layout::UNIT_HEIGHT() / 2.0,
            scale::LABEL,
            color::TEXT_DIM(),
        );

        // Draw units on this floor
        let floor_apartments: Vec<_> = building
            .apartments
            .iter()
            .filter(|a| a.floor == floor)
            .collect();

        // Calculate total floor width (accounting for penthouse double-width)
        let mut floor_total_width = 0.0;
        for apt in &floor_apartments {
            let unit_w = if matches!(apt.size, ApartmentSize::Penthouse) {
                (layout::UNIT_WIDTH() * 2.0) + layout::UNIT_GAP() // Double width
            } else {
                layout::UNIT_WIDTH()
            };
            floor_total_width += unit_w + layout::UNIT_GAP();
        }
        floor_total_width -= layout::UNIT_GAP(); // Remove trailing gap

        // Center this floor's units
        let floor_start_x = center_x - floor_total_width / 2.0;

        let mut current_x = floor_start_x;
        for apt in floor_apartments.iter() {
            let unit_w = if matches!(apt.size, ApartmentSize::Penthouse) {
                (layout::UNIT_WIDTH() * 2.0) + layout::UNIT_GAP()
            } else {
                layout::UNIT_WIDTH()
            };

            if let Some(apt_action) = draw_apartment_unit_sized(
                apt, tenants, current_x, floor_y, unit_w, selection, assets,
            ) {
                action = Some(apt_action);
            }

            current_x += unit_w + layout::UNIT_GAP();
        }
    }

    // Draw hallway at bottom
    let hallway_y = start_y + 20.0;
    let hallway_width = total_width - layout::UNIT_GAP();
    let hallway_h = 44.0;

    let hallway_selected = matches!(selection, Selection::Hallway);
    let hallway_hovered = is_hovered(start_x, hallway_y, hallway_width, hallway_h);

    let hallway_color = if hallway_selected {
        color::SELECTED()
    } else if hallway_hovered {
        color::HOVERED()
    } else {
        color::SURFACE_ALT()
    };

    // Use texture for hallway if available
    let drawn_texture = if let Some(tex) = assets.get_texture("hallway") {
        draw_texture_ex(
            tex,
            start_x,
            hallway_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(hallway_width, hallway_h)),
                ..Default::default()
            },
        );
        true
    } else {
        draw_rectangle(start_x, hallway_y, hallway_width, hallway_h, hallway_color);
        false
    };

    let hallway_border = if hallway_selected {
        color::PRIMARY()
    } else {
        color::BORDER()
    };
    draw_rectangle_lines(
        start_x,
        hallway_y,
        hallway_width,
        hallway_h,
        if hallway_selected || !drawn_texture {
            2.0
        } else {
            1.0
        },
        hallway_border,
    );

    // Hallway label and condition
    draw_ui_text(
        "HALLWAY",
        start_x + space::MD,
        hallway_y + hallway_h / 2.0 + scale::LABEL / 2.0,
        scale::LABEL,
        color::TEXT_BRIGHT(),
    );

    let cond_color = condition_color(building.hallway_condition);
    progress_bar(
        start_x + hallway_width - 110.0,
        hallway_y + (hallway_h - 14.0) / 2.0,
        100.0,
        14.0,
        building.hallway_condition as f32,
        100.0,
        cond_color,
    );

    if was_clicked(start_x, hallway_y, hallway_width, hallway_h) {
        action = Some(UiAction::SelectHallway);
    }

    // Top action buttons (clear of the header band).
    let btn_y = view_y + space::MD;
    let btn_h = 34.0;
    if button_at(
        Rect::new(start_x, btn_y, 140.0, btn_h),
        "Applications",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::SelectApplications(None));
    }
    if button_at(
        Rect::new(start_x + 150.0, btn_y, 130.0, btn_h),
        "Ownership",
        true,
        Tone::Secondary,
    ) {
        action = Some(UiAction::SelectOwnership);
    }

    action
}

fn draw_apartment_unit_sized(
    apt: &Apartment,
    tenants: &[Tenant],
    x: f32,
    y: f32,
    w: f32,
    selection: &Selection,
    assets: &AssetManager,
) -> Option<UiAction> {
    let h = layout::UNIT_HEIGHT();

    let is_selected = matches!(selection, Selection::Apartment(id) if *id == apt.id);
    let unit_hovered = is_hovered(x, y, w, h);

    // Background color (fallback when no design texture)
    let bg_color = if apt.is_vacant() {
        color::VACANT()
    } else {
        color::OCCUPIED()
    };

    // Draw Design Texture as background
    let design_id = match apt.design {
        DesignType::Bare => "design_bare",
        DesignType::Practical => "design_practical",
        DesignType::Cozy => "design_cozy",
        DesignType::Luxury => "design_luxury",
        DesignType::Opulent => "design_opulent",
    };

    if let Some(tex) = assets.get_texture(design_id) {
        draw_texture_ex(
            tex,
            x,
            y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(w, h)),
                ..Default::default()
            },
        );
    } else {
        draw_rectangle(x, y, w, h, bg_color);
    }

    // Selection / hover tint
    if is_selected {
        draw_rectangle(
            x,
            y,
            w,
            h,
            Color::new(
                color::PRIMARY().r,
                color::PRIMARY().g,
                color::PRIMARY().b,
                0.16,
            ),
        );
    } else if unit_hovered {
        draw_rectangle(x, y, w, h, Color::new(1.0, 1.0, 1.0, 0.08));
    }

    // Legibility strip behind the unit number / size.
    draw_rectangle(x, y, w, 22.0, Color::new(0.0, 0.0, 0.0, 0.45));

    // Border
    let (border_w, border_color) = if is_selected {
        (2.0, color::PRIMARY())
    } else if unit_hovered {
        (1.0, color::BORDER_STRONG())
    } else {
        (1.0, color::BORDER())
    };
    draw_rectangle_lines(x, y, w, h, border_w, border_color);

    // Unit number + size
    draw_ui_text(
        &apt.unit_number,
        x + space::SM,
        y + 16.0,
        scale::BODY,
        color::TEXT_BRIGHT(),
    );
    let size_text = match apt.size {
        ApartmentSize::Small => "S",
        ApartmentSize::Medium => "M",
        ApartmentSize::Large => "L",
        ApartmentSize::Penthouse => "PH",
    };
    let size_w = measure_ui_text(size_text, None, scale::LABEL as u16, 1.0).width;
    draw_ui_text(
        size_text,
        x + w - size_w - space::SM,
        y + 16.0,
        scale::LABEL,
        color::TEXT_DIM(),
    );

    // Condition meter
    let cond_color = condition_color(apt.condition);
    progress_bar(
        x + space::SM,
        y + 27.0,
        w - space::SM * 2.0,
        6.0,
        apt.condition as f32,
        100.0,
        cond_color,
    );

    // Noise indicator (if high)
    if matches!(apt.effective_noise(), NoiseLevel::High) {
        if let Some(icon) = assets.get_texture("icon_noise") {
            draw_texture_ex(
                icon,
                x + space::SM,
                y + 38.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(18.0, 18.0)),
                    ..Default::default()
                },
            );
        } else {
            draw_ui_text("!", x + space::SM, y + 50.0, scale::LABEL, color::WARNING());
        }
    }

    // Soundproofing indicator
    if apt.has_soundproofing {
        if let Some(icon) = assets.get_texture("icon_soundproofing") {
            draw_texture_ex(
                icon,
                x + 30.0,
                y + 38.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(18.0, 18.0)),
                    ..Default::default()
                },
            );
        } else {
            draw_ui_text("S", x + 30.0, y + 50.0, scale::LABEL, color::POSITIVE());
        }
    }

    // Low Condition Warning
    if apt.condition < 40 {
        draw_ui_text(
            "!",
            x + w - 16.0,
            y + 50.0,
            scale::HEADING,
            color::NEGATIVE(),
        );
    }

    // Tenant / vacant content
    if let Some(tenant_id) = apt.tenant_id {
        if let Some(tenant) = tenants.iter().find(|t| t.id == tenant_id) {
            let portrait_id = format!("tenant_{}", tenant.archetype.name().to_lowercase());
            if let Some(tex) = assets.get_texture(&portrait_id) {
                draw_texture_ex(
                    tex,
                    x + (w - 40.0) / 2.0,
                    y + 38.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(40.0, 40.0)),
                        ..Default::default()
                    },
                );
            } else {
                draw_rectangle(
                    x + space::SM,
                    y + h - 16.0,
                    3.0,
                    12.0,
                    archetype_color(&tenant.archetype),
                );
            }

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
                    x + w - 24.0,
                    y + h - 24.0,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(Vec2::new(20.0, 20.0)),
                        ..Default::default()
                    },
                );
            } else {
                // Colored happiness dot fallback.
                draw_circle(
                    x + w - 12.0,
                    y + h - 12.0,
                    6.0,
                    happiness_color(tenant.happiness),
                );
            }
        }
    } else {
        let window_tex = if matches!(apt.effective_noise(), NoiseLevel::High) {
            "window_street"
        } else {
            "window_quiet"
        };
        if let Some(tex) = assets.get_texture(window_tex) {
            draw_texture_ex(
                tex,
                x + (w - 40.0) / 2.0,
                y + 38.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(Vec2::new(40.0, 40.0)),
                    ..Default::default()
                },
            );
        }

        draw_ui_text(
            "VACANT",
            x + space::SM,
            y + h - 8.0,
            scale::CAPTION,
            color::TEXT_DIM(),
        );
        let rent = format!("${}", apt.rent_price);
        let rent_w = measure_ui_text(&rent, None, scale::CAPTION as u16, 1.0).width;
        draw_ui_text(
            &rent,
            x + w - rent_w - space::SM,
            y + h - 8.0,
            scale::CAPTION,
            color::PRIMARY(),
        );
    }

    // Handle click
    if was_clicked(x, y, w, h) {
        return Some(UiAction::SelectApartment(apt.id));
    }

    None
}
