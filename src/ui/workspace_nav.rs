//! Persistent navigation between the game's management workspaces.

use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_surface, SurfaceStyle};

use super::theme::{color, space, Tone};
use super::widgets::button_at;
use super::UiAction;

pub const STATUS_BAR_HEIGHT: f32 = 64.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkspaceTab {
    Building,
    Tenants,
    Finances,
    City,
    Inbox,
    Tasks,
}

pub fn draw_workspace_nav(
    active: WorkspaceTab,
    unread_mail: usize,
    pending_tasks: usize,
) -> Option<UiAction> {
    let y = STATUS_BAR_HEIGHT;
    let h = (crate::ui::layout::HEADER_HEIGHT() - y).max(40.0);
    draw_surface(
        Rect::new(0.0, y, screen_width(), h),
        &SurfaceStyle::new(color::SURFACE_HEADER()).with_border(1.0, color::BORDER_STRONG()),
    );

    let labels = [
        (WorkspaceTab::Building, "Building", UiAction::OpenBuilding),
        (WorkspaceTab::Tenants, "Tenants", UiAction::OpenTenants),
        (WorkspaceTab::Finances, "Finances", UiAction::OpenFinances),
        (WorkspaceTab::City, "City", UiAction::OpenCityMap),
        (WorkspaceTab::Inbox, "Inbox", UiAction::OpenMail),
        (WorkspaceTab::Tasks, "Tasks", UiAction::OpenTasks),
    ];
    let gap = if screen_width() < 900.0 {
        space::XS
    } else {
        space::SM
    };
    let outer = if screen_width() < 900.0 {
        space::SM
    } else {
        space::LG
    };
    let available = screen_width() - outer * 2.0 - gap * (labels.len() as f32 - 1.0);
    let button_w = available / labels.len() as f32;
    let button_h = h - space::SM * 2.0;
    let mut x = outer;
    let mut action = None;

    for (tab, base_label, intent) in labels {
        let count = match tab {
            WorkspaceTab::Inbox => unread_mail,
            WorkspaceTab::Tasks => pending_tasks,
            _ => 0,
        };
        let base_label = if screen_width() < 700.0 {
            match tab {
                WorkspaceTab::Building => "Build",
                WorkspaceTab::Tenants => "People",
                WorkspaceTab::Finances => "Money",
                WorkspaceTab::City => "City",
                WorkspaceTab::Inbox => "Inbox",
                WorkspaceTab::Tasks => "Tasks",
            }
        } else {
            base_label
        };
        let label = if count > 0 {
            format!("{} ({})", base_label, count)
        } else {
            base_label.to_string()
        };
        let tone = if tab == active {
            Tone::Primary
        } else {
            Tone::Secondary
        };
        if button_at(
            Rect::new(x, y + space::SM, button_w, button_h),
            &label,
            true,
            tone,
        ) {
            action = Some(intent);
        }
        x += button_w + gap;
    }
    action
}
