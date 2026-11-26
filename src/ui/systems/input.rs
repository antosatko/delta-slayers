use bevy::prelude::*;

use crate::{
    gameplay::{
        exploration::{ExplorationTravelRequest, NearbyExplorationPortal, PortalAction},
        zones::NearbyZone,
        pickups::{NearbyPickup, PickupRequest},
    },
    ui::{
        components::{ExplorationSetupViewModel, PopupConfirmRequested},
        stack::{UiLayer, UiStack},
    },
};

pub fn handle_ui_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    nearby: Res<NearbyZone>,
    nearby_portal: Res<NearbyExplorationPortal>,
    nearby_pickup: Res<NearbyPickup>,
    mut pickup_request: ResMut<PickupRequest>,
    mut travel_request: ResMut<ExplorationTravelRequest>,
    mut setup: ResMut<ExplorationSetupViewModel>,
    mut ui: ResMut<UiStack>,
    mut confirm_request: ResMut<PopupConfirmRequested>,
) {
    let confirm = keys.just_pressed(KeyCode::KeyE) || confirm_request.0;
    confirm_request.0 = false;

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
        } else if let Some(entity) = nearby_pickup.0 {
            pickup_request.0 = Some(entity);
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
