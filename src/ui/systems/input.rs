use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    ui::{ComputedNode, RelativeCursorPosition, ScrollPosition},
};

use crate::{
    gameplay::{
        exploration::{ActiveExplorationMap, ExplorationTravelRequest, NearbyExplorationPortal, PortalAction},
        zones::NearbyZone,
    },
    ui::{
        components::{ExplorationSetupViewModel, PopupConfirmRequested, VegetationLabRoot},
        stack::{UiLayer, UiStack},
    },
};

pub fn handle_ui_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nearby: Res<NearbyZone>,
    nearby_portal: Res<NearbyExplorationPortal>,
    mut travel_request: ResMut<ExplorationTravelRequest>,
    mut setup: ResMut<ExplorationSetupViewModel>,
    mut ui: ResMut<UiStack>,
    mut confirm_request: ResMut<PopupConfirmRequested>,
    active_map: Res<ActiveExplorationMap>,
) {
    if active_map.0.is_none() {
        ui.remove(UiLayer::VegetationLab);
    }

    let confirm = keys.just_pressed(KeyCode::KeyE) || confirm_request.0;
    confirm_request.0 = false;

    if keys.just_pressed(KeyCode::F4)
        && active_map.0.is_some()
        && !ui.paused()
        && !ui.game_menu_open()
        && !ui.exploration_setup_open()
        && !ui.inventory_open()
        && ui.active_panel().is_none()
    {
        if ui.contains(UiLayer::VegetationLab) {
            ui.remove(UiLayer::VegetationLab);
        } else {
            ui.push(UiLayer::VegetationLab);
        }
    }

    if confirm
        && !ui.game_menu_open()
        && !ui.paused()
        && ui.active_panel().is_none()
        && !ui.inventory_open()
        && !ui.exploration_setup_open()
    {
        if let Some(action) = nearby_portal.0 {
            match action {
                PortalAction::Enter(config) => {
                    setup.open(config);
                    ui.push(UiLayer::ExplorationSetup);
                }
                PortalAction::Return => {
                    travel_request.0 = Some(PortalAction::Return);
                }
            }
        } else if let Some(kind) = nearby.0 {
            ui.push(UiLayer::Panel(kind));
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        if ui.is_empty() {
            ui.push(UiLayer::GameMenu);
        } else {
            ui.pop_top();
        }
    }

    if mouse.just_pressed(MouseButton::Right) {
        if ui.is_empty() {
            ui.push(UiLayer::Inventory);
        } else {
            ui.pop_top();
        }
    }

    if keys.just_pressed(KeyCode::KeyP) && !ui.game_menu_open() && !ui.exploration_setup_open() {
        ui.toggle_pause();
    }

    if let Some(active) = ui.active_panel()
        && nearby.0 != Some(active) {
            ui.remove(UiLayer::Panel(active));
        }
}

/// Applies mouse-wheel input to the vegetation panel's clipped scroll region.
/// Bevy exposes ScrollPosition but leaves wheel routing to the application.
pub fn vegetation_lab_scroll(
    mut wheel_events: MessageReader<MouseWheel>,
    mut panels: Query<
        (
            &mut ScrollPosition,
            &ComputedNode,
            &RelativeCursorPosition,
            &InheritedVisibility,
        ),
        With<VegetationLabRoot>,
    >,
) {
    let events = wheel_events.read().copied().collect::<Vec<_>>();
    if events.is_empty() {
        return;
    }

    let Ok((mut scroll, computed, cursor, visibility)) = panels.single_mut() else {
        return;
    };
    if !visibility.get() || !cursor.cursor_over {
        return;
    }

    let max_scroll_y =
        ((computed.content_size.y - computed.size.y) * computed.inverse_scale_factor).max(0.0);
    for event in events {
        let delta = match event.unit {
            MouseScrollUnit::Line => event.y * 24.0,
            MouseScrollUnit::Pixel => event.y,
        };
        if delta.is_finite() {
            scroll.0.y = (scroll.0.y - delta).clamp(0.0, max_scroll_y);
        }
    }
}
