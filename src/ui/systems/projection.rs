use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    gameplay::{
        player::Player,
        zones::{NearbyZone, TriggerZone, VisitedZones, ZoneKind},
    },
    ui::{
        components::{
            FastTravelButton, FastTravelMenu, InventoryMenuModal, MinimapZoneDot, PanelRoot,
            PauseOverlay, PopupRoot,
        },
        inventory::InventoryRoot,
        stack::UiStack,
    },
};

pub fn panel_visibility_system(ui: Res<UiStack>, mut panels: Query<(&PanelRoot, &mut Visibility)>) {
    let active = ui.active_panel();
    for (panel, mut visibility) in &mut panels {
        *visibility = if active == Some(panel.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn popup_visibility_system(
    nearby: Res<NearbyZone>,
    ui: Res<UiStack>,
    mut popups: Query<&mut Visibility, With<PopupRoot>>,
) {
    let visible =
        nearby.0.is_some() && ui.active_panel().is_none() && !ui.inventory_open() && !ui.paused();

    for mut visibility in &mut popups {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_menu_visibility_system(
    ui: Res<UiStack>,
    mut menus: Query<&mut Visibility, With<FastTravelMenu>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn fast_travel_button_visibility_system(
    ui: Res<UiStack>,
    visited: Res<VisitedZones>,
    mut buttons: Query<(&FastTravelButton, &mut Visibility)>,
) {
    for (button, mut visibility) in &mut buttons {
        *visibility = if ui.inventory_open() && visited.0.contains(&button.0) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn inventory_visibility_system(
    ui: Res<UiStack>,
    mut roots: Query<&mut Visibility, With<InventoryRoot>>,
) {
    for mut visibility in &mut roots {
        *visibility = if ui.inventory_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn inventory_menu_visibility_system(
    ui: Res<UiStack>,
    mut menus: Query<&mut Visibility, With<InventoryMenuModal>>,
) {
    for mut visibility in &mut menus {
        *visibility = if ui.inventory_menu_open() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn pause_overlay_visibility_system(
    ui: Res<UiStack>,
    mut overlays: Query<&mut Visibility, With<PauseOverlay>>,
) {
    for mut visibility in &mut overlays {
        *visibility = if ui.paused() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

pub fn sync_cursor(ui: Res<UiStack>, mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    let grabbed = ui.cursor_grabbed();
    cursor.visible = !grabbed;
    cursor.grab_mode = if grabbed {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
}

pub fn update_minimap(
    player: Query<&Transform, With<Player>>,
    zones: Query<(&Transform, &TriggerZone)>,
    mut dots: Query<(&MinimapZoneDot, &mut Node, &mut Visibility)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let player_position = player_transform.translation;
    let zone_positions: Vec<(ZoneKind, Vec3)> = zones
        .iter()
        .map(|(transform, zone)| (zone.kind, transform.translation))
        .collect();

    for (dot, mut node, mut visibility) in &mut dots {
        let Some((_, position)) = zone_positions.iter().find(|(kind, _)| *kind == dot.0) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        let relative = *position - player_position;
        let x = 60.0 + relative.x * 4.0 - 3.0;
        let y = 60.0 - relative.z * 4.0 - 3.0;

        if !(0.0..=114.0).contains(&x) || !(0.0..=114.0).contains(&y) {
            *visibility = Visibility::Hidden;
        } else {
            *visibility = Visibility::Visible;
            node.left = px(x);
            node.top = px(y);
        }
    }
}
