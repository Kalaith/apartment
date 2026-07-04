use crate::narrative::events::NarrativeEvent;
use crate::ui::theme::{color, scale, space, Tone};
use crate::ui::widgets::{self, button_at, draw_panel, line_height, wrap};
use crate::ui::UiAction;
use macroquad::prelude::*;
use macroquad_toolkit::ui::draw_ui_text;

pub fn draw_event_modal(event: &NarrativeEvent) -> Option<UiAction> {
    let screen_w = screen_width();
    let screen_h = screen_height();

    draw_rectangle(0., 0., screen_w, screen_h, Color::new(0., 0., 0., 0.6));

    let modal_w = (screen_w * 0.55).clamp(480.0, 680.0);
    let content_w = modal_w - space::PAD * 2.0;

    let body_lines = wrap(&event.description, content_w, scale::BODY);
    let body_h = body_lines.len() as f32 * line_height(scale::BODY);

    let btn_h = 44.0;
    let btn_count = event.choices.len().max(1) as f32;
    let buttons_h = btn_count * btn_h + (btn_count - 1.0).max(0.0) * space::SM;

    let header_h = 38.0;
    let modal_h = header_h + space::SM + body_h + space::LG + buttons_h + space::MD;

    let x = (screen_w - modal_w) / 2.0;
    let y = ((screen_h - modal_h) / 2.0).max(space::XL);

    let content = draw_panel(Rect::new(x, y, modal_w, modal_h), &event.headline);

    let mut text_y = content.y;
    for line in &body_lines {
        draw_ui_text(
            line,
            content.x,
            text_y + scale::BODY,
            scale::BODY,
            color::TEXT,
        );
        text_y += line_height(scale::BODY);
    }

    let btn_x = content.x;
    let btn_w = content.w;

    if event.choices.is_empty() {
        let w = widgets::button_width("Continue", btn_h).max(120.0);
        let rect = Rect::new(btn_x + btn_w - w, y + modal_h - space::MD - btn_h, w, btn_h);
        if button_at(rect, "Continue", true, Tone::Primary) {
            return Some(UiAction::ResolveEventChoice {
                event_id: event.id,
                choice_index: 0,
            });
        }
        return None;
    }

    let mut btn_y = y + modal_h - space::MD - btn_h;
    for (i, choice) in event.choices.iter().enumerate().rev() {
        let label = if choice.reputation_change != 0 {
            format!("{}   (Rep {:+})", choice.label, choice.reputation_change)
        } else {
            choice.label.clone()
        };
        let tone = if choice.reputation_change > 0 {
            Tone::Positive
        } else if choice.reputation_change < 0 {
            Tone::Danger
        } else {
            Tone::Secondary
        };

        let rect = Rect::new(btn_x, btn_y, btn_w, btn_h);
        if button_at(rect, &label, true, tone) {
            return Some(UiAction::ResolveEventChoice {
                event_id: event.id,
                choice_index: i,
            });
        }
        btn_y -= btn_h + space::SM;
    }

    None
}
