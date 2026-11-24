use bevy::prelude::*;

use crate::{
    gameplay::zones::NearbyZone,
    ui::{
        components::PopupConfirmRequested,
        stack::{UiLayer, UiStack},
    },
};

pub fn handle_ui_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nearby: Res<NearbyZone>,
    mut ui: ResMut<UiStack>,
    mut confirm_request: ResMut<PopupConfirmRequested>,
) {
    let confirm = keys.just_pressed(KeyCode::KeyE) || confirm_request.0;
    confirm_request.0 = false;

    if confirm && !ui.game_menu_open() && ui.active_panel().is_none() && !ui.inventory_open() {
        if let Some(kind) = nearby.0 {
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
            if ui.cursor_capture_requested() {
                ui.push(UiLayer::Inventory);
            } else {
                ui.capture_cursor();
            }
        } else {
            ui.pop_top();
        }
    }

    if keys.just_pressed(KeyCode::KeyP) && !ui.game_menu_open() {
        ui.toggle_pause();
    }

    if let Some(active) = ui.active_panel() {
        if nearby.0 != Some(active) {
            ui.remove(UiLayer::Panel(active));
            if ui.is_empty() {
                ui.capture_cursor();
            }
        }
    }
}
